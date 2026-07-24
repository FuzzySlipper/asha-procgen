fn physical_connection_plan_command(args: PhysicalConnectionPlanArgs) -> Result<(), String> {
    let candidate: Candidate = read_json(&args.candidate)?;
    let intermediate: IntermediateBreakdown = read_json(&args.intermediate)?;
    let plan = plan_physical_connections(&candidate, &intermediate, &args)?;
    write_json(&args.out, &plan)
}

fn plan_physical_connections(
    candidate: &Candidate,
    intermediate: &IntermediateBreakdown,
    args: &PhysicalConnectionPlanArgs,
) -> Result<PhysicalConnectionPlan, String> {
    if intermediate.candidate_id != candidate.candidate_id {
        return Err(format!(
            "intermediate candidate {} does not match candidate {}",
            intermediate.candidate_id, candidate.candidate_id
        ));
    }
    let edges = candidate
        .graph
        .edges
        .iter()
        .map(|edge| (edge.id.as_str(), edge))
        .collect::<BTreeMap<_, _>>();
    let mut grouped: BTreeMap<String, Vec<(&IntermediateConnector, &Edge)>> = BTreeMap::new();
    for connector in &intermediate.connectors {
        let edge = edges.get(connector.edge_id.as_str()).copied().ok_or_else(|| {
            format!("connector {} references missing edge {}", connector.id, connector.edge_id)
        })?;
        let mergeable_open = edge.traversal == TraversalKind::Open
            && edge.required_item.is_none()
            && connector.traversal_hint == "open";
        let key = if mergeable_open {
            let mut terminals = [connector.from_region.as_str(), connector.to_region.as_str()];
            terminals.sort();
            format!("open:{}:{}", terminals[0], terminals[1])
        } else {
            format!("edge:{}", connector.id)
        };
        grouped.entry(key).or_default().push((connector, edge));
    }

    let mut sections = Vec::new();
    let mut edge_mappings = Vec::new();
    for (group_key, members) in grouped {
        let mut terminal_regions = members
            .iter()
            .flat_map(|(connector, _)| [connector.from_region.clone(), connector.to_region.clone()])
            .collect::<Vec<_>>();
        terminal_regions.sort();
        terminal_regions.dedup();
        if terminal_regions.len() != 2 {
            return Err(format!(
                "physical connection group {group_key} requires exactly two terminals; found {}",
                terminal_regions.len()
            ));
        }
        let section_suffix = group_key
            .strip_prefix("edge:")
            .map(slugify_label)
            .unwrap_or_else(|| "open".to_owned());
        let section_id = format!(
            "section.{}.{}.{}",
            slugify_label(terminal_regions[0].as_str()),
            slugify_label(terminal_regions[1].as_str()),
            section_suffix
        );
        let mut source_connectors = Vec::new();
        let mut source_edges = Vec::new();
        let mut traversal_refs = Vec::new();
        let mut semantic_tags = Vec::new();
        let mut width = 0;
        for (connector, edge) in members {
            source_connectors.push(connector.id.clone());
            source_edges.push(edge.id.clone());
            semantic_tags.extend(corridor_semantic_tags(connector));
            width = width.max(corridor_width(connector));
            traversal_refs.push(PhysicalTraversalRef {
                connector_id: connector.id.clone(),
                edge_id: edge.id.clone(),
                from_region: connector.from_region.clone(),
                to_region: connector.to_region.clone(),
                traversal: edge.traversal.as_str().to_owned(),
                required_item: edge.required_item.clone(),
            });
            edge_mappings.push(PhysicalEdgeMapping {
                edge_id: edge.id.clone(),
                connector_id: connector.id.clone(),
                section_id: section_id.clone(),
                from_region: connector.from_region.clone(),
                to_region: connector.to_region.clone(),
            });
        }
        source_connectors.sort();
        source_edges.sort();
        traversal_refs.sort_by(|left, right| left.edge_id.cmp(&right.edge_id));
        sections.push(PhysicalConnectionSection {
            id: section_id,
            topology: "corridor_2".to_owned(),
            terminal_regions,
            source_connectors,
            source_edges,
            traversal_refs,
            width,
            semantic_tags: dedupe_strings(semantic_tags),
        });
    }
    sections.sort_by(|left, right| left.id.cmp(&right.id));
    edge_mappings.sort_by(|left, right| left.edge_id.cmp(&right.edge_id));
    Ok(PhysicalConnectionPlan {
        kind: "asha_procgen.physical_connection_plan.v1".to_owned(),
        schema_version: 1,
        plan_id: format!("physical_connections.{}", candidate.candidate_id),
        candidate_id: candidate.candidate_id.clone(),
        source_candidate_ref: display_path(&args.candidate),
        source_intermediate_ref: display_path(&args.intermediate),
        sections,
        edge_mappings,
    })
}

