#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildingType {
    // Residential
    House,
    Apartment,
    Dormitory,
    ResidentialGeneric,

    // Commercial
    Shop,
    Supermarket,
    Mall,
    DepartmentStore,
    ConvenienceStore,
    RetailStore,
    Restaurant,
    Cafe,
    FastFood,
    Bakery,
    Bar,
    Pub,
    FoodCourt,
    Hotel,
    Motel,
    Hostel,
    GuestHouse,
    ShoppingCenter,
    Market,

    // Office
    Office,
    Government,
    Corporate,
    Bank,
    Financial,
    Coworking,
    Police,
    FireStation,
    Courthouse,
    Embassy,

    // Education
    School,
    Kindergarten,
    College,
    University,
    HighSchool,
    ElementarySchool,
    TrainingCenter,
    ResearchInstitute,

    // Healthcare
    Hospital,
    Clinic,
    Pharmacy,
    NursingHome,
    DentalClinic,
    VeterinaryClinic,
    MedicalCenter,
    Laboratory,
    RehabilitationCenter,

    // Industrial
    Factory,
    Warehouse,
    Workshop,
    IndustrialGeneric,
    ManufacturingPlant,
    DistributionCenter,
    LogisticsCenter,
    Depot,
    PowerPlant,
    WaterPlant,

    // Public
    Library,
    Museum,
    CommunityCenter,
    PublicBuilding,
    TownHall,
    SportsCenter,
    Stadium,
    Theater,
    Cinema,
    ConventionCenter,
    CulturalCenter,

    // Religious
    Church,
    Temple,
    Mosque,
    Shrine,

    // Transport
    Station,
    Terminal,
    TransportBuilding,
    TrainStation,
    BusStation,
    Airport,
    AirportTerminal,
    SubwayStation,
    FerryTerminal,
    ParkingStructure,

    // Agricultural
    Barn,
    Stable,
    FarmBuilding,
    AgriculturalWarehouse,
    GreenhouseFacility,
    LivestockBuilding,
    DairyBuilding,

    // Special
    Garage,
    ParkingGarage,
    Shed,
    Greenhouse,
    Tower,
    Skyscraper,
    Landmark,
    Historic,
    Castle,
    Manor,
    Villa,

    Unknown,
}

