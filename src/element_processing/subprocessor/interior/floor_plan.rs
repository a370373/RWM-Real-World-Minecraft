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

    /// Validates semantic room topology.
    ///
    /// A room is considered topologically connected when it shares
    /// a positive-length wall segment with another room.
    ///
    /// Corner-only contact is intentionally not considered connected.
    ///
    /// This function only validates room relationships. It never
    /// modifies room geometry or reconstructed building geometry.
    pub fn is_topology_valid(&self) -> bool {
        if self.rooms.len() <= 1 {
            return true;
        }

        for i in 0..self.rooms.len() {
            let a = self.rooms[i].bounds;

            let mut has_wall_neighbor = false;

            for j in 0..self.rooms.len() {
                if i == j {
                    continue;
                }

                let b = self.rooms[j].bounds;

                // Vertical shared wall:
                //
                // a.max_x == b.min_x
                // or
                // b.max_x == a.min_x
                //
                // The Z ranges must overlap by at least one block.
                let vertical_touch = (a.max_x == b.min_x || b.max_x == a.min_x)
                    && a.min_z.max(b.min_z) <= a.max_z.min(b.max_z);

                // Horizontal shared wall:
                //
                // a.max_z == b.min_z
                // or
                // b.max_z == a.min_z
                //
                // The X ranges must overlap by at least one block.
                let horizontal_touch = (a.max_z == b.min_z || b.max_z == a.min_z)
                    && a.min_x.max(b.min_x) <= a.max_x.min(b.max_x);

                if vertical_touch || horizontal_touch {
                    has_wall_neighbor = true;
                    break;
                }
            }

            if !has_wall_neighbor {
                return false;
            }
        }

        true
    }

    pub fn is_valid(&self) -> bool {
        if self.rooms.is_empty() {
            return false;
        }

        // Every room must remain inside the reconstructed building bounds
        // and satisfy the minimum geometric size.
        if !self.rooms.iter().all(|room| {
            room.bounds.min_x >= self.bounds.min_x
                && room.bounds.max_x <= self.bounds.max_x
                && room.bounds.min_z >= self.bounds.min_z
                && room.bounds.max_z <= self.bounds.max_z
                && room.bounds.width() >= 2
                && room.bounds.depth() >= 2
        }) {
            return false;
        }

        // Rooms must never overlap.
        for i in 0..self.rooms.len() {
            for j in (i + 1)..self.rooms.len() {
                let a = self.rooms[i].bounds;
                let b = self.rooms[j].bounds;

                let overlaps_x = a.min_x <= b.max_x && b.min_x <= a.max_x;
                let overlaps_z = a.min_z <= b.max_z && b.min_z <= a.max_z;

                if overlaps_x && overlaps_z {
                    return false;
                }
            }
        }

        // =========================================================
        // ROOM TOPOLOGY VALIDATION
        // =========================================================
        //
        // Every generated room must participate in the interior
        // topology by sharing a real wall boundary with at least
        // one other room.
        //
        // Corner-only contact does NOT count as adjacency.
        //
        // This validates the semantic room topology only. It does
        // not modify the reconstructed building geometry.
        if !self.is_topology_valid() {
            return false;
        }

        // Partial-success policy:
        // A valid floor plan does not require every footprint block to be
        // assigned to a semantic room. Successfully generated rooms are
        // preserved even when a small or geometrically unusable remainder
        // cannot be converted into another room.
        //
        // Exterior building geometry remains authoritative and untouched.
        true
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

        return Some(plan);
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

    // Partial-success policy:
    // A failed semantic room allocation must never discard the
    // successfully generated interior layout.
    //
    // Any unresolved footprint is converted into a neutral Corridor
    // so the FloorPlan remains spatially complete and downstream
    // Doorway / RoomGraph / Vertical Access systems still receive
    // a valid plan.
    if remaining.area() > 0 {
        if remaining.can_fit(2, 2) {
            plan.rooms.push(Room {
                room_type: RoomType::Corridor,
                bounds: remaining,
            });
        } else {
            // Tiny unresolved fragments are intentionally absorbed
            // by the existing valid room layout rather than causing
            // the whole building interior to fail.
        }
    }

    Some(plan)
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
    // =========================================================
    // STRICT INTERIOR BOUNDARY
    // =========================================================
    //
    // The reconstructed building footprint is authoritative.
    //
    // Interior Intelligence may only operate INSIDE the
    // already reconstructed building interior.
    //
    // NEVER:
    // - expand the building
    // - modify the BBox
    // - cross exterior walls
    // - generate exterior geometry
    //
    // The effective interior is the intersection of:
    //   1. the planner bounds
    //   2. the real-world spatial constraint bounds
    //
    let interior = Rect {
        min_x: bounds.min_x.max(constraints.bounds.min_x),
        min_z: bounds.min_z.max(constraints.bounds.min_z),
        max_x: bounds.max_x.min(constraints.bounds.max_x),
        max_z: bounds.max_z.min(constraints.bounds.max_z),
    };

    if interior.width() < 6 || interior.depth() < 6 {
        return None;
    }

    if allocations.is_empty() {
        return None;
    }

    let mut plan = FloorPlan::new(interior);
    let mut remaining = interior;

    let mut ordered: Vec<RoomAllocation> = allocations.to_vec();

    ordered.sort_by(|a, b| {
        b.priority
            .cmp(&a.priority)
            .then_with(|| b.required_area.cmp(&a.required_area))
    });

    // =========================================================
    // ROOM PARTITIONING
    // =========================================================
    //
    // Rooms are carved from the existing interior footprint.
    //
    // Every generated room MUST:
    //   - remain completely inside `interior`
    //   - remain completely inside `constraints.bounds`
    //   - satisfy its minimum dimensions
    //   - never overlap another room
    //
    // The solver may fail, but failure must NEVER cause the
    // entire building to become a single unrestricted solid room.
    //
    for index in 0..ordered.len() {
        let allocation = &ordered[index];

        // -----------------------------------------------------
        // Last allocation
        // -----------------------------------------------------
        //
        // Only use the remaining area if it is a valid room.
        // This is still bounded by the strict interior.
        //
        if index + 1 == ordered.len() {
            let min_width = allocation.min_width.max(2);
            let min_depth = allocation.min_depth.max(2);

            if remaining.can_fit(min_width, min_depth) {
                plan.rooms.push(Room {
                    room_type: allocation.room_type,
                    bounds: remaining,
                });
            }

            break;
        }

        // -----------------------------------------------------
        // Semantic / spatial solver
        // -----------------------------------------------------

        if let Some(candidate) = find_best_split(
            remaining,
            allocation,
            constraints,
            &ordered[index + 1..],
            floor,
        ) {
            let room = candidate.room;

            // HARD SAFETY CHECK
            //
            // A solver result is accepted only if it is fully
            // contained by the strict interior.
            if room.min_x >= interior.min_x
                && room.max_x <= interior.max_x
                && room.min_z >= interior.min_z
                && room.max_z <= interior.max_z
                && room.width() >= allocation.min_width.max(2)
                && room.depth() >= allocation.min_depth.max(2)
            {
                plan.rooms.push(Room {
                    room_type: allocation.room_type,
                    bounds: room,
                });

                remaining = candidate.remaining;

                // Re-clamp remaining to the authoritative
                // interior so no downstream solver drift can
                // escape the reconstructed building.
                remaining = Rect {
                    min_x: remaining.min_x.max(interior.min_x),
                    min_z: remaining.min_z.max(interior.min_z),
                    max_x: remaining.max_x.min(interior.max_x),
                    max_z: remaining.max_z.min(interior.max_z),
                };

                continue;
            }
        }

        // -----------------------------------------------------
        // Deterministic fallback
        // -----------------------------------------------------
        //
        // If the semantic solver cannot find a valid split,
        // carve a smaller room from the remaining footprint.
        //
        // IMPORTANT:
        // Never fall back to the entire building footprint.
        //

        let min_width = allocation.min_width.max(2);
        let min_depth = allocation.min_depth.max(2);

        let width = remaining.width();
        let depth = remaining.depth();

        let remaining_allocations = ordered.len() - index - 1;

        let room = if width >= min_width * 2 && width >= depth {
            let desired = allocation.required_area.max(min_width * min_depth);

            let reserved_width = min_width * remaining_allocations.max(1) as i32;

            let max_room_width = (width - reserved_width).max(min_width);

            let room_width = (desired / depth).max(min_width).min(max_room_width);

            Rect {
                min_x: remaining.min_x,
                min_z: remaining.min_z,
                max_x: remaining.min_x + room_width - 1,
                max_z: remaining.max_z,
            }
        } else if depth >= min_depth * 2 {
            let desired = allocation.required_area.max(min_width * min_depth);

            let reserved_depth = min_depth * remaining_allocations.max(1) as i32;

            let max_room_depth = (depth - reserved_depth).max(min_depth);

            let room_depth = (desired / width).max(min_depth).min(max_room_depth);

            Rect {
                min_x: remaining.min_x,
                min_z: remaining.min_z,
                max_x: remaining.max_x,
                max_z: remaining.min_z + room_depth - 1,
            }
        } else {
            // Remaining area cannot safely support another room.
            continue;
        };

        // -----------------------------------------------------
        // HARD ROOM BOUNDARY CHECK
        // -----------------------------------------------------

        if room.width() < min_width
            || room.depth() < min_depth
            || room.min_x < interior.min_x
            || room.max_x > interior.max_x
            || room.min_z < interior.min_z
            || room.max_z > interior.max_z
        {
            continue;
        }

        // -----------------------------------------------------
        // NO OVERLAP CHECK
        // -----------------------------------------------------

        let overlaps_existing = plan.rooms.iter().any(|existing| {
            let a = existing.bounds;
            let b = room;

            a.min_x <= b.max_x && b.min_x <= a.max_x && a.min_z <= b.max_z && b.min_z <= a.max_z
        });

        if overlaps_existing {
            continue;
        }

        plan.rooms.push(Room {
            room_type: allocation.room_type,
            bounds: room,
        });

        // -----------------------------------------------------
        // Update remaining space
        // -----------------------------------------------------

        remaining = if room.max_x < remaining.max_x {
            Rect {
                min_x: room.max_x + 1,
                min_z: remaining.min_z,
                max_x: remaining.max_x,
                max_z: remaining.max_z,
            }
        } else if room.max_z < remaining.max_z {
            Rect {
                min_x: remaining.min_x,
                min_z: room.max_z + 1,
                max_x: remaining.max_x,
                max_z: remaining.max_z,
            }
        } else {
            // Nothing usable remains.
            Rect {
                min_x: 1,
                min_z: 1,
                max_x: 0,
                max_z: 0,
            }
        };

        // Absolute clamp.
        remaining = Rect {
            min_x: remaining.min_x.max(interior.min_x),
            min_z: remaining.min_z.max(interior.min_z),
            max_x: remaining.max_x.min(interior.max_x),
            max_z: remaining.max_z.min(interior.max_z),
        };
    }

    // =========================================================
    // STRICT VALIDATION
    // =========================================================
    //
    // Every room must be inside the reconstructed interior.
    // Any invalid room is discarded.
    //
    plan.rooms.retain(|room| {
        let r = room.bounds;

        r.min_x >= interior.min_x
            && r.max_x <= interior.max_x
            && r.min_z >= interior.min_z
            && r.max_z <= interior.max_z
            && r.width() >= 2
            && r.depth() >= 2
    });

    // =========================================================
    // NO SOLID-BUILDING FALLBACK
    // =========================================================
    //
    // DO NOT create:
    //
    //     Room { bounds: interior }
    //
    // merely because the solver failed.
    //
    // A failed room plan is preferable to turning the whole
    // reconstructed building into one artificial solid room.
    //

    if plan.rooms.is_empty() {
        return None;
    }

    // =========================================================
    // FINAL OVERLAP VALIDATION
    // =========================================================

    for i in 0..plan.rooms.len() {
        for j in (i + 1)..plan.rooms.len() {
            let a = plan.rooms[i].bounds;
            let b = plan.rooms[j].bounds;

            let overlaps_x = a.min_x <= b.max_x && b.min_x <= a.max_x;

            let overlaps_z = a.min_z <= b.max_z && b.min_z <= a.max_z;

            if overlaps_x && overlaps_z {
                return None;
            }
        }
    }

    // =========================================================
    // FINAL FLOOR PLAN VALIDATION
    // =========================================================

    if !plan.is_valid() {
        return None;
    }

    println!(
        "[BI FLOOR] floor={} interior=({},{})-({},{}) rooms={} area={}",
        floor,
        interior.min_x,
        interior.min_z,
        interior.max_x,
        interior.max_z,
        plan.rooms.len(),
        plan.total_room_area()
    );

    Some(plan)
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
    remaining_allocations: &[RoomAllocation],
    floor: i32,
) -> Option<SplitCandidate> {
    let mut best: Option<SplitCandidate> = None;

    // ---------------------------------------------------------
    // X AXIS CANDIDATES
    // ---------------------------------------------------------

    let min_x_split = allocation.min_width.max(2);

    let max_x_split = remaining.width() - 2;

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

            if !can_reserve_remaining_space(next, remaining_allocations) {
                continue;
            }

            let base_score = score_room(room, allocation, constraints, floor);
            let topology_score = topology_candidate_score(room, next);
            let score = base_score + topology_score;

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

    let max_z_split = remaining.depth() - 2;

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

            if !can_reserve_remaining_space(next, remaining_allocations) {
                continue;
            }

            let base_score = score_room(room, allocation, constraints, floor);
            let topology_score = topology_candidate_score(room, next);
            let score = base_score + topology_score;

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

/// Scores a candidate room according to its topology with the
/// remaining interior region.
///
/// Topology is a preference during candidate selection, not a replacement
/// for the existing spatial constraints or semantic room score.
///
/// The function never modifies room geometry.
fn topology_candidate_score(room: Rect, remaining: Rect) -> i32 {
    let mut score = 0;

    // Shared vertical boundary.
    let vertical_touch = (room.max_x + 1 == remaining.min_x || remaining.max_x + 1 == room.min_x)
        && room.min_z.max(remaining.min_z) <= room.max_z.min(remaining.max_z);

    // Shared horizontal boundary.
    let horizontal_touch = (room.max_z + 1 == remaining.min_z || remaining.max_z + 1 == room.min_z)
        && room.min_x.max(remaining.min_x) <= room.max_x.min(remaining.max_x);

    if vertical_touch {
        score += 100;

        let shared_length = room.max_z.min(remaining.max_z) - room.min_z.max(remaining.min_z) + 1;

        score += shared_length.min(20);
    }

    if horizontal_touch {
        score += 100;

        let shared_length = room.max_x.min(remaining.max_x) - room.min_x.max(remaining.min_x) + 1;

        score += shared_length.min(20);
    }

    score
}

fn can_reserve_remaining_space(remaining: Rect, allocations: &[RoomAllocation]) -> bool {
    if allocations.is_empty() {
        return true;
    }

    // Only verify that the remaining rectangle can still
    // physically contain the minimum dimensions required by
    // at least one remaining semantic room.
    //
    // Do NOT require the full sum of required_area here.
    // required_area is a semantic target, not a hard rectangle
    // packing constraint. Treating it as such can reject every
    // valid spatial split and leave the renderer with no rooms.
    allocations
        .iter()
        .any(|room| remaining.can_fit(room.min_width.max(2), room.min_depth.max(2)))
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
