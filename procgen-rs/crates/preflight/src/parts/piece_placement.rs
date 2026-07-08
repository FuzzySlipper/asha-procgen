fn build_assemble_command(args: BuildAssembleArgs) -> Result<(), String> {
    let catalog: ShapeCatalog = read_json(&args.catalog)?;
    let plan: PieceBuildPlan = read_json(&args.piece_plan)?;
    let shape_match: PieceShapeMatchReport = read_json(&args.shape_match)?;
    let placement = assemble_piece_placement(&catalog, &plan, &shape_match, &args)?;
    write_json(&args.out, &placement)
}

fn build_validate_placement_command(args: ReportOutArgs) -> Result<(), String> {
    let placement: PiecePlacement = read_json(&args.state)?;
    let report = validate_piece_placement(&placement);
    write_json(&args.out, &report)?;
    if !report.ok {
        return Err(format!(
            "piece placement validation failed with {} fatal diagnostic(s); see {}",
            report.fatal_count,
            args.out.display()
        ));
    }
    Ok(())
}

fn assemble_piece_placement(
    catalog: &ShapeCatalog,
    plan: &PieceBuildPlan,
    shape_match: &PieceShapeMatchReport,
    args: &BuildAssembleArgs,
) -> Result<PiecePlacement, String> {
    if shape_match.plan_id != plan.plan_id {
        return Err(format!(
            "shape match plan {} does not match piece plan {}",
            shape_match.plan_id, plan.plan_id
        ));
    }
    if shape_match.catalog_id != catalog.catalog_id {
        return Err(format!(
            "shape match catalog {} does not match catalog {}",
            shape_match.catalog_id, catalog.catalog_id
        ));
    }
    if !shape_match.ok {
        return Err(format!(
            "shape match report {} has {} unmatched requirement(s)",
            shape_match.match_id, shape_match.unmatched_count
        ));
    }

    let requirements = plan
        .requirements
        .iter()
        .map(|requirement| (requirement.piece_id.as_str(), requirement))
        .collect::<BTreeMap<_, _>>();
    let shapes = catalog
        .shapes
        .iter()
        .map(|shape| (shape.shape_id.as_str(), shape))
        .collect::<BTreeMap<_, _>>();

    let mut instances = Vec::new();
    let mut occupied_cells = Vec::new();
    let mut reserved_cells = Vec::new();
    let mut occupied_positions: BTreeSet<(i32, i32)> = BTreeSet::new();
    let mut reserved_positions: BTreeSet<(i32, i32)> = BTreeSet::new();
    for (index, matched) in shape_match.matches.iter().enumerate() {
        let Some(requirement) = requirements.get(matched.piece_id.as_str()).copied() else {
            return Err(format!(
                "shape match references missing requirement {}",
                matched.piece_id
            ));
        };
        let Some(shape) = shapes.get(matched.shape_id.as_str()).copied() else {
            return Err(format!(
                "shape match references missing catalog shape {}",
                matched.shape_id
            ));
        };
        let instance_id = format!("instance.{}", slugify_label(matched.piece_id.as_str()));
        let desired_origin = desired_origin_for_requirement(requirement, index);
        let origin = find_available_origin(
            shape,
            matched.transform.as_str(),
            &desired_origin,
            &occupied_positions,
            &reserved_positions,
        );
        let occupied = transform_cells(&shape.footprint, matched.transform.as_str(), &origin);
        let reserved = transform_cells(&shape.reserved_cells, matched.transform.as_str(), &origin);
        for cell in &occupied {
            occupied_positions.insert((cell.x, cell.y));
        }
        for cell in &reserved {
            reserved_positions.insert((cell.x, cell.y));
        }

        occupied_cells.extend(occupied.iter().map(|cell| PlacementCellRef {
            instance_id: instance_id.clone(),
            x: cell.x,
            y: cell.y,
        }));
        reserved_cells.extend(reserved.iter().map(|cell| PlacementCellRef {
            instance_id: instance_id.clone(),
            x: cell.x,
            y: cell.y,
        }));

        instances.push(PieceInstance {
            instance_id,
            piece_id: matched.piece_id.clone(),
            requirement_kind: matched.requirement_kind.clone(),
            role: requirement.role.clone(),
            shape_id: matched.shape_id.clone(),
            transform: matched.transform.clone(),
            origin,
            occupied_cells: occupied,
            reserved_cells: reserved,
            exit_map: matched.exit_map.clone(),
            feature_placements: matched.socket_map.clone(),
            source_requirement_ref: matched.source_requirement_ref.clone(),
            source_refs: requirement.source_refs.clone(),
            tags: requirement.tags.clone(),
        });
    }

    let mut placement = PiecePlacement {
        kind: "asha_procgen.piece_placement.v1".to_owned(),
        schema_version: 1,
        placement_id: format!("piece_placement.{}", shape_match.match_id),
        plan_id: plan.plan_id.clone(),
        catalog_id: catalog.catalog_id.clone(),
        match_id: shape_match.match_id.clone(),
        source_plan_ref: display_path(&args.piece_plan),
        source_catalog_ref: display_path(&args.catalog),
        source_match_ref: display_path(&args.shape_match),
        cell_size: catalog.cell_size,
        grid_connectivity: args.connectivity,
        instances,
        glued_exits: Vec::new(),
        occupied_cells,
        connection_cells: Vec::new(),
        reserved_cells,
        dangling_exits: Vec::new(),
    };
    placement.glued_exits = derive_glued_exits(plan, &placement.instances);
    placement.connection_cells = derive_connection_cells(&placement);
    Ok(placement)
}

