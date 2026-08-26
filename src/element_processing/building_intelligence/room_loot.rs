use crate::block_definitions::{BlockWithProperties, CHEST};
use crate::world_editor::WorldEditor;
use fastnbt::Value;
use rand::Rng;
use std::collections::HashMap;

/// ------------------------------------------------------------
/// RWM Room Loot System
/// ------------------------------------------------------------
///
/// Rules:
/// - Room type is semantic information only.
/// - cached_floor_area is the physical authority.
/// - Never expands the building.
/// - Never modifies FloorPlan.
/// - Never modifies BBox.
/// - Never creates loot outside the already reconstructed room.
///
/// Known room type:
///     -> room-specific loot
///
/// Unknown / unrecognized room type:
///     -> generic random-life chest fallback
///
/// Empty / invalid physical room:
///     -> no container
/// ------------------------------------------------------------

const MAX_ITEMS_PER_CHEST: usize = 4;

#[derive(Clone, Copy)]
struct LootEntry {
    id: &'static str,
    min: i32,
    max: i32,
    weight: u32,
}

fn item(id: &'static str, min: i32, max: i32, weight: u32) -> LootEntry {
    LootEntry {
        id,
        min,
        max,
        weight,
    }
}

/// ------------------------------------------------------------
/// Generic fallback loot
/// ------------------------------------------------------------
///
/// Deliberately ordinary household / survival material.
/// This is NOT treasure loot.
///
/// Unknown room should still feel like a real occupied building,
/// rather than an empty shell.
/// ------------------------------------------------------------
fn generic_fallback_pool() -> Vec<LootEntry> {
    vec![
        item("minecraft:bread", 1, 4, 12),
        item("minecraft:apple", 1, 4, 10),
        item("minecraft:paper", 1, 8, 10),
        item("minecraft:book", 1, 3, 8),
        item("minecraft:stick", 2, 8, 8),
        item("minecraft:coal", 1, 6, 6),
        item("minecraft:charcoal", 1, 6, 6),
        item("minecraft:oak_planks", 1, 8, 8),
        item("minecraft:glass_bottle", 1, 3, 7),
        item("minecraft:feather", 1, 5, 6),
        item("minecraft:string", 1, 5, 6),
        item("minecraft:iron_nugget", 1, 4, 4),
        item("minecraft:leather", 1, 3, 5),
        item("minecraft:wheat", 1, 5, 7),
    ]
}

/// ------------------------------------------------------------
/// Room-specific pools
/// ------------------------------------------------------------

fn bedroom_pool() -> Vec<LootEntry> {
    vec![
        item("minecraft:book", 1, 3, 12),
        item("minecraft:paper", 1, 6, 10),
        item("minecraft:leather", 1, 3, 8),
        item("minecraft:string", 1, 4, 7),
        item("minecraft:apple", 1, 3, 8),
        item("minecraft:bread", 1, 3, 8),
        item("minecraft:iron_nugget", 1, 3, 3),
    ]
}

fn kitchen_pool() -> Vec<LootEntry> {
    vec![
        item("minecraft:bread", 1, 5, 14),
        item("minecraft:apple", 1, 5, 10),
        item("minecraft:wheat", 1, 6, 10),
        item("minecraft:potato", 1, 6, 9),
        item("minecraft:carrot", 1, 6, 9),
        item("minecraft:beetroot", 1, 5, 7),
        item("minecraft:glass_bottle", 1, 3, 6),
        item("minecraft:bowl", 1, 3, 6),
        item("minecraft:coal", 1, 5, 5),
    ]
}

fn living_room_pool() -> Vec<LootEntry> {
    vec![
        item("minecraft:book", 1, 4, 14),
        item("minecraft:paper", 1, 8, 10),
        item("minecraft:apple", 1, 3, 8),
        item("minecraft:bread", 1, 3, 7),
        item("minecraft:feather", 1, 5, 7),
        item("minecraft:string", 1, 4, 6),
        item("minecraft:stick", 2, 8, 6),
    ]
}

fn bathroom_pool() -> Vec<LootEntry> {
    vec![
        item("minecraft:glass_bottle", 1, 4, 15),
        item("minecraft:paper", 1, 8, 14),
        item("minecraft:leather", 1, 2, 5),
        item("minecraft:water_bucket", 1, 1, 3),
        item("minecraft:coal", 1, 3, 3),
    ]
}

fn office_pool() -> Vec<LootEntry> {
    vec![
        item("minecraft:paper", 2, 10, 15),
        item("minecraft:book", 1, 5, 14),
        item("minecraft:feather", 1, 5, 10),
        item("minecraft:ink_sac", 1, 3, 7),
        item("minecraft:stick", 1, 4, 5),
        item("minecraft:iron_nugget", 1, 3, 4),
    ]
}

fn storage_pool() -> Vec<LootEntry> {
    vec![
        item("minecraft:stick", 2, 12, 12),
        item("minecraft:oak_planks", 2, 12, 12),
        item("minecraft:coal", 2, 8, 10),
        item("minecraft:iron_nugget", 1, 5, 7),
        item("minecraft:string", 1, 6, 8),
        item("minecraft:leather", 1, 4, 7),
        item("minecraft:paper", 1, 6, 5),
    ]
}

fn dining_pool() -> Vec<LootEntry> {
    vec![
        item("minecraft:bread", 1, 5, 14),
        item("minecraft:apple", 1, 4, 10),
        item("minecraft:bowl", 1, 4, 10),
        item("minecraft:wheat", 1, 5, 8),
        item("minecraft:potato", 1, 5, 7),
        item("minecraft:carrot", 1, 5, 7),
        item("minecraft:glass_bottle", 1, 3, 5),
    ]
}

