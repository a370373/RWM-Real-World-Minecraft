#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntranceSide {
    North,
    South,
    West,
    East,
}

#[derive(Debug, Clone, Copy)]
pub struct EntranceCandidate {
    pub side: EntranceSide,
    pub x: i32,
    pub z: i32,
    pub has_road: bool,
    pub has_footway: bool,
    pub has_parking: bool,
    pub has_entrance_poi: bool,
    pub score: f32,
}

impl EntranceCandidate {
    pub fn new(side: EntranceSide, x: i32, z: i32) -> Self {
        Self {
            side,
            x,
            z,
            has_road: false,
            has_footway: false,
            has_parking: false,
            has_entrance_poi: false,
            score: 0.0,
        }
    }

    pub fn calculate_score(&mut self) {
        self.score = 0.0;

        // A reconstructed real-world road directly adjacent to
        // this candidate side is strong entrance evidence.
        if self.has_road {
            self.score += 25.0;
        }

        if self.has_footway {
            self.score += 30.0;
        }

        if self.has_parking {
            self.score += 20.0;
        }

        if self.has_entrance_poi {
            self.score += 100.0;
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BuildingEntrance {
    pub side: EntranceSide,
    pub x: i32,
    pub z: i32,
}

impl BuildingEntrance {
    pub fn from_candidate(candidate: EntranceCandidate) -> Self {
        Self {
            side: candidate.side,
            x: candidate.x,
            z: candidate.z,
        }
    }
}
