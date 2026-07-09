#[cfg(test)]
mod tests {
    use super::*;

    fn test_intent(id: &str) -> SeedIntent {
        SeedIntent {
            kind: "asha_procgen.seed_intent.v1".to_owned(),
            id: id.to_owned(),
            title: "Test".to_owned(),
            target_dimension: "topology_graph".to_owned(),
            desired_patterns: Vec::new(),
            notes: Vec::new(),
        }
    }

    #[test]
    fn rejects_private_engine_paths() {
        let private_path = format!("{}/{}/{}", "../asha-engine", "engine-rs", "crates/state");
        let error = reject_private_engine_path("demo", private_path.as_str())
            .expect_err("private engine path should be rejected");
        assert!(error.contains("private ASHA internals"));
    }

    #[test]
    fn geometry_2d_contract_round_trips_minimal_layout() {
        let geometry = Geometry2dArtifact {
            kind: "asha_procgen.geometry_2d.v1".to_owned(),
            schema_version: 1,
            geometry_id: "geometry.test.1".to_owned(),
            candidate_id: "candidate.test.1".to_owned(),
            seed: 99,
            source_candidate_ref: "artifacts/test/candidate.json".to_owned(),
            source_intermediate_ref: "artifacts/test/intermediate-breakdown.json".to_owned(),
            bounds: GeometryBounds {
                width: 480,
                height: 320,
                grid: 8,
            },
            rooms: vec![GeometryRoom {
                id: "room.start".to_owned(),
                source_region: "region.start".to_owned(),
                source_nodes: vec!["start".to_owned()],
                role: "start".to_owned(),
                geometry_role: "entry".to_owned(),
                footprint_class: "marker_room".to_owned(),
                rect: GeometryRect {
                    x: 32,
                    y: 48,
                    width: 96,
                    height: 72,
                },
                style_tags: vec!["entry".to_owned()],
            }],
            corridors: vec![GeometryCorridor {
                id: "corridor.start.goal".to_owned(),
                source_connector: "connector.edge_start_goal".to_owned(),
                source_edge: "edge.start.goal".to_owned(),
                from_room: "room.start".to_owned(),
                to_room: "room.goal".to_owned(),
                traversal_hint: "open".to_owned(),
                semantic_tags: vec!["corridor".to_owned()],
                width: 16,
                points: vec![
                    GeometryPoint { x: 128, y: 84 },
                    GeometryPoint { x: 240, y: 84 },
                ],
            }],
            contents: vec![GeometryContent {
                id: "content.start.marker".to_owned(),
                room_id: "room.start".to_owned(),
                source_ref: "start".to_owned(),
                kind: "marker".to_owned(),
                label: "Start".to_owned(),
                tags: vec!["entry".to_owned()],
            }],
            skipped_connectors: Vec::new(),
        };
        let encoded = serde_json::to_string(&geometry).expect("geometry should encode");
        let decoded: Geometry2dArtifact =
            serde_json::from_str(&encoded).expect("geometry should decode");
        assert_eq!(decoded.kind, "asha_procgen.geometry_2d.v1");
        assert_eq!(decoded.rooms[0].rect.width, 96);
        assert_eq!(decoded.corridors[0].points.len(), 2);

        let preview = HtmlPreviewArtifact {
            kind: "asha_procgen.html_preview.v1".to_owned(),
            schema_version: 1,
            preview_id: "preview.test.1".to_owned(),
            geometry_ref: "artifacts/test/geometry.json".to_owned(),
            validation_ref: "artifacts/test/geometry.validation.json".to_owned(),
            html_ref: "artifacts/test/preview.html".to_owned(),
            screenshot_hint: None,
        };
        let encoded = serde_json::to_string(&preview).expect("preview should encode");
        let decoded: HtmlPreviewArtifact =
            serde_json::from_str(&encoded).expect("preview should decode");
        assert_eq!(decoded.kind, "asha_procgen.html_preview.v1");
        assert_eq!(decoded.html_ref, "artifacts/test/preview.html");
    }

    #[test]
    fn validates_lock_key_loop() {
        let intent = SeedIntent {
            kind: "asha_procgen.seed_intent.v1".to_owned(),
            id: "test".to_owned(),
            title: "Test".to_owned(),
            target_dimension: "topology_graph".to_owned(),
            desired_patterns: Vec::new(),
            notes: Vec::new(),
        };
        let mut candidate = create_initial_candidate(&intent, 7);
        assert!(apply_graph_rule(&mut candidate, GraphRule::LockKeyLoop, 8).is_empty());
        let report = validate_graph(&candidate);
        assert!(report.ok, "{report:?}");
    }

    #[test]
    fn validates_v2_graph_grammar_rules() {
        let intent = SeedIntent {
            kind: "asha_procgen.seed_intent.v1".to_owned(),
            id: "v2".to_owned(),
            title: "V2".to_owned(),
            target_dimension: "topology_graph".to_owned(),
            desired_patterns: Vec::new(),
            notes: Vec::new(),
        };
        let mut candidate = create_initial_candidate(&intent, 11);
        for (index, rule) in [
            GraphRule::LockKeyLoop,
            GraphRule::HubSpokeCluster,
            GraphRule::NestedLockKeyChain,
            GraphRule::HazardResourceTradeoff,
            GraphRule::BossPreparationLoop,
            GraphRule::GatedTreasureBranch,
            GraphRule::BranchMergeShortcut,
        ]
        .into_iter()
        .enumerate()
        {
            let diagnostics = apply_graph_rule(&mut candidate, rule, 12 + index as u64);
            assert!(diagnostics.is_empty(), "{rule:?} rejected: {diagnostics:?}");
        }
        let report = validate_graph(&candidate);
        assert!(report.ok, "{report:?}");
        let score = score_graph(&candidate);
        assert_eq!(score.metrics.get("hubCount"), Some(&1.0));
        assert_eq!(score.metrics.get("bossCount"), Some(&1.0));
        assert!(
            score
                .metrics
                .get("pressureEdgeCount")
                .copied()
                .unwrap_or(0.0)
                >= 2.0
        );
    }

    #[test]
    fn rule_metadata_includes_v2_compatibility_hints() {
        let report = rule_metadata_report();
        assert_eq!(report.kind, "asha_procgen.rule_metadata.v1");
        let nested = report
            .rules
            .iter()
            .find(|rule| rule.id == "nested_lock_key_chain")
            .expect("nested lock metadata should exist");
        assert!(nested
            .required_patterns
            .contains(&"lock_key_loop".to_owned()));
        assert!(nested
            .compatibility_hints
            .iter()
            .any(|hint| hint.contains("lock_key_loop first")));
        let merge = report
            .rules
            .iter()
            .find(|rule| rule.id == "branch_merge_shortcut")
            .expect("merge shortcut metadata should exist");
        assert!(merge
            .duplicate_markers
            .contains(&"junction.merge_1".to_owned()));
    }

    #[test]
    fn graph_summary_reports_metrics_and_provenance_tail() {
        let intent = SeedIntent {
            kind: "asha_procgen.seed_intent.v1".to_owned(),
            id: "summary".to_owned(),
            title: "Summary".to_owned(),
            target_dimension: "topology_graph".to_owned(),
            desired_patterns: Vec::new(),
            notes: Vec::new(),
        };
        let mut candidate = create_initial_candidate(&intent, 31);
        candidate.provenance.push(ProvenanceStep {
            step: 1,
            command: "init".to_owned(),
            seed: Some(31),
            summary: "Initialized test candidate".to_owned(),
        });
        assert!(apply_graph_rule(&mut candidate, GraphRule::LockKeyLoop, 32).is_empty());
        candidate.provenance.push(ProvenanceStep {
            step: 2,
            command: "graph apply-rule lock_key_loop".to_owned(),
            seed: Some(32),
            summary: "Applied lock_key_loop".to_owned(),
        });
        let validation = validate_graph(&candidate);
        let score = score_graph(&candidate);
        let summary =
            graph_summary_report(&candidate, &validation, &score).expect("summary should encode");
        assert_eq!(summary.kind, "asha_procgen.graph_summary.v1");
        assert!(summary.validation_ok);
        assert_eq!(summary.node_count, candidate.graph.nodes.len());
        assert!(summary.locked_items.contains(&"item.gate_key_1".to_owned()));
        assert!(summary.tags.contains(&"critical".to_owned()));
        assert_eq!(summary.provenance_tail.len(), 2);
        assert!(summary.metrics.contains_key("lockedEdgeCount"));
    }