fn desired_origin_for_requirement(requirement: &PieceRequirement, index: usize) -> GridCell {
    const GEOMETRY_CELL_SIZE: i32 = 24;
    for hint in &requirement.placement_hints {
        if let Some(rest) = hint.strip_prefix("geometryRect:") {
            let values = parse_i32_parts(rest);
            if values.len() == 4 {
                return GridCell {
                    x: values[0] / GEOMETRY_CELL_SIZE,
                    y: values[1] / GEOMETRY_CELL_SIZE,
                };
            }
        }
        if let Some(rest) = hint.strip_prefix("segment:") {
            let values = parse_i32_parts(rest);
            if values.len() == 4 {
                return GridCell {
                    x: ((values[0] + values[2]) / 2) / GEOMETRY_CELL_SIZE,
                    y: ((values[1] + values[3]) / 2) / GEOMETRY_CELL_SIZE,
                };
            }
        }
        if let Some(rest) = hint.strip_prefix("bend:").or_else(|| hint.strip_prefix("point:")) {
            let values = parse_i32_parts(rest);
            if values.len() == 2 {
                return GridCell {
                    x: values[0] / GEOMETRY_CELL_SIZE,
                    y: values[1] / GEOMETRY_CELL_SIZE,
                };
            }
        }
    }
    GridCell {
        x: (index as i32 % 24) * 5,
        y: (index as i32 / 24) * 5,
    }
}

fn parse_i32_parts(value: &str) -> Vec<i32> {
    value
        .split(':')
        .filter_map(|part| part.parse::<i32>().ok())
        .collect()
}

fn find_available_origin(
    shape: &CatalogShape,
    transform: &str,
    desired_origin: &GridCell,
    occupied_positions: &BTreeSet<(i32, i32)>,
    reserved_positions: &BTreeSet<(i32, i32)>,
) -> GridCell {
    for radius in 0_i32..=40 {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx.abs().max(dy.abs()) != radius {
                    continue;
                }
                let origin = GridCell {
                    x: desired_origin.x + dx,
                    y: desired_origin.y + dy,
                };
                if origin_available(
                    shape,
                    transform,
                    &origin,
                    occupied_positions,
                    reserved_positions,
                ) {
                    return origin;
                }
            }
        }
    }
    desired_origin.clone()
}

