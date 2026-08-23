use super::window_analysis::WindowInfo;
use crate::element_processing::subprocessor::interior::BuildingType;

/// Real-world information extracted from OSM / Overture and the
/// already-generated Minecraft building shell.
///
/// This is an INPUT to the interior decision layer.
/// It must not generate or modify the exterior building.
#[derive(Debug, Clone)]
pub struct BuildingProfile {
    pub building_type: BuildingType,

    pub min_x: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_z: i32,

    pub floors: i32,

    pub osm_tags: Vec<(String, String)>,

    pub entrances: Vec<EntranceInfo>,

    pub windows: Vec<WindowInfo>,
}

impl BuildingProfile {
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

    pub fn primary_entrance(&self) -> Option<&EntranceInfo> {
        let center = self.center();

        self.entrances.iter().min_by_key(|entrance| {
            let dx = entrance.x - center.0;
            let dz = entrance.z - center.1;
            dx * dx + dz * dz
        })
    }
}

/// A real-world entrance/door extracted from OSM / Overture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntranceInfo {
    pub x: i32,
    pub z: i32,

    pub floor: i32,

    /// 0 = north, 1 = east, 2 = south, 3 = west.
    pub facing: u8,

    /// True when the entrance was explicitly mapped.
    pub mapped: bool,
}