impl BuildingType {
    pub fn from_tags(
        building: Option<&str>,
        building_use: Option<&str>,
        amenity: Option<&str>,
        shop: Option<&str>,
        office: Option<&str>,
        healthcare: Option<&str>,
    ) -> Self {
        let b = building.unwrap_or("").to_ascii_lowercase();
        let u = building_use.unwrap_or("").to_ascii_lowercase();
        let a = amenity.unwrap_or("").to_ascii_lowercase();
        let s = shop.unwrap_or("").to_ascii_lowercase();
        let o = office.unwrap_or("").to_ascii_lowercase();
        let h = healthcare.unwrap_or("").to_ascii_lowercase();

        let has = |values: &[&str]| {
            values
                .iter()
                .any(|v| b == *v || u == *v || a == *v || s == *v || o == *v || h == *v)
        };

        // Education
        if has(&["university"]) {
            return Self::University;
        }
        if has(&["college"]) {
            return Self::College;
        }
        if has(&["kindergarten", "nursery"]) {
            return Self::Kindergarten;
        }
        if has(&["high_school"]) {
            return Self::HighSchool;
        }
        if has(&["elementary_school"]) {
            return Self::ElementarySchool;
        }
        if has(&["school"]) {
            return Self::School;
        }
        if has(&["training", "training_center"]) {
            return Self::TrainingCenter;
        }
        if has(&["research_institute"]) {
            return Self::ResearchInstitute;
        }

        // Healthcare
        if has(&["hospital"]) {
            return Self::Hospital;
        }
        if has(&["clinic", "doctors", "medical"]) {
            return Self::Clinic;
        }
        if has(&["pharmacy", "chemist"]) {
            return Self::Pharmacy;
        }
        if has(&["nursing_home", "care_home"]) {
            return Self::NursingHome;
        }
        if has(&["dentist", "dental"]) {
            return Self::DentalClinic;
        }
        if has(&["veterinary", "veterinary_clinic"]) {
            return Self::VeterinaryClinic;
        }
        if has(&["medical_centre", "medical_center"]) {
            return Self::MedicalCenter;
        }
        if has(&["laboratory", "lab"]) {
            return Self::Laboratory;
        }
        if has(&["rehabilitation"]) {
            return Self::RehabilitationCenter;
        }

        // Food / hospitality
        if has(&["fast_food"]) {
            return Self::FastFood;
        }
        if has(&["restaurant"]) {
            return Self::Restaurant;
        }
        if has(&["cafe", "coffee_shop"]) {
            return Self::Cafe;
        }
        if has(&["bakery"]) {
            return Self::Bakery;
        }
        if has(&["bar"]) {
            return Self::Bar;
        }
        if has(&["pub"]) {
            return Self::Pub;
        }
        if has(&["food_court"]) {
            return Self::FoodCourt;
        }

        if has(&["hotel"]) {
            return Self::Hotel;
        }
        if has(&["motel"]) {
            return Self::Motel;
        }
        if has(&["hostel"]) {
            return Self::Hostel;
        }
        if has(&["guest_house"]) {
            return Self::GuestHouse;
        }

        // Commercial
        if has(&["supermarket", "hypermarket"]) {
            return Self::Supermarket;
        }
        if has(&["department_store"]) {
            return Self::DepartmentStore;
        }
        if has(&["convenience"]) {
            return Self::ConvenienceStore;
        }
        if has(&["retail"]) {
            return Self::RetailStore;
        }
        if has(&["shopping_centre", "shopping_center", "mall"]) {
            return Self::Mall;
        }
        if has(&["market"]) {
            return Self::Market;
        }
        if has(&["shop"]) {
            return Self::Shop;
        }

        // Office / institutional
        if has(&["government"]) {
            return Self::Government;
        }
        if has(&["bank"]) {
            return Self::Bank;
        }
        if has(&["financial"]) {
            return Self::Financial;
        }
        if has(&["coworking"]) {
            return Self::Coworking;
        }
        if has(&["police"]) {
            return Self::Police;
        }
        if has(&["fire_station"]) {
            return Self::FireStation;
        }
        if has(&["courthouse"]) {
            return Self::Courthouse;
        }
        if has(&["embassy"]) {
            return Self::Embassy;
        }
        if has(&["corporate", "company"]) {
            return Self::Corporate;
        }
        if has(&["office", "commercial"]) {
            return Self::Office;
        }

        // Residential
        if has(&["apartments", "apartment"]) {
            return Self::Apartment;
        }
        if has(&["dormitory", "student_accommodation"]) {
            return Self::Dormitory;
        }
        if has(&[
            "house",
            "detached",
            "semidetached_house",
            "semi-detached",
            "terrace",
            "residential",
            "bungalow",
        ]) {
            return Self::House;
        }

        // Industrial
        if has(&["manufacturing", "manufacturing_plant"]) {
            return Self::ManufacturingPlant;
        }
        if has(&["distribution", "distribution_center", "distribution_centre"]) {
            return Self::DistributionCenter;
        }
        if has(&["logistics", "logistics_center", "logistics_centre"]) {
            return Self::LogisticsCenter;
        }
        if has(&["depot"]) {
            return Self::Depot;
        }
        if has(&["power_plant"]) {
            return Self::PowerPlant;
        }
        if has(&["water_plant", "waterworks"]) {
            return Self::WaterPlant;
        }
        if has(&["factory", "industrial"]) {
            return Self::Factory;
        }
        if has(&["warehouse"]) {
            return Self::Warehouse;
        }
        if has(&["workshop"]) {
            return Self::Workshop;
        }

        // Public / cultural
        if has(&["library"]) {
            return Self::Library;
        }
        if has(&["museum"]) {
            return Self::Museum;
        }
        if has(&["community_centre", "community_center"]) {
            return Self::CommunityCenter;
        }
        if has(&["townhall", "town_hall"]) {
            return Self::TownHall;
        }
        if has(&["sports_centre", "sports_center"]) {
            return Self::SportsCenter;
        }
        if has(&["stadium"]) {
            return Self::Stadium;
        }
        if has(&["theatre", "theater"]) {
            return Self::Theater;
        }
        if has(&["cinema"]) {
            return Self::Cinema;
        }
        if has(&["convention_centre", "convention_center"]) {
            return Self::ConventionCenter;
        }
        if has(&["cultural_centre", "cultural_center"]) {
            return Self::CulturalCenter;
        }
        if has(&["public"]) {
            return Self::PublicBuilding;
        }

        // Religious
        if has(&["church"]) {
            return Self::Church;
        }
        if has(&["mosque"]) {
            return Self::Mosque;
        }
        if has(&["temple"]) {
            return Self::Temple;
        }
        if has(&["shrine"]) {
            return Self::Shrine;
        }

        // Transport
        if has(&["airport_terminal"]) {
            return Self::AirportTerminal;
        }
        if has(&["airport"]) {
            return Self::Airport;
        }
        if has(&["train_station"]) {
            return Self::TrainStation;
        }
        if has(&["bus_station"]) {
            return Self::BusStation;
        }
        if has(&["subway_station"]) {
            return Self::SubwayStation;
        }
        if has(&["ferry_terminal"]) {
            return Self::FerryTerminal;
        }
        if has(&["parking_structure"]) {
            return Self::ParkingStructure;
        }
        if has(&["terminal"]) {
            return Self::Terminal;
        }
        if has(&["station", "railway_station"]) {
            return Self::Station;
        }
        if has(&["transport"]) {
            return Self::TransportBuilding;
        }

        // Agricultural
        if has(&["agricultural_warehouse"]) {
            return Self::AgriculturalWarehouse;
        }
        if has(&["livestock"]) {
            return Self::LivestockBuilding;
        }
        if has(&["dairy"]) {
            return Self::DairyBuilding;
        }
        if has(&["greenhouse_facility"]) {
            return Self::GreenhouseFacility;
        }
        if has(&["barn"]) {
            return Self::Barn;
        }
        if has(&["stable"]) {
            return Self::Stable;
        }
        if has(&["farm"]) {
            return Self::FarmBuilding;
        }

        // Special
        if has(&["parking_garage"]) {
            return Self::ParkingGarage;
        }
        if has(&["garage"]) {
            return Self::Garage;
        }
        if has(&["shed"]) {
            return Self::Shed;
        }
        if has(&["greenhouse"]) {
            return Self::Greenhouse;
        }
        if has(&["skyscraper"]) {
            return Self::Skyscraper;
        }
        if has(&["tower"]) {
            return Self::Tower;
        }
        if has(&["landmark"]) {
            return Self::Landmark;
        }
        if has(&["historic"]) {
            return Self::Historic;
        }
        if has(&["castle"]) {
            return Self::Castle;
        }
        if has(&["manor"]) {
            return Self::Manor;
        }
        if has(&["villa"]) {
            return Self::Villa;
        }

        Self::Unknown
    }

    pub fn is_residential(self) -> bool {
        matches!(
            self,
            Self::House | Self::Apartment | Self::Dormitory | Self::ResidentialGeneric
        )
    }

    pub fn is_commercial(self) -> bool {
        matches!(
            self,
            Self::Shop
                | Self::Supermarket
                | Self::Mall
                | Self::Restaurant
                | Self::Cafe
                | Self::FastFood
                | Self::Hotel
        )
    }

    pub fn is_office(self) -> bool {
        matches!(self, Self::Office | Self::Government | Self::Corporate)
    }

    pub fn is_public(self) -> bool {
        matches!(
            self,
            Self::School
                | Self::Kindergarten
                | Self::College
                | Self::University
                | Self::Hospital
                | Self::Clinic
                | Self::Library
                | Self::Museum
                | Self::CommunityCenter
                | Self::PublicBuilding
        )
    }
}
