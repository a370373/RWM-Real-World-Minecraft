pub mod room_loot;
pub mod circulation;
pub mod furniture_planner;
pub mod input;
pub mod layout_solver;
pub mod vertical;
pub use decision::decide_building;
pub mod decision;
pub mod entrance;
pub mod entrance_detector;
pub mod furniture;
pub mod planner;
pub mod room_graph;
pub mod types;

pub use entrance::{EntranceCandidate, EntranceSide};
pub use entrance_detector::{detect_main_entrance, detect_mapped_entrance, EntranceEvidence};
pub use furniture::{furniture_profile, FurnitureItem, FurnitureKind};
pub use planner::build_floor_plan;
pub use types::PlannedBuilding;
pub use types::{BuildingContext, BuildingEnvironment};

pub mod renderer;
pub use renderer::generate_intelligent_building_interior;
