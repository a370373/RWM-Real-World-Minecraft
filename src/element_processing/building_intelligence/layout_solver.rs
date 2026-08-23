use crate::element_processing::subprocessor::interior::{Rect, RoomType};

#[derive(Debug, Clone, Copy)]
pub struct LayoutConstraint {
    pub room_type: RoomType,
    pub required_area: i32,
    pub min_width: i32,
    pub min_depth: i32,
    pub daylight_required: bool,
    pub priority: u8,
}

pub fn score_layout(room: Rect, constraint: LayoutConstraint, windows: &[(i32, i32)]) -> i32 {
    if !room.can_fit(constraint.min_width, constraint.min_depth) {
        return i32::MIN;
    }

    let area = room.area();

    let area_score = if area >= constraint.required_area {
        constraint.required_area * 2
            - (area - constraint.required_area).min(constraint.required_area)
    } else {
        area * 2
    };

    let mut score = area_score;

    if constraint.daylight_required {
        let window_count = windows
            .iter()
            .filter(|&&(x, z)| {
                room.contains(x, z)
                    || x == room.min_x
                    || x == room.max_x
                    || z == room.min_z
                    || z == room.max_z
            })
            .count();

        if window_count > 0 {
            score += 100 + (window_count as i32 * 20);
        } else {
            score -= 100;
        }
    }

    score += constraint.priority as i32 * 5;

    score
}
