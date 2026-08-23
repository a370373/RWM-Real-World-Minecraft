#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoomType {
    // Residential
    LivingRoom,
    Kitchen,
    Bedroom,
    Bathroom,
    Toilet,
    DiningRoom,
    Balcony,
    Hallway,
    EntranceHall,
    Laundry,
    Storage,
    UtilityRoom,

    // Restaurant / Food
    DiningArea,
    KitchenService,
    Counter,
    StaffRoom,
    ServiceArea,

    // Commercial
    ProductArea,
    DisplayArea,
    Checkout,

    // Office
    Office,
    Reception,
    MeetingRoom,
    BreakRoom,
    ServerRoom,

    // Education
    Classroom,
    Laboratory,
    Library,
    ReadingArea,

    // Healthcare
    Ward,
    ExaminationRoom,
    TreatmentRoom,
    NursingStation,
    WaitingArea,
    Pharmacy,

    // Generic
    Corridor,

    // Industrial
    ProductionArea,
    LoadingArea,
    Workshop,

    // Public / Special
    ExhibitionArea,
    CommunityRoom,
    PrayerRoom,
    PlatformArea,
}

impl RoomType {
    pub fn is_service_room(self) -> bool {
        matches!(
            self,
            Self::Kitchen
                | Self::KitchenService
                | Self::Bathroom
                | Self::Toilet
                | Self::Storage
                | Self::UtilityRoom
                | Self::Laundry
                | Self::StaffRoom
                | Self::ServiceArea
                | Self::ServerRoom
        )
    }

    pub fn is_public_room(self) -> bool {
        matches!(
            self,
            Self::LivingRoom
                | Self::DiningRoom
                | Self::DiningArea
                | Self::ProductArea
                | Self::DisplayArea
                | Self::Checkout
                | Self::Classroom
                | Self::Library
                | Self::ReadingArea
                | Self::MeetingRoom
                | Self::Reception
                | Self::WaitingArea
                | Self::ExhibitionArea
                | Self::CommunityRoom
                | Self::PrayerRoom
                | Self::PlatformArea
        )
    }

    pub fn is_private_room(self) -> bool {
        matches!(
            self,
            Self::Bedroom
                | Self::Office
                | Self::Ward
                | Self::ExaminationRoom
                | Self::TreatmentRoom
                | Self::Laboratory
        )
    }
}