fn geometry_emit_2d_command(args: GeometryEmit2dArgs) -> Result<(), String> {
    let candidate: Candidate = read_json(&args.candidate)?;
    let intermediate: IntermediateBreakdown = read_json(&args.intermediate)?;
    let connection_plan: PhysicalConnectionPlan = read_json(&args.connection_plan)?;
    let geometry = emit_geometry_2d(&candidate, &intermediate, &connection_plan, &args, args.seed)?;
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
    if geometry.source_connection_plan_ref.is_empty() || geometry.connection_plan_id.is_empty() {
        diagnostics.push(fatal(
            "geometry_connection_plan_ref_missing",
            None,
            None,
            "Geometry must identify the exact physical connection plan it projects.",
        ));
    }
    match validate_geometry_layout_policy(&geometry.layout_policy) {
        Err(error) => diagnostics.push(fatal(
            "geometry_layout_policy_invalid",
            None,
            None,
            error,
        )),
        Ok(()) => {
            let search = &geometry.layout_search;
            if search.spacing_tier >= geometry.layout_policy.max_spacing_tiers
                || search.room_order_attempt
                    >= geometry.layout_policy.room_order_attempts_per_tier
                || search.port_order_attempt >= GEOMETRY_PORT_ORDER_COUNT
                || search.route_order_attempt >= GEOMETRY_ROUTE_ORDER_COUNT
                || search.search_attempts == 0
                || search.search_attempts > geometry.layout_policy.max_search_attempts
            {
                diagnostics.push(fatal(
                    "geometry_layout_search_evidence_invalid",
                    None,
                    None,
                    "Geometry layout search evidence exceeds its policy tier, ordering, or attempt bounds.",
                ));
            } else if geometry_spacing_for_tier(
                &geometry.layout_policy,
                search.spacing_tier,
            )
            .is_ok_and(|expected| expected != search.effective_spacing)
            {
                diagnostics.push(fatal(
                    "geometry_layout_search_spacing_mismatch",
                    None,
                    None,
                    "Geometry effective spacing does not match its recorded policy tier.",
                ));
            }
        }
    }

    let mut rooms_by_id = BTreeMap::new();
    let mut rooms_by_region = BTreeMap::new();
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
        rooms_by_region.insert(room.source_region.as_str(), room);
        let mut room_port_positions = BTreeSet::new();
        let mut room_port_sections = BTreeSet::new();
        for port in &room.ports {
            if !geometry_point_on_rect_boundary(&port.point, &room.rect) {
                diagnostics.push(fatal(
                    "geometry_room_port_detached",
                    room.source_nodes.first().map(String::as_str),
                    None,
                    format!("Room port {} is not on room {} boundary.", port.id, room.id),
                ));
            }
            if !room_port_positions.insert((port.point.x, port.point.y)) {
                diagnostics.push(fatal(
                    "geometry_room_port_span_reused",
                    room.source_nodes.first().map(String::as_str),
                    None,
                    format!("Room {} reuses doorway position {},{}.", room.id, port.point.x, port.point.y),
                ));
            }
            if !room_port_sections.insert(port.section_id.as_str()) {
                diagnostics.push(fatal(
                    "geometry_room_port_section_duplicate",
                    room.source_nodes.first().map(String::as_str),
                    None,
                    format!("Room {} has duplicate ports for section {}.", room.id, port.section_id),
                ));
            }
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
    let mut represented_edges = BTreeSet::new();
    let mut represented_sections = BTreeSet::new();
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
        if !represented_sections.insert(corridor.physical_section.as_str()) {
            diagnostics.push(fatal(
                "geometry_physical_section_duplicate",
                None,
                Some(corridor.id.as_str()),
                format!(
                    "Physical section {} is represented by more than one corridor.",
                    corridor.physical_section
                ),
            ));
        }
        if corridor.physical_section.is_empty()
            || corridor.source_connectors.is_empty()
            || corridor.source_edges.is_empty()
            || corridor.traversal_refs.is_empty()
        {
            diagnostics.push(fatal(
                "geometry_corridor_source_missing",
                None,
                Some(corridor.id.as_str()),
                "Corridor must preserve source connector and source edge refs.",
            ));
        } else {
            for connector in &corridor.source_connectors {
                if !represented_connectors.insert(connector.as_str()) {
                    diagnostics.push(fatal(
                        "geometry_corridor_duplicate_connector",
                        None,
                        Some(corridor.id.as_str()),
                        format!("Connector {connector} is represented by more than one physical section."),
                    ));
                }
            }
            for edge in &corridor.source_edges {
                if !represented_edges.insert(edge.as_str()) {
                    diagnostics.push(fatal(
                        "geometry_corridor_duplicate_edge",
                        None,
                        Some(corridor.id.as_str()),
                        format!("Source edge {edge} is mapped to more than one physical section."),
                    ));
                }
            }
            let traversal_edges = corridor
                .traversal_refs
                .iter()
                .map(|reference| reference.edge_id.as_str())
                .collect::<BTreeSet<_>>();
            let traversal_connectors = corridor
                .traversal_refs
                .iter()
                .map(|reference| reference.connector_id.as_str())
                .collect::<BTreeSet<_>>();
            if traversal_edges
                != corridor.source_edges.iter().map(String::as_str).collect::<BTreeSet<_>>()
                || traversal_connectors
                    != corridor
                        .source_connectors
                        .iter()
                        .map(String::as_str)
                        .collect::<BTreeSet<_>>()
            {
                diagnostics.push(fatal(
                    "geometry_corridor_traversal_mapping_mismatch",
                    None,
                    Some(corridor.id.as_str()),
                    "Corridor traversal refs must exactly cover its source connectors and edges.",
                ));
            }
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
            if !from_room
                .ports
                .iter()
                .any(|port| port.id == corridor.from_port && port.section_id == corridor.physical_section)
                || !to_room
                    .ports
                    .iter()
                    .any(|port| port.id == corridor.to_port && port.section_id == corridor.physical_section)
            {
                diagnostics.push(fatal(
                    "geometry_corridor_port_mismatch",
                    None,
                    Some(corridor.id.as_str()),
                    format!("Corridor {} does not identify its planned terminal ports.", corridor.id),
                ));
            }
            for traversal in &corridor.traversal_refs {
                let terminal_pair = sorted_pair(
                    traversal.from_region.as_str(),
                    traversal.to_region.as_str(),
                );
                let room_pair = sorted_pair(
                    from_room.source_region.as_str(),
                    to_room.source_region.as_str(),
                );
                if terminal_pair != room_pair {
                    diagnostics.push(fatal(
                        "geometry_corridor_terminal_mapping_mismatch",
                        None,
                        Some(traversal.edge_id.as_str()),
                        format!(
                            "Traversal {} does not terminate at corridor rooms {} and {}.",
                            traversal.edge_id, from_room.source_region, to_room.source_region
                        ),
                    ));
                }
                if let (Some(source), Some(target)) = (
                    rooms_by_region.get(traversal.from_region.as_str()),
                    rooms_by_region.get(traversal.to_region.as_str()),
                ) {
                    adjacency.entry(source.id.as_str()).or_default().push(target.id.as_str());
                }
            }
        }
        if corridor.traversal_refs.iter().any(|reference| reference.traversal == "locked")
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
        if corridor.traversal_refs.iter().any(|reference| reference.traversal == "hidden")
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

    validate_exclusive_geometry_routes(geometry, &rooms_by_id, &mut diagnostics);

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

fn validate_exclusive_geometry_routes(
    geometry: &Geometry2dArtifact,
    rooms_by_id: &BTreeMap<&str, &GeometryRoom>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut cells = BTreeMap::<(i32, i32), (&str, i32)>::new();
    let mut route_cells = Vec::new();
    for corridor in &geometry.corridors {
        for point in rasterize_geometry_corridor(corridor) {
            for room in &geometry.rooms {
                if room.id != corridor.from_room
                    && room.id != corridor.to_room
                    && point.x > room.rect.x
                    && point.x < room.rect.x + room.rect.width
                    && point.y > room.rect.y
                    && point.y < room.rect.y + room.rect.height
                {
                    diagnostics.push(fatal(
                        "geometry_corridor_room_intrusion",
                        room.source_nodes.first().map(String::as_str),
                        Some(corridor.id.as_str()),
                        format!("Physical section {} enters unrelated room {}.", corridor.physical_section, room.id),
                    ));
                }
            }
            if let Some((other, _)) = cells.insert(
                (point.x, point.y),
                (corridor.physical_section.as_str(), corridor.width),
            ) {
                if other != corridor.physical_section {
                    diagnostics.push(fatal(
                        "geometry_physical_section_overlap",
                        None,
                        Some(corridor.id.as_str()),
                        format!("Physical sections {} and {} overlap at {},{}.", other, corridor.physical_section, point.x, point.y),
                    ));
                }
            }
            route_cells.push(((point.x, point.y), corridor.physical_section.as_str(), corridor.width));
        }
    }
    let mut reported = BTreeSet::new();
    for (position, section, width) in route_cells {
        let max_radius = align_geometry(width / 2 + 10 + GEOMETRY_CORRIDOR_SEPARATION, GEOMETRY_ROUTE_GRID)
            / GEOMETRY_ROUTE_GRID;
        for dy in -max_radius..=max_radius {
            for dx in -max_radius..=max_radius {
                let Some((other_section, other_width)) = cells.get(&(
                    position.0 + dx * GEOMETRY_ROUTE_GRID,
                    position.1 + dy * GEOMETRY_ROUTE_GRID,
                )) else {
                    continue;
                };
                if *other_section == section {
                    continue;
                }
                let distance = (dx.abs() + dy.abs()) * GEOMETRY_ROUTE_GRID;
                let required = width / 2 + *other_width / 2 + GEOMETRY_CORRIDOR_SEPARATION;
                if distance < required {
                    let pair = sorted_pair(section, other_section);
                    if reported.insert(pair.clone()) {
                        diagnostics.push(fatal(
                            "geometry_physical_section_contact",
                            None,
                            None,
                            format!("Unrelated physical sections {} and {} violate separation.", pair.0, pair.1),
                        ));
                    }
                }
            }
        }
    }
    let _ = rooms_by_id;
}

fn rasterize_geometry_corridor(corridor: &GeometryCorridor) -> Vec<GeometryPoint> {
    let mut cells = Vec::new();
    for segment in corridor.points.windows(2) {
        let from = &segment[0];
        let to = &segment[1];
        let dx = (to.x - from.x).signum() * GEOMETRY_ROUTE_GRID;
        let dy = (to.y - from.y).signum() * GEOMETRY_ROUTE_GRID;
        let mut cursor = (from.x, from.y);
        cells.push(GeometryPoint { x: cursor.0, y: cursor.1 });
        while cursor != (to.x, to.y) {
            cursor = (cursor.0 + dx, cursor.1 + dy);
            cells.push(GeometryPoint { x: cursor.0, y: cursor.1 });
        }
    }
    dedupe_points(cells)
}

const GEOMETRY_ROUTE_GRID: i32 = 8;
const GEOMETRY_PORT_MARGIN: i32 = 32;
const GEOMETRY_PORT_SPACING: i32 = 48;
const GEOMETRY_CORRIDOR_SEPARATION: i32 = 8;
const GEOMETRY_ROUTE_ORDER_COUNT: u32 = 4;
const GEOMETRY_PORT_ORDER_COUNT: u32 = 2;

#[derive(Clone, Debug)]
struct PhysicalPortDemand {
    section_id: String,
    side: String,
    width: i32,
    opposite_order: usize,
}

#[derive(Debug)]
struct GeometryPlacementResult {
    rooms: Vec<GeometryRoom>,
    corridors: Vec<GeometryCorridor>,
    bounds: GeometryBounds,
    search: GeometryLayoutSearchEvidence,
}

#[derive(Debug)]
enum GeometryPlacementAttemptError {
    Invalid(String),
    RoutesUnavailable {
        attempted_orders: u32,
        last_error: String,
    },
}

#[derive(Debug)]
enum PhysicalRouteAttemptError {
    Invalid(String),
    Unavailable(String),
}

fn emit_geometry_2d(
    candidate: &Candidate,
    intermediate: &IntermediateBreakdown,
    connection_plan: &PhysicalConnectionPlan,
    args: &GeometryEmit2dArgs,
    seed: u64,
) -> Result<Geometry2dArtifact, String> {
    if intermediate.candidate_id != candidate.candidate_id {
        return Err(format!(
            "intermediate candidate {} does not match candidate {}",
            intermediate.candidate_id, candidate.candidate_id
        ));
    }
    if connection_plan.candidate_id != candidate.candidate_id
        || connection_plan.kind != "asha_procgen.physical_connection_plan.v1"
    {
        return Err("physical connection plan does not match the supplied candidate".to_owned());
    }
    let layout_policy = match &args.layout_policy {
        Some(path) => read_json(path)?,
        None => default_geometry_layout_policy(),
    };
    validate_geometry_layout_policy(&layout_policy)?;
    let region_specs = ordered_geometry_region_specs(candidate, intermediate);
    let placement = place_and_route_physical_geometry(
        &region_specs,
        connection_plan,
        seed,
        &layout_policy,
    )?;
    let contents = geometry_contents(candidate, intermediate, &placement.rooms);
    Ok(Geometry2dArtifact {
        kind: "asha_procgen.geometry_2d.v1".to_owned(),
        schema_version: 1,
        geometry_id: format!("geometry.{}.{}", candidate.candidate_id, seed),
        candidate_id: candidate.candidate_id.clone(),
        seed,
        source_candidate_ref: display_path(&args.candidate),
        source_intermediate_ref: display_path(&args.intermediate),
        source_connection_plan_ref: display_path(&args.connection_plan),
        connection_plan_id: connection_plan.plan_id.clone(),
        layout_policy,
        layout_search: placement.search,
        bounds: placement.bounds,
        rooms: placement.rooms,
        corridors: placement.corridors,
        contents,
        skipped_connectors: Vec::new(),
    })
}

fn ordered_geometry_region_specs<'a>(
    candidate: &Candidate,
    intermediate: &'a IntermediateBreakdown,
) -> Vec<(usize, String, String, &'a IntermediateRegion)> {
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
    region_specs
}

fn default_geometry_layout_policy() -> GeometryLayoutPolicy {
    GeometryLayoutPolicy {
        kind: "asha_procgen.geometry_layout_policy.v1".to_owned(),
        schema_version: 1,
        initial_room_margin: 96,
        initial_column_gap: 144,
        initial_row_gap: 64,
        room_margin_growth: 48,
        column_gap_growth: 72,
        row_gap_growth: 40,
        max_spacing_tiers: 5,
        room_order_attempts_per_tier: 4,
        max_search_attempts: 80,
    }
}

fn validate_geometry_layout_policy(policy: &GeometryLayoutPolicy) -> Result<(), String> {
    if policy.kind != "asha_procgen.geometry_layout_policy.v1" || policy.schema_version != 1 {
        return Err("unsupported geometry layout policy; expected asha_procgen.geometry_layout_policy.v1".to_owned());
    }
    for (label, value, minimum, maximum) in [
        ("initialRoomMargin", policy.initial_room_margin, 32, 1_024),
        ("initialColumnGap", policy.initial_column_gap, 32, 1_024),
        ("initialRowGap", policy.initial_row_gap, 32, 1_024),
        ("roomMarginGrowth", policy.room_margin_growth, 0, 512),
        ("columnGapGrowth", policy.column_gap_growth, 0, 512),
        ("rowGapGrowth", policy.row_gap_growth, 0, 512),
    ] {
        if value < minimum || value > maximum || value % GEOMETRY_ROUTE_GRID != 0 {
            return Err(format!(
                "geometry layout policy {label} must be a multiple of {GEOMETRY_ROUTE_GRID} from {minimum} through {maximum}"
            ));
        }
    }
    if policy.max_spacing_tiers == 0 || policy.max_spacing_tiers > 8 {
        return Err("geometry layout policy maxSpacingTiers must be from 1 through 8".to_owned());
    }
    if policy.room_order_attempts_per_tier == 0 || policy.room_order_attempts_per_tier > 32 {
        return Err(
            "geometry layout policy roomOrderAttemptsPerTier must be from 1 through 32".to_owned(),
        );
    }
    let available_attempts = policy
        .max_spacing_tiers
        .saturating_mul(policy.room_order_attempts_per_tier)
        .saturating_mul(GEOMETRY_ROUTE_ORDER_COUNT);
    if policy.max_search_attempts == 0 || policy.max_search_attempts > available_attempts {
        return Err(format!(
            "geometry layout policy maxSearchAttempts must be from 1 through {available_attempts}"
        ));
    }
    let final_tier = policy.max_spacing_tiers - 1;
    for (label, initial, growth) in [
        (
            "roomMargin",
            policy.initial_room_margin,
            policy.room_margin_growth,
        ),
        (
            "columnGap",
            policy.initial_column_gap,
            policy.column_gap_growth,
        ),
        (
            "rowGap",
            policy.initial_row_gap,
            policy.row_gap_growth,
        ),
    ] {
        let final_value = initial
            .checked_add(
                growth
                    .checked_mul(i32::try_from(final_tier).unwrap_or(i32::MAX))
                    .unwrap_or(i32::MAX),
            )
            .unwrap_or(i32::MAX);
        if final_value > 2_048 {
            return Err(format!(
                "geometry layout policy {label} exceeds 2048 units at the final tier"
            ));
        }
    }
    Ok(())
}

fn geometry_spacing_for_tier(
    policy: &GeometryLayoutPolicy,
    tier: u32,
) -> Result<GeometrySpacing, String> {
    let grow = |initial: i32, per_tier: i32| {
        initial.checked_add(
            per_tier
                .checked_mul(i32::try_from(tier).unwrap_or(i32::MAX))
                .unwrap_or(i32::MAX),
        )
    };
    let spacing = GeometrySpacing {
        room_margin: grow(policy.initial_room_margin, policy.room_margin_growth)
            .ok_or_else(|| "geometry layout policy room margin overflowed".to_owned())?,
        column_gap: grow(policy.initial_column_gap, policy.column_gap_growth)
            .ok_or_else(|| "geometry layout policy column gap overflowed".to_owned())?,
        row_gap: grow(policy.initial_row_gap, policy.row_gap_growth)
            .ok_or_else(|| "geometry layout policy row gap overflowed".to_owned())?,
    };
    if spacing.room_margin > 2_048 || spacing.column_gap > 2_048 || spacing.row_gap > 2_048 {
        return Err("geometry layout policy effective spacing exceeds 2048 units".to_owned());
    }
    Ok(spacing)
}

fn place_and_route_physical_geometry(
    base_specs: &[(usize, String, String, &IntermediateRegion)],
    connection_plan: &PhysicalConnectionPlan,
    seed: u64,
    policy: &GeometryLayoutPolicy,
) -> Result<GeometryPlacementResult, String> {
    let mut search_attempts = 0_u32;
    let mut last_error = "no physical route order was attempted".to_owned();
    let mut last_spacing = geometry_spacing_for_tier(policy, 0)?;
    let mut spacing_tiers_attempted = 0_u32;
    for spacing_tier in 0..policy.max_spacing_tiers {
        if search_attempts >= policy.max_search_attempts {
            break;
        }
        let spacing = geometry_spacing_for_tier(policy, spacing_tier)?;
        last_spacing = spacing.clone();
        spacing_tiers_attempted += 1;
        for room_order_attempt in 0..policy.room_order_attempts_per_tier {
            if search_attempts >= policy.max_search_attempts {
                break;
            }
            let mut specs = base_specs.to_vec();
            specs.sort_by(|left, right| {
                left.0.cmp(&right.0).then_with(|| {
                    if room_order_attempt == 0 {
                        left.1.cmp(&right.1).then_with(|| left.2.cmp(&right.2))
                    } else {
                        geometry_layout_order_key(
                            left.3.id.as_str(),
                            seed,
                            u64::from(spacing_tier)
                                .saturating_mul(u64::from(policy.room_order_attempts_per_tier))
                                .saturating_add(u64::from(room_order_attempt)),
                        )
                        .cmp(&geometry_layout_order_key(
                            right.3.id.as_str(),
                            seed,
                            u64::from(spacing_tier)
                                .saturating_mul(u64::from(policy.room_order_attempts_per_tier))
                                .saturating_add(u64::from(room_order_attempt)),
                        ))
                        .then_with(|| left.2.cmp(&right.2))
                    }
                })
            });
            let remaining_attempts = policy.max_search_attempts - search_attempts;
            match place_and_route_physical_geometry_attempt(
                &specs,
                connection_plan,
                &spacing,
                seed,
                room_order_attempt,
                remaining_attempts.min(GEOMETRY_ROUTE_ORDER_COUNT),
            ) {
                Ok((
                    rooms,
                    corridors,
                    bounds,
                    port_order_attempt,
                    route_order_attempt,
                    attempted_orders,
                )) => {
                    search_attempts += attempted_orders;
                    return Ok(GeometryPlacementResult {
                        rooms,
                        corridors,
                        bounds,
                        search: GeometryLayoutSearchEvidence {
                            spacing_tier,
                            room_order_attempt,
                            port_order_attempt,
                            route_order_attempt,
                            search_attempts,
                            effective_spacing: spacing,
                        },
                    });
                }
                Err(GeometryPlacementAttemptError::Invalid(error)) => {
                    return Err(format!("invalid physical geometry plan: {error}"));
                }
                Err(GeometryPlacementAttemptError::RoutesUnavailable {
                    attempted_orders,
                    last_error: error,
                }) => {
                    search_attempts += attempted_orders;
                    last_error = error;
                }
            }
        }
    }
    Err(format!(
        "geometry search exhausted after {search_attempts} route attempt(s) across {} spacing tier(s); initial spacing margin/column/row={}/{}/{}, final spacing={}/{}/{}; last route failure: {last_error}",
        spacing_tiers_attempted,
        policy.initial_room_margin,
        policy.initial_column_gap,
        policy.initial_row_gap,
        last_spacing.room_margin,
        last_spacing.column_gap,
        last_spacing.row_gap,
    ))
}

fn geometry_layout_order_key(id: &str, seed: u64, attempt: u64) -> u64 {
    let mut value = seed ^ attempt.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    for byte in id.bytes() {
        value ^= u64::from(byte);
        value = value.wrapping_mul(0x100_0000_01B3);
        value ^= value >> 29;
    }
    value
}

fn place_and_route_physical_geometry_attempt(
    region_specs: &[(usize, String, String, &IntermediateRegion)],
    connection_plan: &PhysicalConnectionPlan,
    spacing: &GeometrySpacing,
    seed: u64,
    room_order_attempt: u32,
    max_route_attempts: u32,
) -> Result<
    (
        Vec<GeometryRoom>,
        Vec<GeometryCorridor>,
        GeometryBounds,
        u32,
        u32,
        u32,
    ),
    GeometryPlacementAttemptError,
> {
    let region_depths = region_specs
        .iter()
        .map(|(depth, _, _, region)| (region.id.as_str(), *depth))
        .collect::<BTreeMap<_, _>>();
    let mut next_order_by_depth = BTreeMap::new();
    let mut region_orders = BTreeMap::new();
    for (depth, _, _, region) in region_specs {
        let order = next_order_by_depth.entry(*depth).or_insert(0_usize);
        region_orders.insert(region.id.as_str(), *order);
        *order += 1;
    }
    let canonical_attempts = max_route_attempts.min(2);
    let port_attempts = [
        (0_u32, canonical_attempts),
        (1_u32, max_route_attempts.saturating_sub(canonical_attempts)),
    ];
    let mut attempted_orders = 0_u32;
    let mut last_error = "no physical route order was attempted".to_owned();
    for (port_order_attempt, route_attempt_limit) in port_attempts {
        if route_attempt_limit == 0 {
            continue;
        }
        let port_demands = physical_port_demands(
            connection_plan,
            &region_depths,
            &region_orders,
            seed,
            port_order_attempt,
        )
        .map_err(GeometryPlacementAttemptError::Invalid)?;
        let (rooms, bounds) =
            place_geometry_rooms(region_specs, spacing, &port_demands)?;
        match route_physical_sections(
            connection_plan,
            &rooms,
            &bounds,
            seed,
            room_order_attempt
                .saturating_mul(GEOMETRY_PORT_ORDER_COUNT)
                .saturating_add(port_order_attempt),
            route_attempt_limit,
        ) {
            Ok((corridors, route_order_attempt, routes_tried)) => {
                attempted_orders += routes_tried;
                return Ok((
                    rooms,
                    corridors,
                    bounds,
                    port_order_attempt,
                    route_order_attempt,
                    attempted_orders,
                ));
            }
            Err(GeometryPlacementAttemptError::Invalid(error)) => {
                return Err(GeometryPlacementAttemptError::Invalid(error));
            }
            Err(GeometryPlacementAttemptError::RoutesUnavailable {
                attempted_orders: routes_tried,
                last_error: error,
            }) => {
                attempted_orders += routes_tried;
                last_error = error;
            }
        }
    }
    Err(GeometryPlacementAttemptError::RoutesUnavailable {
        attempted_orders,
        last_error,
    })
}

fn place_geometry_rooms(
    region_specs: &[(usize, String, String, &IntermediateRegion)],
    spacing: &GeometrySpacing,
    port_demands: &BTreeMap<String, Vec<PhysicalPortDemand>>,
) -> Result<(Vec<GeometryRoom>, GeometryBounds), GeometryPlacementAttemptError> {
    let mut column_widths = BTreeMap::<usize, i32>::new();
    for (depth, _, _, region) in region_specs {
        let (width, _) = connection_aware_room_size(
            region,
            port_demands
                .get(region.id.as_str())
                .map(Vec::as_slice)
                .unwrap_or_default(),
        );
        column_widths
            .entry(*depth)
            .and_modify(|existing| *existing = (*existing).max(width))
            .or_insert(width);
    }
    let mut column_origins = BTreeMap::new();
    let mut next_x = spacing.room_margin;
    for (depth, width) in &column_widths {
        column_origins.insert(*depth, next_x);
        next_x += *width + spacing.column_gap;
    }
    let mut next_y_by_depth = BTreeMap::<usize, i32>::new();
    let mut rooms = Vec::new();
    for (depth, _role, _id, region) in region_specs.iter().cloned() {
        let demands = port_demands
            .get(region.id.as_str())
            .map(Vec::as_slice)
            .unwrap_or_default();
        let (width, height) = connection_aware_room_size(region, demands);
        let x = *column_origins
            .get(&depth)
            .ok_or_else(|| {
                GeometryPlacementAttemptError::Invalid(format!(
                    "missing room column for graph depth {depth}"
                ))
            })?;
        let y = *next_y_by_depth.entry(depth).or_insert(spacing.room_margin);
        next_y_by_depth.insert(depth, y + height + spacing.row_gap);
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
            ports: Vec::new(),
            style_tags: geometry_room_style_tags(region),
        });
    }
    assign_physical_room_ports(&mut rooms, port_demands)
        .map_err(GeometryPlacementAttemptError::Invalid)?;
    let bounds = geometry_bounds(&rooms, GEOMETRY_ROUTE_GRID, spacing.room_margin);
    Ok((rooms, bounds))
}

