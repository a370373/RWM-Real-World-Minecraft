use crate::args::OutsideTerrain;
use crate::block_definitions::{DIRT, GRASS_BLOCK, SAND, SANDSTONE, SNOW_BLOCK, STONE, WATER};
use crate::coordinate_system::cartesian::{XZBBox, XZPoint};
use crate::ground::Ground;
use crate::world_editor::WorldEditor;

/// Generate configurable deterministic terrain outside the authoritative RWM bbox.
///
/// The generated area is:
///   bbox expanded by `padding` blocks
///
/// The authoritative RWM area is NEVER overwritten.
///
/// Supported terrain:
///   Void       -> completely empty
///   Normal     -> grass + dirt + stone
///   Ocean      -> flat sea + sand + sandstone
///   Superflat  -> grass + dirt
///   Desert     -> sand + sandstone
///   Snow       -> snow + dirt + stone
pub fn generate_outside_terrain(
    editor: &mut WorldEditor,
    ground: &Ground,
    bbox: &XZBBox,
    padding: i32,
    terrain: OutsideTerrain,
) {
    if padding <= 0 {
        return;
    }

    let min_x = bbox.min_x() - padding;
    let max_x = bbox.max_x() + padding;
    let min_z = bbox.min_z() - padding;
    let max_z = bbox.max_z() + padding;

    for x in min_x..=max_x {
        for z in min_z..=max_z {
            let point = XZPoint::new(x, z);

            // NEVER overwrite authoritative RWM terrain.
            if bbox.contains(&point) {
                continue;
            }

            match terrain {
                OutsideTerrain::Void => {
                    // Intentionally empty.
                }

                OutsideTerrain::Normal => {
                    let ground_y = ground.level(point);

                    fill_column(editor, x, z, ground_y, STONE, DIRT, GRASS_BLOCK);
                }

                OutsideTerrain::Ocean => {
                    // Keep the outside ocean at one consistent height.
                    // Align the sea surface with the RWM world's ground
                    // level instead of using local outside elevation.
                    let ocean_surface = ground.level(XZPoint::new(
                        (bbox.min_x() + bbox.max_x()) / 2,
                        (bbox.min_z() + bbox.max_z()) / 2,
                    ));

                    const OCEAN_DEPTH: i32 = 8;

                    let floor_y = ocean_surface - OCEAN_DEPTH;

                    // Ocean floor.
                    editor.fill_outside_column(x, z, floor_y - 4, floor_y - 1, SANDSTONE);

                    // Sand layer.
                    editor.set_outside_block_absolute(SAND, x, floor_y, z);

                    // Water up to the RWM-aligned sea level.
                    editor.fill_outside_column(x, z, floor_y + 1, ocean_surface, WATER);
                }

                OutsideTerrain::Superflat => {
                    let ground_y = 64;

                    // Classic simple superflat:
                    // stone 32 blocks below the surface
                    // dirt 4 blocks
                    // grass surface
                    editor.fill_outside_column(x, z, ground_y - 32, ground_y - 5, STONE);

                    editor.fill_outside_column(x, z, ground_y - 4, ground_y - 1, DIRT);

                    editor.set_outside_block_absolute(GRASS_BLOCK, x, ground_y, z);
                }

                OutsideTerrain::Desert => {
                    let ground_y = ground.level(point);

                    // Stone/sandstone foundation.
                    editor.fill_outside_column(x, z, ground_y - 8, ground_y - 5, SANDSTONE);

                    // Sand layers.
                    editor.fill_outside_column(x, z, ground_y - 4, ground_y, SAND);
                }

                OutsideTerrain::Snow => {
                    let ground_y = ground.level(point);

                    fill_column(editor, x, z, ground_y, STONE, DIRT, SNOW_BLOCK);
                }
            }
        }
    }
}

/// Normal terrain column.
fn fill_column(
    editor: &mut WorldEditor,
    x: i32,
    z: i32,
    ground_y: i32,
    deep_block: crate::block_definitions::Block,
    under_block: crate::block_definitions::Block,
    surface_block: crate::block_definitions::Block,
) {
    editor.fill_outside_column(x, z, ground_y - 4, ground_y - 2, deep_block);

    editor.set_outside_block_absolute(under_block, x, ground_y - 1, z);

    editor.set_outside_block_absolute(surface_block, x, ground_y, z);
}
