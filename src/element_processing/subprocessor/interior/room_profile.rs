use super::building_type::BuildingType;
use super::room_type::RoomType;

#[derive(Debug, Clone)]
pub struct RoomRequirement {
    pub room_type: RoomType,
    pub min_count: usize,
    pub max_count: usize,
    pub min_area: i32,
    pub priority: u8,
}

impl RoomRequirement {
    pub const fn new(
        room_type: RoomType,
        min_count: usize,
        max_count: usize,
        min_area: i32,
        priority: u8,
    ) -> Self {
        Self {
            room_type,
            min_count,
            max_count,
            min_area,
            priority,
        }
    }
}

pub fn room_profile(building_type: BuildingType) -> Vec<RoomRequirement> {
    match building_type {
        // =========================================================
        // Residential
        // =========================================================
        BuildingType::House | BuildingType::Apartment | BuildingType::ResidentialGeneric => vec![
            RoomRequirement::new(RoomType::LivingRoom, 1, 1, 16, 100),
            RoomRequirement::new(RoomType::Kitchen, 1, 1, 9, 90),
            RoomRequirement::new(RoomType::Bedroom, 1, 4, 9, 80),
            RoomRequirement::new(RoomType::Bathroom, 1, 2, 4, 100),
        ],

        BuildingType::Dormitory => vec![
            RoomRequirement::new(RoomType::Bedroom, 2, 20, 9, 100),
            RoomRequirement::new(RoomType::Kitchen, 1, 3, 9, 70),
            RoomRequirement::new(RoomType::Bathroom, 1, 6, 4, 100),
            RoomRequirement::new(RoomType::Corridor, 1, 4, 8, 100),
        ],

        // =========================================================
        // Commercial / Food
        // =========================================================
        BuildingType::Restaurant => vec![
            RoomRequirement::new(RoomType::DiningArea, 1, 3, 30, 100),
            RoomRequirement::new(RoomType::Kitchen, 1, 2, 12, 100),
            RoomRequirement::new(RoomType::Toilet, 1, 3, 4, 90),
            RoomRequirement::new(RoomType::Storage, 1, 2, 6, 70),
        ],

        BuildingType::Cafe | BuildingType::FastFood => vec![
            RoomRequirement::new(RoomType::DiningArea, 1, 2, 16, 100),
            RoomRequirement::new(RoomType::Kitchen, 1, 1, 9, 100),
            RoomRequirement::new(RoomType::Toilet, 1, 2, 4, 90),
            RoomRequirement::new(RoomType::Storage, 1, 2, 4, 70),
        ],

        BuildingType::Shop => vec![
            RoomRequirement::new(RoomType::ProductArea, 1, 2, 30, 100),
            RoomRequirement::new(RoomType::Checkout, 1, 4, 4, 100),
            RoomRequirement::new(RoomType::Storage, 1, 3, 8, 80),
        ],

        BuildingType::Supermarket | BuildingType::Mall => vec![
            RoomRequirement::new(RoomType::ProductArea, 1, 8, 40, 100),
            RoomRequirement::new(RoomType::Checkout, 1, 12, 4, 100),
            RoomRequirement::new(RoomType::Storage, 1, 6, 12, 90),
            RoomRequirement::new(RoomType::Toilet, 1, 4, 6, 80),
            RoomRequirement::new(RoomType::Corridor, 1, 4, 10, 100),
        ],

        BuildingType::Hotel => vec![
            RoomRequirement::new(RoomType::Bedroom, 2, 40, 12, 100),
            RoomRequirement::new(RoomType::Bathroom, 2, 40, 4, 100),
            RoomRequirement::new(RoomType::DiningArea, 1, 3, 30, 90),
            RoomRequirement::new(RoomType::Kitchen, 1, 2, 12, 80),
            RoomRequirement::new(RoomType::Corridor, 1, 6, 10, 100),
        ],

        // =========================================================
        // Office / Corporate / Government
        // =========================================================
        BuildingType::Office | BuildingType::Corporate | BuildingType::Government => vec![
            RoomRequirement::new(RoomType::Office, 2, 40, 9, 100),
            RoomRequirement::new(RoomType::MeetingRoom, 1, 8, 12, 90),
            RoomRequirement::new(RoomType::BreakRoom, 1, 3, 6, 70),
            RoomRequirement::new(RoomType::Toilet, 1, 4, 6, 90),
            RoomRequirement::new(RoomType::Corridor, 1, 6, 10, 100),
        ],

        // =========================================================
        // Education
        // =========================================================
        BuildingType::School => vec![
            RoomRequirement::new(RoomType::Classroom, 2, 20, 20, 100),
            RoomRequirement::new(RoomType::Office, 1, 8, 9, 80),
            RoomRequirement::new(RoomType::Toilet, 1, 6, 6, 90),
            RoomRequirement::new(RoomType::MeetingRoom, 1, 3, 12, 70),
            RoomRequirement::new(RoomType::Corridor, 1, 6, 10, 100),
        ],

        BuildingType::Kindergarten => vec![
            RoomRequirement::new(RoomType::Classroom, 2, 8, 20, 100),
            RoomRequirement::new(RoomType::DiningArea, 1, 2, 20, 80),
            RoomRequirement::new(RoomType::Office, 1, 3, 9, 70),
            RoomRequirement::new(RoomType::Toilet, 1, 6, 6, 100),
            RoomRequirement::new(RoomType::Corridor, 1, 4, 8, 100),
        ],

        BuildingType::College | BuildingType::University => vec![
            RoomRequirement::new(RoomType::Classroom, 4, 30, 20, 100),
            RoomRequirement::new(RoomType::Office, 2, 20, 9, 80),
            RoomRequirement::new(RoomType::MeetingRoom, 1, 8, 12, 80),
            RoomRequirement::new(RoomType::Toilet, 1, 8, 6, 90),
            RoomRequirement::new(RoomType::Corridor, 1, 8, 10, 100),
        ],

        // =========================================================
        // Healthcare
        // =========================================================
        BuildingType::Hospital => vec![
            RoomRequirement::new(RoomType::Ward, 2, 30, 20, 100),
            RoomRequirement::new(RoomType::ExaminationRoom, 1, 12, 12, 100),
            RoomRequirement::new(RoomType::TreatmentRoom, 1, 8, 12, 100),
            RoomRequirement::new(RoomType::NursingStation, 1, 6, 9, 100),
            RoomRequirement::new(RoomType::WaitingArea, 1, 6, 16, 90),
            RoomRequirement::new(RoomType::Pharmacy, 0, 2, 9, 70),
            RoomRequirement::new(RoomType::Toilet, 1, 10, 4, 90),
            RoomRequirement::new(RoomType::Corridor, 1, 8, 10, 100),
        ],

        BuildingType::Clinic => vec![
            RoomRequirement::new(RoomType::ExaminationRoom, 1, 8, 12, 100),
            RoomRequirement::new(RoomType::TreatmentRoom, 0, 4, 12, 90),
            RoomRequirement::new(RoomType::NursingStation, 1, 3, 9, 90),
            RoomRequirement::new(RoomType::WaitingArea, 1, 3, 12, 100),
            RoomRequirement::new(RoomType::Office, 1, 5, 9, 70),
            RoomRequirement::new(RoomType::Toilet, 1, 3, 4, 90),
            RoomRequirement::new(RoomType::Corridor, 1, 4, 8, 100),
        ],

        BuildingType::Pharmacy => vec![
            RoomRequirement::new(RoomType::ProductArea, 1, 3, 20, 100),
            RoomRequirement::new(RoomType::Checkout, 1, 3, 4, 100),
            RoomRequirement::new(RoomType::Storage, 1, 3, 8, 90),
            RoomRequirement::new(RoomType::Office, 1, 2, 9, 60),
        ],

        BuildingType::NursingHome => vec![
            RoomRequirement::new(RoomType::Bedroom, 2, 20, 12, 100),
            RoomRequirement::new(RoomType::Bathroom, 2, 20, 4, 100),
            RoomRequirement::new(RoomType::DiningArea, 1, 3, 20, 90),
            RoomRequirement::new(RoomType::NursingStation, 1, 4, 9, 100),
            RoomRequirement::new(RoomType::WaitingArea, 0, 2, 12, 70),
            RoomRequirement::new(RoomType::Corridor, 1, 6, 10, 100),
        ],

        // =========================================================
        // Industrial
        // =========================================================
        BuildingType::Factory | BuildingType::IndustrialGeneric => vec![
            RoomRequirement::new(RoomType::ProductionArea, 1, 8, 40, 100),
            RoomRequirement::new(RoomType::Workshop, 0, 4, 20, 90),
            RoomRequirement::new(RoomType::LoadingArea, 0, 4, 20, 90),
            RoomRequirement::new(RoomType::Storage, 1, 8, 30, 100),
            RoomRequirement::new(RoomType::Office, 1, 6, 9, 60),
            RoomRequirement::new(RoomType::Toilet, 1, 4, 6, 80),
            RoomRequirement::new(RoomType::Corridor, 1, 4, 10, 70),
        ],

        BuildingType::Warehouse => vec![
            RoomRequirement::new(RoomType::Storage, 1, 20, 50, 100),
            RoomRequirement::new(RoomType::LoadingArea, 1, 6, 20, 100),
            RoomRequirement::new(RoomType::Office, 1, 4, 9, 60),
            RoomRequirement::new(RoomType::Toilet, 1, 3, 4, 70),
        ],

        BuildingType::Workshop => vec![
            RoomRequirement::new(RoomType::Workshop, 1, 6, 20, 100),
            RoomRequirement::new(RoomType::Storage, 1, 6, 20, 90),
            RoomRequirement::new(RoomType::LoadingArea, 0, 2, 16, 80),
            RoomRequirement::new(RoomType::Office, 1, 3, 9, 60),
            RoomRequirement::new(RoomType::Toilet, 1, 3, 4, 70),
        ],

        // =========================================================
        // Public / Cultural
        // =========================================================
        BuildingType::Library => vec![
            RoomRequirement::new(RoomType::Library, 1, 6, 30, 100),
            RoomRequirement::new(RoomType::ReadingArea, 0, 4, 16, 90),
            RoomRequirement::new(RoomType::MeetingRoom, 0, 3, 12, 70),
            RoomRequirement::new(RoomType::Office, 1, 4, 9, 70),
            RoomRequirement::new(RoomType::Toilet, 1, 4, 6, 90),
            RoomRequirement::new(RoomType::Corridor, 1, 6, 10, 100),
        ],

        BuildingType::Museum => vec![
            RoomRequirement::new(RoomType::ExhibitionArea, 1, 8, 40, 100),
            RoomRequirement::new(RoomType::Storage, 1, 4, 20, 80),
            RoomRequirement::new(RoomType::Office, 1, 4, 9, 70),
            RoomRequirement::new(RoomType::MeetingRoom, 0, 3, 12, 70),
            RoomRequirement::new(RoomType::Toilet, 1, 4, 6, 90),
            RoomRequirement::new(RoomType::Corridor, 1, 6, 10, 100),
        ],

        BuildingType::CommunityCenter => vec![
            RoomRequirement::new(RoomType::CommunityRoom, 1, 4, 30, 100),
            RoomRequirement::new(RoomType::MeetingRoom, 1, 4, 12, 90),
            RoomRequirement::new(RoomType::Office, 1, 4, 9, 70),
            RoomRequirement::new(RoomType::Storage, 1, 3, 8, 70),
            RoomRequirement::new(RoomType::Toilet, 1, 4, 6, 90),
            RoomRequirement::new(RoomType::Corridor, 1, 6, 10, 100),
        ],

        BuildingType::PublicBuilding => vec![
            RoomRequirement::new(RoomType::Reception, 1, 2, 12, 100),
            RoomRequirement::new(RoomType::Office, 1, 8, 9, 90),
            RoomRequirement::new(RoomType::MeetingRoom, 0, 4, 12, 80),
            RoomRequirement::new(RoomType::WaitingArea, 0, 3, 12, 80),
            RoomRequirement::new(RoomType::Toilet, 1, 4, 6, 90),
            RoomRequirement::new(RoomType::Corridor, 1, 6, 10, 100),
        ],

        // =========================================================
        // Religious
        // =========================================================
        BuildingType::Church
        | BuildingType::Temple
        | BuildingType::Mosque
        | BuildingType::Shrine => vec![
            RoomRequirement::new(RoomType::PrayerRoom, 1, 4, 40, 100),
            RoomRequirement::new(RoomType::EntranceHall, 1, 2, 12, 90),
            RoomRequirement::new(RoomType::Storage, 1, 3, 8, 70),
            RoomRequirement::new(RoomType::Office, 0, 3, 9, 60),
            RoomRequirement::new(RoomType::Toilet, 0, 4, 4, 80),
        ],

        // =========================================================
        // Transport
        // =========================================================
        BuildingType::Station => vec![
            RoomRequirement::new(RoomType::PlatformArea, 1, 8, 30, 100),
            RoomRequirement::new(RoomType::WaitingArea, 1, 6, 20, 100),
            RoomRequirement::new(RoomType::Checkout, 0, 8, 4, 80),
            RoomRequirement::new(RoomType::Office, 1, 8, 9, 80),
            RoomRequirement::new(RoomType::Toilet, 1, 8, 6, 100),
            RoomRequirement::new(RoomType::Corridor, 1, 8, 10, 100),
        ],

        BuildingType::Terminal => vec![
            RoomRequirement::new(RoomType::PlatformArea, 1, 12, 40, 100),
            RoomRequirement::new(RoomType::WaitingArea, 1, 10, 30, 100),
            RoomRequirement::new(RoomType::Checkout, 1, 12, 4, 90),
            RoomRequirement::new(RoomType::ProductArea, 0, 8, 20, 70),
            RoomRequirement::new(RoomType::Office, 1, 10, 9, 80),
            RoomRequirement::new(RoomType::Toilet, 1, 10, 6, 100),
            RoomRequirement::new(RoomType::Corridor, 1, 10, 10, 100),
        ],

        BuildingType::TransportBuilding => vec![
            RoomRequirement::new(RoomType::WaitingArea, 1, 6, 20, 100),
            RoomRequirement::new(RoomType::PlatformArea, 1, 6, 30, 100),
            RoomRequirement::new(RoomType::Office, 1, 6, 9, 80),
            RoomRequirement::new(RoomType::Storage, 1, 4, 12, 70),
            RoomRequirement::new(RoomType::Toilet, 1, 6, 6, 90),
            RoomRequirement::new(RoomType::Corridor, 1, 8, 10, 100),
        ],

        // =========================================================
        // Agricultural
        // =========================================================
        BuildingType::Barn => vec![
            RoomRequirement::new(RoomType::Storage, 1, 8, 30, 100),
            RoomRequirement::new(RoomType::UtilityRoom, 0, 2, 12, 70),
        ],

        BuildingType::Stable => vec![
            RoomRequirement::new(RoomType::Storage, 1, 6, 20, 90),
            RoomRequirement::new(RoomType::UtilityRoom, 1, 3, 12, 80),
            RoomRequirement::new(RoomType::Workshop, 0, 2, 20, 70),
        ],

        BuildingType::FarmBuilding => vec![
            RoomRequirement::new(RoomType::Storage, 1, 8, 30, 100),
            RoomRequirement::new(RoomType::Workshop, 0, 3, 20, 80),
            RoomRequirement::new(RoomType::UtilityRoom, 0, 3, 12, 70),
            RoomRequirement::new(RoomType::Office, 0, 2, 9, 50),
        ],

        // =========================================================
        // Special
        // =========================================================
        BuildingType::Garage => vec![
            RoomRequirement::new(RoomType::Storage, 1, 3, 20, 100),
            RoomRequirement::new(RoomType::Workshop, 0, 2, 16, 80),
        ],

        BuildingType::Shed => vec![
            RoomRequirement::new(RoomType::Storage, 1, 2, 8, 100),
        ],

        BuildingType::Greenhouse => vec![
            RoomRequirement::new(RoomType::UtilityRoom, 1, 2, 12, 80),
            RoomRequirement::new(RoomType::Storage, 0, 2, 12, 70),
        ],

        BuildingType::Tower | BuildingType::Historic => vec![
            RoomRequirement::new(RoomType::Office, 1, 5, 9, 70),
            RoomRequirement::new(RoomType::Storage, 1, 3, 8, 60),
            RoomRequirement::new(RoomType::Corridor, 1, 4, 6, 80),
        ],

        BuildingType::Unknown => Vec::new(),
    }
}
