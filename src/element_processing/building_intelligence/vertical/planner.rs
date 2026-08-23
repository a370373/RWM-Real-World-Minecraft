use super::super::decision::{StairSize, VerticalAccessKind};
use super::super::room_graph::{RoomConnectionKind, RoomGraph};
use crate::element_processing::building_intelligence::types::BuildingContext;
use crate::element_processing::subprocessor::interior::{FloorPlan, Rect};

#[derive(Debug, Clone, Copy)]
pub struct VerticalAccessPlan {
    pub kind: VerticalAccessKind,
    pub size: StairSize,

    pub from_floor: usize,
    pub to_floor: usize,

    pub x: i32,
    pub z: i32,

    pub width: i32,
    pub length: i32,

    pub lower_y: i32,
    pub upper_y: i32,
}

impl VerticalAccessPlan {
    pub fn is_valid(&self, context: &BuildingContext) -> bool {
        if self.from_floor >= self.to_floor {
            return false;
        }

        if self.width <= 0 || self.length <= 0 {
            return false;
        }

        // Validate the complete physical footprint, not only its anchor.
        let max_x = self.x + self.width - 1;
        let max_z = self.z + self.length - 1;

        self.x >= context.min_x
            && max_x <= context.max_x
            && self.z >= context.min_z
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

        let length = match size {
            StairSize::Compact => 2,
            StairSize::Small => 2,
            StairSize::Medium => 3,
            StairSize::Large => 4,
            StairSize::Grand => 5,
        };

        /*
         * Prefer a position that exists in both floor geometries.
         *
         * The anchor solver will later account for the requested
         * vertical-access footprint instead of selecting an arbitrary
         * point from the overlap.
         */
        let Some((x, z)) = overlapping_anchor(from_bounds, to_bounds, width.max(1), length) else {
            continue;
        };

        let plan = VerticalAccessPlan {
            kind,
            size,
            from_floor: from_room.floor,
            to_floor: to_room.floor,
            x,
            z,
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

fn overlapping_anchor(a: Rect, b: Rect, width: i32, length: i32) -> Option<(i32, i32)> {
    let min_x = a.min_x.max(b.min_x);
    let max_x = a.max_x.min(b.max_x);
    let min_z = a.min_z.max(b.min_z);
    let max_z = a.max_z.min(b.max_z);

    if min_x > max_x || min_z > max_z {
        return None;
    }

    let width = width.max(1);
    let length = length.max(1);

    let overlap_width = max_x - min_x + 1;
    let overlap_length = max_z - min_z + 1;

    if overlap_width < width || overlap_length < length {
        return None;
    }

    let safe_max_x = max_x - width + 1;
    let safe_max_z = max_z - length + 1;

    let x = min_x + (safe_max_x - min_x) / 2;
    let z = min_z + (safe_max_z - min_z) / 2;

    Some((x, z))
}
