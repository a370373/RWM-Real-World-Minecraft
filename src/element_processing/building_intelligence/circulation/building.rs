use std::collections::{HashMap, HashSet, VecDeque};

use super::super::room_graph::RoomGraph;
use super::doorways::DoorwayPlan;

#[derive(Debug, Clone, Default)]
pub struct BuildingCirculationPlan {
    pub entrance_room: Option<usize>,
    pub reachable_rooms: Vec<usize>,
    pub isolated_rooms: Vec<usize>,
    pub route: Vec<usize>,
}

impl BuildingCirculationPlan {
    pub fn is_connected(&self) -> bool {
        self.isolated_rooms.is_empty()
    }
}

/// Validate whole-building circulation starting from the already-detected
/// real-world main entrance.
///
/// This stage only reads:
/// - RoomGraph
/// - existing doorway decisions
///
/// It never modifies rooms, doors, windows, exterior geometry, or BBox.
pub fn build_building_circulation(
    graph: &RoomGraph,
    doorway_plan: &DoorwayPlan,
) -> BuildingCirculationPlan {
    let Some(start) = graph.entrance_room else {
        return BuildingCirculationPlan {
            entrance_room: None,
            reachable_rooms: Vec::new(),
            isolated_rooms: graph.rooms.iter().map(|room| room.id).collect(),
            route: Vec::new(),
        };
    };

    if graph.room(start).is_none() {
        return BuildingCirculationPlan {
            entrance_room: Some(start),
            reachable_rooms: Vec::new(),
            isolated_rooms: graph.rooms.iter().map(|room| room.id).collect(),
            route: Vec::new(),
        };
    }

    let mut adjacency: HashMap<usize, Vec<usize>> = HashMap::new();

    for door in &doorway_plan.doors {
        adjacency
            .entry(door.from_room)
            .or_default()
            .push(door.to_room);

        adjacency
            .entry(door.to_room)
            .or_default()
            .push(door.from_room);
    }

    let mut visited = HashSet::new();
    let mut previous: HashMap<usize, usize> = HashMap::new();
    let mut queue = VecDeque::new();

    visited.insert(start);
    queue.push_back(start);

    while let Some(room) = queue.pop_front() {
        if let Some(neighbours) = adjacency.get(&room) {
            for &next in neighbours {
                if visited.insert(next) {
                    previous.insert(next, room);
                    queue.push_back(next);
                }
            }
        }
    }

    let reachable_rooms = graph
        .rooms
        .iter()
        .filter(|room| visited.contains(&room.id))
        .map(|room| room.id)
        .collect::<Vec<_>>();

    let isolated_rooms = graph
        .rooms
        .iter()
        .filter(|room| !visited.contains(&room.id))
        .map(|room| room.id)
        .collect::<Vec<_>>();

    // Prefer the first non-entrance room as a deterministic validation route.
    let target = reachable_rooms.iter().copied().find(|&room| room != start);

    let mut route = Vec::new();

    if let Some(target) = target {
        let mut cursor = target;
        route.push(cursor);

        while cursor != start {
            let Some(parent) = previous.get(&cursor).copied() else {
                route.clear();
                break;
            };

            cursor = parent;
            route.push(cursor);
        }

        route.reverse();
    }

    BuildingCirculationPlan {
        entrance_room: Some(start),
        reachable_rooms,
        isolated_rooms,
        route,
    }
}
