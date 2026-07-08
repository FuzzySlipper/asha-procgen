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
    /// Analyze graph topology.
    Analyze(AnalyzeCommand),
    /// Add pre-geometry annotations.
    Annotate(AnnotateCommand),
    /// Emit or validate intermediate layout breakdowns.
    Breakdown(BreakdownCommand),
    /// Validate candidates.
    Validate(ValidateCommand),
    /// Suggest repair actions for invalid or warning-heavy candidates.
    Repair(RepairCommand),
    /// Score candidates.
    Score(ScoreCommand),
    /// Embed candidates into inspectable layouts.
    Embed(EmbedCommand),
    /// Accept a validated candidate/layout as an artifact.
    Accept(AcceptArgs),
    /// Produce the first deterministic sample run.
    Baseline(BaselineArgs),
    /// Generate a deterministic batch and selection report.
    Batch(BatchCommand),
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
    CompatibleRules(ReportOutArgs),
    Fork(ForkArgs),
    Rules(RuleMetadataArgs),
    Summarize(SummarizeArgs),
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

#[derive(Args)]
struct ForkArgs {
    #[arg(long)]
    state: PathBuf,
    #[arg(long)]
    label: String,
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
    HubSpokeCluster,
    NestedLockKeyChain,
    HazardResourceTradeoff,
    BossPreparationLoop,
    GatedTreasureBranch,
    BranchMergeShortcut,
}

