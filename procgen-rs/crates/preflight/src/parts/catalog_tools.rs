fn build_catalog_inspect_command(args: BuildCatalogInspectArgs) -> Result<(), String> {
    let catalog: ShapeCatalog = read_json(&args.catalog)?;
    let report = inspect_shape_catalog(&catalog, &args.catalog);
    write_json(&args.out, &report)
}

fn inspect_shape_catalog(catalog: &ShapeCatalog, catalog_path: &Path) -> CatalogInspectionReport {
    let mut piece_kinds = BTreeSet::new();
    let mut feature_sockets = BTreeSet::new();
    let mut exit_directions = BTreeSet::new();
    let mut transforms = BTreeSet::new();
    let mut diagnostics = Vec::new();
    let mut seen_shapes = BTreeSet::new();
    let mut shapes = Vec::new();

    if catalog.kind != "asha_procgen.shape_catalog.v1" {
        diagnostics.push(fatal(
            "catalog_kind_invalid",
            None,
            None,
            format!("Expected asha_procgen.shape_catalog.v1, got {}.", catalog.kind),
        ));
    }
    if catalog.cell_size <= 0 {
        diagnostics.push(fatal(
            "catalog_cell_size_invalid",
            None,
            None,
            "Catalog cellSize must be positive.",
        ));
    }

    for shape in &catalog.shapes {
        if !seen_shapes.insert(shape.shape_id.as_str()) {
            diagnostics.push(fatal(
                "catalog_shape_duplicate",
                None,
                None,
                format!("Duplicate shape id {}.", shape.shape_id),
            ));
        }
        if shape.piece_kinds.is_empty() {
            diagnostics.push(fatal(
                "catalog_shape_piece_kind_missing",
                None,
                None,
                format!("Shape {} has no piece kinds.", shape.shape_id),
            ));
        }
        if shape.footprint.is_empty() {
            diagnostics.push(fatal(
                "catalog_shape_footprint_missing",
                None,
                None,
                format!("Shape {} has no footprint cells.", shape.shape_id),
            ));
        }
        if shape.exits.is_empty() {
            diagnostics.push(fatal(
                "catalog_shape_exit_missing",
                None,
                None,
                format!("Shape {} has no exits.", shape.shape_id),
            ));
        }
        if shape.allowed_transforms.is_empty() {
            diagnostics.push(fatal(
                "catalog_shape_transform_missing",
                None,
                None,
                format!("Shape {} has no allowed transforms.", shape.shape_id),
            ));
        }

        piece_kinds.extend(shape.piece_kinds.iter().cloned());
        transforms.extend(shape.allowed_transforms.iter().cloned());
        exit_directions.extend(shape.exits.iter().map(|exit| exit.direction.clone()));
        feature_sockets.extend(
            shape
                .feature_sockets
                .iter()
                .map(|socket| socket.kind.clone()),
        );
        shapes.push(CatalogShapeSummary {
            shape_id: shape.shape_id.clone(),
            piece_kinds: shape.piece_kinds.clone(),
            footprint_cells: shape.footprint.len(),
            reserved_cells: shape.reserved_cells.len(),
            exit_count: shape.exits.len(),
            feature_socket_kinds: dedupe_strings(
                shape
                    .feature_sockets
                    .iter()
                    .map(|socket| socket.kind.clone())
                    .collect(),
            ),
            allowed_transforms: shape.allowed_transforms.clone(),
            tags: shape.tags.clone(),
        });
    }

    CatalogInspectionReport {
        kind: "asha_procgen.catalog_inspection.v1".to_owned(),
        schema_version: 1,
        catalog_id: catalog.catalog_id.clone(),
        catalog_ref: display_path(catalog_path),
        shape_count: catalog.shapes.len(),
        piece_kinds: piece_kinds.into_iter().collect(),
        feature_sockets: feature_sockets.into_iter().collect(),
        exit_directions: exit_directions.into_iter().collect(),
        transforms: transforms.into_iter().collect(),
        shapes,
        diagnostics,
    }
}