fn physical_port_demands(
    plan: &PhysicalConnectionPlan,
    depths: &BTreeMap<&str, usize>,
    orders: &BTreeMap<&str, usize>,
    seed: u64,
    port_order_attempt: u32,
) -> Result<BTreeMap<String, Vec<PhysicalPortDemand>>, String> {
    let mut demands = BTreeMap::<String, Vec<PhysicalPortDemand>>::new();
    for section in &plan.sections {
        if section.topology != "corridor_2" || section.terminal_regions.len() != 2 {
            return Err(format!("unsupported physical section topology on {}", section.id));
        }
        let left = section.terminal_regions[0].as_str();
        let right = section.terminal_regions[1].as_str();
        let left_depth = depths.get(left).copied().unwrap_or(0);
        let right_depth = depths.get(right).copied().unwrap_or(0);
        let left_order = orders.get(left).copied().unwrap_or(0);
        let right_order = orders.get(right).copied().unwrap_or(0);
        let (left_side, right_side) = physical_port_sides(
            left_depth,
            right_depth,
            left_order,
            right_order,
            seed,
            port_order_attempt,
            section.id.as_str(),
        );
        demands.entry(left.to_owned()).or_default().push(PhysicalPortDemand {
            section_id: section.id.clone(),
            side: left_side.to_owned(),
            width: section.width,
            opposite_order: right_order,
        });
        demands.entry(right.to_owned()).or_default().push(PhysicalPortDemand {
            section_id: section.id.clone(),
            side: right_side.to_owned(),
            width: section.width,
            opposite_order: left_order,
        });
    }
    for room_demands in demands.values_mut() {
        room_demands.sort_by(|left, right| {
            left.side.cmp(&right.side).then_with(|| {
                compare_physical_port_demands(left, right, seed, port_order_attempt)
            })
        });
    }
    Ok(demands)
}

