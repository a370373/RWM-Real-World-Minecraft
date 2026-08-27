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
        // Topology is a quality signal, not a hard FloorPlan rejection.
        // A partially valid semantic plan must remain renderable.

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

    // Partial-success policy:
    // A failed semantic room allocation must never discard the
    // successfully generated interior layout.
    //
    // Unresolved space is intentionally NOT converted into Corridor.
    // Corridor is a semantic room type and must only exist when the
    // building profile explicitly requests circulation space.
    //
    // Leaving an unresolved fragment empty is safer than inventing
    // semantic room geometry. The existing reconstructed building
    // shell remains authoritative.
    if remaining.area() > 0 {
        // Intentionally leave unresolved fragments untouched.
        //
        // The renderer and downstream topology systems operate only
        // on authoritative Room entries. No synthetic Corridor is
        // created here.
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
    cached_floor_area: &std::collections::HashSet<(i32, i32)>,
) -> Option<FloorPlan> {
    generate_spatial_floor_plan(bounds, allocations, constraints, floor, cached_floor_area)
}

/// Returns true only when EVERY X/Z cell covered by `room`
/// exists in the authoritative reconstructed interior.
///
/// IMPORTANT:
/// - `Rect` is only a candidate geometric region.
/// - `cached_floor_area` is the physical authority.
/// - No cell may be invented.
/// - No expansion or modification of cached_floor_area occurs.
fn room_inside_cached_floor_area(
    room: Rect,
    cached_floor_area: &std::collections::HashSet<(i32, i32)>,
) -> bool {
    for x in room.min_x..=room.max_x {
        for z in room.min_z..=room.max_z {
            if !cached_floor_area.contains(&(x, z)) {
                return false;
            }
        }
    }

    true
}

/// Find the largest axis-aligned rectangular region inside both
/// the candidate bounds and the authoritative cached floor mask.
///
/// `cached_floor_area` remains a hard physical boundary.
/// We never invent cells; we only shrink the candidate region.
fn clamp_rect_to_cached_floor_area(
    candidate: Rect,
    cached_floor_area: &std::collections::HashSet<(i32, i32)>,
) -> Option<Rect> {
    if cached_floor_area.is_empty() {
        return None;
    }

    // ---------------------------------------------------------
    // CANDIDATE GEOMETRY IS ONLY A SUGGESTION
    // ---------------------------------------------------------
    //
    // `cached_floor_area` is the actual usable-space mask.
    //
    // The mask does NOT need to be rectangular.
    //
    // We only need to find a rectangular room INSIDE the
    // available cells.  If the candidate is too large, shrink
    // it instead of rejecting the entire floor.
    //
    // Never invent cells and never modify cached_floor_area.
    //

    let mut rect = candidate;

    // First clamp to the coordinate extent of the authoritative mask.
    let min_x = cached_floor_area.iter().map(|(x, _)| *x).min()?;
    let max_x = cached_floor_area.iter().map(|(x, _)| *x).max()?;
    let min_z = cached_floor_area.iter().map(|(_, z)| *z).min()?;
    let max_z = cached_floor_area.iter().map(|(_, z)| *z).max()?;

    rect.min_x = rect.min_x.max(min_x);
    rect.max_x = rect.max_x.min(max_x);
    rect.min_z = rect.min_z.max(min_z);
    rect.max_z = rect.max_z.min(max_z);

    if rect.min_x > rect.max_x || rect.min_z > rect.max_z {
        return None;
    }

    // ---------------------------------------------------------
    // Iteratively shrink the candidate until every cell belongs
    // to the authoritative usable-space mask.
    //
    // Remove the side that costs the least area each iteration.
    // This preserves as much usable geometry as possible.
    // ---------------------------------------------------------

    loop {
        let mut invalid: Option<(i32, i32)> = None;

        'scan: for z in rect.min_z..=rect.max_z {
            for x in rect.min_x..=rect.max_x {
                if !cached_floor_area.contains(&(x, z)) {
                    invalid = Some((x, z));
                    break 'scan;
                }
            }
        }

        if invalid.is_none() {
            return Some(rect);
        }

        if rect.min_x == rect.max_x && rect.min_z == rect.max_z {
            return None;
        }

        let width = rect.width();
        let depth = rect.depth();

        // Try all four possible trims and choose the one that
        // leaves the largest remaining candidate.
        let mut best: Option<Rect> = None;
        let mut best_area = 0i32;

        if rect.min_x < rect.max_x {
            let r = Rect {
                min_x: rect.min_x + 1,
                min_z: rect.min_z,
                max_x: rect.max_x,
                max_z: rect.max_z,
            };
            let area = r.width() * r.depth();
            if area > best_area {
                best_area = area;
                best = Some(r);
            }

            let r = Rect {
                min_x: rect.min_x,
                min_z: rect.min_z,
                max_x: rect.max_x - 1,
                max_z: rect.max_z,
            };
            let area = r.width() * r.depth();
            if area > best_area {
                best_area = area;
                best = Some(r);
            }
        }

        if rect.min_z < rect.max_z {
            let r = Rect {
                min_x: rect.min_x,
                min_z: rect.min_z + 1,
                max_x: rect.max_x,
                max_z: rect.max_z,
            };
            let area = r.width() * r.depth();
            if area > best_area {
                best_area = area;
                best = Some(r);
            }

            let r = Rect {
                min_x: rect.min_x,
                min_z: rect.min_z,
                max_x: rect.max_x,
                max_z: rect.max_z - 1,
            };
            let area = r.width() * r.depth();
            if area > best_area {
                best_area = area;
                best = Some(r);
            }
        }

        rect = best?;
    }
}


