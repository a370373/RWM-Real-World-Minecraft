use crate::element_processing::subprocessor::interior::RoomType;

use super::decision::{DoorDecision, DoorKind};
use super::entrance::EntranceCandidate;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomConnectionKind {
    MainEntrance,
    InteriorDoor,
    ServiceConnection,
    VerticalAccess,
}

#[derive(Debug, Clone)]
pub struct RoomNode {
    pub id: usize,
    pub floor: usize,
    /// Index of the corresponding room inside FloorPlan.rooms.
    pub floor_room_index: usize,
    pub room_type: RoomType,
}

#[derive(Debug, Clone)]
pub struct RoomConnection {
    pub from: usize,
    pub to: usize,
    pub kind: RoomConnectionKind,
    /// Semantic door type carried from BuildingDecision.
    pub door_kind: DoorKind,
    pub preferred_width: i32,

    /// Preferred doorway position in Minecraft/world coordinates.
    /// This is an INTENT only. Renderer places the actual blocks.
    pub door_x: Option<i32>,
    pub door_z: Option<i32>,
}

#[derive(Debug, Clone, Default)]
pub struct RoomGraph {
    pub rooms: Vec<RoomNode>,
    pub connections: Vec<RoomConnection>,

    /// Room directly connected to the real-world main entrance.
    pub entrance_room: Option<usize>,

    /// Real-world entrance metadata.
    pub entrance: Option<EntranceCandidate>,

    /// Decision-layer metadata for the real-world main entrance.
    /// Renderer consumes this as intent; it does not modify exterior geometry.
    pub main_entrance: Option<DoorDecision>,
}

impl RoomGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_room(
        &mut self,
        floor: usize,
        floor_room_index: usize,
        room_type: RoomType,
    ) -> usize {
        let id = self.rooms.len();
        self.rooms.push(RoomNode {
            id,
            floor,
            floor_room_index,
            room_type,
        });
        id
    }

    pub fn connect(
        &mut self,
        from: usize,
        to: usize,
        kind: RoomConnectionKind,
        door_kind: DoorKind,
        preferred_width: i32,
    ) {
        if from == to {
            return;
        }

        if from >= self.rooms.len() || to >= self.rooms.len() {
            return;
        }

        let already_connected = self.connections.iter().any(|connection| {
            ((connection.from == from && connection.to == to)
                || (connection.from == to && connection.to == from))
                && connection.kind == kind
        });

        if already_connected {
            return;
        }

        self.connections.push(RoomConnection {
            from,
            to,
            kind,
            door_kind,
            preferred_width,
            door_x: None,
            door_z: None,
        });
    }

    pub fn connect_at(
        &mut self,
        from: usize,
        to: usize,
        kind: RoomConnectionKind,
        door_kind: DoorKind,
        preferred_width: i32,
        door_x: i32,
        door_z: i32,
    ) {
        if from == to {
            return;
        }

        if from >= self.rooms.len() || to >= self.rooms.len() {
            return;
        }

        let already_connected = self.connections.iter().any(|connection| {
            ((connection.from == from && connection.to == to)
                || (connection.from == to && connection.to == from))
                && connection.kind == kind
        });

        if already_connected {
            return;
        }

        self.connections.push(RoomConnection {
            from,
            to,
            kind,
            door_kind,
            preferred_width,
            door_x: Some(door_x),
            door_z: Some(door_z),
        });
    }

    pub fn set_entrance_room(&mut self, room_id: usize) {
        if room_id < self.rooms.len() {
            self.entrance_room = Some(room_id);
        }
    }

    pub fn set_entrance(&mut self, entrance: Option<EntranceCandidate>) {
        self.entrance = entrance;
    }

    pub fn set_main_entrance(&mut self, main_entrance: Option<DoorDecision>) {
        self.main_entrance = main_entrance;
    }

    pub fn main_entrance(&self) -> Option<&DoorDecision> {
        self.main_entrance.as_ref()
    }

    pub fn room(&self, id: usize) -> Option<&RoomNode> {
        self.rooms.get(id)
    }

    pub fn connections_for(&self, room_id: usize) -> Vec<&RoomConnection> {
        self.connections
            .iter()
            .filter(|connection| connection.from == room_id || connection.to == room_id)
            .collect()
    }

    pub fn rooms_of_type(&self, room_type: RoomType) -> Vec<&RoomNode> {
        self.rooms
            .iter()
            .filter(|room| room.room_type == room_type)
            .collect()
    }
}

