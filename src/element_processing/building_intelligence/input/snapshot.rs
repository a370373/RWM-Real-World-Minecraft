use crate::element_processing::subprocessor::interior::BuildingType;

#[derive(Debug, Clone, Copy)]
pub struct ExistingWindow {
    pub x: i32,
    pub z: i32,
    pub width: i32,
    pub height: i32,
    pub floor: i32,
    pub side: WindowSide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowSide {
    North,
    South,
    East,
    West,
}

#[derive(Debug, Clone, Copy)]
pub struct ExistingDoor {
    pub x: i32,
    pub z: i32,
    pub width: i32,
    pub side: WindowSide,
}

#[derive(Debug, Clone)]
pub struct BuildingSnapshot {
    pub building_type: BuildingType,

    pub min_x: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_z: i32,

    pub floors: usize,

    /// Existing vertical placement from the world reconstruction engine.
    /// Interior intelligence may READ this value only.
    pub start_y_offset: i32,

    /// Existing geometry from the world reconstruction engine.
    /// Interior intelligence may READ these values only.
    pub windows: Vec<ExistingWindow>,
    pub doors: Vec<ExistingDoor>,

    /// Existing real-world semantic tags.
    pub osm_tags: Vec<(String, String)>,

    /// Existing exterior context.
    pub nearby_road_distance: Option<i32>,
    /// Nearest reconstructed road block coordinate. READ-ONLY.
    pub nearby_road_position: Option<(i32, i32)>,
    pub nearby_path_distance: Option<i32>,
    pub nearby_parking_distance: Option<i32>,
}

impl BuildingSnapshot {
    pub fn width(&self) -> i32 {
        self.max_x - self.min_x + 1
    }

    pub fn depth(&self) -> i32 {
        self.max_z - self.min_z + 1
    }

    pub fn area(&self) -> i32 {
        self.width() * self.depth()
    }
}
