pub mod building_profile;
pub mod circulation;
pub mod entrance_analysis;
pub mod furniture;
pub mod lighting;
pub mod room_allocation;
pub mod spatial_constraints;
pub mod window_analysis;

pub use building_profile::{BuildingProfile, EntranceInfo};
pub use window_analysis::{analyze_daylight, FacadeDaylight, WindowInfo};

pub use room_allocation::{allocate_rooms, RoomAllocation, RoomAllocationPlan};

pub use spatial_constraints::{score_room, SpatialConstraints};

pub use lighting::{plan_lighting, LightKind, LightingPlan};
