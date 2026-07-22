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
        let corridor_pieces =
            emit_corridor_piece_requirements(corridor, connector, edge, &mut requirements);
        link_piece_chain(
            corridor,
            edge,
            &from_piece,
            &to_piece,
            &corridor_pieces,
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

#[derive(Clone, Debug)]
struct CorridorChainPiece {
    piece_id: String,
    inbound_exit: String,
    outbound_exit: String,
}

fn emit_corridor_piece_requirements(
    corridor: &GeometryCorridor,
    connector: Option<&IntermediateConnector>,
    edge: Option<&Edge>,
    requirements: &mut Vec<PieceRequirement>,
) -> Vec<CorridorChainPiece> {
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
        placement_hints: vec![
            "glue:from_room".to_owned(),
            format!("point:{}:{}", corridor.points[0].x, corridor.points[0].y),
        ],
    });
    piece_ids.push(CorridorChainPiece {
        piece_id: start_id,
        inbound_exit: "exit.room".to_owned(),
        outbound_exit: "exit.corridor".to_owned(),
    });

    let start_direction = corridor_endpoint_direction(corridor, true);
    let end_direction = corridor_endpoint_direction(corridor, false);
    if let (Some(start_direction), Some(end_direction)) = (start_direction, end_direction) {
        let inbound = opposite_direction(start_direction.as_str()).to_owned();
        let outbound = opposite_direction(end_direction.as_str()).to_owned();
        if opposite_direction(start_direction.as_str()) == end_direction {
            push_corridor_bridge_piece(
                corridor,
                "corridor",
                1,
                inbound,
                outbound,
                &source_refs,
                &base_tags,
                requirements,
                &mut piece_ids,
            );
        } else if start_direction != end_direction {
            push_corridor_bridge_piece(
                corridor,
                "bend",
                1,
                inbound,
                outbound,
                &source_refs,
                &base_tags,
                requirements,
                &mut piece_ids,
            );
        } else {
            let turn = match start_direction.as_str() {
                "north" | "south" => "east",
                _ => "north",
            };
            push_corridor_bridge_piece(
                corridor,
                "bend",
                1,
                inbound,
                turn.to_owned(),
                &source_refs,
                &base_tags,
                requirements,
                &mut piece_ids,
            );
            push_corridor_bridge_piece(
                corridor,
                "bend",
                2,
                opposite_direction(turn).to_owned(),
                outbound,
                &source_refs,
                &base_tags,
                requirements,
                &mut piece_ids,
            );
        }
    }

    let end_id = format!("piece.connector.{}.end", slugify_label(corridor.id.as_str()));
    let end_point = corridor.points.last().unwrap_or(&corridor.points[0]);
    requirements.push(PieceRequirement {
        piece_id: end_id.clone(),
        kind: "connector".to_owned(),
        role: "corridor_connector".to_owned(),
        source_refs,
        required_exits: connector_exits(corridor, false),
        required_sockets: Vec::new(),
        tags: base_tags,
        placement_hints: vec![
            "glue:to_room".to_owned(),
            format!("point:{}:{}", end_point.x, end_point.y),
        ],
    });
    piece_ids.push(CorridorChainPiece {
        piece_id: end_id,
        inbound_exit: "exit.corridor".to_owned(),
        outbound_exit: "exit.room".to_owned(),
    });

    piece_ids
}

