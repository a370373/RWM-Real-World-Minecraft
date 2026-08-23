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
    Restaurant,
    Cafe,
    FastFood,
    Hotel,

    // Office
    Office,
    Government,
    Corporate,

    // Education
    School,
    Kindergarten,
    College,
    University,

    // Healthcare
    Hospital,
    Clinic,
    Pharmacy,
    NursingHome,

    // Industrial
    Factory,
    Warehouse,
    Workshop,
    IndustrialGeneric,

    // Public
    Library,
    Museum,
    CommunityCenter,
    PublicBuilding,

    // Religious
    Church,
    Temple,
    Mosque,
    Shrine,

    // Transport
    Station,
    Terminal,
    TransportBuilding,

    // Agricultural
    Barn,
    Stable,
    FarmBuilding,

    // Special
    Garage,
    Shed,
    Greenhouse,
    Tower,
    Historic,

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
        let values = [building, building_use, amenity, shop, office, healthcare];

        let matches_any = |needles: &[&str]| {
            values.iter().flatten().any(|value| {
                let value = value.to_ascii_lowercase();
                needles.iter().any(|needle| value.contains(needle))
            })
        };

        // Education
        if matches_any(&["university"]) {
            return Self::University;
        }

        if matches_any(&["college"]) {
            return Self::College;
        }

        if matches_any(&["kindergarten", "nursery"]) {
            return Self::Kindergarten;
        }

        if matches_any(&["school"]) {
            return Self::School;
        }

        // Healthcare
        if matches_any(&["hospital"]) {
            return Self::Hospital;
        }

        if matches_any(&["clinic", "doctors", "medical"]) {
            return Self::Clinic;
        }

        if matches_any(&["pharmacy", "chemist"]) {
            return Self::Pharmacy;
        }

        if matches_any(&["nursing_home", "care_home"]) {
            return Self::NursingHome;
        }

        // Food / Restaurant
        if matches_any(&["fast_food"]) {
            return Self::FastFood;
        }

        if matches_any(&["restaurant"]) {
            return Self::Restaurant;
        }

        if matches_any(&["cafe", "coffee_shop"]) {
            return Self::Cafe;
        }

        // Commercial
        if matches_any(&["supermarket", "hypermarket"]) {
            return Self::Supermarket;
        }

        if matches_any(&["mall", "shopping_centre", "shopping_center"]) {
            return Self::Mall;
        }

        if matches_any(&["convenience", "department_store", "retail", "shop"]) {
            return Self::Shop;
        }

        // Office
        if matches_any(&["government"]) {
            return Self::Government;
        }

        if matches_any(&["office", "commercial"]) {
            return Self::Office;
        }

        if matches_any(&["company", "corporate"]) {
            return Self::Corporate;
        }

        // Residential
        if matches_any(&["apartments", "apartment"]) {
            return Self::Apartment;
        }

        if matches_any(&["dormitory", "student_accommodation"]) {
            return Self::Dormitory;
        }

        if matches_any(&[
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
        if matches_any(&["factory", "industrial"]) {
            return Self::Factory;
        }

        if matches_any(&["warehouse"]) {
            return Self::Warehouse;
        }

        if matches_any(&["workshop"]) {
            return Self::Workshop;
        }

        // Public
        if matches_any(&["library"]) {
            return Self::Library;
        }

        if matches_any(&["museum"]) {
            return Self::Museum;
        }

        if matches_any(&["community_centre", "community_center"]) {
            return Self::CommunityCenter;
        }

        // Religious
        if matches_any(&["church"]) {
            return Self::Church;
        }

        if matches_any(&["mosque"]) {
            return Self::Mosque;
        }

        if matches_any(&["temple"]) {
            return Self::Temple;
        }

        if matches_any(&["shrine"]) {
            return Self::Shrine;
        }

        // Transport
        if matches_any(&["station", "railway_station"]) {
            return Self::Station;
        }

        if matches_any(&["terminal"]) {
            return Self::Terminal;
        }

        if matches_any(&["transport"]) {
            return Self::TransportBuilding;
        }

        // Agricultural
        if matches_any(&["barn"]) {
            return Self::Barn;
        }

        if matches_any(&["stable"]) {
            return Self::Stable;
        }

        if matches_any(&["farm"]) {
            return Self::FarmBuilding;
        }

        // Special
        if matches_any(&["garage"]) {
            return Self::Garage;
        }

        if matches_any(&["shed"]) {
            return Self::Shed;
        }

        if matches_any(&["greenhouse"]) {
            return Self::Greenhouse;
        }

        if matches_any(&["tower"]) {
            return Self::Tower;
        }

        if matches_any(&["historic"]) {
            return Self::Historic;
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
