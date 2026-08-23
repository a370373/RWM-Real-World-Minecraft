use super::super::room_graph::{RoomConnectionKind, RoomGraph};
use crate::element_processing::building_intelligence::decision::DoorKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorOrientation {
    VerticalWall,
    HorizontalWall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InteriorDoor {
    pub from_room: usize,
    pub to_room: usize,
    pub x: i32,
    pub z: i32,
    pub width: i32,
    pub kind: RoomConnectionKind,
    /// Semantic door type from BuildingDecision.
    pub door_kind: DoorKind,
    pub orientation: DoorOrientation,
}

#[derive(Debug, Clone, Default)]
pub struct DoorwayPlan {
    pub doors: Vec<InteriorDoor>,
    pub main_entrance:
        Option<crate::element_processing::building_intelligence::decision::DoorDecision>,
}

fn room_bounds(
    graph: &RoomGraph,
    floor_plans: &[crate::element_processing::subprocessor::interior::FloorPlan],
    room_id: usize,
) -> Option<crate::element_processing::subprocessor::interior::Rect> {
    let node = graph.room(room_id)?;

    floor_plans
        .get(node.floor)?
        .rooms
        .iter()
        .filter(|room| room.room_type == node.room_type)
        .nth(
            graph
                .rooms
                .iter()
                .filter(|other| other.floor == node.floor && other.room_type == node.room_type)
                .position(|other| other.id == room_id)?,
        )
        .map(|room| room.bounds)
}

fn shared_wall_door(
    a: crate::element_processing::subprocessor::interior::Rect,
    b: crate::element_processing::subprocessor::interior::Rect,
    requested_width: i32,
) -> Option<(i32, i32, i32, DoorOrientation)> {
    let requested_width = requested_width.max(1);

    // A east wall touches B west wall.
    if a.max_x + 1 == b.min_x {
        let min_z = a.min_z.max(b.min_z);
        let max_z = a.max_z.min(b.max_z);

        if min_z <= max_z {
            let available = max_z - min_z + 1;
            let width = requested_width.min(available);
            let start = min_z + (available - width) / 2;
            let end = start + width - 1;

            return Some((
                a.max_x,
                start + (end - start) / 2,
                width,
                DoorOrientation::VerticalWall,
            ));
        }
    }

    // A west wall touches B east wall.
    if b.max_x + 1 == a.min_x {
        let min_z = a.min_z.max(b.min_z);
        let max_z = a.max_z.min(b.max_z);

        if min_z <= max_z {
            let available = max_z - min_z + 1;
            let width = requested_width.min(available);
            let start = min_z + (available - width) / 2;
            let end = start + width - 1;

            return Some((
                a.min_x,
                start + (end - start) / 2,
                width,
                DoorOrientation::VerticalWall,
            ));
        }
    }

    // A south wall touches B north wall.
    if a.max_z + 1 == b.min_z {
        let min_x = a.min_x.max(b.min_x);
        let max_x = a.max_x.min(b.max_x);

        if min_x <= max_x {
            let available = max_x - min_x + 1;
            let width = requested_width.min(available);
            let start = min_x + (available - width) / 2;
            let end = start + width - 1;

            return Some((
                start + (end - start) / 2,
                a.max_z,
                width,
                DoorOrientation::HorizontalWall,
            ));
        }
    }

    // A north wall touches B south wall.
    if b.max_z + 1 == a.min_z {
        let min_x = a.min_x.max(b.min_x);
        let max_x = a.max_x.min(b.max_x);

        if min_x <= max_x {
            let available = max_x - min_x + 1;
            let width = requested_width.min(available);
            let start = min_x + (available - width) / 2;
            let end = start + width - 1;

            return Some((
                start + (end - start) / 2,
                a.min_z,
                width,
                DoorOrientation::HorizontalWall,
            ));
        }
    }

    None
}

/// Converts semantic RoomGraph connections into physical interior doorway intents.
///
/// This function NEVER changes room geometry.
/// It only searches for a shared boundary between two already-generated rooms.
pub fn build_doorway_plan(
    graph: &RoomGraph,
    floor_plans: &[crate::element_processing::subprocessor::interior::FloorPlan],
) -> DoorwayPlan {
    let mut plan = DoorwayPlan::default();

    for connection in &graph.connections {
        if connection.kind == RoomConnectionKind::VerticalAccess {
            continue;
        }

        let Some(a) = room_bounds(graph, floor_plans, connection.from) else {
            continue;
        };

        let Some(b) = room_bounds(graph, floor_plans, connection.to) else {
            continue;
        };

        let Some((x, z, width, orientation)) = shared_wall_door(a, b, connection.preferred_width)
        else {
            continue;
        };

        plan.doors.push(InteriorDoor {
            from_room: connection.from,
            to_room: connection.to,
            x,
            z,
            width,
            kind: connection.kind,
            door_kind: connection.door_kind,
            orientation,
        });
    }

    plan
}
