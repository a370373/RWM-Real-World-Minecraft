use super::building_profile::BuildingProfile;
use super::spatial_constraints::SpatialConstraints;
use super::window_analysis::FacadeDaylight;
use crate::element_processing::subprocessor::interior::{room_profile, RoomType};

/// A room selected by the interior decision layer.
///
/// This describes what the room SHOULD be.
/// It does not place blocks or modify the building.
#[derive(Debug, Clone)]
pub struct RoomAllocation {
    pub room_type: RoomType,
    pub required_area: i32,
    pub min_width: i32,
    pub min_depth: i32,
    pub preferred_floor: Option<i32>,
    pub daylight_required: bool,
    pub priority: u8,
    /// Whether this semantic room is preferred to receive the
    /// already-existing primary real-world entrance.
    pub entrance_preferred: bool,
}

/// Complete room decision for a building.
#[derive(Debug, Clone)]
pub struct RoomAllocationPlan {
    pub rooms: Vec<RoomAllocation>,
}

impl RoomAllocationPlan {
    pub fn total_required_area(&self) -> i32 {
        self.rooms.iter().map(|room| room.required_area).sum()
    }

    pub fn rooms_of_type(&self, room_type: RoomType) -> usize {
        self.rooms
            .iter()
            .filter(|room| room.room_type == room_type)
            .count()
    }
}

/// Building-Type Interior Intelligence.
///
/// This layer translates the already-detected real-world
/// BuildingType into a semantic interior program.
///
/// IMPORTANT:
/// - Does NOT generate windows.
/// - Does NOT modify doors/windows of the exterior.
/// - Does NOT modify the building footprint.
/// - Does NOT modify the BBox.
/// - Uses existing daylight information only as a constraint.
/// - Produces semantic room requirements for downstream systems.
pub fn allocate_rooms(profile: &BuildingProfile, daylight: &FacadeDaylight) -> RoomAllocationPlan {
    let requirements = room_profile(profile.building_type);

    if requirements.is_empty() {
        return fallback_unknown(profile);
    }

    let available_area = profile.area().max(1);

    let entrance_preferred_room = profile.primary_entrance().and_then(|_| {
        if profile.building_type.is_residential() {
            Some(RoomType::LivingRoom)
        } else if profile.building_type.is_commercial() {
            Some(RoomType::ProductArea)
        } else {
            None
        }
    });

    let mut rooms = Vec::new();
    let mut used_area = 0i32;

    /*
     * Allocate higher-priority semantic requirements first.
     *
     * This is important because room_profile() describes semantics,
     * not physical placement order. High-priority rooms must get
     * first access to the real building's available area.
     */
    let mut ordered_requirements = requirements;
    ordered_requirements.sort_by(|a, b| {
        b.priority
            .cmp(&a.priority)
            .then_with(|| b.min_area.cmp(&a.min_area))
    });

    for requirement in ordered_requirements {
        if requirement.min_count == 0 {
            continue;
        }

        /*
         * Calculate capacity from the ACTUAL remaining area.
         *
         * Never use the original building area here once other rooms
         * have already consumed part of it.
         */
        let remaining_area = (available_area - used_area).max(0);

        let max_by_remaining = (remaining_area / requirement.min_area.max(1)) as usize;

        /*
         * If even the minimum semantic count cannot fit anymore,
         * skip this requirement rather than manufacturing impossible
         * rooms or making the entire allocation fail.
         */
        if max_by_remaining < requirement.min_count {
            continue;
        }

        let target_count = requirement.max_count.min(max_by_remaining);

        for index in 0..target_count {
            let remaining = available_area - used_area;

            if remaining < requirement.min_area {
                break;
            }

            /*
             * Once the minimum semantic count has been satisfied,
             * additional instances are optional and only consume
             * genuinely available area.
             */
            if index >= requirement.min_count && remaining < requirement.min_area {
                break;
            }

            let (min_width, min_depth) = minimum_dimensions(requirement.min_area);

            let daylight_required =
                daylight.total() > 0.0 && naturally_lit_room(requirement.room_type);

            rooms.push(RoomAllocation {
                room_type: requirement.room_type,
                required_area: requirement.min_area,
                min_width,
                min_depth,
                preferred_floor: preferred_floor(profile, requirement.room_type),
                daylight_required,
                priority: requirement.priority,
                entrance_preferred: entrance_preferred_room == Some(requirement.room_type),
            });

            used_area += requirement.min_area;
        }
    }

    /*
     * If daylight exists, make sure the building has at least
     * one room that can meaningfully use it when possible.
     *
     * This does not create or move windows.
     */
    if daylight.total() > 0.0 && !rooms.iter().any(|room| room.daylight_required) {
        if let Some(room) = rooms.first_mut() {
            room.daylight_required = true;
        }
    }

    /*
     * Highest-priority semantic rooms should be considered first
     * by downstream spatial allocation.
     */
    rooms.sort_by(|a, b| {
        b.priority
            .cmp(&a.priority)
            .then_with(|| b.required_area.cmp(&a.required_area))
    });

    RoomAllocationPlan { rooms }
}

