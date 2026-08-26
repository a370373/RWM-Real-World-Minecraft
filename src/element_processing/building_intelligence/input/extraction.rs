use crate::element_processing::building_intelligence::input::{
    DoorSource, ExistingDoor, ExistingWindow, WindowSide,
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
/// This is READ-ONLY real-world reconstruction input.
///
/// Every ExistingDoor keeps:
/// - its original mapped coordinates
/// - its original OSM semantic tags
/// - the semantic source that caused it to be accepted
/// - per-door exterior evidence distances
///
/// Geometry is never created, moved, removed, or modified here.
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
        .map(|node| {
            let entrance_value = node.tags.get("entrance");
            let door_value = node.tags.get("door");

            let source = match (entrance_value.is_some(), door_value.is_some()) {
                (true, true) => DoorSource::EntranceAndDoor,
                (true, false) => DoorSource::Entrance,
                (false, true) => DoorSource::Door,
                (false, false) => unreachable!("door filter guarantees entrance or door"),
            };

            let tags = node
                .tags
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            ExistingDoor {
                x: node.x,
                z: node.z,
                width: node
                    .tags
                    .get("door:width")
                    .or_else(|| node.tags.get("width"))
                    .and_then(|v| v.trim().parse::<f32>().ok())
                    .map(|v| v.round() as i32)
                    .unwrap_or(1)
                    .clamp(1, 16),

                side: infer_side(node.x, node.z, min_x, min_z, max_x, max_z),

                tags,
                source,

                // Evidence distances are populated by the building
                // reconstruction layer once global masks are available.
                footway_distance: None,
                road_distance: None,
                parking_distance: None,
            }
        })
        .collect()
}

/// Calculate distance from a door to the existing RWM road mask.
///
/// This reuses the existing road reconstruction mask and does not
/// modify or regenerate any road geometry.
pub fn nearest_road_distance(
    x: i32,
    z: i32,
    road_mask: &crate::floodfill_cache::RoadMaskBitmap,
    max_radius: i32,
) -> Option<i32> {
    crate::element_processing::get_nearest_road_block(x, z, max_radius, road_mask)
        .map(|(rx, rz)| (rx - x).abs() + (rz - z).abs())
}

/// Calculate Manhattan distance from a coordinate to a set of world coordinates.
fn nearest_distance(
    x: i32,
    z: i32,
    mask: Option<&std::collections::HashSet<(i32, i32)>>,
) -> Option<i32> {
    let mask = mask?;

    mask.iter()
        .map(|&(mx, mz)| (mx - x).abs() + (mz - z).abs())
        .min()
}

/// Attach real-world exterior evidence distances to existing doors.
///
/// This function is READ-ONLY with respect to world geometry.
/// It only enriches the door decision input model.
pub fn enrich_existing_door_evidence(
    doors: &mut [ExistingDoor],
    footway_mask: Option<&std::collections::HashSet<(i32, i32)>>,
    road_mask: Option<&std::collections::HashSet<(i32, i32)>>,
    parking_mask: Option<&std::collections::HashSet<(i32, i32)>>,
) {
    for door in doors.iter_mut() {
        door.footway_distance = nearest_distance(door.x, door.z, footway_mask);

        door.road_distance = nearest_distance(door.x, door.z, road_mask);

        door.parking_distance = nearest_distance(door.x, door.z, parking_mask);
    }
}

/// Returns true when a processed way represents pedestrian access.
pub fn is_footway_way(way: &crate::osm_parser::ProcessedWay) -> bool {
    match way.tags.get("highway").map(String::as_str) {
        Some("footway")
        | Some("pedestrian")
        | Some("path")
        | Some("steps")
        | Some("living_street") => true,
        _ => false,
    }
}

/// Returns true when a processed way represents parking.
pub fn is_parking_way(way: &crate::osm_parser::ProcessedWay) -> bool {
    way.tags.get("amenity").map(String::as_str) == Some("parking")
}
