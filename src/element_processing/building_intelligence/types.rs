use crate::element_processing::subprocessor::interior::{FloorPlan, RoomType};

use super::circulation::{
    BuildingCirculationPlan, CirculationPlan, DoorwayPlan, InteriorCirculationPlan,
};
use super::decision::BuildingDecision;
use super::entrance::EntranceCandidate;
use super::furniture::FurnitureItem;
use super::room_graph::RoomGraph;
use super::vertical::VerticalAccessPlan;
use crate::element_processing::subprocessor::interior::decision::LightingPlan;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildingEnvironment {
    RoadFront,
    PathFront,
    ParkingFront,
    OpenFront,
    DenseUrban,
    Rural,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct BuildingContext {
    pub building_type: crate::element_processing::subprocessor::interior::BuildingType,

    pub min_x: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_z: i32,

    pub floors: usize,

    pub nearby_road_distance: Option<i32>,
    /// Nearest reconstructed road block coordinate. READ-ONLY.
    pub nearby_road_position: Option<(i32, i32)>,
    pub nearby_path_distance: Option<i32>,
    pub nearby_parking_distance: Option<i32>,

    pub environment: BuildingEnvironment,
}

impl BuildingContext {
    pub fn width(&self) -> i32 {
        self.max_x - self.min_x + 1
    }

    pub fn depth(&self) -> i32 {
        self.max_z - self.min_z + 1
    }

    pub fn area(&self) -> i32 {
        self.width() * self.depth()
    }

    pub fn center(&self) -> (i32, i32) {
        ((self.min_x + self.max_x) / 2, (self.min_z + self.max_z) / 2)
    }

    pub fn is_large(&self) -> bool {
        self.area() >= 400
    }
}

/// Complete procedural interpretation of a real-world building.
///
/// This is the bridge between:
///
///     Real-world building data
///             ↓
///     Building Intelligence
///             ↓
///     Minecraft interior renderer
///
/// The renderer should consume this structure instead of independently
/// deciding what rooms, doors, stairs or furniture a building needs.
#[derive(Debug, Clone)]
pub struct PlannedBuilding {
    /// Original real-world building context.
    pub context: BuildingContext,

    /// Central intelligence result.
    pub decision: BuildingDecision,

    /// Geometric room layouts generated from the decision.
    pub floor_plans: Vec<FloorPlan>,

    /// Detected / inferred real-world main entrance.
    pub entrance: Option<EntranceCandidate>,

    /// Furniture intent generated from room semantics.
    pub furniture: Vec<FurnitureItem>,

    /// Semantic connectivity graph between rooms.
    pub room_graph: RoomGraph,

    /// Artificial lighting plan derived from existing rooms and daylight information.
    /// This is interior-only data and never modifies the reconstructed exterior.
    pub lighting: LightingPlan,
    /// Planned vertical access derived from Building Intelligence.
    /// Read-only intent; renderer owns physical block placement.

    /// Semantic circulation plan for the whole building.
    /// READ-ONLY validation/intent; never modifies exterior geometry or BBox.
    pub circulation: CirculationPlan,

    /// Physical doorway intents derived from existing room boundaries.
    /// These are intents only; the renderer owns block placement.
    pub doorway_plan: DoorwayPlan,

    /// Whole-building connectivity validation using the planned doorways.
    pub building_circulation: BuildingCirculationPlan,

    /// Per-floor interior path validation around planned furniture.
    pub floor_circulation: Vec<InteriorCirculationPlan>,
}

impl PlannedBuilding {
    pub fn has_floor_plan(&self) -> bool {
        !self.floor_plans.is_empty()
    }

    pub fn total_rooms(&self) -> usize {
        self.floor_plans.iter().map(|plan| plan.rooms.len()).sum()
    }

    pub fn room_count(&self, room_type: RoomType) -> usize {
        self.floor_plans
            .iter()
            .flat_map(|plan| plan.rooms.iter())
            .filter(|room| room.room_type == room_type)
            .count()
    }

    pub fn has_main_entrance(&self) -> bool {
        self.entrance.is_some()
    }
}
