use std::collections::{HashMap, HashSet, VecDeque};

use crate::element_processing::subprocessor::interior::Rect;

use super::furniture_clearance::{circulation_cell_blocked, FurnitureObstacle};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WalkCell {
    pub x: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Default)]
pub struct InteriorPath {
    pub cells: Vec<WalkCell>,
}

impl InteriorPath {
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }
}

/// Grid-based indoor path search.
///
/// This operates only inside an already-existing room rectangle.
/// It does not modify the room, building footprint, furniture,
/// windows, doors, or BBox.
pub fn find_interior_path(
    bounds: Rect,
    start: WalkCell,
    goal: WalkCell,
    obstacles: &[FurnitureObstacle],
) -> InteriorPath {
    if start == goal {
        return InteriorPath { cells: vec![start] };
    }

    let inside = |cell: WalkCell| {
        cell.x >= bounds.min_x
            && cell.x <= bounds.max_x
            && cell.z >= bounds.min_z
            && cell.z <= bounds.max_z
    };

    if !inside(start) || !inside(goal) {
        return InteriorPath::default();
    }

    let blocked = |cell: WalkCell| {
        circulation_cell_blocked(
            Rect {
                min_x: cell.x,
                min_z: cell.z,
                max_x: cell.x,
                max_z: cell.z,
            },
            obstacles,
        )
    };

    if blocked(start) || blocked(goal) {
        return InteriorPath::default();
    }

    let neighbours = |cell: WalkCell| -> [WalkCell; 4] {
        [
            WalkCell {
                x: cell.x + 1,
                z: cell.z,
            },
            WalkCell {
                x: cell.x - 1,
                z: cell.z,
            },
            WalkCell {
                x: cell.x,
                z: cell.z + 1,
            },
            WalkCell {
                x: cell.x,
                z: cell.z - 1,
            },
        ]
    };

    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    let mut previous: HashMap<WalkCell, WalkCell> = HashMap::new();

    queue.push_back(start);
    visited.insert(start);

    while let Some(current) = queue.pop_front() {
        for next in neighbours(current) {
            if !inside(next) || blocked(next) || visited.contains(&next) {
                continue;
            }

            visited.insert(next);
            previous.insert(next, current);

            if next == goal {
                let mut cells = vec![goal];
                let mut cursor = goal;

                while cursor != start {
                    let Some(parent) = previous.get(&cursor).copied() else {
                        return InteriorPath::default();
                    };

                    cursor = parent;
                    cells.push(cursor);
                }

                cells.reverse();

                return InteriorPath { cells };
            }

            queue.push_back(next);
        }
    }

    InteriorPath::default()
}
