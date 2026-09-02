use crate::block_definitions::*;
use crate::element_processing::building_intelligence::circulation::DoorOrientation;
use crate::element_processing::building_intelligence::room_graph::RoomConnectionKind;
use crate::element_processing::building_intelligence::vertical::{
    render_vertical_access, VerticalAccessEditor,
};
use crate::element_processing::building_intelligence::FurnitureKind;
use crate::element_processing::building_intelligence::room_loot::generate_room_loot;
use crate::element_processing::building_intelligence::PlannedBuilding;
use crate::element_processing::subprocessor::buildings_interior::generate_building_interior;
use crate::element_processing::subprocessor::interior::{FloorPlan, RoomType};
use crate::floodfill_cache::CoordinateBitmap;
use crate::world_editor::WorldEditor;
use fastnbt::Value;

/// Build a real Minecraft door block with explicit Java block-state properties.
///
/// The semantic doorway plan remains the source of truth for geometry.
/// This helper only materializes the already-approved doorway as a two-block
/// Minecraft door and never changes room geometry, BBox, or doorway width.
fn interior_door_block_with_state(
    orientation: &str,
    upper: bool,
    hinge_left: bool,
    open: bool,
) -> BlockWithProperties {
    let block = OAK_DOOR;

    let mut props = std::collections::HashMap::<String, Value>::new();
    props.insert("facing".to_string(), Value::String(orientation.to_string()));
    props.insert(
        "half".to_string(),
        Value::String(if upper { "upper" } else { "lower" }.to_string()),
    );
    props.insert(
        "hinge".to_string(),
        Value::String(if hinge_left { "left" } else { "right" }.to_string()),
    );
    props.insert(
        "open".to_string(),
        Value::String(if open { "true" } else { "false" }.to_string()),
    );

    BlockWithProperties::new(block, Some(Value::Compound(props)))
}

struct WorldEditorVerticalAdapter<'a, 'b> {
    editor: &'a mut WorldEditor<'b>,
}

impl<'a, 'b> VerticalAccessEditor for WorldEditorVerticalAdapter<'a, 'b> {
    fn place_block(&mut self, block: BlockWithProperties, x: i32, y: i32, z: i32) {
        self.editor
            .set_block_with_properties_absolute(block, x, y, z, None, None);
    }

    fn clear_block(&mut self, x: i32, y: i32, z: i32) {
        self.editor
            .set_block_absolute(AIR, x, y, z, None, Some(&[]));
    }

    fn block_at(&self, x: i32, y: i32, z: i32) -> Option<Block> {
        self.editor.get_block_absolute(x, y, z)
    }
}

/// ---------------------------------------------------------
/// ⑭ CACHE CONSUMPTION
/// ---------------------------------------------------------
/// Existing cached floor-area data is authoritative for the
/// already reconstructed building geometry.
///
/// Cache is read-only intelligence:
/// - never modifies FloorPlan
/// - never modifies exterior geometry
/// - never modifies BBox
/// - never regenerates real-world geometry
/// Convert the existing cached floor bitmap into the coordinate
/// representation required by the legacy interior renderer.
///
/// This function only reads the already-existing building footprint.
/// It never expands the footprint, changes BBox coordinates, performs
/// new flood-fill operations, or creates geographic geometry.
fn cached_floor_area_to_vec(
    cached_floor_area: &[(i32, i32)],
    min_x: i32,
    min_z: i32,
    max_x: i32,
    max_z: i32,
) -> Vec<(i32, i32)> {
    let mut floor_area = Vec::new();

    for z in min_z..=max_z {
        for x in min_x..=max_x {
            if cached_floor_area.contains(&(x, z)) {
                floor_area.push((x, z));
            }
        }
    }

    floor_area
}

/// Map semantic furniture into deterministic Minecraft block layouts.
///
/// This function only returns block placements relative to the furniture
/// origin. It does not modify the building footprint or geographic bounds.
fn furniture_blocks(kind: FurnitureKind) -> &'static [(Block, i32, i32, i32)] {
    match kind {
        // 2 x 3 semantic footprint.
        FurnitureKind::Bed => &[
            (RED_BED_NORTH_HEAD, 0, 0, 0),
            (RED_BED_NORTH_FOOT, 1, 0, 0),
            (RED_BED_NORTH_HEAD, 0, 0, 1),
            (RED_BED_NORTH_FOOT, 1, 0, 1),
            (RED_BED_NORTH_HEAD, 0, 0, 2),
            (RED_BED_NORTH_FOOT, 1, 0, 2),
        ],

        // 2 x 1.
        FurnitureKind::Sofa => &[(OAK_STAIRS, 0, 0, 0), (OAK_STAIRS, 1, 0, 0)],

        // 2 x 2.
        FurnitureKind::Table => &[
            (OAK_FENCE, 0, 0, 0),
            (OAK_FENCE, 1, 0, 0),
            (OAK_SLAB_TOP, 0, 1, 0),
            (OAK_SLAB_TOP, 1, 1, 0),
        ],

        // 1 x 1.
        FurnitureKind::Chair => &[(OAK_STAIRS, 0, 0, 0)],

        // 2 x 1.
        FurnitureKind::KitchenCounter => &[(CRAFTING_TABLE, 0, 0, 0), (CRAFTING_TABLE, 1, 0, 0)],

        // 1 x 1.
        FurnitureKind::Sink => &[(CAULDRON, 0, 0, 0)],

        // 1 x 1.
        FurnitureKind::Toilet => &[(CAULDRON, 0, 0, 0)],

        // 1 x 1.
        FurnitureKind::Shower => &[(CAULDRON, 0, 0, 0)],

        // 2 x 1.
        FurnitureKind::Bathtub => &[(CAULDRON, 0, 0, 0), (CAULDRON, 1, 0, 0)],

        // 2 x 1.
        FurnitureKind::Desk => &[
            (OAK_FENCE, 0, 0, 0),
            (OAK_FENCE, 1, 0, 0),
            (OAK_SLAB_TOP, 0, 1, 0),
            (OAK_SLAB_TOP, 1, 1, 0),
        ],

        // 2 x 1.
        FurnitureKind::Bookshelf => &[
            (BOOKSHELF, 0, 0, 0),
            (BOOKSHELF, 1, 0, 0),
            (BOOKSHELF, 0, 1, 0),
            (BOOKSHELF, 1, 1, 0),
        ],

        // 2 x 1.
        FurnitureKind::Checkout => &[
            (CRAFTING_TABLE, 0, 0, 0),
            (CRAFTING_TABLE, 1, 0, 0),
            (OAK_SLAB_TOP, 0, 1, 0),
            (OAK_SLAB_TOP, 1, 1, 0),
        ],

        // 2 x 1.
        FurnitureKind::Shelf => &[
            (OAK_FENCE, 0, 0, 0),
            (OAK_FENCE, 1, 0, 0),
            (OAK_SLAB_TOP, 0, 1, 0),
            (OAK_SLAB_TOP, 1, 1, 0),
        ],

        // 2 x 3 semantic footprint.
        FurnitureKind::HospitalBed => &[
            (RED_BED_NORTH_HEAD, 0, 0, 0),
            (RED_BED_NORTH_FOOT, 1, 0, 0),
            (RED_BED_NORTH_HEAD, 0, 0, 1),
            (RED_BED_NORTH_FOOT, 1, 0, 1),
            (RED_BED_NORTH_HEAD, 0, 0, 2),
            (RED_BED_NORTH_FOOT, 1, 0, 2),
        ],

        // 2 x 1.
        FurnitureKind::MedicalDesk => &[
            (OAK_FENCE, 0, 0, 0),
            (OAK_FENCE, 1, 0, 0),
            (OAK_SLAB_TOP, 0, 1, 0),
            (OAK_SLAB_TOP, 1, 1, 0),
        ],

        // 2 x 1.
        FurnitureKind::ClassroomDesk => &[
            (OAK_FENCE, 0, 0, 0),
            (OAK_FENCE, 1, 0, 0),
            (OAK_SLAB_TOP, 0, 1, 0),
            (OAK_SLAB_TOP, 1, 1, 0),
        ],

        // 1 x 1.
        FurnitureKind::StorageShelf => &[(BARREL, 0, 0, 0), (BARREL, 0, 1, 0)],

        // 3 x 2.
        FurnitureKind::DiningTable => &[
            (OAK_FENCE, 0, 0, 0),
            (OAK_FENCE, 1, 0, 0),
            (OAK_FENCE, 2, 0, 0),
            (OAK_FENCE, 0, 0, 1),
            (OAK_FENCE, 1, 0, 1),
            (OAK_FENCE, 2, 0, 1),
            (OAK_SLAB_TOP, 0, 1, 0),
            (OAK_SLAB_TOP, 1, 1, 0),
            (OAK_SLAB_TOP, 2, 1, 0),
            (OAK_SLAB_TOP, 0, 1, 1),
            (OAK_SLAB_TOP, 1, 1, 1),
            (OAK_SLAB_TOP, 2, 1, 1),
        ],
    }
}