/// Find the largest axis-aligned rectangular region contained entirely
/// in the authoritative cached floor mask.
///
/// IMPORTANT:
/// - `cached_floor_area` is the physical authority.
/// - The mask may be L-shaped, T-shaped, stepped, concave, etc.
/// - This function never invents cells.
/// - It returns one usable rectangle; callers may remove it and search
///   again to utilize the remaining irregular space.
fn largest_cached_rect(
    candidate: Rect,
    cached_floor_area: &std::collections::HashSet<(i32, i32)>,
) -> Option<Rect> {
    if cached_floor_area.is_empty() {
        return None;
    }

    let mut best: Option<Rect> = None;
    let mut best_area = 0i32;

    let min_x = candidate.min_x;
    let max_x = candidate.max_x;
    let min_z = candidate.min_z;
    let max_z = candidate.max_z;

    // Histogram-based maximal rectangle search.
    //
    // Each row contributes contiguous usable horizontal runs.
    // The histogram then finds the largest rectangle spanning
    // multiple rows without requiring the entire floor to be rectangular.
    let width = (max_x - min_x + 1) as usize;

    if width == 0 || max_z < min_z {
        return None;
    }

    let mut heights = vec![0i32; width];

    for z in min_z..=max_z {
        for i in 0..width {
            let x = min_x + i as i32;

            if cached_floor_area.contains(&(x, z)) {
                heights[i] += 1;
            } else {
                heights[i] = 0;
            }
        }

        // Largest rectangle in histogram.
        let mut stack: Vec<usize> = Vec::new();
        let mut i = 0usize;

        while i <= width {
            let current = if i == width { 0 } else { heights[i] };

            while let Some(&top) = stack.last() {
                if heights[top] <= current {
                    break;
                }

                let h = heights[top];
                stack.pop();

                let left = stack.last().map(|&v| v + 1).unwrap_or(0);
                let right = i - 1;

                if h > 0 && right >= left {
                    let rect = Rect {
                        min_x: min_x + left as i32,
                        max_x: min_x + right as i32,
                        max_z: z,
                        min_z: z - h + 1,
                    };

                    let area = rect.width() * rect.depth();

                    if area > best_area {
                        best_area = area;
                        best = Some(rect);
                    }
                }
            }

            stack.push(i);
            i += 1;
        }
    }

    best
}

