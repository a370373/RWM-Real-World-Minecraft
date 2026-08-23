use super::super::decision::VerticalAccessKind;
use super::planner::VerticalAccessPlan;

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
    type Block: Copy;

    fn place_block(&mut self, block: Self::Block, x: i32, y: i32, z: i32);
}

/// Render one vertical access structure.
///
/// The caller is responsible for providing the block types used by
/// the current Minecraft backend.
pub fn render_vertical_access<E: VerticalAccessEditor>(
    editor: &mut E,
    plan: &VerticalAccessPlan,
    stair_block: E::Block,
    ladder_block: E::Block,
) {
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
    block: E::Block,
) {
    if plan.upper_y <= plan.lower_y {
        return;
    }

    /*
     * Ladder occupies the existing vertical access position.
     *
     * No wall removal happens here.
     * The surrounding building geometry remains owned by
     * the original reconstruction engine.
     */
    for y in plan.lower_y..=plan.upper_y {
        for dx in 0..plan.width.max(1) {
            editor.place_block(block, plan.x + dx, y, plan.z);
        }
    }
}

fn render_stair<E: VerticalAccessEditor>(
    editor: &mut E,
    plan: &VerticalAccessPlan,
    block: E::Block,
) {
    if plan.upper_y <= plan.lower_y {
        return;
    }

    let height = plan.upper_y - plan.lower_y;
    if height <= 0 {
        return;
    }

    let length = plan.length.max(1);
    let width = plan.width.max(1);

    /*
     * Conservative non-destructive staircase geometry.
     *
     * IMPORTANT:
     * - Uses the already calculated vertical-access footprint.
     * - Does not modify walls, rooms, doors or windows.
     * - Does not change the building footprint.
     * - Y comes entirely from VerticalAccessPlan.
     *
     * The horizontal progression is distributed across the entire
     * vertical span so every stair level has a deterministic position.
     */

    for step in 0..=height {
        let y = plan.lower_y + step;

        let progress = if height == 0 {
            0
        } else {
            (step * length) / height
        };

        let z = plan.z + progress.min(length - 1);

        for dx in 0..width {
            editor.place_block(block, plan.x + dx, y, z);
        }
    }
}
