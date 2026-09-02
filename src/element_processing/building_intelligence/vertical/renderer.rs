use super::super::decision::VerticalAccessKind;
use super::planner::{VerticalAccessDirection, VerticalAccessPlan};
use crate::block_definitions::{Block, AIR};
use crate::world_editor::MIN_Y;

use crate::block_definitions::{
    create_stair_with_properties, BlockWithProperties, StairFacing, StairShape,
};

/// Block-level vertical access renderer.
///
/// IMPORTANT:
/// - Only consumes an already calculated VerticalAccessPlan.
/// - Never changes building footprint.
/// - Never creates or moves windows.
/// - Never modifies exterior geometry.
/// - Every placement is bounded by the reconstructed building context.
///
/// The concrete Minecraft editor is supplied by the existing renderer.
/// This layer only defines the physical vertical-access contract.
pub trait VerticalAccessEditor {
    fn place_block(&mut self, block: BlockWithProperties, x: i32, y: i32, z: i32);
    fn clear_block(&mut self, x: i32, y: i32, z: i32);
    fn block_at(&self, x: i32, y: i32, z: i32) -> Option<Block>;
}

/// Render one vertical access structure.
pub fn render_vertical_access<E: VerticalAccessEditor>(
    editor: &mut E,
    plan: &VerticalAccessPlan,
    stair_block: BlockWithProperties,
    ladder_block: BlockWithProperties,
    cached_floor_area: &[(i32, i32)],
) {
    // HARD SAFETY RULE:
    // Vertical access may only exist inside the already reconstructed
    // physical interior footprint.
    //
    // cached_floor_area is authoritative. We do NOT expand it, modify it,
    // or infer additional floor cells from Room/FloorPlan/BBox geometry.
    //
    // If even ONE physical X/Z cell of the complete vertical-access
    // footprint is outside cached_floor_area, reject the ENTIRE structure.
    let cached_floor_area_set: std::collections::HashSet<(i32, i32)> =
        cached_floor_area.iter().copied().collect();

    if plan
        .footprint_cells()
        .iter()
        .any(|cell| !cached_floor_area_set.contains(cell))
    {
        eprintln!("[BI VERTICAL] REJECT: vertical access footprint outside cached_floor_area");
        return;
    }

    // ---------------------------------------------------------
    // FLOOR OPENING
    // ---------------------------------------------------------
    // cached_floor_area is the ONLY authority for X/Z placement.
    //
    // Open the floor only at existing vertical-access footprint
    // cells. Never create new floor cells and never expand bounds.
    //
    // Every floor transition is validated independently.
    fn clear_floor_openings<E: VerticalAccessEditor>(
        editor: &mut E,
        plan: &VerticalAccessPlan,
        cached_floor_area_set: &std::collections::HashSet<(i32, i32)>,
    ) {
        if plan.upper_y <= plan.lower_y {
            return;
        }

        let footprint = plan.footprint_cells();

        if footprint.is_empty() {
            return;
        }

        // One opening per floor transition.
        //
        // The vertical structure crosses from lower_y to upper_y.
        // Only clear the actual stair/ladder footprint at each
        // floor boundary. No other floor geometry is touched.
        for floor_y in
            (plan.lower_y..plan.upper_y).step_by(((plan.upper_y - plan.lower_y).max(1)) as usize)
        {
            let next_floor_y = floor_y + 1;

            for &(x, z) in &footprint {
                if !cached_floor_area_set.contains(&(x, z)) {
                    continue;
                }

                editor.clear_block(x, next_floor_y, z);
            }
        }
    }

    clear_floor_openings(editor, plan, &cached_floor_area_set);

    // ---------------------------------------------------------
    // FLOOR OPENING
    // ---------------------------------------------------------
    // cached_floor_area is authoritative.
    //
    // Every X/Z cell of the vertical structure MUST already exist
    // in cached_floor_area. No new floor cell may be invented.
    //
    // Open the floor at every floor transition crossed by this plan.
    let footprint = plan.footprint_cells();

    if footprint.is_empty() {
        eprintln!("[BI VERTICAL] REJECT: empty footprint");
        return;
    }

    // Re-validate the complete footprint locally.
    if footprint
        .iter()
        .any(|cell| !cached_floor_area_set.contains(cell))
    {
        eprintln!("[BI VERTICAL] REJECT: floor opening outside cached_floor_area");
        return;
    }

    // The plan already carries authoritative world-space Y levels.
    // For every floor boundary between from_floor and to_floor,
    // clear only the physical vertical-access footprint.
    //
    // Do NOT clear the bottom floor.
    // Do NOT clear arbitrary Y values.
    //
    // Each transition is checked independently.
    let floor_count = plan.to_floor.saturating_sub(plan.from_floor);

    if floor_count == 0 {
        return;
    }

    let vertical_span = plan.upper_y - plan.lower_y;

    if vertical_span <= 0 {
        eprintln!("[BI VERTICAL] REJECT: invalid Y span");
        return;
    }

    for level in 1..=floor_count {
        let boundary_y = plan.lower_y + (vertical_span * level as i32) / floor_count as i32;

        // The final upper floor is not opened; it is the destination floor.
        if level == floor_count {
            break;
        }

        for &(x, z) in &footprint {
            // cached_floor_area remains the sole authority.
            if !cached_floor_area_set.contains(&(x, z)) {
                continue;
            }

            editor.clear_block(x, boundary_y, z);
        }
    }

    match plan.kind {
        VerticalAccessKind::None => {}

        VerticalAccessKind::Ladder => {
            render_ladder(editor, plan, ladder_block);
        }

        VerticalAccessKind::Stair | VerticalAccessKind::MultiStair => {
            render_stair(editor, plan, stair_block, cached_floor_area);
        }
    }
}

