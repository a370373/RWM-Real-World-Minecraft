pub mod doorways;
use super::room_graph::{RoomConnectionKind, RoomGraph};
use super::vertical::planner::VerticalAccessPlan;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CirculationRole {
    Entrance,
    Public,
    Private,
    Service,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CirculationNode {
    pub room_id: usize,
    pub role: CirculationRole,
    pub reachable_from_entrance: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CirculationEdge {
    pub from: usize,
    pub to: usize,
    pub kind: RoomConnectionKind,
    pub preferred_width: i32,
    pub door_x: Option<i32>,
    pub door_z: Option<i32>,
}

#[derive(Debug, Clone, Default)]
pub struct CirculationPlan {
    pub entrance_room: Option<usize>,
    pub nodes: Vec<CirculationNode>,
    pub edges: Vec<CirculationEdge>,
    pub reachable_rooms: Vec<usize>,
    pub unreachable_rooms: Vec<usize>,

    /// Vertical access plans consumed from VerticalAccessPlanner.
    /// Read-only circulation intelligence; never modifies geometry.
    pub vertical_access: Vec<VerticalAccessPlan>,
}

fn role_for_room(graph: &RoomGraph, room_id: usize) -> CirculationRole {
    if graph.entrance_room == Some(room_id) {
        return CirculationRole::Entrance;
    }

    // A room participating in a real vertical-access graph edge is
    // explicitly part of vertical circulation.
    //
    // This is semantic only. It does not alter room geometry,
    // doorway geometry, floor plans, exterior geometry or BBox.
    if graph.connections.iter().any(|connection| {
        connection.kind == RoomConnectionKind::VerticalAccess
            && (connection.from == room_id || connection.to == room_id)
    }) {
        return CirculationRole::Vertical;
    }

    let room = match graph.room(room_id) {
        Some(room) => room,
        None => return CirculationRole::Public,
    };

    use crate::element_processing::subprocessor::interior::RoomType;

    match room.room_type {
        RoomType::Hallway
        | RoomType::EntranceHall
        | RoomType::LivingRoom
        | RoomType::DiningRoom
        | RoomType::DiningArea
        | RoomType::DisplayArea
        | RoomType::WaitingArea
        | RoomType::CommunityRoom
        | RoomType::Library => CirculationRole::Public,

        RoomType::Bedroom
        | RoomType::Office
        | RoomType::MeetingRoom
        | RoomType::Classroom
        | RoomType::Laboratory
        | RoomType::Ward
        | RoomType::ExaminationRoom
        | RoomType::TreatmentRoom => CirculationRole::Private,

        RoomType::Kitchen
        | RoomType::KitchenService
        | RoomType::Storage
        | RoomType::UtilityRoom
        | RoomType::Laundry
        | RoomType::ServiceArea
        | RoomType::LoadingArea
        | RoomType::Workshop
        | RoomType::ProductionArea => CirculationRole::Service,

        _ => CirculationRole::Public,
    }
}

pub fn build_circulation_plan(graph: &RoomGraph) -> CirculationPlan {
    let mut plan = CirculationPlan {
        entrance_room: graph.entrance_room,
        ..Default::default()
    };

    for room in &graph.rooms {
        plan.nodes.push(CirculationNode {
            room_id: room.id,
            role: role_for_room(graph, room.id),
            reachable_from_entrance: false,
        });
    }

    for connection in &graph.connections {
        plan.edges.push(CirculationEdge {
            from: connection.from,
            to: connection.to,
            kind: connection.kind,
            preferred_width: connection.preferred_width,
            door_x: connection.door_x,
            door_z: connection.door_z,
        });
    }

    let Some(start) = graph.entrance_room else {
        plan.unreachable_rooms = graph.rooms.iter().map(|r| r.id).collect();
        return plan;
    };

    if graph.room(start).is_none() {
        plan.unreachable_rooms = graph.rooms.iter().map(|r| r.id).collect();
        return plan;
    }

    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();

    visited.insert(start);
    queue.push_back(start);

    while let Some(current) = queue.pop_front() {
        for connection in &graph.connections {
            let next = if connection.from == current {
                connection.to
            } else if connection.to == current {
                connection.from
            } else {
                continue;
            };

            if graph.room(next).is_some() && visited.insert(next) {
                queue.push_back(next);
            }
        }
    }

    for node in &mut plan.nodes {
        node.reachable_from_entrance = visited.contains(&node.room_id);
    }

    for room in &graph.rooms {
        if visited.contains(&room.id) {
            plan.reachable_rooms.push(room.id);
        } else {
            plan.unreachable_rooms.push(room.id);
        }
    }

    plan
}

impl CirculationPlan {
    pub fn is_connected(&self) -> bool {
        self.unreachable_rooms.is_empty()
    }

    pub fn room_is_reachable(&self, room_id: usize) -> bool {
        self.reachable_rooms.contains(&room_id)
    }

    pub fn vertical_rooms(&self) -> Vec<usize> {
        self.nodes
            .iter()
            .filter(|node| node.role == CirculationRole::Vertical)
            .map(|node| node.room_id)
            .collect()
    }

    pub fn has_vertical_access(&self) -> bool {
        !self.vertical_access.is_empty()
            || self
                .nodes
                .iter()
                .any(|node| node.role == CirculationRole::Vertical)
    }

    /// Attach the already-planned vertical circulation structures.
    ///
    /// This is read-only semantic wiring:
    /// - does not modify RoomGraph
    /// - does not modify FloorPlan
    /// - does not modify room geometry
    /// - does not modify exterior geometry
    /// - does not modify BBox
    pub fn attach_vertical_access(&mut self, vertical_access: &[VerticalAccessPlan]) {
        self.vertical_access = vertical_access.to_vec();
    }

    pub fn vertical_access_plans(&self) -> &[VerticalAccessPlan] {
        &self.vertical_access
    }

    pub fn vertical_access_between(
        &self,
        from_floor: usize,
        to_floor: usize,
    ) -> Vec<&VerticalAccessPlan> {
        self.vertical_access
            .iter()
            .filter(|plan| plan.from_floor == from_floor && plan.to_floor == to_floor)
            .collect()
    }
}

pub use doorways::{build_doorway_plan, DoorOrientation, DoorwayPlan};

pub mod furniture_clearance;

pub mod pathfinding;

pub mod planner;

pub mod building;

pub use building::{build_building_circulation, BuildingCirculationPlan};
pub use planner::{build_floor_circulation, InteriorCirculationPlan};
