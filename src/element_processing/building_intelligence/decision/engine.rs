use super::{
    decide_main_door, decide_room_door, decide_vertical_access, DoorDecision,
    VerticalAccessDecision,
};

use crate::element_processing::building_intelligence::{BuildingContext, EntranceCandidate};

use crate::element_processing::subprocessor::interior::decision::entrance_analysis::EntranceDecision;
use crate::element_processing::subprocessor::interior::decision::{
    allocate_rooms, analyze_daylight, BuildingProfile, FacadeDaylight, RoomAllocationPlan,
};

#[derive(Debug, Clone)]
pub struct BuildingDecision {
    /// Rooms selected by the central interior intelligence.
    pub rooms: RoomAllocationPlan,

    /// Daylight information derived ONLY from already-existing
    /// real-world windows.
    pub daylight: FacadeDaylight,

    /// Vertical circulation decision.
    pub vertical: VerticalAccessDecision,

    /// Main real-world entrance decision.
    pub main_door: Option<DoorDecision>,

    /// Entrance analysis derived from the existing real-world entrance.
    pub entrance: Option<EntranceDecision>,

    /// Semantic doors for generated rooms.
    pub room_doors: Vec<DoorDecision>,
}

fn apply_entrance_preference(
    mut rooms: RoomAllocationPlan,
    entrance: Option<&EntranceDecision>,
) -> RoomAllocationPlan {
    let Some(entrance) = entrance else {
        return rooms;
    };

    if !entrance.is_primary {
        return rooms;
    }

    let Some(preferred_room) = entrance.preferred_room else {
        return rooms;
    };

    for room in &mut rooms.rooms {
        if room.room_type == preferred_room {
            room.priority = room.priority.saturating_add(entrance.circulation_priority);
        }
    }

    rooms
}

/// Central building-intelligence decision.
///
/// IMPORTANT:
/// - Does NOT generate windows.
/// - Does NOT modify the exterior.
/// - Does NOT modify the BBox.
/// - Existing OSM / Overture windows are INPUT only.
/// - Daylight analysis informs room allocation.
pub fn decide_building(
    context: &BuildingContext,
    entrance: Option<&EntranceCandidate>,
    profile: &BuildingProfile,
    entrance_decision: Option<EntranceDecision>,
) -> BuildingDecision {
    // ---------------------------------------------------------
    // 1. EXISTING REAL-WORLD WINDOWS -> DAYLIGHT ANALYSIS
    // ---------------------------------------------------------
    //
    // BuildingProfile.windows contains windows that already
    // belong to the real-world building shell.
    //
    // This layer does NOT create, move, remove, or modify them.
    //
    let daylight = analyze_daylight(&profile.windows);

    // ---------------------------------------------------------
    // 2. ROOM ALLOCATION
    // ---------------------------------------------------------
    //
    // Building type + footprint + existing daylight determine
    // which rooms should exist.
    //
    let spatial_constraints =
        crate::element_processing::subprocessor::interior::decision::SpatialConstraints::from_profile(profile);

    let mut rooms = crate::element_processing::subprocessor::interior::decision::room_allocation::allocate_rooms_with_constraints(
        profile,
        &daylight,
        &spatial_constraints,
    );

    rooms = apply_entrance_preference(rooms, entrance_decision.as_ref());

    // ---------------------------------------------------------
    // 3. VERTICAL ACCESS
    // ---------------------------------------------------------

    let vertical = decide_vertical_access(
        context.building_type,
        context.width(),
        context.depth(),
        context.floors,
        rooms.rooms.len(),
    );

    // ---------------------------------------------------------
    // 4. MAIN REAL-WORLD ENTRANCE
    // ---------------------------------------------------------

    let main_door = decide_main_door(entrance);

    // ---------------------------------------------------------
    // 5. ROOM DOORS
    // ---------------------------------------------------------

    let room_doors = rooms
        .rooms
        .iter()
        .map(|room| decide_room_door(room.room_type))
        .collect();

    BuildingDecision {
        rooms,
        daylight,
        vertical,
        main_door,
        entrance: entrance_decision,
        room_doors,
    }
}