fn origin_available(
    shape: &CatalogShape,
    transform: &str,
    origin: &GridCell,
    occupied_positions: &BTreeSet<(i32, i32)>,
    reserved_positions: &BTreeSet<(i32, i32)>,
) -> bool {
    let occupied = transform_cells(&shape.footprint, transform, origin);
    let reserved = transform_cells(&shape.reserved_cells, transform, origin);
    occupied.iter().all(|cell| {
        !occupied_positions.contains(&(cell.x, cell.y))
            && !reserved_positions.contains(&(cell.x, cell.y))
    }) && reserved.iter().all(|cell| {
        !occupied_positions.contains(&(cell.x, cell.y))
            && !reserved_positions.contains(&(cell.x, cell.y))
    })
}

fn derive_glued_exits(plan: &PieceBuildPlan, instances: &[PieceInstance]) -> Vec<GluedExit> {
    let instances_by_piece = instances
        .iter()
        .map(|instance| (instance.piece_id.as_str(), instance))
        .collect::<BTreeMap<_, _>>();
    let mut glued = Vec::new();
    for link in &plan.links {
        let Some(from) = instances_by_piece.get(link.from_piece.as_str()).copied() else {
            continue;
        };
        let Some(to) = instances_by_piece.get(link.to_piece.as_str()).copied() else {
            continue;
        };
        let Some((from_exit, to_exit)) = compatible_exit_pair(from, to) else {
            continue;
        };
        glued.push(GluedExit {
            id: format!("glue.{}", slugify_label(link.id.as_str())),
            link_id: link.id.clone(),
            from_instance: from.instance_id.clone(),
            from_exit: from_exit.requirement_exit_id.clone(),
            to_instance: to.instance_id.clone(),
            to_exit: to_exit.requirement_exit_id.clone(),
            source_ref: link.source_ref.clone(),
            tags: link.tags.clone(),
        });
    }
    glued
}

fn compatible_exit_pair<'a>(
    from: &'a PieceInstance,
    to: &'a PieceInstance,
) -> Option<(&'a MatchedExit, &'a MatchedExit)> {
    for from_exit in &from.exit_map {
        for to_exit in &to.exit_map {
            if opposite_direction(from_exit.direction.as_str()) == to_exit.direction {
                return Some((from_exit, to_exit));
            }
        }
    }
    None
}

fn derive_connection_cells(placement: &PiecePlacement) -> Vec<PlacementCellRef> {
    let instances = placement
        .instances
        .iter()
        .map(|instance| (instance.instance_id.as_str(), instance))
        .collect::<BTreeMap<_, _>>();
    let occupied = placement
        .occupied_cells
        .iter()
        .map(|cell| (cell.x, cell.y))
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    let mut cells = Vec::new();
    for glued in &placement.glued_exits {
        let Some(from) = instances.get(glued.from_instance.as_str()) else {
            continue;
        };
        let Some(to) = instances.get(glued.to_instance.as_str()) else {
            continue;
        };
        let Some((start, end)) = nearest_cell_pair(
            &from.occupied_cells,
            &to.occupied_cells,
            placement.grid_connectivity,
        ) else {
            continue;
        };
        if cells_adjacent(start, end, placement.grid_connectivity) {
            continue;
        }
        let instance_id = format!("connection.{}", slugify_label(glued.id.as_str()));
        for cell in bridge_cells(start, end, placement.grid_connectivity) {
            if occupied.contains(&(cell.x, cell.y)) {
                continue;
            }
            if seen.insert((cell.x, cell.y)) {
                cells.push(PlacementCellRef {
                    instance_id: instance_id.clone(),
                    x: cell.x,
                    y: cell.y,
                });
            }
        }
    }
    cells
}

fn nearest_cell_pair<'a>(
    from: &'a [GridCell],
    to: &'a [GridCell],
    connectivity: GridConnectivity,
) -> Option<(&'a GridCell, &'a GridCell)> {
    let mut best: Option<(&GridCell, &GridCell, i32)> = None;
    for from_cell in from {
        for to_cell in to {
            let distance = grid_distance(from_cell, to_cell, connectivity);
            if best
                .map(|(_, _, best_distance)| distance < best_distance)
                .unwrap_or(true)
            {
                best = Some((from_cell, to_cell, distance));
            }
        }
    }
    best.map(|(from_cell, to_cell, _)| (from_cell, to_cell))
}

