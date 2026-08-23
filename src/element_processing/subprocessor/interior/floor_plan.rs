use super::decision::{score_room, RoomAllocation, SpatialConstraints};
use super::room_type::RoomType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub min_x: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_z: i32,
}

impl Rect {
    pub fn width(self) -> i32 {
        self.max_x - self.min_x + 1
    }

    pub fn depth(self) -> i32 {
        self.max_z - self.min_z + 1
    }

    pub fn area(self) -> i32 {
        self.width() * self.depth()
    }

    pub fn center(self) -> (i32, i32) {
        ((self.min_x + self.max_x) / 2, (self.min_z + self.max_z) / 2)
    }

    pub fn contains(self, x: i32, z: i32) -> bool {
        x >= self.min_x && x <= self.max_x && z >= self.min_z && z <= self.max_z
    }

    pub fn can_fit(self, min_width: i32, min_depth: i32) -> bool {
        self.width() >= min_width && self.depth() >= min_depth
    }
}

#[derive(Debug, Clone)]
pub struct Room {
    pub room_type: RoomType,
    pub bounds: Rect,
}

#[derive(Debug, Clone)]
pub struct FloorPlan {
    pub bounds: Rect,
    pub rooms: Vec<Room>,
}

impl FloorPlan {
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            rooms: Vec::new(),
        }
    }

    pub fn total_room_area(&self) -> i32 {
        self.rooms.iter().map(|room| room.bounds.area()).sum()
    }

    pub fn is_valid(&self) -> bool {
        if self.rooms.is_empty() {
            return false;
        }

        self.rooms.iter().all(|room| {
            room.bounds.min_x >= self.bounds.min_x
                && room.bounds.max_x <= self.bounds.max_x
                && room.bounds.min_z >= self.bounds.min_z
                && room.bounds.max_z <= self.bounds.max_z
                && room.bounds.width() >= 2
                && room.bounds.depth() >= 2
        })
    }
}

/// Legacy-compatible first-pass generator.
///
/// This remains available for existing callers.
/// New intelligent generation should use
/// `generate_spatial_floor_plan`.
pub fn generate_floor_plan(bounds: Rect, rooms: &[(RoomType, i32)]) -> Option<FloorPlan> {
    if bounds.width() < 6 || bounds.depth() < 6 || rooms.is_empty() {
        return None;
    }

    let mut plan = FloorPlan::new(bounds);

    let room_count = rooms.len();

    if room_count == 1 {
        plan.rooms.push(Room {
            room_type: rooms[0].0,
            bounds,
        });

        return plan.is_valid().then_some(plan);
    }

    let split_vertical = bounds.width() >= bounds.depth();
    let mut remaining = bounds;

    for index in 0..room_count {
        let rooms_left = room_count - index;

        if rooms_left == 1 {
            let room = Room {
                room_type: rooms[index].0,
                bounds: remaining,
            };

            if !room.bounds.can_fit(2, 2) {
                return None;
            }

            plan.rooms.push(room);
            break;
        }

        let axis_size = if split_vertical {
            remaining.width()
        } else {
            remaining.depth()
        };

        let minimum_remaining = ((rooms_left - 1) as i32) * 2;

        if axis_size < minimum_remaining + 2 {
            return None;
        }

        let desired = rooms[index].1.max(4);

        let split_size = desired.min(axis_size - minimum_remaining).max(2);

        if split_vertical {
            let max_x = remaining.min_x + split_size - 1;

            let first = Rect {
                min_x: remaining.min_x,
                min_z: remaining.min_z,
                max_x,
                max_z: remaining.max_z,
            };

            let next = Rect {
                min_x: max_x + 1,
                min_z: remaining.min_z,
                max_x: remaining.max_x,
                max_z: remaining.max_z,
            };

            if !first.can_fit(2, 2) || !next.can_fit(2, 2) {
                return None;
            }

            plan.rooms.push(Room {
                room_type: rooms[index].0,
                bounds: first,
            });

            remaining = next;
        } else {
            let max_z = remaining.min_z + split_size - 1;

            let first = Rect {
                min_x: remaining.min_x,
                min_z: remaining.min_z,
                max_x: remaining.max_x,
                max_z,
            };

            let next = Rect {
                min_x: remaining.min_x,
                min_z: max_z + 1,
                max_x: remaining.max_x,
                max_z: remaining.max_z,
            };

            if !first.can_fit(2, 2) || !next.can_fit(2, 2) {
                return None;
            }

            plan.rooms.push(Room {
                room_type: rooms[index].0,
                bounds: first,
            });

            remaining = next;
        }
    }

    plan.is_valid().then_some(plan)
}