#[allow(clippy::too_many_arguments)]
fn physical_port_sides(
    left_depth: usize,
    right_depth: usize,
    left_order: usize,
    right_order: usize,
    seed: u64,
    attempt: u32,
    section_id: &str,
) -> (&'static str, &'static str) {
    let vertical = if left_order <= right_order {
        ("south", "north")
    } else {
        ("north", "south")
    };
    let horizontal = if left_depth <= right_depth {
        ("east", "west")
    } else {
        ("west", "east")
    };
    if attempt == 0 {
        return if left_depth == right_depth {
            vertical
        } else {
            horizontal
        };
    }

    let variant = geometry_layout_order_key(section_id, seed, u64::from(attempt)) % 3;
    if left_depth == right_depth {
        match variant {
            0 => horizontal,
            1 => (vertical.0, horizontal.1),
            _ => (horizontal.0, vertical.1),
        }
    } else {
        match variant {
            0 => vertical,
            1 => (horizontal.0, vertical.1),
            _ => (vertical.0, horizontal.1),
        }
    }
}

fn compare_physical_port_demands(
    left: &PhysicalPortDemand,
    right: &PhysicalPortDemand,
    seed: u64,
    attempt: u32,
) -> Ordering {
    match attempt % GEOMETRY_PORT_ORDER_COUNT {
        0 => left
            .opposite_order
            .cmp(&right.opposite_order)
            .then_with(|| left.section_id.cmp(&right.section_id)),
        1 => right
            .opposite_order
            .cmp(&left.opposite_order)
            .then_with(|| left.section_id.cmp(&right.section_id)),
        2 => geometry_layout_order_key(left.section_id.as_str(), seed, u64::from(attempt))
            .cmp(&geometry_layout_order_key(
                right.section_id.as_str(),
                seed,
                u64::from(attempt),
            ))
            .then_with(|| left.section_id.cmp(&right.section_id)),
        _ => geometry_layout_order_key(right.section_id.as_str(), seed, u64::from(attempt))
            .cmp(&geometry_layout_order_key(
                left.section_id.as_str(),
                seed,
                u64::from(attempt),
            ))
            .then_with(|| left.section_id.cmp(&right.section_id)),
    }
}