/// Build a semantic room graph from generated floor plans.
///
/// The graph represents:
///
/// Real-world entrance
///        ↓
/// Entrance room
///        ↓
/// Public / circulation rooms
///        ↓
/// Private / service rooms
///        ↓
/// Vertical access
///
/// This stage does NOT place Minecraft blocks.
pub fn build_room_graph(
    floor_plans: &[crate::element_processing::subprocessor::interior::FloorPlan],
    entrance: Option<&EntranceCandidate>,
    entrance_decision: Option<
        &crate::element_processing::subprocessor::interior::decision::entrance_analysis::EntranceDecision
    >,
    main_door: Option<&crate::element_processing::building_intelligence::decision::DoorDecision>,
    room_door_widths: &[(RoomType, i32)],
) -> RoomGraph {
    let mut graph = RoomGraph::new();

    graph.set_entrance(entrance.copied());
    graph.set_main_entrance(main_door.copied());

    // ---------------------------------------------------------
    // 1. CREATE ROOM NODES FROM ACTUAL FLOOR-PLAN GEOMETRY
    // ---------------------------------------------------------
    //
    // The RoomGraph does not invent rooms.
    //
    // Every node corresponds directly to a Room generated by
    // the constraint-aware FloorPlan solver.
    //
    // The actual Rect remains owned by FloorPlan.
    // We only keep the room-id mapping here.
    //

    let mut floor_room_ids: Vec<Vec<usize>> = Vec::new();

    for (floor_index, plan) in floor_plans.iter().enumerate() {
        let mut ids = Vec::new();

        for (room_index, room) in plan.rooms.iter().enumerate() {
            let id = graph.add_room(floor_index, room_index, room.room_type);
            ids.push(id);
        }

        floor_room_ids.push(ids);
    }

    // ---------------------------------------------------------
    // 2. REAL-WORLD MAIN ENTRANCE -> ENTRANCE ROOM
    // ---------------------------------------------------------
    //
    // The exterior entrance comes from the already reconstructed
    // world.
    //
    // We DO NOT move it.
    // We DO NOT generate another exterior entrance.
    //
    // We only determine which interior room receives it.
    //

    if let Some(first_plan) = floor_plans.first() {
        if let Some(first_floor_ids) = floor_room_ids.first() {
            let entrance_room = entrance_decision
                .map(|decision| decision.entrance)
                .and_then(|real_entrance| {
                    first_plan
                        .rooms
                        .iter()
                        .enumerate()
                        .find(|(_, room)| room.bounds.contains(real_entrance.x, real_entrance.z))
                        .map(|(index, _)| first_floor_ids[index])
                })
                .or_else(|| {
                    entrance.and_then(|real_entrance| {
                        first_plan
                            .rooms
                            .iter()
                            .enumerate()
                            .find(|(_, room)| {
                                room.bounds.contains(real_entrance.x, real_entrance.z)
                            })
                            .map(|(index, _)| first_floor_ids[index])
                    })
                })
                .or_else(|| {
                    entrance_decision
                        .and_then(|decision| decision.preferred_room)
                        .and_then(|preferred_room| {
                            first_plan
                                .rooms
                                .iter()
                                .enumerate()
                                .find(|(_, room)| room.room_type == preferred_room)
                                .map(|(index, _)| first_floor_ids[index])
                        })
                })
                .or_else(|| {
                    first_plan
                        .rooms
                        .iter()
                        .enumerate()
                        .find(|(_, room)| {
                            matches!(
                                room.room_type,
                                RoomType::EntranceHall
                                    | RoomType::Reception
                                    | RoomType::LivingRoom
                                    | RoomType::DiningArea
                                    | RoomType::ProductArea
                                    | RoomType::DisplayArea
                                    | RoomType::Corridor
                                    | RoomType::Hallway
                                    | RoomType::PlatformArea
                            )
                        })
                        .map(|(index, _)| first_floor_ids[index])
                })
                .or_else(|| first_floor_ids.first().copied());

            if let Some(room_id) = entrance_room {
                graph.set_entrance_room(room_id);
            }
        }
    }

    // ---------------------------------------------------------
    // 3. REAL-WORLD ENTRANCE -> INTERIOR TOPOLOGY
    // ---------------------------------------------------------
    //
    // The exterior entrance is already reconstructed from real-world
    // data. We never create, move, or modify that entrance.
    //
    // This stage only connects the room receiving the real entrance
    // to the nearest suitable public/circulation room.
    //
    // The actual doorway position is derived later from the shared
    // room boundary. Room geometry remains owned by FloorPlan.
    //
    if let Some(entrance_room) = graph.entrance_room {
        let entrance_floor = graph.room(entrance_room).map(|room| room.floor);

        if let Some(floor_index) = entrance_floor {
            if let Some(plan) = floor_plans.get(floor_index) {
                let ids = &floor_room_ids[floor_index];

                let mut best: Option<(usize, i32, i32, i32)> = None;

                for (index, room) in plan.rooms.iter().enumerate() {
                    let room_id = ids[index];

                    if room_id == entrance_room {
                        continue;
                    }

                    if !(room.room_type.is_public_room()
                        || matches!(
                            room.room_type,
                            RoomType::Corridor
                                | RoomType::Hallway
                                | RoomType::EntranceHall
                                | RoomType::Reception
                                | RoomType::LivingRoom
                                | RoomType::DiningArea
                        ))
                    {
                        continue;
                    }

                    let Some(entrance_rect) = graph.room(entrance_room).and_then(|node| {
                        if node.floor == floor_index {
                            plan.rooms.get(node.floor_room_index).map(|r| r.bounds)
                        } else {
                            None
                        }
                    }) else {
                        continue;
                    };

                    let Some((door_x, door_z)) =
                        shared_wall_door_position(entrance_rect, room.bounds)
                    else {
                        continue;
                    };

                    let (cx, cz) = graph
                        .room(entrance_room)
                        .and_then(|node| {
                            if node.floor == floor_index {
                                plan.rooms.get(node.floor_room_index).map(|r| r.bounds)
                            } else {
                                None
                            }
                        })
                        .map(|bounds| bounds.center())
                        .unwrap_or_else(|| room.bounds.center());

                    let (ox, oz) = room.bounds.center();
                    let distance = (cx - ox).abs() + (cz - oz).abs();

                    if best
                        .as_ref()
                        .map(|(_, _, _, current)| distance < *current)
                        .unwrap_or(true)
                    {
                        best = Some((room_id, door_x, door_z, distance));
                    }
                }

                if let Some((to, door_x, door_z, _)) = best {
                    let width = room_door_widths
                        .iter()
                        .find(|(kind, _)| *kind == RoomType::EntranceHall)
                        .map(|(_, width)| *width)
                        .unwrap_or(2);

                    graph.connect_at(
                        entrance_room,
                        to,
                        RoomConnectionKind::MainEntrance,
                        DoorKind::MainEntrance,
                        width,
                        door_x,
                        door_z,
                    );
                }
            }
        }
    }

    // ---------------------------------------------------------
    // 3. SAME-FLOOR TOPOLOGY
    // ---------------------------------------------------------
    //
    // Rooms are connected ONLY when their actual Rects touch.
    //
    // This is important:
    //
    //     Room A | Room B
    //            ^
    //        shared wall
    //
    // becomes:
    //
    //     Room A ---- InteriorDoor ---- Room B
    //
    // No arbitrary room-to-room teleport-style connection.
    //

    for (floor_index, plan) in floor_plans.iter().enumerate() {
        let ids = &floor_room_ids[floor_index];

        for i in 0..plan.rooms.len() {
            for j in (i + 1)..plan.rooms.len() {
                let a = &plan.rooms[i];
                let b = &plan.rooms[j];

                if let Some((door_x, door_z)) = shared_wall_door_position(a.bounds, b.bounds) {
                    let from = ids[i];
                    let to = ids[j];

                    let room_type = b.room_type;

                    let width = room_door_widths
                        .iter()
                        .find(|(kind, _)| *kind == room_type)
                        .map(|(_, width)| *width)
                        .unwrap_or_else(|| default_door_width(room_type));

                    graph.connect_at(
                        from,
                        to,
                        if room_type.is_service_room() {
                            RoomConnectionKind::ServiceConnection
                        } else {
                            RoomConnectionKind::InteriorDoor
                        },
                        if room_type.is_service_room() {
                            DoorKind::Service
                        } else {
                            DoorKind::Interior
                        },
                        width,
                        door_x,
                        door_z,
                    );
                }
            }
        }
    }

    // ---------------------------------------------------------
    // 4. SERVICE-ROOM TOPOLOGY
    // ---------------------------------------------------------
    //
    // Service rooms should preferentially connect to a nearby
    // public/circulation room.
    //
    // We do NOT create a connection unless the two rooms actually
    // share a wall.
    //

    for (floor_index, plan) in floor_plans.iter().enumerate() {
        let ids = &floor_room_ids[floor_index];

        for i in 0..plan.rooms.len() {
            let room = &plan.rooms[i];

            if !room.room_type.is_service_room() {
                continue;
            }

            let mut best: Option<(usize, i32, i32, i32)> = None;

            for j in 0..plan.rooms.len() {
                if i == j {
                    continue;
                }

                let other = &plan.rooms[j];

                if !(other.room_type.is_public_room()
                    || matches!(
                        other.room_type,
                        RoomType::Corridor | RoomType::Hallway | RoomType::EntranceHall
                    ))
                {
                    continue;
                }

                let Some((door_x, door_z)) = shared_wall_door_position(room.bounds, other.bounds)
                else {
                    continue;
                };

                let (cx, cz) = room.bounds.center();
                let (ox, oz) = other.bounds.center();

                let distance = (cx - ox).abs() + (cz - oz).abs();

                if best
                    .as_ref()
                    .map(|(_, _, _, current_distance)| distance < *current_distance)
                    .unwrap_or(true)
                {
                    best = Some((j, door_x, door_z, distance));
                }
            }

            if let Some((j, door_x, door_z, _)) = best {
                let from = ids[j];
                let to = ids[i];

                let width = room_door_widths
                    .iter()
                    .find(|(kind, _)| *kind == room.room_type)
                    .map(|(_, width)| *width)
                    .unwrap_or_else(|| default_door_width(room.room_type));

                graph.connect_at(
                    from,
                    to,
                    RoomConnectionKind::ServiceConnection,
                    DoorKind::Service,
                    width,
                    door_x,
                    door_z,
                );
            }
        }
    }

    // ---------------------------------------------------------
    // 5. VERTICAL TOPOLOGY
    // ---------------------------------------------------------
    //
    // The graph records the need for vertical circulation.
    //
    // Actual staircase/ladder blocks are NOT placed here.
    // VerticalAccessPlanner / Renderer handles that later.
    //

    for floor_index in 0..floor_room_ids.len().saturating_sub(1) {
        let lower_ids = &floor_room_ids[floor_index];
        let upper_ids = &floor_room_ids[floor_index + 1];

        let Some(lower_plan) = floor_plans.get(floor_index) else {
            continue;
        };

        let Some(upper_plan) = floor_plans.get(floor_index + 1) else {
            continue;
        };

        /*
         * ---------------------------------------------------------
         * VERTICAL ACCESS CANDIDATE SELECTION
         * ---------------------------------------------------------
         *
         * Multiple rooms on adjacent floors may overlap in X/Z.
         *
         * That overlap only means that a vertical connection is
         * geometrically possible. It does NOT mean that every pair
         * of rooms needs its own staircase.
         *
         * Collect all valid candidates first, then select the
         * closest lower/upper room pair for this floor transition.
         *
         * Result:
         *
         *     Floor N -> Floor N+1
         *
         * produces at most ONE VerticalAccess connection.
         *
         * The actual physical stair footprint is still decided by
         * VerticalAccessPlanner.
         */
        let mut candidates: Vec<(i32, usize, usize)> = Vec::new();

        for (lower_index, &lower_id) in lower_ids.iter().enumerate() {
            let Some(lower_room) = lower_plan.rooms.get(lower_index) else {
                continue;
            };

            for (upper_index, &upper_id) in upper_ids.iter().enumerate() {
                let Some(upper_room) = upper_plan.rooms.get(upper_index) else {
                    continue;
                };

                if !rects_overlap(lower_room.bounds, upper_room.bounds) {
                    continue;
                }

                let (lower_x, lower_z) = lower_room.bounds.center();
                let (upper_x, upper_z) = upper_room.bounds.center();

                /*
                 * Prefer the pair whose room centers are closest.
                 *
                 * This keeps the vertical connection inside the
                 * existing floor-plan geometry instead of creating
                 * arbitrary cross-building vertical links.
                 */
                let distance = (lower_x - upper_x).abs() + (lower_z - upper_z).abs();

                candidates.push((distance, lower_id, upper_id));
            }
        }

        /*
         * Choose exactly one best candidate for this floor
         * transition.
         *
         * If there is no geometrically overlapping room pair,
         * no vertical connection is created.
         */
        if let Some((_, lower_id, upper_id)) =
            candidates.into_iter().min_by_key(|candidate| candidate.0)
        {
            graph.connect(
                lower_id,
                upper_id,
                RoomConnectionKind::VerticalAccess,
                DoorKind::Interior,
                2,
            );
        }
    }

    graph
}

