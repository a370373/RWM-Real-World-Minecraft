use crate::element_processing::building_intelligence::FurnitureItem;
use crate::element_processing::subprocessor::interior::{FloorPlan, Rect};

use super::super::room_graph::RoomGraph;
use super::doorways::{DoorOrientation, DoorwayPlan};
use super::furniture_clearance::build_furniture_obstacles;
use super::pathfinding::{find_interior_path, InteriorPath, WalkCell};

#[derive(Debug, Clone)]
pub struct RoomCirculation {
    pub room_id: usize,
    pub entrance: WalkCell,
    pub target: WalkCell,
    pub path: InteriorPath,
    pub reachable: bool,
}

#[derive(Debug, Clone, Default)]
pub struct InteriorCirculationPlan {
    pub rooms: Vec<RoomCirculation>,
    pub all_rooms_reachable: bool,
}

impl InteriorCirculationPlan {
    pub fn reachable_rooms(&self) -> usize {
        self.rooms.iter().filter(|room| room.reachable).count()
    }

    pub fn unreachable_rooms(&self) -> usize {
        self.rooms.iter().filter(|room| !room.reachable).count()
    }
}

/// Select a safe interior target point.
///
/// The target is semantic only: it does not alter the room geometry.
fn room_target(room: Rect) -> WalkCell {
    WalkCell {
        x: (room.min_x + room.max_x) / 2,
        z: (room.min_z + room.max_z) / 2,
    }
}

/// Convert a doorway position on the room boundary into a walkable
/// cell just inside the room.
///
/// This reads existing doorway intent only.
/// It never modifies the doorway or room geometry.
fn doorway_entry(room: Rect, door_x: i32, door_z: i32, orientation: DoorOrientation) -> WalkCell {
    match orientation {
        DoorOrientation::VerticalWall => {
            if door_x == room.min_x {
                WalkCell {
                    x: (room.min_x + 1).min(room.max_x),
                    z: door_z.clamp(room.min_z, room.max_z),
                }
            } else if door_x == room.max_x {
                WalkCell {
                    x: (room.max_x - 1).max(room.min_x),
                    z: door_z.clamp(room.min_z, room.max_z),
                }
            } else {
                WalkCell {
                    x: door_x.clamp(room.min_x, room.max_x),
                    z: door_z.clamp(room.min_z, room.max_z),
                }
            }
        }

        DoorOrientation::HorizontalWall => {
            if door_z == room.min_z {
                WalkCell {
                    x: door_x.clamp(room.min_x, room.max_x),
                    z: (room.min_z + 1).min(room.max_z),
                }
            } else if door_z == room.max_z {
                WalkCell {
                    x: door_x.clamp(room.min_x, room.max_x),
                    z: (room.max_z - 1).max(room.min_z),
                }
            } else {
                WalkCell {
                    x: door_x.clamp(room.min_x, room.max_x),
                    z: door_z.clamp(room.min_z, room.max_z),
                }
            }
        }
    }
}

/// Find the floor-plan room corresponding to a global RoomGraph id.
fn room_bounds(graph: &RoomGraph, floor_plans: &[FloorPlan], room_id: usize) -> Option<Rect> {
    let node = graph.room(room_id)?;
    let floor_plan = floor_plans.get(node.floor)?;

    floor_plan
        .rooms
        .get(node.floor_room_index)
        .map(|room| room.bounds)
}

/// Find the first physical doorway connected to this room.
fn room_doorway(doorway_plan: &DoorwayPlan, room_id: usize) -> Option<(i32, i32, DoorOrientation)> {
    doorway_plan
        .doors
        .iter()
        .find(|door| door.from_room == room_id || door.to_room == room_id)
        .map(|door| (door.x, door.z, door.orientation))
}

/// Build circulation intent for an already-generated building.
///
/// This stage only reads existing room geometry, doorway intent and
/// furniture intent, then computes circulation paths.
///
/// It does NOT modify:
/// - room geometry
/// - furniture placement
/// - doors
/// - windows
/// - building footprint
/// - BBox
pub fn build_floor_circulation(
    graph: &RoomGraph,
    doorway_plan: &DoorwayPlan,
    floor_index: usize,
    floor_plan: &FloorPlan,
    furniture: &[FurnitureItem],
) -> InteriorCirculationPlan {
    let all_obstacles = build_furniture_obstacles(furniture);

    let mut result = InteriorCirculationPlan::default();

    for (floor_room_index, room) in floor_plan.rooms.iter().enumerate() {
        let Some(room_id) = graph
            .rooms
            .iter()
            .find(|node| node.floor == floor_index && node.floor_room_index == floor_room_index)
            .map(|node| node.id)
        else {
            result.rooms.push(RoomCirculation {
                room_id: floor_room_index,
                entrance: room_target(room.bounds),
                target: room_target(room.bounds),
                path: InteriorPath::default(),
                reachable: false,
            });
            continue;
        };

        let entry = room_doorway(doorway_plan, room_id)
            .map(|(x, z, orientation)| doorway_entry(room.bounds, x, z, orientation))
            .unwrap_or_else(|| room_target(room.bounds));

        let target = room_target(room.bounds);

        // FurnitureItem.room_id uses the RoomGraph room id.
        // Therefore obstacle filtering must use room_id, not the
        // local floor-room index.
        let obstacles = all_obstacles
            .iter()
            .filter(|obstacle| obstacle.room_id == room_id)
            .copied()
            .collect::<Vec<_>>();

        let path = find_interior_path(room.bounds, entry, target, &obstacles);

        result.rooms.push(RoomCirculation {
            room_id,
            entrance: entry,
            target,
            reachable: !path.is_empty(),
            path,
        });
    }

    result.all_rooms_reachable =
        !result.rooms.is_empty() && result.rooms.iter().all(|room| room.reachable);

    result
}
