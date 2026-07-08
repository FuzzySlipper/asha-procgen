fn build_emit_piece_plan_command(args: BuildEmitPiecePlanArgs) -> Result<(), String> {
    let candidate: Candidate = read_json(&args.candidate)?;
    let intermediate: IntermediateBreakdown = read_json(&args.intermediate)?;
    let geometry: Geometry2dArtifact = read_json(&args.geometry)?;
    let plan = emit_piece_build_plan(&candidate, &intermediate, &geometry, &args)?;
    write_json(&args.out, &plan)
}

fn emit_piece_build_plan(
    candidate: &Candidate,
    intermediate: &IntermediateBreakdown,
    geometry: &Geometry2dArtifact,
    args: &BuildEmitPiecePlanArgs,
) -> Result<PieceBuildPlan, String> {
    validate_piece_plan_inputs(candidate, intermediate, geometry)?;

    let regions_by_id = intermediate
        .regions
        .iter()
        .map(|region| (region.id.as_str(), region))
        .collect::<BTreeMap<_, _>>();
    let connectors_by_id = intermediate
        .connectors
        .iter()
        .map(|connector| (connector.id.as_str(), connector))
        .collect::<BTreeMap<_, _>>();
    let edges_by_id = candidate
        .graph
        .edges
        .iter()
        .map(|edge| (edge.id.as_str(), edge))
        .collect::<BTreeMap<_, _>>();
    let contents_by_room = geometry_contents_by_room(geometry);

    let mut requirements = Vec::new();
    let mut links = Vec::new();
    let mut content_requirements = Vec::new();
    let mut room_piece_ids = BTreeMap::new();

    for room in &geometry.rooms {
        let region = regions_by_id.get(room.source_region.as_str()).copied();
        let room_contents = contents_by_room
            .get(room.id.as_str())
            .cloned()
            .unwrap_or_default();
        let piece_id = piece_id_for_room(room);
        room_piece_ids.insert(room.id.as_str(), piece_id.clone());
        let required_exits = room_exit_requirements(room, geometry);
        let required_sockets = dedupe_strings(
            room_contents
                .iter()
                .map(|content| socket_for_content_kind(content.kind.as_str()))
                .collect(),
        );
        let mut tags = vec![
            room.role.clone(),
            room.geometry_role.clone(),
            room.footprint_class.clone(),
        ];
        tags.extend(room.style_tags.clone());
        tags.extend(room_contents.iter().map(|content| content.kind.clone()));
        if let Some(region) = region {
            tags.extend(region.entrance_expectations.clone());
        }

        requirements.push(PieceRequirement {
            piece_id: piece_id.clone(),
            kind: piece_kind_for_room(room, &room_contents),
            role: room.role.clone(),
            source_refs: room_source_refs(room),
            required_exits,
            required_sockets,
            tags: dedupe_strings(tags),
            placement_hints: room_placement_hints(room, region),
        });

        for content in room_contents {
            content_requirements.push(PieceContentRequirement {
                id: format!(
                    "piece_content.{}.{}",
                    slugify_label(piece_id.as_str()),
                    slugify_label(content.kind.as_str())
                ),
                piece_id: piece_id.clone(),
                source_ref: content.source_ref.clone(),
                kind: content.kind.clone(),
                label: content.label.clone(),
                required_socket: socket_for_content_kind(content.kind.as_str()),
                tags: dedupe_strings(content.tags.clone()),
            });
        }
    }

    for corridor in &geometry.corridors {
        let Some(from_piece) = room_piece_ids.get(corridor.from_room.as_str()).cloned() else {
            continue;
        };
        let Some(to_piece) = room_piece_ids.get(corridor.to_room.as_str()).cloned() else {
            continue;
        };
        let connector = connectors_by_id
            .get(corridor.source_connector.as_str())
            .copied();
        let edge = edges_by_id.get(corridor.source_edge.as_str()).copied();
        let corridor_piece_ids =
            emit_corridor_piece_requirements(corridor, connector, edge, &mut requirements);
        link_piece_chain(
            corridor,
            &from_piece,
            &to_piece,
            &corridor_piece_ids,
            &mut links,
        );
    }

    Ok(PieceBuildPlan {
        kind: "asha_procgen.piece_build_plan.v1".to_owned(),
        schema_version: 1,
        plan_id: format!("piece_plan.{}", geometry.geometry_id),
        candidate_id: candidate.candidate_id.clone(),
        geometry_id: geometry.geometry_id.clone(),
        source_candidate_ref: display_path(&args.candidate),
        source_intermediate_ref: display_path(&args.intermediate),
        source_geometry_ref: display_path(&args.geometry),
        requirements,
        links,
        content_requirements,
    })
}

