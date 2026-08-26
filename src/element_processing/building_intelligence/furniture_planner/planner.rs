use crate::element_processing::building_intelligence::room_graph::RoomGraph;
use crate::element_processing::building_intelligence::{
    furniture_profile, FurnitureItem, FurnitureKind,
};
use crate::element_processing::subprocessor::interior::decision::SpatialConstraints;
use crate::element_processing::subprocessor::interior::{FloorPlan, Room, RoomType};

/// Upgrade 07
///
/// FurniturePlanner is a READ-ONLY consumer of the reconstructed world.
///
/// It never:
/// - creates windows
/// - moves windows
/// - removes windows
/// - modifies exterior geometry
/// - modifies the BBox
///
/// It only creates semantic furniture intent for the Renderer.
pub struct FurniturePlanner;

impl FurniturePlanner {
    pub fn new() -> Self {
        Self
    }

    /// Plan furniture using:
    /// - room semantics
    /// - room geometry
    /// - existing windows
    /// - existing entrances
    /// - circulation constraints
    /// - furniture dimensions
    ///
    /// SpatialConstraints are read-only.
    pub fn plan_floor(
        &self,
        floor_plan: &FloorPlan,
        constraints: &SpatialConstraints,
        room_graph: &RoomGraph,
        floor: i32,
    ) -> Vec<FurnitureItem> {
        let mut furniture = Vec::new();

        for (floor_room_index, room) in floor_plan.rooms.iter().enumerate() {
            let Some(room_node) = room_graph.rooms.iter().find(|node| {
                node.floor == floor as usize && node.floor_room_index == floor_room_index
            }) else {
                continue;
            };

            // FurnitureItem.room_id is the ONLY ownership key.
            //
            // It must be the stable RoomGraph::RoomNode.id.
            // Renderer resolves:
            //
            //     room_id
            //        -> RoomGraph::RoomNode
            //        -> floor
            //        -> floor_room_index
            //        -> FloorPlan.rooms[floor_room_index]
            //
            // Never use RoomType or local room ordering as ownership.
            self.plan_room(room_node.id, room, constraints, floor, &mut furniture);
        }

        eprintln!(
            "[RWM-FURNITURE] floor={} rooms={} furniture={}",
            floor,
            floor_plan.rooms.len(),
            furniture.len()
        );

        furniture
    }

    fn plan_room(
        &self,
        room_id: usize,
        room: &Room,
        constraints: &SpatialConstraints,
        floor: i32,
        output: &mut Vec<FurnitureItem>,
    ) {
        let profile = furniture_profile(room.room_type);

        for &kind in profile {
            if let Some((x, z)) = find_position(room_id, room, kind, constraints, floor, output) {
                output.push(FurnitureItem {
                    room_id,
                    kind,
                    room_type: room.room_type,
                    relative_x: x,
                    relative_z: z,
                });
            }
        }
    }
}

fn furniture_size(kind: FurnitureKind) -> (i32, i32) {
    match kind {
        FurnitureKind::Bed => (2, 1),
        FurnitureKind::Sofa => (2, 1),
        FurnitureKind::Table => (1, 1),
        FurnitureKind::Chair => (1, 1),
        FurnitureKind::KitchenCounter => (2, 1),
        FurnitureKind::Sink => (1, 1),
        FurnitureKind::Toilet => (1, 1),
        FurnitureKind::Shower => (1, 1),
        FurnitureKind::Bathtub => (2, 1),
        FurnitureKind::Desk => (1, 1),
        FurnitureKind::Bookshelf => (1, 1),
        FurnitureKind::Checkout => (2, 1),
        FurnitureKind::Shelf => (1, 1),
        FurnitureKind::HospitalBed => (2, 1),
        FurnitureKind::MedicalDesk => (1, 1),
        FurnitureKind::ClassroomDesk => (1, 1),
        FurnitureKind::StorageShelf => (1, 1),
        FurnitureKind::DiningTable => (2, 2),
    }
}