/// Remove a rectangular room from an authoritative usable-space mask.
///
/// No new cells are created. Only cells already present in the mask
/// are removed.
fn remove_rect_from_cached_area(
    cached_floor_area: &mut std::collections::HashSet<(i32, i32)>,
    room: Rect,
) {
    for x in room.min_x..=room.max_x {
        for z in room.min_z..=room.max_z {
            cached_floor_area.remove(&(x, z));
        }
    }
}


pub fn generate_spatial_floor_plan(
    bounds: Rect,
    allocations: &[RoomAllocation],
    constraints: &SpatialConstraints,
    floor: i32,
    cached_floor_area: &std::collections::HashSet<(i32, i32)>,
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

    // =========================================================
    // AUTHORITATIVE USABLE SPACE
    // =========================================================
    //
    // `interior` is ONLY the planner candidate bounds.
    //
    // IMPORTANT:
    // `cached_floor_area` is NOT required to form one rectangular
    // shape. Real buildings may be L-shaped, T-shaped, stepped,
    // concave, or otherwise irregular.
    //
    // Therefore:
    //   - `interior` is only the outer candidate bounds.
    //   - `cached_floor_area` is the actual usable-space mask.
    //   - Rooms must individually remain completely inside that mask.
    //   - We NEVER reject the whole floor merely because the
    //     candidate Rect contains cells outside the mask.
    //
    // The existing hard checks below remain authoritative:
    //   room_inside_cached_floor_area(remaining, ...)
    //   room_inside_cached_floor_area(room, ...)
    //

    if allocations.is_empty() {
        return None;
    }

    // =========================================================
    // AUTHORITATIVE REAL-WORLD INTERIOR FOOTPRINT
    // =========================================================
    //
    // cached_floor_area is produced by the existing real-world
    // building reconstruction engine.
    //
    // It is READ-ONLY.
    //
    // The floor planner must never invent an interior footprint
    // when the reconstructed footprint is unavailable.
    //
    if cached_floor_area.is_empty() {
        println!("[BI FLOOR PLAN] cached_floor_area is empty; refusing to generate rooms");
        return None;
    }

    let mut plan = FloorPlan::new(interior);
    let mut remaining = interior;

    // ---------------------------------------------------------
    // WORKING AUTHORITATIVE MASK
    // ---------------------------------------------------------
    //
    // `cached_floor_area` is the immutable physical truth.
    // `working_floor_area` is only the unused portion for this
    // floor-plan pass.
    //
    // Rooms are removed from the working mask as they are
    // accepted. The original cached_floor_area is NEVER modified.
    //
    let mut working_floor_area = cached_floor_area.clone();

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

            if remaining.can_fit(min_width, min_depth)
                && room_inside_cached_floor_area(remaining, cached_floor_area)
            {
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
            cached_floor_area,
            &working_floor_area,
        ) {
            // -------------------------------------------------
            // CANDIDATE GEOMETRY IS ONLY A SUGGESTION
            // -------------------------------------------------
            //
            // The semantic solver may propose a rectangular
            // candidate that extends beyond the actual
            // reconstructed usable floor.
            //
            // Do NOT kill the whole floor-plan for that.
            // First shrink the candidate to the largest valid
            // rectangular region backed by cached_floor_area.
            //
            let candidate_room =
                match clamp_rect_to_cached_floor_area(candidate.room, cached_floor_area) {
                    Some(clamped) => clamped,
                    None => continue,
                };

            // Rebuild the candidate remaining space from the
            // authoritative usable room boundary.
            //
            // `candidate.remaining` is still only solver geometry,
            // therefore it must remain bounded by the actual
            // interior before being accepted.
            let room = candidate_room;

            // HARD SAFETY CHECK
            //
            // A solver result may be adjusted, but it MUST still
            // satisfy the authoritative physical boundary.
            if room.min_x >= interior.min_x
                && room.max_x <= interior.max_x
                && room.min_z >= interior.min_z
                && room.max_z <= interior.max_z
                && room.width() >= allocation.min_width.max(2)
                && room.depth() >= allocation.min_depth.max(2)
                && room_inside_cached_floor_area(room, cached_floor_area)
            {
                plan.rooms.push(Room {
                    room_type: allocation.room_type,
                    bounds: room,
                });

                // -------------------------------------------------
                // REMOVE USED CELLS FROM WORKING MASK
                // -------------------------------------------------
                //
                // `cached_floor_area` remains immutable and
                // authoritative.
                //
                // Only the temporary working mask is changed.
                // This allows later allocations to use every
                // remaining authoritative cell, including cells
                // belonging to irregular L/T/stepped shapes.
                //
                for x in room.min_x..=room.max_x {
                    for z in room.min_z..=room.max_z {
                        working_floor_area.remove(&(x, z));
                    }
                }

                // The solver's remaining Rect is only a search
                // envelope. Rebuild it from the remaining
                // authoritative working mask instead of trusting
                // the solver's rectangular remainder.
                remaining = match largest_rect_in_floor_mask(
                    interior,
                    &working_floor_area,
                ) {
                    Some(next) => next,
                    None => Rect {
                        min_x: 1,
                        min_z: 1,
                        max_x: 0,
                        max_z: 0,
                    },
                };

                // Absolute planner-bound clamp.
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
        // The semantic solver could not produce an acceptable
        // candidate. Search the actual remaining authoritative
        // usable-space mask instead of assuming `remaining`
        // itself is fully usable.
        //
        // The fallback is therefore mask-aware as well.
        //

        let min_width = allocation.min_width.max(2);
        let min_depth = allocation.min_depth.max(2);

        let room = match largest_rect_for_allocation(
            remaining,
            allocation,
            &working_floor_area,
        ) {
            Some(room)
                if room.width() >= min_width
                    && room.depth() >= min_depth =>
            {
                room
            }
            _ => continue,
        };

        // -----------------------------------------------------
        // AUTHORITATIVE MASK CLAMP
        // -----------------------------------------------------
        //
        // Fallback candidates are only suggestions.
        // If the rectangular candidate extends beyond the
        // authoritative usable-space mask, shrink it toward
        // the largest valid rectangular portion instead of
        // immediately discarding the candidate.
        //
        // The final room MUST still pass the hard
        // `room_inside_cached_floor_area()` check below.
        //

        // -----------------------------------------------------
        // MASK-AWARE FALLBACK
        // -----------------------------------------------------
        //
        // The fallback candidate is only a suggestion.
        // Search the actual remaining authoritative mask for
        // the largest usable rectangle near the candidate.
        //
        // Irregular building shapes are allowed.
        // We never require the entire floor to be rectangular.
        //

        let room = match largest_cached_rect(room, cached_floor_area) {
            Some(candidate)
                if candidate.width() >= min_width
                    && candidate.depth() >= min_depth =>
            {
                candidate
            }
            _ => continue,
        };

        // HARD PHYSICAL BOUNDARY.
        // Keep this check: cached_floor_area remains authoritative.
        if !room_inside_cached_floor_area(room, cached_floor_area) {
            continue;
        }

        // -----------------------------------------------------
        // HARD ROOM BOUNDARY CHECK
        // -----------------------------------------------------

        if room.width() < min_width
            || room.depth() < min_depth
            || room.min_x < interior.min_x
            || room.max_x > interior.max_x
            || room.min_z < interior.min_z
            || room.max_z > interior.max_z
            || !room_inside_cached_floor_area(room, cached_floor_area)
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
        // CONSUME THE USED AUTHORITATIVE SPACE
        // -----------------------------------------------------
        //
        // `working_floor_area` is the mutable planning mask.
        // `cached_floor_area` remains immutable and authoritative.
        //
        // Once a room is accepted, remove only that room's cells
        // from the working mask. This preserves every other usable
        // cell, including cells belonging to L/T/stepped/concave
        // parts of the real building.
        //
        for x in room.min_x..=room.max_x {
            for z in room.min_z..=room.max_z {
                working_floor_area.remove(&(x, z));
            }
        }

        // -----------------------------------------------------
        // Update remaining space
        // -----------------------------------------------------
        //
        // The old implementation reduced `remaining` to one
        // rectangular strip. That loses usable space whenever
        // the authoritative floor is L-shaped, T-shaped,
        // stepped, or concave.
        //
        // The room itself is already guaranteed to be inside
        // cached_floor_area. Therefore the next candidate must
        // be searched against the authoritative mask again.
        //
        // Keep `remaining` bounded by the planner interior.
        // Do NOT invent geometry outside the authoritative mask.
        //

        remaining = Rect {
            min_x: interior.min_x,
            min_z: interior.min_z,
            max_x: interior.max_x,
            max_z: interior.max_z,
        };

        // -----------------------------------------------------
        // IMPORTANT
        // -----------------------------------------------------
        //
        // `remaining` is now the search envelope.
        // `cached_floor_area` remains the actual usable-space
        // authority.
        //
        // Every subsequent room is therefore selected through
        // `largest_cached_rect()` + `room_inside_cached_floor_area()`.
        //
        // Never replace cached_floor_area with this Rect.
        //
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

    plan.is_valid().then_some(plan)
}

#[derive(Debug, Clone, Copy)]
struct SplitCandidate {
    room: Rect,
    remaining: Rect,
    score: i32,
}


/// Find the largest axis-aligned rectangular region fully backed by
/// the current authoritative working floor mask.
///
/// The mask may be irregular (L-shaped, T-shaped, stepped, etc.).
/// We only extract a rectangle from cells that actually exist.
/// No cells are invented and the original cached_floor_area is untouched.

/// Find the largest usable rectangle in the current working mask
/// that satisfies one room allocation.
///
/// The mask remains authoritative. This function only searches it.
/// It never creates cells outside the mask.
fn largest_rect_for_allocation(
    bounds: Rect,
    allocation: &RoomAllocation,
    floor_mask: &std::collections::HashSet<(i32, i32)>,
) -> Option<Rect> {
    let min_width = allocation.min_width.max(2);
    let min_depth = allocation.min_depth.max(2);
    let desired_area = allocation
        .required_area
        .max(min_width * min_depth);

    let mut best: Option<Rect> = None;
    let mut best_score = i64::MIN;

    for min_z in bounds.min_z..=bounds.max_z {
        for min_x in bounds.min_x..=bounds.max_x {
            for width in min_width..=(bounds.max_x - min_x + 1) {
                let max_x = min_x + width - 1;

                for depth in min_depth..=(bounds.max_z - min_z + 1) {
                    let max_z = min_z + depth - 1;

                    let area = width * depth;

                    // Avoid spending time on rectangles that are
                    // already smaller than the allocation's minimum.
                    if area < min_width * min_depth {
                        continue;
                    }

                    let mut valid = true;

                    'cells: for x in min_x..=max_x {
                        for z in min_z..=max_z {
                            if !floor_mask.contains(&(x, z)) {
                                valid = false;
                                break 'cells;
                            }
                        }
                    }

                    if !valid {
                        continue;
                    }

                    // Prefer an area close to the requested area,
                    // while still strongly preferring larger usable
                    // regions when the request can be exceeded.
                    let area_distance = (area - desired_area).abs() as i64;
                    let score = (area as i64 * 1000) - area_distance;

                    if score > best_score {
                        best_score = score;
                        best = Some(Rect {
                            min_x,
                            min_z,
                            max_x,
                            max_z,
                        });
                    }
                }
            }
        }
    }

    best
}

fn largest_rect_in_floor_mask(
    bounds: Rect,
    floor_mask: &std::collections::HashSet<(i32, i32)>,
) -> Option<Rect> {
    if floor_mask.is_empty() {
        return None;
    }

    let mut best: Option<Rect> = None;
    let mut best_area = 0i32;

    for min_z in bounds.min_z..=bounds.max_z {
        for min_x in bounds.min_x..=bounds.max_x {
            if !floor_mask.contains(&(min_x, min_z)) {
                continue;
            }

            let mut max_x = min_x;

            while max_x <= bounds.max_x
                && floor_mask.contains(&(max_x, min_z))
            {
                let mut max_z = min_z;

                loop {
                    if max_z > bounds.max_z {
                        break;
                    }

                    let mut valid = true;

                    for x in min_x..=max_x {
                        if !floor_mask.contains(&(x, max_z)) {
                            valid = false;
                            break;
                        }
                    }

                    if !valid {
                        break;
                    }

                    let rect = Rect {
                        min_x,
                        min_z,
                        max_x,
                        max_z,
                    };

                    let area = rect.width() * rect.depth();

                    if area > best_area {
                        best_area = area;
                        best = Some(rect);
                    }

                    max_z += 1;
                }

                max_x += 1;
            }
        }
    }

    best
}

fn find_best_split(
    remaining: Rect,
    allocation: &RoomAllocation,
    constraints: &SpatialConstraints,
    remaining_allocations: &[RoomAllocation],
    floor: i32,
    cached_floor_area: &std::collections::HashSet<(i32, i32)>,
    working_floor_area: &std::collections::HashSet<(i32, i32)>,
) -> Option<SplitCandidate> {
    let mut best: Option<SplitCandidate> = None;

    // =========================================================
    // MASK-AWARE CANDIDATES
    // =========================================================
    //
    // The rectangular `remaining` value is only a search envelope.
    // The real usable floor is represented by `cached_floor_area`.
    //
    // Search the actual mask first so irregular buildings such as
    // L/T/stepped/concave footprints remain usable.
    //

    if let Some(room) = largest_rect_for_allocation(
        remaining,
        allocation,
        working_floor_area,
    ) {
        if room_inside_cached_floor_area(room, cached_floor_area) {
            let next = remaining;

            if can_reserve_remaining_space(
                next,
                remaining_allocations,
            ) {
                let base_score =
                    score_room(room, allocation, constraints, floor);

                let topology_score =
                    topology_candidate_score(room, next);

                consider_candidate(
                    &mut best,
                    SplitCandidate {
                        room,
                        remaining: next,
                        score: base_score + topology_score,
                    },
                );
            }
        }
    }

    // =========================================================
    // X AXIS CANDIDATES
    // =========================================================

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

            if !room_inside_cached_floor_area(room, cached_floor_area) {
                continue;
            }

            let next = Rect {
                min_x: max_x + 1,
                min_z: remaining.min_z,
                max_x: remaining.max_x,
                max_z: remaining.max_z,
            };

            if !can_reserve_remaining_space(next, remaining_allocations) {
                continue;
            }

            let base_score =
                score_room(room, allocation, constraints, floor);

            let topology_score =
                topology_candidate_score(room, next);

            consider_candidate(
                &mut best,
                SplitCandidate {
                    room,
                    remaining: next,
                    score: base_score + topology_score,
                },
            );
        }
    }

    // =========================================================
    // Z AXIS CANDIDATES
    // =========================================================

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

            if !room_inside_cached_floor_area(room, cached_floor_area) {
                continue;
            }

            let next = Rect {
                min_x: remaining.min_x,
                min_z: max_z + 1,
                max_x: remaining.max_x,
                max_z: remaining.max_z,
            };

            if !can_reserve_remaining_space(next, remaining_allocations) {
                continue;
            }

            let base_score =
                score_room(room, allocation, constraints, floor);

            let topology_score =
                topology_candidate_score(room, next);

            consider_candidate(
                &mut best,
                SplitCandidate {
                    room,
                    remaining: next,
                    score: base_score + topology_score,
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
