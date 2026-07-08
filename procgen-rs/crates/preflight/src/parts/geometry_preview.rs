fn geometry_emit_2d_command(args: GeometryEmit2dArgs) -> Result<(), String> {
    let candidate: Candidate = read_json(&args.candidate)?;
    let intermediate: IntermediateBreakdown = read_json(&args.intermediate)?;
    let geometry = emit_geometry_2d(&candidate, &intermediate, &args, args.seed)?;
    write_json(&args.out, &geometry)
}

fn geometry_validate_2d_command(args: ReportOutArgs) -> Result<(), String> {
    let geometry: Geometry2dArtifact = read_json(&args.state)?;
    let report = validate_geometry_2d(&geometry);
    write_json(&args.out, &report)?;
    if report.ok {
        Ok(())
    } else {
        Err(format!(
            "2D geometry validation failed with {} fatal diagnostic(s); see {}",
            report.fatal_count,
            args.out.display()
        ))
    }
}

fn preview_html_command(args: PreviewHtmlArgs) -> Result<(), String> {
    let geometry: Geometry2dArtifact = read_json(&args.geometry)?;
    let validation: ValidationReport = read_json(&args.validation)?;
    validate_preview_inputs(&geometry, &validation, args.allow_invalid)?;
    let html = render_geometry_preview_html(
        &geometry,
        &validation,
        &display_path(&args.geometry),
        &display_path(&args.validation),
    );
    write_text(&args.out, &html)
}

fn validate_preview_inputs(
    geometry: &Geometry2dArtifact,
    validation: &ValidationReport,
    allow_invalid: bool,
) -> Result<(), String> {
    if validation.kind != "asha_procgen.validation.geometry_2d.v1" {
        return Err(format!(
            "preview html requires geometry validation kind asha_procgen.validation.geometry_2d.v1, got {}",
            validation.kind
        ));
    }
    let geometry_hash = hash_json(geometry)?;
    if validation.state_hash != geometry_hash {
        return Err("preview html validation hash does not match geometry input".to_owned());
    }
    if !validation.ok && !allow_invalid {
        return Err(format!(
            "preview html refused invalid geometry with {} fatal diagnostic(s); pass --allow-invalid to render diagnostics",
            validation.fatal_count
        ));
    }
    Ok(())
}

