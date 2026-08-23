use crate::element_processing::subprocessor::interior::BuildingType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAccessKind {
    None,
    Ladder,
    Stair,
    MultiStair,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StairSize {
    Compact,
    Small,
    Medium,
    Large,
    Grand,
}

#[derive(Debug, Clone, Copy)]
pub struct VerticalAccessDecision {
    pub kind: VerticalAccessKind,
    pub size: StairSize,
    pub width: i32,
    pub floors: usize,
}

pub fn decide_vertical_access(
    building_type: BuildingType,
    width: i32,
    depth: i32,
    floors: usize,
    room_count: usize,
) -> VerticalAccessDecision {
    let area = width * depth;

    if floors <= 1 {
        return VerticalAccessDecision {
            kind: VerticalAccessKind::None,
            size: StairSize::Compact,
            width: 1,
            floors,
        };
    }

    let small_building = area < 100;
    let medium_building = area < 400;

    if small_building && floors <= 2 {
        return VerticalAccessDecision {
            kind: VerticalAccessKind::Ladder,
            size: StairSize::Compact,
            width: 1,
            floors,
        };
    }

    let kind = match building_type {
        BuildingType::Shed | BuildingType::Garage | BuildingType::Barn | BuildingType::Stable
            if small_building =>
        {
            VerticalAccessKind::Ladder
        }

        BuildingType::School
        | BuildingType::College
        | BuildingType::University
        | BuildingType::Hospital
        | BuildingType::Mall
        | BuildingType::Station
        | BuildingType::Terminal
            if area >= 600 && floors >= 2 =>
        {
            VerticalAccessKind::MultiStair
        }

        _ => VerticalAccessKind::Stair,
    };

    let size = if area >= 1200 || room_count >= 20 {
        StairSize::Grand
    } else if area >= 600 || room_count >= 12 {
        StairSize::Large
    } else if medium_building {
        StairSize::Medium
    } else {
        StairSize::Small
    };

    let stair_width = match size {
        StairSize::Compact => 1,
        StairSize::Small => 1,
        StairSize::Medium => 2,
        StairSize::Large => 2,
        StairSize::Grand => 3,
    };

    VerticalAccessDecision {
        kind,
        size,
        width: stair_width,
        floors,
    }
}
