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

        BuildingType::Shed => vec![RoomRequirement::new(RoomType::Storage, 1, 2, 8, 100)],

        BuildingType::Greenhouse => vec![
            RoomRequirement::new(RoomType::UtilityRoom, 1, 2, 12, 80),
            RoomRequirement::new(RoomType::Storage, 0, 2, 12, 70),
        ],

        BuildingType::Tower | BuildingType::Historic => vec![
            RoomRequirement::new(RoomType::Office, 1, 5, 9, 70),
            RoomRequirement::new(RoomType::Storage, 1, 3, 8, 60),
            RoomRequirement::new(RoomType::Corridor, 1, 4, 6, 80),
        ],

        // =========================================================
        // Extended Commercial
        // =========================================================
        BuildingType::DepartmentStore => vec![
            RoomRequirement::new(RoomType::ProductArea, 2, 12, 30, 100),
            RoomRequirement::new(RoomType::DisplayArea, 2, 12, 25, 90),
            RoomRequirement::new(RoomType::Checkout, 1, 12, 4, 100),
            RoomRequirement::new(RoomType::Storage, 1, 6, 12, 90),
            RoomRequirement::new(RoomType::StaffRoom, 1, 4, 8, 70),
            RoomRequirement::new(RoomType::ServiceArea, 0, 4, 8, 70),
            RoomRequirement::new(RoomType::Toilet, 1, 6, 6, 90),
        ],

        BuildingType::ConvenienceStore => vec![
            RoomRequirement::new(RoomType::ProductArea, 1, 3, 20, 100),
            RoomRequirement::new(RoomType::Checkout, 1, 2, 4, 100),
            RoomRequirement::new(RoomType::Storage, 1, 2, 8, 90),
            RoomRequirement::new(RoomType::StaffRoom, 0, 1, 6, 60),
        ],

        BuildingType::RetailStore => vec![
            RoomRequirement::new(RoomType::ProductArea, 1, 4, 20, 100),
            RoomRequirement::new(RoomType::DisplayArea, 1, 4, 16, 90),
            RoomRequirement::new(RoomType::Checkout, 1, 3, 4, 100),
            RoomRequirement::new(RoomType::Storage, 1, 3, 8, 80),
        ],

        BuildingType::ShoppingCenter | BuildingType::Market => vec![
            RoomRequirement::new(RoomType::ProductArea, 2, 12, 30, 100),
            RoomRequirement::new(RoomType::DisplayArea, 1, 10, 20, 90),
            RoomRequirement::new(RoomType::Checkout, 1, 12, 4, 100),
            RoomRequirement::new(RoomType::Storage, 1, 8, 12, 90),
            RoomRequirement::new(RoomType::Toilet, 1, 6, 6, 90),
            RoomRequirement::new(RoomType::Corridor, 1, 8, 10, 100),
        ],

        BuildingType::Bakery => vec![
            RoomRequirement::new(RoomType::ProductArea, 1, 2, 12, 100),
            RoomRequirement::new(RoomType::Counter, 1, 2, 6, 100),
            RoomRequirement::new(RoomType::KitchenService, 1, 2, 12, 100),
            RoomRequirement::new(RoomType::Storage, 1, 2, 6, 80),
            RoomRequirement::new(RoomType::ServiceArea, 0, 2, 6, 70),
        ],

        BuildingType::Bar | BuildingType::Pub => vec![
            RoomRequirement::new(RoomType::DiningArea, 1, 3, 20, 100),
            RoomRequirement::new(RoomType::Counter, 1, 2, 6, 100),
            RoomRequirement::new(RoomType::KitchenService, 1, 2, 10, 90),
            RoomRequirement::new(RoomType::Toilet, 1, 3, 4, 90),
            RoomRequirement::new(RoomType::Storage, 1, 2, 6, 70),
        ],

        BuildingType::FoodCourt => vec![
            RoomRequirement::new(RoomType::DiningArea, 1, 6, 30, 100),
            RoomRequirement::new(RoomType::Counter, 1, 8, 6, 100),
            RoomRequirement::new(RoomType::KitchenService, 1, 8, 10, 90),
            RoomRequirement::new(RoomType::Storage, 1, 4, 8, 70),
            RoomRequirement::new(RoomType::Toilet, 1, 4, 6, 90),
        ],

        BuildingType::Motel | BuildingType::Hostel | BuildingType::GuestHouse => vec![
            RoomRequirement::new(RoomType::Bedroom, 2, 30, 10, 100),
            RoomRequirement::new(RoomType::Bathroom, 1, 30, 4, 100),
            RoomRequirement::new(RoomType::Reception, 1, 2, 10, 90),
            RoomRequirement::new(RoomType::DiningArea, 0, 2, 20, 70),
            RoomRequirement::new(RoomType::Storage, 1, 4, 8, 70),
            RoomRequirement::new(RoomType::Corridor, 1, 6, 8, 100),
        ],

        // =========================================================
        // Extended Office / Civic
        // =========================================================
        BuildingType::Bank | BuildingType::Financial => vec![
            RoomRequirement::new(RoomType::Reception, 1, 2, 12, 100),
            RoomRequirement::new(RoomType::Office, 2, 20, 9, 100),
            RoomRequirement::new(RoomType::ServiceArea, 1, 4, 10, 90),
            RoomRequirement::new(RoomType::WaitingArea, 1, 3, 12, 90),
            RoomRequirement::new(RoomType::MeetingRoom, 0, 4, 12, 80),
            RoomRequirement::new(RoomType::Storage, 1, 3, 8, 70),
            RoomRequirement::new(RoomType::Toilet, 1, 4, 6, 90),
        ],

        BuildingType::Coworking => vec![
            RoomRequirement::new(RoomType::Office, 2, 20, 9, 100),
            RoomRequirement::new(RoomType::MeetingRoom, 1, 8, 12, 100),
            RoomRequirement::new(RoomType::BreakRoom, 1, 3, 8, 80),
            RoomRequirement::new(RoomType::Reception, 1, 2, 10, 90),
            RoomRequirement::new(RoomType::Toilet, 1, 4, 6, 90),
            RoomRequirement::new(RoomType::Corridor, 1, 5, 8, 100),
        ],

        BuildingType::Police | BuildingType::FireStation => vec![
            RoomRequirement::new(RoomType::Reception, 1, 2, 10, 100),
            RoomRequirement::new(RoomType::Office, 2, 12, 9, 90),
            RoomRequirement::new(RoomType::StaffRoom, 1, 4, 8, 90),
            RoomRequirement::new(RoomType::BreakRoom, 1, 2, 8, 80),
            RoomRequirement::new(RoomType::Storage, 1, 4, 10, 80),
            RoomRequirement::new(RoomType::Toilet, 1, 4, 6, 90),
            RoomRequirement::new(RoomType::Corridor, 1, 5, 8, 100),
        ],

        BuildingType::Courthouse | BuildingType::Embassy => vec![
            RoomRequirement::new(RoomType::Reception, 1, 2, 12, 100),
            RoomRequirement::new(RoomType::Office, 2, 20, 9, 100),
            RoomRequirement::new(RoomType::MeetingRoom, 1, 6, 14, 90),
            RoomRequirement::new(RoomType::WaitingArea, 1, 4, 12, 90),
            RoomRequirement::new(RoomType::Storage, 1, 3, 8, 70),
            RoomRequirement::new(RoomType::Toilet, 1, 4, 6, 90),
            RoomRequirement::new(RoomType::Corridor, 1, 6, 10, 100),
        ],

        // =========================================================
        // Extended Education
        // =========================================================
        BuildingType::HighSchool | BuildingType::ElementarySchool => vec![
            RoomRequirement::new(RoomType::Classroom, 4, 30, 20, 100),
            RoomRequirement::new(RoomType::Office, 1, 8, 9, 80),
            RoomRequirement::new(RoomType::Library, 0, 2, 25, 70),
            RoomRequirement::new(RoomType::DiningArea, 0, 2, 20, 70),
            RoomRequirement::new(RoomType::Laboratory, 0, 6, 20, 80),
            RoomRequirement::new(RoomType::Toilet, 1, 8, 6, 100),
            RoomRequirement::new(RoomType::Corridor, 1, 8, 10, 100),
        ],

        BuildingType::TrainingCenter => vec![
            RoomRequirement::new(RoomType::Classroom, 2, 12, 20, 100),
            RoomRequirement::new(RoomType::MeetingRoom, 1, 6, 12, 90),
            RoomRequirement::new(RoomType::Office, 1, 5, 9, 70),
            RoomRequirement::new(RoomType::WaitingArea, 1, 2, 12, 70),
            RoomRequirement::new(RoomType::Toilet, 1, 4, 6, 90),
            RoomRequirement::new(RoomType::Corridor, 1, 5, 10, 100),
        ],

        BuildingType::ResearchInstitute => vec![
            RoomRequirement::new(RoomType::Laboratory, 2, 20, 20, 100),
            RoomRequirement::new(RoomType::Office, 2, 20, 9, 90),
            RoomRequirement::new(RoomType::MeetingRoom, 1, 6, 12, 80),
            RoomRequirement::new(RoomType::Storage, 1, 6, 10, 80),
            RoomRequirement::new(RoomType::Toilet, 1, 6, 6, 90),
            RoomRequirement::new(RoomType::Corridor, 1, 8, 10, 100),
        ],

        // =========================================================
        // Extended Healthcare
        // =========================================================
        BuildingType::DentalClinic | BuildingType::MedicalCenter => vec![
            RoomRequirement::new(RoomType::ExaminationRoom, 1, 10, 12, 100),
            RoomRequirement::new(RoomType::TreatmentRoom, 1, 6, 12, 100),
            RoomRequirement::new(RoomType::WaitingArea, 1, 4, 12, 100),
            RoomRequirement::new(RoomType::Office, 1, 6, 9, 80),
            RoomRequirement::new(RoomType::Storage, 1, 3, 8, 70),
            RoomRequirement::new(RoomType::Toilet, 1, 4, 4, 90),
            RoomRequirement::new(RoomType::Corridor, 1, 5, 8, 100),
        ],

        BuildingType::VeterinaryClinic | BuildingType::RehabilitationCenter => vec![
            RoomRequirement::new(RoomType::ExaminationRoom, 1, 6, 12, 100),
            RoomRequirement::new(RoomType::TreatmentRoom, 1, 6, 12, 100),
            RoomRequirement::new(RoomType::WaitingArea, 1, 3, 12, 90),
            RoomRequirement::new(RoomType::Storage, 1, 3, 8, 80),
            RoomRequirement::new(RoomType::Office, 1, 4, 9, 70),
            RoomRequirement::new(RoomType::Toilet, 1, 3, 4, 90),
        ],

        BuildingType::Laboratory => vec![
            RoomRequirement::new(RoomType::Laboratory, 1, 12, 20, 100),
            RoomRequirement::new(RoomType::Office, 1, 6, 9, 80),
            RoomRequirement::new(RoomType::Storage, 1, 6, 10, 80),
            RoomRequirement::new(RoomType::WaitingArea, 0, 2, 12, 60),
            RoomRequirement::new(RoomType::Toilet, 1, 4, 6, 90),
        ],

        // =========================================================
        // Extended Industrial
        // =========================================================
        BuildingType::ManufacturingPlant => vec![
            RoomRequirement::new(RoomType::ProductionArea, 2, 12, 50, 100),
            RoomRequirement::new(RoomType::Storage, 2, 12, 30, 100),
            RoomRequirement::new(RoomType::LoadingArea, 1, 8, 20, 100),
            RoomRequirement::new(RoomType::Workshop, 0, 4, 20, 80),
            RoomRequirement::new(RoomType::Office, 1, 8, 9, 70),
            RoomRequirement::new(RoomType::Toilet, 1, 6, 6, 90),
        ],

        BuildingType::DistributionCenter | BuildingType::LogisticsCenter => vec![
            RoomRequirement::new(RoomType::Storage, 2, 20, 50, 100),
            RoomRequirement::new(RoomType::LoadingArea, 2, 12, 20, 100),
            RoomRequirement::new(RoomType::Office, 1, 8, 9, 70),
            RoomRequirement::new(RoomType::StaffRoom, 1, 4, 8, 70),
            RoomRequirement::new(RoomType::Toilet, 1, 6, 6, 90),
        ],

        BuildingType::Depot => vec![
            RoomRequirement::new(RoomType::Storage, 1, 10, 30, 100),
            RoomRequirement::new(RoomType::Workshop, 1, 5, 20, 90),
            RoomRequirement::new(RoomType::Office, 1, 4, 9, 70),
            RoomRequirement::new(RoomType::LoadingArea, 1, 6, 20, 90),
        ],

        BuildingType::PowerPlant | BuildingType::WaterPlant => vec![
            RoomRequirement::new(RoomType::ProductionArea, 1, 8, 40, 100),
            RoomRequirement::new(RoomType::UtilityRoom, 1, 8, 15, 100),
            RoomRequirement::new(RoomType::Workshop, 0, 4, 20, 80),
            RoomRequirement::new(RoomType::Office, 1, 6, 9, 70),
            RoomRequirement::new(RoomType::Storage, 1, 4, 12, 70),
            RoomRequirement::new(RoomType::Toilet, 1, 4, 6, 90),
        ],

        // =========================================================
        // Extended Public / Cultural
        // =========================================================
        BuildingType::TownHall => vec![
            RoomRequirement::new(RoomType::Reception, 1, 2, 12, 100),
            RoomRequirement::new(RoomType::Office, 2, 20, 9, 100),
            RoomRequirement::new(RoomType::MeetingRoom, 1, 8, 16, 100),
            RoomRequirement::new(RoomType::WaitingArea, 1, 4, 12, 80),
            RoomRequirement::new(RoomType::Toilet, 1, 6, 6, 90),
            RoomRequirement::new(RoomType::Corridor, 1, 8, 10, 100),
        ],

        BuildingType::SportsCenter => vec![
            RoomRequirement::new(RoomType::CommunityRoom, 1, 4, 30, 100),
            RoomRequirement::new(RoomType::Reception, 1, 2, 10, 90),
            RoomRequirement::new(RoomType::StaffRoom, 1, 4, 8, 80),
            RoomRequirement::new(RoomType::Storage, 1, 6, 12, 90),
            RoomRequirement::new(RoomType::Toilet, 1, 6, 6, 100),
            RoomRequirement::new(RoomType::Corridor, 1, 6, 10, 100),
        ],

        BuildingType::Stadium => vec![
            RoomRequirement::new(RoomType::CommunityRoom, 1, 8, 50, 100),
            RoomRequirement::new(RoomType::Reception, 1, 4, 12, 90),
            RoomRequirement::new(RoomType::Storage, 1, 8, 20, 90),
            RoomRequirement::new(RoomType::Toilet, 2, 20, 8, 100),
            RoomRequirement::new(RoomType::Corridor, 1, 12, 10, 100),
        ],

        BuildingType::Theater | BuildingType::Cinema => vec![
            RoomRequirement::new(RoomType::CommunityRoom, 1, 6, 40, 100),
            RoomRequirement::new(RoomType::WaitingArea, 1, 4, 20, 90),
            RoomRequirement::new(RoomType::Reception, 1, 2, 10, 90),
            RoomRequirement::new(RoomType::Storage, 1, 4, 12, 70),
            RoomRequirement::new(RoomType::Toilet, 1, 6, 6, 90),
        ],

        BuildingType::ConventionCenter | BuildingType::CulturalCenter => vec![
            RoomRequirement::new(RoomType::CommunityRoom, 2, 12, 30, 100),
            RoomRequirement::new(RoomType::ExhibitionArea, 1, 8, 30, 90),
            RoomRequirement::new(RoomType::MeetingRoom, 1, 10, 16, 90),
            RoomRequirement::new(RoomType::Reception, 1, 3, 12, 90),
            RoomRequirement::new(RoomType::Storage, 1, 6, 12, 70),
            RoomRequirement::new(RoomType::Toilet, 1, 8, 6, 90),
            RoomRequirement::new(RoomType::Corridor, 1, 10, 10, 100),
        ],

        // =========================================================
        // Extended Transport
        // =========================================================
        BuildingType::TrainStation | BuildingType::BusStation | BuildingType::SubwayStation => {
            vec![
                RoomRequirement::new(RoomType::PlatformArea, 1, 10, 30, 100),
                RoomRequirement::new(RoomType::WaitingArea, 1, 6, 20, 100),
                RoomRequirement::new(RoomType::Reception, 0, 2, 10, 80),
                RoomRequirement::new(RoomType::Office, 1, 8, 9, 80),
                RoomRequirement::new(RoomType::Toilet, 1, 8, 6, 100),
                RoomRequirement::new(RoomType::Corridor, 1, 8, 10, 100),
            ]
        }

        BuildingType::Airport | BuildingType::AirportTerminal | BuildingType::FerryTerminal => {
            vec![
                RoomRequirement::new(RoomType::WaitingArea, 2, 20, 40, 100),
                RoomRequirement::new(RoomType::PlatformArea, 1, 12, 40, 100),
                RoomRequirement::new(RoomType::Checkout, 1, 20, 4, 90),
                RoomRequirement::new(RoomType::ProductArea, 0, 10, 20, 70),
                RoomRequirement::new(RoomType::Office, 1, 12, 9, 80),
                RoomRequirement::new(RoomType::Toilet, 1, 12, 6, 100),
                RoomRequirement::new(RoomType::Corridor, 1, 12, 10, 100),
            ]
        }

        BuildingType::ParkingStructure => vec![
            RoomRequirement::new(RoomType::Storage, 1, 4, 20, 70),
            RoomRequirement::new(RoomType::UtilityRoom, 1, 4, 12, 80),
            RoomRequirement::new(RoomType::Toilet, 0, 3, 4, 60),
        ],

        // =========================================================
        // Extended Agricultural
        // =========================================================
        BuildingType::AgriculturalWarehouse => vec![
            RoomRequirement::new(RoomType::Storage, 1, 12, 40, 100),
            RoomRequirement::new(RoomType::LoadingArea, 1, 6, 20, 90),
            RoomRequirement::new(RoomType::UtilityRoom, 0, 3, 12, 70),
        ],

        BuildingType::GreenhouseFacility => vec![
            RoomRequirement::new(RoomType::UtilityRoom, 1, 4, 15, 90),
            RoomRequirement::new(RoomType::Storage, 1, 4, 12, 80),
            RoomRequirement::new(RoomType::ServiceArea, 0, 3, 12, 70),
        ],

        BuildingType::LivestockBuilding => vec![
            RoomRequirement::new(RoomType::UtilityRoom, 1, 6, 20, 100),
            RoomRequirement::new(RoomType::Storage, 1, 6, 20, 90),
            RoomRequirement::new(RoomType::ServiceArea, 1, 4, 12, 80),
        ],

        BuildingType::DairyBuilding => vec![
            RoomRequirement::new(RoomType::ProductionArea, 1, 5, 30, 100),
            RoomRequirement::new(RoomType::UtilityRoom, 1, 4, 15, 90),
            RoomRequirement::new(RoomType::Storage, 1, 5, 20, 80),
            RoomRequirement::new(RoomType::ServiceArea, 1, 3, 12, 80),
        ],

        // =========================================================
        // Extended Special
        // =========================================================
        BuildingType::ParkingGarage => vec![
            RoomRequirement::new(RoomType::Storage, 1, 4, 20, 70),
            RoomRequirement::new(RoomType::UtilityRoom, 1, 4, 12, 80),
            RoomRequirement::new(RoomType::Toilet, 0, 3, 4, 60),
        ],

        BuildingType::Skyscraper => vec![
            RoomRequirement::new(RoomType::Office, 4, 80, 12, 100),
            RoomRequirement::new(RoomType::MeetingRoom, 1, 20, 12, 90),
            RoomRequirement::new(RoomType::BreakRoom, 1, 10, 8, 80),
            RoomRequirement::new(RoomType::Reception, 1, 4, 12, 90),
            RoomRequirement::new(RoomType::Toilet, 2, 20, 6, 100),
            RoomRequirement::new(RoomType::Corridor, 2, 30, 10, 100),
        ],

        BuildingType::Landmark | BuildingType::Historic => vec![
            RoomRequirement::new(RoomType::ExhibitionArea, 0, 6, 30, 80),
            RoomRequirement::new(RoomType::CommunityRoom, 0, 4, 20, 70),
            RoomRequirement::new(RoomType::Storage, 1, 4, 10, 70),
            RoomRequirement::new(RoomType::Corridor, 1, 6, 8, 80),
        ],

        BuildingType::Castle => vec![
            RoomRequirement::new(RoomType::LivingRoom, 1, 6, 20, 100),
            RoomRequirement::new(RoomType::Bedroom, 2, 20, 12, 90),
            RoomRequirement::new(RoomType::DiningRoom, 1, 4, 20, 90),
            RoomRequirement::new(RoomType::Kitchen, 1, 3, 12, 90),
            RoomRequirement::new(RoomType::Storage, 1, 8, 10, 80),
            RoomRequirement::new(RoomType::Corridor, 1, 10, 8, 100),
        ],

        BuildingType::Manor | BuildingType::Villa => vec![
            RoomRequirement::new(RoomType::LivingRoom, 1, 3, 20, 100),
            RoomRequirement::new(RoomType::Kitchen, 1, 2, 10, 90),
            RoomRequirement::new(RoomType::Bedroom, 2, 10, 12, 90),
            RoomRequirement::new(RoomType::Bathroom, 1, 6, 5, 100),
            RoomRequirement::new(RoomType::DiningRoom, 1, 2, 16, 80),
            RoomRequirement::new(RoomType::Storage, 1, 4, 8, 70),
        ],

        // =========================================================
        // Transport / Airport structures
        // =========================================================
        BuildingType::ParkingGarage | BuildingType::ParkingStructure => vec![
            RoomRequirement::new(RoomType::UtilityRoom, 1, 4, 12, 80),
            RoomRequirement::new(RoomType::Storage, 0, 4, 20, 60),
            RoomRequirement::new(RoomType::Toilet, 0, 3, 4, 60),
        ],

        BuildingType::Unknown => Vec::new(),
    }
}