fn render_geometry_preview_html(
    geometry: &Geometry2dArtifact,
    validation: &ValidationReport,
    geometry_ref: &str,
    validation_ref: &str,
) -> String {
    let svg_width = geometry.bounds.width.max(320);
    let svg_height = geometry.bounds.height.max(240);
    let mut corridors = String::new();
    for corridor in &geometry.corridors {
        let points = corridor
            .points
            .iter()
            .map(|point| format!("{},{}", point.x, point.y))
            .collect::<Vec<_>>()
            .join(" ");
        corridors.push_str(&format!(
            r#"<polyline class="corridor corridor-{}" data-source-edge="{}" points="{}" stroke-width="{}" />
"#,
            css_token(&corridor.traversal_hint),
            escape_attr(&corridor.source_edge),
            escape_attr(&points),
            corridor.width.max(2)
        ));
    }

    let mut rooms = String::new();
    for room in &geometry.rooms {
        let fill = room_fill(room);
        rooms.push_str(&format!(
            r#"<g class="room room-{}" data-room-id="{}" data-role="{}">
  <rect x="{}" y="{}" width="{}" height="{}" rx="6" fill="{}" />
  <text class="room-label" x="{}" y="{}">{}</text>
"#,
            css_token(&room.role),
            escape_attr(&room.id),
            escape_attr(&room.role),
            room.rect.x,
            room.rect.y,
            room.rect.width,
            room.rect.height,
            fill,
            room.rect.x + 10,
            room.rect.y + 20,
            escape_html(&room_label(room))
        ));
        for (index, content) in geometry
            .contents
            .iter()
            .filter(|content| content.room_id == room.id)
            .enumerate()
        {
            rooms.push_str(&format!(
                r#"  <text class="content-label content-{}" x="{}" y="{}">{}</text>
"#,
                css_token(&content.kind),
                room.rect.x + 10,
                room.rect.y + 38 + index as i32 * 14,
                escape_html(&content.label)
            ));
        }
        rooms.push_str("</g>\n");
    }

    let diagnostics = if validation.diagnostics.is_empty() {
        "<li>No diagnostics.</li>".to_owned()
    } else {
        validation
            .diagnostics
            .iter()
            .map(|diagnostic| {
                format!(
                    "<li><strong>{}</strong> [{}] {}</li>",
                    escape_html(&diagnostic.code),
                    escape_html(severity_label(diagnostic.severity)),
                    escape_html(&diagnostic.detail)
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let legend_items = [
        ("Start/Goal", "#5fb3ff"),
        ("Gate/Boss", "#f0b35f"),
        ("Hazard", "#ff6b6b"),
        ("Reward/Key", "#7bd88f"),
        ("Secret/Shortcut", "#c792ea"),
        ("Standard", "#94a3b8"),
    ]
    .iter()
    .map(|(label, color)| {
        format!(
            r#"<li><span style="background:{}"></span>{}</li>"#,
            color,
            escape_html(label)
        )
    })
    .collect::<Vec<_>>()
    .join("\n");

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Asha Procgen Dungeon Preview</title>
<style>
:root {{ color-scheme: dark; font-family: Inter, ui-sans-serif, system-ui, sans-serif; background: #0b0d10; color: #e8edf2; }}
body {{ margin: 0; background: #0b0d10; }}
main {{ min-height: 100vh; display: grid; grid-template-columns: minmax(0, 1fr) 320px; gap: 0; }}
.stage {{ overflow: auto; padding: 24px; background: #0f1318; }}
.panel {{ border-left: 1px solid #2b3440; padding: 20px; background: #121820; }}
h1 {{ margin: 0 0 12px; font-size: 20px; }}
h2 {{ margin: 20px 0 8px; font-size: 14px; color: #b7c4d3; text-transform: uppercase; }}
p, li {{ color: #c7d1dc; font-size: 13px; line-height: 1.45; }}
code {{ color: #f8d67a; overflow-wrap: anywhere; }}
svg {{ display: block; min-width: {}px; min-height: {}px; background: #151b22; border: 1px solid #2d3743; }}
.corridor {{ fill: none; stroke: #6e7f93; stroke-linecap: round; stroke-linejoin: round; opacity: 0.82; }}
.corridor-locked {{ stroke: #f0b35f; }}
.corridor-hidden {{ stroke: #c792ea; stroke-dasharray: 10 8; }}
.corridor-one-way-return {{ stroke: #5fb3ff; stroke-dasharray: 16 6; }}
.room rect {{ stroke: #d3deea; stroke-width: 1.5; }}
.room-label {{ fill: #f4f8fb; font-size: 13px; font-weight: 700; }}
.content-label {{ fill: #d6e0eb; font-size: 11px; }}
.legend {{ list-style: none; padding: 0; margin: 0; }}
.legend li {{ display: flex; align-items: center; gap: 8px; margin: 6px 0; }}
.legend span {{ display: inline-block; width: 12px; height: 12px; border-radius: 2px; }}
.status-ok {{ color: #7bd88f; }}
.status-bad {{ color: #ff6b6b; }}
@media (max-width: 900px) {{ main {{ grid-template-columns: 1fr; }} .panel {{ border-left: 0; border-top: 1px solid #2b3440; }} }}
</style>
</head>
<body data-preview-kind="asha_procgen.html_preview.v1" data-kind="{}">
<main>
<section class="stage" aria-label="Dungeon floor plan">
<svg xmlns="http://www.w3.org/2000/svg" role="img" aria-labelledby="preview-title" viewBox="0 0 {} {}">
<title id="preview-title">Generated dungeon preview for {}</title>
<g class="corridors">
{}</g>
<g class="rooms">
{}</g>
</svg>
</section>
<aside class="panel">
<h1>Dungeon Preview</h1>
<p class="{}">Validation: {}</p>
<p>Geometry: <code>{}</code></p>
<p>Validation: <code>{}</code></p>
<p>Rooms: {} · Corridors: {} · Contents: {}</p>
<h2>Legend</h2>
<ul class="legend">
{}
</ul>
<h2>Diagnostics</h2>
<ul>
{}
</ul>
</aside>
</main>
</body>
</html>
"#,
        svg_width,
        svg_height,
        escape_attr(&geometry.kind),
        svg_width,
        svg_height,
        escape_html(&geometry.geometry_id),
        corridors,
        rooms,
        if validation.ok {
            "status-ok"
        } else {
            "status-bad"
        },
        if validation.ok { "ok" } else { "invalid" },
        escape_html(geometry_ref),
        escape_html(validation_ref),
        geometry.rooms.len(),
        geometry.corridors.len(),
        geometry.contents.len(),
        legend_items,
        diagnostics
    )
}

fn room_label(room: &GeometryRoom) -> String {
    if room.role == room.geometry_role {
        room.role.clone()
    } else {
        format!("{} / {}", room.role, room.geometry_role)
    }
}

fn room_fill(room: &GeometryRoom) -> &'static str {
    match room.role.as_str() {
        "start" | "goal" => "#1f5f89",
        "gate" | "boss_gate" => "#725124",
        "pressure" => "#733238",
        "reward" => "#245a38",
        "landmark_hub" => "#394762",
        _ if room.geometry_role.contains("secret") || room.geometry_role.contains("shortcut") => {
            "#563d72"
        }
        _ => "#2d3a47",
    }
}

fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Info => "info",
        Severity::Warning => "warning",
        Severity::Fatal => "fatal",
    }
}

fn css_token(value: &str) -> String {
    let token = slugify_label(value).replace('_', "-");
    if token.is_empty() {
        "unknown".to_owned()
    } else {
        token
    }
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attr(value: &str) -> String {
    escape_html(value).replace('"', "&quot;")
}

fn validate_geometry_2d(geometry: &Geometry2dArtifact) -> ValidationReport {
    let mut diagnostics = Vec::new();
    if geometry.kind != "asha_procgen.geometry_2d.v1" {
        diagnostics.push(fatal(
            "geometry_kind_invalid",
            None,
            None,
            "Geometry artifact kind must be asha_procgen.geometry_2d.v1.",
        ));
    }
    if geometry.bounds.width <= 0 || geometry.bounds.height <= 0 || geometry.bounds.grid <= 0 {
        diagnostics.push(fatal(
            "geometry_bounds_invalid",
            None,
            None,
            "Geometry bounds width, height, and grid must be positive.",
        ));
    }

    let mut rooms_by_id = BTreeMap::new();
    for room in &geometry.rooms {
        if room.id.is_empty() {
            diagnostics.push(fatal(
                "geometry_room_id_missing",
                room.source_nodes.first().map(String::as_str),
                None,
                "Room id must not be empty.",
            ));
            continue;
        }
        if rooms_by_id.insert(room.id.as_str(), room).is_some() {
            diagnostics.push(fatal(
                "geometry_room_duplicate",
                room.source_nodes.first().map(String::as_str),
                None,
                format!("Room id {} appears more than once.", room.id),
            ));
        }
        if room.rect.width <= 0 || room.rect.height <= 0 {
            diagnostics.push(fatal(
                "geometry_room_rect_invalid",
                room.source_nodes.first().map(String::as_str),
                None,
                format!("Room {} has a non-positive rectangle.", room.id),
            ));
        }
        if room.rect.x < 0
            || room.rect.y < 0
            || room.rect.x + room.rect.width > geometry.bounds.width
            || room.rect.y + room.rect.height > geometry.bounds.height
        {
            diagnostics.push(fatal(
                "geometry_room_out_of_bounds",
                room.source_nodes.first().map(String::as_str),
                None,
                format!("Room {} extends outside geometry bounds.", room.id),
            ));
        }
    }
    for (index, left) in geometry.rooms.iter().enumerate() {
        for right in geometry.rooms.iter().skip(index + 1) {
            if geometry_rectangles_overlap(&left.rect, &right.rect) {
                diagnostics.push(fatal(
                    "geometry_room_overlap",
                    left.source_nodes.first().map(String::as_str),
                    None,
                    format!("Room {} overlaps {}.", left.id, right.id),
                ));
            }
        }
    }

    let mut represented_connectors = BTreeSet::new();
    let mut adjacency: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    if geometry.rooms.len() > 1
        && geometry.corridors.is_empty()
        && geometry.skipped_connectors.is_empty()
    {
        diagnostics.push(fatal(
            "geometry_connector_coverage_missing",
            None,
            None,
            "Multi-room geometry must include routed corridors or explicit skipped connectors.",
        ));
    }
    for corridor in &geometry.corridors {
        if corridor.source_connector.is_empty() || corridor.source_edge.is_empty() {
            diagnostics.push(fatal(
                "geometry_corridor_source_missing",
                None,
                Some(corridor.id.as_str()),
                "Corridor must preserve source connector and source edge refs.",
            ));
        } else if !represented_connectors.insert(corridor.source_connector.as_str()) {
            diagnostics.push(fatal(
                "geometry_corridor_duplicate_connector",
                None,
                Some(corridor.id.as_str()),
                format!(
                    "Connector {} is represented by more than one corridor.",
                    corridor.source_connector
                ),
            ));
        }
        let from_room = rooms_by_id.get(corridor.from_room.as_str()).copied();
        let to_room = rooms_by_id.get(corridor.to_room.as_str()).copied();
        if from_room.is_none() || to_room.is_none() {
            diagnostics.push(fatal(
                "geometry_corridor_room_missing",
                None,
                Some(corridor.id.as_str()),
                format!(
                    "Corridor {} references a missing room endpoint.",
                    corridor.id
                ),
            ));
        }
        if corridor.points.len() < 2 {
            diagnostics.push(fatal(
                "geometry_corridor_points_missing",
                None,
                Some(corridor.id.as_str()),
                format!("Corridor {} must have at least two points.", corridor.id),
            ));
        }
        if let (Some(from_room), Some(to_room), Some(first), Some(last)) = (
            from_room,
            to_room,
            corridor.points.first(),
            corridor.points.last(),
        ) {
            if !geometry_point_on_rect_boundary(first, &from_room.rect)
                || !geometry_point_on_rect_boundary(last, &to_room.rect)
            {
                diagnostics.push(fatal(
                    "geometry_corridor_endpoint_detached",
                    None,
                    Some(corridor.id.as_str()),
                    format!(
                        "Corridor {} endpoints must attach to source and target room bounds.",
                        corridor.id
                    ),
                ));
            }
            adjacency
                .entry(corridor.from_room.as_str())
                .or_default()
                .push(corridor.to_room.as_str());
        }
        if corridor.traversal_hint == "locked"
            && !corridor
                .semantic_tags
                .iter()
                .any(|tag| tag == "locked_threshold")
        {
            diagnostics.push(fatal(
                "geometry_locked_semantics_missing",
                None,
                Some(corridor.id.as_str()),
                "Locked corridors must preserve locked_threshold semantics.",
            ));
        }
        if corridor.traversal_hint == "hidden"
            && !corridor
                .semantic_tags
                .iter()
                .any(|tag| tag == "hidden_route" || tag == "hidden_passage")
        {
            diagnostics.push(fatal(
                "geometry_hidden_semantics_missing",
                None,
                Some(corridor.id.as_str()),
                "Hidden corridors must preserve hidden route semantics.",
            ));
        }
        if corridor
            .semantic_tags
            .iter()
            .any(|tag| tag == "shortcut_link")
            && corridor.source_edge.is_empty()
        {
            diagnostics.push(fatal(
                "geometry_shortcut_source_missing",
                None,
                Some(corridor.id.as_str()),
                "Shortcut corridors must preserve source edge refs.",
            ));
        }
    }

    let mut skipped_connectors = BTreeSet::new();
    for skipped in &geometry.skipped_connectors {
        if skipped.source_connector.is_empty() || skipped.reason.is_empty() {
            diagnostics.push(fatal(
                "geometry_skipped_connector_invalid",
                None,
                None,
                "Skipped connectors must include source connector and reason.",
            ));
        } else if !skipped_connectors.insert(skipped.source_connector.as_str()) {
            diagnostics.push(fatal(
                "geometry_skipped_connector_duplicate",
                None,
                Some(skipped.source_connector.as_str()),
                format!(
                    "Skipped connector {} appears more than once.",
                    skipped.source_connector
                ),
            ));
        }
        if represented_connectors.contains(skipped.source_connector.as_str()) {
            diagnostics.push(fatal(
                "geometry_connector_represented_and_skipped",
                None,
                Some(skipped.source_connector.as_str()),
                format!(
                    "Connector {} is both routed and skipped.",
                    skipped.source_connector
                ),
            ));
        }
    }

    validate_geometry_content_anchors(geometry, &rooms_by_id, &mut diagnostics);
    validate_geometry_reachability(geometry, &adjacency, &mut diagnostics);

    let fatal_count = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Fatal)
        .count();
    ValidationReport {
        kind: "asha_procgen.validation.geometry_2d.v1".to_owned(),
        schema_version: 1,
        state_hash: hash_json(geometry).unwrap_or_else(|_| "hash_error".to_owned()),
        ok: fatal_count == 0,
        fatal_count,
        diagnostics,
    }
}

fn validate_geometry_content_anchors(
    geometry: &Geometry2dArtifact,
    rooms_by_id: &BTreeMap<&str, &GeometryRoom>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut content_ids = BTreeSet::new();
    for content in &geometry.contents {
        if content.id.is_empty()
            || content.kind.is_empty()
            || content.label.is_empty()
            || content.source_ref.is_empty()
        {
            diagnostics.push(fatal(
                "geometry_content_metadata_missing",
                None,
                None,
                "Content annotations must include id, kind, label, and source ref.",
            ));
        } else if !content_ids.insert(content.id.as_str()) {
            diagnostics.push(fatal(
                "geometry_content_duplicate",
                None,
                None,
                format!("Content id {} appears more than once.", content.id),
            ));
        }
        if !rooms_by_id.contains_key(content.room_id.as_str()) {
            diagnostics.push(fatal(
                "geometry_content_room_missing",
                None,
                None,
                format!(
                    "Content {} references missing room {}.",
                    content.id, content.room_id
                ),
            ));
        }
    }
}

fn validate_geometry_reachability(
    geometry: &Geometry2dArtifact,
    adjacency: &BTreeMap<&str, Vec<&str>>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let start_rooms = geometry
        .rooms
        .iter()
        .filter(|room| room.role == "start")
        .map(|room| room.id.as_str())
        .collect::<Vec<_>>();
    let goal_rooms = geometry
        .rooms
        .iter()
        .filter(|room| room.role == "goal")
        .map(|room| room.id.as_str())
        .collect::<BTreeSet<_>>();
    if start_rooms.is_empty() {
        diagnostics.push(fatal(
            "geometry_start_missing",
            Some("start"),
            None,
            "Geometry must include a start room.",
        ));
    }
    if goal_rooms.is_empty() {
        diagnostics.push(fatal(
            "geometry_goal_missing",
            Some("goal"),
            None,
            "Geometry must include a goal room.",
        ));
    }
    if start_rooms.is_empty() || goal_rooms.is_empty() {
        return;
    }

    let mut visited = BTreeSet::new();
    let mut queue = VecDeque::new();
    for start_room in start_rooms {
        visited.insert(start_room);
        queue.push_back(start_room);
    }
    while let Some(room_id) = queue.pop_front() {
        if let Some(next_rooms) = adjacency.get(room_id) {
            for next_room in next_rooms {
                if visited.insert(*next_room) {
                    queue.push_back(*next_room);
                }
            }
        }
    }
    if !goal_rooms.iter().any(|goal| visited.contains(goal)) {
        diagnostics.push(fatal(
            "geometry_goal_unreachable",
            Some("goal"),
            None,
            "Goal room is not reachable from start through directed corridors.",
        ));
    }
}

fn geometry_rectangles_overlap(left: &GeometryRect, right: &GeometryRect) -> bool {
    left.x < right.x + right.width
        && left.x + left.width > right.x
        && left.y < right.y + right.height
        && left.y + left.height > right.y
}

fn geometry_point_on_rect_boundary(point: &GeometryPoint, rect: &GeometryRect) -> bool {
    let on_vertical = (point.x == rect.x || point.x == rect.x + rect.width)
        && point.y >= rect.y
        && point.y <= rect.y + rect.height;
    let on_horizontal = (point.y == rect.y || point.y == rect.y + rect.height)
        && point.x >= rect.x
        && point.x <= rect.x + rect.width;
    on_vertical || on_horizontal
}

fn emit_geometry_2d(
    candidate: &Candidate,
    intermediate: &IntermediateBreakdown,
    args: &GeometryEmit2dArgs,
    seed: u64,
) -> Result<Geometry2dArtifact, String> {
    if intermediate.candidate_id != candidate.candidate_id {
        return Err(format!(
            "intermediate candidate {} does not match candidate {}",
            intermediate.candidate_id, candidate.candidate_id
        ));
    }
    let depths = graph_depths(candidate);
    let mut region_specs = intermediate
        .regions
        .iter()
        .map(|region| {
            let depth = region
                .node_ids
                .iter()
                .filter_map(|node_id| depths.get(node_id.as_str()).copied())
                .min()
                .unwrap_or(0);
            (depth, region.role.clone(), region.id.clone(), region)
        })
        .collect::<Vec<_>>();
    region_specs.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| left.2.cmp(&right.2))
    });

    let mut rows_by_depth: BTreeMap<usize, usize> = BTreeMap::new();
    let mut rooms = Vec::new();
    let grid = 8;
    for (depth, _role, _id, region) in region_specs {
        let row = rows_by_depth.entry(depth).or_insert(0);
        let (width, height) = room_size_for_region(region);
        let x = 64 + depth as i32 * 260;
        let y = 64 + *row as i32 * 156;
        *row += 1;
        rooms.push(GeometryRoom {
            id: room_id(region.id.as_str()),
            source_region: region.id.clone(),
            source_nodes: region.node_ids.clone(),
            role: region.role.clone(),
            geometry_role: region.geometry_role.clone(),
            footprint_class: region.footprint_class.clone(),
            rect: GeometryRect {
                x,
                y,
                width,
                height,
            },
            style_tags: geometry_room_style_tags(region),
        });
    }
    let bounds = geometry_bounds(&rooms, grid);
    let room_by_region = rooms
        .iter()
        .map(|room| (room.source_region.as_str(), room))
        .collect::<BTreeMap<_, _>>();
    let mut corridors = Vec::new();
    let mut skipped_connectors = Vec::new();
    for connector in &intermediate.connectors {
        let Some(from_room) = room_by_region.get(connector.from_region.as_str()).copied() else {
            skipped_connectors.push(SkippedConnector {
                source_connector: connector.id.clone(),
                reason: "missing_from_room".to_owned(),
            });
            continue;
        };
        let Some(to_room) = room_by_region.get(connector.to_region.as_str()).copied() else {
            skipped_connectors.push(SkippedConnector {
                source_connector: connector.id.clone(),
                reason: "missing_to_room".to_owned(),
            });
            continue;
        };
        corridors.push(route_corridor(connector, from_room, to_room));
    }
    let contents = geometry_contents(candidate, intermediate, &rooms);
    Ok(Geometry2dArtifact {
        kind: "asha_procgen.geometry_2d.v1".to_owned(),
        schema_version: 1,
        geometry_id: format!("geometry.{}.{}", candidate.candidate_id, seed),
        candidate_id: candidate.candidate_id.clone(),
        seed,
        source_candidate_ref: display_path(&args.candidate),
        source_intermediate_ref: display_path(&args.intermediate),
        bounds,
        rooms,
        corridors,
        contents,
        skipped_connectors,
    })
}

fn geometry_contents(
    candidate: &Candidate,
    intermediate: &IntermediateBreakdown,
    rooms: &[GeometryRoom],
) -> Vec<GeometryContent> {
    let nodes_by_id = candidate
        .graph
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let regions_by_id = intermediate
        .regions
        .iter()
        .map(|region| (region.id.as_str(), region))
        .collect::<BTreeMap<_, _>>();
    let mut contents = Vec::new();
    for room in rooms {
        let region = regions_by_id.get(room.source_region.as_str()).copied();
        for node_id in &room.source_nodes {
            let Some(node) = nodes_by_id.get(node_id.as_str()).copied() else {
                continue;
            };
            let Some((kind, label, tags)) = content_annotation_for_node(node, room, region) else {
                continue;
            };
            contents.push(GeometryContent {
                id: format!(
                    "content.{}.{}",
                    slugify_label(room.id.as_str()),
                    slugify_label(kind.as_str())
                ),
                room_id: room.id.clone(),
                source_ref: format!("node:{};region:{}", node.id, room.source_region),
                kind,
                label,
                tags,
            });
        }
    }
    contents
}

fn content_annotation_for_node(
    node: &Node,
    room: &GeometryRoom,
    region: Option<&IntermediateRegion>,
) -> Option<(String, String, Vec<String>)> {
    let (kind, label) = if node.kind == NodeKind::Start {
        ("start_marker", "Start")
    } else if node.kind == NodeKind::Goal {
        ("goal_marker", "Goal")
    } else if node_has_tag(node, "boss") {
        ("boss_threshold", "Boss Threshold")
    } else if node_has_tag(node, "hazard") || node.kind == NodeKind::Hazard {
        ("hazard", "Hazard")
    } else if node_has_tag(node, "reward") || node.kind == NodeKind::Treasure {
        ("reward_cache", "Reward Cache")
    } else if node.kind == NodeKind::Key {
        ("key_pickup", "Key Pickup")
    } else if node.kind == NodeKind::Gate || node_has_tag(node, "lock") {
        ("locked_gate", "Locked Gate")
    } else if node.kind == NodeKind::Shortcut {
        ("shortcut_marker", "Shortcut Marker")
    } else if node.kind == NodeKind::Secret {
        ("secret_route_marker", "Secret Route")
    } else if node.kind == NodeKind::Resource {
        ("resource_clue", "Resource Clue")
    } else {
        return None;
    };
    let mut tags = vec![
        kind.to_owned(),
        node.kind.as_str().to_owned(),
        room.role.clone(),
        room.geometry_role.clone(),
    ];
    tags.extend(node.tags.clone());
    tags.extend(room.style_tags.clone());
    if let Some(region) = region {
        tags.extend(region.entrance_expectations.clone());
    }
    Some((kind.to_owned(), label.to_owned(), dedupe_strings(tags)))
}

fn route_corridor(
    connector: &IntermediateConnector,
    from_room: &GeometryRoom,
    to_room: &GeometryRoom,
) -> GeometryCorridor {
    let start = corridor_anchor(&from_room.rect, &to_room.rect);
    let end = corridor_anchor(&to_room.rect, &from_room.rect);
    let mid_x = (start.x + end.x) / 2;
    let points = dedupe_points(vec![
        start.clone(),
        GeometryPoint {
            x: mid_x,
            y: start.y,
        },
        GeometryPoint { x: mid_x, y: end.y },
        end,
    ]);
    GeometryCorridor {
        id: format!("corridor.{}", slugify_label(connector.id.as_str())),
        source_connector: connector.id.clone(),
        source_edge: connector.edge_id.clone(),
        from_room: from_room.id.clone(),
        to_room: to_room.id.clone(),
        traversal_hint: connector.traversal_hint.clone(),
        semantic_tags: corridor_semantic_tags(connector),
        width: corridor_width(connector),
        points,
    }
}

fn corridor_anchor(from: &GeometryRect, to: &GeometryRect) -> GeometryPoint {
    let from_center = rect_center(from);
    let to_center = rect_center(to);
    let dx = to_center.x - from_center.x;
    let dy = to_center.y - from_center.y;
    if dx.abs() >= dy.abs() {
        GeometryPoint {
            x: if dx >= 0 { from.x + from.width } else { from.x },
            y: from_center.y,
        }
    } else {
        GeometryPoint {
            x: from_center.x,
            y: if dy >= 0 {
                from.y + from.height
            } else {
                from.y
            },
        }
    }
}

fn rect_center(rect: &GeometryRect) -> GeometryPoint {
    GeometryPoint {
        x: rect.x + rect.width / 2,
        y: rect.y + rect.height / 2,
    }
}

fn dedupe_points(points: Vec<GeometryPoint>) -> Vec<GeometryPoint> {
    let mut deduped = Vec::new();
    for point in points {
        if !deduped
            .last()
            .is_some_and(|last: &GeometryPoint| last.x == point.x && last.y == point.y)
        {
            deduped.push(point);
        }
    }
    deduped
}

fn corridor_width(connector: &IntermediateConnector) -> i32 {
    if connector
        .affordances
        .iter()
        .any(|affordance| affordance == "locked_threshold")
    {
        18
    } else if connector
        .affordances
        .iter()
        .any(|affordance| affordance == "pressure_route")
    {
        20
    } else if connector
        .affordances
        .iter()
        .any(|affordance| affordance == "hidden_passage")
    {
        10
    } else if connector
        .affordances
        .iter()
        .any(|affordance| affordance == "shortcut_link")
    {
        14
    } else {
        12
    }
}

fn corridor_semantic_tags(connector: &IntermediateConnector) -> Vec<String> {
    let mut tags = vec![connector.traversal_hint.clone()];
    tags.extend(connector.intents.clone());
    tags.extend(connector.affordances.clone());
    dedupe_strings(tags)
}

fn room_size_for_region(region: &IntermediateRegion) -> (i32, i32) {
    match (region.scale_band.as_str(), region.footprint_class.as_str()) {
        ("large", "hub") => (152, 112),
        ("large", _) => (144, 104),
        ("medium", "pressure_lane") => (136, 80),
        ("medium", "threshold") | ("medium", "threshold_large") => (112, 80),
        ("medium", _) => (120, 88),
        ("small", "small_pocket") | ("small", "pocket") => (88, 72),
        ("small", "small_marker") => (80, 64),
        ("small", _) => (96, 72),
        _ => match region.role.as_str() {
            "landmark_hub" => (152, 112),
            "reward" => (88, 72),
            "pressure" => (136, 80),
            "gate" | "boss_gate" => (112, 80),
            _ => (120, 88),
        },
    }
}

fn geometry_room_style_tags(region: &IntermediateRegion) -> Vec<String> {
    let mut tags = vec![
        region.role.clone(),
        region.geometry_role.clone(),
        region.scale_band.clone(),
    ];
    tags.extend(region.entrance_expectations.clone());
    dedupe_strings(tags)
}

fn geometry_bounds(rooms: &[GeometryRoom], grid: i32) -> GeometryBounds {
    let width = rooms
        .iter()
        .map(|room| room.rect.x + room.rect.width)
        .max()
        .unwrap_or(0)
        + 96;
    let height = rooms
        .iter()
        .map(|room| room.rect.y + room.rect.height)
        .max()
        .unwrap_or(0)
        + 96;
    GeometryBounds {
        width: width.max(640),
        height: height.max(480),
        grid,
    }
}

fn room_id(region_id: &str) -> String {
    format!("room.{}", slugify_label(region_id))
}