fn validate_piece_plan_inputs(
    candidate: &Candidate,
    intermediate: &IntermediateBreakdown,
    geometry: &Geometry2dArtifact,
) -> Result<(), String> {
    if intermediate.candidate_id != candidate.candidate_id {
        return Err(format!(
            "intermediate candidate {} does not match candidate {}",
            intermediate.candidate_id, candidate.candidate_id
        ));
    }
    if geometry.candidate_id != candidate.candidate_id {
        return Err(format!(
            "geometry candidate {} does not match candidate {}",
            geometry.candidate_id, candidate.candidate_id
        ));
    }
    if geometry.kind != "asha_procgen.geometry_2d.v1" {
        return Err(format!(
            "piece plan requires geometry kind asha_procgen.geometry_2d.v1, got {}",
            geometry.kind
        ));
    }
    Ok(())
}

fn geometry_contents_by_room(
    geometry: &Geometry2dArtifact,
) -> BTreeMap<&str, Vec<&GeometryContent>> {
    let mut by_room: BTreeMap<&str, Vec<&GeometryContent>> = BTreeMap::new();
    for content in &geometry.contents {
        by_room
            .entry(content.room_id.as_str())
            .or_default()
            .push(content);
    }
    by_room
}

fn piece_id_for_room(room: &GeometryRoom) -> String {
    format!("piece.room.{}", slugify_label(room.id.as_str()))
}

fn piece_kind_for_room(room: &GeometryRoom, contents: &[&GeometryContent]) -> String {
    if room.role == "boss_gate" || room.geometry_role == "boss_threshold" {
        "boss".to_owned()
    } else if room.role == "gate" || room.geometry_role == "threshold" {
        "threshold".to_owned()
    } else if room.role == "pressure" || room.geometry_role == "pressure_lane" {
        "hazard".to_owned()
    } else if room.role == "reward" || room.geometry_role == "reward_pocket" {
        "reward".to_owned()
    } else if room.role == "secret"
        || room.geometry_role.contains("secret")
        || contents
            .iter()
            .any(|content| content.kind == "secret_route_marker")
    {
        "secret".to_owned()
    } else if room.role == "shortcut"
        || room.geometry_role.contains("shortcut")
        || contents
            .iter()
            .any(|content| content.kind == "shortcut_marker")
    {
        "shortcut".to_owned()
    } else if room.role == "resource"
        || room.geometry_role.contains("resource")
        || contents
        .iter()
        .any(|content| content.kind == "resource_clue")
    {
        "resource".to_owned()
    } else if contents.iter().any(|content| content.kind == "key_pickup") {
        "key".to_owned()
    } else {
        "room".to_owned()
    }
}

fn room_source_refs(room: &GeometryRoom) -> Vec<String> {
    let mut refs = vec![
        format!("geometryRoom:{}", room.id),
        format!("region:{}", room.source_region),
    ];
    refs.extend(
        room.source_nodes
            .iter()
            .map(|node_id| format!("node:{node_id}")),
    );
    refs
}

fn room_placement_hints(
    room: &GeometryRoom,
    region: Option<&IntermediateRegion>,
) -> Vec<String> {
    let mut hints = vec![
        format!("geometryRect:{}:{}:{}:{}", room.rect.x, room.rect.y, room.rect.width, room.rect.height),
        format!("footprintClass:{}", room.footprint_class),
    ];
    if let Some(region) = region {
        hints.push(format!("scaleBand:{}", region.scale_band));
        hints.push(format!("anchorQuality:{}", region.anchor_quality));
    }
    dedupe_strings(hints)
}

fn room_exit_requirements(
    room: &GeometryRoom,
    geometry: &Geometry2dArtifact,
) -> Vec<PieceExitRequirement> {
    let mut exits = Vec::new();
    for corridor in &geometry.corridors {
        if corridor.from_room == room.id {
            if let Some(direction) = corridor_endpoint_direction(corridor, true) {
                exits.push(PieceExitRequirement {
                    id: format!("exit.{}.{}", slugify_label(corridor.id.as_str()), direction),
                    direction,
                    width: corridor.width,
                    tags: dedupe_strings(corridor.semantic_tags.clone()),
                });
            }
        } else if corridor.to_room == room.id {
            if let Some(direction) = corridor_endpoint_direction(corridor, false) {
                exits.push(PieceExitRequirement {
                    id: format!("exit.{}.{}", slugify_label(corridor.id.as_str()), direction),
                    direction,
                    width: corridor.width,
                    tags: dedupe_strings(corridor.semantic_tags.clone()),
                });
            }
        }
    }
    exits
}

