use super::building_profile::{BuildingProfile, EntranceInfo};
use super::room_allocation::RoomAllocation;
use super::window_analysis::WindowInfo;

use crate::element_processing::subprocessor::interior::{Rect, RoomType};

/// Existing real-world window constraint.
///
/// This is READ-ONLY information originating from the already
/// reconstructed building shell.
#[derive(Debug, Clone, Copy)]
pub struct ExistingWindowConstraint {
    pub x: i32,
    pub z: i32,
    pub floor: i32,
    pub width: i32,
    pub height: i32,
    pub facing: u8,
}

/// Existing real-world entrance constraint.
///
/// This does NOT create or move the entrance.
#[derive(Debug, Clone, Copy)]
pub struct ExistingEntranceConstraint {
    pub x: i32,
    pub z: i32,
    pub floor: i32,
    pub facing: u8,
    pub mapped: bool,
}

/// Spatial constraints extracted from the existing real-world
/// building.
///
/// IMPORTANT:
/// This structure is decision-only.
/// It never modifies the reconstructed building.
#[derive(Debug, Clone)]
pub struct SpatialConstraints {
    pub bounds: Rect,
    pub windows: Vec<ExistingWindowConstraint>,
    pub entrances: Vec<ExistingEntranceConstraint>,
}

impl SpatialConstraints {
    /// Build spatial constraints from the real-world building profile.
    ///
    /// No geometry is generated or modified here.
    pub fn from_profile(profile: &BuildingProfile) -> Self {
        let bounds = Rect {
            min_x: profile.min_x,
            min_z: profile.min_z,
            max_x: profile.max_x,
            max_z: profile.max_z,
        };

        let windows = profile
            .windows
            .iter()
            .map(|window: &WindowInfo| ExistingWindowConstraint {
                x: window.x,
                z: window.z,
                floor: window.floor,
                width: window.width,
                height: window.height,
                facing: window.facing,
            })
            .collect();

        let entrances = profile
            .entrances
            .iter()
            .map(|entrance: &EntranceInfo| ExistingEntranceConstraint {
                x: entrance.x,
                z: entrance.z,
                floor: entrance.floor,
                facing: entrance.facing,
                mapped: entrance.mapped,
            })
            .collect();

        Self {
            bounds,
            windows,
            entrances,
        }
    }

    pub fn has_windows(&self) -> bool {
        !self.windows.is_empty()
    }

    pub fn has_entrances(&self) -> bool {
        !self.entrances.is_empty()
    }

    /// Returns true when a room touches an existing window.
    ///
    /// The window itself is never modified.
    pub fn touches_window(&self, room: Rect, floor: i32) -> bool {
        self.windows
            .iter()
            .any(|window| window.floor == floor && point_touches_rect(room, window.x, window.z))
    }

    /// Returns the number of existing windows touching a room.
    pub fn window_count(&self, room: Rect, floor: i32) -> usize {
        self.windows
            .iter()
            .filter(|window| {
                window.floor == floor
                    && window.width > 0
                    && window.height > 0
                    && window.facing <= 3
                    && point_touches_rect(room, window.x, window.z)
            })
            .count()
    }

    /// Returns the number of existing entrances touching a room.
    pub fn entrance_count(&self, room: Rect, floor: i32) -> usize {
        self.entrances
            .iter()
            .filter(|entrance| {
                entrance.floor == floor && point_touches_rect(room, entrance.x, entrance.z)
            })
            .count()
    }