fn grid_distance(from: &GridCell, to: &GridCell, connectivity: GridConnectivity) -> i32 {
    let dx = (from.x - to.x).abs();
    let dy = (from.y - to.y).abs();
    match connectivity {
        GridConnectivity::FourWay => dx + dy,
        GridConnectivity::EightWay => dx.max(dy),
    }
}

fn bridge_cells(from: &GridCell, to: &GridCell, connectivity: GridConnectivity) -> Vec<GridCell> {
    let mut cells = Vec::new();
    let mut x = from.x;
    let mut y = from.y;
    while x != to.x || y != to.y {
        match connectivity {
            GridConnectivity::FourWay => {
                if x != to.x {
                    x += (to.x - x).signum();
                } else if y != to.y {
                    y += (to.y - y).signum();
                }
            }
            GridConnectivity::EightWay => {
                if x != to.x {
                    x += (to.x - x).signum();
                }
                if y != to.y {
                    y += (to.y - y).signum();
                }
            }
        }
        if x != to.x || y != to.y {
            cells.push(GridCell { x, y });
        }
    }
    cells
}

fn cells_adjacent(from: &GridCell, to: &GridCell, connectivity: GridConnectivity) -> bool {
    if from.x == to.x && from.y == to.y {
        return true;
    }
    let dx = (from.x - to.x).abs();
    let dy = (from.y - to.y).abs();
    match connectivity {
        GridConnectivity::FourWay => dx + dy == 1,
        GridConnectivity::EightWay => dx.max(dy) == 1,
    }
}

fn validate_piece_placement(placement: &PiecePlacement) -> ValidationReport {
    let mut diagnostics = Vec::new();
    if placement.kind != "asha_procgen.piece_placement.v1" {
        diagnostics.push(fatal(
            "piece_placement_kind_invalid",
            None,
            None,
            "Placement artifact kind must be asha_procgen.piece_placement.v1.",
        ));
    }
    validate_placement_cells(placement, &mut diagnostics);
    validate_placement_links(placement, &mut diagnostics);
    validate_placement_reachability(placement, &mut diagnostics);
    validate_placement_grid_reachability(placement, &mut diagnostics);
    let fatal_count = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Fatal)
        .count();
    ValidationReport {
        kind: "asha_procgen.validation.piece_placement.v1".to_owned(),
        schema_version: 1,
        state_hash: hash_json(placement).unwrap_or_else(|_| "hash_error".to_owned()),
        ok: fatal_count == 0,
        fatal_count,
        diagnostics,
    }
}

fn validate_placement_cells(placement: &PiecePlacement, diagnostics: &mut Vec<Diagnostic>) {
    let mut occupied_by_cell: BTreeMap<(i32, i32), &str> = BTreeMap::new();
    for cell in &placement.occupied_cells {
        if let Some(existing) = occupied_by_cell.insert((cell.x, cell.y), cell.instance_id.as_str()) {
            diagnostics.push(fatal(
                "piece_occupied_cell_overlap",
                None,
                None,
                format!(
                    "Occupied cell {},{} is shared by {} and {}.",
                    cell.x, cell.y, existing, cell.instance_id
                ),
            ));
        }
    }
    let mut reserved_by_cell: BTreeMap<(i32, i32), &str> = BTreeMap::new();
    for cell in &placement.reserved_cells {
        if let Some(occupier) = occupied_by_cell.get(&(cell.x, cell.y)) {
            if *occupier != cell.instance_id {
                diagnostics.push(fatal(
                    "piece_reserved_cell_conflict",
                    None,
                    None,
                    format!(
                        "Reserved cell {},{} for {} is occupied by {}.",
                        cell.x, cell.y, cell.instance_id, occupier
                    ),
                ));
            }
        }
        if let Some(existing) = reserved_by_cell.insert((cell.x, cell.y), cell.instance_id.as_str()) {
            if existing != cell.instance_id {
                diagnostics.push(fatal(
                    "piece_reserved_cell_overlap",
                    None,
                    None,
                    format!(
                        "Reserved cell {},{} is shared by {} and {}.",
                        cell.x, cell.y, existing, cell.instance_id
                    ),
                ));
            }
        }
    }
    let mut connection_by_cell: BTreeMap<(i32, i32), &str> = BTreeMap::new();
    for cell in &placement.connection_cells {
        if let Some(existing) =
            connection_by_cell.insert((cell.x, cell.y), cell.instance_id.as_str())
        {
            diagnostics.push(fatal(
                "piece_connection_cell_overlap",
                None,
                None,
                format!(
                    "Connection cell {},{} is shared by {} and {}.",
                    cell.x, cell.y, existing, cell.instance_id
                ),
            ));
        }
    }
}

