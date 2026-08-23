use super::window_analysis::WindowInfo;
use crate::element_processing::subprocessor::interior::{FloorPlan, RoomType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightKind {
    Ceiling,
    Wall,
}

#[derive(Debug, Clone)]
pub struct LightPlacement {
    pub room_type: RoomType,
    pub floor: i32,
    pub x: i32,
    pub z: i32,
    pub kind: LightKind,
}

#[derive(Debug, Clone, Default)]
pub struct LightingPlan {
    pub placements: Vec<LightPlacement>,
}

pub fn plan_lighting(floor_plans: &[FloorPlan], windows: &[WindowInfo]) -> LightingPlan {
    let mut plan = LightingPlan::default();

    for (floor, floor_plan) in floor_plans.iter().enumerate() {
        for room in &floor_plan.rooms {
            let bounds = room.bounds;

            let has_window = windows
                .iter()
                .any(|window| window.floor == floor as i32 && bounds.contains(window.x, window.z));

            let width = (bounds.max_x - bounds.min_x + 1).max(1);
            let depth = (bounds.max_z - bounds.min_z + 1).max(1);
            let area = width * depth;

            // Natural light is useful information, but artificial
            // lighting is still required for usable interiors.
            let density = if has_window {
                if area >= 100 {
                    2
                } else {
                    1
                }
            } else {
                if area >= 100 {
                    3
                } else {
                    1
                }
            };

            let center_x = (bounds.min_x + bounds.max_x) / 2;
            let center_z = (bounds.min_z + bounds.max_z) / 2;

            if density == 1 {
                plan.placements.push(LightPlacement {
                    room_type: room.room_type,
                    floor: floor as i32,
                    x: center_x,
                    z: center_z,
                    kind: LightKind::Ceiling,
                });
            } else {
                let offsets = [(-width / 4, -depth / 4), (width / 4, depth / 4), (0, 0)];

                for &(dx, dz) in offsets.iter().take(density as usize) {
                    let x = (center_x + dx).clamp(bounds.min_x + 1, bounds.max_x - 1);
                    let z = (center_z + dz).clamp(bounds.min_z + 1, bounds.max_z - 1);

                    plan.placements.push(LightPlacement {
                        room_type: room.room_type,
                        floor: floor as i32,
                        x,
                        z,
                        kind: LightKind::Ceiling,
                    });
                }
            }
        }
    }

    plan
}