fn emit_corridor_piece_requirements(
    corridor: &GeometryCorridor,
    connector: Option<&IntermediateConnector>,
    edge: Option<&Edge>,
    requirements: &mut Vec<PieceRequirement>,
) -> Vec<String> {
    let source_refs = corridor_source_refs(corridor, connector, edge);
    let base_tags = corridor_tags(corridor, connector, edge);
    let mut piece_ids = Vec::new();

    let start_id = format!("piece.connector.{}.start", slugify_label(corridor.id.as_str()));
    requirements.push(PieceRequirement {
        piece_id: start_id.clone(),
        kind: "connector".to_owned(),
        role: "corridor_connector".to_owned(),
        source_refs: source_refs.clone(),
        required_exits: connector_exits(corridor, true),
        required_sockets: Vec::new(),
        tags: base_tags.clone(),
        placement_hints: vec!["glue:from_room".to_owned()],
    });
    piece_ids.push(start_id);

    for (index, pair) in corridor.points.windows(2).enumerate() {
        let Some(direction) = direction_between_points(&pair[0], &pair[1]) else {
            continue;
        };
        let segment_id = format!(
            "piece.corridor.{}.segment_{}",
            slugify_label(corridor.id.as_str()),
            index + 1
        );
        requirements.push(PieceRequirement {
            piece_id: segment_id.clone(),
            kind: "corridor".to_owned(),
            role: "corridor_segment".to_owned(),
            source_refs: source_refs.clone(),
            required_exits: vec![
                PieceExitRequirement {
                    id: format!("exit.segment_{}.in", index + 1),
                    direction: opposite_direction(direction.as_str()).to_owned(),
                    width: corridor.width,
                    tags: base_tags.clone(),
                },
                PieceExitRequirement {
                    id: format!("exit.segment_{}.out", index + 1),
                    direction,
                    width: corridor.width,
                    tags: base_tags.clone(),
                },
            ],
            required_sockets: Vec::new(),
            tags: base_tags.clone(),
            placement_hints: vec![format!(
                "segment:{}:{}:{}:{}",
                pair[0].x, pair[0].y, pair[1].x, pair[1].y
            )],
        });
        piece_ids.push(segment_id);
    }

    for (index, triple) in corridor.points.windows(3).enumerate() {
        let Some(in_direction) = direction_between_points(&triple[0], &triple[1]) else {
            continue;
        };
        let Some(out_direction) = direction_between_points(&triple[1], &triple[2]) else {
            continue;
        };
        if in_direction == out_direction {
            continue;
        }
        let bend_id = format!(
            "piece.bend.{}.bend_{}",
            slugify_label(corridor.id.as_str()),
            index + 1
        );
        requirements.push(PieceRequirement {
            piece_id: bend_id.clone(),
            kind: "bend".to_owned(),
            role: "corridor_bend".to_owned(),
            source_refs: source_refs.clone(),
            required_exits: vec![
                PieceExitRequirement {
                    id: format!("exit.bend_{}.in", index + 1),
                    direction: opposite_direction(in_direction.as_str()).to_owned(),
                    width: corridor.width,
                    tags: base_tags.clone(),
                },
                PieceExitRequirement {
                    id: format!("exit.bend_{}.out", index + 1),
                    direction: out_direction,
                    width: corridor.width,
                    tags: base_tags.clone(),
                },
            ],
            required_sockets: Vec::new(),
            tags: base_tags.clone(),
            placement_hints: vec![format!("bend:{}:{}", triple[1].x, triple[1].y)],
        });
        piece_ids.push(bend_id);
    }

    let end_id = format!("piece.connector.{}.end", slugify_label(corridor.id.as_str()));
    requirements.push(PieceRequirement {
        piece_id: end_id.clone(),
        kind: "connector".to_owned(),
        role: "corridor_connector".to_owned(),
        source_refs,
        required_exits: connector_exits(corridor, false),
        required_sockets: Vec::new(),
        tags: base_tags,
        placement_hints: vec!["glue:to_room".to_owned()],
    });
    piece_ids.push(end_id);

    piece_ids
}