fn render_ladder<E: VerticalAccessEditor>(
    editor: &mut E,
    plan: &VerticalAccessPlan,
    block: BlockWithProperties,
) {
    if plan.upper_y <= plan.lower_y {
        return;
    }

    for y in plan.lower_y..=plan.upper_y {
        for dx in 0..plan.width.max(1) {
            editor.place_block(block.clone(), plan.x + dx, y, plan.z);
        }
    }
}

fn render_stair<E: VerticalAccessEditor>(
    editor: &mut E,
    plan: &VerticalAccessPlan,
    block: BlockWithProperties,
    cached_floor_area: &[(i32, i32)],
) {
    if plan.upper_y <= plan.lower_y {
        return;
    }

    if plan.width <= 0 || plan.length <= 0 {
        return;
    }

    let height = plan.upper_y - plan.lower_y;

    /*
     * HARD STAIR GEOMETRY CONTRACT
     *
     * lower_y = lower-floor standing level
     * upper_y = upper-floor standing level
     *
     * Every stair step advances:
     *
     *   Y  +1
     *   X/Z +1 in the selected direction
     *
     * No interpolation is used.
     * No Y level is duplicated.
     * No stair step is placed between integer levels.
     */

    if plan.length != height + 1 {
        eprintln!(
            "[BI VERTICAL] REJECT: stair length ({}) != floor height + 1 ({})",
            plan.length,
            height + 1
        );
        return;
    }

    let cached_floor_area_set: std::collections::HashSet<(i32, i32)> =
        cached_floor_area.iter().copied().collect();

    /*
     * Stair facing must correspond to the actual ascent direction.
     *
     * Minecraft stair facing represents the direction of the
     * stair's full-block side, so it is opposite to the direction
     * in which the stair rises.
     */
    let facing = match plan.direction {
        VerticalAccessDirection::North => StairFacing::South,
        VerticalAccessDirection::East => StairFacing::West,
        VerticalAccessDirection::South => StairFacing::North,
        VerticalAccessDirection::West => StairFacing::East,
    };

    let oriented_stair = create_stair_with_properties(block.block, facing, StairShape::Straight);

    /*
     * Validate the complete stair footprint before placing ANY block.
     *
     * This prevents a partially rendered stair if one cell is outside
     * cached_floor_area.
     */
    for step in 0..=height {
        let (base_x, base_z) = match plan.direction {
            VerticalAccessDirection::North => (plan.x, plan.z - step),

            VerticalAccessDirection::East => (plan.x + step, plan.z),

            VerticalAccessDirection::South => (plan.x, plan.z + step),

            VerticalAccessDirection::West => (plan.x - step, plan.z),
        };

        for w in 0..plan.width {
            let (x, z) = match plan.direction {
                VerticalAccessDirection::North | VerticalAccessDirection::South => {
                    (base_x + w, base_z)
                }

                VerticalAccessDirection::East | VerticalAccessDirection::West => {
                    (base_x, base_z + w)
                }
            };

            if !cached_floor_area_set.contains(&(x, z)) {
                eprintln!("[BI VERTICAL] REJECT: stair footprint outside cached_floor_area");
                return;
            }
        }
    }

    /*
     * HARD STAIR OPENING CONTRACT
     *
     * The stair is two blocks wide. At the upper exit:
     *
     *   - clear the 2-wide stair exit itself
     *   - extend the opening two more blocks forward
     *   - clear two vertical blocks for player headroom
     *
     * Total opening length = 4 blocks.
     *
     * The opening is cleared BEFORE the stairs are placed so the
     * top stair itself is restored afterwards.
     *
     * cached_floor_area remains authoritative.
     */
    let opening_width = plan.width.min(2).max(1);
    let opening_length = 4;

    for forward in 0..opening_length {
        let (base_x, base_z) = match plan.direction {
            VerticalAccessDirection::North => {
                (plan.x, plan.z - height - forward)
            }
            VerticalAccessDirection::East => {
                (plan.x + height + forward, plan.z)
            }
            VerticalAccessDirection::South => {
                (plan.x, plan.z + height + forward)
            }
            VerticalAccessDirection::West => {
                (plan.x - height - forward, plan.z)
            }
        };

        for w in 0..opening_width {
            let (x, z) = match plan.direction {
                VerticalAccessDirection::North
                | VerticalAccessDirection::South => {
                    (base_x + w, base_z)
                }
                VerticalAccessDirection::East
                | VerticalAccessDirection::West => {
                    (base_x, base_z + w)
                }
            };

            if !cached_floor_area_set.contains(&(x, z)) {
                eprintln!(
                    "[BI VERTICAL] STAIR OPENING SKIP outside cached_floor_area: ({}, {})",
                    x, z
                );
                continue;
            }

            editor.clear_block(x, plan.upper_y, z);
            editor.clear_block(x, plan.upper_y + 1, z);
        }
    }

    /*
     * Place the actual stair.
     *
     * HARD SUPPORT RULE:
     *
     * If a stair is floating, do NOT add another stair.
     * Instead, extend a support column vertically downward from
     * the same X/Z position until an existing non-AIR block is
     * reached.
     *
     * Every support X/Z coordinate must remain inside
     * cached_floor_area.
     *
     * Existing blocks are never overwritten.
     */
    for step in 0..=height {
        let y = plan.lower_y + step;

        let (base_x, base_z) = match plan.direction {
            VerticalAccessDirection::North => (plan.x, plan.z - step),
            VerticalAccessDirection::East => (plan.x + step, plan.z),
            VerticalAccessDirection::South => (plan.x, plan.z + step),
            VerticalAccessDirection::West => (plan.x - step, plan.z),
        };

        for w in 0..plan.width {
            let (x, z) = match plan.direction {
                VerticalAccessDirection::North | VerticalAccessDirection::South => {
                    (base_x + w, base_z)
                }

                VerticalAccessDirection::East | VerticalAccessDirection::West => {
                    (base_x, base_z + w)
                }
            };

            editor.place_block(oriented_stair.clone(), x, y, z);

            /*
             * Check the block immediately below this stair.
             *
             * y - 1 is the support position.
             * If it already contains a real block, nothing is needed.
             */
            if !cached_floor_area_set.contains(&(x, z)) {
                eprintln!(
                    "[BI VERTICAL] REJECT: support column ({}, {}) outside cached_floor_area",
                    x, z
                );
                return;
            }

            /*
             * HARD SUPPORT CONTRACT
             *
             * A floating stair extends its support vertically
             * downward along the SAME X/Z column.
             *
             * AIR:
             *     continue downward.
             *
             * Existing non-AIR:
             *     real landing/support surface.
             *     STOP.
             *
             * The X/Z column must already exist in
             * cached_floor_area.
             *
             * No ground-level guessing.
             * No footprint expansion.
             * No BBox modification.
             */

            /*
             * HARD STAIR SUPPORT CONTRACT
             *
             * The stair block itself is the staircase.
             *
             * Do NOT scan downward and fill the entire vertical
             * column beneath every stair step. That turns the
             * staircase into a solid "stair column".
             *
             * Only the block directly underneath the current
             * stair step may be used as local support.
             *
             * cached_floor_area remains authoritative.
             */
            let support_y = y - 1;

            if support_y >= MIN_Y
                && cached_floor_area_set.contains(&(x, z))
            {
                match editor.block_at(x, support_y, z) {
                    Some(existing) if existing != AIR => {}
                    _ => {
                        editor.place_block(
                            block.clone(),
                            x,
                            support_y,
                            z,
                        );
                    }
                }
            }
        }
    }
}
