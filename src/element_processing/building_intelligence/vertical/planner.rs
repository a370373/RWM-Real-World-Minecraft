use super::super::decision::{StairSize, VerticalAccessKind};
use super::super::room_graph::{RoomConnectionKind, RoomGraph};
use crate::element_processing::building_intelligence::types::BuildingContext;
use crate::element_processing::subprocessor::interior::{FloorPlan, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAccessDirection {
    North,
    East,
    South,
    West,
}

#[derive(Debug, Clone, Copy)]
pub struct VerticalAccessPlan {
    pub kind: VerticalAccessKind,
    pub size: StairSize,

    pub from_floor: usize,
    pub to_floor: usize,

    pub x: i32,
    pub z: i32,

    pub direction: VerticalAccessDirection,

    pub width: i32,
    pub length: i32,

    pub lower_y: i32,
    pub upper_y: i32,
}

impl VerticalAccessPlan {
    /// Returns every X/Z cell occupied by this vertical-access footprint.
    ///
    /// This is purely geometric and does not modify FloorPlan or building
    /// geometry.
    pub fn footprint_cells(&self) -> Vec<(i32, i32)> {
        let mut cells = Vec::new();

        match self.direction {
            VerticalAccessDirection::North => {
                for distance in 0..self.length {
                    let z = self.z - distance;
                    for w in 0..self.width {
                        cells.push((self.x + w, z));
                    }
                }
            }
            VerticalAccessDirection::East => {
                for distance in 0..self.length {
                    let x = self.x + distance;
                    for w in 0..self.width {
                        cells.push((x, self.z + w));
                    }
                }
            }
            VerticalAccessDirection::South => {
                for distance in 0..self.length {
                    let z = self.z + distance;
                    for w in 0..self.width {
                        cells.push((self.x + w, z));
                    }
                }
            }
            VerticalAccessDirection::West => {
                for distance in 0..self.length {
                    let x = self.x - distance;
                    for w in 0..self.width {
                        cells.push((x, self.z + w));
                    }
                }
            }
        }

        cells
    }

    pub fn is_valid(&self, context: &BuildingContext) -> bool {
        if self.from_floor >= self.to_floor {
            return false;
        }

        if self.width <= 0 || self.length <= 0 {
            return false;
        }

        // Validate the complete physical footprint, including
        // directional stair progression.
        let (min_x, max_x, min_z, max_z) = match self.direction {
            VerticalAccessDirection::North => (
                self.x,
                self.x + self.width - 1,
                self.z - self.length + 1,
                self.z,
            ),
            VerticalAccessDirection::East => (
                self.x,
                self.x + self.length - 1,
                self.z,
                self.z + self.width - 1,
            ),
            VerticalAccessDirection::South => (
                self.x,
                self.x + self.width - 1,
                self.z,
                self.z + self.length - 1,
            ),
            VerticalAccessDirection::West => (
                self.x - self.length + 1,
                self.x,
                self.z,
                self.z + self.width - 1,
            ),
        };

        min_x >= context.min_x
            && max_x <= context.max_x
            && min_z >= context.min_z
            && max_z <= context.max_z
            && self.lower_y < self.upper_y
    }
}

/// Plan vertical circulation from the existing semantic RoomGraph
/// and the existing physical FloorPlan geometry.
///
/// IMPORTANT:
/// This function does not create, move, resize or modify any room.
/// FloorPlan is treated as read-only source geometry.
pub fn plan_vertical_access(
    context: &BuildingContext,
    graph: &RoomGraph,
    floor_plans: &[FloorPlan],
    floor_levels: &[i32],
    kind: VerticalAccessKind,
    size: StairSize,
    width: i32,
) -> Vec<VerticalAccessPlan> {
    if kind == VerticalAccessKind::None || context.floors <= 1 {
        return Vec::new();
    }

    let width = width.max(1);

    /*
     * StairSize is semantic intent.
     *
     * The physical staircase must still span the actual world-space
     * floor height.
     */
    let preferred_length = match size {
        StairSize::Compact => 2,
        StairSize::Small => 2,
        StairSize::Medium => 3,
        StairSize::Large => 4,
        StairSize::Grand => 5,
    };

    let mut plans = Vec::new();

    /*
     * =========================================================
     * PRIMARY PATH
     * =========================================================
     *
     * Prefer explicit semantic VerticalAccess connections.
     *
     * This is the normal path when RoomGraph successfully
     * represents vertical circulation.
     */
    for connection in &graph.connections {
        if connection.kind != RoomConnectionKind::VerticalAccess {
            continue;
        }

        let Some(from_room) = graph.room(connection.from) else {
            continue;
        };

        let Some(to_room) = graph.room(connection.to) else {
            continue;
        };

        if from_room.floor >= to_room.floor {
            continue;
        }

        let Some(plan) = build_vertical_plan_from_rooms(
            context,
            floor_plans,
            floor_levels,
            from_room.floor,
            from_room.floor_room_index,
            to_room.floor,
            to_room.floor_room_index,
            kind,
            size,
            width,
            preferred_length,
        ) else {
            continue;
        };

        plans.push(plan);
    }

    /*
     * =========================================================
     * FALLBACK PATH
     * =========================================================
     *
     * If room-layout/topology generation failed and therefore
     * graph.connections contains no usable VerticalAccess,
     * derive vertical-access INTENT directly from the existing
     * FloorPlan geometry.
     *
     * IMPORTANT:
     * - FloorPlan is read-only.
     * - Rooms are never created or moved.
     * - Building geometry is never changed.
     * - BBox is never changed.
     * - This only creates a VerticalAccessPlan for the renderer.
     */
    if plans.is_empty() {
        for floor in 0..floor_plans.len().saturating_sub(1) {
            let lower_plan = &floor_plans[floor];
            let upper_plan = &floor_plans[floor + 1];

            if lower_plan.rooms.is_empty() || upper_plan.rooms.is_empty() {
                continue;
            }

            let mut best_pair: Option<(usize, usize, i32)> = None;

            for (lower_index, lower_room) in lower_plan.rooms.iter().enumerate() {
                for (upper_index, upper_room) in upper_plan.rooms.iter().enumerate() {
                    let overlap_min_x = lower_room.bounds.min_x.max(upper_room.bounds.min_x);
                    let overlap_max_x = lower_room.bounds.max_x.min(upper_room.bounds.max_x);

                    let overlap_min_z = lower_room.bounds.min_z.max(upper_room.bounds.min_z);
                    let overlap_max_z = lower_room.bounds.max_z.min(upper_room.bounds.max_z);

                    if overlap_min_x > overlap_max_x || overlap_min_z > overlap_max_z {
                        continue;
                    }

                    let overlap_width = overlap_max_x - overlap_min_x + 1;

                    let overlap_depth = overlap_max_z - overlap_min_z + 1;

                    let overlap_area = overlap_width * overlap_depth;

                    if best_pair
                        .map(|(_, _, best_area)| overlap_area > best_area)
                        .unwrap_or(true)
                    {
                        best_pair = Some((lower_index, upper_index, overlap_area));
                    }
                }
            }

            let Some((lower_index, upper_index, _)) = best_pair else {
                continue;
            };

            let Some(plan) = build_vertical_plan_from_rooms(
                context,
                floor_plans,
                floor_levels,
                floor,
                lower_index,
                floor + 1,
                upper_index,
                kind,
                size,
                width,
                preferred_length,
            ) else {
                continue;
            };

            println!(
                "[VERTICAL] FALLBACK PLAN floor {} -> {} room {} -> {} at ({}, {}) {:?}",
                floor,
                floor + 1,
                lower_index,
                upper_index,
                plan.x,
                plan.z,
                plan.direction
            );

            plans.push(plan);
        }
    }

    /*
     * =========================================================
     * DEDUPLICATION
     * =========================================================
     */
    plans.sort_by_key(|p| (p.from_floor, p.to_floor, p.x, p.z));

    plans.dedup_by_key(|p| (p.from_floor, p.to_floor, p.x, p.z));

    println!("[VERTICAL] FINAL PLANS={}", plans.len());

    plans
}

fn build_vertical_plan_from_rooms(
    context: &BuildingContext,
    floor_plans: &[FloorPlan],
    floor_levels: &[i32],
    from_floor: usize,
    from_index: usize,
    to_floor: usize,
    to_index: usize,
    kind: VerticalAccessKind,
    size: StairSize,
    width: i32,
    preferred_length: i32,
) -> Option<VerticalAccessPlan> {
    let from_plan = floor_plans.get(from_floor)?;
    let to_plan = floor_plans.get(to_floor)?;

    let from_bounds = from_plan.rooms.get(from_index)?.bounds;
    let to_bounds = to_plan.rooms.get(to_index)?.bounds;

    let lower_y = *floor_levels.get(from_floor)?;
    let upper_y = *floor_levels.get(to_floor)?;

    if upper_y <= lower_y {
        return None;
    }

    let height = upper_y - lower_y;

    /*
     * Minecraft stairs rise one block per horizontal step.
     *
     * H vertical blocks therefore require at least H + 1
     * physical stair positions.
     */
    /*
     * Minecraft staircase geometry:
     *
     * One horizontal stair position corresponds to one
     * vertical block of rise.
     *
     * Therefore a floor-to-floor height H requires exactly
     * H + 1 physical stair positions.
     *
     * StairSize remains semantic intent only. It must not
     * compress the physical staircase below the real
     * floor-to-floor height.
     */
    let length = (height + 1).max(2);

    let direction = choose_vertical_direction(from_bounds, to_bounds, width, length);

    let (x, z) = overlapping_anchor(from_bounds, to_bounds, width, length, direction)?;

    let plan = VerticalAccessPlan {
        kind,
        size,
        from_floor,
        to_floor,
        x,
        z,
        direction,
        width,
        length,
        lower_y,
        upper_y,
    };

    if !plan.is_valid(context) {
        return None;
    }

    Some(plan)
}

fn choose_vertical_direction(a: Rect, b: Rect, width: i32, length: i32) -> VerticalAccessDirection {
    let width = width.max(1);
    let length = length.max(1);

    let min_x = a.min_x.max(b.min_x);
    let max_x = a.max_x.min(b.max_x);
    let min_z = a.min_z.max(b.min_z);
    let max_z = a.max_z.min(b.max_z);

    if min_x > max_x || min_z > max_z {
        return VerticalAccessDirection::South;
    }

    let overlap_width = max_x - min_x + 1;
    let overlap_depth = max_z - min_z + 1;

    let north_south_fits = overlap_width >= width && overlap_depth >= length;

    let east_west_fits = overlap_width >= length && overlap_depth >= width;

    match (north_south_fits, east_west_fits) {
        (true, false) => {
            if b.min_z >= a.min_z {
                VerticalAccessDirection::South
            } else {
                VerticalAccessDirection::North
            }
        }

        (false, true) => {
            if b.min_x >= a.min_x {
                VerticalAccessDirection::East
            } else {
                VerticalAccessDirection::West
            }
        }

        (true, true) => {
            /*
             * Prefer the orientation with the larger shared span.
             * This keeps the staircase inside the actual overlapping
             * room geometry instead of relying on arbitrary center offsets.
             */
            if overlap_depth >= overlap_width {
                if b.center().1 >= a.center().1 {
                    VerticalAccessDirection::South
                } else {
                    VerticalAccessDirection::North
                }
            } else if b.center().0 >= a.center().0 {
                VerticalAccessDirection::East
            } else {
                VerticalAccessDirection::West
            }
        }

        (false, false) => {
            /*
             * Neither orientation can fit the requested footprint.
             * Return the most natural direction; overlapping_anchor()
             * will reject it safely.
             */
            let (ax, az) = a.center();
            let (bx, bz) = b.center();

            let dx = (bx - ax).abs();
            let dz = (bz - az).abs();

            if dz >= dx {
                if bz >= az {
                    VerticalAccessDirection::South
                } else {
                    VerticalAccessDirection::North
                }
            } else if bx >= ax {
                VerticalAccessDirection::East
            } else {
                VerticalAccessDirection::West
            }
        }
    }
}

fn overlapping_anchor(
    a: Rect,
    b: Rect,
    width: i32,
    length: i32,
    direction: VerticalAccessDirection,
) -> Option<(i32, i32)> {
    let width = width.max(1);
    let length = length.max(1);

    let min_x = a.min_x.max(b.min_x);
    let max_x = a.max_x.min(b.max_x);
    let min_z = a.min_z.max(b.min_z);
    let max_z = a.max_z.min(b.max_z);

    if min_x > max_x || min_z > max_z {
        return None;
    }

    let (footprint_x, footprint_z) = match direction {
        VerticalAccessDirection::North | VerticalAccessDirection::South => (width, length),

        VerticalAccessDirection::East | VerticalAccessDirection::West => (length, width),
    };

    let overlap_width = max_x - min_x + 1;
    let overlap_length = max_z - min_z + 1;

    // The complete staircase footprint must fit inside
    // the shared physical area of the two rooms.
    if overlap_width < footprint_x || overlap_length < footprint_z {
        return None;
    }

    let safe_max_x = max_x - footprint_x + 1;
    let safe_max_z = max_z - footprint_z + 1;

    // Center the complete stair footprint in the valid overlap.
    let anchor_x = min_x + (safe_max_x - min_x) / 2;
    let anchor_z = min_z + (safe_max_z - min_z) / 2;

    match direction {
        VerticalAccessDirection::North => Some((anchor_x, anchor_z + footprint_z - 1)),

        VerticalAccessDirection::South => Some((anchor_x, anchor_z)),

        VerticalAccessDirection::East => Some((anchor_x, anchor_z)),

        VerticalAccessDirection::West => Some((anchor_x + footprint_x - 1, anchor_z)),
    }
}