fn connection_aware_room_size(
    region: &IntermediateRegion,
    demands: &[PhysicalPortDemand],
) -> (i32, i32) {
    let (base_width, base_height) = room_size_for_region(region);
    let count = |side: &str| demands.iter().filter(|demand| demand.side == side).count() as i32;
    let horizontal = count("north").max(count("south"));
    let vertical = count("east").max(count("west"));
    let span = |ports: i32| {
        if ports == 0 { 0 } else { GEOMETRY_PORT_MARGIN * 2 + (ports - 1) * GEOMETRY_PORT_SPACING }
    };
    (
        align_geometry(base_width.max(span(horizontal)), GEOMETRY_ROUTE_GRID * 2),
        align_geometry(base_height.max(span(vertical)), GEOMETRY_ROUTE_GRID * 2),
    )
}

fn align_geometry(value: i32, grid: i32) -> i32 {
    ((value + grid - 1) / grid) * grid
}

fn assign_physical_room_ports(
    rooms: &mut [GeometryRoom],
    demands: &BTreeMap<String, Vec<PhysicalPortDemand>>,
) -> Result<(), String> {
    for room in rooms {
        let room_demands = demands
            .get(room.source_region.as_str())
            .map(Vec::as_slice)
            .unwrap_or_default();
        for side in ["north", "east", "south", "west"] {
            let side_demands = room_demands
                .iter()
                .filter(|demand| demand.side == side)
                .collect::<Vec<_>>();
            let count = side_demands.len() as i32;
            for (index, demand) in side_demands.into_iter().enumerate() {
                let offset = (index as i32 * 2 - (count - 1)) * GEOMETRY_PORT_SPACING / 2;
                let point = match side {
                    "north" => GeometryPoint { x: room.rect.x + room.rect.width / 2 + offset, y: room.rect.y },
                    "east" => GeometryPoint { x: room.rect.x + room.rect.width, y: room.rect.y + room.rect.height / 2 + offset },
                    "south" => GeometryPoint { x: room.rect.x + room.rect.width / 2 + offset, y: room.rect.y + room.rect.height },
                    "west" => GeometryPoint { x: room.rect.x, y: room.rect.y + room.rect.height / 2 + offset },
                    _ => return Err(format!("unsupported room port side {side}")),
                };
                room.ports.push(GeometryRoomPort {
                    id: format!("port.{}.{}", slugify_label(room.id.as_str()), slugify_label(demand.section_id.as_str())),
                    section_id: demand.section_id.clone(),
                    side: side.to_owned(),
                    point,
                    width: demand.width,
                });
            }
        }
    }
    Ok(())
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

fn route_physical_sections(
    plan: &PhysicalConnectionPlan,
    rooms: &[GeometryRoom],
    bounds: &GeometryBounds,
    seed: u64,
    order_nonce: u32,
    max_attempts: u32,
) -> Result<(Vec<GeometryCorridor>, u32, u32), GeometryPlacementAttemptError> {
    let rooms_by_region = rooms
        .iter()
        .map(|room| (room.source_region.as_str(), room))
        .collect::<BTreeMap<_, _>>();
    let mut sections = plan.sections.iter().collect::<Vec<_>>();
    sections.sort_by(|left, right| {
        let left_rooms = section_rooms(left, &rooms_by_region);
        let right_rooms = section_rooms(right, &rooms_by_region);
        let left_distance = left_rooms
            .map(|(from, to)| geometry_room_distance(from, to))
            .unwrap_or(0);
        let right_distance = right_rooms
            .map(|(from, to)| geometry_room_distance(from, to))
            .unwrap_or(0);
        right_distance.cmp(&left_distance).then_with(|| left.id.cmp(&right.id))
    });
    let mut orders = vec![sections.clone()];
    let mut reversed = sections.clone();
    reversed.reverse();
    orders.push(reversed);
    let mut seeded = sections.clone();
    seeded.sort_by(|left, right| {
        geometry_layout_order_key(left.id.as_str(), seed, u64::from(order_nonce))
            .cmp(&geometry_layout_order_key(
                right.id.as_str(),
                seed,
                u64::from(order_nonce),
            ))
            .then_with(|| left.id.cmp(&right.id))
    });
    orders.push(seeded.clone());
    seeded.reverse();
    orders.push(seeded);
    let mut attempted_orders = 0_u32;
    let mut last_error = "no physical route order was attempted".to_owned();
    for (index, order) in orders.into_iter().take(max_attempts as usize).enumerate() {
        attempted_orders += 1;
        match try_route_physical_sections(&order, &rooms_by_region, rooms, bounds) {
            Ok(corridors) => {
                return Ok((corridors, index as u32, attempted_orders));
            }
            Err(PhysicalRouteAttemptError::Invalid(error)) => {
                return Err(GeometryPlacementAttemptError::Invalid(error));
            }
            Err(PhysicalRouteAttemptError::Unavailable(error)) => last_error = error,
        }
    }
    Err(GeometryPlacementAttemptError::RoutesUnavailable {
        attempted_orders,
        last_error,
    })
}

fn section_rooms<'a>(
    section: &PhysicalConnectionSection,
    rooms: &BTreeMap<&str, &'a GeometryRoom>,
) -> Option<(&'a GeometryRoom, &'a GeometryRoom)> {
    if section.terminal_regions.len() != 2 {
        return None;
    }
    Some((
        *rooms.get(section.terminal_regions[0].as_str())?,
        *rooms.get(section.terminal_regions[1].as_str())?,
    ))
}