/// Apply existing-building spatial constraints to the semantic room plan.
///
/// This only reads the reconstructed building shell.
/// It does not create, move, remove, or modify windows, doors,
/// walls, footprint, or BBox.
pub fn allocate_rooms_with_constraints(
    profile: &BuildingProfile,
    daylight: &FacadeDaylight,
    constraints: &SpatialConstraints,
) -> RoomAllocationPlan {
    let mut plan = allocate_rooms(profile, daylight);

    if constraints.has_windows() {
        for room in &mut plan.rooms {
            if naturally_lit_room(room.room_type) {
                room.daylight_required = true;
            }
        }
    }

    plan
}

/// Conservative fallback for buildings whose BuildingType is unknown.
fn fallback_unknown(profile: &BuildingProfile) -> RoomAllocationPlan {
    if profile.area() < 16 {
        return RoomAllocationPlan { rooms: Vec::new() };
    }

    let mut rooms = Vec::new();

    rooms.push(RoomAllocation {
        room_type: RoomType::Corridor,
        required_area: profile.area().min(16),
        min_width: 2,
        min_depth: 2,
        preferred_floor: Some(0),
        daylight_required: false,
        priority: 5,
        entrance_preferred: false,
    });

    RoomAllocationPlan { rooms }
}

/// Infer sensible minimum dimensions from the semantic room area.
///
/// This is only an interior planning constraint.
/// It does not change the real-world building geometry.
fn minimum_dimensions(area: i32) -> (i32, i32) {
    match area {
        0..=4 => (2, 2),
        5..=8 => (2, 3),
        9..=15 => (3, 3),
        16..=24 => (4, 4),
        25..=39 => (5, 5),
        40..=59 => (6, 6),
        60..=99 => (7, 7),
        _ => (8, 8),
    }
}

/// Rooms that benefit strongly from existing natural light.
///
/// This function never creates windows. It only tells spatial
/// allocation which room types should prefer existing daylight.
fn naturally_lit_room(room_type: RoomType) -> bool {
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
            | RoomType::Laboratory
            | RoomType::Library
            | RoomType::Ward
            | RoomType::ExaminationRoom
            | RoomType::TreatmentRoom
            | RoomType::WaitingArea
            | RoomType::ExhibitionArea
            | RoomType::CommunityRoom
            | RoomType::ProductionArea
            | RoomType::Workshop
    )
}

/// Give semantic rooms sensible floor preferences.
///
/// The actual floor-plan generator remains responsible for
/// deciding whether the requested placement is geometrically possible.
fn preferred_floor(profile: &BuildingProfile, room_type: RoomType) -> Option<i32> {
    let last_floor = (profile.floors - 1).max(0);

    match room_type {
        RoomType::LivingRoom
        | RoomType::DiningRoom
        | RoomType::DiningArea
        | RoomType::ProductArea
        | RoomType::DisplayArea
        | RoomType::Checkout
        | RoomType::Reception
        | RoomType::WaitingArea
        | RoomType::ExhibitionArea
        | RoomType::CommunityRoom
        | RoomType::ReadingArea
        | RoomType::PrayerRoom
        | RoomType::PlatformArea
        | RoomType::ProductionArea
        | RoomType::Workshop
        | RoomType::LoadingArea
        | RoomType::EntranceHall => Some(0),

        RoomType::Bedroom
        | RoomType::Bathroom
        | RoomType::Office
        | RoomType::ExaminationRoom
        | RoomType::TreatmentRoom
        | RoomType::NursingStation
        | RoomType::MeetingRoom
        | RoomType::Classroom
        | RoomType::Library
        | RoomType::Laboratory
        | RoomType::Ward => {
            if profile.floors > 1 {
                Some(last_floor.min(2))
            } else {
                Some(0)
            }
        }

        _ => Some(0),
    }
}