    /// Score how well a room satisfies the spatial constraints.
    ///
    /// Higher is better.
    ///
    /// This function only evaluates a proposed room rectangle.
    pub fn score_room(&self, room: Rect, allocation: &RoomAllocation, floor: i32) -> i32 {
        if !room_inside(room, self.bounds) {
            return i32::MIN;
        }

        if !room.can_fit(allocation.min_width, allocation.min_depth) {
            return i32::MIN;
        }

        let mut score = 0;

        // -----------------------------------------------------
        // AREA
        // -----------------------------------------------------

        let area = room.area();

        if area >= allocation.required_area {
            score += 40;
        } else {
            score -= (allocation.required_area - area) * 4;
        }

        // -----------------------------------------------------
        // EXISTING REAL-WORLD WINDOWS / DAYLIGHT
        // -----------------------------------------------------
        //
        // Windows are READ-ONLY geometry from the already
        // reconstructed building shell.
        //
        // Interior Intelligence does not create, move, remove,
        // or resize these windows.
        //

        // Consume geometry/orientation from already-existing windows.
        // These values are analysis inputs only.
        let existing_window_area: i32 = self
            .windows
            .iter()
            .filter(|window| window.floor == floor && point_touches_rect(room, window.x, window.z))
            .map(|window| window.width.max(0) * window.height.max(0))
            .sum();

        let facing_diversity = self
            .windows
            .iter()
            .filter(|window| {
                window.floor == floor
                    && point_touches_rect(room, window.x, window.z)
                    && window.facing <= 3
            })
            .map(|window| window.facing)
            .collect::<std::collections::BTreeSet<_>>()
            .len();

        let windows = self.window_count(room, floor);
        let touches_window = self.touches_window(room, floor);

        let mut window_area = 0i32;
        let mut facing_bonus = 0i32;
        let mut height_bonus = 0i32;

        for window in self
            .windows
            .iter()
            .filter(|window| window.floor == floor && point_touches_rect(room, window.x, window.z))
        {
            // Existing opening size contributes to daylight quality.
            window_area += window.width.max(0) * window.height.max(0);

            // Larger existing windows provide stronger daylight evidence.
            if window.height >= 2 {
                height_bonus += 5;
            }

            // Facing remains analysis-only information.
            // No exterior geometry is changed.
            //
            // All facades are valid; this simply rewards a known,
            // explicitly mapped orientation.
            if window.facing <= 3 {
                facing_bonus += 2;
            }
        }

        if allocation.daylight_required {
            if existing_window_area > 0 {
                score += (existing_window_area.min(64) / 4) as i32;
            }

            score += (facing_diversity.min(4) as i32) * 3;
            if windows > 0 {
                score += 100;
                score += (windows as i32).min(4) * 10;

                if touches_window {
                    score += 20;
                }

                score += window_area.min(40);
                score += height_bonus;
                score += facing_bonus;
            } else {
                score -= 150;
            }
        } else if windows > 0 {
            // Non-daylight rooms may still benefit from existing
            // exterior windows.
            score += 10;
            score += window_area.min(20);
            score += height_bonus;
        }

        // -----------------------------------------------------
        // ENTRANCE RELATIONSHIP
        // -----------------------------------------------------

        let entrances = self.entrance_count(room, floor);

        if is_entrance_room(allocation.room_type) {
            if entrances > 0 {
                score += 140;
            } else if !self.entrances.is_empty() {
                score -= 80;
            }
        } else if entrances > 0 {
            // Ordinary rooms touching the main entrance are not
            // forbidden, but are less desirable.
            score -= 15;
        }

        // -----------------------------------------------------
        // ROOM SEMANTIC PREFERENCES
        // -----------------------------------------------------

        if prefers_daylight(allocation.room_type) && windows > 0 {
            score += 30;
        }

        if prefers_interior(allocation.room_type) && windows == 0 {
            score += 20;
        }

        // -----------------------------------------------------
        // ROOM CLASS SEMANTICS
        // -----------------------------------------------------
        //
        // These semantic classifications come directly from
        // RoomType. They influence candidate placement without
        // changing the real-world building geometry.
        //

        if allocation.room_type.is_public_room() {
            if windows > 0 {
                score += 4;
            }

            if entrances > 0 {
                score += 4;
            }
        }

        if allocation.room_type.is_private_room() {
            // Private rooms benefit from avoiding direct entrance
            // exposure while remaining geometrically valid.
            if entrances == 0 {
                score += 4;
            } else {
                score -= 4;
            }
        }

        if allocation.room_type.is_service_room() {
            // Service rooms naturally prefer interior locations.
            if windows == 0 {
                score += 5;
            }

            if entrances == 0 {
                score += 2;
            }
        }

        // -----------------------------------------------------
        // EXTERIOR ACCESS SEMANTICS
        // -----------------------------------------------------
        //
        // Loading/platform areas are operational spaces.
        // They strongly prefer ground level and contact with
        // the existing building boundary.
        //

        if prefers_exterior_access(allocation.room_type) {
            if floor == 0 {
                score += 12;
            } else {
                score -= 12;
            }

            let touches_boundary =
                room.min_x == self.bounds.min_x
                    || room.max_x == self.bounds.max_x
                    || room.min_z == self.bounds.min_z
                    || room.max_z == self.bounds.max_z;

            if touches_boundary {
                score += 18;
            } else {
                score -= 6;
            }
        }

        // -----------------------------------------------------
        // HEALTHCARE SEMANTICS
        // -----------------------------------------------------
        //
        // Healthcare rooms are treated as a semantic family.
        // Nursing stations receive a stronger preference for
        // operationally central healthcare placement.
        //
        // This does not alter geometry or create adjacency.
        //

        if is_healthcare_room(allocation.room_type) {
            if windows > 0
                && matches!(
                    allocation.room_type,
                    RoomType::Ward
                        | RoomType::ExaminationRoom
                        | RoomType::TreatmentRoom
                        | RoomType::WaitingArea
                )
            {
                score += 6;
            }
        }

        if is_healthcare_support_room(allocation.room_type) {
            match allocation.room_type {
                RoomType::NursingStation => {
                    // Nursing stations are operational support spaces.
                    // Prefer visible/accessible positions rather than
                    // deep service-room placement.
                    if windows > 0 {
                        score += 3;
                    }

                    if entrances > 0 {
                        score += 6;
                    }

                    if windows == 0 && entrances == 0 {
                        score -= 4;
                    }
                }

                RoomType::Ward
                | RoomType::ExaminationRoom
                | RoomType::TreatmentRoom => {
                    // Clinical rooms benefit from daylight when
                    // available, but this remains a soft preference.
                    if windows > 0 {
                        score += 4;
                    }
                }

                _ => {}
            }
        }

        // -----------------------------------------------------
        // PRIORITY
        // -----------------------------------------------------

        score += allocation.priority as i32 * 5;

        score
    }
}