fn geometry_room_distance(left: &GeometryRoom, right: &GeometryRoom) -> u32 {
    let left = rect_center(&left.rect);
    let right = rect_center(&right.rect);
    left.x.abs_diff(right.x) + left.y.abs_diff(right.y)
}

fn try_route_physical_sections(
    sections: &[&PhysicalConnectionSection],
    rooms_by_region: &BTreeMap<&str, &GeometryRoom>,
    rooms: &[GeometryRoom],
    bounds: &GeometryBounds,
) -> Result<Vec<GeometryCorridor>, PhysicalRouteAttemptError> {
    let mut reserved = BTreeSet::new();
    let mut corridors = Vec::new();
    for section in sections {
        let (from_room, to_room) = section_rooms(section, rooms_by_region)
            .ok_or_else(|| {
                PhysicalRouteAttemptError::Invalid(format!(
                    "section {} references missing terminal room",
                    section.id
                ))
            })?;
        let from_port = from_room
            .ports
            .iter()
            .find(|port| port.section_id == section.id)
            .ok_or_else(|| {
                PhysicalRouteAttemptError::Invalid(format!(
                    "room {} lacks port for {}",
                    from_room.id, section.id
                ))
            })?;
        let to_port = to_room
            .ports
            .iter()
            .find(|port| port.section_id == section.id)
            .ok_or_else(|| {
                PhysicalRouteAttemptError::Invalid(format!(
                    "room {} lacks port for {}",
                    to_room.id, section.id
                ))
            })?;
        let path = route_physical_section(
            from_room,
            from_port,
            to_room,
            to_port,
            section.width,
            rooms,
            &reserved,
            bounds,
        )
        .ok_or_else(|| {
            PhysicalRouteAttemptError::Unavailable(format!(
                "single-floor route unavailable for physical section {}",
                section.id
            ))
        })?;
        reserve_geometry_route(&path, section.width, &mut reserved);
        let source_connector = section.source_connectors.first().cloned().unwrap_or_default();
        let source_edge = section.source_edges.first().cloned().unwrap_or_default();
        let traversal_hint = if section
            .traversal_refs
            .iter()
            .all(|reference| reference.traversal == "open")
        {
            "open".to_owned()
        } else {
            section
                .traversal_refs
                .first()
                .map(|reference| reference.traversal.clone())
                .unwrap_or_else(|| "open".to_owned())
        };
        corridors.push(GeometryCorridor {
            id: format!("corridor.{}", slugify_label(section.id.as_str())),
            physical_section: section.id.clone(),
            source_connector,
            source_edge,
            source_connectors: section.source_connectors.clone(),
            source_edges: section.source_edges.clone(),
            traversal_refs: section.traversal_refs.clone(),
            from_room: from_room.id.clone(),
            to_room: to_room.id.clone(),
            traversal_hint,
            semantic_tags: section.semantic_tags.clone(),
            width: section.width,
            from_port: from_port.id.clone(),
            to_port: to_port.id.clone(),
            points: compress_geometry_route(path),
        });
    }
    corridors.sort_by(|left, right| left.physical_section.cmp(&right.physical_section));
    Ok(corridors)
}