impl GraphRule {
    fn as_str(self) -> &'static str {
        match self {
            GraphRule::LockKeyLoop => "lock_key_loop",
            GraphRule::OptionalTreasureDetour => "optional_treasure_detour",
            GraphRule::OneWayShortcut => "one_way_shortcut",
            GraphRule::SecretBypass => "secret_bypass",
            GraphRule::HubSpokeCluster => "hub_spoke_cluster",
            GraphRule::NestedLockKeyChain => "nested_lock_key_chain",
            GraphRule::HazardResourceTradeoff => "hazard_resource_tradeoff",
            GraphRule::BossPreparationLoop => "boss_preparation_loop",
            GraphRule::GatedTreasureBranch => "gated_treasure_branch",
            GraphRule::BranchMergeShortcut => "branch_merge_shortcut",
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
struct RepairCommand {
    #[command(subcommand)]
    command: RepairSubcommand,
}

#[derive(Subcommand)]
enum RepairSubcommand {
    Apply(RepairApplyArgs),
    Suggest(ReportOutArgs),
}

#[derive(Args)]
struct RepairApplyArgs {
    #[arg(long)]
    state: PathBuf,
    #[arg(long)]
    action: RepairAction,
    #[arg(long)]
    target: Option<String>,
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
enum RepairAction {
    AddRejoinEdge,
    RemoveOrphanNode,
}

impl RepairAction {
    fn as_str(self) -> &'static str {
        match self {
            RepairAction::AddRejoinEdge => "add_rejoin_edge",
            RepairAction::RemoveOrphanNode => "remove_orphan_node",
        }
    }
}

#[derive(Args)]
struct AnalyzeCommand {
    #[command(subcommand)]
    command: AnalyzeSubcommand,
}

#[derive(Subcommand)]
enum AnalyzeSubcommand {
    Graph(ReportOutArgs),
}

#[derive(Args)]
struct AnnotateCommand {
    #[command(subcommand)]
    command: AnnotateSubcommand,
}

#[derive(Subcommand)]
enum AnnotateSubcommand {
    SpatialIntent(AnnotateSpatialIntentArgs),
}

#[derive(Args)]
struct AnnotateSpatialIntentArgs {
    #[arg(long)]
    state: PathBuf,
    #[arg(long)]
    analysis: Option<PathBuf>,
    #[arg(long)]
    out: PathBuf,
}

#[derive(Args)]
struct BreakdownCommand {
    #[command(subcommand)]
    command: BreakdownSubcommand,
}

#[derive(Subcommand)]
enum BreakdownSubcommand {
    Emit(BreakdownEmitArgs),
    Validate(ReportOutArgs),
}

#[derive(Args)]
struct BreakdownEmitArgs {
    #[arg(long)]
    state: PathBuf,
    #[arg(long)]
    annotations: PathBuf,
    #[arg(long)]
    out: PathBuf,
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
struct RuleMetadataArgs {
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Args)]
struct SummarizeArgs {
    #[arg(long)]
    state: PathBuf,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    out: Option<PathBuf>,
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

#[derive(Args)]
struct BatchCommand {
    #[command(subcommand)]
    command: BatchSubcommand,
}

#[derive(Subcommand)]
enum BatchSubcommand {
    Generate(BatchGenerateArgs),
}

#[derive(Args)]
struct BatchGenerateArgs {
    #[arg(long)]
    out_dir: PathBuf,
    #[arg(long)]
    profile: Option<PathBuf>,
    #[arg(long, default_value_t = 5201)]
    seed: u64,
    #[arg(long, default_value_t = 10)]
    count: usize,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct IntentBudget {
    max_locked_edges: Option<usize>,
    min_optional_branches: Option<usize>,
    require_hub: Option<bool>,
    require_boss: Option<bool>,
    max_dead_ends: Option<usize>,
}

const DEFAULT_BATCH_PROFILE: &str = "fixtures/batch-profiles/v2-sample.json";

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
    #[serde(skip_serializing_if = "Option::is_none")]
    repair_hint: Option<String>,
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
struct RuleMetadataReport {
    kind: String,
    schema_version: u32,
    rules: Vec<RuleMetadata>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct RuleMetadata {
    id: String,
    intent: String,
    required_patterns: Vec<String>,
    duplicate_markers: Vec<String>,
    emitted_node_tags: Vec<String>,
    emitted_edge_tags: Vec<String>,
    compatibility_hints: Vec<String>,
    repair_hints: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct GraphSummaryReport {
    kind: String,
    schema_version: u32,
    candidate_id: String,
    state_hash: String,
    validation_ok: bool,
    fatal_count: usize,
    score_overall: f64,
    metrics: BTreeMap<String, f64>,
    node_count: usize,
    edge_count: usize,
    tags: Vec<String>,
    locked_items: Vec<String>,
    dead_ends: Vec<String>,
    provenance_tail: Vec<ProvenanceStep>,
    nodes: Vec<NodeSummary>,
    edges: Vec<EdgeSummary>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct RepairReport {
    kind: String,
    schema_version: u32,
    candidate_id: String,
    state_hash: String,
    validation_ok: bool,
    fatal_count: usize,
    suggestions: Vec<RepairSuggestion>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct RepairSuggestion {
    code: String,
    severity: Severity,
    node: Option<String>,
    edge: Option<String>,
    detail: String,
    repair_hint: Option<String>,
    suggested_actions: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct GraphAnalysisReport {
    kind: String,
    schema_version: u32,
    candidate_id: String,
    state_hash: String,
    critical_path: Vec<String>,
    dominators: Vec<String>,
    optional_branches: Vec<BranchAnalysis>,
    lock_key_order: Vec<LockKeyAnalysis>,
    loop_signals: Vec<LoopSignal>,
    shortcut_bypass_risks: Vec<ShortcutRisk>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct BranchAnalysis {
    edge_id: String,
    from: String,
    to: String,
    classification: String,
    rejoins_goal_route: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LockKeyAnalysis {
    edge_id: String,
    required_item: String,
    provider_node: Option<String>,
    provider_reachable_before_lock: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LoopSignal {
    edge_id: String,
    signal: String,
    detail: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ShortcutRisk {
    edge_id: String,
    risk: String,
    detail: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct RuleCompatibilityReport {
    kind: String,
    schema_version: u32,
    candidate_id: String,
    state_hash: String,
    rules: Vec<RuleCompatibility>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct RuleCompatibility {
    rule: String,
    status: String,
    reasons: Vec<String>,
    recommended_actions: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SpatialIntentReport {
    kind: String,
    schema_version: u32,
    candidate_id: String,
    state_hash: String,
    analysis_ref: Option<String>,
    annotations: Vec<SpatialIntentAnnotation>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SpatialIntentAnnotation {
    target_type: String,
    target_id: String,
    intents: Vec<String>,
    rationale: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct IntermediateBreakdown {
    kind: String,
    schema_version: u32,
    candidate_id: String,
    state_hash: String,
    annotation_ref: String,
    regions: Vec<IntermediateRegion>,
    connectors: Vec<IntermediateConnector>,
    constraints: Vec<IntermediateConstraint>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct IntermediateRegion {
    id: String,
    node_ids: Vec<String>,
    role: String,
    anchor_node: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct IntermediateConnector {
    id: String,
    edge_id: String,
    from_region: String,
    to_region: String,
    intents: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct IntermediateConstraint {
    code: String,
    target: String,
    detail: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct NodeSummary {
    id: String,
    kind: NodeKind,
    tags: Vec<String>,
    grants_item: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct EdgeSummary {
    id: String,
    from: String,
    to: String,
    kind: EdgeKind,
    traversal: TraversalKind,
    required_item: Option<String>,
    tags: Vec<String>,
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SelectionReport {
    kind: String,
    schema_version: u32,
    batch_id: String,
    profile_id: String,
    profile_ref: String,
    seed: u64,
    requested_count: usize,
    generated_count: usize,
    accepted: Vec<SelectionEntry>,
    rejected: Vec<SelectionRejection>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SelectionEntry {
    candidate_id: String,
    profile_sequence: String,
    topology_fingerprint: String,
    duplicate_of: Option<String>,
    budget_checks: Vec<BudgetCheck>,
    budget_penalty: f64,
    selection_score: f64,
    artifact_ref: String,
    validation_ref: String,
    score_ref: String,
    layout_ref: String,
    overall: f64,
    metrics: BTreeMap<String, f64>,
    tags: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SelectionRejection {
    candidate_id: String,
    profile_sequence: String,
    candidate_ref: String,
    diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchProfile {
    kind: String,
    schema_version: u32,
    profile_id: String,
    description: String,
    budgets: Option<IntentBudget>,
    sequences: Vec<BatchProfileSequence>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct BudgetCheck {
    code: String,
    ok: bool,
    detail: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchProfileSequence {
    label: String,
    rules: Vec<GraphRule>,
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Preflight(args) => run_preflight_command(&args.repo_root),
        Command::Init(args) => init_candidate(args),
        Command::Graph(command) => match command.command {
            GraphSubcommand::ApplyRule(args) => apply_rule(args),
            GraphSubcommand::CompatibleRules(args) => compatible_rules_command(args),
            GraphSubcommand::Fork(args) => fork_command(args),
            GraphSubcommand::Rules(args) => graph_rules_command(args),
            GraphSubcommand::Summarize(args) => summarize_candidate(args),
        },
        Command::Analyze(command) => match command.command {
            AnalyzeSubcommand::Graph(args) => analyze_graph_command(args),
        },
        Command::Annotate(command) => match command.command {
            AnnotateSubcommand::SpatialIntent(args) => annotate_spatial_intent_command(args),
        },
        Command::Breakdown(command) => match command.command {
            BreakdownSubcommand::Emit(args) => breakdown_emit_command(args),
            BreakdownSubcommand::Validate(args) => breakdown_validate_command(args),
        },
        Command::Validate(command) => match command.command {
            ValidateSubcommand::Graph(args) => validate_graph_command(args),
        },
        Command::Repair(command) => match command.command {
            RepairSubcommand::Apply(args) => repair_apply_command(args),
            RepairSubcommand::Suggest(args) => repair_suggest_command(args),
        },
        Command::Score(command) => match command.command {
            ScoreSubcommand::Graph(args) => score_graph_command(args),
        },
        Command::Embed(command) => match command.command {
            EmbedSubcommand::TwoD(args) => embed_2d_command(args),
        },
        Command::Accept(args) => accept_command(args),
        Command::Baseline(args) => baseline_command(args),
        Command::Batch(command) => match command.command {
            BatchSubcommand::Generate(args) => batch_generate_command(args),
        },
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
        if status == "ok" {
            Some(&args.out)
        } else {
            None
        },
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

fn fork_command(args: ForkArgs) -> Result<(), String> {
    let candidate: Candidate = read_json(&args.state)?;
    let input_hash = hash_file(&args.state)?;
    let forked = fork_candidate(candidate, &args.label, args.seed);
    write_json(&args.out, &forked)?;
    let receipt = receipt(
        "graph fork",
        Some(args.seed),
        Some(&input_hash),
        Some(&hash_json(&forked)?),
        Some(&args.out),
        Vec::new(),
    );
    write_json(&args.receipt, &receipt)?;
    append_transcript(
        args.transcript.as_deref(),
        "graph fork",
        Some(&args.out),
        Some(&args.receipt),
        Some(args.seed),
        json!({
            "state": display_path(&args.state),
            "label": args.label
        }),
    )?;
    Ok(())
}

fn fork_candidate(mut candidate: Candidate, label: &str, seed: u64) -> Candidate {
    let source_id = candidate.candidate_id.clone();
    let label_slug = slugify_label(label);
    candidate.candidate_id = format!("{}.fork.{}.{}", source_id, label_slug, seed);
    candidate.seed = seed;
    candidate.provenance.push(ProvenanceStep {
        step: candidate.provenance.len() as u32 + 1,
        command: "graph fork".to_owned(),
        seed: Some(seed),
        summary: format!(
            "Forked {source_id} as {} with label {label_slug}",
            candidate.candidate_id
        ),
    });
    candidate
}

fn apply_graph_rule(candidate: &mut Candidate, rule: GraphRule, seed: u64) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    match rule {
        GraphRule::LockKeyLoop => {
            if candidate
                .graph
                .nodes
                .iter()
                .any(|node| node.id == "gate.locked_1")
            {
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
            if candidate
                .graph
                .edges
                .iter()
                .any(|edge| edge.id == "edge.goal.start.shortcut")
            {
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
            if candidate
                .graph
                .edges
                .iter()
                .any(|edge| edge.id == "edge.start.goal.secret")
            {
                diagnostics.push(fatal_with_hint(
                    "rule_already_applied",
                    None,
                    Some("edge.start.goal.secret"),
                    "secret_bypass is already present.",
                    "Choose a different bypass rule or start from an earlier candidate.",
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
        GraphRule::HubSpokeCluster => {
            if has_node(candidate, "hub.central_1") {
                diagnostics.push(fatal_with_hint(
                    "rule_already_applied",
                    Some("hub.central_1"),
                    None,
                    "hub_spoke_cluster is already present.",
                    "Use an alternate hub id or apply a different branch pattern.",
                ));
                return diagnostics;
            }
            candidate.graph.nodes.extend([
                Node {
                    id: "hub.central_1".to_owned(),
                    kind: NodeKind::Junction,
                    label: "Wayfinding Hub".to_owned(),
                    tags: vec![
                        "hub".to_owned(),
                        "wayfinding_anchor".to_owned(),
                        "merge".to_owned(),
                    ],
                    grants_item: None,
                },
                Node {
                    id: "resource.clue_1".to_owned(),
                    kind: NodeKind::Resource,
                    label: "Route Clue".to_owned(),
                    tags: vec!["optional".to_owned(), "preparation".to_owned()],
                    grants_item: None,
                },
                Node {
                    id: "hazard.watch_1".to_owned(),
                    kind: NodeKind::Hazard,
                    label: "Watched Passage".to_owned(),
                    tags: vec!["optional".to_owned(), "hazard".to_owned()],
                    grants_item: None,
                },
                Node {
                    id: "treasure.cache_1".to_owned(),
                    kind: NodeKind::Treasure,
                    label: "Hub Cache".to_owned(),
                    tags: vec!["optional".to_owned(), "reward".to_owned()],
                    grants_item: None,
                },
            ]);
            candidate.graph.edges.extend([
                Edge {
                    id: "edge.start.hub_1".to_owned(),
                    from: "start".to_owned(),
                    to: "hub.central_1".to_owned(),
                    kind: EdgeKind::CriticalPath,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["approach".to_owned()],
                },
                Edge {
                    id: "edge.hub_1.goal".to_owned(),
                    from: "hub.central_1".to_owned(),
                    to: "goal".to_owned(),
                    kind: EdgeKind::CriticalPath,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["rejoin".to_owned()],
                },
                Edge {
                    id: "edge.hub_1.clue_1".to_owned(),
                    from: "hub.central_1".to_owned(),
                    to: "resource.clue_1".to_owned(),
                    kind: EdgeKind::OptionalBranch,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["branch".to_owned()],
                },
                Edge {
                    id: "edge.clue_1.hub_1".to_owned(),
                    from: "resource.clue_1".to_owned(),
                    to: "hub.central_1".to_owned(),
                    kind: EdgeKind::OptionalBranch,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["return".to_owned()],
                },
                Edge {
                    id: "edge.hub_1.watch_1".to_owned(),
                    from: "hub.central_1".to_owned(),
                    to: "hazard.watch_1".to_owned(),
                    kind: EdgeKind::OptionalBranch,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["branch".to_owned(), "pressure".to_owned()],
                },
                Edge {
                    id: "edge.watch_1.goal".to_owned(),
                    from: "hazard.watch_1".to_owned(),
                    to: "goal".to_owned(),
                    kind: EdgeKind::OptionalBranch,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["rejoin".to_owned()],
                },
                Edge {
                    id: "edge.hub_1.cache_1".to_owned(),
                    from: "hub.central_1".to_owned(),
                    to: "treasure.cache_1".to_owned(),
                    kind: EdgeKind::OptionalBranch,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["branch".to_owned()],
                },
                Edge {
                    id: "edge.cache_1.hub_1".to_owned(),
                    from: "treasure.cache_1".to_owned(),
                    to: "hub.central_1".to_owned(),
                    kind: EdgeKind::OptionalBranch,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["return".to_owned()],
                },
            ]);
        }
        GraphRule::NestedLockKeyChain => {
            if has_node(candidate, "gate.locked_2") {
                diagnostics.push(fatal_with_hint(
                    "rule_already_applied",
                    Some("gate.locked_2"),
                    None,
                    "nested_lock_key_chain is already present.",
                    "Nested locks use fixed gate/key ids; start from a candidate before this rule.",
                ));
                return diagnostics;
            }
            if !has_node(candidate, "gate.locked_1") {
                diagnostics.push(fatal_with_hint(
                    "missing_required_pattern",
                    Some("gate.locked_1"),
                    None,
                    "nested_lock_key_chain requires an existing first lock/key loop.",
                    "Apply lock_key_loop before nested_lock_key_chain.",
                ));
                return diagnostics;
            }
            candidate
                .graph
                .edges
                .retain(|edge| edge.id != "edge.gate_1.goal");
            candidate.graph.nodes.extend([
                Node {
                    id: "gate.locked_2".to_owned(),
                    kind: NodeKind::Gate,
                    label: "Inner Locked Gate".to_owned(),
                    tags: vec!["lock".to_owned(), "critical".to_owned()],
                    grants_item: None,
                },
                Node {
                    id: "key.deep_2".to_owned(),
                    kind: NodeKind::Key,
                    label: "Inner Gate Key".to_owned(),
                    tags: vec![
                        "key".to_owned(),
                        "branch".to_owned(),
                        "preparation".to_owned(),
                    ],
                    grants_item: Some("item.deep_key_2".to_owned()),
                },
            ]);
            candidate.graph.edges.extend([
                Edge {
                    id: "edge.gate_1.gate_2".to_owned(),
                    from: "gate.locked_1".to_owned(),
                    to: "gate.locked_2".to_owned(),
                    kind: EdgeKind::CriticalPath,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["approach".to_owned()],
                },
                Edge {
                    id: "edge.gate_2.goal".to_owned(),
                    from: "gate.locked_2".to_owned(),
                    to: "goal".to_owned(),
                    kind: EdgeKind::CriticalPath,
                    traversal: TraversalKind::Locked,
                    required_item: Some("item.deep_key_2".to_owned()),
                    tags: vec!["locked".to_owned()],
                },
                Edge {
                    id: "edge.gate_1.key_2".to_owned(),
                    from: "gate.locked_1".to_owned(),
                    to: "key.deep_2".to_owned(),
                    kind: EdgeKind::KeyBranch,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["branch".to_owned()],
                },
                Edge {
                    id: "edge.key_2.gate_2".to_owned(),
                    from: "key.deep_2".to_owned(),
                    to: "gate.locked_2".to_owned(),
                    kind: EdgeKind::KeyBranch,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["return".to_owned()],
                },
            ]);
        }
        GraphRule::HazardResourceTradeoff => {
            if has_node(candidate, "hazard.sluice_1") {
                diagnostics.push(fatal_with_hint(
                    "rule_already_applied",
                    Some("hazard.sluice_1"),
                    None,
                    "hazard_resource_tradeoff is already present.",
                    "Apply a different pressure pattern or use a fresh candidate.",
                ));
                return diagnostics;
            }
            candidate.graph.nodes.extend([
                Node {
                    id: "hazard.sluice_1".to_owned(),
                    kind: NodeKind::Hazard,
                    label: "Flooded Sluice".to_owned(),
                    tags: vec!["optional".to_owned(), "hazard".to_owned()],
                    grants_item: None,
                },
                Node {
                    id: "resource.safety_1".to_owned(),
                    kind: NodeKind::Resource,
                    label: "Safety Cache".to_owned(),
                    tags: vec!["optional".to_owned(), "preparation".to_owned()],
                    grants_item: Some("item.safety_cache_1".to_owned()),
                },
            ]);
            candidate.graph.edges.extend([
                Edge {
                    id: "edge.start.safety_1".to_owned(),
                    from: "start".to_owned(),
                    to: "resource.safety_1".to_owned(),
                    kind: EdgeKind::OptionalBranch,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["branch".to_owned()],
                },
                Edge {
                    id: "edge.safety_1.sluice_1".to_owned(),
                    from: "resource.safety_1".to_owned(),
                    to: "hazard.sluice_1".to_owned(),
                    kind: EdgeKind::OptionalBranch,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["preparation".to_owned()],
                },
                Edge {
                    id: "edge.start.sluice_1".to_owned(),
                    from: "start".to_owned(),
                    to: "hazard.sluice_1".to_owned(),
                    kind: EdgeKind::OptionalBranch,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["branch".to_owned(), "pressure".to_owned()],
                },
                Edge {
                    id: "edge.sluice_1.goal".to_owned(),
                    from: "hazard.sluice_1".to_owned(),
                    to: "goal".to_owned(),
                    kind: EdgeKind::OptionalBranch,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["rejoin".to_owned()],
                },
            ]);
        }
        GraphRule::BossPreparationLoop => {
            if has_node(candidate, "gate.boss_1") {
                diagnostics.push(fatal_with_hint(
                    "rule_already_applied",
                    Some("gate.boss_1"),
                    None,
                    "boss_preparation_loop is already present.",
                    "Boss preparation currently uses one fixed boss gate per candidate.",
                ));
                return diagnostics;
            }
            let approach_from = if has_node(candidate, "gate.locked_2") {
                candidate
                    .graph
                    .edges
                    .retain(|edge| edge.id != "edge.gate_2.goal");
                "gate.locked_2"
            } else if has_node(candidate, "gate.locked_1") {
                candidate
                    .graph
                    .edges
                    .retain(|edge| edge.id != "edge.gate_1.goal");
                "gate.locked_1"
            } else {
                candidate
                    .graph
                    .edges
                    .retain(|edge| edge.id != "edge.start.goal");
                "start"
            };
            candidate.graph.nodes.extend([
                Node {
                    id: "gate.boss_1".to_owned(),
                    kind: NodeKind::Gate,
                    label: "Boss Threshold".to_owned(),
                    tags: vec!["boss".to_owned(), "critical".to_owned()],
                    grants_item: None,
                },
                Node {
                    id: "resource.boss_prep_1".to_owned(),
                    kind: NodeKind::Resource,
                    label: "Boss Preparation".to_owned(),
                    tags: vec!["preparation".to_owned(), "optional".to_owned()],
                    grants_item: Some("item.boss_preparation_1".to_owned()),
                },
            ]);
            candidate.graph.edges.extend([
                Edge {
                    id: "edge.approach.boss_1".to_owned(),
                    from: approach_from.to_owned(),
                    to: "gate.boss_1".to_owned(),
                    kind: EdgeKind::CriticalPath,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["approach".to_owned()],
                },
                Edge {
                    id: "edge.boss_1.goal".to_owned(),
                    from: "gate.boss_1".to_owned(),
                    to: "goal".to_owned(),
                    kind: EdgeKind::CriticalPath,
                    traversal: TraversalKind::Locked,
                    required_item: Some("item.boss_preparation_1".to_owned()),
                    tags: vec!["locked".to_owned(), "boss".to_owned()],
                },
                Edge {
                    id: "edge.approach.boss_prep_1".to_owned(),
                    from: approach_from.to_owned(),
                    to: "resource.boss_prep_1".to_owned(),
                    kind: EdgeKind::OptionalBranch,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["branch".to_owned(), "preparation".to_owned()],
                },
                Edge {
                    id: "edge.boss_prep_1.boss_1".to_owned(),
                    from: "resource.boss_prep_1".to_owned(),
                    to: "gate.boss_1".to_owned(),
                    kind: EdgeKind::OptionalBranch,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["return".to_owned()],
                },
            ]);
        }
        GraphRule::GatedTreasureBranch => {
            if has_node(candidate, "treasure.gated_1") {
                diagnostics.push(fatal_with_hint(
                    "rule_already_applied",
                    Some("treasure.gated_1"),
                    None,
                    "gated_treasure_branch is already present.",
                    "Use optional_treasure_detour for repeatable reward branches.",
                ));
                return diagnostics;
            }
            candidate.graph.nodes.extend([
                Node {
                    id: "key.treasure_1".to_owned(),
                    kind: NodeKind::Key,
                    label: "Treasure Key".to_owned(),
                    tags: vec!["key".to_owned(), "optional".to_owned()],
                    grants_item: Some("item.treasure_key_1".to_owned()),
                },
                Node {
                    id: "treasure.gated_1".to_owned(),
                    kind: NodeKind::Treasure,
                    label: "Gated Treasure".to_owned(),
                    tags: vec!["optional".to_owned(), "reward".to_owned()],
                    grants_item: None,
                },
            ]);
            candidate.graph.edges.extend([
                Edge {
                    id: "edge.start.treasure_key_1".to_owned(),
                    from: "start".to_owned(),
                    to: "key.treasure_1".to_owned(),
                    kind: EdgeKind::KeyBranch,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["branch".to_owned()],
                },
                Edge {
                    id: "edge.treasure_key_1.treasure_1".to_owned(),
                    from: "key.treasure_1".to_owned(),
                    to: "treasure.gated_1".to_owned(),
                    kind: EdgeKind::OptionalBranch,
                    traversal: TraversalKind::Locked,
                    required_item: Some("item.treasure_key_1".to_owned()),
                    tags: vec!["locked".to_owned(), "branch".to_owned()],
                },
                Edge {
                    id: "edge.treasure_1.goal".to_owned(),
                    from: "treasure.gated_1".to_owned(),
                    to: "goal".to_owned(),
                    kind: EdgeKind::OptionalBranch,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["rejoin".to_owned()],
                },
            ]);
        }
        GraphRule::BranchMergeShortcut => {
            if has_node(candidate, "junction.merge_1") {
                diagnostics.push(fatal_with_hint(
                    "rule_already_applied",
                    Some("junction.merge_1"),
                    None,
                    "branch_merge_shortcut is already present.",
                    "Merge shortcuts use a fixed merge node until batch generation adds variant ids.",
                ));
                return diagnostics;
            }
            let secondary_source = if has_node(candidate, "hub.central_1") {
                "hub.central_1"
            } else if has_node(candidate, "treasure.gated_1") {
                "treasure.gated_1"
            } else if has_node(candidate, "key.gate_1") {
                "key.gate_1"
            } else {
                diagnostics.push(fatal_with_hint(
                    "missing_required_pattern",
                    None,
                    None,
                    "branch_merge_shortcut needs an existing branch or hub to merge.",
                    "Apply hub_spoke_cluster, gated_treasure_branch, or lock_key_loop first.",
                ));
                return diagnostics;
            };
            candidate.graph.nodes.push(Node {
                id: "junction.merge_1".to_owned(),
                kind: NodeKind::Junction,
                label: "Branch Merge".to_owned(),
                tags: vec!["merge".to_owned(), "wayfinding_anchor".to_owned()],
                grants_item: None,
            });
            candidate.graph.edges.extend([
                Edge {
                    id: "edge.start.merge_1".to_owned(),
                    from: "start".to_owned(),
                    to: "junction.merge_1".to_owned(),
                    kind: EdgeKind::OptionalBranch,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["branch".to_owned()],
                },
                Edge {
                    id: "edge.secondary.merge_1".to_owned(),
                    from: secondary_source.to_owned(),
                    to: "junction.merge_1".to_owned(),
                    kind: EdgeKind::OptionalBranch,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["rejoin".to_owned()],
                },
                Edge {
                    id: "edge.merge_1.goal.shortcut".to_owned(),
                    from: "junction.merge_1".to_owned(),
                    to: "goal".to_owned(),
                    kind: EdgeKind::Shortcut,
                    traversal: TraversalKind::Open,
                    required_item: None,
                    tags: vec!["shortcut".to_owned(), "rejoin".to_owned()],
                },
            ]);
        }
    }
    diagnostics
}

fn has_node(candidate: &Candidate, node_id: &str) -> bool {
    candidate.graph.nodes.iter().any(|node| node.id == node_id)
}

fn has_edge(candidate: &Candidate, edge_id: &str) -> bool {
    candidate.graph.edges.iter().any(|edge| edge.id == edge_id)
}

fn graph_rules_command(args: RuleMetadataArgs) -> Result<(), String> {
    let report = rule_metadata_report();
    if let Some(out) = args.out {
        write_json(&out, &report)
    } else {
        let text = serde_json::to_string_pretty(&report)
            .map_err(|error| format!("failed to encode rule metadata: {error}"))?;
        println!("{text}");
        Ok(())
    }
}

fn rule_metadata_report() -> RuleMetadataReport {
    RuleMetadataReport {
        kind: "asha_procgen.rule_metadata.v1".to_owned(),
        schema_version: 1,
        rules: vec![
            RuleMetadata {
                id: GraphRule::LockKeyLoop.as_str().to_owned(),
                intent: "Replace direct start-goal route with a locked gate and reachable key branch."
                    .to_owned(),
                required_patterns: Vec::new(),
                duplicate_markers: vec!["gate.locked_1".to_owned()],
                emitted_node_tags: vec!["critical".to_owned(), "key".to_owned(), "lock".to_owned()],
                emitted_edge_tags: vec!["approach".to_owned(), "branch".to_owned(), "return".to_owned()],
                compatibility_hints: vec![
                    "Useful first structural rule for nested locks and boss approaches.".to_owned(),
                ],
                repair_hints: vec!["Apply before nested_lock_key_chain.".to_owned()],
            },
            RuleMetadata {
                id: GraphRule::OptionalTreasureDetour.as_str().to_owned(),
                intent: "Add a repeatable optional reward detour that rejoins the goal route.".to_owned(),
                required_patterns: Vec::new(),
                duplicate_markers: Vec::new(),
                emitted_node_tags: vec!["optional".to_owned(), "reward".to_owned()],
                emitted_edge_tags: vec!["detour".to_owned(), "rejoin".to_owned()],
                compatibility_hints: vec![
                    "Seed-derived ids allow multiple treasure detours when seeds differ.".to_owned(),
                ],
                repair_hints: vec!["Use when score needs branch value without changing critical path.".to_owned()],
            },
            RuleMetadata {
                id: GraphRule::OneWayShortcut.as_str().to_owned(),
                intent: "Add a one-way return route from goal back to start.".to_owned(),
                required_patterns: Vec::new(),
                duplicate_markers: vec!["edge.goal.start.shortcut".to_owned()],
                emitted_node_tags: vec!["shortcut".to_owned()],
                emitted_edge_tags: vec!["return".to_owned(), "shortcut".to_owned()],
                compatibility_hints: vec![
                    "Best after the critical route is already meaningful.".to_owned(),
                ],
                repair_hints: vec!["Start from a pre-shortcut candidate if duplicate rejected.".to_owned()],
            },
            RuleMetadata {
                id: GraphRule::SecretBypass.as_str().to_owned(),
                intent: "Add a hidden optional bypass from start to goal.".to_owned(),
                required_patterns: Vec::new(),
                duplicate_markers: vec!["edge.start.goal.secret".to_owned()],
                emitted_node_tags: vec!["bypass".to_owned(), "secret".to_owned()],
                emitted_edge_tags: vec!["bypass".to_owned(), "hidden".to_owned()],
                compatibility_hints: vec![
                    "Can reduce perceived lock importance; use when bypasses are desired.".to_owned(),
                ],
                repair_hints: vec!["Avoid if selection wants strict lock/key progression.".to_owned()],
            },
            RuleMetadata {
                id: GraphRule::HubSpokeCluster.as_str().to_owned(),
                intent: "Create a wayfinding hub with optional spokes, returns, and rejoin routes."
                    .to_owned(),
                required_patterns: Vec::new(),
                duplicate_markers: vec!["hub.central_1".to_owned()],
                emitted_node_tags: vec![
                    "hazard".to_owned(),
                    "hub".to_owned(),
                    "merge".to_owned(),
                    "optional".to_owned(),
                    "preparation".to_owned(),
                    "reward".to_owned(),
                    "wayfinding_anchor".to_owned(),
                ],
                emitted_edge_tags: vec![
                    "approach".to_owned(),
                    "branch".to_owned(),
                    "pressure".to_owned(),
                    "rejoin".to_owned(),
                    "return".to_owned(),
                ],
                compatibility_hints: vec![
                    "Good early rule when an agent needs local choice and orientation.".to_owned(),
                ],
                repair_hints: vec![
                    "If hub diagnostics fire, add return/rejoin routes or a wayfinding anchor.".to_owned(),
                ],
            },
            RuleMetadata {
                id: GraphRule::NestedLockKeyChain.as_str().to_owned(),
                intent: "Add a second gate/key layer behind the first lock.".to_owned(),
                required_patterns: vec!["lock_key_loop".to_owned()],
                duplicate_markers: vec!["gate.locked_2".to_owned()],
                emitted_node_tags: vec![
                    "branch".to_owned(),
                    "critical".to_owned(),
                    "key".to_owned(),
                    "lock".to_owned(),
                    "preparation".to_owned(),
                ],
                emitted_edge_tags: vec!["approach".to_owned(), "branch".to_owned(), "locked".to_owned(), "return".to_owned()],
                compatibility_hints: vec![
                    "Requires gate.locked_1; apply lock_key_loop first.".to_owned(),
                ],
                repair_hints: vec!["Move key branches before locked edges if validation fails.".to_owned()],
            },
            RuleMetadata {
                id: GraphRule::HazardResourceTradeoff.as_str().to_owned(),
                intent: "Add a pressure branch paired with a preparation resource.".to_owned(),
                required_patterns: Vec::new(),
                duplicate_markers: vec!["hazard.sluice_1".to_owned()],
                emitted_node_tags: vec!["hazard".to_owned(), "optional".to_owned(), "preparation".to_owned()],
                emitted_edge_tags: vec!["branch".to_owned(), "preparation".to_owned(), "pressure".to_owned(), "rejoin".to_owned()],
                compatibility_hints: vec![
                    "Pairs well with hubs and boss preparation loops.".to_owned(),
                ],
                repair_hints: vec!["Add rejoin edges after hazards if diagnostics report terminal pressure.".to_owned()],
            },
            RuleMetadata {
                id: GraphRule::BossPreparationLoop.as_str().to_owned(),
                intent: "Insert a boss gate with a preparation branch returning to the approach.".to_owned(),
                required_patterns: Vec::new(),
                duplicate_markers: vec!["gate.boss_1".to_owned()],
                emitted_node_tags: vec!["boss".to_owned(), "critical".to_owned(), "optional".to_owned(), "preparation".to_owned()],
                emitted_edge_tags: vec!["approach".to_owned(), "boss".to_owned(), "branch".to_owned(), "locked".to_owned(), "preparation".to_owned(), "return".to_owned()],
                compatibility_hints: vec![
                    "Uses deepest known lock gate as approach if one exists.".to_owned(),
                ],
                repair_hints: vec!["Keep preparation reachable before the boss locked edge.".to_owned()],
            },
            RuleMetadata {
                id: GraphRule::GatedTreasureBranch.as_str().to_owned(),
                intent: "Add an optional key-gated treasure branch that rejoins progression.".to_owned(),
                required_patterns: Vec::new(),
                duplicate_markers: vec!["treasure.gated_1".to_owned()],
                emitted_node_tags: vec!["key".to_owned(), "optional".to_owned(), "reward".to_owned()],
                emitted_edge_tags: vec!["branch".to_owned(), "locked".to_owned(), "rejoin".to_owned()],
                compatibility_hints: vec![
                    "Useful when a candidate needs reward tension without blocking the goal.".to_owned(),
                ],
                repair_hints: vec!["Ensure the treasure key remains reachable before the gated reward.".to_owned()],
            },
            RuleMetadata {
                id: GraphRule::BranchMergeShortcut.as_str().to_owned(),
                intent: "Add a merge node and shortcut from an existing branch or hub back to goal.".to_owned(),
                required_patterns: vec![
                    "hub_spoke_cluster or gated_treasure_branch or lock_key_loop".to_owned(),
                ],
                duplicate_markers: vec!["junction.merge_1".to_owned()],
                emitted_node_tags: vec!["merge".to_owned(), "wayfinding_anchor".to_owned()],
                emitted_edge_tags: vec!["branch".to_owned(), "rejoin".to_owned(), "shortcut".to_owned()],
                compatibility_hints: vec![
                    "Requires an upstream branch source; hub_spoke_cluster is the clearest pairing.".to_owned(),
                ],
                repair_hints: vec!["Apply a branch or hub rule first if missing_required_pattern is reported.".to_owned()],
            },
        ],
    }
}

fn summarize_candidate(args: SummarizeArgs) -> Result<(), String> {
    let candidate: Candidate = read_json(&args.state)?;
    let report = validate_graph(&candidate);
    let score = score_graph(&candidate);
    if args.json || args.out.is_some() {
        let summary = graph_summary_report(&candidate, &report, &score)?;
        if let Some(out) = args.out {
            write_json(&out, &summary)?;
        } else {
            let text = serde_json::to_string_pretty(&summary)
                .map_err(|error| format!("failed to encode graph summary: {error}"))?;
            println!("{text}");
        }
        return Ok(());
    }
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

fn graph_summary_report(
    candidate: &Candidate,
    validation: &ValidationReport,
    score: &ScoreReport,
) -> Result<GraphSummaryReport, String> {
    let mut locked_items = BTreeSet::new();
    for edge in &candidate.graph.edges {
        if let Some(item) = edge.required_item.as_deref() {
            locked_items.insert(item.to_owned());
        }
    }
    let dead_ends = candidate
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
        .map(|node| node.id.clone())
        .collect();
    let provenance_tail = candidate
        .provenance
        .iter()
        .rev()
        .take(5)
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    Ok(GraphSummaryReport {
        kind: "asha_procgen.graph_summary.v1".to_owned(),
        schema_version: 1,
        candidate_id: candidate.candidate_id.clone(),
        state_hash: hash_json(candidate)?,
        validation_ok: validation.ok,
        fatal_count: validation.fatal_count,
        score_overall: score.overall,
        metrics: score.metrics.clone(),
        node_count: candidate.graph.nodes.len(),
        edge_count: candidate.graph.edges.len(),
        tags: collect_tags(candidate),
        locked_items: locked_items.into_iter().collect(),
        dead_ends,
        provenance_tail,
        nodes: candidate
            .graph
            .nodes
            .iter()
            .map(|node| NodeSummary {
                id: node.id.clone(),
                kind: node.kind,
                tags: node.tags.clone(),
                grants_item: node.grants_item.clone(),
            })
            .collect(),
        edges: candidate
            .graph
            .edges
            .iter()
            .map(|edge| EdgeSummary {
                id: edge.id.clone(),
                from: edge.from.clone(),
                to: edge.to.clone(),
                kind: edge.kind,
                traversal: edge.traversal,
                required_item: edge.required_item.clone(),
                tags: edge.tags.clone(),
            })
            .collect(),
    })
}

fn analyze_graph_command(args: ReportOutArgs) -> Result<(), String> {
    let candidate: Candidate = read_json(&args.state)?;
    let report = analyze_graph(&candidate)?;
    write_json(&args.out, &report)
}

fn analyze_graph(candidate: &Candidate) -> Result<GraphAnalysisReport, String> {
    let critical_path = shortest_path_nodes(candidate, "start", "goal").unwrap_or_default();
    let dominators = dominator_nodes(candidate);
    let optional_branches = candidate
        .graph
        .edges
        .iter()
        .filter(|edge| edge.kind == EdgeKind::OptionalBranch || edge.kind == EdgeKind::SecretBypass)
        .map(|edge| BranchAnalysis {
            edge_id: edge.id.clone(),
            from: edge.from.clone(),
            to: edge.to.clone(),
            classification: if edge_has_tag(edge, "pressure") {
                "pressure".to_owned()
            } else if edge_has_tag(edge, "rejoin") || edge_has_tag(edge, "return") {
                "rejoin".to_owned()
            } else {
                "optional".to_owned()
            },
            rejoins_goal_route: path_exists(candidate, edge.to.as_str(), "goal"),
        })
        .collect();
    let lock_key_order = candidate
        .graph
        .edges
        .iter()
        .filter_map(|edge| {
            let required_item = edge.required_item.clone()?;
            let provider = candidate
                .graph
                .nodes
                .iter()
                .find(|node| node.grants_item.as_deref() == Some(required_item.as_str()));
            let provider_node = provider.map(|node| node.id.clone());
            let provider_reachable_before_lock = provider.is_some_and(|node| {
                shortest_path_len(candidate, "start", node.id.as_str())
                    .zip(shortest_path_len(candidate, "start", edge.from.as_str()))
                    .is_some_and(|(provider_depth, lock_depth)| provider_depth <= lock_depth + 2)
            });
            Some(LockKeyAnalysis {
                edge_id: edge.id.clone(),
                required_item,
                provider_node,
                provider_reachable_before_lock,
            })
        })
        .collect();
    let loop_signals = candidate
        .graph
        .edges
        .iter()
        .filter(|edge| {
            edge_has_tag(edge, "return")
                || edge_has_tag(edge, "rejoin")
                || edge.kind == EdgeKind::Shortcut
        })
        .map(|edge| LoopSignal {
            edge_id: edge.id.clone(),
            signal: if edge.kind == EdgeKind::Shortcut {
                "shortcut_loop".to_owned()
            } else {
                "rejoin_loop".to_owned()
            },
            detail: format!("{} reconnects {} to {}", edge.id, edge.from, edge.to),
        })
        .collect();
    let locked_count = candidate
        .graph
        .edges
        .iter()
        .filter(|edge| edge.traversal == TraversalKind::Locked)
        .count();
    let shortcut_bypass_risks = candidate
        .graph
        .edges
        .iter()
        .filter(|edge| edge.kind == EdgeKind::Shortcut || edge.kind == EdgeKind::SecretBypass)
        .map(|edge| ShortcutRisk {
            edge_id: edge.id.clone(),
            risk: if locked_count > 0 && path_exists(candidate, edge.to.as_str(), "goal") {
                "may_bypass_lock".to_owned()
            } else {
                "low".to_owned()
            },
            detail: format!(
                "{} can compress route from {} to {}",
                edge.id, edge.from, edge.to
            ),
        })
        .collect();
    Ok(GraphAnalysisReport {
        kind: "asha_procgen.graph_analysis.v1".to_owned(),
        schema_version: 1,
        candidate_id: candidate.candidate_id.clone(),
        state_hash: hash_json(candidate)?,
        critical_path,
        dominators,
        optional_branches,
        lock_key_order,
        loop_signals,
        shortcut_bypass_risks,
    })
}

fn compatible_rules_command(args: ReportOutArgs) -> Result<(), String> {
    let candidate: Candidate = read_json(&args.state)?;
    let report = compatible_rules_report(&candidate)?;
    write_json(&args.out, &report)
}

fn compatible_rules_report(candidate: &Candidate) -> Result<RuleCompatibilityReport, String> {
    Ok(RuleCompatibilityReport {
        kind: "asha_procgen.rule_compatibility.v1".to_owned(),
        schema_version: 1,
        candidate_id: candidate.candidate_id.clone(),
        state_hash: hash_json(candidate)?,
        rules: all_graph_rules()
            .into_iter()
            .map(|rule| rule_compatibility(candidate, rule))
            .collect(),
    })
}

fn all_graph_rules() -> Vec<GraphRule> {
    vec![
        GraphRule::LockKeyLoop,
        GraphRule::OptionalTreasureDetour,
        GraphRule::OneWayShortcut,
        GraphRule::SecretBypass,
        GraphRule::HubSpokeCluster,
        GraphRule::NestedLockKeyChain,
        GraphRule::HazardResourceTradeoff,
        GraphRule::BossPreparationLoop,
        GraphRule::GatedTreasureBranch,
        GraphRule::BranchMergeShortcut,
    ]
}

fn rule_compatibility(candidate: &Candidate, rule: GraphRule) -> RuleCompatibility {
    let mut status = "applicable".to_owned();
    let mut reasons = Vec::new();
    let mut recommended_actions = Vec::new();
    let duplicate = match rule {
        GraphRule::LockKeyLoop => has_node(candidate, "gate.locked_1"),
        GraphRule::OneWayShortcut => {
            has_edge(candidate, "edge.goal.start.shortcut")
                || has_edge(candidate, "edge.goal.shortcut_1")
                || has_edge(candidate, "edge.shortcut_1.start")
        }
        GraphRule::SecretBypass => has_edge(candidate, "edge.start.goal.secret"),
        GraphRule::HubSpokeCluster => has_node(candidate, "hub.central_1"),
        GraphRule::NestedLockKeyChain => has_node(candidate, "gate.locked_2"),
        GraphRule::HazardResourceTradeoff => has_node(candidate, "hazard.sluice_1"),
        GraphRule::BossPreparationLoop => has_node(candidate, "gate.boss_1"),
        GraphRule::GatedTreasureBranch => has_node(candidate, "treasure.gated_1"),
        GraphRule::BranchMergeShortcut => has_node(candidate, "junction.merge_1"),
        GraphRule::OptionalTreasureDetour => false,
    };
    if duplicate {
        status = "duplicate".to_owned();
        reasons.push("Fixed marker for this rule already exists.".to_owned());
        recommended_actions
            .push("Fork from an earlier candidate or choose another rule.".to_owned());
    }
    if rule == GraphRule::NestedLockKeyChain && !has_node(candidate, "gate.locked_1") {
        status = "blocked".to_owned();
        reasons.push("Requires lock_key_loop / gate.locked_1 first.".to_owned());
        recommended_actions.push("Apply lock_key_loop before nested_lock_key_chain.".to_owned());
    }
    if rule == GraphRule::BranchMergeShortcut
        && !(has_node(candidate, "hub.central_1")
            || has_node(candidate, "treasure.gated_1")
            || has_node(candidate, "key.gate_1"))
    {
        status = "blocked".to_owned();
        reasons.push("Requires an existing branch, hub, or key route to merge.".to_owned());
        recommended_actions.push(
            "Apply hub_spoke_cluster, gated_treasure_branch, or lock_key_loop first.".to_owned(),
        );
    }
    if status == "applicable"
        && rule == GraphRule::SecretBypass
        && candidate
            .graph
            .edges
            .iter()
            .any(|edge| edge.traversal == TraversalKind::Locked)
    {
        status = "risky".to_owned();
        reasons.push("Secret bypass may trivialize existing locked progression.".to_owned());
        recommended_actions.push("Use only when bypass routes are intended.".to_owned());
    }
    RuleCompatibility {
        rule: rule.as_str().to_owned(),
        status,
        reasons,
        recommended_actions,
    }
}

fn annotate_spatial_intent_command(args: AnnotateSpatialIntentArgs) -> Result<(), String> {
    let candidate: Candidate = read_json(&args.state)?;
    if let Some(analysis) = args.analysis.as_deref() {
        let _: GraphAnalysisReport = read_json(analysis)?;
    }
    let report = spatial_intent_report(&candidate, args.analysis.as_deref())?;
    write_json(&args.out, &report)
}

fn spatial_intent_report(
    candidate: &Candidate,
    analysis_ref: Option<&Path>,
) -> Result<SpatialIntentReport, String> {
    let mut annotations = Vec::new();
    for node in &candidate.graph.nodes {
        let mut intents = Vec::new();
        if node.kind == NodeKind::Start {
            intents.push("entry_orientation".to_owned());
        }
        if node.kind == NodeKind::Goal {
            intents.push("destination_readability".to_owned());
        }
        if node_has_tag(node, "hub") || node_has_tag(node, "wayfinding_anchor") {
            intents.push("landmark_hub".to_owned());
        }
        if node_has_tag(node, "boss") {
            intents.push("gated_reveal".to_owned());
        }
        if node_has_tag(node, "hazard") {
            intents.push("pressure_path".to_owned());
        }
        if node_has_tag(node, "lock") || node.kind == NodeKind::Gate {
            intents.push("gated_reveal".to_owned());
        }
        if node_has_tag(node, "reward") {
            intents.push("reward_pocket".to_owned());
        }
        let intents = dedupe_strings(intents);
        if !intents.is_empty() {
            annotations.push(SpatialIntentAnnotation {
                target_type: "node".to_owned(),
                target_id: node.id.clone(),
                rationale: format!("Node {} carries spatial role tags.", node.id),
                intents,
            });
        }
    }
    for edge in &candidate.graph.edges {
        let mut intents = Vec::new();
        if edge.traversal == TraversalKind::Locked || edge.required_item.is_some() {
            intents.push("visible_before_reachable".to_owned());
            intents.push("gated_connector".to_owned());
        }
        if edge_has_tag(edge, "pressure") {
            intents.push("pressure_path".to_owned());
        }
        if edge.kind == EdgeKind::Shortcut || edge_has_tag(edge, "shortcut") {
            intents.push("shortcut_connector".to_owned());
        }
        if edge.traversal == TraversalKind::OneWayReturn {
            intents.push("one_way_drop".to_owned());
        }
        if edge.traversal == TraversalKind::Hidden || edge_has_tag(edge, "hidden") {
            intents.push("hidden_route".to_owned());
        }
        if edge_has_tag(edge, "return") || edge_has_tag(edge, "rejoin") {
            intents.push("merge_rejoin_clarity".to_owned());
        }
        if edge.kind == EdgeKind::SecretBypass {
            intents.push("hidden_route".to_owned());
        }
        let intents = dedupe_strings(intents);
        if !intents.is_empty() {
            annotations.push(SpatialIntentAnnotation {
                target_type: "edge".to_owned(),
                target_id: edge.id.clone(),
                rationale: format!("Edge {} has traversal or topology intent.", edge.id),
                intents,
            });
        }
    }
    Ok(SpatialIntentReport {
        kind: "asha_procgen.spatial_intent.v1".to_owned(),
        schema_version: 1,
        candidate_id: candidate.candidate_id.clone(),
        state_hash: hash_json(candidate)?,
        analysis_ref: analysis_ref.map(display_path),
        annotations,
    })
}

fn breakdown_emit_command(args: BreakdownEmitArgs) -> Result<(), String> {
    let candidate: Candidate = read_json(&args.state)?;
    let annotations: SpatialIntentReport = read_json(&args.annotations)?;
    let breakdown = intermediate_breakdown(&candidate, &annotations, &args.annotations)?;
    write_json(&args.out, &breakdown)
}

fn intermediate_breakdown(
    candidate: &Candidate,
    annotations: &SpatialIntentReport,
    annotation_path: &Path,
) -> Result<IntermediateBreakdown, String> {
    let mut annotations_by_target: BTreeMap<&str, Vec<&SpatialIntentAnnotation>> = BTreeMap::new();
    for annotation in &annotations.annotations {
        annotations_by_target
            .entry(annotation.target_id.as_str())
            .or_default()
            .push(annotation);
    }
    let regions = candidate
        .graph
        .nodes
        .iter()
        .map(|node| {
            let intents = annotations_by_target
                .get(node.id.as_str())
                .into_iter()
                .flat_map(|items| items.iter())
                .flat_map(|annotation| annotation.intents.iter().map(String::as_str))
                .collect::<BTreeSet<_>>();
            let role = region_role(node, &intents);
            let anchor_node = if matches!(
                role.as_str(),
                "start" | "goal" | "landmark_hub" | "boss_gate"
            ) {
                Some(node.id.clone())
            } else {
                None
            };
            IntermediateRegion {
                id: region_id(node.id.as_str()),
                node_ids: vec![node.id.clone()],
                role,
                anchor_node,
            }
        })
        .collect::<Vec<_>>();
    let connectors = candidate
        .graph
        .edges
        .iter()
        .map(|edge| {
            let intents = annotations_by_target
                .get(edge.id.as_str())
                .into_iter()
                .flat_map(|items| items.iter())
                .flat_map(|annotation| annotation.intents.clone())
                .collect::<Vec<_>>();
            IntermediateConnector {
                id: format!("connector.{}", slugify_label(edge.id.as_str())),
                edge_id: edge.id.clone(),
                from_region: region_id(edge.from.as_str()),
                to_region: region_id(edge.to.as_str()),
                intents: dedupe_strings(intents),
            }
        })
        .collect::<Vec<_>>();
    let constraints = annotations
        .annotations
        .iter()
        .flat_map(|annotation| {
            annotation
                .intents
                .iter()
                .filter_map(|intent| constraint_for_intent(annotation, intent))
                .collect::<Vec<_>>()
        })
        .collect();
    Ok(IntermediateBreakdown {
        kind: "asha_procgen.intermediate_breakdown.v1".to_owned(),
        schema_version: 1,
        candidate_id: candidate.candidate_id.clone(),
        state_hash: hash_json(candidate)?,
        annotation_ref: display_path(annotation_path),
        regions,
        connectors,
        constraints,
    })
}

fn region_role(node: &Node, intents: &BTreeSet<&str>) -> String {
    if node.kind == NodeKind::Start {
        "start".to_owned()
    } else if node.kind == NodeKind::Goal {
        "goal".to_owned()
    } else if intents.contains("landmark_hub") {
        "landmark_hub".to_owned()
    } else if node_has_tag(node, "boss") {
        "boss_gate".to_owned()
    } else if node_has_tag(node, "hazard") || intents.contains("pressure_path") {
        "pressure".to_owned()
    } else if node_has_tag(node, "reward") {
        "reward".to_owned()
    } else if node.kind == NodeKind::Gate {
        "gate".to_owned()
    } else {
        "standard".to_owned()
    }
}

fn constraint_for_intent(
    annotation: &SpatialIntentAnnotation,
    intent: &str,
) -> Option<IntermediateConstraint> {
    let code = match intent {
        "visible_before_reachable" => "preserve_lock_preview",
        "gated_reveal" => "preserve_reveal_sequence",
        "landmark_hub" => "preserve_wayfinding_anchor",
        "pressure_path" => "preserve_pressure_read",
        "one_way_drop" => "preserve_one_way_return",
        "hidden_route" => "preserve_hidden_route",
        _ => return None,
    };
    Some(IntermediateConstraint {
        code: code.to_owned(),
        target: annotation.target_id.clone(),
        detail: format!("Preserve {intent} for {}.", annotation.target_id),
    })
}

fn breakdown_validate_command(args: ReportOutArgs) -> Result<(), String> {
    let breakdown: IntermediateBreakdown = read_json(&args.state)?;
    let report = validate_intermediate_breakdown(&breakdown);
    write_json(&args.out, &report)?;
    if report.ok {
        Ok(())
    } else {
        Err(format!(
            "intermediate breakdown validation failed with {} fatal diagnostic(s); see {}",
            report.fatal_count,
            args.out.display()
        ))
    }
}

fn validate_intermediate_breakdown(breakdown: &IntermediateBreakdown) -> ValidationReport {
    let mut diagnostics = Vec::new();
    let region_ids = breakdown
        .regions
        .iter()
        .map(|region| region.id.as_str())
        .collect::<BTreeSet<_>>();
    if !breakdown
        .regions
        .iter()
        .any(|region| region.role == "start")
    {
        diagnostics.push(fatal(
            "intermediate_start_missing",
            Some("start"),
            None,
            "Intermediate breakdown must contain a start region.",
        ));
    }
    if !breakdown.regions.iter().any(|region| region.role == "goal") {
        diagnostics.push(fatal(
            "intermediate_goal_missing",
            Some("goal"),
            None,
            "Intermediate breakdown must contain a goal region.",
        ));
    }
    for region in &breakdown.regions {
        if region.role == "landmark_hub" && region.anchor_node.is_none() {
            diagnostics.push(fatal(
                "intermediate_anchor_missing",
                region.node_ids.first().map(String::as_str),
                None,
                "Landmark hub region must declare an anchor node.",
            ));
        }
    }
    for connector in &breakdown.connectors {
        if connector.edge_id.is_empty() {
            diagnostics.push(fatal(
                "intermediate_connector_unbound",
                None,
                Some(connector.id.as_str()),
                "Connector must be bound to a graph edge id.",
            ));
        }
        if !region_ids.contains(connector.from_region.as_str())
            || !region_ids.contains(connector.to_region.as_str())
        {
            diagnostics.push(fatal(
                "intermediate_connector_endpoint_missing",
                None,
                Some(connector.id.as_str()),
                "Connector references a missing source or target region.",
            ));
        }
        if connector
            .intents
            .iter()
            .any(|intent| intent == "vertical_candidate")
        {
            diagnostics.push(fatal(
                "intermediate_vertical_candidate_unsupported",
                None,
                Some(connector.id.as_str()),
                "Vertical candidates require a later geometry-capable schema.",
            ));
        }
    }
    let fatal_count = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == Severity::Fatal)
        .count();
    ValidationReport {
        kind: "asha_procgen.validation.intermediate.v1".to_owned(),
        schema_version: 1,
        state_hash: hash_json(breakdown).unwrap_or_else(|_| "hash_error".to_owned()),
        ok: fatal_count == 0,
        fatal_count,
        diagnostics,
    }
}

fn region_id(node_id: &str) -> String {
    format!("region.{}", slugify_label(node_id))
}

fn dedupe_strings(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
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

fn repair_suggest_command(args: ReportOutArgs) -> Result<(), String> {
    let candidate: Candidate = read_json(&args.state)?;
    let report = repair_report(&candidate)?;
    write_json(&args.out, &report)
}

fn repair_apply_command(args: RepairApplyArgs) -> Result<(), String> {
    let mut candidate: Candidate = read_json(&args.state)?;
    let input_hash = hash_file(&args.state)?;
    let diagnostics = apply_repair_action(
        &mut candidate,
        args.action,
        args.target.as_deref(),
        args.seed,
    );
    let has_fatal = diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Fatal);
    let status = if has_fatal {
        "rejected"
    } else {
        candidate.provenance.push(ProvenanceStep {
            step: candidate.provenance.len() as u32 + 1,
            command: format!("repair apply {}", args.action.as_str()),
            seed: Some(args.seed),
            summary: format!(
                "Applied {}{}",
                args.action.as_str(),
                args.target
                    .as_deref()
                    .map(|target| format!(" to {target}"))
                    .unwrap_or_default()
            ),
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
        command: format!("repair apply {}", args.action.as_str()),
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
        "repair apply",
        if status == "ok" {
            Some(&args.out)
        } else {
            None
        },
        Some(&args.receipt),
        Some(args.seed),
        json!({
            "state": display_path(&args.state),
            "action": args.action.as_str(),
            "target": args.target
        }),
    )?;
    if status == "ok" {
        Ok(())
    } else {
        Err("repair action was rejected; see receipt diagnostics".to_owned())
    }
}

fn apply_repair_action(
    candidate: &mut Candidate,
    action: RepairAction,
    target: Option<&str>,
    seed: u64,
) -> Vec<Diagnostic> {
    let Some(target) = target else {
        return vec![fatal(
            "repair_target_required",
            None,
            None,
            format!("{} requires --target <node_id>.", action.as_str()),
        )];
    };
    if !has_node(candidate, target) {
        return vec![fatal(
            "repair_target_missing",
            Some(target),
            None,
            "Repair target node does not exist.",
        )];
    }
    match action {
        RepairAction::AddRejoinEdge => apply_add_rejoin_edge(candidate, target, seed),
        RepairAction::RemoveOrphanNode => apply_remove_orphan_node(candidate, target),
    }
}

fn apply_add_rejoin_edge(candidate: &mut Candidate, target: &str, seed: u64) -> Vec<Diagnostic> {
    if target == "goal" {
        return vec![fatal(
            "repair_target_invalid",
            Some(target),
            None,
            "Goal does not need a rejoin edge.",
        )];
    }
    if candidate.graph.edges.iter().any(|edge| edge.from == target) {
        return vec![fatal_with_hint(
            "repair_target_ambiguous",
            Some(target),
            None,
            "Target already has outgoing routes.",
            "Use add_rejoin_edge only on simple terminal branch nodes.",
        )];
    }
    let edge_id = format!(
        "edge.repair.{}.goal.{}",
        slugify_label(target),
        stable_suffix(seed)
    );
    if has_edge(candidate, edge_id.as_str()) {
        return vec![fatal(
            "repair_edge_duplicate",
            None,
            Some(edge_id.as_str()),
            "Repair edge already exists.",
        )];
    }
    candidate.graph.edges.push(Edge {
        id: edge_id,
        from: target.to_owned(),
        to: "goal".to_owned(),
        kind: EdgeKind::OptionalBranch,
        traversal: TraversalKind::Open,
        required_item: None,
        tags: vec!["repair".to_owned(), "rejoin".to_owned()],
    });
    Vec::new()
}

fn apply_remove_orphan_node(candidate: &mut Candidate, target: &str) -> Vec<Diagnostic> {
    if target == "start" || target == "goal" {
        return vec![fatal(
            "repair_target_invalid",
            Some(target),
            None,
            "Start and goal nodes cannot be removed by bounded repair.",
        )];
    }
    if candidate.graph.edges.iter().any(|edge| edge.to == target) {
        return vec![fatal_with_hint(
            "repair_target_not_orphan",
            Some(target),
            None,
            "Target has incoming routes and is not an orphan.",
            "Use remove_orphan_node only for nodes with no incoming route.",
        )];
    }
    candidate.graph.nodes.retain(|node| node.id != target);
    candidate
        .graph
        .edges
        .retain(|edge| edge.from != target && edge.to != target);
    Vec::new()
}

fn repair_report(candidate: &Candidate) -> Result<RepairReport, String> {
    let validation = validate_graph(candidate);
    let mut suggestions: Vec<RepairSuggestion> = validation
        .diagnostics
        .iter()
        .map(repair_suggestion_for_diagnostic)
        .collect();
    suggestions.sort_by(|left, right| {
        severity_rank(left.severity)
            .cmp(&severity_rank(right.severity))
            .then_with(|| left.code.cmp(&right.code))
            .then_with(|| left.node.cmp(&right.node))
            .then_with(|| left.edge.cmp(&right.edge))
    });
    Ok(RepairReport {
        kind: "asha_procgen.repair_report.v1".to_owned(),
        schema_version: 1,
        candidate_id: candidate.candidate_id.clone(),
        state_hash: hash_json(candidate)?,
        validation_ok: validation.ok,
        fatal_count: validation.fatal_count,
        suggestions,
    })
}

fn repair_suggestion_for_diagnostic(diagnostic: &Diagnostic) -> RepairSuggestion {
    RepairSuggestion {
        code: diagnostic.code.clone(),
        severity: diagnostic.severity,
        node: diagnostic.node.clone(),
        edge: diagnostic.edge.clone(),
        detail: diagnostic.detail.clone(),
        repair_hint: diagnostic.repair_hint.clone(),
        suggested_actions: suggested_actions_for_diagnostic(diagnostic),
    }
}

fn severity_rank(severity: Severity) -> u8 {
    match severity {
        Severity::Fatal => 0,
        Severity::Warning => 1,
        Severity::Info => 2,
    }
}

fn suggested_actions_for_diagnostic(diagnostic: &Diagnostic) -> Vec<String> {
    match diagnostic.code.as_str() {
        "required_item_unavailable" => vec![
            "Inspect graph rules for a key/resource provider that grants the required item."
                .to_owned(),
            "Fork the candidate before adding or moving a provider branch.".to_owned(),
        ],
        "locked_edge_never_traversed" => vec![
            "Move the provider branch before the locked edge or add an open route to the provider."
                .to_owned(),
            "Run validate graph again before scoring.".to_owned(),
        ],
        "goal_unreachable" => vec![
            "Reconnect the critical path from start to goal under current lock constraints."
                .to_owned(),
            "Use graph summarize --json to inspect dead ends and locked items.".to_owned(),
        ],
        "missing_required_pattern" => vec![
            "Run graph rules and apply the prerequisite pattern before retrying this rule."
                .to_owned(),
            "Fork from an earlier candidate if the prerequisite would conflict with current structure."
                .to_owned(),
        ],
        "hub_incident_edges_low" | "hub_missing_return_or_rejoin" => vec![
            "Add or repair hub spokes so at least one branch returns or rejoins.".to_owned(),
            "Prefer hub_spoke_cluster on a fork when the current hub is too sparse.".to_owned(),
        ],
        "hub_missing_wayfinding_anchor" => {
            vec!["Tag the hub as wayfinding_anchor or replace it with hub_spoke_cluster.".to_owned()]
        }
        "boss_missing_preparation" | "boss_preparation_missing_return" => vec![
            "Add a reachable preparation resource before the boss approach.".to_owned(),
            "Ensure the preparation branch returns or rejoins at the boss gate.".to_owned(),
        ],
        "hazard_missing_rejoin" => {
            vec!["Add a rejoin/return edge after the hazard or remove the terminal pressure branch."
                .to_owned()]
        }
        "merge_upstream_routes_low" => {
            vec!["Add a second upstream branch route before treating this node as a merge.".to_owned()]
        }
        "non_goal_dead_end" => vec![
            "Add a return/rejoin edge unless this is an intentional terminal reward.".to_owned(),
            diagnostic
                .node
                .as_deref()
                .map(|node| format!("Run repair apply --action add_rejoin_edge --target {node}."))
                .unwrap_or_else(|| {
                    "Run repair apply --action add_rejoin_edge with a terminal node target."
                        .to_owned()
                }),
        ],
        "orphan_node" => vec![
            "Add an incoming approach or branch edge from a reachable node.".to_owned(),
            diagnostic
                .node
                .as_deref()
                .map(|node| format!("Run repair apply --action remove_orphan_node --target {node}."))
                .unwrap_or_else(|| {
                    "Run repair apply --action remove_orphan_node with the orphan node target."
                        .to_owned()
                }),
        ],
        "rule_already_applied" => {
            vec!["Fork from an earlier candidate or choose a rule with seed-derived ids.".to_owned()]
        }
        _ => diagnostic
            .repair_hint
            .iter()
            .map(|hint| format!("Use repair hint: {hint}"))
            .collect(),
    }
}

fn validate_graph(candidate: &Candidate) -> ValidationReport {
    let mut diagnostics = Vec::new();
    let node_ids: BTreeSet<&str> = candidate
        .graph
        .nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect();
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
            diagnostics.push(fatal_with_hint(
                "edge_from_missing",
                None,
                Some(edge.id.as_str()),
                "Edge source node is missing.",
                "Create the source node or remove this edge before applying more rules.",
            ));
        }
        if !node_ids.contains(edge.to.as_str()) {
            diagnostics.push(fatal_with_hint(
                "edge_to_missing",
                None,
                Some(edge.id.as_str()),
                "Edge target node is missing.",
                "Create the target node or remove this edge before applying more rules.",
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
                diagnostics.push(fatal_with_hint(
                    "required_item_unavailable",
                    None,
                    Some(edge.id.as_str()),
                    format!("Edge requires {required_item}, but no node grants it."),
                    "Add a reachable key/resource node that grants the required item.",
                ));
            }
        }
    }

    if start_count == 1 && goal_count == 1 {
        let reachable = reachable_with_items(candidate);
        if !reachable.goal_reached {
            diagnostics.push(fatal_with_hint(
                "goal_unreachable",
                Some("goal"),
                None,
                "Goal is not reachable under lock/key constraints.",
                "Add an open route, move item providers before locks, or reconnect the critical path.",
            ));
        }
        for edge in &candidate.graph.edges {
            if edge.traversal == TraversalKind::Locked
                && !reachable.traversed_edges.contains(edge.id.as_str())
            {
                diagnostics.push(fatal_with_hint(
                    "locked_edge_never_traversed",
                    None,
                    Some(edge.id.as_str()),
                    "Locked edge could not be traversed after item collection.",
                    "Move the item provider earlier or add a branch that reaches it before the lock.",
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
        if node.kind != NodeKind::Goal && outgoing.get(node.id.as_str()).copied().unwrap_or(0) == 0
        {
            diagnostics.push(warning_with_hint(
                "non_goal_dead_end",
                Some(node.id.as_str()),
                None,
                "Non-goal node has no outgoing route.",
                "Add a return/rejoin edge or tag this as an intentional terminal reward in a later schema.",
            ));
        }
        if node.kind != NodeKind::Start && incoming.get(node.id.as_str()).copied().unwrap_or(0) == 0
        {
            diagnostics.push(warning_with_hint(
                "orphan_node",
                Some(node.id.as_str()),
                None,
                "Node has no incoming route.",
                "Add an incoming approach or branch edge from a reachable node.",
            ));
        }
    }

    validate_v2_patterns(candidate, &incoming, &outgoing, &mut diagnostics);

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

fn validate_v2_patterns(
    candidate: &Candidate,
    incoming: &BTreeMap<&str, usize>,
    outgoing: &BTreeMap<&str, usize>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for node in candidate
        .graph
        .nodes
        .iter()
        .filter(|node| node_has_tag(node, "hub"))
    {
        let incident = incoming.get(node.id.as_str()).copied().unwrap_or(0)
            + outgoing.get(node.id.as_str()).copied().unwrap_or(0);
        if incident < 3 {
            diagnostics.push(warning_with_hint(
                "hub_incident_edges_low",
                Some(node.id.as_str()),
                None,
                "Hub has fewer than three incident edges.",
                "Add at least two spokes plus a critical approach or continuation.",
            ));
        }
        if !node_has_tag(node, "wayfinding_anchor") {
            diagnostics.push(warning_with_hint(
                "hub_missing_wayfinding_anchor",
                Some(node.id.as_str()),
                None,
                "Hub is missing a wayfinding anchor tag.",
                "Tag the hub as wayfinding_anchor so later embedding can preserve orientation.",
            ));
        }
        let returns_to_hub = candidate.graph.edges.iter().any(|edge| {
            (edge.from == node.id || edge.to == node.id)
                && (edge_has_tag(edge, "return") || edge_has_tag(edge, "rejoin"))
        });
        if !returns_to_hub {
            diagnostics.push(warning_with_hint(
                "hub_missing_return_or_rejoin",
                Some(node.id.as_str()),
                None,
                "Hub has no spoke return or rejoin edge.",
                "Add a return/rejoin edge from at least one spoke back to the hub or main route.",
            ));
        }
    }

    let preparation_nodes: BTreeSet<&str> = candidate
        .graph
        .nodes
        .iter()
        .filter(|node| node_has_tag(node, "preparation"))
        .map(|node| node.id.as_str())
        .collect();
    for boss in candidate
        .graph
        .nodes
        .iter()
        .filter(|node| node_has_tag(node, "boss"))
    {
        if preparation_nodes.is_empty() {
            diagnostics.push(fatal_with_hint(
                "boss_missing_preparation",
                Some(boss.id.as_str()),
                None,
                "Boss node has no preparation branch.",
                "Add a reachable resource or clue tagged preparation before the boss approach.",
            ));
        }
        let preparation_rejoins_boss = candidate.graph.edges.iter().any(|edge| {
            edge.to == boss.id
                && preparation_nodes.contains(edge.from.as_str())
                && (edge_has_tag(edge, "return") || edge_has_tag(edge, "rejoin"))
        });
        if !preparation_nodes.is_empty() && !preparation_rejoins_boss {
            diagnostics.push(warning_with_hint(
                "boss_preparation_missing_return",
                Some(boss.id.as_str()),
                None,
                "Preparation branch does not return to the boss approach.",
                "Add a return/rejoin edge from a preparation node to the boss gate.",
            ));
        }
    }

    for hazard in candidate
        .graph
        .nodes
        .iter()
        .filter(|node| node_has_tag(node, "hazard"))
    {
        let rejoins = candidate.graph.edges.iter().any(|edge| {
            edge.from == hazard.id && (edge_has_tag(edge, "rejoin") || edge_has_tag(edge, "return"))
        });
        if !rejoins {
            diagnostics.push(warning_with_hint(
                "hazard_missing_rejoin",
                Some(hazard.id.as_str()),
                None,
                "Hazard branch does not visibly rejoin progression.",
                "Add a rejoin edge after the hazard or mark the branch as a deliberate terminal.",
            ));
        }
    }

    for merge in candidate
        .graph
        .nodes
        .iter()
        .filter(|node| node_has_tag(node, "merge"))
    {
        let incoming_count = incoming.get(merge.id.as_str()).copied().unwrap_or(0);
        if incoming_count < 2 {
            diagnostics.push(warning_with_hint(
                "merge_upstream_routes_low",
                Some(merge.id.as_str()),
                None,
                "Merge node has fewer than two upstream routes.",
                "Add another branch or shortcut edge into the merge node.",
            ));
        }
    }
}

fn node_has_tag(node: &Node, tag: &str) -> bool {
    node.tags.iter().any(|candidate_tag| candidate_tag == tag)
}

fn edge_has_tag(edge: &Edge, tag: &str) -> bool {
    edge.tags.iter().any(|candidate_tag| candidate_tag == tag)
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
                    visited.insert("start");
                    queue.clear();
                    queue.push_back("start");
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
    let hub_count = count_nodes_with_tag(candidate, "hub") as f64;
    let wayfinding_anchor_count = count_nodes_with_tag(candidate, "wayfinding_anchor") as f64;
    let preparation_count = count_nodes_with_tag(candidate, "preparation") as f64;
    let hazard_count = count_nodes_with_tag(candidate, "hazard") as f64;
    let boss_count = count_nodes_with_tag(candidate, "boss") as f64;
    let merge_count = count_nodes_with_tag(candidate, "merge") as f64;
    let pressure_edge_count = count_edges_with_tag(candidate, "pressure") as f64;
    let rejoin_edge_count = count_edges_with_tag(candidate, "rejoin") as f64;
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
    metrics.insert("hubCount".to_owned(), hub_count);
    metrics.insert("wayfindingAnchorCount".to_owned(), wayfinding_anchor_count);
    metrics.insert("preparationCount".to_owned(), preparation_count);
    metrics.insert("hazardCount".to_owned(), hazard_count);
    metrics.insert("bossCount".to_owned(), boss_count);
    metrics.insert("mergeCount".to_owned(), merge_count);
    metrics.insert("pressureEdgeCount".to_owned(), pressure_edge_count);
    metrics.insert("rejoinEdgeCount".to_owned(), rejoin_edge_count);

    let raw = 0.10
        + (critical_path.min(8.0) * 0.025)
        + (loop_bonus.min(8.0) * 0.018)
        + (optional_count.min(10.0) * 0.012)
        + (locked_count.min(4.0) * 0.025)
        + (shortcut_count.min(3.0) * 0.018)
        + (hub_count.min(1.0) * 0.035)
        + (wayfinding_anchor_count.min(3.0) * 0.018)
        + (preparation_count.min(4.0) * 0.018)
        + (pressure_edge_count.min(4.0) * 0.015)
        + (rejoin_edge_count.min(6.0) * 0.012)
        + (merge_count.min(3.0) * 0.018)
        + (boss_count.min(1.0) * 0.035)
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

fn count_nodes_with_tag(candidate: &Candidate, tag: &str) -> usize {
    candidate
        .graph
        .nodes
        .iter()
        .filter(|node| node_has_tag(node, tag))
        .count()
}

fn count_edges_with_tag(candidate: &Candidate, tag: &str) -> usize {
    candidate
        .graph
        .edges
        .iter()
        .filter(|edge| edge_has_tag(edge, tag))
        .count()
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

fn shortest_path_nodes(candidate: &Candidate, start: &str, goal: &str) -> Option<Vec<String>> {
    let mut adjacency: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for edge in &candidate.graph.edges {
        adjacency
            .entry(edge.from.as_str())
            .or_default()
            .push(edge.to.as_str());
    }
    let mut queue = VecDeque::from([start]);
    let mut visited = BTreeSet::from([start]);
    let mut previous: BTreeMap<&str, &str> = BTreeMap::new();
    while let Some(node) = queue.pop_front() {
        if node == goal {
            let mut path = vec![node.to_owned()];
            let mut cursor = node;
            while let Some(prev) = previous.get(cursor).copied() {
                path.push(prev.to_owned());
                cursor = prev;
            }
            path.reverse();
            return Some(path);
        }
        for next in adjacency.get(node).into_iter().flatten() {
            if visited.insert(next) {
                previous.insert(next, node);
                queue.push_back(next);
            }
        }
    }
    None
}

fn path_exists(candidate: &Candidate, start: &str, goal: &str) -> bool {
    shortest_path_len(candidate, start, goal).is_some()
}

fn dominator_nodes(candidate: &Candidate) -> Vec<String> {
    candidate
        .graph
        .nodes
        .iter()
        .filter(|node| node.id != "start" && node.id != "goal")
        .filter(|node| path_exists(candidate, "start", node.id.as_str()))
        .filter(|node| !path_exists_avoiding_node(candidate, "start", "goal", node.id.as_str()))
        .map(|node| node.id.clone())
        .collect()
}

fn path_exists_avoiding_node(candidate: &Candidate, start: &str, goal: &str, avoid: &str) -> bool {
    if start == avoid || goal == avoid {
        return false;
    }
    let mut adjacency: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for edge in &candidate.graph.edges {
        if edge.from == avoid || edge.to == avoid {
            continue;
        }
        adjacency
            .entry(edge.from.as_str())
            .or_default()
            .push(edge.to.as_str());
    }
    let mut queue = VecDeque::from([start]);
    let mut visited = BTreeSet::from([start]);
    while let Some(node) = queue.pop_front() {
        if node == goal {
            return true;
        }
        for next in adjacency.get(node).into_iter().flatten() {
            if visited.insert(next) {
                queue.push_back(next);
            }
        }
    }
    false
}

fn cycle_count(candidate: &Candidate) -> usize {
    let node_count = candidate.graph.nodes.len();
    let edge_count = candidate.graph.edges.len();
    if node_count == 0 {
        return 0;
    }
    let component_count = 1;
    edge_count
        .saturating_sub(node_count)
        .saturating_add(component_count)
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
        let receipt_path =
            args.out_dir
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

fn batch_generate_command(args: BatchGenerateArgs) -> Result<(), String> {
    fs::create_dir_all(&args.out_dir)
        .map_err(|error| format!("failed to create {}: {error}", args.out_dir.display()))?;
    let profile_path = args
        .profile
        .clone()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_BATCH_PROFILE));
    let profile = load_batch_profile(&profile_path)?;
    let mut accepted = Vec::new();
    let mut rejected = Vec::new();
    let mut seen_fingerprints: BTreeMap<String, String> = BTreeMap::new();
    for index in 0..args.count {
        let sequence = batch_profile_sequence(&profile, index)?;
        let candidate_seed = args.seed + index as u64 * 100;
        let run_dir = args.out_dir.join(format!("candidate-{index:03}"));
        fs::create_dir_all(&run_dir)
            .map_err(|error| format!("failed to create {}: {error}", run_dir.display()))?;
        let transcript = run_dir.join("transcript.jsonl");
        if transcript.exists() {
            fs::remove_file(&transcript)
                .map_err(|error| format!("failed to reset {}: {error}", transcript.display()))?;
        }

        let mut current = run_dir.join("candidate-000-base.json");
        init_candidate(InitArgs {
            intent: PathBuf::from("fixtures/intents/first-slice.intent.json"),
            seed: candidate_seed,
            out: current.clone(),
            receipt: run_dir.join("receipt-000-init.json"),
            transcript: Some(transcript.clone()),
        })?;

        for (rule_index, rule) in sequence.rules.iter().copied().enumerate() {
            let next = run_dir.join(format!(
                "candidate-{:03}-{}.json",
                rule_index + 1,
                rule.as_str()
            ));
            apply_rule(ApplyRuleArgs {
                state: current,
                rule,
                seed: candidate_seed + rule_index as u64 + 1,
                out: next.clone(),
                receipt: run_dir.join(format!(
                    "receipt-{:03}-{}.json",
                    rule_index + 1,
                    rule.as_str()
                )),
                transcript: Some(transcript.clone()),
            })?;
            current = next;
        }

        let candidate: Candidate = read_json(&current)?;
        let validation = validate_graph(&candidate);
        let validation_path = run_dir.join("validation.graph.json");
        write_json(&validation_path, &validation)?;
        append_transcript(
            Some(&transcript),
            "validate graph",
            Some(&validation_path),
            None,
            None,
            json!({ "state": display_path(&current) }),
        )?;

        if !validation.ok {
            rejected.push(SelectionRejection {
                candidate_id: candidate.candidate_id,
                profile_sequence: sequence.label.clone(),
                candidate_ref: display_path(&current),
                diagnostics: validation.diagnostics,
            });
            continue;
        }

        let score = score_graph(&candidate);
        let topology_fingerprint = topology_fingerprint(&candidate);
        let duplicate_of = seen_fingerprints
            .get(topology_fingerprint.as_str())
            .cloned();
        if duplicate_of.is_none() {
            seen_fingerprints.insert(topology_fingerprint.clone(), candidate.candidate_id.clone());
        }
        let budget_checks = budget_checks(profile.budgets.as_ref(), &score, &candidate);
        let budget_penalty = budget_checks.iter().filter(|check| !check.ok).count() as f64 * 0.05;
        let selection_score = (score.overall - budget_penalty).max(0.0);
        let score_path = run_dir.join("score.graph.json");
        write_json(&score_path, &score)?;
        append_transcript(
            Some(&transcript),
            "score graph",
            Some(&score_path),
            None,
            None,
            json!({ "state": display_path(&current) }),
        )?;

        let layout_path = run_dir.join("layout-2d.json");
        embed_2d_command(Embed2dArgs {
            state: current.clone(),
            seed: candidate_seed + 90,
            out: layout_path.clone(),
            receipt: run_dir.join("receipt-090-embed-2d.json"),
            transcript: Some(transcript.clone()),
        })?;
        let artifact_path = run_dir.join("accepted.json");
        accept_command(AcceptArgs {
            candidate: current.clone(),
            layout: layout_path.clone(),
            validation: validation_path.clone(),
            score: score_path.clone(),
            out: artifact_path.clone(),
            receipt: run_dir.join("receipt-091-accept.json"),
            transcript: Some(transcript),
        })?;

        accepted.push(SelectionEntry {
            candidate_id: candidate.candidate_id.clone(),
            profile_sequence: sequence.label.clone(),
            topology_fingerprint,
            duplicate_of,
            budget_checks,
            budget_penalty,
            selection_score,
            artifact_ref: display_path(&artifact_path),
            validation_ref: display_path(&validation_path),
            score_ref: display_path(&score_path),
            layout_ref: display_path(&layout_path),
            overall: score.overall,
            metrics: score.metrics,
            tags: collect_tags(&candidate),
        });
    }

    accepted.sort_by(|left, right| {
        right
            .selection_score
            .partial_cmp(&left.selection_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.candidate_id.cmp(&right.candidate_id))
    });
    let report = SelectionReport {
        kind: "asha_procgen.selection_report.v1".to_owned(),
        schema_version: 1,
        batch_id: format!("batch.v2.{}", args.seed),
        profile_id: profile.profile_id,
        profile_ref: display_path(&profile_path),
        seed: args.seed,
        requested_count: args.count,
        generated_count: accepted.len() + rejected.len(),
        accepted,
        rejected,
    };
    write_json(&args.out_dir.join("selection-report.json"), &report)?;
    println!(
        "batch run wrote {} accepted and {} rejected candidate(s) to {}",
        report.accepted.len(),
        report.rejected.len(),
        args.out_dir.display()
    );
    Ok(())
}

fn load_batch_profile(path: &Path) -> Result<BatchProfile, String> {
    let profile: BatchProfile = read_json(path)?;
    if profile.kind != "asha_procgen.batch_profile.v1" {
        return Err(format!(
            "batch profile {} has unsupported kind {}",
            path.display(),
            profile.kind
        ));
    }
    if profile.sequences.is_empty() {
        return Err(format!(
            "batch profile {} must contain at least one sequence",
            path.display()
        ));
    }
    for sequence in &profile.sequences {
        if sequence.rules.is_empty() {
            return Err(format!(
                "batch profile {} sequence {} has no rules",
                path.display(),
                sequence.label
            ));
        }
    }
    Ok(profile)
}

fn batch_profile_sequence(
    profile: &BatchProfile,
    index: usize,
) -> Result<&BatchProfileSequence, String> {
    profile
        .sequences
        .get(index % profile.sequences.len())
        .ok_or_else(|| "batch profile has no sequences".to_owned())
}

fn topology_fingerprint(candidate: &Candidate) -> String {
    let mut lines = Vec::new();
    for node in &candidate.graph.nodes {
        let mut tags = node.tags.clone();
        tags.sort();
        let incoming = candidate
            .graph
            .edges
            .iter()
            .filter(|edge| edge.to == node.id)
            .count();
        let outgoing = candidate
            .graph
            .edges
            .iter()
            .filter(|edge| edge.from == node.id)
            .count();
        lines.push(format!(
            "node:{}:{incoming}:{outgoing}:{}",
            node.kind.as_str(),
            tags.join(",")
        ));
    }
    for edge in &candidate.graph.edges {
        let mut tags = edge.tags.clone();
        tags.sort();
        lines.push(format!(
            "edge:{:?}:{:?}:required={}:{}",
            edge.kind,
            edge.traversal,
            edge.required_item.is_some(),
            tags.join(",")
        ));
    }
    lines.sort();
    format!("topology:{:016x}", fnv1a64(lines.join("\n").as_bytes()))
}

fn budget_checks(
    budgets: Option<&IntentBudget>,
    score: &ScoreReport,
    candidate: &Candidate,
) -> Vec<BudgetCheck> {
    let Some(budgets) = budgets else {
        return Vec::new();
    };
    let mut checks = Vec::new();
    if let Some(max_locked_edges) = budgets.max_locked_edges {
        let actual = metric_usize(score, "lockedEdgeCount");
        checks.push(BudgetCheck {
            code: "max_locked_edges".to_owned(),
            ok: actual <= max_locked_edges,
            detail: format!("locked edges {actual} <= budget {max_locked_edges}"),
        });
    }
    if let Some(min_optional_branches) = budgets.min_optional_branches {
        let actual = metric_usize(score, "optionalBranchCount");
        checks.push(BudgetCheck {
            code: "min_optional_branches".to_owned(),
            ok: actual >= min_optional_branches,
            detail: format!("optional branches {actual} >= budget {min_optional_branches}"),
        });
    }
    if budgets.require_hub.unwrap_or(false) {
        let actual = metric_usize(score, "hubCount");
        checks.push(BudgetCheck {
            code: "require_hub".to_owned(),
            ok: actual > 0,
            detail: format!("hub count {actual} > 0"),
        });
    }
    if budgets.require_boss.unwrap_or(false) {
        let actual = metric_usize(score, "bossCount");
        checks.push(BudgetCheck {
            code: "require_boss".to_owned(),
            ok: actual > 0,
            detail: format!("boss count {actual} > 0"),
        });
    }
    if let Some(max_dead_ends) = budgets.max_dead_ends {
        let actual = dead_end_count(candidate);
        checks.push(BudgetCheck {
            code: "max_dead_ends".to_owned(),
            ok: actual <= max_dead_ends,
            detail: format!("dead ends {actual} <= budget {max_dead_ends}"),
        });
    }
    checks
}

fn metric_usize(score: &ScoreReport, metric: &str) -> usize {
    score.metrics.get(metric).copied().unwrap_or(0.0) as usize
}

fn collect_tags(candidate: &Candidate) -> Vec<String> {
    let mut tags = BTreeSet::new();
    for node in &candidate.graph.nodes {
        for tag in &node.tags {
            tags.insert(tag.clone());
        }
    }
    for edge in &candidate.graph.edges {
        for tag in &edge.tags {
            tags.insert(tag.clone());
        }
    }
    tags.into_iter().collect()
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    serde_json::from_str(&text)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))
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
    let bytes =
        fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    Ok(format!("fnv1a64:{:016x}", fnv1a64(&bytes)))
}

fn hash_json<T: Serialize>(value: &T) -> Result<String, String> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| format!("failed to encode hash input: {error}"))?;
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

fn slugify_label(label: &str) -> String {
    let mut slug = String::new();
    let mut last_was_separator = false;
    for character in label.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            last_was_separator = false;
        } else if !last_was_separator && !slug.is_empty() {
            slug.push('_');
            last_was_separator = true;
        }
    }
    while slug.ends_with('_') {
        slug.pop();
    }
    if slug.is_empty() {
        "fork".to_owned()
    } else {
        slug
    }
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
        repair_hint: None,
    }
}

fn fatal_with_hint(
    code: &str,
    node: Option<&str>,
    edge: Option<&str>,
    detail: impl Into<String>,
    repair_hint: impl Into<String>,
) -> Diagnostic {
    Diagnostic {
        code: code.to_owned(),
        severity: Severity::Fatal,
        node: node.map(str::to_owned),
        edge: edge.map(str::to_owned),
        detail: detail.into(),
        repair_hint: Some(repair_hint.into()),
    }
}

fn warning_with_hint(
    code: &str,
    node: Option<&str>,
    edge: Option<&str>,
    detail: impl Into<String>,
    repair_hint: impl Into<String>,
) -> Diagnostic {
    Diagnostic {
        code: code.to_owned(),
        severity: Severity::Warning,
        node: node.map(str::to_owned),
        edge: edge.map(str::to_owned),
        detail: detail.into(),
        repair_hint: Some(repair_hint.into()),
    }
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

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
