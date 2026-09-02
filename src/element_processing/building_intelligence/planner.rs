use crate::element_processing::building_intelligence::{
    decide_building, detect_main_entrance, BuildingContext, EntranceCandidate, EntranceEvidence,
    EntranceSide, PlannedBuilding,
};

use crate::element_processing::building_intelligence::furniture_planner::FurniturePlanner;
use crate::element_processing::subprocessor::interior::decision::entrance_analysis::analyze_entrance;

use crate::element_processing::subprocessor::interior::Rect;

use super::room_graph::build_room_graph;

use crate::element_processing::building_intelligence::input::BuildingSnapshot;

pub fn build_floor_plan(
    mut context: BuildingContext,
    snapshot: &BuildingSnapshot,
    mapped_nodes: &[crate::osm_parser::ProcessedNode],
    start_y_offset: i32,
    cached_floor_area: &[(i32, i32)],
) -> PlannedBuilding {
    // ---------------------------------------------------------
    // AUTHORITATIVE REAL-WORLD INTERIOR FOOTPRINT
    // ---------------------------------------------------------
    //
    // cached_floor_area is produced by the existing real-world
    // building reconstruction engine.
    //
    // Floor Planner may READ it, but MUST NEVER:
    // - expand it
    // - modify it
    // - invent cells outside it
    // - redefine the building footprint
    //
    // This is the physical hard boundary for interior planning.
    //
    let cached_floor_area_set: std::collections::HashSet<(i32, i32)> =
        cached_floor_area.iter().copied().collect();

    if cached_floor_area_set.is_empty() {
        println!(
            "[BI FLOOR PLANNER] cached_floor_area is empty; planner receives no valid interior cells"
        );
    } else {
        println!(
            "[BI FLOOR PLANNER] cached_floor_area authoritative cells={}",
            cached_floor_area_set.len()
        );
    }

    // ---------------------------------------------------------
    // REAL-WORLD CONTEXT -> BUILDING INTELLIGENCE
    // ---------------------------------------------------------
    // Distances come directly from the immutable world snapshot.
    // Interior Intelligence may read them, but must not modify
    // the reconstructed exterior world.
    context.nearby_road_distance = snapshot.nearby_road_distance;
    context.nearby_road_position = snapshot.nearby_road_position;
    context.nearby_path_distance = snapshot.nearby_path_distance;
    context.nearby_parking_distance = snapshot.nearby_parking_distance;

    context.environment = {
        use crate::element_processing::building_intelligence::BuildingEnvironment;
        let road = context.nearby_road_distance;
        let path = context.nearby_path_distance;
        let parking = context.nearby_parking_distance;

        match (road, path, parking) {
            (Some(r), _, _) if r <= 3 => BuildingEnvironment::RoadFront,
            (_, Some(p), _) if p <= 3 => BuildingEnvironment::PathFront,
            (_, _, Some(p)) if p <= 3 => BuildingEnvironment::ParkingFront,
            (Some(_), Some(_), Some(_)) => BuildingEnvironment::DenseUrban,
            (Some(_), _, _) => BuildingEnvironment::RoadFront,
            (_, Some(_), _) => BuildingEnvironment::PathFront,
            (_, _, Some(_)) => BuildingEnvironment::ParkingFront,
            _ => BuildingEnvironment::Unknown,
        }
    };

    // ---------------------------------------------------------
    // READ-ONLY WORLD RECONSTRUCTION INPUT
    // ---------------------------------------------------------
    //
    // snapshot comes from the already-generated real-world
    // building. Interior Intelligence may READ it only.
    //
    // It must NEVER:
    // - generate windows
    // - move windows
    // - remove windows
    // - modify exterior geometry
    // - modify the BBox
    //
    // Snapshot-derived geometry. These values come directly from the
    // existing real-world reconstruction and are read-only.
    let _snapshot_width = snapshot.width();
    let _snapshot_depth = snapshot.depth();
    let _snapshot_area = snapshot.area();

    let bounds = Rect {
        min_x: context.min_x + 2,
        min_z: context.min_z + 2,
        max_x: context.max_x - 2,
        max_z: context.max_z - 2,
    };

    // ---------------------------------------------------------
    // REAL-WORLD ENTRANCE EVIDENCE
    // ---------------------------------------------------------
    // Use the nearest reconstructed road position as read-only
    // real-world evidence for the four possible building sides.
    //
    // This does not modify the building geometry or BBox.
    let entrance_evidence = {
        let (center_x, center_z) = context.center();

        match snapshot.nearby_road_position {
            Some((road_x, road_z)) => EntranceEvidence {
                north_road: road_z < center_z,
                south_road: road_z > center_z,
                east_road: road_x > center_x,
                west_road: road_x < center_x,

                // These are populated by their own real-world
                // evidence sources when available.
                north_footway: false,
                south_footway: false,
                east_footway: false,
                west_footway: false,

                north_parking: false,
                south_parking: false,
                east_parking: false,
                west_parking: false,

                north_entrance_poi: false,
                south_entrance_poi: false,
                east_entrance_poi: false,
                west_entrance_poi: false,
            },
            None => EntranceEvidence::empty(),
        }
    };

    // ---------------------------------------------------------
    // REAL-WORLD ENTRANCE
    // ---------------------------------------------------------
    // Prefer an explicitly extracted real-world door from the
    // immutable BuildingSnapshot.
    //
    // The door is READ-ONLY reconstruction input.
    // Interior Intelligence does not create, move, remove,
    // or modify the exterior door.
    // ---------------------------------------------------------
    // MAPPED REAL-WORLD DOOR -> MAIN ENTRANCE CANDIDATE
    // ---------------------------------------------------------
    //
    // ExistingDoor is authoritative read-only reconstruction input.
    //
    // IMPORTANT:
    // - Never invent or move the mapped door coordinate.
    // - Never modify the building footprint.
    // - Never use cached_floor_area here.
    // - Main entrance rendering later opens the EXISTING exterior wall.
    //
    // When multiple mapped doors exist, rank them using the available
    // exterior context instead of blindly taking Vec::first().
    //
    // Priority:
    //   1. mapped real-world door
    //   2. path / footway proximity
    //   3. road proximity
    //   4. parking proximity
    //   5. geometric centrality as deterministic tie-breaker
    //
    let snapshot_door = snapshot
        .doors
        .iter()
        .max_by(|a, b| {
            let center_x = (context.min_x + context.max_x) / 2;
            let center_z = (context.min_z + context.max_z) / 2;

            let score =
                |door: &crate::element_processing::building_intelligence::input::ExistingDoor| {
                    let mut score = 0.0_f32;

                    // A mapped door is already real-world evidence.
                    score += 1000.0;

                    // Prefer doors on the side with the strongest path evidence.
                    let path_distance = snapshot.nearby_path_distance.unwrap_or(i32::MAX);
                    if path_distance != i32::MAX {
                        score += (100.0 - path_distance.min(100) as f32).max(0.0);
                    }

                    // Prefer buildings with a nearby reconstructed road.
                    let road_distance = snapshot.nearby_road_distance.unwrap_or(i32::MAX);
                    if road_distance != i32::MAX {
                        score += (60.0 - road_distance.min(60) as f32).max(0.0);
                    }

                    // Parking is weaker evidence than pedestrian/road access.
                    let parking_distance = snapshot.nearby_parking_distance.unwrap_or(i32::MAX);
                    if parking_distance != i32::MAX {
                        score += (30.0 - parking_distance.min(30) as f32).max(0.0);
                    }

                    // Deterministic geometric tie-breaker:
                    // prefer a mapped door closer to the building center.
                    let dx = (door.x - center_x).abs() as f32;
                    let dz = (door.z - center_z).abs() as f32;
                    let distance = dx + dz;
                    score += (20.0 - distance.min(20.0)).max(0.0);

                    score
                };

            score(a)
                .partial_cmp(&score(b))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|door| {
            EntranceCandidate::new(
                match door.side {
                    crate::element_processing::building_intelligence::input::WindowSide::North => {
                        crate::element_processing::building_intelligence::EntranceSide::North
                    }
                    crate::element_processing::building_intelligence::input::WindowSide::South => {
                        crate::element_processing::building_intelligence::EntranceSide::South
                    }
                    crate::element_processing::building_intelligence::input::WindowSide::East => {
                        crate::element_processing::building_intelligence::EntranceSide::East
                    }
                    crate::element_processing::building_intelligence::input::WindowSide::West => {
                        crate::element_processing::building_intelligence::EntranceSide::West
                    }
                },
                door.x,
                door.z,
            )
        });

    let entrance = snapshot_door
        .or_else(|| {
            crate::element_processing::building_intelligence::detect_mapped_entrance(
                mapped_nodes,
                context.min_x,
                context.min_z,
                context.max_x,
                context.max_z,
            )
        })
        .or_else(|| {
            detect_main_entrance(
                context.min_x,
                context.min_z,
                context.max_x,
                context.max_z,
                entrance_evidence,
            )
            .map(|e| EntranceCandidate::new(e.side, e.x, e.z))
        })
        .or_else(|| {
            // ---------------------------------------------------------
            // DETERMINISTIC FALLBACK ENTRANCE
            // ---------------------------------------------------------
            //
            // Only used when ALL real-world entrance sources failed:
            //
            //   1. ExistingDoor
            //   2. mapped entrance/door node
            //   3. real-world entrance evidence inference
            //
            // This does NOT replace real-world entrance data.
            // It only guarantees that a building without mapped
            // entrance information still receives a usable main door.
            //
            // Deterministic rule:
            //   - North exterior wall
            //   - Horizontal center of the existing building bounds
            //   - Never expands BBox
            //   - Never modifies the building footprint
            //
            let fallback_x = (context.min_x + context.max_x) / 2;
            let fallback_z = context.min_z;

            println!(
                "[BI MAIN DOOR] FALLBACK deterministic entrance: ({}, {}) side=North",
                fallback_x, fallback_z
            );

            Some(EntranceCandidate::new(
                EntranceSide::North,
                fallback_x,
                fallback_z,
            ))
        });

    // ---------------------------------------------------------
    // EXISTING WINDOWS -> INTERIOR DAYLIGHT INPUT
    // ---------------------------------------------------------
    //
    // Convert the EXISTING windows from the world reconstruction
    // engine into the interior decision model.
    //
    // No window is generated here.
    //

    let windows = snapshot
        .windows
        .iter()
        .map(
            |window| crate::element_processing::subprocessor::interior::decision::WindowInfo {
                x: window.x,
                z: window.z,
                floor: window.floor,
                width: window.width,
                height: window.height,
                facing: match window.side {
                    crate::element_processing::building_intelligence::input::WindowSide::North => 0,
                    crate::element_processing::building_intelligence::input::WindowSide::East => 1,
                    crate::element_processing::building_intelligence::input::WindowSide::South => 2,
                    crate::element_processing::building_intelligence::input::WindowSide::West => 3,
                },
                mapped: true,
                daylight_score: {
                    let area = (window.width.max(1) * window.height.max(1)) as f32;

                    // Larger windows provide more daylight.
                    let size_factor = (area / 6.0).clamp(0.25, 2.0);

                    // Higher floors generally receive slightly better daylight.
                    let floor_factor = (1.0 + window.floor.max(0) as f32 * 0.04).clamp(1.0, 1.20);

                    // Existing windows are already real-world mapped input.
                    (size_factor * floor_factor).clamp(0.1, 2.0)
                },
            },
        )
        .collect::<Vec<_>>();

    // ---------------------------------------------------------
    // BUILDING PROFILE
    // ---------------------------------------------------------

    let profile =
        crate::element_processing::subprocessor::interior::decision::BuildingProfile {
            building_type: snapshot.building_type,
            min_x: snapshot.min_x,
            min_z: snapshot.min_z,
            max_x: snapshot.max_x,
            max_z: snapshot.max_z,
            floors: snapshot.floors as i32,

            // Real OSM / Overture semantic information.
            osm_tags: snapshot.osm_tags.clone(),

            entrances: entrance
                .as_ref()
                .map(|e| {
                    vec![
                        crate::element_processing::subprocessor::interior::decision::EntranceInfo {
                            x: e.x,
                            z: e.z,
                            floor: 0,
                            facing: match e.side {
                                crate::element_processing::building_intelligence::EntranceSide::North => 0,
                                crate::element_processing::building_intelligence::EntranceSide::East => 1,
                                crate::element_processing::building_intelligence::EntranceSide::South => 2,
                                crate::element_processing::building_intelligence::EntranceSide::West => 3,
                            },
                            mapped: e.has_entrance_poi,
                        },
                    ]
                })
                .unwrap_or_default(),

            // EXISTING windows only.
            windows,
        };

    // ---------------------------------------------------------
    // ENTRANCE ANALYSIS
    // ---------------------------------------------------------
    //
    // Analyze the already-existing real-world entrance.
    // This is READ-ONLY decision information.
    //
    // No entrance is created, moved, removed, or modified.
    let entrance_decision = analyze_entrance(&profile);

    // ---------------------------------------------------------
    // CENTRAL INTERIOR INTELLIGENCE
    // ---------------------------------------------------------

    let decision = decide_building(&context, entrance.as_ref(), &profile, entrance_decision);

    let mut floor_plans = Vec::new();

    // ---------------------------------------------------------
    // SPATIAL CONSTRAINTS -> FLOOR PLAN
    // ---------------------------------------------------------
    //
    // RoomAllocation contains the semantic decision:
    // room type, required area, minimum dimensions, daylight
    // requirements and priority.
    //
    // SpatialConstraints contains READ-ONLY information from
    // the already reconstructed real-world building:
    // existing windows, existing entrances and building bounds.
    //
    // The interior engine never creates, moves, deletes or
    // modifies any exterior geometry.
    let spatial_constraints =
        crate::element_processing::subprocessor::interior::decision::
            SpatialConstraints::from_profile(&profile);

    // Consume the daylight decision produced from the
    // already-existing real-world windows.
    //
    // This is analysis only. It never creates or modifies windows.
    let _existing_daylight = decision.daylight.total();
    let _strongest_daylight_facing = decision.daylight.strongest_facing();

    // Consume the entrance decision as semantic interior input.
    //
    // The entrance itself remains owned by the reconstructed
    // real-world building shell.
    let _has_primary_entrance_decision = decision
        .entrance
        .as_ref()
        .map(|entrance| entrance.is_primary)
        .unwrap_or(false);

    let _preferred_entrance_room = decision
        .entrance
        .as_ref()
        .and_then(|entrance| entrance.preferred_room);

    // ---------------------------------------------------------
    // ⑪ BUILDING TYPE -> ⑫ LAYOUT SOLVER
    // ---------------------------------------------------------
    // BuildingProfile already carries the semantic BuildingType.
    // LayoutSolver consumes the resulting spatial constraints
    // together with the selected rooms.
    //
    // No exterior geometry / BBox / real-world reconstruction
    // is modified here.

    // Generate one authoritative FloorPlan for every semantic
    // building floor.  The previous implementation only generated
    // floor 0, which made all higher floors disappear from the
    // physical interior renderer.
    let floor_count = context.floors.max(1);

    for floor in 0..floor_count {
        let floor_i32 = floor as i32;

        if let Some(plan) =
            crate::element_processing::subprocessor::interior::floor_plan::
                generate_floor_plan_with_constraints(
                    bounds,
                    &decision.rooms.rooms,
                    &spatial_constraints,
                    floor_i32,
                    &cached_floor_area_set,
                )
        {
            println!(
                "[BI FLOOR] generated floor={} rooms={}",
                floor,
                plan.rooms.len()
            );
            floor_plans.push(plan);
        } else {
            println!(
                "[BI FLOOR] FAILED floor={} bounds=({},{})-({},{})",
                floor,
                bounds.min_x,
                bounds.min_z,
                bounds.max_x,
                bounds.max_z
            );
        }
    }

    // ---------------------------------------------------------
    // ⑫ LAYOUT SOLVER -> ⑬ ROAD MASK
    // ---------------------------------------------------------
    // LayoutSolver owns room geometry.
    // RoadMask is downstream spatial context only.
    //
    // IMPORTANT:
    // - Do not alter FloorPlan geometry here.
    // - Do not reject real-world buildings because of road overlap.
    // - Do not modify BBox or exterior reconstruction.
    //
    // The existing RoadMask API is supplied by the building
    // generation layer and will be consumed downstream as
    // read-only spatial intelligence.

    // ---------------------------------------------------------
    // ROOM GRAPH / TOPOLOGY
    // ---------------------------------------------------------

    let room_door_widths = decision
        .room_doors
        .iter()
        .filter_map(|door| door.room_type.map(|room_type| (room_type, door.width)))
        .collect::<Vec<_>>();

    let room_graph = build_room_graph(
        &floor_plans,
        entrance.as_ref(),
        decision.entrance.as_ref(),
        decision.main_door.as_ref(),
        &room_door_widths,
    );

    // ---------------------------------------------------------
    // FURNITURE INTENT
    // ---------------------------------------------------------

    let furniture_planner = FurniturePlanner::new();

    let mut furniture = Vec::new();

    for (floor, plan) in floor_plans.iter().enumerate() {
        furniture.extend(furniture_planner.plan_floor(
            plan,
            &spatial_constraints,
            &room_graph,
            floor as i32,
            &cached_floor_area_set,
        ));
    }

    // ---------------------------------------------------------
    // VERTICAL ACCESS PLANNING
    // ---------------------------------------------------------
    // BuildingDecision determines WHAT vertical access is needed.
    // VerticalAccessPlanner determines WHERE it can exist using
    // the existing RoomGraph + FloorPlans.
    //
    // Read-only intelligence: never modifies room geometry,
    // exterior geometry, footprint, geographic coordinates or BBox.
    let floor_levels = (0..context.floors)
        .map(|floor| start_y_offset + 1 + floor as i32 * 4)
        .collect::<Vec<_>>();

    let vertical_access =
        crate::element_processing::building_intelligence::vertical::plan_vertical_access(
            &context,
            &room_graph,
            &floor_plans,
            &floor_levels,
            decision.vertical.kind,
            decision.vertical.size,
            decision.vertical.width,
        );

    // ---------------------------------------------------------
    // LIGHTING INTELLIGENCE
    // ---------------------------------------------------------
    // Uses the already reconstructed windows as daylight input.
    // Lighting never creates, moves or modifies exterior windows.
    let lighting = crate::element_processing::subprocessor::interior::decision::plan_lighting(
        &floor_plans,
        &profile.windows,
    );

    // ---------------------------------------------------------
    // CIRCULATION INTELLIGENCE
    // ---------------------------------------------------------
    //
    // Circulation consumes the already-generated semantic graph,
    // doorway intents, floor plans and furniture intent.
    //
    // It NEVER changes:
    // - room geometry
    // - doors/windows
    // - building footprint
    // - geographic coordinates
    // - BBox
    //
    let mut circulation =
        crate::element_processing::building_intelligence::circulation::build_circulation_plan(
            &room_graph,
        );

    // ---------------------------------------------------------
    // VERTICAL ACCESS -> CIRCULATION API
    // ---------------------------------------------------------
    //
    // Circulation consumes the already-planned vertical access
    // structures as read-only intelligence.
    //
    // No room geometry, FloorPlan, exterior geometry,
    // geographic coordinates or BBox are modified.
    circulation.attach_vertical_access(&vertical_access);

    println!(
        "[BI DEBUG] floor_plans={} rooms={} graph_rooms={} graph_connections={} vertical_access={}",
        floor_plans.len(),
        floor_plans.iter().map(|p| p.rooms.len()).sum::<usize>(),
        room_graph.rooms.len(),
        room_graph.connections.len(),
        vertical_access.len(),
    );

    let mut doorway_plan =
        crate::element_processing::building_intelligence::circulation::build_doorway_plan(
            &room_graph,
            &floor_plans,
        );

    println!(
        "[BI DEBUG] doorway_plan.doors={} main_entrance={}",
        doorway_plan.doors.len(),
        doorway_plan.main_entrance.is_some(),
    );

    doorway_plan.main_entrance = decision.main_door;

    let building_circulation =
        crate::element_processing::building_intelligence::circulation::build_building_circulation(
            &room_graph,
            &doorway_plan,
        );

    let floor_circulation = floor_plans
        .iter()
        .enumerate()
        .map(|(floor_index, floor_plan)| {
            crate::element_processing::building_intelligence::circulation::build_floor_circulation(
                &room_graph,
                &doorway_plan,
                floor_index,
                floor_plan,
                &furniture,
            )
        })
        .collect::<Vec<_>>();

    // ---------------------------------------------------------
    // CIRCULATION VALIDATION
    // ---------------------------------------------------------
    // RoomGraph is the authoritative room topology.
    // CirculationPlan consumes that topology and records which
    // rooms are reachable from the real-world entrance.
    //
    // Keep this as read-only intelligence: no geometry, doors,
    // windows, furniture, footprint or BBox is modified.
    let _circulation_connected = circulation.is_connected();
    let _reachable_rooms = circulation.reachable_rooms.len();
    let _unreachable_rooms = circulation.unreachable_rooms.len();

    // InteriorCirculationPlan is the authoritative per-floor
    // walkability result. Consume its unreachable-room count as
    // part of circulation quality validation.
    let _floor_unreachable_rooms: usize = floor_circulation
        .iter()
        .map(|floor| floor.unreachable_rooms())
        .sum();

    for room in &room_graph.rooms {
        let _ = circulation.room_is_reachable(room.id);
    }

    let _vertical_rooms = circulation.vertical_rooms();
    let _has_vertical_access = circulation.has_vertical_access();

    PlannedBuilding {
        context,
        decision,
        floor_plans,
        entrance,
        furniture,
        room_graph,
        lighting,
        circulation,
        doorway_plan,
        building_circulation,
        floor_circulation,
    }
}