#[allow(clippy::too_many_arguments)]
fn route_physical_section(
    from_room: &GeometryRoom,
    from_port: &GeometryRoomPort,
    to_room: &GeometryRoom,
    to_port: &GeometryRoomPort,
    width: i32,
    rooms: &[GeometryRoom],
    reserved: &BTreeSet<(i32, i32)>,
    bounds: &GeometryBounds,
) -> Option<Vec<GeometryPoint>> {
    let start = (from_port.point.x, from_port.point.y);
    let end = (to_port.point.x, to_port.point.y);
    let mut queue = VecDeque::from([start]);
    let mut seen = HashSet::from([start]);
    let mut previous = HashMap::new();
    while let Some(position) = queue.pop_front() {
        if position == end {
            break;
        }
        let mut neighbors = vec![
            (position.0 + GEOMETRY_ROUTE_GRID, position.1),
            (position.0, position.1 + GEOMETRY_ROUTE_GRID),
            (position.0 - GEOMETRY_ROUTE_GRID, position.1),
            (position.0, position.1 - GEOMETRY_ROUTE_GRID),
        ];
        neighbors.sort_by_key(|neighbor| neighbor.0.abs_diff(end.0) + neighbor.1.abs_diff(end.1));
        for neighbor in neighbors {
            if !seen.insert(neighbor)
                || !geometry_route_available(
                    neighbor,
                    from_room,
                    from_port,
                    to_room,
                    to_port,
                    width,
                    rooms,
                    reserved,
                    bounds,
                )
            {
                continue;
            }
            previous.insert(neighbor, position);
            queue.push_back(neighbor);
        }
    }
    if !seen.contains(&end) {
        return None;
    }
    let mut path = vec![GeometryPoint { x: end.0, y: end.1 }];
    let mut cursor = end;
    while cursor != start {
        cursor = *previous.get(&cursor)?;
        path.push(GeometryPoint { x: cursor.0, y: cursor.1 });
    }
    path.reverse();
    Some(path)
}