/// Find a valid furniture location.
///
/// Priority:
/// 1. Keep furniture inside room.
/// 2. Keep circulation margin.
/// 3. Avoid existing windows.
/// 4. Avoid existing entrances.
/// 5. Avoid previously planned furniture.
/// 6. Prefer locations away from the main circulation axis.
fn find_position(
    room_id: usize,
    room: &Room,
    kind: FurnitureKind,
    constraints: &SpatialConstraints,
    floor: i32,
    existing: &[FurnitureItem],
) -> Option<(i32, i32)> {
    let (width, depth) = furniture_size(kind);

    let bounds = room.bounds;

    if bounds.width() < width || bounds.depth() < depth {
        return None;
    }

    // One-block circulation margin.
    let max_x = bounds.width() - width - 1;
    let max_z = bounds.depth() - depth - 1;

    if max_x < 1 || max_z < 1 {
        return None;
    }

    let mut candidates = Vec::new();

    for z in 1..=max_z {
        for x in 1..=max_x {
            if collides(x, z, width, depth, room_id, existing) {
                continue;
            }

            if touches_existing_window(room, x, z, width, depth, constraints, floor) {
                continue;
            }

            if touches_existing_entrance(room, x, z, width, depth, constraints, floor) {
                continue;
            }

            let score = candidate_score(room, kind, x, z, width, depth, constraints, floor);

            candidates.push((score, x, z));
        }
    }

    candidates
        .into_iter()
        .max_by_key(|(score, _, _)| *score)
        .map(|(_, x, z)| (x, z))
}

/// Score furniture placement.
///
/// Higher score means better placement.
fn candidate_score(
    room: &Room,
    kind: FurnitureKind,
    x: i32,
    z: i32,
    width: i32,
    depth: i32,
    constraints: &SpatialConstraints,
    floor: i32,
) -> i32 {
    let mut score = 0;

    let center_x = x + width / 2;
    let center_z = z + depth / 2;

    // ---------------------------------------------------------
    // Prefer furniture against room boundaries.
    // This leaves the center available for circulation.
    // ---------------------------------------------------------

    let near_left = x <= 1;
    let near_top = z <= 1;
    let near_right = x + width >= room.bounds.width() - 1;
    let near_bottom = z + depth >= room.bounds.depth() - 1;

    if near_left {
        score += 10;
    }

    if near_top {
        score += 10;
    }

    if near_right {
        score += 10;
    }

    if near_bottom {
        score += 10;
    }

    // ---------------------------------------------------------
    // Daylight-aware placement.
    //
    // Furniture that benefits from natural light gets a small
    // preference toward rooms with windows.
    // ---------------------------------------------------------

    let daylight_preferred = matches!(
        kind,
        FurnitureKind::Sofa
            | FurnitureKind::Table
            | FurnitureKind::DiningTable
            | FurnitureKind::Desk
            | FurnitureKind::ClassroomDesk
    );

    if daylight_preferred && constraints.window_count(room.bounds, floor) > 0 {
        let distance = distance_to_nearest_window(center_x, center_z, room, constraints, floor);

        // Prefer being reasonably close to daylight,
        // but never directly occupying the window zone.
        score += (20 - distance.min(20)) as i32;
    }

    // ---------------------------------------------------------
    // Room-specific semantic preferences.
    // ---------------------------------------------------------

    match room.room_type {
        RoomType::Bedroom => {
            if matches!(kind, FurnitureKind::Bed) {
                score += 35;
            }
        }

        RoomType::LivingRoom => {
            if matches!(kind, FurnitureKind::Sofa) {
                score += 35;
            }
        }

        RoomType::Kitchen => {
            if matches!(kind, FurnitureKind::KitchenCounter) {
                score += 40;
            }

            if matches!(kind, FurnitureKind::Sink) {
                score += 30;
            }
        }

        RoomType::Bathroom | RoomType::Toilet => {
            if matches!(
                kind,
                FurnitureKind::Toilet
                    | FurnitureKind::Shower
                    | FurnitureKind::Bathtub
                    | FurnitureKind::Sink
            ) {
                score += 30;
            }
        }

        RoomType::Office => {
            if matches!(kind, FurnitureKind::Desk) {
                score += 35;
            }
        }

        RoomType::Classroom => {
            if matches!(kind, FurnitureKind::ClassroomDesk) {
                score += 35;
            }
        }

        RoomType::DiningArea | RoomType::DiningRoom => {
            if matches!(kind, FurnitureKind::DiningTable) {
                score += 40;
            }
        }

        RoomType::Ward | RoomType::ExaminationRoom => {
            if matches!(kind, FurnitureKind::HospitalBed) {
                score += 35;
            }
        }

        RoomType::Storage => {
            if matches!(kind, FurnitureKind::StorageShelf) {
                score += 40;
            }
        }

        RoomType::ProductArea => {
            if matches!(kind, FurnitureKind::Shelf) {
                score += 40;
            }
        }

        RoomType::Checkout => {
            if matches!(kind, FurnitureKind::Checkout) {
                score += 40;
            }
        }

        _ => {}
    }

    // ---------------------------------------------------------
    // Prefer preserving the center of the room.
    // ---------------------------------------------------------

    let room_center = room.bounds.center();

    let dx = (center_x - room_center.0).abs();
    let dz = (center_z - room_center.1).abs();

    score += (dx + dz).min(10);

    score
}

