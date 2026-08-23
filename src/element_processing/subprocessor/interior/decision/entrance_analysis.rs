use super::building_profile::BuildingProfile;

/// Decision-layer representation of an already-existing real-world entrance.
///
/// This does NOT create or move a door.
/// It only describes how the interior generator should use the
/// entrance that was already obtained from OSM / Overture.
#[derive(Debug, Clone)]
pub struct EntranceDecision {
    pub entrance: super::building_profile::EntranceInfo,
    pub preferred_room: Option<crate::element_processing::subprocessor::interior::RoomType>,
    pub is_primary: bool,
    pub circulation_priority: u8,
}

/// Analyze an existing building entrance.
///
/// The entrance geometry/source comes from the real-world building data.
/// This function only decides how the interior should connect to it.
pub fn analyze_entrance(profile: &BuildingProfile) -> Option<EntranceDecision> {
    let entrance = profile.primary_entrance().copied()?;

    let preferred_room = if profile.building_type.is_residential() {
        Some(crate::element_processing::subprocessor::interior::RoomType::LivingRoom)
    } else if profile.building_type.is_commercial() {
        Some(crate::element_processing::subprocessor::interior::RoomType::ProductArea)
    } else {
        None
    };

    Some(EntranceDecision {
        entrance,
        preferred_room,
        is_primary: true,
        circulation_priority: 10,
    })
}
