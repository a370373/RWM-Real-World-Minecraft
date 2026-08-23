use crate::element_processing::building_intelligence::input::{
    ExistingDoor, ExistingWindow, WindowSide,
};
use crate::osm_parser::ProcessedNode;

/// Returns true when a node carries explicit real-world window semantics.
fn is_window_node(node: &ProcessedNode) -> bool {
    node.tags.contains_key("window")
        || node.tags.contains_key("window:type")
        || node.tags.contains_key("window:material")
        || node.tags.contains_key("window:orientation")
        || node.tags.contains_key("building:window")
        || node.tags.contains_key("building:windows")
        || node.tags.contains_key("glazing")
        || node.tags.get("natural").map(String::as_str) == Some("window")
}

/// Infer which facade side a node belongs to.
fn infer_side(x: i32, z: i32, min_x: i32, min_z: i32, max_x: i32, max_z: i32) -> WindowSide {
    let north = (z - min_z).abs();
    let south = (z - max_z).abs();
    let west = (x - min_x).abs();
    let east = (x - max_x).abs();

    [
        (WindowSide::North, north),
        (WindowSide::South, south),
        (WindowSide::West, west),
        (WindowSide::East, east),
    ]
    .into_iter()
    .min_by_key(|(_, distance)| *distance)
    .map(|(side, _)| side)
    .unwrap_or(WindowSide::North)
}

/// Extract explicitly mapped real-world windows.
///
/// This function is READ-ONLY:
/// it only interprets ProcessedNode tags and never modifies geometry.
pub fn collect_existing_windows(
    nodes: &[ProcessedNode],
    min_x: i32,
    min_z: i32,
    max_x: i32,
    max_z: i32,
) -> Vec<ExistingWindow> {
    nodes
        .iter()
        .filter(|node| is_window_node(node))
        .map(|node| {
            // Prefer explicit real-world window dimensions when available.
            // Values are interpreted as blocks/metres already converted by
            // the surrounding RWM coordinate pipeline.
            let width = node
                .tags
                .get("window:width")
                .or_else(|| node.tags.get("width"))
                .and_then(|v| v.trim().parse::<f32>().ok())
                .map(|v| v.round() as i32)
                .unwrap_or(1)
                .clamp(1, 16);

            let height = node
                .tags
                .get("window:height")
                .or_else(|| node.tags.get("height"))
                .and_then(|v| v.trim().parse::<f32>().ok())
                .map(|v| v.round() as i32)
                .unwrap_or(1)
                .clamp(1, 16);

            ExistingWindow {
                x: node.x,
                z: node.z,
                width,
                height,
                floor: node
                    .tags
                    .get("level")
                    .and_then(|v| v.trim().parse::<i32>().ok())
                    .unwrap_or(0),
                side: infer_side(node.x, node.z, min_x, min_z, max_x, max_z),
            }
        })
        .collect()
}

/// Extract explicitly mapped real-world doors.
///
/// Kept here beside window extraction so BuildingSnapshot has one
/// deterministic read-only input pipeline for facade openings.
pub fn collect_existing_doors(
    nodes: &[ProcessedNode],
    min_x: i32,
    min_z: i32,
    max_x: i32,
    max_z: i32,
) -> Vec<ExistingDoor> {
    nodes
        .iter()
        .filter(|node| node.tags.contains_key("entrance") || node.tags.contains_key("door"))
        .map(|node| ExistingDoor {
            x: node.x,
            z: node.z,
            width: 1,
            side: infer_side(node.x, node.z, min_x, min_z, max_x, max_z),
        })
        .collect()
}