    #[test]
    fn fork_candidate_preserves_graph_and_adds_provenance() {
        let intent = SeedIntent {
            kind: "asha_procgen.seed_intent.v1".to_owned(),
            id: "fork".to_owned(),
            title: "Fork".to_owned(),
            target_dimension: "topology_graph".to_owned(),
            desired_patterns: Vec::new(),
            notes: Vec::new(),
        };
        let mut candidate = create_initial_candidate(&intent, 41);
        candidate.provenance.push(ProvenanceStep {
            step: 1,
            command: "init".to_owned(),
            seed: Some(41),
            summary: "Initialized fork source".to_owned(),
        });
        apply_graph_rule(&mut candidate, GraphRule::LockKeyLoop, 42);
        let source_id = candidate.candidate_id.clone();
        let source_graph = candidate.graph.clone();
        let forked = fork_candidate(candidate, "Boss Prep Attempt!", 77);
        assert_eq!(
            forked.candidate_id,
            format!("{source_id}.fork.boss_prep_attempt.77")
        );
        assert_eq!(forked.seed, 77);
        assert_eq!(forked.graph.nodes.len(), source_graph.nodes.len());
        assert_eq!(forked.graph.edges.len(), source_graph.edges.len());
        assert_eq!(forked.provenance.len(), 2);
        let fork_step = forked.provenance.last().expect("fork step should exist");
        assert_eq!(fork_step.command, "graph fork");
        assert_eq!(fork_step.seed, Some(77));
        assert!(fork_step.summary.contains(&source_id));
    }