#[allow(clippy::too_many_arguments)]
fn geometry_route_available(
    position: (i32, i32),
    from_room: &GeometryRoom,
    from_port: &GeometryRoomPort,
    to_room: &GeometryRoom,
    to_port: &GeometryRoomPort,
    width: i32,
    rooms: &[GeometryRoom],
    reserved: &BTreeSet<(i32, i32)>,
    bounds: &GeometryBounds,
) -> bool {
    if position.0 < 0
        || position.1 < 0
        || position.0 > bounds.width
        || position.1 > bounds.height
        || reserved.contains(&position)
    {
        return false;
    }
    let clearance = width / 2 + GEOMETRY_CORRIDOR_SEPARATION;
    rooms.iter().all(|room| {
        let blocked = position.0 >= room.rect.x - clearance
            && position.0 <= room.rect.x + room.rect.width + clearance
            && position.1 >= room.rect.y - clearance
            && position.1 <= room.rect.y + room.rect.height + clearance;
        if !blocked {
            return true;
        }
        (room.id == from_room.id && geometry_port_approach_contains(position, from_port, clearance))
            || (room.id == to_room.id && geometry_port_approach_contains(position, to_port, clearance))
    })
}

fn geometry_port_approach_contains(
    position: (i32, i32),
    port: &GeometryRoomPort,
    clearance: i32,
) -> bool {
    let (dx, dy) = direction_vector(port.side.as_str());
    let steps = align_geometry(clearance, GEOMETRY_ROUTE_GRID) / GEOMETRY_ROUTE_GRID + 1;
    (0..=steps).any(|step| {
        position
            == (
                port.point.x + dx * step * GEOMETRY_ROUTE_GRID,
                port.point.y + dy * step * GEOMETRY_ROUTE_GRID,
            )
    })
}

fn reserve_geometry_route(
    path: &[GeometryPoint],
    width: i32,
    reserved: &mut BTreeSet<(i32, i32)>,
) {
    let radius = align_geometry(width / 2 + GEOMETRY_CORRIDOR_SEPARATION + 10, GEOMETRY_ROUTE_GRID)
        / GEOMETRY_ROUTE_GRID;
    for point in path {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx.abs() + dy.abs() <= radius {
                    reserved.insert((
                        point.x + dx * GEOMETRY_ROUTE_GRID,
                        point.y + dy * GEOMETRY_ROUTE_GRID,
                    ));
                }
            }
        }
    }
}

fn compress_geometry_route(path: Vec<GeometryPoint>) -> Vec<GeometryPoint> {
    if path.len() <= 2 {
        return path;
    }
    let mut compressed = vec![path[0].clone()];
    for index in 1..path.len() - 1 {
        let previous = &path[index - 1];
        let current = &path[index];
        let next = &path[index + 1];
        if (current.x - previous.x, current.y - previous.y)
            != (next.x - current.x, next.y - current.y)
        {
            compressed.push(current.clone());
        }
    }
    compressed.push(path[path.len() - 1].clone());
    dedupe_points(compressed)
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

fn geometry_bounds(rooms: &[GeometryRoom], grid: i32, room_margin: i32) -> GeometryBounds {
    let width = rooms
        .iter()
        .map(|room| room.rect.x + room.rect.width)
        .max()
        .unwrap_or(0)
        + room_margin;
    let height = rooms
        .iter()
        .map(|room| room.rect.y + room.rect.height)
        .max()
        .unwrap_or(0)
        + room_margin;
    GeometryBounds {
        width: width.max(640),
        height: height.max(480),
        grid,
    }
}

fn room_id(region_id: &str) -> String {
    format!("room.{}", slugify_label(region_id))
}