/// Existing windows are INPUT ONLY.
///
/// Furniture must not occupy the window zone.
fn touches_existing_window(
    room: &Room,
    x: i32,
    z: i32,
    width: i32,
    depth: i32,
    constraints: &SpatialConstraints,
    floor: i32,
) -> bool {
    let world_x = room.bounds.min_x + x;
    let world_z = room.bounds.min_z + z;

    constraints.windows.iter().any(|window| {
        if window.floor != floor {
            return false;
        }

        rectangles_overlap(
            world_x,
            world_z,
            width,
            depth,
            window.x - 1,
            window.z - 1,
            window.width.max(1) + 2,
            2,
        )
    })
}

/// Existing entrances are INPUT ONLY.
///
/// Furniture must preserve an approach zone.
fn touches_existing_entrance(
    room: &Room,
    x: i32,
    z: i32,
    width: i32,
    depth: i32,
    constraints: &SpatialConstraints,
    floor: i32,
) -> bool {
    let world_x = room.bounds.min_x + x;
    let world_z = room.bounds.min_z + z;

    constraints.entrances.iter().any(|entrance| {
        if entrance.floor != floor {
            return false;
        }

        rectangles_overlap(
            world_x,
            world_z,
            width,
            depth,
            entrance.x - 2,
            entrance.z - 2,
            5,
            5,
        )
    })
}

fn distance_to_nearest_window(
    x: i32,
    z: i32,
    room: &Room,
    constraints: &SpatialConstraints,
    floor: i32,
) -> i32 {
    constraints
        .windows
        .iter()
        .filter(|window| window.floor == floor)
        .map(|window| {
            let wx = window.x - room.bounds.min_x;
            let wz = window.z - room.bounds.min_z;

            (x - wx).abs() + (z - wz).abs()
        })
        .min()
        .unwrap_or(20)
}

fn collides(
    x: i32,
    z: i32,
    width: i32,
    depth: i32,
    room_id: usize,
    existing: &[FurnitureItem],
) -> bool {
    existing.iter().any(|item| {
        if item.room_id != room_id {
            return false;
        }

        let (ew, ed) = furniture_size(item.kind);

        rectangles_overlap(x, z, width, depth, item.relative_x, item.relative_z, ew, ed)
    })
}

fn rectangles_overlap(
    ax: i32,
    az: i32,
    aw: i32,
    ad: i32,
    bx: i32,
    bz: i32,
    bw: i32,
    bd: i32,
) -> bool {
    ax < bx + bw && ax + aw > bx && az < bz + bd && az + ad > bz
}