    #[test]
    fn rejects_duplicate_v2_rule_with_repair_hint() {
        let intent = SeedIntent {
            kind: "asha_procgen.seed_intent.v1".to_owned(),
            id: "duplicate".to_owned(),
            title: "Duplicate".to_owned(),
            target_dimension: "topology_graph".to_owned(),
            desired_patterns: Vec::new(),
            notes: Vec::new(),
        };
        let mut candidate = create_initial_candidate(&intent, 15);
        assert!(apply_graph_rule(&mut candidate, GraphRule::HubSpokeCluster, 16).is_empty());
        let diagnostics = apply_graph_rule(&mut candidate, GraphRule::HubSpokeCluster, 17);
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "rule_already_applied" && diagnostic.repair_hint.is_some()
        }));
    }

    #[test]
    fn rejects_missing_required_item() {
        let intent = SeedIntent {
            kind: "asha_procgen.seed_intent.v1".to_owned(),
            id: "broken".to_owned(),
            title: "Broken".to_owned(),
            target_dimension: "topology_graph".to_owned(),
            desired_patterns: Vec::new(),
            notes: Vec::new(),
        };
        let mut candidate = create_initial_candidate(&intent, 9);
        candidate.graph.edges[0].required_item = Some("missing.key".to_owned());
        candidate.graph.edges[0].traversal = TraversalKind::Locked;
        let report = validate_graph(&candidate);
        assert!(!report.ok);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "required_item_unavailable"
                && diagnostic.repair_hint.is_some()));
    }

    #[test]
    fn rejects_incompatible_v2_rule_with_repair_hint() {
        let intent = SeedIntent {
            kind: "asha_procgen.seed_intent.v1".to_owned(),
            id: "incompatible".to_owned(),
            title: "Incompatible".to_owned(),
            target_dimension: "topology_graph".to_owned(),
            desired_patterns: Vec::new(),
            notes: Vec::new(),
        };
        let mut candidate = create_initial_candidate(&intent, 19);
        let diagnostics = apply_graph_rule(&mut candidate, GraphRule::NestedLockKeyChain, 20);
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "missing_required_pattern" && diagnostic.repair_hint.is_some()
        }));
    }

    #[test]
    fn validates_v2_structural_repair_hints() {
        let intent = SeedIntent {
            kind: "asha_procgen.seed_intent.v1".to_owned(),
            id: "structural".to_owned(),
            title: "Structural".to_owned(),
            target_dimension: "topology_graph".to_owned(),
            desired_patterns: Vec::new(),
            notes: Vec::new(),
        };
        let mut candidate = create_initial_candidate(&intent, 21);
        candidate.graph.nodes.extend([
            Node {
                id: "hub.broken".to_owned(),
                kind: NodeKind::Junction,
                label: "Broken Hub".to_owned(),
                tags: vec!["hub".to_owned()],
                grants_item: None,
            },
            Node {
                id: "gate.boss_broken".to_owned(),
                kind: NodeKind::Gate,
                label: "Unprepared Boss".to_owned(),
                tags: vec!["boss".to_owned()],
                grants_item: None,
            },
        ]);
        candidate.graph.edges.extend([
            Edge {
                id: "edge.start.broken_hub".to_owned(),
                from: "start".to_owned(),
                to: "hub.broken".to_owned(),
                kind: EdgeKind::OptionalBranch,
                traversal: TraversalKind::Open,
                required_item: None,
                tags: vec!["branch".to_owned()],
            },
            Edge {
                id: "edge.start.boss_broken".to_owned(),
                from: "start".to_owned(),
                to: "gate.boss_broken".to_owned(),
                kind: EdgeKind::CriticalPath,
                traversal: TraversalKind::Open,
                required_item: None,
                tags: vec!["approach".to_owned()],
            },
            Edge {
                id: "edge.boss_broken.goal".to_owned(),
                from: "gate.boss_broken".to_owned(),
                to: "goal".to_owned(),
                kind: EdgeKind::CriticalPath,
                traversal: TraversalKind::Open,
                required_item: None,
                tags: vec!["boss".to_owned()],
            },
        ]);
        let report = validate_graph(&candidate);
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "hub_missing_wayfinding_anchor" && diagnostic.repair_hint.is_some()
        }));
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "boss_missing_preparation" && diagnostic.repair_hint.is_some()
        }));
    }

    #[test]
    fn repair_report_prioritizes_missing_provider_actions() {
        let intent = SeedIntent {
            kind: "asha_procgen.seed_intent.v1".to_owned(),
            id: "repair".to_owned(),
            title: "Repair".to_owned(),
            target_dimension: "topology_graph".to_owned(),
            desired_patterns: Vec::new(),
            notes: Vec::new(),
        };
        let mut candidate = create_initial_candidate(&intent, 51);
        candidate.graph.edges[0].required_item = Some("missing.key".to_owned());
        candidate.graph.edges[0].traversal = TraversalKind::Locked;
        let report = repair_report(&candidate).expect("repair report should encode");
        assert_eq!(report.kind, "asha_procgen.repair_report.v1");
        assert!(!report.validation_ok);
        let suggestion = report
            .suggestions
            .iter()
            .find(|suggestion| suggestion.code == "required_item_unavailable")
            .expect("missing provider suggestion should exist");
        assert_eq!(suggestion.severity, Severity::Fatal);
        assert!(suggestion.repair_hint.is_some());
        assert!(suggestion
            .suggested_actions
            .iter()
            .any(|action| action.contains("provider")));
    }

    #[test]
    fn repair_mapping_covers_missing_required_pattern() {
        let diagnostic = fatal_with_hint(
            "missing_required_pattern",
            Some("gate.locked_1"),
            None,
            "nested_lock_key_chain requires an existing first lock/key loop.",
            "Apply lock_key_loop before nested_lock_key_chain.",
        );
        let suggestion = repair_suggestion_for_diagnostic(&diagnostic);
        assert_eq!(suggestion.code, "missing_required_pattern");
        assert!(suggestion
            .suggested_actions
            .iter()
            .any(|action| action.contains("prerequisite pattern")));
    }

    #[test]
    fn repair_report_maps_v2_structural_hints() {
        let intent = SeedIntent {
            kind: "asha_procgen.seed_intent.v1".to_owned(),
            id: "repair-structural".to_owned(),
            title: "Repair Structural".to_owned(),
            target_dimension: "topology_graph".to_owned(),
            desired_patterns: Vec::new(),
            notes: Vec::new(),
        };
        let mut candidate = create_initial_candidate(&intent, 52);
        candidate.graph.nodes.push(Node {
            id: "gate.boss_broken".to_owned(),
            kind: NodeKind::Gate,
            label: "Unprepared Boss".to_owned(),
            tags: vec!["boss".to_owned()],
            grants_item: None,
        });
        candidate.graph.edges.extend([
            Edge {
                id: "edge.start.boss_broken".to_owned(),
                from: "start".to_owned(),
                to: "gate.boss_broken".to_owned(),
                kind: EdgeKind::CriticalPath,
                traversal: TraversalKind::Open,
                required_item: None,
                tags: vec!["approach".to_owned()],
            },
            Edge {
                id: "edge.boss_broken.goal".to_owned(),
                from: "gate.boss_broken".to_owned(),
                to: "goal".to_owned(),
                kind: EdgeKind::CriticalPath,
                traversal: TraversalKind::Open,
                required_item: None,
                tags: vec!["boss".to_owned()],
            },
        ]);
        let report = repair_report(&candidate).expect("repair report should encode");
        let suggestion = report
            .suggestions
            .iter()
            .find(|suggestion| suggestion.code == "boss_missing_preparation")
            .expect("boss preparation suggestion should exist");
        assert!(suggestion
            .suggested_actions
            .iter()
            .any(|action| action.contains("preparation")));
    }

    #[test]
    fn graph_analysis_reports_lock_and_shortcut_signals() {
        let intent = test_intent("analysis");
        let mut candidate = create_initial_candidate(&intent, 61);
        assert!(apply_graph_rule(&mut candidate, GraphRule::LockKeyLoop, 62).is_empty());
        assert!(apply_graph_rule(&mut candidate, GraphRule::OneWayShortcut, 63).is_empty());
        let report = analyze_graph(&candidate).expect("analysis should encode");
        assert_eq!(report.kind, "asha_procgen.graph_analysis.v1");
        assert_eq!(
            report.critical_path.first().map(String::as_str),
            Some("start")
        );
        assert_eq!(
            report.critical_path.last().map(String::as_str),
            Some("goal")
        );
        assert!(report
            .lock_key_order
            .iter()
            .any(|entry| entry.required_item == "item.gate_key_1"
                && entry.provider_reachable_before_lock));
        assert!(report
            .loop_signals
            .iter()
            .any(|signal| signal.signal == "shortcut_loop"));
        assert!(report
            .shortcut_bypass_risks
            .iter()
            .any(|risk| risk.risk == "may_bypass_lock"));
    }

    #[test]
    fn compatible_rules_reports_blocked_duplicate_and_risky() {
        let intent = test_intent("compatibility");
        let mut candidate = create_initial_candidate(&intent, 71);
        let initial = compatible_rules_report(&candidate).expect("compatibility report");
        let nested = initial
            .rules
            .iter()
            .find(|rule| rule.rule == "nested_lock_key_chain")
            .expect("nested rule should be present");
        assert_eq!(nested.status, "blocked");
        assert!(apply_graph_rule(&mut candidate, GraphRule::LockKeyLoop, 72).is_empty());
        assert!(apply_graph_rule(&mut candidate, GraphRule::OneWayShortcut, 73).is_empty());
        let report = compatible_rules_report(&candidate).expect("compatibility report");
        assert_eq!(
            report
                .rules
                .iter()
                .find(|rule| rule.rule == "lock_key_loop")
                .map(|rule| rule.status.as_str()),
            Some("duplicate")
        );
        assert_eq!(
            report
                .rules
                .iter()
                .find(|rule| rule.rule == "one_way_shortcut")
                .map(|rule| rule.status.as_str()),
            Some("duplicate")
        );
        assert_eq!(
            report
                .rules
                .iter()
                .find(|rule| rule.rule == "secret_bypass")
                .map(|rule| rule.status.as_str()),
            Some("risky")
        );
    }

    #[test]
    fn repair_apply_adds_rejoin_and_refuses_ambiguous_target() {
        let intent = test_intent("repair-apply");
        let mut candidate = create_initial_candidate(&intent, 81);
        candidate.graph.nodes.push(Node {
            id: "treasure.loose".to_owned(),
            kind: NodeKind::Treasure,
            label: "Loose Treasure".to_owned(),
            tags: vec!["optional".to_owned(), "reward".to_owned()],
            grants_item: None,
        });
        candidate.graph.edges.push(Edge {
            id: "edge.start.loose".to_owned(),
            from: "start".to_owned(),
            to: "treasure.loose".to_owned(),
            kind: EdgeKind::OptionalBranch,
            traversal: TraversalKind::Open,
            required_item: None,
            tags: vec!["branch".to_owned()],
        });
        let diagnostics = apply_repair_action(
            &mut candidate,
            RepairAction::AddRejoinEdge,
            Some("treasure.loose"),
            82,
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert!(candidate
            .graph
            .edges
            .iter()
            .any(|edge| edge.from == "treasure.loose"
                && edge.to == "goal"
                && edge_has_tag(edge, "repair")));
        let rejected = apply_repair_action(
            &mut candidate,
            RepairAction::AddRejoinEdge,
            Some("start"),
            83,
        );
        assert!(rejected
            .iter()
            .any(|diagnostic| diagnostic.code == "repair_target_ambiguous"));
    }

    #[test]
    fn repair_apply_removes_orphan_node() {
        let intent = test_intent("repair-orphan");
        let mut candidate = create_initial_candidate(&intent, 84);
        candidate.graph.nodes.push(Node {
            id: "secret.orphan".to_owned(),
            kind: NodeKind::Secret,
            label: "Orphan Secret".to_owned(),
            tags: vec!["secret".to_owned()],
            grants_item: None,
        });
        let diagnostics = apply_repair_action(
            &mut candidate,
            RepairAction::RemoveOrphanNode,
            Some("secret.orphan"),
            85,
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert!(!has_node(&candidate, "secret.orphan"));
    }

    #[test]
    fn topology_fingerprint_is_stable_and_budget_checks_fail_cleanly() {
        let intent = test_intent("fingerprint");
        let mut left = create_initial_candidate(&intent, 91);
        let mut right = create_initial_candidate(&intent, 92);
        assert!(apply_graph_rule(&mut left, GraphRule::LockKeyLoop, 93).is_empty());
        assert!(apply_graph_rule(&mut left, GraphRule::OptionalTreasureDetour, 94).is_empty());
        assert!(apply_graph_rule(&mut right, GraphRule::LockKeyLoop, 95).is_empty());
        assert!(apply_graph_rule(&mut right, GraphRule::OptionalTreasureDetour, 96).is_empty());
        assert_eq!(topology_fingerprint(&left), topology_fingerprint(&right));
        let budgets = IntentBudget {
            require_hub: Some(true),
            min_optional_branches: Some(3),
            max_dead_ends: Some(0),
            ..IntentBudget::default()
        };
        let checks = budget_checks(Some(&budgets), &score_graph(&left), &left);
        assert!(checks
            .iter()
            .any(|check| check.code == "require_hub" && !check.ok));
        assert!(checks
            .iter()
            .any(|check| check.code == "max_dead_ends" && check.ok));
    }

    #[test]
    fn spatial_intent_annotation_marks_core_intents() {
        let intent = test_intent("spatial");
        let mut candidate = create_initial_candidate(&intent, 101);
        for (index, rule) in [
            GraphRule::LockKeyLoop,
            GraphRule::HubSpokeCluster,
            GraphRule::HazardResourceTradeoff,
            GraphRule::OneWayShortcut,
        ]
        .into_iter()
        .enumerate()
        {
            assert!(apply_graph_rule(&mut candidate, rule, 102 + index as u64).is_empty());
        }
        let report = spatial_intent_report(&candidate, None).expect("spatial intent report");
        assert!(report.annotations.iter().any(|annotation| {
            annotation.target_id == "hub.central_1"
                && annotation.intents.contains(&"landmark_hub".to_owned())
        }));
        assert!(report.annotations.iter().any(|annotation| {
            annotation.target_id == "edge.gate_1.goal"
                && annotation
                    .intents
                    .contains(&"visible_before_reachable".to_owned())
        }));
        assert!(report.annotations.iter().any(|annotation| {
            annotation
                .intents
                .contains(&"shortcut_connector".to_owned())
        }));
        assert!(report
            .annotations
            .iter()
            .any(|annotation| { annotation.intents.contains(&"pressure_path".to_owned()) }));
    }

    #[test]
    fn intermediate_breakdown_validates_and_catches_invalid_cases() {
        let intent = test_intent("breakdown");
        let mut candidate = create_initial_candidate(&intent, 111);
        assert!(apply_graph_rule(&mut candidate, GraphRule::LockKeyLoop, 112).is_empty());
        assert!(apply_graph_rule(&mut candidate, GraphRule::HubSpokeCluster, 113).is_empty());
        let annotations = spatial_intent_report(&candidate, None).expect("spatial intent report");
        let mut breakdown = intermediate_breakdown(
            &candidate,
            &annotations,
            Path::new("artifacts/test/spatial-intent.json"),
        )
        .expect("breakdown should encode");
        let report = validate_intermediate_breakdown(&breakdown);
        assert!(report.ok, "{report:?}");
        breakdown.regions.retain(|region| region.role != "goal");
        let missing_goal = validate_intermediate_breakdown(&breakdown);
        assert!(missing_goal.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "intermediate_goal_missing" && diagnostic.severity == Severity::Fatal
        }));
        let connector = breakdown
            .connectors
            .first_mut()
            .expect("connector should exist");
        connector.to_region = "region.missing".to_owned();
        connector.intents.push("vertical_candidate".to_owned());
        let invalid_connector = validate_intermediate_breakdown(&breakdown);
        assert!(invalid_connector
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == "intermediate_connector_endpoint_missing" }));
        assert!(invalid_connector.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "intermediate_vertical_candidate_unsupported"
        }));
    }

    #[test]
    fn intermediate_breakdown_emits_geometry_prep_hints() {
        let intent = test_intent("geometry-prep");
        let mut candidate = create_initial_candidate(&intent, 121);
        for (index, rule) in [
            GraphRule::LockKeyLoop,
            GraphRule::HubSpokeCluster,
            GraphRule::HazardResourceTradeoff,
            GraphRule::GatedTreasureBranch,
            GraphRule::BranchMergeShortcut,
        ]
        .into_iter()
        .enumerate()
        {
            assert!(apply_graph_rule(&mut candidate, rule, 122 + index as u64).is_empty());
        }
        let annotations = spatial_intent_report(&candidate, None).expect("spatial intent report");
        let breakdown = intermediate_breakdown(
            &candidate,
            &annotations,
            Path::new("artifacts/test/spatial-intent.json"),
        )
        .expect("breakdown should encode");
        assert_eq!(breakdown.schema_version, 2);
        let hub = breakdown
            .regions
            .iter()
            .find(|region| region.node_ids == vec!["hub.central_1".to_owned()])
            .expect("hub region should exist");
        assert_eq!(hub.geometry_role, "landmark_junction");
        assert_eq!(hub.footprint_class, "hub");
        assert_eq!(hub.scale_band, "large");
        assert_eq!(hub.anchor_quality, "explicit");
        assert!(hub
            .entrance_expectations
            .contains(&"multi_spoke_orientation".to_owned()));

        let gate = breakdown
            .regions
            .iter()
            .find(|region| region.node_ids == vec!["gate.locked_1".to_owned()])
            .expect("gate region should exist");
        assert_eq!(gate.geometry_role, "threshold");
        assert!(gate
            .entrance_expectations
            .contains(&"locked_threshold_preview".to_owned()));

        let hazard = breakdown
            .regions
            .iter()
            .find(|region| region.node_ids == vec!["hazard.sluice_1".to_owned()])
            .expect("hazard region should exist");
        assert_eq!(hazard.footprint_class, "pressure_lane");
        assert!(hazard
            .entrance_expectations
            .contains(&"readable_hazard_approach".to_owned()));

        let reward = breakdown
            .regions
            .iter()
            .find(|region| region.node_ids == vec!["treasure.gated_1".to_owned()])
            .expect("reward region should exist");
        assert_eq!(reward.geometry_role, "reward_pocket");
        assert_eq!(reward.scale_band, "small");

        let locked_connector = breakdown
            .connectors
            .iter()
            .find(|connector| connector.edge_id == "edge.gate_1.goal")
            .expect("locked connector should exist");
        assert_eq!(locked_connector.traversal_hint, "locked");
        assert!(locked_connector
            .affordances
            .contains(&"locked_threshold".to_owned()));
        assert!(locked_connector
            .constraint_refs
            .iter()
            .any(|reference| reference.contains("preserve_lock_preview")));

        let shortcut_connector = breakdown
            .connectors
            .iter()
            .find(|connector| connector.edge_id == "edge.merge_1.goal.shortcut")
            .expect("shortcut connector should exist");
        assert!(shortcut_connector
            .affordances
            .contains(&"shortcut_link".to_owned()));
        assert!(shortcut_connector
            .constraint_refs
            .iter()
            .any(|reference| reference.contains("preserve_shortcut_connector")));

        assert!(breakdown.constraints.iter().any(|constraint| {
            constraint.target_type == "edge"
                && constraint
                    .graph_refs
                    .contains(&"edge.gate_1.goal".to_owned())
                && constraint
                    .source_intents
                    .contains(&"visible_before_reachable".to_owned())
        }));
    }

    #[test]
    fn intermediate_validation_catches_geometry_prep_gaps() {
        let intent = test_intent("geometry-prep-validation");
        let mut candidate = create_initial_candidate(&intent, 131);
        for (index, rule) in [
            GraphRule::LockKeyLoop,
            GraphRule::HubSpokeCluster,
            GraphRule::GatedTreasureBranch,
            GraphRule::BranchMergeShortcut,
            GraphRule::SecretBypass,
        ]
        .into_iter()
        .enumerate()
        {
            assert!(apply_graph_rule(&mut candidate, rule, 132 + index as u64).is_empty());
        }
        let annotations = spatial_intent_report(&candidate, None).expect("spatial intent report");
        let breakdown = intermediate_breakdown(
            &candidate,
            &annotations,
            Path::new("artifacts/test/spatial-intent.json"),
        )
        .expect("breakdown should encode");
        let valid = validate_intermediate_breakdown(&breakdown);
        assert!(valid.ok, "{valid:?}");

        let mut missing_affordance = breakdown.clone();
        missing_affordance
            .connectors
            .first_mut()
            .expect("connector should exist")
            .affordances
            .clear();
        let report = validate_intermediate_breakdown(&missing_affordance);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == "intermediate_connector_affordance_missing" }));

        let mut missing_gated_constraint = breakdown.clone();
        let locked = missing_gated_constraint
            .connectors
            .iter_mut()
            .find(|connector| connector.edge_id == "edge.gate_1.goal")
            .expect("locked connector should exist");
        locked.constraint_refs.clear();
        let report = validate_intermediate_breakdown(&missing_gated_constraint);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "intermediate_gated_constraint_missing"));

        let mut missing_shortcut_affordance = breakdown.clone();
        let shortcut = missing_shortcut_affordance
            .connectors
            .iter_mut()
            .find(|connector| connector.edge_id == "edge.merge_1.goal.shortcut")
            .expect("shortcut connector should exist");
        shortcut
            .affordances
            .retain(|affordance| affordance != "shortcut_link");
        let report = validate_intermediate_breakdown(&missing_shortcut_affordance);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == "intermediate_shortcut_affordance_missing" }));

        let mut missing_region_prep = breakdown.clone();
        missing_region_prep
            .regions
            .first_mut()
            .expect("region should exist")
            .geometry_role
            .clear();
        let report = validate_intermediate_breakdown(&missing_region_prep);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == "intermediate_region_geometry_prep_missing" }));

        let mut unsupported_3d = breakdown;
        unsupported_3d
            .connectors
            .first_mut()
            .expect("connector should exist")
            .affordances
            .push("vertical_shaft".to_owned());
        let report = validate_intermediate_breakdown(&unsupported_3d);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "intermediate_3d_claim_unsupported"));
    }

    #[test]
    fn geometry_emit_2d_places_variable_non_overlapping_rooms() {
        let intent = test_intent("geometry-emit");
        let mut candidate = create_initial_candidate(&intent, 141);
        for (index, rule) in [
            GraphRule::LockKeyLoop,
            GraphRule::HubSpokeCluster,
            GraphRule::HazardResourceTradeoff,
            GraphRule::BossPreparationLoop,
            GraphRule::GatedTreasureBranch,
            GraphRule::BranchMergeShortcut,
        ]
        .into_iter()
        .enumerate()
        {
            assert!(apply_graph_rule(&mut candidate, rule, 142 + index as u64).is_empty());
        }
        let annotations = spatial_intent_report(&candidate, None).expect("spatial intent report");
        let intermediate = intermediate_breakdown(
            &candidate,
            &annotations,
            Path::new("artifacts/test/spatial-intent.json"),
        )
        .expect("breakdown should encode");
        let args = GeometryEmit2dArgs {
            candidate: PathBuf::from("artifacts/test/candidate.json"),
            intermediate: PathBuf::from("artifacts/test/intermediate-breakdown.json"),
            seed: 150,
            out: PathBuf::from("artifacts/test/geometry.json"),
        };
        let geometry =
            emit_geometry_2d(&candidate, &intermediate, &args, 150).expect("geometry should emit");
        assert_eq!(geometry.kind, "asha_procgen.geometry_2d.v1");
        assert_eq!(geometry.rooms.len(), intermediate.regions.len());
        assert_eq!(geometry.corridors.len(), intermediate.connectors.len());
        assert_eq!(geometry.skipped_connectors.len(), 0);
        assert!(geometry.bounds.width > 640);
        let hub = geometry
            .rooms
            .iter()
            .find(|room| room.source_region == "region.hub_central_1")
            .expect("hub room should exist");
        let reward = geometry
            .rooms
            .iter()
            .find(|room| room.source_region == "region.treasure_gated_1")
            .expect("reward room should exist");
        let hazard = geometry
            .rooms
            .iter()
            .find(|room| room.source_region == "region.hazard_sluice_1")
            .expect("hazard room should exist");
        let gate = geometry
            .rooms
            .iter()
            .find(|room| room.source_region == "region.gate_locked_1")
            .expect("gate room should exist");
        let start = geometry
            .rooms
            .iter()
            .find(|room| room.source_region == "region.start")
            .expect("start room should exist");
        let goal = geometry
            .rooms
            .iter()
            .find(|room| room.source_region == "region.goal")
            .expect("goal room should exist");
        assert!(hub.rect.width > reward.rect.width);
        assert!(hazard.rect.width > reward.rect.width);
        assert!(gate.style_tags.contains(&"threshold".to_owned()));
        assert!(start.style_tags.contains(&"entry".to_owned()));
        assert!(goal.style_tags.contains(&"destination".to_owned()));
        for (index, left) in geometry.rooms.iter().enumerate() {
            for right in geometry.rooms.iter().skip(index + 1) {
                assert!(
                    !rectangles_overlap(&left.rect, &right.rect),
                    "{} overlaps {}",
                    left.id,
                    right.id
                );
            }
        }
    }

    #[test]
    fn geometry_emit_2d_routes_semantic_corridors() {
        let intent = test_intent("geometry-corridors");
        let mut candidate = create_initial_candidate(&intent, 251);
        for (index, rule) in [
            GraphRule::LockKeyLoop,
            GraphRule::HubSpokeCluster,
            GraphRule::HazardResourceTradeoff,
            GraphRule::GatedTreasureBranch,
            GraphRule::BranchMergeShortcut,
            GraphRule::OneWayShortcut,
            GraphRule::SecretBypass,
        ]
        .into_iter()
        .enumerate()
        {
            assert!(apply_graph_rule(&mut candidate, rule, 252 + index as u64).is_empty());
        }
        let annotations = spatial_intent_report(&candidate, None).expect("spatial intent report");
        let intermediate = intermediate_breakdown(
            &candidate,
            &annotations,
            Path::new("artifacts/test/spatial-intent.json"),
        )
        .expect("breakdown should encode");
        let args = GeometryEmit2dArgs {
            candidate: PathBuf::from("artifacts/test/candidate.json"),
            intermediate: PathBuf::from("artifacts/test/intermediate-breakdown.json"),
            seed: 253,
            out: PathBuf::from("artifacts/test/geometry.json"),
        };
        let geometry =
            emit_geometry_2d(&candidate, &intermediate, &args, 253).expect("geometry should emit");
        assert_eq!(geometry.corridors.len(), intermediate.connectors.len());
        assert!(geometry.skipped_connectors.is_empty());

        let locked = geometry
            .corridors
            .iter()
            .find(|corridor| corridor.source_edge == "edge.gate_1.goal")
            .expect("locked threshold corridor should exist");
        assert_eq!(locked.width, 18);
        assert!(locked
            .semantic_tags
            .contains(&"locked_threshold".to_owned()));

        let secret = geometry
            .corridors
            .iter()
            .find(|corridor| corridor.source_edge == "edge.start.secret_1")
            .expect("secret corridor should exist");
        assert_eq!(secret.width, 10);
        assert!(secret.semantic_tags.contains(&"hidden_route".to_owned()));

        let shortcut = geometry
            .corridors
            .iter()
            .find(|corridor| corridor.source_edge == "edge.merge_1.goal.shortcut")
            .expect("shortcut corridor should exist");
        assert!(shortcut.semantic_tags.contains(&"shortcut_link".to_owned()));

        let pressure = geometry
            .corridors
            .iter()
            .find(|corridor| corridor.source_edge == "edge.start.sluice_1")
            .expect("pressure route corridor should exist");
        assert_eq!(pressure.width, 20);

        let rooms_by_id = geometry
            .rooms
            .iter()
            .map(|room| (room.id.as_str(), room))
            .collect::<BTreeMap<_, _>>();
        for corridor in &geometry.corridors {
            let first = corridor
                .points
                .first()
                .expect("corridor should have a start point");
            let last = corridor
                .points
                .last()
                .expect("corridor should have an end point");
            let from_room = rooms_by_id
                .get(corridor.from_room.as_str())
                .expect("corridor from room should resolve");
            let to_room = rooms_by_id
                .get(corridor.to_room.as_str())
                .expect("corridor to room should resolve");
            assert!(
                point_on_rect_boundary(first, &from_room.rect),
                "{} does not start on {}",
                corridor.id,
                from_room.id
            );
            assert!(
                point_on_rect_boundary(last, &to_room.rect),
                "{} does not end on {}",
                corridor.id,
                to_room.id
            );
        }
    }

    #[test]
    fn geometry_emit_2d_annotates_room_contents() {
        let intent = test_intent("geometry-contents");
        let mut candidate = create_initial_candidate(&intent, 351);
        for (index, rule) in [
            GraphRule::LockKeyLoop,
            GraphRule::HubSpokeCluster,
            GraphRule::HazardResourceTradeoff,
            GraphRule::BossPreparationLoop,
            GraphRule::GatedTreasureBranch,
            GraphRule::BranchMergeShortcut,
            GraphRule::OneWayShortcut,
            GraphRule::SecretBypass,
        ]
        .into_iter()
        .enumerate()
        {
            assert!(apply_graph_rule(&mut candidate, rule, 352 + index as u64).is_empty());
        }
        let annotations = spatial_intent_report(&candidate, None).expect("spatial intent report");
        let intermediate = intermediate_breakdown(
            &candidate,
            &annotations,
            Path::new("artifacts/test/spatial-intent.json"),
        )
        .expect("breakdown should encode");
        let args = GeometryEmit2dArgs {
            candidate: PathBuf::from("artifacts/test/candidate.json"),
            intermediate: PathBuf::from("artifacts/test/intermediate-breakdown.json"),
            seed: 353,
            out: PathBuf::from("artifacts/test/geometry.json"),
        };
        let geometry =
            emit_geometry_2d(&candidate, &intermediate, &args, 353).expect("geometry should emit");
        let content_kinds = geometry
            .contents
            .iter()
            .map(|content| content.kind.as_str())
            .collect::<BTreeSet<_>>();
        for expected in [
            "key_pickup",
            "locked_gate",
            "hazard",
            "reward_cache",
            "boss_threshold",
            "shortcut_marker",
            "secret_route_marker",
        ] {
            assert!(
                content_kinds.contains(expected),
                "{expected} content missing"
            );
        }
        for content in &geometry.contents {
            assert!(!content.label.is_empty());
            assert!(content.source_ref.contains("node:"));
            assert!(content.source_ref.contains("region:"));
            assert!(geometry.rooms.iter().any(|room| room.id == content.room_id));
            assert!(
                content.tags.contains(&content.kind),
                "{} tags should include kind",
                content.id
            );
        }
    }

    #[test]
    fn geometry_validate_2d_accepts_valid_full_stack_geometry() {
        let geometry = full_stack_geometry_fixture(451);
        let report = validate_geometry_2d(&geometry);
        assert!(report.ok, "{:?}", report.diagnostics);
        assert_eq!(report.kind, "asha_procgen.validation.geometry_2d.v1");
    }

    #[test]
    fn geometry_validate_2d_catches_invalid_cases() {
        let geometry = full_stack_geometry_fixture(551);

        let mut overlapping = geometry.clone();
        overlapping.rooms[1].rect = overlapping.rooms[0].rect.clone();
        let report = validate_geometry_2d(&overlapping);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "geometry_room_overlap"));

        let mut missing_corridors = geometry.clone();
        missing_corridors.corridors.clear();
        let report = validate_geometry_2d(&missing_corridors);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "geometry_connector_coverage_missing"));

        let mut bad_content_anchor = geometry.clone();
        bad_content_anchor
            .contents
            .first_mut()
            .expect("content should exist")
            .room_id = "room.missing".to_owned();
        let report = validate_geometry_2d(&bad_content_anchor);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "geometry_content_room_missing"));

        let mut unreachable_goal = geometry;
        let goal_room_id = unreachable_goal
            .rooms
            .iter()
            .find(|room| room.role == "goal")
            .expect("goal room should exist")
            .id
            .clone();
        unreachable_goal
            .corridors
            .retain(|corridor| corridor.to_room != goal_room_id);
        let report = validate_geometry_2d(&unreachable_goal);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "geometry_goal_unreachable"));
    }

    #[test]
    fn preview_html_renders_required_sections() {
        let geometry = full_stack_geometry_fixture(651);
        let validation = validate_geometry_2d(&geometry);
        assert!(validation.ok, "{:?}", validation.diagnostics);
        let html = render_geometry_preview_html(
            &geometry,
            &validation,
            "artifacts/test/geometry.json",
            "artifacts/test/geometry.validation.json",
        );
        for expected in [
            "<!doctype html>",
            "data-preview-kind=\"asha_procgen.html_preview.v1\"",
            "<svg",
            "<polyline",
            "<rect",
            "Dungeon Preview",
            "Validation: ok",
            "Key Pickup",
            "Boss Threshold",
            "Reward Cache",
            "Secret Route",
            "Shortcut Marker",
        ] {
            assert!(html.contains(expected), "{expected} missing");
        }
    }

    #[test]
    fn preview_html_refuses_invalid_geometry_without_override() {
        let mut geometry = full_stack_geometry_fixture(751);
        geometry.rooms[1].rect = geometry.rooms[0].rect.clone();
        let validation = validate_geometry_2d(&geometry);
        assert!(!validation.ok);
        let error = validate_preview_inputs(&geometry, &validation, false)
            .expect_err("invalid geometry should need explicit preview override");
        assert!(error.contains("--allow-invalid"));
        validate_preview_inputs(&geometry, &validation, true)
            .expect("allow-invalid should render diagnostics");
        let html = render_geometry_preview_html(
            &geometry,
            &validation,
            "artifacts/test/geometry.json",
            "artifacts/test/geometry.validation.json",
        );
        assert!(html.contains("Validation: invalid"));
        assert!(html.contains("geometry_room_overlap"));
    }

    #[test]
    fn piece_plan_emits_explicit_room_corridor_and_semantic_requirements() {
        let (candidate, intermediate, geometry) = full_stack_geometry_inputs(851);
        let args = BuildEmitPiecePlanArgs {
            candidate: PathBuf::from("artifacts/test/candidate.json"),
            intermediate: PathBuf::from("artifacts/test/intermediate-breakdown.json"),
            geometry: PathBuf::from("artifacts/test/geometry.json"),
            out: PathBuf::from("artifacts/test/piece-plan.json"),
        };
        let plan = emit_piece_build_plan(&candidate, &intermediate, &geometry, &args)
            .expect("piece plan should emit");

        assert_eq!(plan.kind, "asha_procgen.piece_build_plan.v1");
        assert_eq!(plan.candidate_id, candidate.candidate_id);
        assert_eq!(plan.geometry_id, geometry.geometry_id);
        assert!(plan.requirements.len() > geometry.rooms.len() + geometry.corridors.len() * 2);
        assert!(plan.links.len() > geometry.corridors.len());

        let requirement_kinds = plan
            .requirements
            .iter()
            .map(|requirement| requirement.kind.as_str())
            .collect::<BTreeSet<_>>();
        for required in [
            "room",
            "threshold",
            "key",
            "hazard",
            "reward",
            "boss",
            "secret",
            "shortcut",
            "resource",
            "connector",
            "corridor",
            "bend",
        ] {
            assert!(requirement_kinds.contains(required), "{required} requirement missing");
        }

        for corridor in &geometry.corridors {
            let corridor_ref = format!("geometryCorridor:{}", corridor.id);
            let corridor_requirements = plan
                .requirements
                .iter()
                .filter(|requirement| requirement.source_refs.contains(&corridor_ref))
                .collect::<Vec<_>>();
            assert!(
                corridor_requirements
                    .iter()
                    .any(|requirement| requirement.kind == "connector"),
                "{} missing connector piece requirements",
                corridor.id
            );
            assert!(
                corridor_requirements
                    .iter()
                    .any(|requirement| requirement.kind == "corridor"),
                "{} missing corridor segment piece requirements",
                corridor.id
            );
        }

        let sockets = plan
            .content_requirements
            .iter()
            .map(|requirement| requirement.required_socket.as_str())
            .collect::<BTreeSet<_>>();
        for required in [
            "gate_line",
            "key_pickup",
            "hazard_zone",
            "reward_cache",
            "boss_space",
            "secret_marker",
            "shortcut_marker",
            "resource_clue",
        ] {
            assert!(sockets.contains(required), "{required} content socket missing");
        }

        let link_tags = plan
            .links
            .iter()
            .flat_map(|link| link.tags.iter().map(String::as_str))
            .collect::<BTreeSet<_>>();
        for required in [
            "locked_threshold",
            "hidden_route",
            "shortcut_link",
            "pressure_route",
            "rejoin_corridor",
        ] {
            assert!(link_tags.contains(required), "{required} link tag missing");
        }
        assert!(
            plan.links
                .iter()
                .any(|link| link.traversal_hint == "open"),
            "normal open corridor link missing"
        );

        let locked_requirement = plan
            .requirements
            .iter()
            .find(|requirement| requirement.tags.contains(&"locked_threshold".to_owned()))
            .expect("locked corridor requirement should exist");
        assert!(
            locked_requirement
                .source_refs
                .iter()
                .any(|source_ref| source_ref.starts_with("edge:"))
        );
    }

    #[test]
    fn shape_matcher_rotates_exits_for_piece_requirements() {
        let catalog = test_shape_catalog(vec![test_catalog_shape(
            "shape.room.one_east",
            &["room"],
            &["identity", "rotate90", "rotate180", "rotate270"],
            vec![test_catalog_exit("exit.east", "east")],
            Vec::new(),
            &["standard_room"],
        )]);
        let plan = test_piece_plan(vec![test_piece_requirement(
            "piece.room.start",
            "room",
            vec![test_piece_exit("exit.required.north", "north")],
            Vec::new(),
            &["room"],
        )]);
        let args = test_match_args(9001);
        let report = match_shapes(&catalog, &plan, &args);

        assert!(report.ok);
        assert_eq!(report.matches.len(), 1);
        assert_eq!(report.matches[0].shape_id, "shape.room.one_east");
        assert_eq!(report.matches[0].transform, "rotate270");
        assert_eq!(report.matches[0].exit_map[0].catalog_exit_id, "exit.east");
    }

    #[test]
    fn shape_matcher_reports_missing_sockets_and_exit_count() {
        let catalog = test_shape_catalog(vec![
            test_catalog_shape(
                "shape.threshold.no_gate_line",
                &["threshold"],
                &["identity"],
                vec![test_catalog_exit("exit.west", "west"), test_catalog_exit("exit.east", "east")],
                Vec::new(),
                &["threshold"],
            ),
            test_catalog_shape(
                "shape.corridor.one_exit",
                &["corridor"],
                &["identity"],
                vec![test_catalog_exit("exit.east", "east")],
                Vec::new(),
                &["corridor"],
            ),
        ]);
        let plan = test_piece_plan(vec![
            test_piece_requirement(
                "piece.threshold.locked",
                "threshold",
                vec![test_piece_exit("exit.required.west", "west")],
                vec!["gate_line".to_owned()],
                &["locked_threshold"],
            ),
            test_piece_requirement(
                "piece.corridor.two_exit",
                "corridor",
                vec![
                    test_piece_exit("exit.required.west", "west"),
                    test_piece_exit("exit.required.east", "east"),
                ],
                Vec::new(),
                &["corridor"],
            ),
        ]);
        let report = match_shapes(&catalog, &plan, &test_match_args(9002));

        assert!(!report.ok);
        assert_eq!(report.unmatched_count, 2);
        assert!(report.diagnostics.iter().all(|diagnostic| {
            diagnostic.code == "shape_match_missing" && diagnostic.severity == Severity::Fatal
        }));
        assert!(report.rejections.iter().any(|rejection| {
            rejection.piece_id == "piece.threshold.locked"
                && rejection
                    .reasons
                    .iter()
                    .any(|reason| reason.contains("missing_sockets: gate_line"))
        }));
        assert!(report.rejections.iter().any(|rejection| {
            rejection.piece_id == "piece.corridor.two_exit"
                && rejection
                    .reasons
                    .iter()
                    .any(|reason| reason.contains("exit_count_mismatch"))
        }));
    }

    #[test]
    fn shape_matcher_tie_breaks_deterministically() {
        let left = test_catalog_shape(
            "shape.room.tie_left",
            &["room"],
            &["identity"],
            vec![test_catalog_exit("exit.east", "east")],
            Vec::new(),
            &["standard_room"],
        );
        let right = test_catalog_shape(
            "shape.room.tie_right",
            &["room"],
            &["identity"],
            vec![test_catalog_exit("exit.east", "east")],
            Vec::new(),
            &["standard_room"],
        );
        let plan = test_piece_plan(vec![test_piece_requirement(
            "piece.room.tie",
            "room",
            vec![test_piece_exit("exit.required.east", "east")],
            Vec::new(),
            &["room"],
        )]);

        let report_a = match_shapes(
            &test_shape_catalog(vec![left.clone(), right.clone()]),
            &plan,
            &test_match_args(42),
        );
        let report_b = match_shapes(
            &test_shape_catalog(vec![right, left]),
            &plan,
            &test_match_args(42),
        );

        assert!(report_a.ok);
        assert!(report_b.ok);
        assert_eq!(report_a.matches[0].shape_id, report_b.matches[0].shape_id);
        assert_eq!(report_a.matches[0].transform, report_b.matches[0].transform);
    }

    #[test]
    fn piece_placement_assembles_full_stack_without_overlap() {
        let placement = full_stack_piece_placement_fixture(951);
        let report = validate_piece_placement(&placement);

        assert!(report.ok, "{:?}", report.diagnostics);
        assert_eq!(placement.kind, "asha_procgen.piece_placement.v1");
        assert_eq!(placement.grid_connectivity, GridConnectivity::FourWay);
        assert!(placement.occupied_cells.len() >= placement.instances.len());
        assert!(!placement.connection_cells.is_empty());
        assert!(placement
            .instances
            .iter()
            .all(|instance| !instance.occupied_cells.is_empty()));
        let (width, height) = placement_bounds(&placement);
        assert!(width < 200, "placement should not collapse into a long atlas: {width}x{height}");
        assert!(height > 10, "placement should preserve source geometry depth: {width}x{height}");
        assert!(placement.instances.iter().all(|instance| {
            !instance.shape_id.is_empty()
                && !instance.source_requirement_ref.is_empty()
                && !instance.source_refs.is_empty()
        }));
        assert!(!placement.glued_exits.is_empty());
        assert!(placement.dangling_exits.is_empty());
    }

    #[test]
    fn piece_placement_validation_catches_overlap_reservation_dangling_and_reachability() {
        let placement = full_stack_piece_placement_fixture(1051);

        let mut overlap = placement.clone();
        let first = overlap.occupied_cells[0].clone();
        overlap.occupied_cells[1].x = first.x;
        overlap.occupied_cells[1].y = first.y;
        let report = validate_piece_placement(&overlap);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "piece_occupied_cell_overlap"));

        let mut reservation = placement.clone();
        let occupied = reservation.occupied_cells[0].clone();
        let reserver = reservation.instances[1].instance_id.clone();
        reservation.reserved_cells.push(PlacementCellRef {
            instance_id: reserver,
            x: occupied.x,
            y: occupied.y,
        });
        let report = validate_piece_placement(&reservation);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "piece_reserved_cell_conflict"));

        let mut touching = placement.clone();
        let first = touching.occupied_cells[0].clone();
        let touching_index = touching
            .occupied_cells
            .iter()
            .position(|cell| cell.instance_id != first.instance_id)
            .expect("fixture should have more than one instance");
        touching.occupied_cells[touching_index].x = first.x + 1;
        touching.occupied_cells[touching_index].y = first.y;
        touching.glued_exits.clear();
        let report = validate_piece_placement(&touching);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "piece_unplanned_occupied_adjacency"));

        let mut dangling = placement.clone();
        dangling.dangling_exits.push(DanglingExit {
            instance_id: dangling.instances[0].instance_id.clone(),
            exit_id: "exit.test".to_owned(),
            reason: "test".to_owned(),
        });
        let report = validate_piece_placement(&dangling);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "piece_required_exit_dangling"));

        let mut unreachable = placement;
        unreachable.glued_exits.clear();
        unreachable.dangling_exits.clear();
        let report = validate_piece_placement(&unreachable);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "piece_goal_unreachable"));

        let mut grid_unreachable = full_stack_piece_placement_fixture(1052);
        grid_unreachable.connection_cells.clear();
        let report = validate_piece_placement(&grid_unreachable);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "piece_grid_instance_unreachable"));
    }

    #[test]
    fn grid_connectivity_distinguishes_cardinal_and_diagonal_neighbors() {
        let origin = GridCell { x: 0, y: 0 };
        let cardinal = GridCell { x: 1, y: 0 };
        let diagonal = GridCell { x: 1, y: 1 };

        assert!(cells_adjacent(&origin, &cardinal, GridConnectivity::FourWay));
        assert!(!cells_adjacent(&origin, &diagonal, GridConnectivity::FourWay));
        assert!(cells_adjacent(&origin, &diagonal, GridConnectivity::EightWay));
    }

    fn full_stack_geometry_fixture(seed: u64) -> Geometry2dArtifact {
        full_stack_geometry_inputs(seed).2
    }

    fn full_stack_geometry_inputs(
        seed: u64,
    ) -> (Candidate, IntermediateBreakdown, Geometry2dArtifact) {
        let intent = test_intent("geometry-validation");
        let mut candidate = create_initial_candidate(&intent, seed);
        for (index, rule) in [
            GraphRule::LockKeyLoop,
            GraphRule::HubSpokeCluster,
            GraphRule::HazardResourceTradeoff,
            GraphRule::BossPreparationLoop,
            GraphRule::GatedTreasureBranch,
            GraphRule::BranchMergeShortcut,
            GraphRule::OneWayShortcut,
            GraphRule::SecretBypass,
        ]
        .into_iter()
        .enumerate()
        {
            assert!(apply_graph_rule(&mut candidate, rule, seed + 1 + index as u64).is_empty());
        }
        let annotations = spatial_intent_report(&candidate, None).expect("spatial intent report");
        let intermediate = intermediate_breakdown(
            &candidate,
            &annotations,
            Path::new("artifacts/test/spatial-intent.json"),
        )
        .expect("breakdown should encode");
        let args = GeometryEmit2dArgs {
            candidate: PathBuf::from("artifacts/test/candidate.json"),
            intermediate: PathBuf::from("artifacts/test/intermediate-breakdown.json"),
            seed: seed + 20,
            out: PathBuf::from("artifacts/test/geometry.json"),
        };
        let geometry =
            emit_geometry_2d(&candidate, &intermediate, &args, seed + 20).expect("geometry should emit");
        (candidate, intermediate, geometry)
    }

    fn full_stack_piece_placement_fixture(seed: u64) -> PiecePlacement {
        let (candidate, intermediate, geometry) = full_stack_geometry_inputs(seed);
        let piece_plan_args = BuildEmitPiecePlanArgs {
            candidate: PathBuf::from("artifacts/test/candidate.json"),
            intermediate: PathBuf::from("artifacts/test/intermediate-breakdown.json"),
            geometry: PathBuf::from("artifacts/test/geometry.json"),
            out: PathBuf::from("artifacts/test/piece-plan.json"),
        };
        let piece_plan = emit_piece_build_plan(
            &candidate,
            &intermediate,
            &geometry,
            &piece_plan_args,
        )
        .expect("piece plan should emit");
        let catalog_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .join(DEFAULT_SHAPE_CATALOG);
        let catalog: ShapeCatalog = read_json(&catalog_path).expect("shape catalog should load");
        let match_args = test_match_args(seed + 30);
        let shape_match = match_shapes(&catalog, &piece_plan, &match_args);
        assert!(shape_match.ok, "{:?}", shape_match.diagnostics);
        let assemble_args = BuildAssembleArgs {
            catalog: PathBuf::from("fixtures/shape-catalogs/2d-basic.json"),
            piece_plan: PathBuf::from("artifacts/test/piece-plan.json"),
            shape_match: PathBuf::from("artifacts/test/piece-shape-match.json"),
            connectivity: GridConnectivity::FourWay,
            out: PathBuf::from("artifacts/test/piece-placement.json"),
        };
        assemble_piece_placement(&catalog, &piece_plan, &shape_match, &assemble_args)
            .expect("piece placement should assemble")
    }

    fn rectangles_overlap(left: &GeometryRect, right: &GeometryRect) -> bool {
        geometry_rectangles_overlap(left, right)
    }

    fn point_on_rect_boundary(point: &GeometryPoint, rect: &GeometryRect) -> bool {
        geometry_point_on_rect_boundary(point, rect)
    }

    fn test_shape_catalog(shapes: Vec<CatalogShape>) -> ShapeCatalog {
        ShapeCatalog {
            kind: "asha_procgen.shape_catalog.v1".to_owned(),
            schema_version: 1,
            catalog_id: "shape_catalog.test.v1".to_owned(),
            cell_size: 1,
            shapes,
        }
    }

    fn test_catalog_shape(
        shape_id: &str,
        piece_kinds: &[&str],
        allowed_transforms: &[&str],
        exits: Vec<CatalogExit>,
        feature_sockets: Vec<FeatureSocket>,
        tags: &[&str],
    ) -> CatalogShape {
        CatalogShape {
            shape_id: shape_id.to_owned(),
            label: shape_id.to_owned(),
            piece_kinds: piece_kinds.iter().map(|kind| (*kind).to_owned()).collect(),
            footprint: vec![GridCell { x: 0, y: 0 }],
            reserved_cells: Vec::new(),
            exits,
            allowed_transforms: allowed_transforms
                .iter()
                .map(|transform| (*transform).to_owned())
                .collect(),
            feature_sockets,
            tags: tags.iter().map(|tag| (*tag).to_owned()).collect(),
        }
    }

    fn test_catalog_exit(id: &str, direction: &str) -> CatalogExit {
        CatalogExit {
            id: id.to_owned(),
            x: 0,
            y: 0,
            direction: direction.to_owned(),
            width: 1,
            tags: Vec::new(),
        }
    }

    fn test_piece_plan(requirements: Vec<PieceRequirement>) -> PieceBuildPlan {
        PieceBuildPlan {
            kind: "asha_procgen.piece_build_plan.v1".to_owned(),
            schema_version: 1,
            plan_id: "piece_plan.test".to_owned(),
            candidate_id: "candidate.test".to_owned(),
            geometry_id: "geometry.test".to_owned(),
            source_candidate_ref: "artifacts/test/candidate.json".to_owned(),
            source_intermediate_ref: "artifacts/test/intermediate.json".to_owned(),
            source_geometry_ref: "artifacts/test/geometry.json".to_owned(),
            requirements,
            links: Vec::new(),
            content_requirements: Vec::new(),
        }
    }

    fn test_piece_requirement(
        piece_id: &str,
        kind: &str,
        required_exits: Vec<PieceExitRequirement>,
        required_sockets: Vec<String>,
        tags: &[&str],
    ) -> PieceRequirement {
        PieceRequirement {
            piece_id: piece_id.to_owned(),
            kind: kind.to_owned(),
            role: kind.to_owned(),
            source_refs: Vec::new(),
            required_exits,
            required_sockets,
            tags: tags.iter().map(|tag| (*tag).to_owned()).collect(),
            placement_hints: Vec::new(),
        }
    }

    fn test_piece_exit(id: &str, direction: &str) -> PieceExitRequirement {
        PieceExitRequirement {
            id: id.to_owned(),
            direction: direction.to_owned(),
            width: 1,
            tags: Vec::new(),
        }
    }

    fn test_match_args(seed: u64) -> BuildMatchShapesArgs {
        BuildMatchShapesArgs {
            catalog: PathBuf::from("fixtures/shape-catalogs/2d-basic.json"),
            piece_plan: PathBuf::from("artifacts/test/piece-plan.json"),
            seed,
            out: PathBuf::from("artifacts/test/piece-shape-match.json"),
        }
    }

    fn placement_bounds(placement: &PiecePlacement) -> (i32, i32) {
        let min_x = placement
            .occupied_cells
            .iter()
            .map(|cell| cell.x)
            .min()
            .unwrap_or(0);
        let max_x = placement
            .occupied_cells
            .iter()
            .map(|cell| cell.x)
            .max()
            .unwrap_or(0);
        let min_y = placement
            .occupied_cells
            .iter()
            .map(|cell| cell.y)
            .min()
            .unwrap_or(0);
        let max_y = placement
            .occupied_cells
            .iter()
            .map(|cell| cell.y)
            .max()
            .unwrap_or(0);
        (max_x - min_x + 1, max_y - min_y + 1)
    }

    #[test]
    fn loads_default_batch_profile_fixture() {
        let profile_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .join(DEFAULT_BATCH_PROFILE);
        let profile = load_batch_profile(&profile_path).expect("default profile should load");
        assert_eq!(profile.kind, "asha_procgen.batch_profile.v1");
        assert_eq!(profile.sequences.len(), 6);
        let first = batch_profile_sequence(&profile, 0).expect("first sequence");
        assert_eq!(first.label, "hub-merge");
        assert_eq!(
            first.rules,
            vec![
                GraphRule::LockKeyLoop,
                GraphRule::HubSpokeCluster,
                GraphRule::BranchMergeShortcut
            ]
        );
        let cycled = batch_profile_sequence(&profile, 6).expect("cycled sequence");
        assert_eq!(cycled.label, "hub-merge");
    }

    #[test]
    fn loads_default_shape_catalog_fixture() {
        let catalog_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .join(DEFAULT_SHAPE_CATALOG);
        let catalog: ShapeCatalog = read_json(&catalog_path).expect("shape catalog should load");
        assert_eq!(catalog.kind, "asha_procgen.shape_catalog.v1");
        assert_eq!(catalog.catalog_id, "shape_catalog.2d_basic.v1");
        assert!(catalog.cell_size > 0);
        assert!(catalog.shapes.len() >= 12);

        let mut shape_ids = BTreeSet::new();
        let mut piece_kinds = BTreeSet::new();
        let mut feature_kinds = BTreeSet::new();
        let allowed_directions = BTreeSet::from(["north", "east", "south", "west"]);
        let allowed_transforms = BTreeSet::from([
            "identity",
            "rotate90",
            "rotate180",
            "rotate270",
            "mirrorX",
            "mirrorY",
        ]);
        for shape in &catalog.shapes {
            assert!(
                shape_ids.insert(shape.shape_id.as_str()),
                "duplicate shape id {}",
                shape.shape_id
            );
            assert!(!shape.piece_kinds.is_empty(), "{} has no piece kinds", shape.shape_id);
            assert!(!shape.footprint.is_empty(), "{} has no footprint", shape.shape_id);
            assert!(!shape.exits.is_empty(), "{} has no exits", shape.shape_id);
            assert!(
                shape
                    .allowed_transforms
                    .iter()
                    .all(|transform| allowed_transforms.contains(transform.as_str())),
                "{} has an unsupported transform",
                shape.shape_id
            );
            for exit in &shape.exits {
                assert!(
                    allowed_directions.contains(exit.direction.as_str()),
                    "{} has unsupported exit direction {}",
                    shape.shape_id,
                    exit.direction
                );
                assert!(exit.width > 0, "{} has non-positive exit width", shape.shape_id);
            }
            piece_kinds.extend(shape.piece_kinds.iter().map(String::as_str));
            feature_kinds.extend(shape.feature_sockets.iter().map(|socket| socket.kind.as_str()));
        }

        for required in [
            "room",
            "corridor",
            "bend",
            "threshold",
            "reward",
            "hazard",
            "boss",
            "secret",
            "shortcut",
            "resource",
        ] {
            assert!(piece_kinds.contains(required), "{required} piece kind missing");
        }
        for required in [
            "container",
            "boss_space",
            "gate_line",
            "hazard_zone",
            "reward_cache",
            "key_pickup",
            "secret_marker",
            "shortcut_marker",
            "resource_clue",
        ] {
            assert!(feature_kinds.contains(required), "{required} socket missing");
        }

        for exit_count in 1..=4 {
            assert!(
                catalog
                    .shapes
                    .iter()
                    .any(|shape| shape.piece_kinds.iter().any(|kind| kind == "room")
                        && shape.exits.len() == exit_count),
                "{exit_count}-exit room shape missing"
            );
        }
    }

    #[test]
    fn catalog_inspect_reports_default_shape_vocabulary() {
        let catalog_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .join(DEFAULT_SHAPE_CATALOG);
        let catalog: ShapeCatalog = read_json(&catalog_path).expect("shape catalog should load");
        let report = inspect_shape_catalog(&catalog, &catalog_path);

        assert_eq!(report.kind, "asha_procgen.catalog_inspection.v1");
        assert_eq!(report.catalog_id, "shape_catalog.2d_basic.v1");
        assert_eq!(report.shape_count, catalog.shapes.len());
        assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
        for required in ["room", "corridor", "connector", "threshold", "reward", "key"] {
            assert!(report.piece_kinds.contains(&required.to_owned()));
        }
        for required in ["north", "east", "south", "west"] {
            assert!(report.exit_directions.contains(&required.to_owned()));
        }
        assert!(report.transforms.contains(&"rotate90".to_owned()));
    }

    #[test]
    fn scoring_rewards_cycles() {
        let intent = SeedIntent {
            kind: "asha_procgen.seed_intent.v1".to_owned(),
            id: "score".to_owned(),
            title: "Score".to_owned(),
            target_dimension: "topology_graph".to_owned(),
            desired_patterns: Vec::new(),
            notes: Vec::new(),
        };
        let mut candidate = create_initial_candidate(&intent, 1);
        let base = score_graph(&candidate).overall;
        apply_graph_rule(&mut candidate, GraphRule::LockKeyLoop, 2);
        apply_graph_rule(&mut candidate, GraphRule::OptionalTreasureDetour, 3);
        let richer = score_graph(&candidate).overall;
        assert!(richer > base);
    }

    #[test]
    fn embeds_valid_graph() {
        let intent = SeedIntent {
            kind: "asha_procgen.seed_intent.v1".to_owned(),
            id: "embed".to_owned(),
            title: "Embed".to_owned(),
            target_dimension: "topology_graph".to_owned(),
            desired_patterns: Vec::new(),
            notes: Vec::new(),
        };
        let mut candidate = create_initial_candidate(&intent, 1);
        apply_graph_rule(&mut candidate, GraphRule::LockKeyLoop, 2);
        let layout = embed_2d(&candidate, 3);
        assert_eq!(layout.candidate_id, candidate.candidate_id);
        assert_eq!(layout.rooms.len(), candidate.graph.nodes.len());
        assert_eq!(layout.links.len(), candidate.graph.edges.len());
    }
}