#[allow(clippy::too_many_arguments)]
fn push_corridor_bridge_piece(
    corridor: &GeometryCorridor,
    kind: &str,
    index: usize,
    inbound_direction: String,
    outbound_direction: String,
    source_refs: &[String],
    base_tags: &[String],
    requirements: &mut Vec<PieceRequirement>,
    piece_ids: &mut Vec<CorridorChainPiece>,
) {
    let piece_id = format!(
        "piece.{}.{}.bridge_{}",
        kind,
        slugify_label(corridor.id.as_str()),
        index
    );
    let inbound_exit = format!("exit.bridge_{}.in", index);
    let outbound_exit = format!("exit.bridge_{}.out", index);
    let first = &corridor.points[0];
    let last = corridor.points.last().unwrap_or(first);
    requirements.push(PieceRequirement {
        piece_id: piece_id.clone(),
        kind: kind.to_owned(),
        role: if kind == "bend" {
            "corridor_bend".to_owned()
        } else {
            "corridor_segment".to_owned()
        },
        source_refs: source_refs.to_vec(),
        required_exits: vec![
            PieceExitRequirement {
                id: inbound_exit.clone(),
                direction: inbound_direction,
                width: corridor.width,
                tags: base_tags.to_vec(),
            },
            PieceExitRequirement {
                id: outbound_exit.clone(),
                direction: outbound_direction,
                width: corridor.width,
                tags: base_tags.to_vec(),
            },
        ],
        required_sockets: Vec::new(),
        tags: base_tags.to_vec(),
        placement_hints: vec![format!(
            "segment:{}:{}:{}:{}",
            first.x, first.y, last.x, last.y
        )],
    });
    piece_ids.push(CorridorChainPiece {
        piece_id,
        inbound_exit,
        outbound_exit,
    });
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
    edge: Option<&Edge>,
    from_piece: &str,
    to_piece: &str,
    corridor_pieces: &[CorridorChainPiece],
    links: &mut Vec<PieceLink>,
) {
    if corridor_pieces.is_empty() {
        return;
    }
    let from_room_exit = room_corridor_exit_id(corridor, true);
    let to_room_exit = room_corridor_exit_id(corridor, false);
    let mut chain = Vec::with_capacity(corridor_pieces.len() + 2);
    chain.push((from_piece.to_owned(), from_room_exit, None));
    chain.extend(corridor_pieces.iter().map(|piece| {
        (
            piece.piece_id.clone(),
            piece.outbound_exit.clone(),
            Some(piece.inbound_exit.clone()),
        )
    }));
    chain.push((to_piece.to_owned(), String::new(), Some(to_room_exit)));
    for (index, pair) in chain.windows(2).enumerate() {
        links.push(PieceLink {
            id: format!(
                "piece_link.{}.{}",
                slugify_label(corridor.id.as_str()),
                index + 1
            ),
            from_piece: pair[0].0.clone(),
            from_exit: pair[0].1.clone(),
            to_piece: pair[1].0.clone(),
            to_exit: pair[1].2.clone().unwrap_or_default(),
            source_section: corridor.physical_section.clone(),
            source_corridor: corridor.id.clone(),
            source_edge: corridor.source_edge.clone(),
            source_edges: corridor.source_edges.clone(),
            traversal_refs: corridor.traversal_refs.clone(),
            source_ref: format!(
                "physicalSection:{};geometryCorridor:{};connector:{};edge:{}",
                corridor.physical_section, corridor.id, corridor.source_connector, corridor.source_edge
            ),
            traversal: edge
                .map(|source_edge| source_edge.traversal.as_str().to_owned())
                .unwrap_or_else(|| corridor.traversal_hint.clone()),
            required_item: edge.and_then(|source_edge| source_edge.required_item.clone()),
            tags: dedupe_strings(corridor.semantic_tags.clone()),
        });
    }
}

fn room_corridor_exit_id(corridor: &GeometryCorridor, start: bool) -> String {
    let direction = corridor_endpoint_direction(corridor, start)
        .unwrap_or_else(|| "unknown".to_owned());
    format!("exit.{}.{}", slugify_label(corridor.id.as_str()), direction)
}

fn corridor_source_refs(
    corridor: &GeometryCorridor,
    connector: Option<&IntermediateConnector>,
    edge: Option<&Edge>,
) -> Vec<String> {
    let mut refs = vec![
        format!("physicalSection:{}", corridor.physical_section),
        format!("geometryCorridor:{}", corridor.id),
        format!("room:{}", corridor.from_room),
        format!("room:{}", corridor.to_room),
    ];
    refs.extend(corridor.source_connectors.iter().map(|value| format!("connector:{value}")));
    refs.extend(corridor.source_edges.iter().map(|value| format!("edge:{value}")));
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