fn connector_exits(corridor: &GeometryCorridor, start: bool) -> Vec<PieceExitRequirement> {
    let Some(direction) = corridor_endpoint_direction(corridor, start) else {
        return Vec::new();
    };
    vec![
        PieceExitRequirement {
            id: "exit.room".to_owned(),
            direction: opposite_direction(direction.as_str()).to_owned(),
            width: corridor.width,
            tags: dedupe_strings(corridor.semantic_tags.clone()),
        },
        PieceExitRequirement {
            id: "exit.corridor".to_owned(),
            direction,
            width: corridor.width,
            tags: dedupe_strings(corridor.semantic_tags.clone()),
        },
    ]
}

fn link_piece_chain(
    corridor: &GeometryCorridor,
    from_piece: &str,
    to_piece: &str,
    corridor_piece_ids: &[String],
    links: &mut Vec<PieceLink>,
) {
    if corridor_piece_ids.is_empty() {
        return;
    }
    let mut chain = Vec::with_capacity(corridor_piece_ids.len() + 2);
    chain.push(from_piece.to_owned());
    chain.extend(corridor_piece_ids.iter().cloned());
    chain.push(to_piece.to_owned());
    for (index, pair) in chain.windows(2).enumerate() {
        links.push(PieceLink {
            id: format!(
                "piece_link.{}.{}",
                slugify_label(corridor.id.as_str()),
                index + 1
            ),
            from_piece: pair[0].clone(),
            to_piece: pair[1].clone(),
            source_ref: format!(
                "geometryCorridor:{};connector:{};edge:{}",
                corridor.id, corridor.source_connector, corridor.source_edge
            ),
            traversal_hint: corridor.traversal_hint.clone(),
            tags: dedupe_strings(corridor.semantic_tags.clone()),
        });
    }
}

fn corridor_source_refs(
    corridor: &GeometryCorridor,
    connector: Option<&IntermediateConnector>,
    edge: Option<&Edge>,
) -> Vec<String> {
    let mut refs = vec![
        format!("geometryCorridor:{}", corridor.id),
        format!("connector:{}", corridor.source_connector),
        format!("edge:{}", corridor.source_edge),
        format!("room:{}", corridor.from_room),
        format!("room:{}", corridor.to_room),
    ];
    if let Some(connector) = connector {
        refs.extend(
            connector
                .constraint_refs
                .iter()
                .map(|constraint| format!("constraint:{constraint}")),
        );
    }
    if let Some(edge) = edge {
        refs.push(format!("graphEdge:{}:{}", edge.from, edge.to));
    }
    dedupe_strings(refs)
}

fn corridor_tags(
    corridor: &GeometryCorridor,
    connector: Option<&IntermediateConnector>,
    edge: Option<&Edge>,
) -> Vec<String> {
    let mut tags = vec![
        corridor.traversal_hint.clone(),
        "explicit_corridor_piece".to_owned(),
    ];
    tags.extend(corridor.semantic_tags.clone());
    if let Some(connector) = connector {
        tags.extend(connector.intents.clone());
        tags.extend(connector.affordances.clone());
    }
    if let Some(edge) = edge {
        tags.push(edge.kind.as_str().to_owned());
        tags.push(edge.traversal.as_str().to_owned());
        tags.extend(edge.tags.clone());
        if let Some(required_item) = &edge.required_item {
            tags.push(format!("requires:{required_item}"));
        }
    }
    dedupe_strings(tags)
}

fn corridor_endpoint_direction(corridor: &GeometryCorridor, start: bool) -> Option<String> {
    if corridor.points.len() < 2 {
        return None;
    }
    if start {
        direction_between_points(&corridor.points[0], &corridor.points[1])
    } else {
        let last = corridor.points.len() - 1;
        direction_between_points(&corridor.points[last], &corridor.points[last - 1])
    }
}

fn direction_between_points(from: &GeometryPoint, to: &GeometryPoint) -> Option<String> {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    if dx.abs() >= dy.abs() && dx != 0 {
        Some(if dx > 0 { "east" } else { "west" }.to_owned())
    } else if dy != 0 {
        Some(if dy > 0 { "south" } else { "north" }.to_owned())
    } else {
        None
    }
}

fn opposite_direction(direction: &str) -> &'static str {
    match direction {
        "north" => "south",
        "east" => "west",
        "south" => "north",
        "west" => "east",
        _ => "unknown",
    }
}

fn socket_for_content_kind(kind: &str) -> String {
    match kind {
        "boss_threshold" => "boss_space".to_owned(),
        "hazard" => "hazard_zone".to_owned(),
        "locked_gate" => "gate_line".to_owned(),
        "secret_route_marker" => "secret_marker".to_owned(),
        "start_marker" | "goal_marker" => "marker".to_owned(),
        other => other.to_owned(),
    }
}
