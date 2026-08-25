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
    Pantry,
    WalkInCloset,
    DressingRoom,
    StudyRoom,
    GuestRoom,
    Nursery,
    MasterBedroom,
    EnsuiteBathroom,

    // Restaurant / Food
    DiningArea,
    KitchenService,
    Counter,
    StaffRoom,
    ServiceArea,
    BarArea,
    KitchenPrep,
    StorageKitchen,
    DiningRoomPrivate,

    // Commercial
    ProductArea,
    DisplayArea,
    Checkout,
    SalesFloor,
    StockRoom,
    CashOffice,
    SecurityRoom,
    CustomerService,

    // Office
    Office,
    Reception,
    MeetingRoom,
    BreakRoom,
    ServerRoom,
    ArchiveRoom,
    OpenOffice,
    PrivateOffice,
    ExecutiveOffice,
    CopyRoom,
    ReceptionArea,

    // Education
    Classroom,
    Laboratory,
    Library,
    ReadingArea,
    LectureHall,
    ComputerLab,
    ScienceLab,
    StaffRoomEducation,
    TeachersRoom,
    AdministrationOffice,

    // Healthcare
    Ward,
    ExaminationRoom,
    TreatmentRoom,
    NursingStation,
    WaitingArea,
    Pharmacy,
    OperatingRoom,
    IntensiveCareUnit,
    EmergencyRoom,
    ConsultationRoom,
    IsolationRoom,
    SterilizationRoom,
    Morgue,
    StaffLounge,

    // Generic
    Corridor,

    // Industrial
    ProductionArea,
    LoadingArea,
    Workshop,
    AssemblyArea,
    MachineryRoom,
    MaintenanceRoom,
    StorageWarehouse,
    EquipmentRoom,
    ControlRoom,

    // Public / Special
    ExhibitionArea,
    CommunityRoom,
    PrayerRoom,
    PlatformArea,
    Stage,
    Auditorium,
    Gallery,
    TicketHall,
    LockerRoom,
    Gym,
    PoolArea,
    ChangingRoom,
    SecurityOffice,
    ControlCenter,
    MechanicalRoom,
    ElectricalRoom,
    BoilerRoom,
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
