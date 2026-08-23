use crate::element_processing::building_intelligence::{EntranceCandidate, EntranceSide};
use crate::element_processing::subprocessor::interior::RoomType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorKind {
    MainEntrance,
    Interior,
    Service,
}

#[derive(Debug, Clone, Copy)]
pub struct DoorDecision {
    pub kind: DoorKind,
    pub width: i32,
    pub room_type: Option<RoomType>,
    pub side: Option<EntranceSide>,

    /// Preferred world/Minecraft coordinate for the doorway.
    ///
    /// This is decision-layer intent only.
    /// The renderer places the actual door blocks.
    pub x: Option<i32>,
    pub z: Option<i32>,
}

pub fn decide_main_door(entrance: Option<&EntranceCandidate>) -> Option<DoorDecision> {
    let entrance = entrance?;

    Some(DoorDecision {
        kind: DoorKind::MainEntrance,
        width: 2,
        room_type: None,
        side: Some(entrance.side),

        // Existing real-world entrance coordinate.
        // Never move or regenerate the exterior entrance.
        x: Some(entrance.x),
        z: Some(entrance.z),
    })
}

pub fn decide_room_door(room_type: RoomType) -> DoorDecision {
    let width = match room_type {
        RoomType::LivingRoom
        | RoomType::DiningRoom
        | RoomType::DiningArea
        | RoomType::Corridor
        | RoomType::PlatformArea => 2,

        RoomType::Bedroom
        | RoomType::Kitchen
        | RoomType::Bathroom
        | RoomType::Toilet
        | RoomType::Storage
        | RoomType::Office => 1,

        _ => 1,
    };

    DoorDecision {
        kind: DoorKind::Interior,
        width,
        room_type: Some(room_type),
        side: None,
        x: None,
        z: None,
    }
}