/// Render the result of Building Intelligence into Minecraft.
///
/// IMPORTANT:
/// - Does not modify BBox.
/// - Does not modify geographic coordinates.
/// - Does not expand the real-world building footprint.
/// - Uses the existing cached floor area as the hard geometry boundary.
/// - Falls back to the established interior renderer when validation fails.
#[allow(clippy::too_many_arguments)]
pub fn generate_intelligent_building_interior(
    editor: &mut WorldEditor,
    planned_building: &PlannedBuilding,
    cached_floor_area: &[(i32, i32)],
    min_x: i32,
    min_z: i32,
    max_x: i32,
    max_z: i32,
    start_y_offset: i32,
    effective_building_height: i32,
    wall_block: Block,
    floor_levels: &[i32],
    abs_terrain_offset: i32,
    is_abandoned_building: bool,
    effective_passages: &CoordinateBitmap,
    has_sloped_roof: bool,
) {
    // ---------------------------------------------------------
    // Safety boundary
    // ---------------------------------------------------------
    println!(
        "[BI RENDER DEBUG] context=({}, {})-({}, {}), renderer=({}, {})-({}, {})",
        planned_building.context.min_x,
        planned_building.context.min_z,
        planned_building.context.max_x,
        planned_building.context.max_z,
        min_x,
        min_z,
        max_x,
        max_z,
    );
    //
    // Building Intelligence must describe exactly the same
    // real-world building footprint that the existing generation
    // layer supplied.
    //
    if planned_building.context.min_x != min_x
        || planned_building.context.min_z != min_z
        || planned_building.context.max_x != max_x
        || planned_building.context.max_z != max_z
    {
        generate_building_interior(
            editor,
            cached_floor_area,
            min_x,
            min_z,
            max_x,
            max_z,
            start_y_offset,
            effective_building_height,
            wall_block,
            floor_levels,
            abs_terrain_offset,
            is_abandoned_building,
            &effective_passages,
            has_sloped_roof,
        );

        return;
    }

    // ---------------------------------------------------------
    // Plan validation
    // ---------------------------------------------------------

    if planned_building.floor_plans.is_empty() {
        println!("[BI RENDER] NO FLOOR PLANS -> legacy interior fallback");

        generate_building_interior(
            editor,
            cached_floor_area,
            min_x,
            min_z,
            max_x,
            max_z,
            start_y_offset,
            effective_building_height,
            wall_block,
            floor_levels,
            abs_terrain_offset,
            is_abandoned_building,
            &effective_passages,
            has_sloped_roof,
        );
        return;
    }

    if planned_building.room_graph.rooms.is_empty() {
        println!("[BI RENDER] FLOOR PLANS EXIST BUT ROOM GRAPH EMPTY -> rendering rooms anyway");
    }
    // ---------------------------------------------------------
    // Semantic graph validation
    // ---------------------------------------------------------
    //
    // Room geometry comes from FloorPlan and must not depend on
    // RoomGraph being populated. RoomGraph only controls topology.
    //
    if !planned_building.room_graph.rooms.is_empty() {
        let graph_valid = planned_building
            .room_graph
            .connections
            .iter()
            .all(|connection| {
                connection.from < planned_building.room_graph.rooms.len()
                    && connection.to < planned_building.room_graph.rooms.len()
            });

        if !graph_valid {
            println!(
                "[BI RENDER] INVALID ROOM GRAPH -> topology disabled,                  continuing with FloorPlan room rendering"
            );
        }
    } else {
        println!(
            "[BI RENDER] NO ROOM GRAPH -> topology disabled,              continuing with FloorPlan room rendering"
        );
    }

    // ---------------------------------------------------------
    // Intelligent physical rendering
    // ---------------------------------------------------------
    println!(
        "[BI RENDER DEBUG] ENTERING ROOM RENDER: floors={} rooms={}",
        planned_building.floor_plans.len(),
        planned_building
            .floor_plans
            .iter()
            .map(|p| p.rooms.len())
            .sum::<usize>(),
    );
    //
    // Building Intelligence is now the semantic source of truth
    // for the interior layout.
    //
    // The existing building footprint remains the hard geometry
    // boundary. Room plans are rendered only inside that boundary.
    //
    // Legacy generation remains available through the validation
    // fallbacks above.

    // ---------------------------------------------------------
    // FLOOR PLAN -> PHYSICAL ROOM MATERIALIZATION
    // ---------------------------------------------------------
    //
    // FloorPlan owns room geometry.
    // RoomGraph owns topology only.
    //
    // Every planned room is physically materialized here.
    //

    println!(
        "[BI ROOM] MATERIALIZATION START floors={} rooms={}",
        planned_building.floor_plans.len(),
        planned_building
            .floor_plans
            .iter()
            .map(|p| p.rooms.len())
            .sum::<usize>()
    );

    // ---------------------------------------------------------
    // Strict real-building interior boundary
    // ---------------------------------------------------------
    //
    // Room geometry is semantic planning data.
    //
    // The cached floor area is the authoritative physical
    // interior boundary of the already reconstructed building.
    //
    // IMPORTANT:
    // - Room bounds NEVER expand the building.
    // - BBox is NOT used as the room interior boundary.
    // - No room block may be materialized outside cached_floor_area.
    // - Existing exterior geometry remains authoritative.
    //
    // Build a fast membership set once instead of repeatedly
    // scanning the cached floor-area vector.
    let cached_floor_area_set: std::collections::HashSet<(i32, i32)> =
        cached_floor_area.iter().copied().collect();

    println!(
        "[BI ROOM] STRICT INTERIOR AREA cells={}",
        cached_floor_area_set.len()
    );

    for (floor_index, floor_plan) in planned_building.floor_plans.iter().enumerate() {
        let floor_y = floor_levels
            .get(floor_index)
            .copied()
            .unwrap_or(start_y_offset + (floor_index as i32 * 4));

        for (room_index, room) in floor_plan.rooms.iter().enumerate() {
            // -------------------------------------------------
            // Room bounds are planning geometry only.
            //
            // First intersect them with the building renderer
            // bounds, then every individual cell is additionally
            // validated against the authoritative cached floor
            // area below.
            // -------------------------------------------------
            let room_min_x = room.bounds.min_x.max(min_x);
            let room_max_x = room.bounds.max_x.min(max_x);
            let room_min_z = room.bounds.min_z.max(min_z);
            let room_max_z = room.bounds.max_z.min(max_z);

            if room_min_x > room_max_x || room_min_z > room_max_z {
                println!(
                    "[BI ROOM] SKIP floor={} room={} invalid_bounds",
                    floor_index, room_index
                );
                continue;
            }

            // -------------------------------------------------
            // Calculate the actual physical cells belonging to
            // BOTH:
            //
            //   1. semantic room bounds
            //   2. real reconstructed building floor area
            //
            // This is the only area in which this room may exist.
            // -------------------------------------------------
            let mut interior_cells = Vec::new();

            for z in room_min_z..=room_max_z {
                for x in room_min_x..=room_max_x {
                    if cached_floor_area_set.contains(&(x, z)) {
                        interior_cells.push((x, z));
                    }
                }
            }

            if interior_cells.is_empty() {
                println!(
                    "[BI ROOM] SKIP floor={} room={} outside_real_building",
                    floor_index, room_index
                );
                continue;
            }

            println!(
                "[BI ROOM] MATERIALIZE floor={} room={} type={:?} \
bounds=({},{})-({},{}) valid_cells={} y={}",
                floor_index,
                room_index,
                room.room_type,
                room_min_x,
                room_min_z,
                room_max_x,
                room_max_z,
                interior_cells.len(),
                floor_y
            );
            // -------------------------------------------------
            // Interior clearance
            // -------------------------------------------------
            //
            // HARD PHYSICAL RULE:
            //
            // The Room interior volume must remain AIR.
            //
            // Room floor:
            //     floor_y
            //
            // Room interior:
            //     floor_y + 1
            //     floor_y + 2
            //     floor_y + 3
            //
            // Top physical floor ceiling:
            //     floor_y + 4
            //
            // Only cells already belonging to the authoritative
            // cached_floor_area may be cleared.
            // -------------------------------------------------
            for &(x, z) in &interior_cells {
                if !cached_floor_area_set.contains(&(x, z)) {
                    continue;
                }

                editor.set_block_absolute(AIR, x, floor_y + 1, z, None, Some(&[]));

                editor.set_block_absolute(AIR, x, floor_y + 2, z, None, Some(&[]));

                editor.set_block_absolute(AIR, x, floor_y + 3, z, None, Some(&[]));
            }

            // -------------------------------------------------
            // Room floor
            // -------------------------------------------------
            //
            // HARD PHYSICAL RULE:
            //
            // RoomPlan is semantic planning data.
            // cached_floor_area is the authoritative physical
            // interior footprint.
            //
            // A room may NEVER create floor blocks outside
            // the already reconstructed building interior.
            //
            // -------------------------------------------------
            for &(x, z) in &interior_cells {
                if !cached_floor_area_set.contains(&(x, z)) {
                    continue;
                }

                editor.set_block_absolute(wall_block, x, floor_y, z, None, None);
            } // -------------------------------------------------
              // Room walls
              // -------------------------------------------------
              //
              // A wall is generated only when:
              //
              //   - the cell belongs to the real building floor area
              //   - the cell lies on the semantic room boundary
              //
              // This prevents a room rectangle from becoming an
              // exterior structure outside the actual building.
              // -------------------------------------------------
            let room_cell_set: std::collections::HashSet<(i32, i32)> =
                interior_cells.iter().copied().collect();

            for &(x, z) in &interior_cells {
                // -------------------------------------------------
                // Topology-based room boundary
                // -------------------------------------------------
                //
                // RoomPlan bounds are only semantic planning data.
                // They must NOT be treated as the physical wall boundary.
                //
                // A wall exists only when this room cell has a horizontal
                // neighbour which:
                //
                //   1. belongs to the authoritative real building floor area
                //   2. does NOT belong to this room
                //
                // This makes room walls follow the actual room topology
                // instead of blindly following the rectangular room bounds.
                //
                // If a neighbour is outside cached_floor_area, we do NOT
                // create an interior wall there. The exterior building
                // geometry remains owned by the existing building renderer.
                // -------------------------------------------------

                let neighbours = [(x - 1, z), (x + 1, z), (x, z - 1), (x, z + 1)];

                let boundary = neighbours.iter().any(|&(nx, nz)| {
                    cached_floor_area_set.contains(&(nx, nz)) && !room_cell_set.contains(&(nx, nz))
                });

                if !boundary {
                    continue;
                }

                for y in 1..=3 {
                    editor.set_block_absolute(wall_block, x, floor_y + y, z, None, None);
                }
            }
            // -------------------------------------------------
            // Room ceiling
            // -------------------------------------------------
            //
            // HARD PHYSICAL RULE:
            //
            // A semantic room MUST NOT create an intermediate
            // floor/ceiling simply because another room exists
            // above or below it in RoomPlan.
            //
            // The physical floor structure is controlled by
            // floor_levels, not by individual room rectangles.
            //
            // Therefore only the TOP physical floor receives
            // the building ceiling here.
            //
            // This prevents:
            //
            //     Room A
            //     ========  <- accidental intermediate ceiling
            //     Room B
            //
            // from appearing inside a real tall building.
            //
            // -------------------------------------------------
            let is_top_floor = floor_index + 1 >= floor_levels.len();

            if is_top_floor {
                for &(x, z) in &interior_cells {
                    if !cached_floor_area_set.contains(&(x, z)) {
                        continue;
                    }

                    editor.set_block_absolute(wall_block, x, floor_y + 4, z, None, None);
                }
            }

            // -------------------------------------------------
            // Room loot / container materialization
            // -------------------------------------------------
            //
            // HARD RULES:
            // - Uses the already-materialized physical room cells.
            // - Never expands the real building footprint.
            // - RoomPlan remains semantic.
            // - cached_floor_area remains authoritative.
            // - Placement and loot selection are delegated to
            //   room_loot.rs.
            //
            // The existing Room Loot system handles:
            //   RoomType -> loot pool -> items -> chest NBT.
            //
            // -------------------------------------------------
            let room_type_debug = format!("{:?}", room.room_type);

            let _ = generate_room_loot(
                editor,
                &room_type_debug,
                &interior_cells,
                floor_y,
            );
        }
    }

    println!("[BI ROOM] MATERIALIZATION END");

    // ---------------------------------------------------------
    // Main entrance rendering
    // ---------------------------------------------------------
    //
    // IMPORTANT:
    // The main entrance is NOT an interior-room object.
    //
    // It is allowed to modify the already reconstructed BUILDING
    // EXTERIOR wall, because the doorway is part of the real-world
    // building entrance.
    //
    // Therefore:
    //   - cached_floor_area is NOT used as the sole gate here.
    //   - The authoritative entrance coordinate comes ONLY from
    //     planned_building.doorway_plan.main_entrance.
    //   - BBox is never expanded.
    //   - The building footprint is never regenerated.
    //   - The renderer never invents another entrance.
    //   - The door opening is restricted to the existing building
    //     renderer bounds.
    //   - The semantic entrance coordinate remains authoritative.
    //
    // Interior rooms/furniture remain strictly constrained by
    // cached_floor_area elsewhere in this renderer.
    // ---------------------------------------------------------
    if let Some(main_door) = planned_building.doorway_plan.main_entrance {
        if let (Some(requested_x), Some(requested_z), Some(side)) =
            (main_door.x, main_door.z, main_door.side)
        {
            let orientation = match side {
                crate::element_processing::building_intelligence::EntranceSide::North
                | crate::element_processing::building_intelligence::EntranceSide::South => "north",

                crate::element_processing::building_intelligence::EntranceSide::East
                | crate::element_processing::building_intelligence::EntranceSide::West => "east",
            };

            let floor_y =
                floor_levels.first().copied().unwrap_or(start_y_offset);
            let door_y = floor_y + 1;
            let width = main_door.width.max(1);

            /*
             * HARD MAIN ENTRANCE CONTRACT
             *
             * The semantic entrance candidate is authoritative when
             * it physically corresponds to the reconstructed building
             * shell.
             *
             * If it does NOT touch the real shell, resolve a fallback
             * from cached_floor_area:
             *
             *     exterior shell
             *          │
             *          │
             *     interior boundary cell
             *
             * The fallback is valid ONLY when:
             *
             *   1. every interior doorway cell is inside
             *      cached_floor_area
             *
             *   2. every outward wall cell is outside
             *      cached_floor_area
             *
             *   3. the outward wall cell remains inside the
             *      reconstructed building renderer bounds
             *
             *   4. an actual non-AIR shell block exists at the
             *      doorway height
             *
             * No coordinate is fabricated.
             * No BBox expansion is allowed.
             */

            let cached_floor = &cached_floor_area_set;

            let width = width.min(2).max(1);
            let half = width / 2;

            /*
             * Build a complete candidate footprint.
             *
             * Returns:
             *
             *   (interior_cells, wall_cells)
             *
             * where the wall cells are the actual exterior shell
             * coordinates and the interior cells are the first
             * cached-floor cells behind them.
             */
            let build_candidate = |center_x: i32,
                                   center_z: i32,
                                   candidate_side:
                                       crate::element_processing::building_intelligence::EntranceSide|
             -> Option<(Vec<(i32, i32)>, Vec<(i32, i32)>)> {
                let mut interior_cells = Vec::new();
                let mut wall_cells = Vec::new();

                for offset in -half..=(width - half - 1) {
                    let (ix, iz, wx, wz) = match candidate_side {
                        crate::element_processing::building_intelligence::EntranceSide::North => {
                            (
                                center_x + offset,
                                center_z,
                                center_x + offset,
                                center_z - 1,
                            )
                        }

                        crate::element_processing::building_intelligence::EntranceSide::South => {
                            (
                                center_x + offset,
                                center_z,
                                center_x + offset,
                                center_z + 1,
                            )
                        }

                        crate::element_processing::building_intelligence::EntranceSide::East => {
                            (
                                center_x,
                                center_z + offset,
                                center_x + 1,
                                center_z + offset,
                            )
                        }

                        crate::element_processing::building_intelligence::EntranceSide::West => {
                            (
                                center_x,
                                center_z + offset,
                                center_x - 1,
                                center_z + offset,
                            )
                        }
                    };

                    // Interior MUST be real reconstructed floor.
                    if !cached_floor.contains(&(ix, iz)) {
                        return None;
                    }

                    // Exterior wall MUST NOT be part of interior floor.
                    if cached_floor.contains(&(wx, wz)) {
                        return None;
                    }

                    // Wall must stay inside the reconstructed renderer bounds.
                    if wx < min_x
                        || wx > max_x
                        || wz < min_z
                        || wz > max_z
                    {
                        return None;
                    }

                    // There must actually be reconstructed shell material
                    // at the doorway height.
                    let lower = editor.get_block_absolute(
                        wx,
                        door_y,
                        wz,
                    );

                    let upper = editor.get_block_absolute(
                        wx,
                        door_y + 1,
                        wz,
                    );

                    let has_shell =
                        lower.map(|b| b != AIR).unwrap_or(false)
                        || upper.map(|b| b != AIR).unwrap_or(false);

                    if !has_shell {
                        return None;
                    }

                    interior_cells.push((ix, iz));
                    wall_cells.push((wx, wz));
                }

                Some((interior_cells, wall_cells))
            };

            /*
             * First try the semantic entrance coordinate exactly as
             * supplied by the detector.
             */
            let authoritative_candidate =
                build_candidate(
                    requested_x,
                    requested_z,
                    side,
                );

            /*
             * If the semantic coordinate does not physically touch
             * the reconstructed shell, search the real cached-floor
             * boundary for a valid fallback.
             */
            let candidate = if authoritative_candidate.is_some() {
                println!(
                    "[BI MAIN DOOR] AUTHORITATIVE SHELL MATCH: ({}, {}) side={:?}",
                    requested_x,
                    requested_z,
                    side
                );

                Some((
                    requested_x,
                    requested_z,
                    authoritative_candidate.unwrap(),
                ))
            } else {
                println!(
                    "[BI MAIN DOOR] AUTHORITATIVE ENTRANCE DOES NOT TOUCH REAL SHELL: ({}, {}) side={:?}",
                    requested_x,
                    requested_z,
                    side
                );

                let mut fallback_candidates = Vec::new();

                for &(ix, iz) in cached_floor_area {
                    let candidate = build_candidate(ix, iz, side);

                    if let Some(candidate) = candidate {
                        let distance =
                            (ix - requested_x).abs()
                            + (iz - requested_z).abs();

                        fallback_candidates.push((
                            distance,
                            ix,
                            iz,
                            candidate,
                        ));
                    }
                }

                fallback_candidates
                    .into_iter()
                    .min_by_key(|(distance, _, _, _)| *distance)
                    .map(|(_, ix, iz, candidate)| {
                        println!(
                            "[BI MAIN DOOR] FALLBACK REAL SHELL BOUNDARY: interior=({}, {}) side={:?}",
                            ix,
                            iz,
                            side
                        );

                        (ix, iz, candidate)
                    })
            };

            let Some((interior_anchor_x, interior_anchor_z, (interior_cells, wall_cells))) =
                candidate
            else {
                println!(
                    "[BI MAIN DOOR] SKIP: no valid real shell boundary found for side={:?}",
                    side
                );

                // IMPORTANT:
                // Never fabricate an entrance when the reconstructed
                // shell and cached floor do not provide a valid boundary.
                return;
            };

            /*
             * TRANSACTIONAL VALIDATION COMPLETE.
             *
             * At this point:
             *   - every interior cell is cached
             *   - every wall cell is inside building bounds
             *   - every wall cell is outside cached floor
             *   - every wall cell contains real shell material
             *
             * Only NOW may geometry be opened.
             */

            for &(wx, wz) in &wall_cells {
                for y in door_y..=(door_y + 1) {
                    editor.set_block_absolute(
                        AIR,
                        wx,
                        y,
                        wz,
                        None,
                        Some(&[]),
                    );
                }
            }

            println!(
                "[BI MAIN DOOR] REAL SHELL OPENED 2-high: wall=({}, {}) cells={}",
                interior_anchor_x,
                interior_anchor_z,
                wall_cells.len()
            );

            /*
             * Clear the first interior cells so the entrance is
             * physically connected to the reconstructed interior.
             */
            for &(ix, iz) in &interior_cells {
                for y in door_y..=(door_y + 1) {
                    editor.set_block_absolute(
                        AIR,
                        ix,
                        y,
                        iz,
                        None,
                        Some(&[]),
                    );
                }
            }

            /*
             * Materialize the actual Minecraft door at the REAL
             * building shell position.
             *
             * For a width-2 door, use the first wall cell as the
             * physical anchor because the current renderer stores
             * one actual door block pair.
             */
            let wall_index =
                (wall_cells.len().saturating_sub(1)) / 2;

            let (wall_x, wall_z) =
                wall_cells[wall_index];

            let lower =
                interior_door_block_with_state(
                    orientation,
                    false,
                    false,
                    false,
                );

            let upper =
                interior_door_block_with_state(
                    orientation,
                    true,
                    false,
                    false,
                );

            editor.set_block_with_properties_absolute(
                lower,
                wall_x,
                door_y,
                wall_z,
                None,
                None,
            );

            editor.set_block_with_properties_absolute(
                upper,
                wall_x,
                door_y + 1,
                wall_z,
                None,
                None,
            );

            println!(
                "[BI MAIN DOOR] MATERIALIZED shell=({}, {}) interior=({}, {}) side={:?} width={} y={}..{}",
                wall_x,
                wall_z,
                interior_anchor_x,
                interior_anchor_z,
                side,
                width,
                door_y,
                door_y + 1
            );
        } else {
            println!(
                "[BI MAIN DOOR] SKIP incomplete authoritative entrance data"
            );
        }
    }

    // ---------------------------------------------------------
    // ---------------------------------------------------------
    // Interior doorway rendering
    // ---------------------------------------------------------
    //
    // HARD PHYSICAL RULE:
    //
    // RoomGraph / DoorwayPlan = semantic intent
    // FloorPlan = semantic room geometry
    // cached_floor_area = REAL reconstructed interior footprint
    //
    // Interior doors MUST NEVER create space outside the
    // reconstructed building interior.
    //
    // Therefore:
    //
    //   1. Find the authoritative shared wall from FloorPlan.
    //   2. Clamp the requested doorway width to that wall.
    //   3. Validate EVERY doorway cell against cached_floor_area.
    //   4. If ANY cell is invalid -> modify ZERO blocks.
    //   5. Only after complete validation, open the wall and
    //      materialize the actual Minecraft door.
    //
    // This stage NEVER modifies:
    //   - FloorPlan geometry
    //   - RoomGraph topology
    //   - building footprint
    //   - BBox
    //   - cached_floor_area
    //
    // ---------------------------------------------------------

    
for door in &planned_building.doorway_plan.doors {
    let Some(from_room) =
        planned_building.room_graph.rooms.get(door.from_room)
    else {
        println!(
            "[BI INTERIOR DOOR] SKIP invalid from_room={}",
            door.from_room
        );
        continue;
    };

    let Some(to_room) =
        planned_building.room_graph.rooms.get(door.to_room)
    else {
        println!(
            "[BI INTERIOR DOOR] SKIP invalid to_room={}",
            door.to_room
        );
        continue;
    };

    // Interior doors only connect rooms on the same floor.
    if from_room.floor != to_room.floor {
        println!(
            "[BI INTERIOR DOOR] SKIP different floors {} -> {}",
            from_room.floor,
            to_room.floor
        );
        continue;
    }

    let Some(floor_plan) =
        planned_building.floor_plans.get(from_room.floor)
    else {
        println!(
            "[BI INTERIOR DOOR] SKIP missing floor plan floor={}",
            from_room.floor
        );
        continue;
    };

    let Some(from_plan) =
        floor_plan.rooms.get(from_room.floor_room_index)
    else {
        println!(
            "[BI INTERIOR DOOR] SKIP missing from room plan floor={} room={}",
            from_room.floor,
            from_room.floor_room_index
        );
        continue;
    };

    let Some(to_plan) =
        floor_plan.rooms.get(to_room.floor_room_index)
    else {
        println!(
            "[BI INTERIOR DOOR] SKIP missing to room plan floor={} room={}",
            to_room.floor,
            to_room.floor_room_index
        );
        continue;
    };

    let a = from_plan.bounds;
    let b = to_plan.bounds;

    let floor_y = floor_levels
        .get(from_room.floor)
        .copied()
        .unwrap_or(start_y_offset + (from_room.floor as i32 * 4));

    let requested_width = door.width.max(1);

    // ========================================================
    // CASE 1
    // Rooms touch along X.
    //
    //      A | B
    //
    // Shared wall + 2 cells into A + 2 cells into B.
    // ========================================================

    if a.max_x + 1 == b.min_x || b.max_x + 1 == a.min_x {
        let wall_x = if a.max_x + 1 == b.min_x {
            a.max_x
        } else {
            b.max_x
        };

        let shared_min_z = a.min_z.max(b.min_z);
        let shared_max_z = a.max_z.min(b.max_z);

        if shared_min_z > shared_max_z {
            println!(
                "[BI INTERIOR DOOR] SKIP no shared Z wall"
            );
            continue;
        }

        let available = shared_max_z - shared_min_z + 1;
        let actual_width = requested_width.min(available);

        if actual_width <= 0 {
            continue;
        }

        let center_z =
            door.z.clamp(shared_min_z, shared_max_z);

        let mut start_z =
            center_z - ((actual_width - 1) / 2);

        let mut end_z =
            start_z + actual_width - 1;

        if start_z < shared_min_z {
            start_z = shared_min_z;
            end_z = start_z + actual_width - 1;
        }

        if end_z > shared_max_z {
            end_z = shared_max_z;
            start_z = end_z - actual_width + 1;
        }

        // Determine which direction each room extends
        // away from the shared wall.
        let from_inside_x_1 =
            if a.max_x == wall_x {
                wall_x - 1
            } else {
                wall_x + 1
            };

        let from_inside_x_2 =
            if a.max_x == wall_x {
                wall_x - 2
            } else {
                wall_x + 2
            };

        let to_inside_x_1 =
            if b.max_x == wall_x {
                wall_x - 1
            } else {
                wall_x + 1
            };

        let to_inside_x_2 =
            if b.max_x == wall_x {
                wall_x - 2
            } else {
                wall_x + 2
            };

        // ----------------------------------------------------
        // TRANSACTIONAL VALIDATION
        // ----------------------------------------------------

        let mut valid = true;

        for z in start_z..=end_z {
            let required_cells = [
                (wall_x, z),
                (from_inside_x_1, z),
                (from_inside_x_2, z),
                (to_inside_x_1, z),
                (to_inside_x_2, z),
            ];

            for &(check_x, check_z) in &required_cells {
                if !cached_floor_area_set
                    .contains(&(check_x, check_z))
                {
                    println!(
                        "[BI INTERIOR DOOR] REJECT outside cached_floor_area: ({}, {})",
                        check_x,
                        check_z
                    );

                    valid = false;
                    break;
                }
            }

            if !valid {
                break;
            }
        }

        if !valid {
            continue;
        }

        // ----------------------------------------------------
        // MATERIALIZE
        //
        // Shared wall + 2 cells on BOTH sides.
        // Everything is 2 blocks high.
        // ----------------------------------------------------

        for z in start_z..=end_z {
            for y in (floor_y + 1)..=(floor_y + 2) {
                // Shared wall.
                editor.set_block_absolute(
                    AIR,
                    wall_x,
                    y,
                    z,
                    None,
                    Some(&[]),
                );

                // From-room depth 1.
                editor.set_block_absolute(
                    AIR,
                    from_inside_x_1,
                    y,
                    z,
                    None,
                    Some(&[]),
                );

                // From-room depth 2.
                editor.set_block_absolute(
                    AIR,
                    from_inside_x_2,
                    y,
                    z,
                    None,
                    Some(&[]),
                );

                // To-room depth 1.
                editor.set_block_absolute(
                    AIR,
                    to_inside_x_1,
                    y,
                    z,
                    None,
                    Some(&[]),
                );

                // To-room depth 2.
                editor.set_block_absolute(
                    AIR,
                    to_inside_x_2,
                    y,
                    z,
                    None,
                    Some(&[]),
                );
            }
        }

        let center_z =
            start_z + ((actual_width - 1) / 2);

        let lower =
            interior_door_block_with_state(
                "east",
                false,
                false,
                false,
            );

        let upper =
            interior_door_block_with_state(
                "east",
                true,
                false,
                false,
            );

        editor.set_block_with_properties_absolute(
            lower,
            wall_x,
            floor_y + 1,
            center_z,
            None,
            None,
        );

        editor.set_block_with_properties_absolute(
            upper,
            wall_x,
            floor_y + 2,
            center_z,
            None,
            None,
        );

        println!(
            "[BI INTERIOR DOOR] MATERIALIZED vertical wall=({}, {}) depth=2+2 width={} floor_y={}",
            wall_x,
            center_z,
            actual_width,
            floor_y
        );

        continue;
    }

    // ========================================================
    // CASE 2
    // Rooms touch along Z.
    //
    //      A
    //      -
    //      B
    //
    // Shared wall + 2 cells into A + 2 cells into B.
    // ========================================================

    if a.max_z + 1 == b.min_z || b.max_z + 1 == a.min_z {
        let wall_z = if a.max_z + 1 == b.min_z {
            a.max_z
        } else {
            b.max_z
        };

        let shared_min_x = a.min_x.max(b.min_x);
        let shared_max_x = a.max_x.min(b.max_x);

        if shared_min_x > shared_max_x {
            println!(
                "[BI INTERIOR DOOR] SKIP no shared X wall"
            );
            continue;
        }

        let available =
            shared_max_x - shared_min_x + 1;

        let actual_width =
            requested_width.min(available);

        if actual_width <= 0 {
            continue;
        }

        let center_x =
            door.x.clamp(shared_min_x, shared_max_x);

        let mut start_x =
            center_x - ((actual_width - 1) / 2);

        let mut end_x =
            start_x + actual_width - 1;

        if start_x < shared_min_x {
            start_x = shared_min_x;
            end_x = start_x + actual_width - 1;
        }

        if end_x > shared_max_x {
            end_x = shared_max_x;
            start_x = end_x - actual_width + 1;
        }

        let from_inside_z_1 =
            if a.max_z == wall_z {
                wall_z - 1
            } else {
                wall_z + 1
            };

        let from_inside_z_2 =
            if a.max_z == wall_z {
                wall_z - 2
            } else {
                wall_z + 2
            };

        let to_inside_z_1 =
            if b.max_z == wall_z {
                wall_z - 1
            } else {
                wall_z + 1
            };

        let to_inside_z_2 =
            if b.max_z == wall_z {
                wall_z - 2
            } else {
                wall_z + 2
            };

        // ----------------------------------------------------
        // TRANSACTIONAL VALIDATION
        // ----------------------------------------------------

        let mut valid = true;

        for x in start_x..=end_x {
            let required_cells = [
                (x, wall_z),
                (x, from_inside_z_1),
                (x, from_inside_z_2),
                (x, to_inside_z_1),
                (x, to_inside_z_2),
            ];

            for &(check_x, check_z) in &required_cells {
                if !cached_floor_area_set
                    .contains(&(check_x, check_z))
                {
                    println!(
                        "[BI INTERIOR DOOR] REJECT outside cached_floor_area: ({}, {})",
                        check_x,
                        check_z
                    );

                    valid = false;
                    break;
                }
            }

            if !valid {
                break;
            }
        }

        if !valid {
            continue;
        }

        // ----------------------------------------------------
        // MATERIALIZE
        // ----------------------------------------------------

        for x in start_x..=end_x {
            for y in (floor_y + 1)..=(floor_y + 2) {
                // Shared wall.
                editor.set_block_absolute(
                    AIR,
                    x,
                    y,
                    wall_z,
                    None,
                    Some(&[]),
                );

                // From-room depth 1.
                editor.set_block_absolute(
                    AIR,
                    x,
                    y,
                    from_inside_z_1,
                    None,
                    Some(&[]),
                );

                // From-room depth 2.
                editor.set_block_absolute(
                    AIR,
                    x,
                    y,
                    from_inside_z_2,
                    None,
                    Some(&[]),
                );

                // To-room depth 1.
                editor.set_block_absolute(
                    AIR,
                    x,
                    y,
                    to_inside_z_1,
                    None,
                    Some(&[]),
                );

                // To-room depth 2.
                editor.set_block_absolute(
                    AIR,
                    x,
                    y,
                    to_inside_z_2,
                    None,
                    Some(&[]),
                );
            }
        }

        let center_x =
            start_x + ((actual_width - 1) / 2);

        let lower =
            interior_door_block_with_state(
                "north",
                false,
                false,
                false,
            );

        let upper =
            interior_door_block_with_state(
                "north",
                true,
                false,
                false,
            );

        editor.set_block_with_properties_absolute(
            lower,
            center_x,
            floor_y + 1,
            wall_z,
            None,
            None,
        );

        editor.set_block_with_properties_absolute(
            upper,
            center_x,
            floor_y + 2,
            wall_z,
            None,
            None,
        );

        println!(
            "[BI INTERIOR DOOR] MATERIALIZED horizontal wall=({}, {}) depth=2+2 width={} floor_y={}",
            center_x,
            wall_z,
            actual_width,
            floor_y
        );

        continue;
    }

    println!(
        "[BI INTERIOR DOOR] SKIP no physical shared wall: from={} to={}",
        door.from_room,
        door.to_room
    );
}


    // ---------------------------------------------------------
    // Furniture rendering
    // ---------------------------------------------------------
    //
    // FurnitureKind is semantic intent.
    // furniture_blocks() converts that intent into actual Minecraft
    // block combinations.
    //
    // FurnitureItem.room_id is the ONLY ownership key.
    //
    // It identifies RoomGraph::RoomNode.id, which resolves to:
    //
    //     room_id
    //        ↓
    //     RoomGraph::RoomNode
    //        ↓
    //     floor
    //        ↓
    //     floor_room_index
    //        ↓
    //     FloorPlan.rooms[floor_room_index]
    //
    // NEVER resolve furniture ownership by:
    //   - RoomType
    //   - matching semantic room types
    //   - renderer-side room ordering
    //
    // Physical placement remains strictly constrained by:
    //   1. owning Room bounds
    //   2. cached_floor_area
    //   3. renderer building bounds
    //   4. WorldEditor BBox
    //

    for furniture in &planned_building.furniture {
        // -----------------------------------------------------
        // 1. Resolve the unique owning Room through RoomGraph
        // -----------------------------------------------------
        let Some(room_node) = planned_building
            .room_graph
            .rooms
            .iter()
            .find(|node| node.id == furniture.room_id)
        else {
            println!(
                "[BI FURNITURE] SKIP invalid room ownership: room_id={} kind={:?}",
                furniture.room_id, furniture.kind
            );
            continue;
        };

        // -----------------------------------------------------
        // 2. Resolve the concrete FloorPlan
        // -----------------------------------------------------
        let floor_index = room_node.floor;

        let Some(floor_plan) = planned_building.floor_plans.get(floor_index) else {
            println!(
                "[BI FURNITURE] SKIP invalid floor ownership: room_id={} floor={}",
                furniture.room_id, floor_index
            );
            continue;
        };

        // -----------------------------------------------------
        // 3. Resolve the concrete Room
        // -----------------------------------------------------
        let Some(room) = floor_plan.rooms.get(room_node.floor_room_index) else {
            println!(
                "[BI FURNITURE] SKIP invalid room index: room_id={} floor={} floor_room_index={}",
                furniture.room_id, floor_index, room_node.floor_room_index
            );
            continue;
        };

        // -----------------------------------------------------
        // Ownership integrity check
        // -----------------------------------------------------
        //
        // The semantic room type is metadata only.
        // It must NEVER be used to select the owning room.
        //
        // If it differs, ownership still follows room_id.
        if room.room_type != furniture.room_type {
            println!(
                "[BI FURNITURE] ROOM TYPE MISMATCH: room_id={} graph_type={:?} furniture_type={:?} -- ownership follows room_id",
                furniture.room_id,
                room.room_type,
                furniture.room_type
            );
        }

        let floor_y = floor_levels
            .get(floor_index)
            .copied()
            .unwrap_or(start_y_offset + (floor_index as i32 * 4));

        let origin_x = room.bounds.min_x + furniture.relative_x;
        let origin_z = room.bounds.min_z + furniture.relative_z;

        // -----------------------------------------------------
        // Validate the COMPLETE furniture footprint first.
        //
        // A furniture item is atomic: if any block would leave
        // the owning room, cached floor area, renderer building
        // bounds, or BBox boundary, reject the ENTIRE item.
        // -----------------------------------------------------
        let blocks = furniture_blocks(furniture.kind);

        let mut valid_placement = true;

        for &(_, dx, _dy, dz) in blocks {
            let x = origin_x + dx;
            let z = origin_z + dz;

            // Owning Room is a semantic/planning boundary.
            if !room.bounds.contains(x, z) {
                println!(
                    "[BI FURNITURE] REJECT outside owning room: floor={} room_id={} kind={:?} pos=({}, {})",
                    floor_index,
                    furniture.room_id,
                    furniture.kind,
                    x,
                    z
                );
                valid_placement = false;
                break;
            }

            // cached_floor_area is the authoritative physical
            // boundary of the reconstructed real-world building.
            if !cached_floor_area_set.contains(&(x, z)) {
                println!(
                    "[BI FURNITURE] REJECT outside cached_floor_area: floor={} room_id={} kind={:?} pos=({}, {})",
                    floor_index,
                    furniture.room_id,
                    furniture.kind,
                    x,
                    z
                );
                valid_placement = false;
                break;
            }

            // Renderer building bounds remain an additional guard.
            if x < min_x || x > max_x || z < min_z || z > max_z {
                println!(
                    "[BI FURNITURE] REJECT outside renderer bounds: floor={} room_id={} kind={:?} pos=({}, {})",
                    floor_index,
                    furniture.room_id,
                    furniture.kind,
                    x,
                    z
                );
                valid_placement = false;
                break;
            }
        }

        if !valid_placement {
            continue;
        }

        // -----------------------------------------------------
        // Commit only after the COMPLETE footprint passed.
        // -----------------------------------------------------
        for &(block, dx, dy, dz) in blocks {
            let x = origin_x + dx;
            let y = floor_y + 1 + dy;
            let z = origin_z + dz;

            editor.set_block_absolute(block, x, y, z, None, None);
        }
    }

    // ---------------------------------------------------------
    // Intelligent lighting rendering
    // ---------------------------------------------------------
    //
    // LightingPlan is produced by Interior Intelligence from:
    //   existing real-world windows
    //   room geometry
    //   room area
    //
    // This stage only renders already-decided interior lights.
    // It never creates or modifies exterior windows or geometry.
    //
    for light in &planned_building.lighting.placements {
        let matching_room = planned_building
            .floor_plans
            .get(light.floor as usize)
            .and_then(|plan| {
                plan.rooms.iter().find(|room| {
                    room.room_type == light.room_type && room.bounds.contains(light.x, light.z)
                })
            });

        let Some(room) = matching_room else {
            continue;
        };

        // Keep every light strictly inside its owning room.
        if !room.bounds.contains(light.x, light.z) {
            continue;
        }

        // Keep every light strictly inside the original
        // real-world building footprint.
        if !cached_floor_area.contains(&(light.x, light.z)) {
            continue;
        }
        if light.x < min_x || light.x > max_x || light.z < min_z || light.z > max_z {
            continue;
        }

        let floor_y = floor_levels
            .get(light.floor as usize)
            .copied()
            .unwrap_or(start_y_offset + (light.floor * 4));

        match light.kind {
            crate::element_processing::subprocessor::interior::decision::LightKind::Ceiling => {
                editor.set_block_absolute(GLOWSTONE, light.x, floor_y + 3, light.z, None, None);
            }

            crate::element_processing::subprocessor::interior::decision::LightKind::Wall => {
                editor.set_block_absolute(LANTERN, light.x, floor_y + 2, light.z, None, None);
            }
        }
    }

    // ---------------------------------------------------------

    // ---------------------------------------------------------
    // Semantic rendering summary
    // ---------------------------------------------------------

    let _floor_plan_count = planned_building.floor_plans.len();
    let _room_count = planned_building.room_graph.rooms.len();
    let _furniture_count = planned_building.furniture.len();
    let _has_entrance = planned_building.entrance.is_some();

    // ---------------------------------------------------------
    // Vertical access rendering
    // ---------------------------------------------------------
    //
    // HARD RULE:
    // cached_floor_area is the authoritative reconstructed physical
    // interior footprint.
    //
    // A vertical-access structure is rendered ONLY when EVERY X/Z
    // coordinate in its complete physical footprint exists in
    // cached_floor_area.
    //
    // We never:
    // - expand cached_floor_area
    // - modify cached_floor_area
    // - modify FloorPlan geometry
    // - modify Room geometry
    // - expand the building BBox
    // - infer missing interior floor cells
    //
    // If one single footprint cell is outside cached_floor_area,
    // the ENTIRE vertical-access structure is rejected.
    let cached_floor_area_set: std::collections::HashSet<(i32, i32)> =
        cached_floor_area.iter().copied().collect();

    let vertical_plans = planned_building.circulation.vertical_access_plans();

    for plan in vertical_plans {
        let footprint = plan.footprint_cells();

        if footprint.is_empty() {
            eprintln!("[BI VERTICAL] REJECT: empty vertical-access footprint");
            continue;
        }

        let outside_cell = footprint
            .iter()
            .find(|&&(x, z)| !cached_floor_area_set.contains(&(x, z)));

        if let Some(&(x, z)) = outside_cell {
            eprintln!(
                "[BI VERTICAL] REJECT: footprint outside cached_floor_area at ({}, {})",
                x, z
            );
            continue;
        }

        // Only an already-approved footprint reaches the physical
        // Minecraft renderer.
        let mut vertical_editor = WorldEditorVerticalAdapter { editor };

        render_vertical_access(
            &mut vertical_editor,
            plan,
            BlockWithProperties::new(OAK_STAIRS, None),
            BlockWithProperties::new(LADDER, None),
            cached_floor_area,
        );
    }

    let _vertical_connections = vertical_plans.len();

    let _floor_plans: &[FloorPlan] = &planned_building.floor_plans;

    // Keep the existing geographic/rendering parameters explicitly
    // acknowledged here. They remain owned by the outer building
    // generation pipeline and are not modified by Intelligence.
    let _ = (
        effective_building_height,
        abs_terrain_offset,
        is_abandoned_building,
        has_sloped_roof,
        effective_passages,
        cached_floor_area,
    );
}