fn validate_placement_links(placement: &PiecePlacement, diagnostics: &mut Vec<Diagnostic>) {
    let instances = placement
        .instances
        .iter()
        .map(|instance| (instance.instance_id.as_str(), instance))
        .collect::<BTreeMap<_, _>>();
    let mut glued_ids = BTreeSet::new();
    for glued in &placement.glued_exits {
        if !glued_ids.insert(glued.id.as_str()) {
            diagnostics.push(fatal(
                "piece_glued_exit_duplicate",
                None,
                None,
                format!("Duplicate glued exit id {}.", glued.id),
            ));
        }
        let Some(from) = instances.get(glued.from_instance.as_str()) else {
            diagnostics.push(fatal(
                "piece_glued_exit_instance_missing",
                None,
                None,
                format!("Glued exit {} references missing from instance.", glued.id),
            ));
            continue;
        };
        let Some(to) = instances.get(glued.to_instance.as_str()) else {
            diagnostics.push(fatal(
                "piece_glued_exit_instance_missing",
                None,
                None,
                format!("Glued exit {} references missing to instance.", glued.id),
            ));
            continue;
        };
        let Some(from_exit) = from
            .exit_map
            .iter()
            .find(|exit| exit.requirement_exit_id == glued.from_exit)
        else {
            diagnostics.push(fatal(
                "piece_glued_exit_endpoint_missing",
                None,
                None,
                format!("Glued exit {} references missing from exit.", glued.id),
            ));
            continue;
        };
        let Some(to_exit) = to
            .exit_map
            .iter()
            .find(|exit| exit.requirement_exit_id == glued.to_exit)
        else {
            diagnostics.push(fatal(
                "piece_glued_exit_endpoint_missing",
                None,
                None,
                format!("Glued exit {} references missing to exit.", glued.id),
            ));
            continue;
        };
        if opposite_direction(from_exit.direction.as_str()) != to_exit.direction {
            diagnostics.push(fatal(
                "piece_glued_exit_incompatible",
                None,
                None,
                format!(
                    "Glued exit {} joins {} to {}, which are not opposite directions.",
                    glued.id, from_exit.direction, to_exit.direction
                ),
            ));
        }
    }
    for dangling in &placement.dangling_exits {
        diagnostics.push(fatal(
            "piece_required_exit_dangling",
            None,
            None,
            format!(
                "Instance {} has dangling required exit {} ({})",
                dangling.instance_id, dangling.exit_id, dangling.reason
            ),
        ));
    }
}

fn validate_placement_reachability(placement: &PiecePlacement, diagnostics: &mut Vec<Diagnostic>) {
    let starts = placement
        .instances
        .iter()
        .filter(|instance| instance.tags.iter().any(|tag| tag == "start_marker" || tag == "start"))
        .map(|instance| instance.instance_id.as_str())
        .collect::<Vec<_>>();
    let goals = placement
        .instances
        .iter()
        .filter(|instance| instance.tags.iter().any(|tag| tag == "goal_marker" || tag == "goal"))
        .map(|instance| instance.instance_id.as_str())
        .collect::<BTreeSet<_>>();
    if starts.is_empty() || goals.is_empty() {
        return;
    }
    let mut adjacency: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for glued in &placement.glued_exits {
        adjacency
            .entry(glued.from_instance.as_str())
            .or_default()
            .push(glued.to_instance.as_str());
        adjacency
            .entry(glued.to_instance.as_str())
            .or_default()
            .push(glued.from_instance.as_str());
    }
    let mut queue = VecDeque::new();
    let mut seen = BTreeSet::new();
    for start in starts {
        queue.push_back(start);
        seen.insert(start);
    }
    while let Some(instance) = queue.pop_front() {
        if goals.contains(instance) {
            return;
        }
        for next in adjacency.get(instance).into_iter().flatten() {
            if seen.insert(*next) {
                queue.push_back(*next);
            }
        }
    }
    diagnostics.push(fatal(
        "piece_goal_unreachable",
        None,
        None,
        "No glued-exit path reaches a goal instance from a start instance.",
    ));
}

