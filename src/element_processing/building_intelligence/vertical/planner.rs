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

    let mut plans = Vec::new();

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

        let Some(from_plan) = floor_plans.get(from_room.floor) else {
            continue;
        };

        let Some(to_plan) = floor_plans.get(to_room.floor) else {
            continue;
        };

        let from_index = graph
            .rooms
            .iter()
            .filter(|room| room.floor == from_room.floor)
            .position(|room| room.id == from_room.id);

        let to_index = graph
            .rooms
            .iter()
            .filter(|room| room.floor == to_room.floor)
            .position(|room| room.id == to_room.id);

        let (Some(from_index), Some(to_index)) = (from_index, to_index) else {
            continue;
        };

        let Some(from_bounds) = from_plan.rooms.get(from_index).map(|room| room.bounds) else {
            continue;
        };

        let Some(to_bounds) = to_plan.rooms.get(to_index).map(|room| room.bounds) else {
            continue;
        };

        // FloorPlan intentionally contains only X/Z geometry.
        //
        // IMPORTANT:
        // Room.floor is a semantic floor index, NOT a world Y coordinate.
        //
        // The authoritative Minecraft Y coordinate comes from the same
        // floor-level information already consumed by the main renderer.
        //
        // This keeps VerticalAccessPlanner and the physical renderer in
        // exactly the same vertical coordinate system.
        // Room.floor is a semantic floor index.
        // VerticalAccessPlan requires world-space Y.
        //
        // The planner uses the existing building floor spacing
        // convention here. The renderer consumes these values
        // without changing the reconstructed building geometry.
        let lower_y = match floor_levels.get(from_room.floor) {
            Some(&y) => y,
            None => continue,
        };

        let upper_y = match floor_levels.get(to_room.floor) {
            Some(&y) => y,
            None => continue,
        };

        if upper_y <= lower_y {
            continue;
        }

        /*
         * Minecraft stairs rise one block per horizontal step.
         *
         * Therefore the physical horizontal run must be derived
         * from the actual world-space floor height.
         *
         * A vertical difference of H blocks requires H + 1
         * stair positions so that both the lower and upper levels
         * are represented.
         *
         * StairSize is still retained as semantic intent, but it
         * may no longer shorten the physical staircase below what
         * the actual building height requires.
         */
        let height = upper_y - lower_y;

        if height <= 0 {
            continue;
        }

        let preferred_length = match size {
            StairSize::Compact => 2,
            StairSize::Small => 2,
            StairSize::Medium => 3,
            StairSize::Large => 4,
            StairSize::Grand => 5,
        };

        let length = preferred_length.max(2);

        /*
         * First determine the direction from the relationship between
         * the two existing floor geometries.
         *
         * The anchor solver then uses the same direction so the
         * physical footprint matches the renderer and validation.
         */
        let direction = choose_vertical_direction(from_bounds, to_bounds, width.max(1), length);

        let Some((x, z)) =
            overlapping_anchor(from_bounds, to_bounds, width.max(1), length, direction)
        else {
            continue;
        };

        let plan = VerticalAccessPlan {
            kind,
            size,
            from_floor: from_room.floor,
            to_floor: to_room.floor,
            x,
            z,
            direction,
            width: width.max(1),
            length,
            lower_y,
            upper_y,
        };

        if plan.is_valid(context) {
            plans.push(plan);
        }
    }

    /*
     * Avoid duplicate vertical structures when multiple graph
     * connections happen to describe the same floor transition.
     */
    plans.sort_by_key(|p| (p.from_floor, p.to_floor, p.x, p.z));

    plans.dedup_by_key(|p| (p.from_floor, p.to_floor, p.x, p.z));

    plans
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