/// ------------------------------------------------------------
/// Room type classification
/// ------------------------------------------------------------
///
/// We intentionally use Debug text instead of hard-coding the
/// RoomType enum variants.
///
/// This makes this module resilient if RoomType gets additional
/// variants later.
///
/// The semantic RoomType itself remains untouched.
/// ------------------------------------------------------------
fn classify_room(room_type_debug: &str) -> Option<Vec<LootEntry>> {
    let name = room_type_debug.to_ascii_lowercase();

    if name.contains("bedroom")
        || name.contains("sleep")
        || name.contains("dorm")
    {
        return Some(bedroom_pool());
    }

    if name.contains("kitchen")
        || name.contains("cook")
    {
        return Some(kitchen_pool());
    }

    if name.contains("bathroom")
        || name.contains("toilet")
        || name.contains("shower")
        || name.contains("wash")
    {
        return Some(bathroom_pool());
    }

    if name.contains("office")
        || name.contains("study")
        || name.contains("work")
    {
        return Some(office_pool());
    }

    if name.contains("storage")
        || name.contains("utility")
        || name.contains("closet")
    {
        return Some(storage_pool());
    }

    if name.contains("dining")
        || name.contains("restaurant")
    {
        return Some(dining_pool());
    }

    if name.contains("living")
        || name.contains("lounge")
        || name.contains("family")
    {
        return Some(living_room_pool());
    }

    None
}

/// ------------------------------------------------------------
/// Loot generation
/// ------------------------------------------------------------

fn pick_weighted<'a>(
    pool: &'a [LootEntry],
    rng: &mut impl Rng,
) -> &'a LootEntry {
    let total: u32 = pool.iter().map(|entry| entry.weight).sum();

    let mut pick = rng.random_range(0..total);

    for entry in pool {
        if pick < entry.weight {
            return entry;
        }

        pick -= entry.weight;
    }

    &pool[pool.len() - 1]
}

fn build_items(
    pool: &[LootEntry],
    rng: &mut impl Rng,
) -> Vec<HashMap<String, Value>> {
    let count = rng.random_range(1..=MAX_ITEMS_PER_CHEST);
    let mut used_slots = [false; 27];
    let mut output = Vec::with_capacity(count);

    for _ in 0..count {
        let entry = pick_weighted(pool, rng);

        let mut slot = None;

        for _ in 0..8 {
            let candidate = rng.random_range(0..27);

            if !used_slots[candidate] {
                slot = Some(candidate);
                break;
            }
        }

        let Some(slot) = slot else {
            continue;
        };

        used_slots[slot] = true;

        let amount = rng.random_range(entry.min..=entry.max);

        let mut item_nbt = HashMap::new();

        item_nbt.insert(
            "id".to_string(),
            Value::String(entry.id.to_string()),
        );

        item_nbt.insert(
            "Slot".to_string(),
            Value::Byte(slot as i8),
        );

        // Minecraft 1.20.5+ item component format used by the
        // existing RWM container implementation.
        item_nbt.insert(
            "count".to_string(),
            Value::Int(amount),
        );

        output.push(item_nbt);
    }

    output
}

/// ------------------------------------------------------------
/// Find a safe physical placement cell
/// ------------------------------------------------------------
///
/// The room's actual physical cells are supplied by the renderer.
/// We never search outside them.
///
/// Prefer an interior-ish cell, then fall back to the first cell.
/// ------------------------------------------------------------
fn choose_container_cell(
    cells: &[(i32, i32)],
) -> Option<(i32, i32)> {
    if cells.is_empty() {
        return None;
    }

    let min_x = cells.iter().map(|(x, _)| *x).min()?;
    let max_x = cells.iter().map(|(x, _)| *x).max()?;
    let min_z = cells.iter().map(|(_, z)| *z).min()?;
    let max_z = cells.iter().map(|(_, z)| *z).max()?;

    let center_x = (min_x + max_x) / 2;
    let center_z = (min_z + max_z) / 2;

    cells
        .iter()
        .min_by_key(|(x, z)| {
            (*x - center_x).abs() + (*z - center_z).abs()
        })
        .copied()
}

/// ------------------------------------------------------------
/// Public room materialization API
/// ------------------------------------------------------------

pub fn generate_room_loot(
    editor: &mut WorldEditor,
    room_type_debug: &str,
    physical_cells: &[(i32, i32)],
    floor_y: i32,
) -> bool {
    // HARD RULE:
    // No actual physical room -> no loot.
    if physical_cells.is_empty() {
        println!(
            "[RWM ROOM LOOT] SKIP empty physical room type={}",
            room_type_debug
        );

        return false;
    }

    let Some((x, z)) = choose_container_cell(physical_cells) else {
        println!(
            "[RWM ROOM LOOT] SKIP no valid placement cell type={}",
            room_type_debug
        );

        return false;
    };

    let mut rng = rand::rng();

    let (pool, fallback) = match classify_room(room_type_debug) {
        Some(pool) => (pool, false),
        None => (generic_fallback_pool(), true),
    };

    let items = build_items(&pool, &mut rng);

    if items.is_empty() {
        println!(
            "[RWM ROOM LOOT] SKIP empty loot type={}",
            room_type_debug
        );

        return false;
    }

    let absolute_y = editor.get_absolute_y(x, floor_y + 1, z);

    editor.set_block_entity_with_items_absolute(
        BlockWithProperties::new(CHEST, None),
        x,
        absolute_y,
        z,
        "minecraft:chest",
        items,
    );

    println!(
        "[RWM ROOM LOOT] {} chest room={} cell=({}, {}) floor_y={} physical_cells={}",
        if fallback { "FALLBACK" } else { "ROOM" },
        room_type_debug,
        x,
        z,
        floor_y,
        physical_cells.len()
    );

    true
}