fn validate_placement_grid_reachability(
    placement: &PiecePlacement,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let grid_cells = placement
        .occupied_cells
        .iter()
        .chain(placement.connection_cells.iter())
        .map(|cell| (cell.x, cell.y))
        .collect::<BTreeSet<_>>();
    if grid_cells.is_empty() {
        return;
    }
    let start = placement
        .instances
        .iter()
        .find(|instance| instance.tags.iter().any(|tag| tag == "start_marker" || tag == "start"))
        .and_then(|instance| instance.occupied_cells.first())
        .or_else(|| {
            placement
                .instances
                .iter()
                .find(|instance| !instance.occupied_cells.is_empty())
                .and_then(|instance| instance.occupied_cells.first())
        });
    let Some(start) = start else {
        return;
    };
    let mut seen = BTreeSet::new();
    let mut queue = VecDeque::new();
    if grid_cells.contains(&(start.x, start.y)) {
        seen.insert((start.x, start.y));
        queue.push_back((start.x, start.y));
    }
    while let Some(cell) = queue.pop_front() {
        for neighbor in grid_neighbors(cell, placement.grid_connectivity) {
            if grid_cells.contains(&neighbor) && seen.insert(neighbor) {
                queue.push_back(neighbor);
            }
        }
    }
    for instance in &placement.instances {
        if instance.occupied_cells.is_empty() {
            continue;
        }
        if !instance
            .occupied_cells
            .iter()
            .any(|cell| seen.contains(&(cell.x, cell.y)))
        {
            diagnostics.push(fatal(
                "piece_grid_instance_unreachable",
                None,
                None,
                format!(
                    "Instance {} is not physically reachable on the {:?} placement grid.",
                    instance.instance_id, placement.grid_connectivity
                ),
            ));
        }
    }
}

fn grid_neighbors(cell: (i32, i32), connectivity: GridConnectivity) -> Vec<(i32, i32)> {
    let mut neighbors = vec![
        (cell.0 + 1, cell.1),
        (cell.0 - 1, cell.1),
        (cell.0, cell.1 + 1),
        (cell.0, cell.1 - 1),
    ];
    if connectivity == GridConnectivity::EightWay {
        neighbors.extend([
            (cell.0 + 1, cell.1 + 1),
            (cell.0 + 1, cell.1 - 1),
            (cell.0 - 1, cell.1 + 1),
            (cell.0 - 1, cell.1 - 1),
        ]);
    }
    neighbors
}

fn transform_cells(cells: &[GridCell], transform: &str, origin: &GridCell) -> Vec<GridCell> {
    let mut transformed = cells
        .iter()
        .map(|cell| {
            let (x, y) = transform_cell(cell.x, cell.y, transform);
            GridCell { x, y }
        })
        .collect::<Vec<_>>();
    let min_x = transformed.iter().map(|cell| cell.x).min().unwrap_or(0);
    let min_y = transformed.iter().map(|cell| cell.y).min().unwrap_or(0);
    for cell in &mut transformed {
        cell.x = cell.x - min_x + origin.x;
        cell.y = cell.y - min_y + origin.y;
    }
    transformed
}

fn transform_cell(x: i32, y: i32, transform: &str) -> (i32, i32) {
    match transform {
        "rotate90" => (-y, x),
        "rotate180" => (-x, -y),
        "rotate270" => (y, -x),
        "mirrorX" => (-x, y),
        "mirrorY" => (x, -y),
        _ => (x, y),
    }
}