// -------------------------------------------------------------
// SHARED-WALL DOOR SOLVER
// -------------------------------------------------------------
//
// Returns a preferred doorway coordinate when two room rectangles
// share a real wall.
//
// This is INTENT only.
// Renderer decides the actual wall/door blocks later.
//

fn shared_wall_door_position(
    a: crate::element_processing::subprocessor::interior::Rect,
    b: crate::element_processing::subprocessor::interior::Rect,
) -> Option<(i32, i32)> {
    // A is immediately west of B.
    if a.max_x + 1 == b.min_x {
        let min_z = a.min_z.max(b.min_z);
        let max_z = a.max_z.min(b.max_z);

        if min_z <= max_z {
            return Some((b.min_x, (min_z + max_z) / 2));
        }
    }

    // B is immediately west of A.
    if b.max_x + 1 == a.min_x {
        let min_z = a.min_z.max(b.min_z);
        let max_z = a.max_z.min(b.max_z);

        if min_z <= max_z {
            return Some((a.min_x, (min_z + max_z) / 2));
        }
    }

    // A is immediately north of B.
    if a.max_z + 1 == b.min_z {
        let min_x = a.min_x.max(b.min_x);
        let max_x = a.max_x.min(b.max_x);

        if min_x <= max_x {
            return Some(((min_x + max_x) / 2, b.min_z));
        }
    }

    // B is immediately north of A.
    if b.max_z + 1 == a.min_z {
        let min_x = a.min_x.max(b.min_x);
        let max_x = a.max_x.min(b.max_x);

        if min_x <= max_x {
            return Some(((min_x + max_x) / 2, a.min_z));
        }
    }

    None
}

fn rects_overlap(
    a: crate::element_processing::subprocessor::interior::Rect,
    b: crate::element_processing::subprocessor::interior::Rect,
) -> bool {
    a.min_x <= b.max_x && a.max_x >= b.min_x && a.min_z <= b.max_z && a.max_z >= b.min_z
}

fn default_door_width(room_type: RoomType) -> i32 {
    match room_type {
        RoomType::LivingRoom
        | RoomType::DiningRoom
        | RoomType::DiningArea
        | RoomType::Corridor
        | RoomType::PlatformArea => 2,

        _ => 1,
    }
}
