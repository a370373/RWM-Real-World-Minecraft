use crate::element_processing::subprocessor::interior::RoomType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FurnitureKind {
    Bed,
    Sofa,
    Table,
    Chair,
    KitchenCounter,
    Sink,
    Toilet,
    Shower,
    Bathtub,
    Desk,
    Bookshelf,
    Checkout,
    Shelf,
    HospitalBed,
    MedicalDesk,
    ClassroomDesk,
    StorageShelf,
    DiningTable,
}

#[derive(Debug, Clone, Copy)]
pub struct FurnitureItem {
    /// Furniture belongs to one concrete room.
    /// This prevents furniture in separate rooms of the same
    /// semantic type from colliding with each other.
    pub room_id: usize,

    pub kind: FurnitureKind,
    pub room_type: RoomType,

    /// Position relative to the owning room's bounds.
    pub relative_x: i32,
    pub relative_z: i32,
}

pub fn furniture_profile(room_type: RoomType) -> &'static [FurnitureKind] {
    match room_type {
        RoomType::LivingRoom => &[
            FurnitureKind::Sofa,
            FurnitureKind::Table,
            FurnitureKind::Chair,
        ],

        RoomType::Kitchen => &[
            FurnitureKind::KitchenCounter,
            FurnitureKind::Sink,
            FurnitureKind::Table,
        ],

        RoomType::Bedroom => &[FurnitureKind::Bed, FurnitureKind::Table],

        RoomType::Bathroom => &[
            FurnitureKind::Toilet,
            FurnitureKind::Shower,
            FurnitureKind::Sink,
        ],

        RoomType::DiningArea => &[FurnitureKind::DiningTable, FurnitureKind::Chair],

        RoomType::Classroom => &[FurnitureKind::ClassroomDesk, FurnitureKind::Chair],

        RoomType::Office => &[FurnitureKind::Desk, FurnitureKind::Chair],

        RoomType::MeetingRoom => &[FurnitureKind::Table, FurnitureKind::Chair],

        RoomType::Ward => &[FurnitureKind::HospitalBed, FurnitureKind::MedicalDesk],

        RoomType::ExaminationRoom => &[FurnitureKind::HospitalBed, FurnitureKind::MedicalDesk],

        RoomType::NursingStation => &[FurnitureKind::Desk, FurnitureKind::Chair],

        RoomType::ProductArea => &[FurnitureKind::Shelf],

        RoomType::Checkout => &[FurnitureKind::Checkout],

        RoomType::Storage => &[FurnitureKind::StorageShelf],

        RoomType::Corridor | RoomType::Toilet | RoomType::BreakRoom => &[],
        _ => &[],
    }
}