/// Spatial-aware floor-plan generation.
///
/// Unlike the legacy generator, this function evaluates multiple
/// candidate placements against:
///
/// - existing real-world windows
/// - existing real-world entrances
/// - real building bounds
/// - room minimum dimensions
/// - required area
/// - daylight requirements
/// - semantic room preferences
///
/// It never creates or modifies exterior geometry.

/// Constraint-aware floor-plan generation.
///
/// Compatibility entry point used by Building Intelligence.
/// This delegates to the spatial-aware solver.
pub fn generate_floor_plan_with_constraints(
    bounds: Rect,
    allocations: &[RoomAllocation],
    constraints: &SpatialConstraints,
    floor: i32,
) -> Option<FloorPlan> {
    generate_spatial_floor_plan(bounds, allocations, constraints, floor)
}

pub fn generate_spatial_floor_plan(
    bounds: Rect,
    allocations: &[RoomAllocation],
    constraints: &SpatialConstraints,
    floor: i32,
) -> Option<FloorPlan> {
    if bounds.width() < 6 || bounds.depth() < 6 || allocations.is_empty() {
        return None;
    }

    if constraints.bounds != bounds {
        return None;
    }

    let mut remaining = bounds;
    let mut plan = FloorPlan::new(bounds);

    let mut ordered: Vec<RoomAllocation> = allocations.to_vec();

    // Highest-priority rooms are allocated first.
    ordered.sort_by(|a, b| {
        b.priority
            .cmp(&a.priority)
            .then_with(|| b.required_area.cmp(&a.required_area))
    });

    for index in 0..ordered.len() {
        let allocation = &ordered[index];
        let rooms_left = ordered.len() - index;

        if rooms_left == 1 {
            let score = score_room(remaining, allocation, constraints, floor);

            if score == i32::MIN {
                return None;
            }

            plan.rooms.push(Room {
                room_type: allocation.room_type,
                bounds: remaining,
            });

            break;
        }

        let candidate = find_best_split(remaining, allocation, constraints, rooms_left, floor)?;

        plan.rooms.push(Room {
            room_type: allocation.room_type,
            bounds: candidate.room,
        });

        remaining = candidate.remaining;
    }

    plan.is_valid().then_some(plan)
}

#[derive(Debug, Clone, Copy)]
struct SplitCandidate {
    room: Rect,
    remaining: Rect,
    score: i32,
}

fn find_best_split(
    remaining: Rect,
    allocation: &RoomAllocation,
    constraints: &SpatialConstraints,
    rooms_left: usize,
    floor: i32,
) -> Option<SplitCandidate> {
    let mut best: Option<SplitCandidate> = None;

    let minimum_remaining = ((rooms_left - 1) as i32) * 2;

    // ---------------------------------------------------------
    // X AXIS CANDIDATES
    // ---------------------------------------------------------

    let min_x_split = allocation.min_width.max(2);

    let max_x_split = remaining.width() - minimum_remaining - 1;

    if min_x_split <= max_x_split {
        for width in min_x_split..=max_x_split {
            let max_x = remaining.min_x + width - 1;

            let room = Rect {
                min_x: remaining.min_x,
                min_z: remaining.min_z,
                max_x,
                max_z: remaining.max_z,
            };

            let next = Rect {
                min_x: max_x + 1,
                min_z: remaining.min_z,
                max_x: remaining.max_x,
                max_z: remaining.max_z,
            };

            if !next.can_fit(2, 2) {
                continue;
            }

            let score = score_room(room, allocation, constraints, floor);

            consider_candidate(
                &mut best,
                SplitCandidate {
                    room,
                    remaining: next,
                    score,
                },
            );
        }
    }

    // ---------------------------------------------------------
    // Z AXIS CANDIDATES
    // ---------------------------------------------------------

    let min_z_split = allocation.min_depth.max(2);

    let max_z_split = remaining.depth() - minimum_remaining - 1;

    if min_z_split <= max_z_split {
        for depth in min_z_split..=max_z_split {
            let max_z = remaining.min_z + depth - 1;

            let room = Rect {
                min_x: remaining.min_x,
                min_z: remaining.min_z,
                max_x: remaining.max_x,
                max_z,
            };

            let next = Rect {
                min_x: remaining.min_x,
                min_z: max_z + 1,
                max_x: remaining.max_x,
                max_z: remaining.max_z,
            };

            if !next.can_fit(2, 2) {
                continue;
            }

            let score = score_room(room, allocation, constraints, floor);

            consider_candidate(
                &mut best,
                SplitCandidate {
                    room,
                    remaining: next,
                    score,
                },
            );
        }
    }

    best
}

fn consider_candidate(best: &mut Option<SplitCandidate>, candidate: SplitCandidate) {
    if candidate.score == i32::MIN {
        return;
    }

    match best {
        None => {
            *best = Some(candidate);
        }

        Some(current) if candidate.score > current.score => {
            *best = Some(candidate);
        }

        _ => {}
    }
}
