pub mod building_type;
pub mod floor_plan;
pub mod room_profile;
pub mod room_type;

pub use building_type::BuildingType;
pub use floor_plan::{FloorPlan, Rect, Room};
pub use room_profile::room_profile;
pub use room_type::RoomType;

// Interior decision layer: analyzes the already-generated real-world building.
pub mod decision;
