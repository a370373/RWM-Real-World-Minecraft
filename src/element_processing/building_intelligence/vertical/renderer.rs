use super::super::decision::VerticalAccessKind;
use super::planner::{VerticalAccessDirection, VerticalAccessPlan};

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
        eprintln!(
            "[BI VERTICAL] REJECT: vertical access footprint outside cached_floor_area"
        );
        return;
    }

    match plan.kind {
        VerticalAccessKind::None => {}

        VerticalAccessKind::Ladder => {
            render_ladder(editor, plan, ladder_block);
        }

        VerticalAccessKind::Stair | VerticalAccessKind::MultiStair => {
            render_stair(editor, plan, stair_block);
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
) {
    if plan.upper_y <= plan.lower_y {
        return;
    }

    let height = plan.upper_y - plan.lower_y;

    if height <= 0 || plan.width <= 0 {
        return;
    }

    /*
     * The planner determines the horizontal stair footprint.
     * The renderer maps that footprint across the complete
     * floor-to-floor vertical span.
     *
     * Room geometry remains authoritative and is never modified.
     */
    let required_length = plan.length;

    let facing = match plan.direction {
        VerticalAccessDirection::North => StairFacing::North,
        VerticalAccessDirection::East => StairFacing::East,
        VerticalAccessDirection::South => StairFacing::South,
        VerticalAccessDirection::West => StairFacing::West,
    };

    let oriented_stair = create_stair_with_properties(block.block, facing, StairShape::Straight);

    for distance in 0..required_length {
        /*
         * Horizontal stair length and vertical floor height are
         * intentionally independent.
         *
         * The building uses a fixed floor-to-floor Y spacing, while
         * StairSize controls the available horizontal footprint.
         *
         * Map the horizontal stair positions across the complete
         * lower -> upper floor height instead of assuming one block
         * of horizontal movement equals one block of vertical rise.
         */
        let y = if required_length <= 1 {
            plan.lower_y
        } else {
            let span = plan.upper_y - plan.lower_y;
            plan.lower_y + ((span * distance) / (required_length - 1))
        };

        let (base_x, base_z) = match plan.direction {
            VerticalAccessDirection::North => (plan.x, plan.z - distance),

            VerticalAccessDirection::East => (plan.x + distance, plan.z),

            VerticalAccessDirection::South => (plan.x, plan.z + distance),

            VerticalAccessDirection::West => (plan.x - distance, plan.z),
        };

        for w in 0..plan.width {
            let (block_x, block_z) = match plan.direction {
                VerticalAccessDirection::North | VerticalAccessDirection::South => {
                    (base_x + w, base_z)
                }

                VerticalAccessDirection::East | VerticalAccessDirection::West => {
                    (base_x, base_z + w)
                }
            };

            editor.place_block(oriented_stair.clone(), block_x, y, block_z);
        }
    }
}
