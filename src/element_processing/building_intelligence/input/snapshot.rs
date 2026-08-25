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

#[derive(Debug, Clone)]
pub struct ExistingDoor {
    pub x: i32,
    pub z: i32,
    pub width: i32,
    pub side: WindowSide,

    /// Original real-world semantic tags carried by the mapped node.
    /// READ-ONLY reconstruction input.
    pub tags: Vec<(String, String)>,

    /// Semantic source of the mapped door.
    ///
    /// This records why the node was accepted as a real-world door:
    /// - entrance=* / entrance
    /// - door=* / door
    /// - both
    pub source: DoorSource,

    /// Manhattan distance from this specific door to the nearest
    /// reconstructed road evidence, when available.
    pub road_distance: Option<i32>,

    /// Manhattan distance from this specific door to the nearest
    /// reconstructed footway/path evidence, when available.
    ///
    /// Kept optional until the real footway/path source is wired.
    pub footway_distance: Option<i32>,

    /// Manhattan distance from this specific door to the nearest
    /// reconstructed parking evidence, when available.
    ///
    /// Kept optional until the real parking source is wired.
    pub parking_distance: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorSource {
    Entrance,
    Door,
    EntranceAndDoor,
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
