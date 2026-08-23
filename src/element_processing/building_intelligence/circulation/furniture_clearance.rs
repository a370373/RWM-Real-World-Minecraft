use crate::element_processing::building_intelligence::FurnitureItem;
use crate::element_processing::subprocessor::interior::Rect;

/// A read-only obstacle derived from already-planned furniture.
///
/// This does NOT change furniture placement.
/// It only describes the space occupied by furniture for circulation checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FurnitureObstacle {
    pub room_id: usize,
    pub min_x: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_z: i32,
}

impl FurnitureObstacle {
    pub fn intersects(&self, rect: Rect) -> bool {
        self.min_x <= rect.max_x
            && self.max_x >= rect.min_x
            && self.min_z <= rect.max_z
            && self.max_z >= rect.min_z
    }
}

/// Convert existing furniture intent into conservative circulation
/// obstacles.
///
/// FurniturePlanner remains the source of truth for furniture.
/// This function only reads it.
pub fn build_furniture_obstacles(furniture: &[FurnitureItem]) -> Vec<FurnitureObstacle> {
    furniture
        .iter()
        .map(|item| {
            let width = match item.kind {
                crate::element_processing::building_intelligence::FurnitureKind::Bed
                | crate::element_processing::building_intelligence::FurnitureKind::HospitalBed => 2,

                crate::element_processing::building_intelligence::FurnitureKind::Sofa => 2,

                crate::element_processing::building_intelligence::FurnitureKind::DiningTable
                | crate::element_processing::building_intelligence::FurnitureKind::Table => 2,

                _ => 1,
            };

            let depth = match item.kind {
                crate::element_processing::building_intelligence::FurnitureKind::Bed
                | crate::element_processing::building_intelligence::FurnitureKind::HospitalBed => 2,

                crate::element_processing::building_intelligence::FurnitureKind::Sofa => 1,

                _ => 1,
            };

            FurnitureObstacle {
                room_id: item.room_id,
                min_x: item.relative_x,
                min_z: item.relative_z,
                max_x: item.relative_x + width - 1,
                max_z: item.relative_z + depth - 1,
            }
        })
        .collect()
}

/// Check whether a proposed circulation cell intersects furniture.
///
/// This is intentionally conservative.
pub fn circulation_cell_blocked(cell: Rect, obstacles: &[FurnitureObstacle]) -> bool {
    obstacles.iter().any(|obstacle| obstacle.intersects(cell))
}
