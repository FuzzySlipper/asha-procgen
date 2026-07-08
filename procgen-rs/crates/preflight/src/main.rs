use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

fn main() {
    let cli = Cli::parse();
    if let Err(error) = run(cli) {
        eprintln!("asha-procgen failed:");
        eprintln!("- {error}");
        std::process::exit(1);
    }
}

#[derive(Parser)]
#[command(name = "asha-procgen")]
#[command(about = "Deterministic dungeon procgen CLI workbench")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Check sibling ASHA engine checkout posture.
    Preflight(PreflightArgs),
    /// Create a minimal candidate from a seed intent.
    Init(InitArgs),
    /// Mutate or summarize intent graphs.
    Graph(GraphCommand),
    /// Validate candidates.
    Validate(ValidateCommand),
    /// Score candidates.
    Score(ScoreCommand),
    /// Embed candidates into inspectable layouts.
    Embed(EmbedCommand),
    /// Accept a validated candidate/layout as an artifact.
    Accept(AcceptArgs),
    /// Produce the first deterministic sample run.
    Baseline(BaselineArgs),
}

#[derive(Args)]
struct PreflightArgs {
    #[arg(default_value = ".")]
    repo_root: PathBuf,
}

#[derive(Args)]
struct InitArgs {
    #[arg(long)]
    intent: PathBuf,
    #[arg(long)]
    seed: u64,
    #[arg(long)]
    out: PathBuf,
    #[arg(long)]
    receipt: PathBuf,
    #[arg(long)]
    transcript: Option<PathBuf>,
}

#[derive(Args)]
struct GraphCommand {
    #[command(subcommand)]
    command: GraphSubcommand,
}

#[derive(Subcommand)]
enum GraphSubcommand {
    ApplyRule(ApplyRuleArgs),
    Summarize(StateArg),
}

