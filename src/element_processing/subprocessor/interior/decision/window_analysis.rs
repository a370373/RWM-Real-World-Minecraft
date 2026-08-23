/// Information about a window that already exists on the generated
/// real-world building exterior.
///
/// This is NOT a window-generation structure.
/// It describes existing exterior openings so the interior decision
/// layer can reason about daylight and room placement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowInfo {
    /// Minecraft X coordinate of the window.
    pub x: i32,

    /// Minecraft Z coordinate of the window.
    pub z: i32,

    /// Floor containing the window.
    pub floor: i32,

    /// Width of the opening in blocks.
    pub width: i32,

    /// Height of the opening in blocks.
    pub height: i32,

    /// Direction the window faces.
    ///
    /// 0 = north
    /// 1 = east
    /// 2 = south
    /// 3 = west
    pub facing: u8,

    /// Whether this window came from explicitly mapped real-world data.
    pub mapped: bool,

    /// Estimated daylight contribution.
    ///
    /// This is intentionally kept as analysis data rather than
    /// directly controlling Minecraft blocks.
    pub daylight_score: f32,
}

impl WindowInfo {
    pub fn area(&self) -> i32 {
        self.width.max(0) * self.height.max(0)
    }

    pub fn is_large(&self) -> bool {
        self.area() >= 6
    }

    pub fn is_small(&self) -> bool {
        self.area() <= 2
    }

    pub fn daylight_weight(&self) -> f32 {
        self.daylight_score.max(0.0) * self.area() as f32
    }
}

/// Aggregated daylight information for one side of a building.
#[derive(Debug, Clone, Copy, Default)]
pub struct FacadeDaylight {
    pub north: f32,
    pub east: f32,
    pub south: f32,
    pub west: f32,
}

impl FacadeDaylight {
    pub fn total(&self) -> f32 {
        self.north + self.east + self.south + self.west
    }

    pub fn strongest_facing(&self) -> Option<u8> {
        let values = [self.north, self.east, self.south, self.west];

        let mut best: Option<(u8, f32)> = None;

        for (facing, value) in values.into_iter().enumerate() {
            if value <= 0.0 {
                continue;
            }

            match best {
                None => best = Some((facing as u8, value)),
                Some((_, best_value)) if value > best_value => {
                    best = Some((facing as u8, value));
                }
                _ => {}
            }
        }

        best.map(|(facing, _)| facing)
    }
}

/// Analyze the existing windows of a building.
///
/// This function does not create, remove, move, or modify windows.
/// It only converts existing window information into daylight data.
pub fn analyze_daylight(windows: &[WindowInfo]) -> FacadeDaylight {
    let mut result = FacadeDaylight::default();

    for window in windows {
        let weight = window.daylight_weight();

        match window.facing {
            0 => result.north += weight,
            1 => result.east += weight,
            2 => result.south += weight,
            3 => result.west += weight,
            _ => {}
        }
    }

    result
}