/// Convenience wrapper used by layout code.
pub fn score_room(
    room: Rect,
    allocation: &RoomAllocation,
    constraints: &SpatialConstraints,
    floor: i32,
) -> i32 {
    constraints.score_room(room, allocation, floor)
}

fn room_inside(room: Rect, bounds: Rect) -> bool {
    room.min_x >= bounds.min_x
        && room.max_x <= bounds.max_x
        && room.min_z >= bounds.min_z
        && room.max_z <= bounds.max_z
}

fn point_touches_rect(room: Rect, x: i32, z: i32) -> bool {
    // A window/entrance is associated with a room only when its
    // mapped point actually lies on or inside that room rectangle.
    //
    // Do not compare individual axes independently: doing so would
    // incorrectly associate unrelated windows/entrances that merely
    // share the same X or Z coordinate.
    room.contains(x, z)
}

fn is_entrance_room(room_type: RoomType) -> bool {
    matches!(
        room_type,
        RoomType::EntranceHall
            | RoomType::Reception
            | RoomType::LivingRoom
            | RoomType::ProductArea
            | RoomType::WaitingArea
            | RoomType::DiningArea
            | RoomType::Counter
            | RoomType::ExhibitionArea
            | RoomType::CommunityRoom
            | RoomType::PrayerRoom
            | RoomType::PlatformArea
            | RoomType::ReadingArea
    )
}

fn prefers_daylight(room_type: RoomType) -> bool {
    matches!(
        room_type,
        RoomType::LivingRoom
            | RoomType::Bedroom
            | RoomType::DiningRoom
            | RoomType::DiningArea
            | RoomType::ProductArea
            | RoomType::DisplayArea
            | RoomType::Office
            | RoomType::MeetingRoom
            | RoomType::Classroom
            | RoomType::Library
            | RoomType::ReadingArea
            | RoomType::WaitingArea
            | RoomType::ExhibitionArea
            | RoomType::CommunityRoom
            | RoomType::Ward
            | RoomType::ProductionArea
            | RoomType::ExaminationRoom
            | RoomType::TreatmentRoom
            | RoomType::NursingStation
            | RoomType::PrayerRoom
            | RoomType::PlatformArea
    )
}

fn is_healthcare_room(room_type: RoomType) -> bool {
    matches!(
        room_type,
        RoomType::Ward
            | RoomType::ExaminationRoom
            | RoomType::TreatmentRoom
            | RoomType::NursingStation
            | RoomType::WaitingArea
            | RoomType::Pharmacy
    )
}

fn is_healthcare_support_room(room_type: RoomType) -> bool {
    matches!(
        room_type,
        RoomType::NursingStation
            | RoomType::ExaminationRoom
            | RoomType::TreatmentRoom
            | RoomType::Ward
    )
}

fn prefers_exterior_access(room_type: RoomType) -> bool {
    matches!(
        room_type,
        RoomType::LoadingArea
            | RoomType::PlatformArea
    )
}

fn prefers_interior(room_type: RoomType) -> bool {
    matches!(
        room_type,
        RoomType::Bathroom
            | RoomType::Toilet
            | RoomType::Storage
            | RoomType::UtilityRoom
            | RoomType::Laundry
            | RoomType::ServerRoom
            | RoomType::KitchenService
            | RoomType::ServiceArea
    )
}