#[derive(Args)]
struct ApplyRuleArgs {
    #[arg(long)]
    state: PathBuf,
    #[arg(long)]
    rule: GraphRule,
    #[arg(long)]
    seed: u64,
    #[arg(long)]
    out: PathBuf,
    #[arg(long)]
    receipt: PathBuf,
    #[arg(long)]
    transcript: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[value(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
enum GraphRule {
    LockKeyLoop,
    OptionalTreasureDetour,
    OneWayShortcut,
    SecretBypass,
}

impl GraphRule {
    fn as_str(self) -> &'static str {
        match self {
            GraphRule::LockKeyLoop => "lock_key_loop",
            GraphRule::OptionalTreasureDetour => "optional_treasure_detour",
            GraphRule::OneWayShortcut => "one_way_shortcut",
            GraphRule::SecretBypass => "secret_bypass",
        }
    }
}

#[derive(Args)]
struct ValidateCommand {
    #[command(subcommand)]
    command: ValidateSubcommand,
}

#[derive(Subcommand)]
enum ValidateSubcommand {
    Graph(ReportOutArgs),
}

#[derive(Args)]
struct ScoreCommand {
    #[command(subcommand)]
    command: ScoreSubcommand,
}

#[derive(Subcommand)]
enum ScoreSubcommand {
    Graph(ReportOutArgs),
}

#[derive(Args)]
struct EmbedCommand {
    #[command(subcommand)]
    command: EmbedSubcommand,
}

#[derive(Subcommand)]
enum EmbedSubcommand {
    #[command(name = "2d")]
    TwoD(Embed2dArgs),
}

#[derive(Args)]
struct StateArg {
    #[arg(long)]
    state: PathBuf,
}

#[derive(Args)]
struct ReportOutArgs {
    #[arg(long)]
    state: PathBuf,
    #[arg(long)]
    out: PathBuf,
}

#[derive(Args)]
struct Embed2dArgs {
    #[arg(long)]
    state: PathBuf,
    #[arg(long)]
    seed: u64,
    #[arg(long)]
    out: PathBuf,
    #[arg(long)]
    receipt: PathBuf,
    #[arg(long)]
    transcript: Option<PathBuf>,
}

#[derive(Args)]
struct AcceptArgs {
    #[arg(long)]
    candidate: PathBuf,
    #[arg(long)]
    layout: PathBuf,
    #[arg(long)]
    validation: PathBuf,
    #[arg(long)]
    score: PathBuf,
    #[arg(long)]
    out: PathBuf,
    #[arg(long)]
    receipt: PathBuf,
    #[arg(long)]
    transcript: Option<PathBuf>,
}

#[derive(Args)]
struct BaselineArgs {
    #[arg(long)]
    out_dir: PathBuf,
    #[arg(long, default_value_t = 4103)]
    seed: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SeedIntent {
    kind: String,
    id: String,
    title: String,
    target_dimension: String,
    desired_patterns: Vec<String>,
    notes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Candidate {
    kind: String,
    schema_version: u32,
    candidate_id: String,
    seed: u64,
    dimension_model: String,
    source_intent: Option<String>,
    provenance: Vec<ProvenanceStep>,
    graph: IntentGraph,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProvenanceStep {
    step: u32,
    command: String,
    seed: Option<u64>,
    summary: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct IntentGraph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Node {
    id: String,
    kind: NodeKind,
    label: String,
    tags: Vec<String>,
    grants_item: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
enum NodeKind {
    Start,
    Goal,
    Gate,
    Key,
    Treasure,
    Shortcut,
    Secret,
    Hazard,
    Resource,
    Junction,
}

impl NodeKind {
    fn as_str(self) -> &'static str {
        match self {
            NodeKind::Start => "start",
            NodeKind::Goal => "goal",
            NodeKind::Gate => "gate",
            NodeKind::Key => "key",
            NodeKind::Treasure => "treasure",
            NodeKind::Shortcut => "shortcut",
            NodeKind::Secret => "secret",
            NodeKind::Hazard => "hazard",
            NodeKind::Resource => "resource",
            NodeKind::Junction => "junction",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Edge {
    id: String,
    from: String,
    to: String,
    kind: EdgeKind,
    traversal: TraversalKind,
    required_item: Option<String>,
    tags: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum EdgeKind {
    CriticalPath,
    KeyBranch,
    OptionalBranch,
    Shortcut,
    SecretBypass,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum TraversalKind {
    Open,
    Locked,
    OneWayReturn,
    Hidden,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Receipt {
    kind: String,
    schema_version: u32,
    command: String,
    status: String,
    seed: Option<u64>,
    input_hash: Option<String>,
    output_hash: Option<String>,
    output_ref: Option<String>,
    diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Diagnostic {
    code: String,
    severity: Severity,
    node: Option<String>,
    edge: Option<String>,
    detail: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Severity {
    Info,
    Warning,
    Fatal,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ValidationReport {
    kind: String,
    schema_version: u32,
    state_hash: String,
    ok: bool,
    fatal_count: usize,
    diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScoreReport {
    kind: String,
    schema_version: u32,
    state_hash: String,
    overall: f64,
    metrics: BTreeMap<String, f64>,
    notes: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LayoutArtifact {
    kind: String,
    schema_version: u32,
    layout_id: String,
    candidate_id: String,
    seed: u64,
    rooms: Vec<LayoutRoom>,
    links: Vec<LayoutLink>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LayoutRoom {
    node_id: String,
    kind: NodeKind,
    label: String,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LayoutLink {
    edge_id: String,
    from_node: String,
    to_node: String,
    kind: EdgeKind,
    traversal: TraversalKind,
    required_item: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AcceptedArtifact {
    kind: String,
    schema_version: u32,
    artifact_id: String,
    candidate_hash: String,
    layout_hash: String,
    validation_ref: String,
    score_ref: String,
    candidate: Candidate,
    layout: LayoutArtifact,
    score_summary: ScoreReport,
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Preflight(args) => run_preflight_command(&args.repo_root),
        Command::Init(args) => init_candidate(args),
        Command::Graph(command) => match command.command {
            GraphSubcommand::ApplyRule(args) => apply_rule(args),
            GraphSubcommand::Summarize(args) => summarize_candidate(args),
        },
        Command::Validate(command) => match command.command {
            ValidateSubcommand::Graph(args) => validate_graph_command(args),
        },
        Command::Score(command) => match command.command {
            ScoreSubcommand::Graph(args) => score_graph_command(args),
        },
        Command::Embed(command) => match command.command {
            EmbedSubcommand::TwoD(args) => embed_2d_command(args),
        },
        Command::Accept(args) => accept_command(args),
        Command::Baseline(args) => baseline_command(args),
    }
}

fn run_preflight_command(repo_root: &Path) -> Result<(), String> {
    let summary = run_preflight(repo_root)?;
    println!(
        "asha-procgen preflight OK: engine source {}, rust lane {}",
        summary.engine_source, summary.rust_dir
    );
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct PreflightSummary {
    engine_source: String,
    rust_dir: String,
}

fn run_preflight(repo_root: &Path) -> Result<PreflightSummary, String> {
    let engine_source = "../asha-engine";
    reject_private_engine_path("engine source", engine_source)?;

    let engine_root = repo_root.join(engine_source);
    if !engine_root.exists() {
        return Err(format!(
            "expected sibling ASHA engine checkout at {}",
            engine_root.display()
        ));
    }

    Ok(PreflightSummary {
        engine_source: engine_source.to_owned(),
        rust_dir: "procgen-rs".to_owned(),
    })
}

fn reject_private_engine_path(label: &str, value: &str) -> Result<(), String> {
    let forbidden_fragments = [
        "../asha-engine/engine-rs",
        "../asha-engine/ts/packages",
        "../asha/engine-rs",
        "../asha/ts/packages",
    ];
    for fragment in forbidden_fragments {
        if value.contains(fragment) {
            return Err(format!(
                "{label} must not reference private ASHA internals: {value}"
            ));
        }
    }
    Ok(())
}

fn init_candidate(args: InitArgs) -> Result<(), String> {
    let intent: SeedIntent = read_json(&args.intent)?;
    let mut candidate = create_initial_candidate(&intent, args.seed);
    candidate.provenance.push(ProvenanceStep {
        step: 1,
        command: "init".to_owned(),
        seed: Some(args.seed),
        summary: format!("Initialized {} from {}", candidate.candidate_id, intent.id),
    });
    write_json(&args.out, &candidate)?;
    let receipt = receipt(
        "init",
        Some(args.seed),
        Some(&hash_file(&args.intent)?),
        Some(&hash_json(&candidate)?),
        Some(&args.out),
        Vec::new(),
    );
    write_json(&args.receipt, &receipt)?;
    append_transcript(
        args.transcript.as_deref(),
        "init",
        Some(&args.out),
        Some(&args.receipt),
        Some(args.seed),
        json!({ "intent": display_path(&args.intent) }),
    )?;
    Ok(())
}

fn create_initial_candidate(intent: &SeedIntent, seed: u64) -> Candidate {
    Candidate {
        kind: "asha_procgen.candidate.v1".to_owned(),
        schema_version: 1,
        candidate_id: format!("candidate.{}.{}", intent.id, seed),
        seed,
        dimension_model: "topology_graph".to_owned(),
        source_intent: Some(intent.id.clone()),
        provenance: Vec::new(),
        graph: IntentGraph {
            nodes: vec![
                Node {
                    id: "start".to_owned(),
                    kind: NodeKind::Start,
                    label: "Start".to_owned(),
                    tags: vec!["critical".to_owned()],
                    grants_item: None,
                },
                Node {
                    id: "goal".to_owned(),
                    kind: NodeKind::Goal,
                    label: "Goal".to_owned(),
                    tags: vec!["critical".to_owned()],
                    grants_item: None,
                },
            ],
            edges: vec![Edge {
                id: "edge.start.goal".to_owned(),
                from: "start".to_owned(),
                to: "goal".to_owned(),
                kind: EdgeKind::CriticalPath,
                traversal: TraversalKind::Open,
                required_item: None,
                tags: vec!["initial".to_owned()],
            }],
        },
    }
}

fn apply_rule(args: ApplyRuleArgs) -> Result<(), String> {
    let mut candidate: Candidate = read_json(&args.state)?;
    let input_hash = hash_file(&args.state)?;
    let diagnostics = apply_graph_rule(&mut candidate, args.rule, args.seed);
    let status = if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Fatal)
    {
        "rejected"
    } else {
        candidate.provenance.push(ProvenanceStep {
            step: candidate.provenance.len() as u32 + 1,
            command: format!("graph apply-rule {}", args.rule.as_str()),
            seed: Some(args.seed),
            summary: format!("Applied {}", args.rule.as_str()),
        });
        write_json(&args.out, &candidate)?;
        "ok"
    };
    let output_hash = if status == "ok" {
        Some(hash_json(&candidate)?)
    } else {
        None
    };
    let receipt = Receipt {
        kind: "asha_procgen.receipt.v1".to_owned(),
        schema_version: 1,
        command: format!("graph apply-rule {}", args.rule.as_str()),
        status: status.to_owned(),
        seed: Some(args.seed),
        input_hash: Some(input_hash),
        output_hash,
        output_ref: if status == "ok" {
            Some(display_path(&args.out))
        } else {
            None
        },
        diagnostics,
    };
    write_json(&args.receipt, &receipt)?;
    append_transcript(
        args.transcript.as_deref(),
        "graph apply-rule",
        if status == "ok" { Some(&args.out) } else { None },
        Some(&args.receipt),
        Some(args.seed),
        json!({ "rule": args.rule.as_str(), "state": display_path(&args.state) }),
    )?;
    if status == "ok" {
        Ok(())
    } else {
        Err("graph rule was rejected; see receipt diagnostics".to_owned())
    }
}

fn apply_graph_rule(candidate: &mut Candidate, rule: GraphRule, seed: u64) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    match rule {
        GraphRule::LockKeyLoop => {
            if candidate.graph.nodes.iter().any(|node| node.id == "gate.locked_1") {
                diagnostics.push(fatal(
                    "rule_already_applied",
                    Some("gate.locked_1"),
                    None,
                    "lock_key_loop is already present.",
                ));
                return diagnostics;
            }
            candidate
                .graph
                .edges
                .retain(|edge| edge.id != "edge.start.goal");
            candidate.graph.nodes.extend([
                Node {
                    id: "gate.locked_1".to_owned(),
                    kind: NodeKind::Gate,
                    label: "Locked Gate".to_owned(),
                    tags: vec!["lock".to_owned(), "critical".to_owned()],
                    grants_item: None,
                },
                Node {
                    id: "key.gate_1".to_owned(),
                    kind: NodeKind::Key,
                    label: "Gate Key".to_owned(),
                    tags: vec!["key".to_owned(), "branch".to_owned()],
                    grants_item: Some("item.gate_key_1".to_owned()),
                },
            ]);
            candidate.graph.edges.extend([
                Edge {
                    id: "edge.start.gate_1".to_owned(),
                    from: "start".to_owned(),
                    to: "gate.locked_1".to_owned(),
                    kind: EdgeKind::CriticalPath,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["approach".to_owned()],
                },
                Edge {
                    id: "edge.gate_1.goal".to_owned(),
                    from: "gate.locked_1".to_owned(),
                    to: "goal".to_owned(),
                    kind: EdgeKind::CriticalPath,
                    traversal: TraversalKind::Locked,
                    required_item: Some("item.gate_key_1".to_owned()),
                    tags: vec!["lock".to_owned()],
                },
                Edge {
                    id: "edge.start.key_1".to_owned(),
                    from: "start".to_owned(),
                    to: "key.gate_1".to_owned(),
                    kind: EdgeKind::KeyBranch,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["branch".to_owned()],
                },
                Edge {
                    id: "edge.key_1.gate_1".to_owned(),
                    from: "key.gate_1".to_owned(),
                    to: "gate.locked_1".to_owned(),
                    kind: EdgeKind::KeyBranch,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["return".to_owned()],
                },
            ]);
        }
        GraphRule::OptionalTreasureDetour => {
            let suffix = stable_suffix(seed);
            let treasure_id = format!("treasure.{suffix}");
            candidate.graph.nodes.push(Node {
                id: treasure_id.clone(),
                kind: NodeKind::Treasure,
                label: "Optional Treasure".to_owned(),
                tags: vec!["optional".to_owned(), "reward".to_owned()],
                grants_item: None,
            });
            candidate.graph.edges.extend([
                Edge {
                    id: format!("edge.start.{treasure_id}"),
                    from: "start".to_owned(),
                    to: treasure_id.clone(),
                    kind: EdgeKind::OptionalBranch,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["detour".to_owned()],
                },
                Edge {
                    id: format!("edge.{treasure_id}.goal"),
                    from: treasure_id,
                    to: "goal".to_owned(),
                    kind: EdgeKind::OptionalBranch,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["rejoin".to_owned()],
                },
            ]);
        }
        GraphRule::OneWayShortcut => {
            if candidate.graph.edges.iter().any(|edge| edge.id == "edge.goal.start.shortcut") {
                diagnostics.push(fatal(
                    "rule_already_applied",
                    None,
                    Some("edge.goal.start.shortcut"),
                    "one_way_shortcut is already present.",
                ));
                return diagnostics;
            }
            candidate.graph.nodes.push(Node {
                id: "shortcut.return_1".to_owned(),
                kind: NodeKind::Shortcut,
                label: "Return Shortcut".to_owned(),
                tags: vec!["shortcut".to_owned()],
                grants_item: None,
            });
            candidate.graph.edges.extend([
                Edge {
                    id: "edge.goal.shortcut_1".to_owned(),
                    from: "goal".to_owned(),
                    to: "shortcut.return_1".to_owned(),
                    kind: EdgeKind::Shortcut,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["shortcut".to_owned()],
                },
                Edge {
                    id: "edge.shortcut_1.start".to_owned(),
                    from: "shortcut.return_1".to_owned(),
                    to: "start".to_owned(),
                    kind: EdgeKind::Shortcut,
                    traversal: TraversalKind::OneWayReturn,
                    required_item: None,
                    tags: vec!["return".to_owned()],
                },
            ]);
        }
        GraphRule::SecretBypass => {
            if candidate.graph.edges.iter().any(|edge| edge.id == "edge.start.goal.secret") {
                diagnostics.push(fatal(
                    "rule_already_applied",
                    None,
                    Some("edge.start.goal.secret"),
                    "secret_bypass is already present.",
                ));
                return diagnostics;
            }
            candidate.graph.nodes.push(Node {
                id: "secret.bypass_1".to_owned(),
                kind: NodeKind::Secret,
                label: "Secret Bypass".to_owned(),
                tags: vec!["secret".to_owned(), "bypass".to_owned()],
                grants_item: None,
            });
            candidate.graph.edges.extend([
                Edge {
                    id: "edge.start.secret_1".to_owned(),
                    from: "start".to_owned(),
                    to: "secret.bypass_1".to_owned(),
                    kind: EdgeKind::SecretBypass,
                    traversal: TraversalKind::Hidden,
                    required_item: None,
                    tags: vec!["hidden".to_owned()],
                },
                Edge {
                    id: "edge.secret_1.goal".to_owned(),
                    from: "secret.bypass_1".to_owned(),
                    to: "goal".to_owned(),
                    kind: EdgeKind::SecretBypass,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["bypass".to_owned()],
                },
            ]);
        }
    }
    diagnostics
}

fn summarize_candidate(args: StateArg) -> Result<(), String> {
    let candidate: Candidate = read_json(&args.state)?;
    let report = validate_graph(&candidate);
    let score = score_graph(&candidate);
    println!("candidate: {}", candidate.candidate_id);
    println!("nodes: {}", candidate.graph.nodes.len());
    println!("edges: {}", candidate.graph.edges.len());
    println!("valid: {}", report.ok);
    println!("overall score: {:.2}", score.overall);
    for node in &candidate.graph.nodes {
        println!("- node {} ({}) {}", node.id, node.kind.as_str(), node.label);
    }
    Ok(())
}

fn validate_graph_command(args: ReportOutArgs) -> Result<(), String> {
    let candidate: Candidate = read_json(&args.state)?;
    let report = validate_graph(&candidate);
    write_json(&args.out, &report)?;
    if report.ok {
        Ok(())
    } else {
        Err(format!(
            "graph validation failed with {} fatal diagnostic(s); see {}",
            report.fatal_count,
            args.out.display()
        ))
    }
}

fn validate_graph(candidate: &Candidate) -> ValidationReport {
    let mut diagnostics = Vec::new();
    let node_ids: BTreeSet<&str> = candidate.graph.nodes.iter().map(|node| node.id.as_str()).collect();
    let start_count = candidate
        .graph
        .nodes
        .iter()
        .filter(|node| node.kind == NodeKind::Start)
        .count();
    let goal_count = candidate
        .graph
        .nodes
        .iter()
        .filter(|node| node.kind == NodeKind::Goal)
        .count();
    if start_count != 1 {
        diagnostics.push(fatal(
            "start_count_invalid",
            None,
            None,
            "Graph must contain exactly one start node.",
        ));
    }
    if goal_count != 1 {
        diagnostics.push(fatal(
            "goal_count_invalid",
            None,
            None,
            "Graph must contain exactly one goal node.",
        ));
    }
    for edge in &candidate.graph.edges {
        if !node_ids.contains(edge.from.as_str()) {
            diagnostics.push(fatal(
                "edge_from_missing",
                None,
                Some(edge.id.as_str()),
                "Edge source node is missing.",
            ));
        }
        if !node_ids.contains(edge.to.as_str()) {
            diagnostics.push(fatal(
                "edge_to_missing",
                None,
                Some(edge.id.as_str()),
                "Edge target node is missing.",
            ));
        }
    }

    let granted_items: BTreeSet<&str> = candidate
        .graph
        .nodes
        .iter()
        .filter_map(|node| node.grants_item.as_deref())
        .collect();
    for edge in &candidate.graph.edges {
        if let Some(required_item) = edge.required_item.as_deref() {
            if !granted_items.contains(required_item) {
                diagnostics.push(fatal(
                    "required_item_unavailable",
                    None,
                    Some(edge.id.as_str()),
                    format!("Edge requires {required_item}, but no node grants it."),
                ));
            }
        }
    }

    if start_count == 1 && goal_count == 1 {
        let reachable = reachable_with_items(candidate);
        if !reachable.goal_reached {
            diagnostics.push(fatal(
                "goal_unreachable",
                Some("goal"),
                None,
                "Goal is not reachable under lock/key constraints.",
            ));
        }
        for edge in &candidate.graph.edges {
            if edge.traversal == TraversalKind::Locked && !reachable.traversed_edges.contains(edge.id.as_str()) {
                diagnostics.push(fatal(
                    "locked_edge_never_traversed",
                    None,
                    Some(edge.id.as_str()),
                    "Locked edge could not be traversed after item collection.",
                ));
            }
        }
    }

    let mut incoming: BTreeMap<&str, usize> = BTreeMap::new();
    let mut outgoing: BTreeMap<&str, usize> = BTreeMap::new();
    for edge in &candidate.graph.edges {
        *incoming.entry(edge.to.as_str()).or_insert(0) += 1;
        *outgoing.entry(edge.from.as_str()).or_insert(0) += 1;
    }
    for node in &candidate.graph.nodes {
        if node.kind != NodeKind::Goal && outgoing.get(node.id.as_str()).copied().unwrap_or(0) == 0 {
            diagnostics.push(warning(
                "non_goal_dead_end",
                Some(node.id.as_str()),
                None,
                "Non-goal node has no outgoing route.",
            ));
        }
        if node.kind != NodeKind::Start && incoming.get(node.id.as_str()).copied().unwrap_or(0) == 0 {
            diagnostics.push(warning(
                "orphan_node",
                Some(node.id.as_str()),
                None,
                "Node has no incoming route.",
            ));
        }
    }

    let fatal_count = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Fatal)
        .count();
    ValidationReport {
        kind: "asha_procgen.validation.graph.v1".to_owned(),
        schema_version: 1,
        state_hash: hash_json(candidate).unwrap_or_else(|_| "hash_error".to_owned()),
        ok: fatal_count == 0,
        fatal_count,
        diagnostics,
    }
}

struct Reachability<'a> {
    goal_reached: bool,
    traversed_edges: BTreeSet<&'a str>,
}

fn reachable_with_items(candidate: &Candidate) -> Reachability<'_> {
    let mut adjacency: BTreeMap<&str, Vec<&Edge>> = BTreeMap::new();
    for edge in &candidate.graph.edges {
        adjacency.entry(edge.from.as_str()).or_default().push(edge);
    }
    let mut queue = VecDeque::new();
    let mut visited = BTreeSet::new();
    let mut traversed_edges = BTreeSet::new();
    let mut items = BTreeSet::new();
    queue.push_back("start");
    visited.insert("start");

    while let Some(node_id) = queue.pop_front() {
        if let Some(node) = candidate.graph.nodes.iter().find(|node| node.id == node_id) {
            if let Some(item) = node.grants_item.as_deref() {
                if items.insert(item) {
                    visited.clear();
                    visited.insert(node_id);
                    queue.clear();
                    queue.push_back(node_id);
                }
            }
        }
        for edge in adjacency.get(node_id).into_iter().flatten() {
            if edge
                .required_item
                .as_deref()
                .is_some_and(|item| !items.contains(item))
            {
                continue;
            }
            traversed_edges.insert(edge.id.as_str());
            if visited.insert(edge.to.as_str()) {
                queue.push_back(edge.to.as_str());
            }
        }
    }

    Reachability {
        goal_reached: visited.contains("goal"),
        traversed_edges,
    }
}

fn score_graph_command(args: ReportOutArgs) -> Result<(), String> {
    let candidate: Candidate = read_json(&args.state)?;
    let report = score_graph(&candidate);
    write_json(&args.out, &report)
}

fn score_graph(candidate: &Candidate) -> ScoreReport {
    let node_count = candidate.graph.nodes.len() as f64;
    let edge_count = candidate.graph.edges.len() as f64;
    let loop_bonus = cycle_count(candidate) as f64;
    let optional_count = candidate
        .graph
        .edges
        .iter()
        .filter(|edge| edge.kind == EdgeKind::OptionalBranch || edge.kind == EdgeKind::SecretBypass)
        .count() as f64;
    let locked_count = candidate
        .graph
        .edges
        .iter()
        .filter(|edge| edge.traversal == TraversalKind::Locked)
        .count() as f64;
    let shortcut_count = candidate
        .graph
        .edges
        .iter()
        .filter(|edge| edge.kind == EdgeKind::Shortcut)
        .count() as f64;
    let critical_path = shortest_path_len(candidate, "start", "goal").unwrap_or(0) as f64;
    let dead_end_count = dead_end_count(candidate) as f64;
    let mut metrics = BTreeMap::new();
    metrics.insert("nodeCount".to_owned(), node_count);
    metrics.insert("edgeCount".to_owned(), edge_count);
    metrics.insert("criticalPathLength".to_owned(), critical_path);
    metrics.insert("loopCount".to_owned(), loop_bonus);
    metrics.insert("optionalBranchCount".to_owned(), optional_count);
    metrics.insert("lockedEdgeCount".to_owned(), locked_count);
    metrics.insert("shortcutCount".to_owned(), shortcut_count);
    metrics.insert("deadEndCount".to_owned(), dead_end_count);

    let raw = 0.18
        + (critical_path.min(6.0) * 0.08)
        + (loop_bonus.min(3.0) * 0.12)
        + (optional_count.min(4.0) * 0.06)
        + (locked_count.min(2.0) * 0.08)
        + (shortcut_count.min(2.0) * 0.05)
        - (dead_end_count * 0.04);
    let overall = (raw.clamp(0.0, 1.0) * 100.0).round() / 100.0;
    ScoreReport {
        kind: "asha_procgen.score.graph.v1".to_owned(),
        schema_version: 1,
        state_hash: hash_json(candidate).unwrap_or_else(|_| "hash_error".to_owned()),
        overall,
        metrics,
        notes: vec![
            "Graph score is a deterministic first-slice heuristic, not a human-quality verdict."
                .to_owned(),
        ],
    }
}

fn shortest_path_len(candidate: &Candidate, start: &str, goal: &str) -> Option<usize> {
    let mut adjacency: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for edge in &candidate.graph.edges {
        adjacency
            .entry(edge.from.as_str())
            .or_default()
            .push(edge.to.as_str());
    }
    let mut queue = VecDeque::from([(start, 0usize)]);
    let mut visited = BTreeSet::from([start]);
    while let Some((node, depth)) = queue.pop_front() {
        if node == goal {
            return Some(depth);
        }
        for next in adjacency.get(node).into_iter().flatten() {
            if visited.insert(next) {
                queue.push_back((next, depth + 1));
            }
        }
    }
    None
}

fn cycle_count(candidate: &Candidate) -> usize {
    let node_count = candidate.graph.nodes.len();
    let edge_count = candidate.graph.edges.len();
    if node_count == 0 {
        return 0;
    }
    let component_count = 1;
    edge_count.saturating_sub(node_count).saturating_add(component_count)
}

fn dead_end_count(candidate: &Candidate) -> usize {
    candidate
        .graph
        .nodes
        .iter()
        .filter(|node| node.kind != NodeKind::Goal)
        .filter(|node| {
            !candidate
                .graph
                .edges
                .iter()
                .any(|edge| edge.from == node.id)
        })
        .count()
}

fn embed_2d_command(args: Embed2dArgs) -> Result<(), String> {
    let candidate: Candidate = read_json(&args.state)?;
    let validation = validate_graph(&candidate);
    if !validation.ok {
        return Err("cannot embed invalid graph candidate".to_owned());
    }
    let input_hash = hash_file(&args.state)?;
    let layout = embed_2d(&candidate, args.seed);
    write_json(&args.out, &layout)?;
    let receipt = receipt(
        "embed 2d",
        Some(args.seed),
        Some(&input_hash),
        Some(&hash_json(&layout)?),
        Some(&args.out),
        Vec::new(),
    );
    write_json(&args.receipt, &receipt)?;
    append_transcript(
        args.transcript.as_deref(),
        "embed 2d",
        Some(&args.out),
        Some(&args.receipt),
        Some(args.seed),
        json!({ "state": display_path(&args.state) }),
    )?;
    Ok(())
}

fn embed_2d(candidate: &Candidate, seed: u64) -> LayoutArtifact {
    let depths = graph_depths(candidate);
    let mut rows_by_depth: BTreeMap<usize, usize> = BTreeMap::new();
    let mut rooms = Vec::new();
    for node in &candidate.graph.nodes {
        let depth = depths.get(node.id.as_str()).copied().unwrap_or(0);
        let row = rows_by_depth.entry(depth).or_insert(0);
        let y_offset = *row as i32;
        *row += 1;
        rooms.push(LayoutRoom {
            node_id: node.id.clone(),
            kind: node.kind,
            label: node.label.clone(),
            x: 80 + depth as i32 * 180,
            y: 80 + y_offset * 110,
            width: 116,
            height: 64,
        });
    }
    LayoutArtifact {
        kind: "asha_procgen.layout_2d.v1".to_owned(),
        schema_version: 1,
        layout_id: format!("layout.{}.{}", candidate.candidate_id, seed),
        candidate_id: candidate.candidate_id.clone(),
        seed,
        rooms,
        links: candidate
            .graph
            .edges
            .iter()
            .map(|edge| LayoutLink {
                edge_id: edge.id.clone(),
                from_node: edge.from.clone(),
                to_node: edge.to.clone(),
                kind: edge.kind,
                traversal: edge.traversal,
                required_item: edge.required_item.clone(),
            })
            .collect(),
    }
}

fn graph_depths(candidate: &Candidate) -> BTreeMap<&str, usize> {
    let mut adjacency: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for edge in &candidate.graph.edges {
        adjacency
            .entry(edge.from.as_str())
            .or_default()
            .push(edge.to.as_str());
    }
    let mut depths = BTreeMap::new();
    let mut queue = VecDeque::from([("start", 0usize)]);
    depths.insert("start", 0);
    while let Some((node, depth)) = queue.pop_front() {
        for next in adjacency.get(node).into_iter().flatten() {
            if !depths.contains_key(next) {
                depths.insert(*next, depth + 1);
                queue.push_back((next, depth + 1));
            }
        }
    }
    depths
}

fn accept_command(args: AcceptArgs) -> Result<(), String> {
    let candidate: Candidate = read_json(&args.candidate)?;
    let layout: LayoutArtifact = read_json(&args.layout)?;
    let validation: ValidationReport = read_json(&args.validation)?;
    let score: ScoreReport = read_json(&args.score)?;
    if !validation.ok {
        return Err("cannot accept artifact with failing validation".to_owned());
    }
    let candidate_hash = hash_json(&candidate)?;
    let layout_hash = hash_json(&layout)?;
    let artifact = AcceptedArtifact {
        kind: "asha_procgen.accepted_artifact.v1".to_owned(),
        schema_version: 1,
        artifact_id: format!("accepted.{}", candidate.candidate_id),
        candidate_hash: candidate_hash.clone(),
        layout_hash: layout_hash.clone(),
        validation_ref: display_path(&args.validation),
        score_ref: display_path(&args.score),
        candidate,
        layout,
        score_summary: score,
    };
    write_json(&args.out, &artifact)?;
    let receipt = receipt(
        "accept",
        None,
        Some(&candidate_hash),
        Some(&hash_json(&artifact)?),
        Some(&args.out),
        Vec::new(),
    );
    write_json(&args.receipt, &receipt)?;
    append_transcript(
        args.transcript.as_deref(),
        "accept",
        Some(&args.out),
        Some(&args.receipt),
        None,
        json!({
            "candidate": display_path(&args.candidate),
            "layout": display_path(&args.layout),
            "validation": display_path(&args.validation),
            "score": display_path(&args.score)
        }),
    )
}

fn baseline_command(args: BaselineArgs) -> Result<(), String> {
    fs::create_dir_all(&args.out_dir)
        .map_err(|error| format!("failed to create {}: {error}", args.out_dir.display()))?;
    let intent_path = PathBuf::from("fixtures/intents/first-slice.intent.json");
    let transcript = args.out_dir.join("transcript.jsonl");
    if transcript.exists() {
        fs::remove_file(&transcript)
            .map_err(|error| format!("failed to reset {}: {error}", transcript.display()))?;
    }
    let base = args.out_dir.join("candidate-000-base.json");
    let base_receipt = args.out_dir.join("receipt-000-init.json");
    init_candidate(InitArgs {
        intent: intent_path,
        seed: args.seed,
        out: base.clone(),
        receipt: base_receipt,
        transcript: Some(transcript.clone()),
    })?;

    let mut current = base;
    for (index, rule) in [
        GraphRule::LockKeyLoop,
        GraphRule::OptionalTreasureDetour,
        GraphRule::OneWayShortcut,
        GraphRule::SecretBypass,
    ]
    .into_iter()
    .enumerate()
    {
        let next = args
            .out_dir
            .join(format!("candidate-{:03}-{}.json", index + 1, rule.as_str()));
        let receipt_path = args
            .out_dir
            .join(format!("receipt-{:03}-{}.json", index + 1, rule.as_str()));
        apply_rule(ApplyRuleArgs {
            state: current,
            rule,
            seed: args.seed + index as u64 + 1,
            out: next.clone(),
            receipt: receipt_path,
            transcript: Some(transcript.clone()),
        })?;
        current = next;
    }

    let validation = args.out_dir.join("validation.graph.json");
    validate_graph_command(ReportOutArgs {
        state: current.clone(),
        out: validation.clone(),
    })?;
    append_transcript(
        Some(&transcript),
        "validate graph",
        Some(&validation),
        None,
        None,
        json!({ "state": display_path(&current) }),
    )?;
    let score = args.out_dir.join("score.graph.json");
    score_graph_command(ReportOutArgs {
        state: current.clone(),
        out: score.clone(),
    })?;
    append_transcript(
        Some(&transcript),
        "score graph",
        Some(&score),
        None,
        None,
        json!({ "state": display_path(&current) }),
    )?;
    let layout = args.out_dir.join("layout-2d.json");
    let layout_receipt = args.out_dir.join("receipt-005-embed-2d.json");
    embed_2d_command(Embed2dArgs {
        state: current.clone(),
        seed: args.seed + 10,
        out: layout.clone(),
        receipt: layout_receipt,
        transcript: Some(transcript.clone()),
    })?;
    accept_command(AcceptArgs {
        candidate: current,
        layout,
        validation,
        score,
        out: args.out_dir.join("accepted.json"),
        receipt: args.out_dir.join("receipt-006-accept.json"),
        transcript: Some(transcript),
    })?;
    println!("baseline run written to {}", args.out_dir.display());
    Ok(())
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("failed to parse {}: {error}", path.display()))
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(value)
        .map_err(|error| format!("failed to encode JSON for {}: {error}", path.display()))?;
    fs::write(path, format!("{text}\n"))
        .map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn append_transcript(
    path: Option<&Path>,
    command: &str,
    state: Option<&Path>,
    receipt: Option<&Path>,
    seed: Option<u64>,
    args: JsonValue,
) -> Result<(), String> {
    let Some(path) = path else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }
    let event = json!({
        "kind": "tool_event",
        "command": command,
        "state": state.map(display_path),
        "receipt": receipt.map(display_path),
        "seed": seed,
        "args": args
    });
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| format!("failed to open {}: {error}", path.display()))?;
    writeln!(file, "{event}")
        .map_err(|error| format!("failed to write transcript {}: {error}", path.display()))
}

fn receipt(
    command: &str,
    seed: Option<u64>,
    input_hash: Option<&str>,
    output_hash: Option<&str>,
    output_ref: Option<&Path>,
    diagnostics: Vec<Diagnostic>,
) -> Receipt {
    Receipt {
        kind: "asha_procgen.receipt.v1".to_owned(),
        schema_version: 1,
        command: command.to_owned(),
        status: "ok".to_owned(),
        seed,
        input_hash: input_hash.map(str::to_owned),
        output_hash: output_hash.map(str::to_owned),
        output_ref: output_ref.map(display_path),
        diagnostics,
    }
}

fn hash_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    Ok(format!("fnv1a64:{:016x}", fnv1a64(&bytes)))
}

fn hash_json<T: Serialize>(value: &T) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(|error| format!("failed to encode hash input: {error}"))?;
    Ok(format!("fnv1a64:{:016x}", fnv1a64(&bytes)))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn stable_suffix(seed: u64) -> String {
    format!("{:04x}", seed & 0xffff)
}

fn fatal(
    code: &str,
    node: Option<&str>,
    edge: Option<&str>,
    detail: impl Into<String>,
) -> Diagnostic {
    Diagnostic {
        code: code.to_owned(),
        severity: Severity::Fatal,
        node: node.map(str::to_owned),
        edge: edge.map(str::to_owned),
        detail: detail.into(),
    }
}

fn warning(
    code: &str,
    node: Option<&str>,
    edge: Option<&str>,
    detail: impl Into<String>,
) -> Diagnostic {
    Diagnostic {
        code: code.to_owned(),
        severity: Severity::Warning,
        node: node.map(str::to_owned),
        edge: edge.map(str::to_owned),
        detail: detail.into(),
    }
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_private_engine_paths() {
        let error = reject_private_engine_path("demo", "../asha-engine/engine-rs/crates/state")
            .expect_err("private engine path should be rejected");
        assert!(error.contains("private ASHA internals"));
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
            .any(|diagnostic| diagnostic.code == "required_item_unavailable"));
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
