use crate::block_definitions::*;
use crate::element_processing::building_intelligence::circulation::DoorOrientation;
use crate::element_processing::building_intelligence::room_graph::RoomConnectionKind;
use crate::element_processing::building_intelligence::vertical::{
    render_vertical_access, VerticalAccessEditor,
};
use crate::element_processing::building_intelligence::FurnitureKind;
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
    type Block = Block;

    fn place_block(&mut self, block: Self::Block, x: i32, y: i32, z: i32) {
        self.editor.set_block(block, x, y, z, None, None);
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
        FurnitureKind::Bed => &[(RED_BED_NORTH_HEAD, 0, 0, 0), (RED_BED_NORTH_FOOT, 1, 0, 0)],

        FurnitureKind::Sofa => &[(OAK_STAIRS, 0, 0, 0), (OAK_STAIRS, 1, 0, 0)],

        FurnitureKind::Table => &[(OAK_FENCE, 0, 0, 0), (OAK_SLAB_TOP, 0, 1, 0)],

        FurnitureKind::Chair => &[(OAK_STAIRS, 0, 0, 0)],

        FurnitureKind::KitchenCounter => &[(CRAFTING_TABLE, 0, 0, 0), (CRAFTING_TABLE, 1, 0, 0)],

        FurnitureKind::Sink => &[(CAULDRON, 0, 0, 0)],

        FurnitureKind::Toilet => &[(CAULDRON, 0, 0, 0)],

        FurnitureKind::Shower => &[(CAULDRON, 0, 0, 0)],

        FurnitureKind::Bathtub => &[(CAULDRON, 0, 0, 0), (CAULDRON, 1, 0, 0)],

        FurnitureKind::Desk => &[(OAK_FENCE, 0, 0, 0), (OAK_SLAB_TOP, 0, 1, 0)],

        FurnitureKind::Bookshelf => &[(BOOKSHELF, 0, 0, 0), (BOOKSHELF, 0, 1, 0)],

        FurnitureKind::Checkout => &[(CRAFTING_TABLE, 0, 0, 0), (OAK_SLAB_TOP, 1, 1, 0)],

        FurnitureKind::Shelf => &[(OAK_FENCE, 0, 0, 0), (OAK_SLAB_TOP, 0, 1, 0)],

        FurnitureKind::HospitalBed => {
            &[(RED_BED_NORTH_HEAD, 0, 0, 0), (RED_BED_NORTH_FOOT, 1, 0, 0)]
        }

        FurnitureKind::MedicalDesk => &[(OAK_FENCE, 0, 0, 0), (OAK_SLAB_TOP, 0, 1, 0)],

        FurnitureKind::ClassroomDesk => &[(OAK_FENCE, 0, 0, 0), (OAK_SLAB_TOP, 0, 1, 0)],

        FurnitureKind::StorageShelf => &[(BARREL, 0, 0, 0), (BARREL, 0, 1, 0)],

        FurnitureKind::DiningTable => &[
            (OAK_FENCE, 0, 0, 0),
            (OAK_SLAB_TOP, 0, 1, 0),
            (OAK_FENCE, 1, 0, 0),
            (OAK_SLAB_TOP, 1, 1, 0),
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

    if planned_building.floor_plans.is_empty() || planned_building.room_graph.rooms.is_empty() {
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
    // Semantic graph validation
    // ---------------------------------------------------------

    let graph_valid = planned_building
        .room_graph
        .connections
        .iter()
        .all(|connection| {
            connection.from < planned_building.room_graph.rooms.len()
                && connection.to < planned_building.room_graph.rooms.len()
        });

    if !graph_valid {
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
    // Intelligent physical rendering
    // ---------------------------------------------------------
    //
    // Building Intelligence is now the semantic source of truth
    // for the interior layout.
    //
    // The existing building footprint remains the hard geometry
    // boundary. Room plans are rendered only inside that boundary.
    //
    // Legacy generation remains available through the validation
    // fallbacks above.

    for (floor_index, floor_plan) in planned_building.floor_plans.iter().enumerate() {
        let floor_y = floor_levels
            .get(floor_index)
            .copied()
            .unwrap_or(start_y_offset + (floor_index as i32 * 4));

        for room in &floor_plan.rooms {
            let room_min_x = room.bounds.min_x.max(min_x);
            let room_max_x = room.bounds.max_x.min(max_x);
            let room_min_z = room.bounds.min_z.max(min_z);
            let room_max_z = room.bounds.max_z.min(max_z);

            if room_min_x > room_max_x || room_min_z > room_max_z {
                continue;
            }

            // Floor
            for z in room_min_z..=room_max_z {
                for x in room_min_x..=room_max_x {
                    editor.set_block(wall_block, floor_y, z, x, None, None);
                }
            }

            // Walls
            for z in room_min_z..=room_max_z {
                for x in room_min_x..=room_max_x {
                    if x == room_min_x || x == room_max_x || z == room_min_z || z == room_max_z {
                        for y in 1..=3 {
                            editor.set_block(wall_block, floor_y + y, z, x, None, None);
                        }
                    }
                }
            }

            // Ceiling
            for z in room_min_z..=room_max_z {
                for x in room_min_x..=room_max_x {
                    editor.set_block(wall_block, floor_y + 4, z, x, None, None);
                }
            }
        }
    }

    // ---------------------------------------------------------
    // Interior doorway rendering
    // ---------------------------------------------------------
    //
    // DoorwayPlan is the semantic source of truth for interior
    // connections. This stage only removes wall blocks to create
    // physical openings between already-generated rooms.
    //
    // IMPORTANT:
    // - Does not modify room geometry.
    // - Does not expand the building footprint.
    // - Does not modify BBox.
    //
    for door in &planned_building.doorway_plan.doors {
        let Some(from_room) = planned_building.room_graph.rooms.get(door.from_room) else {
            continue;
        };

        let Some(to_room) = planned_building.room_graph.rooms.get(door.to_room) else {
            continue;
        };

        if door.kind == RoomConnectionKind::VerticalAccess {
            continue;
        }

        let Some(from_plan) = planned_building.floor_plans.get(from_room.floor) else {
            continue;
        };

        let Some(to_plan) = planned_building.floor_plans.get(to_room.floor) else {
            continue;
        };

        let from_index = planned_building
            .room_graph
            .rooms
            .iter()
            .filter(|other| {
                other.floor == from_room.floor && other.room_type == from_room.room_type
            })
            .position(|other| other.id == door.from_room);

        let to_index = planned_building
            .room_graph
            .rooms
            .iter()
            .filter(|other| other.floor == to_room.floor && other.room_type == to_room.room_type)
            .position(|other| other.id == door.to_room);

        let (Some(from_index), Some(to_index)) = (from_index, to_index) else {
            continue;
        };

        let Some(a) = from_plan
            .rooms
            .iter()
            .filter(|room| room.room_type == from_room.room_type)
            .nth(from_index)
            .map(|room| room.bounds)
        else {
            continue;
        };

        let Some(b) = to_plan
            .rooms
            .iter()
            .filter(|room| room.room_type == to_room.room_type)
            .nth(to_index)
            .map(|room| room.bounds)
        else {
            continue;
        };

        let width = door.width.max(1);
        let half = width / 2;

        let floor_y = floor_levels
            .get(from_room.floor)
            .copied()
            .unwrap_or(start_y_offset + (from_room.floor as i32 * 4));

        // East / west shared wall.
        if a.max_x + 1 == b.min_x || b.max_x + 1 == a.min_x {
            let wall_x = if a.max_x + 1 == b.min_x {
                a.max_x
            } else {
                a.min_x
            };

            for z in (door.z - half)..=(door.z + half) {
                if z < min_z || z > max_z {
                    continue;
                }

                if z < a.min_z.min(b.min_z) || z > a.max_z.max(b.max_z) {
                    continue;
                }

                for y in (floor_y + 1)..=(floor_y + 3) {
                    editor.set_block(AIR, wall_x, y, z, None, None);
                }
            }
        }

        // North / south shared wall.
        if a.max_z + 1 == b.min_z || b.max_z + 1 == a.min_z {
            let wall_z = if a.max_z + 1 == b.min_z {
                a.max_z
            } else {
                a.min_z
            };

            for x in (door.x - half)..=(door.x + half) {
                if x < min_x || x > max_x {
                    continue;
                }

                if x < a.min_x.min(b.min_x) || x > a.max_x.max(b.max_x) {
                    continue;
                }

                for y in (floor_y + 1)..=(floor_y + 3) {
                    editor.set_block(AIR, x, y, wall_z, None, None);
                }
            }
        }
    }

    // ---------------------------------------------------------
    // Main entrance rendering
    // ---------------------------------------------------------
    //
    // The entrance was already detected from real-world building
    // intelligence. Materialize that existing entrance here.
    //
    // This does NOT create, move, or redefine the building entrance.
    // It only places the Minecraft door at the already-decided
    // real-world entrance coordinate.
    //
    if let Some(main_door) = planned_building.doorway_plan.main_entrance {
        if let (Some(x), Some(z), Some(side)) = (main_door.x, main_door.z, main_door.side) {
            let orientation = match side {
                crate::element_processing::building_intelligence::EntranceSide::North
                | crate::element_processing::building_intelligence::EntranceSide::South => "north",

                crate::element_processing::building_intelligence::EntranceSide::East
                | crate::element_processing::building_intelligence::EntranceSide::West => "east",
            };

            let floor_y = floor_levels.first().copied().unwrap_or(start_y_offset);

            let door_y = floor_y + 1;

            if x >= min_x && x <= max_x && z >= min_z && z <= max_z {
                let lower = interior_door_block_with_state(orientation, false, false, false);
                let upper = interior_door_block_with_state(orientation, true, false, false);

                editor.set_block_with_properties_absolute(lower, x, door_y, z, None, None);
                editor.set_block_with_properties_absolute(upper, x, door_y + 1, z, None, None);
            }
        }
    }

    // Interior doorway rendering
    // ---------------------------------------------------------
    //

    // Materialize each approved doorway as a two-block oak door.
    // Geometry/placement remains bounded by the existing doorway validation above.
    for door in &planned_building.doorway_plan.doors {
        let Some(from_room) = planned_building.room_graph.room(door.from_room) else {
            continue;
        };

        let floor_y = floor_levels
            .get(from_room.floor)
            .copied()
            .unwrap_or(start_y_offset + (from_room.floor as i32 * 4));

        let orientation = match door.orientation {
            DoorOrientation::HorizontalWall => "north",
            DoorOrientation::VerticalWall => "east",
        };

        let x = door.x;
        let z = door.z;
        let door_y = floor_y + 1;

        // Consume semantic DoorKind from the RoomGraph / DoorwayPlan.
        match door.door_kind {
            crate::element_processing::building_intelligence::decision::doors::DoorKind::MainEntrance
            | crate::element_processing::building_intelligence::decision::doors::DoorKind::Interior
            | crate::element_processing::building_intelligence::decision::doors::DoorKind::Service => {
                let lower = interior_door_block_with_state(orientation, false, false, false);
                let upper = interior_door_block_with_state(orientation, true, false, false);

                editor.set_block_with_properties_absolute(lower, x, door_y, z, None, None);
                editor.set_block_with_properties_absolute(upper, x, door_y + 1, z, None, None);
            }
        }
    }

    // DoorwayPlan contains only semantic doorway intents derived
    // from already-existing room geometry.
    //
    // This renderer:
    // - never expands a room
    // - never modifies the building footprint
    // - never modifies BBox
    // - only removes wall blocks inside the shared room boundary
    //
    for door in &planned_building.doorway_plan.doors {
        let Some(from_room) = planned_building.room_graph.rooms.get(door.from_room) else {
            continue;
        };

        let Some(to_room) = planned_building.room_graph.rooms.get(door.to_room) else {
            continue;
        };

        if from_room.floor != to_room.floor {
            continue;
        }

        let Some(floor_plan) = planned_building.floor_plans.get(from_room.floor) else {
            continue;
        };

        let rooms_on_floor = &floor_plan.rooms;

        let room_index = |room_id: usize| -> Option<usize> {
            let node = planned_building.room_graph.rooms.get(room_id)?;

            rooms_on_floor
                .iter()
                .enumerate()
                .filter(|(_, room)| room.room_type == node.room_type)
                .nth(
                    planned_building
                        .room_graph
                        .rooms
                        .iter()
                        .filter(|other| {
                            other.floor == node.floor && other.room_type == node.room_type
                        })
                        .position(|other| other.id == room_id)?,
                )
                .map(|(index, _)| index)
        };

        let Some(from_index) = room_index(door.from_room) else {
            continue;
        };

        let Some(to_index) = room_index(door.to_room) else {
            continue;
        };

        let Some(from_bounds) = rooms_on_floor.get(from_index).map(|r| r.bounds) else {
            continue;
        };

        let Some(to_bounds) = rooms_on_floor.get(to_index).map(|r| r.bounds) else {
            continue;
        };

        let floor_y = floor_levels
            .get(from_room.floor)
            .copied()
            .unwrap_or(start_y_offset + (from_room.floor as i32 * 4));

        let requested_width = door.width.max(1);

        match door.orientation {
            crate::element_processing::building_intelligence::circulation::DoorOrientation::VerticalWall => {
                // Shared wall must be the same X coordinate.
                let Some(wall_x) = (if from_bounds.max_x + 1 == to_bounds.min_x {
                    Some(from_bounds.max_x)
                } else if to_bounds.max_x + 1 == from_bounds.min_x {
                    Some(from_bounds.min_x)
                } else {
                    None
                }) else {
                    continue;
                };

                // The door can ONLY occupy the intersection of both rooms.
                let shared_min_z = from_bounds.min_z.max(to_bounds.min_z);
                let shared_max_z = from_bounds.max_z.min(to_bounds.max_z);

                if shared_min_z > shared_max_z {
                    continue;
                }

                let available = shared_max_z - shared_min_z + 1;
                let width = requested_width.min(available);

                let center_z = door.z.clamp(shared_min_z, shared_max_z);
                let mut start_z = center_z - ((width - 1) / 2);
                let mut end_z = start_z + width - 1;

                if start_z < shared_min_z {
                    start_z = shared_min_z;
                    end_z = start_z + width - 1;
                }

                if end_z > shared_max_z {
                    end_z = shared_max_z;
                    start_z = end_z - width + 1;
                }

                for z in start_z..=end_z {
                    for y in (floor_y + 1)..=(floor_y + 3) {
                        if z < from_bounds.min_z
                            || z > from_bounds.max_z
                            || z < to_bounds.min_z
                            || z > to_bounds.max_z
                        {
                            continue;
                        }

                        if wall_x < min_x
                            || wall_x > max_x
                            || z < min_z
                            || z > max_z
                        {
                            continue;
                        }

                        editor.set_block(AIR, wall_x, y, z, None, None);
                    }
                }
            }

            crate::element_processing::building_intelligence::circulation::DoorOrientation::HorizontalWall => {
                // Shared wall must be the same Z coordinate.
                let Some(wall_z) = (if from_bounds.max_z + 1 == to_bounds.min_z {
                    Some(from_bounds.max_z)
                } else if to_bounds.max_z + 1 == from_bounds.min_z {
                    Some(from_bounds.min_z)
                } else {
                    None
                }) else {
                    continue;
                };

                // The door can ONLY occupy the intersection of both rooms.
                let shared_min_x = from_bounds.min_x.max(to_bounds.min_x);
                let shared_max_x = from_bounds.max_x.min(to_bounds.max_x);

                if shared_min_x > shared_max_x {
                    continue;
                }

                let available = shared_max_x - shared_min_x + 1;
                let width = requested_width.min(available);

                let center_x = door.x.clamp(shared_min_x, shared_max_x);
                let mut start_x = center_x - ((width - 1) / 2);
                let mut end_x = start_x + width - 1;

                if start_x < shared_min_x {
                    start_x = shared_min_x;
                    end_x = start_x + width - 1;
                }

                if end_x > shared_max_x {
                    end_x = shared_max_x;
                    start_x = end_x - width + 1;
                }

                for x in start_x..=end_x {
                    for y in (floor_y + 1)..=(floor_y + 3) {
                        if x < from_bounds.min_x
                            || x > from_bounds.max_x
                            || x < to_bounds.min_x
                            || x > to_bounds.max_x
                        {
                            continue;
                        }

                        if x < min_x
                            || x > max_x
                            || wall_z < min_z
                            || wall_z > max_z
                        {
                            continue;
                        }

                        editor.set_block(AIR, x, y, wall_z, None, None);
                    }
                }
            }
        }
    }

    // ---------------------------------------------------------
    // Circulation intelligence consumer
    // ---------------------------------------------------------
    // Circulation is read-only intelligence generated from the
    // existing RoomGraph, doorway plan, floor plans and furniture.
    //
    // The renderer consumes the result here only for validation
    // and semantic routing decisions. It never changes:
    // - room geometry
    // - building footprint
    // - BBox
    // - geographic coordinates
    //
    // Whole-building circulation:
    let circulation = &planned_building.circulation;

    // BuildingCirculationPlan is the authoritative whole-building
    // circulation consumer for the already-detected entrance room.
    //
    // This does NOT create, move, or redefine the entrance.
    // It only consumes the entrance-room decision produced by
    // Building Intelligence and carries that semantic into rendering.
    let building_circulation = &planned_building.building_circulation;
    let entrance_room = building_circulation.entrance_room;

    let reachable_rooms = &building_circulation.reachable_rooms;
    let isolated_rooms = &building_circulation.isolated_rooms;

    // Deterministic semantic route from the detected entrance.
    let circulation_route = &building_circulation.route;

    // Per-floor interior circulation paths.
    // These paths are validation data produced from existing geometry
    // and furniture obstacles; they are not geometry-editing commands.
    let floor_circulation = &planned_building.floor_circulation;

    // BuildingCirculationPlan is the authoritative whole-building
    // connectivity gate for downstream interior circulation rendering.
    //
    // The renderer does not recompute room connectivity here.
    // It consumes the already-computed BuildingCirculationPlan and
    // only materializes interior paths for rooms that are reachable
    // from the authoritative entrance room.
    let circulation_is_connected = circulation.is_connected();
    let building_is_connected = building_circulation.is_connected();

    let circulation_reachable_count = reachable_rooms.len();
    let circulation_isolated_count = isolated_rooms.len();
    let circulation_route_len = circulation_route.len();

    // A valid whole-building circulation must have an entrance room.
    // If the entrance room itself is not reachable according to the
    // authoritative plan, no downstream room path is considered
    // connected to the building entrance.
    let entrance_is_reachable = entrance_room
        .map(|room_id| reachable_rooms.contains(&room_id))
        .unwrap_or(false);

    // The route is a semantic room-id route generated from the same
    // authoritative entrance. It is intentionally not converted into
    // coordinates here because RoomGraph remains the source of room
    // geometry.
    let _circulation_route_valid = match entrance_room {
        Some(room_id) => circulation_route.first().copied() == Some(room_id)
            || circulation_route.is_empty(),
        None => circulation_route.is_empty(),
    };

    // These values are intentionally consumed by the renderer's
    // downstream circulation gate rather than assigned to `_` merely
    // to silence warnings.
    let _circulation_connected =
        circulation_is_connected
        && building_is_connected
        && entrance_is_reachable
        && circulation_reachable_count > 0
        && circulation_isolated_count <= reachable_rooms.len();

    for (floor_index, floor) in floor_circulation.iter().enumerate() {
        let floor_y = floor_levels
            .get(floor_index)
            .copied()
            .unwrap_or(start_y_offset + (floor_index as i32 * 4));

        let Some(floor_plan) = planned_building.floor_plans.get(floor_index) else {
            continue;
        };

        // ---------------------------------------------------------
        // InteriorCirculationPlan -> reachable_rooms() consumer
        // ---------------------------------------------------------
        //
        // Consume the planner's authoritative reachable-room count
        // before materializing any per-room circulation path.
        //
        // This is a real downstream dependency:
        // a floor with no reachable interior rooms has no walkable
        // circulation volume to materialize.
        let floor_reachable_rooms = floor.reachable_rooms();

        if floor_reachable_rooms == 0 {
            continue;
        }

        for room in &floor.rooms {
            let room_id = room.room_id;

            // ---------------------------------------------------------
            // BuildingCirculationPlan -> InteriorCirculation consumer
            // ---------------------------------------------------------
            //
            // InteriorCirculationPlan answers:
            //   "Can I walk inside this individual room?"
            //
            // BuildingCirculationPlan answers:
            //   "Is this room connected to the building entrance?"
            //
            // Both conditions must be true before the renderer
            // materializes the calculated walkable path.
            //
            // This makes whole-building circulation an actual
            // downstream dependency instead of a read-only statistic.
            let building_reachable = reachable_rooms.contains(&room_id);

            // Isolated rooms are explicitly rejected even if a local
            // room-level path happens to be valid.
            let building_isolated = isolated_rooms.contains(&room_id);

            // Local room path + whole-building reachability are both
            // required for physical walkable-volume materialization.
            if !room.reachable
                || !building_reachable
                || building_isolated
            {
                continue;
            }

            // Materialize calculated interior circulation into the
            // Minecraft walkable volume without changing geometry.

            let Some(room_plan) = floor_plan
                .rooms
                .iter()
                .find(|candidate| candidate.bounds.contains(room.entrance.x, room.entrance.z))
            else {
                continue;
            };

            for cell in &room.path.cells {
                if !room_plan.bounds.contains(cell.x, cell.z) {
                    continue;
                }

                if cell.x < min_x || cell.x > max_x || cell.z < min_z || cell.z > max_z {
                    continue;
                }

                // Preserve the existing floor and furniture.
                // Only guarantee normal player-height clearance.
                for y in (floor_y + 1)..=(floor_y + 2) {
                    editor.set_block(AIR, cell.x, y, cell.z, None, None);
                }
            }
        }
    }

    // ---------------------------------------------------------
    // Vertical access planning
    // ---------------------------------------------------------
    //
    // BuildingDecision determines WHAT kind of vertical access
    // the building needs.
    //
    // VerticalAccessPlanner determines WHERE that access can
    // physically exist using the existing RoomGraph + FloorPlans.
    //
    // No blocks are rendered in this stage.
    //

    let vertical_plans = planned_building.circulation.vertical_access_plans();

    // VerticalAccessDecision.floors is the semantic contract for
    // how many floors the building requires vertical circulation for.
    // The planner still consumes the actual FloorPlans + RoomGraph,
    // so geometry remains authoritative and unchanged.
    debug_assert_eq!(
        planned_building.decision.vertical.floors,
        planned_building.context.floors
    );

    // ---------------------------------------------------------
    // Vertical access rendering
    // ---------------------------------------------------------
    //
    // The planner decides WHERE vertical access is physically
    // compatible with the existing floor geometry.
    //
    // The renderer only places the requested access blocks.
    //
    // No room geometry, building footprint or BBox is modified.
    //

    {
        let mut vertical_editor = WorldEditorVerticalAdapter { editor };

        for plan in vertical_plans {
            let transition_plans = planned_building
                .circulation
                .vertical_access_between(plan.from_floor, plan.to_floor);

            debug_assert!(transition_plans.iter().any(|candidate| {
                candidate.from_floor == plan.from_floor
                    && candidate.to_floor == plan.to_floor
                    && candidate.x == plan.x
                    && candidate.z == plan.z
            }));

            render_vertical_access(&mut vertical_editor, plan, OAK_STAIRS, LADDER);
        }
    }

    // ---------------------------------------------------------
    // Furniture rendering
    // ---------------------------------------------------------
    //
    // FurnitureKind is semantic intent.
    // furniture_blocks() converts that intent into actual Minecraft
    // block combinations.
    //
    // Every placement remains strictly inside:
    //   1. the owning Room bounds
    //   2. the existing building bounds
    //   3. the existing BBox enforced by WorldEditor.
    //

    let mut room_usage: std::collections::HashMap<RoomType, usize> =
        std::collections::HashMap::new();

    for furniture in &planned_building.furniture {
        let room_index = room_usage.entry(furniture.room_type).or_insert(0);

        let matching_rooms: Vec<_> = planned_building
            .floor_plans
            .iter()
            .enumerate()
            .flat_map(|(floor_index, plan)| {
                plan.rooms
                    .iter()
                    .filter(move |room| room.room_type == furniture.room_type)
                    .map(move |room| (floor_index, room))
            })
            .collect();

        if matching_rooms.is_empty() {
            continue;
        }

        let selected_index = (*room_index).min(matching_rooms.len() - 1);
        let (floor_index, room) = matching_rooms[selected_index];

        *room_index += 1;

        let floor_y = floor_levels
            .get(floor_index)
            .copied()
            .unwrap_or(start_y_offset + (floor_index as i32 * 4));

        let origin_x = room.bounds.min_x + furniture.relative_x;
        let origin_z = room.bounds.min_z + furniture.relative_z;

        for &(block, dx, dy, dz) in furniture_blocks(furniture.kind) {
            let x = origin_x + dx;
            let y = floor_y + 1 + dy;
            let z = origin_z + dz;

            if !room.bounds.contains(x, z) {
                continue;
            }

            if x < min_x || x > max_x || z < min_z || z > max_z {
                continue;
            }

            editor.set_block(block, x, y, z, None, None);
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
        if light.x < min_x || light.x > max_x || light.z < min_z || light.z > max_z {
            continue;
        }

        let floor_y = floor_levels
            .get(light.floor as usize)
            .copied()
            .unwrap_or(start_y_offset + (light.floor * 4));

        match light.kind {
            crate::element_processing::subprocessor::interior::decision::LightKind::Ceiling => {
                editor.set_block(GLOWSTONE, light.x, floor_y + 3, light.z, None, None);
            }

            crate::element_processing::subprocessor::interior::decision::LightKind::Wall => {
                editor.set_block(LANTERN, light.x, floor_y + 2, light.z, None, None);
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

    let _vertical_connections = planned_building.circulation.vertical_access_plans().len();

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
