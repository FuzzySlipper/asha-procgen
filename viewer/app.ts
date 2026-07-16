import { compilePlacementExtrusion, type VoxelExtrusionPlan } from '../src/voxel-extrusion.js';

interface AcceptedArtifact {
  readonly artifactId: string;
  readonly candidateHash: string;
  readonly layoutHash: string;
  readonly validationRef: string;
  readonly scoreRef: string;
  readonly candidate: CandidateArtifact;
  readonly layout: LayoutArtifact;
  readonly scoreSummary: ScoreReport;
}

interface CandidateArtifact {
  readonly candidateId: string;
  readonly provenance: readonly ProvenanceStep[];
}

interface ProvenanceStep {
  readonly step: number;
  readonly command: string;
  readonly seed: number | null;
  readonly summary: string;
}

interface LayoutArtifact {
  readonly layoutId: string;
  readonly candidateId: string;
  readonly rooms: readonly LayoutRoom[];
  readonly links: readonly LayoutLink[];
}

interface LayoutRoom {
  readonly nodeId: string;
  readonly kind: string;
  readonly label: string;
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

interface LayoutLink {
  readonly edgeId: string;
  readonly fromNode: string;
  readonly toNode: string;
  readonly kind: string;
  readonly traversal: string;
  readonly requiredItem: string | null;
}

interface ScoreReport {
  readonly overall: number;
  readonly metrics: Record<string, number>;
}

interface SelectionReport {
  readonly batchId: string;
  readonly profileId?: string;
  readonly profileRef?: string;
  readonly requestedCount: number;
  readonly generatedCount: number;
  readonly accepted: readonly SelectionEntry[];
  readonly rejected: readonly SelectionRejection[];
}

interface SelectionEntry {
  readonly candidateId: string;
  readonly profileSequence?: string;
  readonly artifactRef: string;
  readonly validationRef: string;
  readonly scoreRef: string;
  readonly layoutRef: string;
  readonly analysisRef?: string;
  readonly compatibleRulesRef?: string;
  readonly spatialIntentRef?: string;
  readonly intermediateBreakdownRef?: string;
  readonly intermediateValidationRef?: string;
  readonly geometryRef?: string;
  readonly geometryValidationRef?: string;
  readonly htmlPreviewRef?: string;
  readonly htmlRef?: string;
  readonly shapeCatalogRef?: string;
  readonly catalogInspectionRef?: string;
  readonly piecePlanRef?: string;
  readonly shapeMatchRef?: string;
  readonly piecePlacementRef?: string;
  readonly piecePlacementValidationRef?: string;
  readonly overall: number;
  readonly metrics: Record<string, number>;
  readonly tags: readonly string[];
}

interface SelectionRejection {
  readonly candidateId: string;
  readonly profileSequence?: string;
  readonly candidateRef: string;
  readonly diagnostics: readonly Diagnostic[];
}

interface ValidationReport {
  readonly ok: boolean;
  readonly fatalCount: number;
  readonly diagnostics: readonly Diagnostic[];
}

interface Diagnostic {
  readonly code: string;
  readonly severity: string;
  readonly node?: string | null;
  readonly edge?: string | null;
  readonly detail: string;
  readonly repairHint?: string;
}

interface SpatialIntentReport {
  readonly annotations: readonly SpatialIntentAnnotation[];
}

interface SpatialIntentAnnotation {
  readonly targetType: string;
  readonly targetId: string;
  readonly intents: readonly string[];
}

interface IntermediateBreakdown {
  readonly schemaVersion: number;
  readonly regions: readonly IntermediateRegion[];
  readonly connectors: readonly IntermediateConnector[];
  readonly constraints: readonly IntermediateConstraint[];
}

interface IntermediateRegion {
  readonly id: string;
  readonly nodeIds?: readonly string[];
  readonly role: string;
  readonly anchorNode?: string | null;
  readonly geometryRole?: string;
  readonly footprintClass?: string;
  readonly scaleBand?: string;
  readonly anchorQuality?: string;
  readonly entranceExpectations?: readonly string[];
}

interface IntermediateConnector {
  readonly id: string;
  readonly edgeId: string;
  readonly fromRegion: string;
  readonly toRegion: string;
  readonly intents: readonly string[];
  readonly affordances?: readonly string[];
  readonly constraintRefs?: readonly string[];
}

interface IntermediateConstraint {
  readonly code: string;
  readonly target: string;
}

interface IntermediateContext {
  readonly spatialIntent: SpatialIntentReport | null;
  readonly breakdown: IntermediateBreakdown | null;
  readonly validation: ValidationReport | null;
}

interface Geometry2dArtifact {
  readonly geometryId: string;
  readonly candidateId: string;
  readonly bounds: GeometryBounds;
  readonly rooms: readonly GeometryRoom[];
  readonly corridors: readonly GeometryCorridor[];
  readonly contents: readonly GeometryContent[];
}

interface GeometryBounds {
  readonly width: number;
  readonly height: number;
  readonly grid: number;
}

interface GeometryRoom {
  readonly id: string;
  readonly sourceRegion: string;
  readonly sourceNodes: readonly string[];
  readonly role: string;
  readonly geometryRole: string;
  readonly footprintClass: string;
  readonly rect: GeometryRect;
  readonly styleTags: readonly string[];
}

interface GeometryRect {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

interface GeometryCorridor {
  readonly id: string;
  readonly sourceConnector: string;
  readonly sourceEdge: string;
  readonly fromRoom: string;
  readonly toRoom: string;
  readonly traversalHint: string;
  readonly semanticTags: readonly string[];
  readonly width: number;
  readonly points: readonly GeometryPoint[];
}

interface GeometryPoint {
  readonly x: number;
  readonly y: number;
}

interface GeometryContent {
  readonly id: string;
  readonly roomId: string;
  readonly sourceRef: string;
  readonly kind: string;
  readonly label: string;
  readonly tags: readonly string[];
}

interface ShapeCatalog {
  readonly kind: string;
  readonly catalogId: string;
  readonly cellSize: number;
  readonly shapes: readonly CatalogShape[];
}

interface CatalogShape {
  readonly shapeId: string;
  readonly label: string;
  readonly pieceKinds: readonly string[];
  readonly footprint: readonly GridCell[];
  readonly reservedCells: readonly GridCell[];
  readonly exits: readonly CatalogExit[];
  readonly allowedTransforms: readonly string[];
  readonly featureSockets: readonly CatalogSocket[];
  readonly tags: readonly string[];
}

interface CatalogExit {
  readonly id: string;
  readonly x: number;
  readonly y: number;
  readonly direction: string;
  readonly width: number;
  readonly tags: readonly string[];
}

interface CatalogSocket {
  readonly id: string;
  readonly kind: string;
  readonly x: number;
  readonly y: number;
  readonly tags: readonly string[];
}

interface PiecePlacement {
  readonly kind: string;
  readonly placementId: string;
  readonly planId: string;
  readonly catalogId: string;
  readonly matchId: string;
  readonly sourceCatalogRef?: string;
  readonly cellSize: number;
  readonly gridConnectivity: 'four_way' | 'eight_way';
  readonly instances: readonly PieceInstance[];
  readonly gluedExits: readonly GluedExit[];
  readonly occupiedCells: readonly PlacementCellRef[];
  readonly connectionCells: readonly PlacementCellRef[];
  readonly reservedCells: readonly PlacementCellRef[];
  readonly danglingExits: readonly DanglingExit[];
}

interface PieceInstance {
  readonly instanceId: string;
  readonly pieceId: string;
  readonly requirementKind: string;
  readonly role: string;
  readonly shapeId: string;
  readonly transform: string;
  readonly origin: GridCell;
  readonly occupiedCells: readonly GridCell[];
  readonly reservedCells: readonly GridCell[];
  readonly exitMap: readonly MatchedExit[];
  readonly featurePlacements: readonly MatchedSocket[];
  readonly sourceRequirementRef: string;
  readonly sourceRefs: readonly string[];
  readonly tags: readonly string[];
}

interface GridCell {
  readonly x: number;
  readonly y: number;
}

interface PlacementCellRef {
  readonly instanceId: string;
  readonly x: number;
  readonly y: number;
}

interface GluedExit {
  readonly id: string;
  readonly linkId: string;
  readonly fromInstance: string;
  readonly fromExit: string;
  readonly toInstance: string;
  readonly toExit: string;
  readonly sourceRef: string;
  readonly tags: readonly string[];
}

interface DanglingExit {
  readonly instanceId: string;
  readonly exitId: string;
  readonly reason: string;
}

interface MatchedExit {
  readonly requirementExitId: string;
  readonly catalogExitId: string;
  readonly direction: string;
  readonly width: number;
}

interface MatchedSocket {
  readonly requiredSocket: string;
  readonly catalogSocketId: string;
  readonly kind: string;
}

interface NativeVoxelEvidence {
  readonly placementId: string;
  readonly ashaEngineCommit: string;
  readonly authority: {
    readonly voxelStateHash: string;
    readonly deterministic: boolean;
    readonly acceptedCommands: number;
    readonly rejectedCommands: number;
  };
}

const svg = document.querySelector<SVGSVGElement>('#layout');
const summary = document.querySelector<HTMLElement>('#summary');
const batchList = document.querySelector<HTMLElement>('#batch-list');
const diagnostics = document.querySelector<HTMLElement>('#diagnostics');
const viewTabs = document.querySelectorAll<HTMLButtonElement>('[data-view]');

if (svg === null || summary === null || batchList === null || diagnostics === null) {
  throw new Error('viewer mount elements are missing');
}

type ViewMode = 'layout' | 'intermediate' | 'build' | 'voxel' | 'catalog';

const layoutSvg = svg;
const summaryPanel = summary;
const batchPanel = batchList;
const diagnosticsPanel = diagnostics;
const batch = await fetchBatch();
const voxelEvidence = await fetchVoxelEvidence();
const requestedCandidate = new URLSearchParams(location.search).get('candidate');
const initialSelection = batch.accepted.find((entry) => entry.candidateId === requestedCandidate)
  ?? batch.accepted[0]
  ?? null;
let activeView: ViewMode = initialViewMode();
let currentLayout: LayoutArtifact | null = null;
let currentIntermediate: IntermediateContext = emptyIntermediateContext();
let currentGeometry: Geometry2dArtifact | null = null;
let currentCatalog: ShapeCatalog | null = null;
let currentCatalogRef: string | null = null;
let currentCatalogError: string | null = null;
let currentPlacement: PiecePlacement | null = null;
let currentPlacementValidation: ValidationReport | null = null;

for (const tab of viewTabs) {
  tab.addEventListener('click', () => {
    const nextView = tab.dataset.view;
    if (nextView === 'layout' || nextView === 'intermediate' || nextView === 'build' || nextView === 'voxel' || nextView === 'catalog') {
      activeView = nextView;
      history.replaceState(null, '', `#${activeView}`);
      renderActiveView();
    }
  });
}

if (initialSelection === null) {
  const artifact = await fetchArtifact('/api/artifacts/first-run');
  const validation = await fetchValidation(artifactUrl(artifact.validationRef));
  currentLayout = artifact.layout;
  currentIntermediate = emptyIntermediateContext();
  currentGeometry = null;
  currentCatalog = await fetchDefaultCatalog();
  currentCatalogRef = currentCatalog === null ? null : 'fixtures/shape-catalogs/2d-basic.json';
  currentCatalogError = currentCatalog === null ? 'failed to load default fixture catalog' : null;
  currentPlacement = null;
  currentPlacementValidation = null;
  renderBatchList(batchPanel, batch, null, selectEntry);
  renderSummary(summaryPanel, artifact, null, batch);
  renderContext(
    diagnosticsPanel,
    artifact,
    null,
    batch,
    validation,
    emptyIntermediateContext(),
    null,
  );
  renderActiveView();
} else {
  await selectEntry(initialSelection);
}

async function selectEntry(entry: SelectionEntry): Promise<void> {
  const artifact = await fetchArtifact(artifactUrl(entry.artifactRef));
  const validation = await fetchValidation(artifactUrl(entry.validationRef));
  const intermediate = await fetchIntermediateContext(entry);
  const [geometry, placement, placementValidation] = await Promise.all([
    fetchOptionalArtifact<Geometry2dArtifact>(entry.geometryRef),
    fetchOptionalArtifact<PiecePlacement>(entry.piecePlacementRef),
    fetchOptionalArtifact<ValidationReport>(entry.piecePlacementValidationRef),
  ]);
  const catalogResult = await fetchCatalogForEntry(entry, placement);
  currentLayout = artifact.layout;
  currentIntermediate = intermediate;
  currentGeometry = geometry;
  currentCatalog = catalogResult.catalog;
  currentCatalogRef = catalogResult.ref;
  currentCatalogError = catalogResult.error;
  currentPlacement = placement;
  currentPlacementValidation = placementValidation;
  renderBatchList(batchPanel, batch, entry.candidateId, selectEntry);
  renderSummary(summaryPanel, artifact, entry, batch);
  renderContext(diagnosticsPanel, artifact, entry, batch, validation, intermediate, placementValidation);
  renderActiveView();
}

async function fetchBatch(): Promise<SelectionReport> {
  const response = await fetch('/api/batches/v2');
  if (!response.ok) {
    return {
      batchId: 'first-run-fallback',
      requestedCount: 1,
      generatedCount: 1,
      accepted: [],
      rejected: [],
    };
  }
  return (await response.json()) as SelectionReport;
}

async function fetchVoxelEvidence(): Promise<NativeVoxelEvidence | null> {
  const response = await fetch('/api/evidence/native-voxel-extrusion');
  if (!response.ok) {
    return null;
  }
  return (await response.json()) as NativeVoxelEvidence;
}

async function fetchArtifact(url: string): Promise<AcceptedArtifact> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`failed to load artifact: ${response.status}`);
  }
  return (await response.json()) as AcceptedArtifact;
}

async function fetchValidation(url: string): Promise<ValidationReport> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`failed to load validation: ${response.status}`);
  }
  return (await response.json()) as ValidationReport;
}

async function fetchIntermediateContext(entry: SelectionEntry): Promise<IntermediateContext> {
  const [spatialIntent, breakdown, validation] = await Promise.all([
    fetchOptionalArtifact<SpatialIntentReport>(entry.spatialIntentRef),
    fetchOptionalArtifact<IntermediateBreakdown>(entry.intermediateBreakdownRef),
    fetchOptionalArtifact<ValidationReport>(entry.intermediateValidationRef),
  ]);
  return { spatialIntent, breakdown, validation };
}

async function fetchOptionalArtifact<T>(path: string | undefined): Promise<T | null> {
  if (path === undefined) {
    return null;
  }
  const response = await fetch(artifactUrl(path));
  if (!response.ok) {
    return null;
  }
  return (await response.json()) as T;
}

async function fetchCatalogForEntry(
  entry: SelectionEntry,
  placement: PiecePlacement | null,
): Promise<{
  readonly catalog: ShapeCatalog | null;
  readonly ref: string | null;
  readonly error: string | null;
}> {
  const refs = [
    entry.shapeCatalogRef,
    placement?.sourceCatalogRef,
    'fixtures/shape-catalogs/2d-basic.json',
  ].filter((value, index, values): value is string => {
    return value !== undefined && values.indexOf(value) === index;
  });
  for (const ref of refs) {
    for (const url of catalogUrls(ref)) {
      try {
        const response = await fetch(url);
        if (!response.ok) {
          continue;
        }
        return {
          catalog: (await response.json()) as ShapeCatalog,
          ref,
          error: null,
        };
      } catch {
        // Try the next URL/ref. The visible tab reports the final failure below.
      }
    }
  }
  return {
    catalog: null,
    ref: refs[0] ?? null,
    error: refs.length === 0
      ? 'no catalog ref was available'
      : `failed to load ${refs.join(', ')}`,
  };
}

function catalogUrls(ref: string): readonly string[] {
  const urls = [artifactUrl(ref)];
  if (ref.startsWith('fixtures/')) {
    urls.push(`/${ref}`);
  }
  return urls;
}

async function fetchDefaultCatalog(): Promise<ShapeCatalog | null> {
  for (const url of catalogUrls('fixtures/shape-catalogs/2d-basic.json')) {
    try {
      const response = await fetch(url);
      if (response.ok) {
        return (await response.json()) as ShapeCatalog;
      }
    } catch {
      // Best-effort fallback for first-run/no-batch mode.
    }
  }
  return null;
}

function emptyIntermediateContext(): IntermediateContext {
  return {
    spatialIntent: null,
    breakdown: null,
    validation: null,
  };
}

function initialViewMode(): ViewMode {
  if (location.hash === '#catalog') {
    return 'catalog';
  }
  if (location.hash === '#intermediate') {
    return 'intermediate';
  }
  if (location.hash === '#build') {
    return 'build';
  }
  if (location.hash === '#voxel') {
    return 'voxel';
  }
  return 'layout';
}

function artifactUrl(path: string): string {
  return `/api/artifacts/by-path?path=${encodeURIComponent(path)}`;
}

function renderBatchList(
  target: HTMLElement,
  report: SelectionReport,
  selectedCandidateId: string | null,
  onSelect: (entry: SelectionEntry) => void,
): void {
  const header = document.createElement('div');
  header.className = 'batch-header';
  header.append(
    metric('Batch', report.batchId),
    metric('Accepted', `${report.accepted.length}/${report.generatedCount}`),
  );

  const buttons = report.accepted.map((entry, index) => {
    const button = document.createElement('button');
    button.className = 'candidate-button';
    button.type = 'button';
    button.dataset.selected = entry.candidateId === selectedCandidateId ? 'true' : 'false';
    button.addEventListener('click', () => onSelect(entry));

    const rank = document.createElement('span');
    rank.className = 'candidate-rank';
    rank.textContent = String(index + 1).padStart(2, '0');
    const name = document.createElement('span');
    name.className = 'candidate-name';
    name.textContent = shortCandidate(entry.candidateId);
    const score = document.createElement('span');
    score.className = 'candidate-score';
    score.textContent = entry.overall.toFixed(2);
    const tags = document.createElement('span');
    tags.className = 'candidate-tags';
    tags.textContent = entry.tags.slice(0, 4).join(' / ');

    button.append(rank, name, score, tags);
    return button;
  });

  target.replaceChildren(header, ...buttons);
}

function renderSummary(
  target: HTMLElement,
  artifact: AcceptedArtifact,
  selection: SelectionEntry | null,
  report: SelectionReport,
): void {
  const metrics = artifact.scoreSummary.metrics;
  const topTags = selection?.tags.slice(0, 8).join(', ') ?? 'first-run';
  target.replaceChildren(
    metric('Artifact', artifact.artifactId),
    metric('Candidate', artifact.layout.candidateId),
    metric('Overall', artifact.scoreSummary.overall.toFixed(2)),
    metric('Nodes', String(metrics.nodeCount ?? artifact.layout.rooms.length)),
    metric('Edges', String(metrics.edgeCount ?? artifact.layout.links.length)),
    metric('Loops', String(metrics.loopCount ?? 0)),
    metric('Hubs', String(metrics.hubCount ?? 0)),
    metric('Pressure', String(metrics.pressureEdgeCount ?? 0)),
    metric('Profile', selection?.profileSequence ?? 'first-run'),
    metric('Rejected', String(report.rejected.length)),
    metric('Tags', topTags),
  );
}

function renderContext(
  target: HTMLElement,
  artifact: AcceptedArtifact,
  selection: SelectionEntry | null,
  report: SelectionReport,
  validation: ValidationReport,
  intermediate: IntermediateContext,
  placementValidation: ValidationReport | null,
): void {
  target.replaceChildren(
    contextSection('Artifact Refs', [
      refLine('artifact', selection?.artifactRef ?? '/api/artifacts/first-run'),
      refLine('validation', artifact.validationRef),
      refLine('score', artifact.scoreRef),
      refLine('layout', selection?.layoutRef ?? artifact.layout.layoutId),
      refLine('profile', report.profileRef ?? 'first-run'),
    ]),
    contextSection('Intermediate Refs', intermediateRefLines(selection)),
    contextSection('Build Refs', buildRefLines(selection)),
    contextSection('Piece Placement', piecePlacementLines(selection, placementValidation)),
    contextSection('Intermediate', intermediateLines(intermediate)),
    contextSection('Validation', validationLines(validation)),
    contextSection('Provenance', provenanceLines(artifact.candidate.provenance)),
    contextSection('Batch Rejections', rejectionLines(report)),
  );
}

function buildRefLines(selection: SelectionEntry | null): readonly HTMLElement[] {
  if (selection === null || selection.geometryRef === undefined) {
    const empty = document.createElement('p');
    empty.className = 'diagnostic-empty';
    empty.textContent = 'No geometry/build artifact refs are available for this selection.';
    return [empty];
  }
  return [
    refLine('geometry', selection.geometryRef),
    refLine('gvalid', selection.geometryValidationRef ?? 'missing'),
    refLine('preview', selection.htmlPreviewRef ?? 'missing'),
    refLine('html', selection.htmlRef ?? 'missing'),
    refLine('catalog', selection.shapeCatalogRef ?? 'missing'),
    refLine('creport', selection.catalogInspectionRef ?? 'missing'),
    refLine('plan', selection.piecePlanRef ?? 'missing'),
    refLine('match', selection.shapeMatchRef ?? 'missing'),
    refLine('place', selection.piecePlacementRef ?? 'missing'),
    refLine('pvalid', selection.piecePlacementValidationRef ?? 'missing'),
  ];
}

function piecePlacementLines(
  selection: SelectionEntry | null,
  validation: ValidationReport | null,
): readonly HTMLElement[] {
  if (selection === null || selection.piecePlacementRef === undefined) {
    const empty = document.createElement('p');
    empty.className = 'diagnostic-empty';
    empty.textContent = 'No catalog piece placement loaded; Build tab will use geometry fallback.';
    return [empty];
  }
  const lines = [
    contextLine('placement', selection.piecePlacementRef),
    contextLine('shape match', selection.shapeMatchRef ?? 'missing'),
  ];
  if (validation !== null) {
    lines.push(...validationLines(validation));
  }
  return lines;
}

function intermediateRefLines(selection: SelectionEntry | null): readonly HTMLElement[] {
  if (selection === null || selection.intermediateBreakdownRef === undefined) {
    const empty = document.createElement('p');
    empty.className = 'diagnostic-empty';
    empty.textContent = 'No intermediate artifact refs are available for this selection.';
    return [empty];
  }
  return [
    refLine('analysis', selection.analysisRef ?? 'missing'),
    refLine('rules', selection.compatibleRulesRef ?? 'missing'),
    refLine('intent', selection.spatialIntentRef ?? 'missing'),
    refLine('breakdown', selection.intermediateBreakdownRef),
    refLine('ivalid', selection.intermediateValidationRef ?? 'missing'),
  ];
}

function intermediateLines(intermediate: IntermediateContext): readonly HTMLElement[] {
  const lines: HTMLElement[] = [];
  if (intermediate.breakdown === null) {
    const empty = document.createElement('p');
    empty.className = 'diagnostic-empty';
    empty.textContent = 'No intermediate breakdown loaded.';
    lines.push(empty);
  } else {
    lines.push(
      contextLine(
        `schema ${intermediate.breakdown.schemaVersion}`,
        `${intermediate.breakdown.regions.length} regions / ${intermediate.breakdown.connectors.length} connectors / ${intermediate.breakdown.constraints.length} constraints`,
      ),
    );
    const roles = tally(intermediate.breakdown.regions.map((region) => region.role));
    lines.push(contextLine('region roles', roles.join(', ')));
    const affordances = tally(
      intermediate.breakdown.connectors.flatMap((connector) => connector.affordances ?? []),
    );
    lines.push(contextLine('affordances', affordances.join(', ') || 'none'));
  }
  if (intermediate.spatialIntent !== null) {
    lines.push(
      contextLine('spatial intent', `${intermediate.spatialIntent.annotations.length} annotations`),
    );
  }
  if (intermediate.validation !== null) {
    lines.push(...validationLines(intermediate.validation));
  }
  return lines;
}

function validationLines(validation: ValidationReport): readonly HTMLElement[] {
  const status = document.createElement('p');
  status.className = validation.ok ? 'status-ok' : 'status-fail';
  status.textContent = validation.ok
    ? 'ok'
    : `${validation.fatalCount} fatal diagnostic(s)`;
  const diagnostics = validation.diagnostics.map(diagnosticLine);
  if (diagnostics.length === 0) {
    const empty = document.createElement('p');
    empty.className = 'diagnostic-empty';
    empty.textContent = 'No validation diagnostics for the selected candidate.';
    return [status, empty];
  }
  return [status, ...diagnostics];
}

function provenanceLines(provenance: readonly ProvenanceStep[]): readonly HTMLElement[] {
  return provenance.slice(-8).map((step) => {
    const item = document.createElement('p');
    item.className = 'context-line';
    const seedText = step.seed === null ? '' : ` seed ${step.seed}`;
    item.textContent = `${step.step}. ${step.command}${seedText}`;
    const detail = document.createElement('small');
    detail.textContent = step.summary;
    item.append(detail);
    return item;
  });
}

function contextLine(label: string, detailText: string): HTMLElement {
  const item = document.createElement('p');
  item.className = 'context-line';
  item.textContent = label;
  const detail = document.createElement('small');
  detail.textContent = detailText;
  item.append(detail);
  return item;
}

function tally(values: readonly string[]): readonly string[] {
  const counts = new Map<string, number>();
  for (const value of values) {
    counts.set(value, (counts.get(value) ?? 0) + 1);
  }
  return [...counts.entries()]
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([value, count]) => `${value}:${count}`);
}

function rejectionLines(report: SelectionReport): readonly HTMLElement[] {
  if (report.rejected.length === 0) {
    const empty = document.createElement('p');
    empty.className = 'diagnostic-empty';
    empty.textContent = 'No rejected candidates in the current sample batch.';
    return [empty];
  }
  return report.rejected.flatMap((rejection) =>
    rejection.diagnostics.map((diagnostic) => {
      const line = diagnosticLine(diagnostic);
      line.prepend(`${shortCandidate(rejection.candidateId)} `);
      return line;
    }),
  );
}

function diagnosticLine(diagnostic: Diagnostic): HTMLElement {
  const item = document.createElement('p');
  item.className = `diagnostic-line ${diagnostic.severity}`;
  const code = document.createElement('strong');
  code.textContent = diagnostic.code;
  const detail = document.createElement('span');
  detail.textContent = ` ${diagnostic.detail}`;
  item.append(code, detail);
  if (diagnostic.repairHint !== undefined) {
    const repair = document.createElement('small');
    repair.textContent = diagnostic.repairHint;
    item.append(repair);
  }
  return item;
}

function contextSection(title: string, children: readonly HTMLElement[]): HTMLElement {
  const section = document.createElement('section');
  section.className = 'context-section';
  const heading = document.createElement('h2');
  heading.textContent = title;
  section.append(heading, ...children);
  return section;
}

function refLine(label: string, value: string): HTMLElement {
  const line = document.createElement('p');
  line.className = 'ref-line';
  const labelElement = document.createElement('strong');
  labelElement.textContent = label;
  const valueElement = document.createElement('span');
  valueElement.textContent = value;
  line.append(labelElement, valueElement);
  return line;
}

function metric(label: string, value: string): HTMLElement {
  const wrapper = document.createElement('div');
  wrapper.className = 'metric';
  const labelElement = document.createElement('span');
  labelElement.className = 'metric-label';
  labelElement.textContent = label;
  const valueElement = document.createElement('span');
  valueElement.className = 'metric-value';
  valueElement.textContent = value;
  wrapper.append(labelElement, valueElement);
  return wrapper;
}

function renderActiveView(): void {
  for (const tab of viewTabs) {
    tab.dataset.selected = tab.dataset.view === activeView ? 'true' : 'false';
  }
  layoutSvg.style.height = '';
  layoutSvg.style.minWidth = '';
  if (activeView === 'build') {
    renderBuildGrid(layoutSvg, currentGeometry, currentPlacement, currentPlacementValidation);
    return;
  }
  if (activeView === 'voxel') {
    renderVoxelBuild(layoutSvg, currentPlacement, voxelEvidence);
    return;
  }
  if (activeView === 'catalog') {
    renderShapeCatalog(layoutSvg, currentCatalog, currentCatalogRef, currentCatalogError);
    return;
  }
  if (activeView === 'intermediate') {
    renderIntermediate(layoutSvg, currentIntermediate.breakdown);
    return;
  }
  if (currentLayout === null) {
    renderEmptySvg(layoutSvg, 'No layout loaded.');
    return;
  }
  renderLayout(layoutSvg, currentLayout);
}

interface VoxelPoint {
  readonly x: number;
  readonly y: number;
  readonly z: number;
}

interface ProjectedPoint {
  readonly x: number;
  readonly y: number;
}

function renderVoxelBuild(
  target: SVGSVGElement,
  placement: PiecePlacement | null,
  evidence: NativeVoxelEvidence | null,
): void {
  target.replaceChildren();
  if (placement === null) {
    renderEmptySvg(target, 'No piece placement is available for voxel extrusion.');
    return;
  }

  let plan: VoxelExtrusionPlan;
  try {
    plan = compilePlacementExtrusion(placement);
  } catch (error) {
    renderEmptySvg(target, `Voxel extrusion unavailable: ${describeError(error)}`);
    return;
  }

  const margin = 36;
  const headerHeight = 112;
  const tileWidth = 15;
  const tileHeight = 8;
  const voxelHeight = 13;
  const bounds = projectedVoxelBounds(plan, tileWidth, tileHeight, voxelHeight);
  const width = Math.max(900, Math.ceil(bounds.maxX - bounds.minX + margin * 2));
  const height = Math.max(620, Math.ceil(bounds.maxY - bounds.minY + margin * 2 + headerHeight));
  const offsetX = margin - bounds.minX;
  const offsetY = margin + headerHeight - bounds.minY;
  target.setAttribute('viewBox', `0 0 ${width} ${height}`);
  target.style.height = `${height}px`;
  target.style.minWidth = `${width}px`;

  const title = createSvg('text');
  title.setAttribute('class', 'voxel-title');
  title.setAttribute('x', String(margin));
  title.setAttribute('y', '30');
  title.textContent = 'Native Voxel Extrusion Cutaway';
  target.append(title);

  const verified = evidence?.placementId === placement.placementId;
  const detail = createSvg('text');
  detail.setAttribute('class', `voxel-detail ${verified ? 'verified' : 'unverified'}`);
  detail.setAttribute('x', String(margin));
  detail.setAttribute('y', '53');
  detail.textContent = verified && evidence !== null
    ? `${plan.solidVoxelCount} voxels / ${evidence.authority.acceptedCommands} native commands / ${evidence.authority.voxelStateHash}`
    : `${plan.solidVoxelCount} voxel proposal / selected placement has no matching native authority receipt`;
  target.append(detail);

  const source = createSvg('text');
  source.setAttribute('class', 'voxel-source');
  source.setAttribute('x', String(margin));
  source.setAttribute('y', '75');
  source.textContent = verified && evidence !== null
    ? `ASHA ${evidence.ashaEngineCommit.slice(0, 12)} / deterministic ${evidence.authority.deterministic ? 'yes' : 'no'} / XZ floor plan with ghosted ceiling`
    : `${placement.placementId} / XZ floor plan with ghosted ceiling`;
  target.append(source);

  appendVoxelLegend(target, margin, 91);

  const solidKeys = new Set(plan.solidVoxels.map((voxel) => voxelKey3(voxel.coord)));
  const voxels = [...plan.solidVoxels].sort((left, right) => {
    const leftDepth = left.coord.x + left.coord.z;
    const rightDepth = right.coord.x + right.coord.z;
    return leftDepth - rightDepth || left.coord.y - right.coord.y || left.coord.x - right.coord.x;
  });
  for (const voxel of voxels) {
    const materialClass = voxelMaterialClass(voxel.material);
    const coord = voxel.coord;
    if (!solidKeys.has(voxelKey3({ x: coord.x, y: coord.y + 1, z: coord.z }))) {
      appendVoxelFace(target, voxelFacePoints(coord, 'top'), materialClass, 'top', offsetX, offsetY, tileWidth, tileHeight, voxelHeight);
    }
    if (!solidKeys.has(voxelKey3({ x: coord.x + 1, y: coord.y, z: coord.z }))) {
      appendVoxelFace(target, voxelFacePoints(coord, 'east'), materialClass, 'east', offsetX, offsetY, tileWidth, tileHeight, voxelHeight);
    }
    if (!solidKeys.has(voxelKey3({ x: coord.x, y: coord.y, z: coord.z + 1 }))) {
      appendVoxelFace(target, voxelFacePoints(coord, 'south'), materialClass, 'south', offsetX, offsetY, tileWidth, tileHeight, voxelHeight);
    }
  }
}

function projectedVoxelBounds(
  plan: VoxelExtrusionPlan,
  tileWidth: number,
  tileHeight: number,
  voxelHeight: number,
): { readonly minX: number; readonly minY: number; readonly maxX: number; readonly maxY: number } {
  const min = plan.buildBounds.min;
  const max = plan.buildBounds.maxExclusive;
  const corners: VoxelPoint[] = [];
  for (const x of [min.x, max.x]) {
    for (const y of [min.y, max.y]) {
      for (const z of [min.z, max.z]) {
        corners.push({ x, y, z });
      }
    }
  }
  const projected = corners.map((point) => projectVoxel(point, tileWidth, tileHeight, voxelHeight));
  return {
    minX: Math.min(...projected.map((point) => point.x)),
    minY: Math.min(...projected.map((point) => point.y)),
    maxX: Math.max(...projected.map((point) => point.x)),
    maxY: Math.max(...projected.map((point) => point.y)),
  };
}

function voxelFacePoints(coord: VoxelPoint, face: 'top' | 'east' | 'south'): readonly VoxelPoint[] {
  const { x, y, z } = coord;
  if (face === 'top') {
    return [
      { x, y: y + 1, z },
      { x: x + 1, y: y + 1, z },
      { x: x + 1, y: y + 1, z: z + 1 },
      { x, y: y + 1, z: z + 1 },
    ];
  }
  if (face === 'east') {
    return [
      { x: x + 1, y, z },
      { x: x + 1, y: y + 1, z },
      { x: x + 1, y: y + 1, z: z + 1 },
      { x: x + 1, y, z: z + 1 },
    ];
  }
  return [
    { x, y, z: z + 1 },
    { x, y: y + 1, z: z + 1 },
    { x: x + 1, y: y + 1, z: z + 1 },
    { x: x + 1, y, z: z + 1 },
  ];
}

function appendVoxelFace(
  target: SVGSVGElement,
  points: readonly VoxelPoint[],
  materialClass: string,
  face: string,
  offsetX: number,
  offsetY: number,
  tileWidth: number,
  tileHeight: number,
  voxelHeight: number,
): void {
  const polygon = createSvg('polygon');
  polygon.setAttribute('class', `voxel-face ${materialClass} ${face}`);
  polygon.setAttribute('points', points.map((point) => {
    const projected = projectVoxel(point, tileWidth, tileHeight, voxelHeight);
    return `${projected.x + offsetX},${projected.y + offsetY}`;
  }).join(' '));
  target.append(polygon);
}

function projectVoxel(
  point: VoxelPoint,
  tileWidth: number,
  tileHeight: number,
  voxelHeight: number,
): ProjectedPoint {
  return {
    x: (point.x - point.z) * tileWidth / 2,
    y: (point.x + point.z) * tileHeight / 2 - point.y * voxelHeight,
  };
}

function appendVoxelLegend(target: SVGSVGElement, x: number, y: number): void {
  const entries = [
    ['wall', 'Wall'],
    ['floor', 'Floor'],
    ['ceiling', 'Ceiling (ghosted)'],
  ] as const;
  entries.forEach(([className, label], index) => {
    const swatch = createSvg('rect');
    swatch.setAttribute('class', `voxel-legend-swatch ${className}`);
    swatch.setAttribute('x', String(x + index * 104));
    swatch.setAttribute('y', String(y));
    swatch.setAttribute('width', '12');
    swatch.setAttribute('height', '12');
    target.append(swatch);
    const text = createSvg('text');
    text.setAttribute('class', 'voxel-legend-label');
    text.setAttribute('x', String(x + 17 + index * 104));
    text.setAttribute('y', String(y + 11));
    text.textContent = label;
    target.append(text);
  });
}

function voxelMaterialClass(material: number): string {
  if (material === 1) {
    return 'wall';
  }
  if (material === 2) {
    return 'floor';
  }
  if (material === 3) {
    return 'ceiling';
  }
  return 'unknown';
}

function voxelKey3(point: VoxelPoint): string {
  return `${point.x},${point.y},${point.z}`;
}

function describeError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function renderShapeCatalog(
  target: SVGSVGElement,
  catalog: ShapeCatalog | null,
  catalogRef: string | null,
  catalogError: string | null,
): void {
  target.replaceChildren();
  if (catalog === null) {
    const detail = catalogError ?? (catalogRef === null ? 'no catalog ref was available' : `could not load ${catalogRef}`);
    renderEmptySvg(target, `No build piece catalog loaded: ${detail}`);
    return;
  }

  const margin = 24;
  const headerHeight = 68;
  const columns = 2;
  const cardWidth = 306;
  const cardHeight = 214;
  const gap = 12;
  const rows = Math.ceil(catalog.shapes.length / columns);
  const width = margin * 2 + columns * cardWidth + (columns - 1) * gap;
  const height = margin * 2 + headerHeight + rows * cardHeight + Math.max(0, rows - 1) * gap;
  target.setAttribute('viewBox', `0 0 ${width} ${height}`);
  target.style.height = `${height}px`;
  target.style.minWidth = `${width}px`;

  const title = createSvg('text');
  title.setAttribute('class', 'intermediate-role-label');
  title.setAttribute('x', String(margin));
  title.setAttribute('y', '28');
  title.textContent = 'Build Piece Catalog';
  target.append(title);

  const stats = createSvg('text');
  stats.setAttribute('class', 'intermediate-region-detail');
  stats.setAttribute('x', String(margin));
  stats.setAttribute('y', '50');
  stats.textContent = `${catalog.catalogId} / ${catalog.shapes.length} shapes / cell size ${catalog.cellSize} / ${catalogRef ?? 'catalog ref unknown'}`;
  target.append(stats);

  for (const [index, shape] of catalog.shapes.entries()) {
    const column = index % columns;
    const row = Math.floor(index / columns);
    const x = margin + column * (cardWidth + gap);
    const y = margin + headerHeight + row * (cardHeight + gap);
    renderCatalogShapeCard(target, shape, x, y, cardWidth, cardHeight);
  }
}

function renderCatalogShapeCard(
  target: SVGSVGElement,
  shape: CatalogShape,
  x: number,
  y: number,
  width: number,
  height: number,
): void {
  const group = createSvg('g');
  group.setAttribute('class', `catalog-shape-card ${slugClass(shape.pieceKinds[0] ?? 'piece')}`);
  group.setAttribute('transform', `translate(${x} ${y})`);

  const frame = createSvg('rect');
  frame.setAttribute('class', 'catalog-card-frame');
  frame.setAttribute('width', String(width));
  frame.setAttribute('height', String(height));
  frame.setAttribute('rx', '6');
  group.append(frame);

  const title = createSvg('text');
  title.setAttribute('class', 'catalog-card-title');
  title.setAttribute('x', '12');
  title.setAttribute('y', '22');
  title.textContent = truncateText(shape.label, 30);
  group.append(title);

  const subtitle = createSvg('text');
  subtitle.setAttribute('class', 'catalog-card-detail');
  subtitle.setAttribute('x', '12');
  subtitle.setAttribute('y', '42');
  subtitle.textContent = shape.shapeId.replace('shape.', '');
  group.append(subtitle);

  renderCatalogMiniShape(group, shape, 14, 60);

  const metadataX = 132;
  const lines = [
    `kinds: ${shape.pieceKinds.join(', ')}`,
    `exits: ${shape.exits.map((exit) => exit.direction[0]?.toUpperCase() ?? '?').join(' ') || 'none'}`,
    `sockets: ${shape.featureSockets.map((socket) => socket.kind).join(', ') || 'none'}`,
    `xforms: ${shape.allowedTransforms.map(shortTransform).join(' ')}`,
    `tags: ${shape.tags.slice(0, 4).join(', ')}`,
  ];
  lines.forEach((line, index) => {
    const text = createSvg('text');
    text.setAttribute('class', 'catalog-card-detail');
    text.setAttribute('x', String(metadataX));
    text.setAttribute('y', String(68 + index * 23));
    text.textContent = truncateText(line, 30);
    group.append(text);
  });

  target.append(group);
}

function renderCatalogMiniShape(
  group: SVGElement,
  shape: CatalogShape,
  x: number,
  y: number,
): void {
  const cellPixels = 16;
  const allCells = [
    ...shape.footprint,
    ...shape.reservedCells,
    ...shape.exits,
    ...shape.featureSockets,
  ];
  const minX = Math.min(...allCells.map((cell) => cell.x), 0);
  const minY = Math.min(...allCells.map((cell) => cell.y), 0);
  const maxX = Math.max(...allCells.map((cell) => cell.x), 1);
  const maxY = Math.max(...allCells.map((cell) => cell.y), 1);
  const columns = maxX - minX + 1;
  const rows = maxY - minY + 1;

  const background = createSvg('rect');
  background.setAttribute('class', 'catalog-mini-bg');
  background.setAttribute('x', String(x));
  background.setAttribute('y', String(y));
  background.setAttribute('width', String(columns * cellPixels));
  background.setAttribute('height', String(rows * cellPixels));
  group.append(background);

  for (const cell of shape.reservedCells) {
    appendCatalogCell(group, cell, minX, minY, x, y, cellPixels, 'reserved');
  }
  for (const cell of shape.footprint) {
    appendCatalogCell(group, cell, minX, minY, x, y, cellPixels, `footprint ${slugClass(shape.pieceKinds[0] ?? 'piece')}`);
  }
  for (const exit of shape.exits) {
    appendCatalogCell(group, exit, minX, minY, x, y, cellPixels, `exit ${slugClass(exit.direction)}`);
  }
  for (const socket of shape.featureSockets) {
    const center = normalizeCatalogPoint(socket, minX, minY, x, y, cellPixels);
    const marker = createSvg('circle');
    marker.setAttribute('class', `catalog-socket ${slugClass(socket.kind)}`);
    marker.setAttribute('cx', String(center.x));
    marker.setAttribute('cy', String(center.y));
    marker.setAttribute('r', '4');
    group.append(marker);

    const label = createSvg('text');
    label.setAttribute('class', 'catalog-socket-label');
    label.setAttribute('x', String(center.x));
    label.setAttribute('y', String(center.y + 3));
    label.textContent = contentSymbol(socket.kind);
    group.append(label);
  }
}

function appendCatalogCell(
  group: SVGElement,
  cell: GridCell,
  minX: number,
  minY: number,
  originX: number,
  originY: number,
  cellPixels: number,
  className: string,
): void {
  const normalized = normalizeCatalogPoint(cell, minX, minY, originX, originY, cellPixels);
  const rect = createSvg('rect');
  rect.setAttribute('class', `catalog-cell ${className}`);
  rect.setAttribute('x', String(normalized.x - cellPixels / 2));
  rect.setAttribute('y', String(normalized.y - cellPixels / 2));
  rect.setAttribute('width', String(cellPixels));
  rect.setAttribute('height', String(cellPixels));
  group.append(rect);
}

function normalizeCatalogPoint(
  cell: GridCell,
  minX: number,
  minY: number,
  originX: number,
  originY: number,
  cellPixels: number,
): { readonly x: number; readonly y: number } {
  return {
    x: originX + (cell.x - minX) * cellPixels + cellPixels / 2,
    y: originY + (cell.y - minY) * cellPixels + cellPixels / 2,
  };
}

function shortTransform(transform: string): string {
  switch (transform) {
    case 'identity':
      return 'I';
    case 'rotate90':
      return 'R90';
    case 'rotate180':
      return 'R180';
    case 'rotate270':
      return 'R270';
    case 'mirrorX':
      return 'MX';
    case 'mirrorY':
      return 'MY';
    default:
      return transform;
  }
}

function renderLayout(target: SVGSVGElement, layout: LayoutArtifact): void {
  target.replaceChildren();
  const roomById = new Map(layout.rooms.map((room) => [room.nodeId, room]));
  const maxX = Math.max(...layout.rooms.map((room) => room.x + room.width), 900);
  const maxY = Math.max(...layout.rooms.map((room) => room.y + room.height), 620);
  target.setAttribute('viewBox', `0 0 ${maxX + 120} ${maxY + 120}`);

  for (const link of layout.links) {
    const from = roomById.get(link.fromNode);
    const to = roomById.get(link.toNode);
    if (from === undefined || to === undefined) {
      continue;
    }
    const fromPoint = center(from);
    const toPoint = center(to);
    const path = createSvg('path');
    const controlX = (fromPoint.x + toPoint.x) / 2;
    path.setAttribute('class', `link ${link.traversal}`);
    path.setAttribute(
      'd',
      `M ${fromPoint.x} ${fromPoint.y} C ${controlX} ${fromPoint.y}, ${controlX} ${toPoint.y}, ${toPoint.x} ${toPoint.y}`,
    );
    target.append(path);

    const labelText = describeLink(link);
    if (labelText !== null) {
      const label = createSvg('text');
      label.setAttribute('class', 'edge-label');
      label.setAttribute('x', String((fromPoint.x + toPoint.x) / 2));
      label.setAttribute('y', String((fromPoint.y + toPoint.y) / 2 - 8));
      label.textContent = labelText;
      target.append(label);
    }
  }

  for (const room of layout.rooms) {
    const rect = createSvg('rect');
    rect.setAttribute('class', `room ${room.kind}`);
    rect.setAttribute('x', String(room.x));
    rect.setAttribute('y', String(room.y));
    rect.setAttribute('width', String(room.width));
    rect.setAttribute('height', String(room.height));
    rect.setAttribute('rx', '6');
    target.append(rect);

    const label = createSvg('text');
    label.setAttribute('class', 'room-label');
    label.setAttribute('x', String(room.x + room.width / 2));
    label.setAttribute('y', String(room.y + room.height / 2 + 4));
    label.textContent = room.label;
    target.append(label);
  }
}

function renderIntermediate(
  target: SVGSVGElement,
  breakdown: IntermediateBreakdown | null,
): void {
  target.replaceChildren();
  if (breakdown === null) {
    renderEmptySvg(target, 'No intermediate breakdown loaded.');
    return;
  }
  const regionsByRole = new Map<string, IntermediateRegion[]>();
  for (const region of breakdown.regions) {
    const regions = regionsByRole.get(region.role) ?? [];
    regions.push(region);
    regionsByRole.set(region.role, regions);
  }
  const roles = [...regionsByRole.keys()].sort();
  const columnWidth = 210;
  const rowHeight = 126;
  const cardWidth = 168;
  const cardHeight = 76;
  const positions = new Map<string, { readonly x: number; readonly y: number }>();
  roles.forEach((role, columnIndex) => {
    const regions = regionsByRole.get(role) ?? [];
    regions
      .slice()
      .sort((left, right) => left.id.localeCompare(right.id))
      .forEach((region, rowIndex) => {
        positions.set(region.id, {
          x: 70 + columnIndex * columnWidth,
          y: 96 + rowIndex * rowHeight,
        });
      });
  });
  const maxRows = Math.max(...[...regionsByRole.values()].map((regions) => regions.length), 1);
  const width = Math.max(900, 140 + roles.length * columnWidth);
  const height = Math.max(620, 160 + maxRows * rowHeight);
  target.setAttribute('viewBox', `0 0 ${width} ${height}`);

  roles.forEach((role, index) => {
    const heading = createSvg('text');
    heading.setAttribute('class', 'intermediate-role-label');
    heading.setAttribute('x', String(70 + index * columnWidth));
    heading.setAttribute('y', '48');
    heading.textContent = role.replaceAll('_', ' ');
    target.append(heading);
  });

  for (const connector of breakdown.connectors) {
    const from = positions.get(connector.fromRegion);
    const to = positions.get(connector.toRegion);
    if (from === undefined || to === undefined) {
      continue;
    }
    const fromPoint = {
      x: from.x + cardWidth,
      y: from.y + cardHeight / 2,
    };
    const toPoint = {
      x: to.x,
      y: to.y + cardHeight / 2,
    };
    const path = createSvg('path');
    const controlX = (fromPoint.x + toPoint.x) / 2;
    path.setAttribute('class', `intermediate-link ${connectorClass(connector)}`);
    path.setAttribute(
      'd',
      `M ${fromPoint.x} ${fromPoint.y} C ${controlX} ${fromPoint.y}, ${controlX} ${toPoint.y}, ${toPoint.x} ${toPoint.y}`,
    );
    target.append(path);

    const badge = createSvg('text');
    badge.setAttribute('class', 'intermediate-edge-label');
    badge.setAttribute('x', String((fromPoint.x + toPoint.x) / 2));
    badge.setAttribute('y', String((fromPoint.y + toPoint.y) / 2 - 8));
    badge.textContent = connectorBadge(connector);
    target.append(badge);
  }

  for (const [regionId, position] of positions) {
    const region = breakdown.regions.find((candidate) => candidate.id === regionId);
    if (region === undefined) {
      continue;
    }
    const group = createSvg('g');
    group.setAttribute('class', `intermediate-region ${slugClass(region.role)}`);
    const rect = createSvg('rect');
    rect.setAttribute('x', String(position.x));
    rect.setAttribute('y', String(position.y));
    rect.setAttribute('width', String(cardWidth));
    rect.setAttribute('height', String(cardHeight));
    rect.setAttribute('rx', '6');
    group.append(rect);

    const title = createSvg('text');
    title.setAttribute('class', 'intermediate-region-title');
    title.setAttribute('x', String(position.x + 12));
    title.setAttribute('y', String(position.y + 22));
    title.textContent = regionLabel(region);
    group.append(title);

    const detail = createSvg('text');
    detail.setAttribute('class', 'intermediate-region-detail');
    detail.setAttribute('x', String(position.x + 12));
    detail.setAttribute('y', String(position.y + 43));
    detail.textContent = `${region.geometryRole ?? 'role?'} / ${region.scaleBand ?? 'scale?'}`;
    group.append(detail);

    const anchor = createSvg('text');
    anchor.setAttribute('class', 'intermediate-region-detail');
    anchor.setAttribute('x', String(position.x + 12));
    anchor.setAttribute('y', String(position.y + 62));
    anchor.textContent = region.anchorNode ?? region.anchorQuality ?? 'derived';
    group.append(anchor);
    target.append(group);
  }
}

interface BuildCell {
  readonly kind: 'room' | 'corridor';
  readonly role: string;
}

interface BuildPlan {
  readonly cellSize: number;
  readonly cellPixels: number;
  readonly columns: number;
  readonly rows: number;
  readonly cells: Map<string, BuildCell>;
}

function renderBuildGrid(
  target: SVGSVGElement,
  geometry: Geometry2dArtifact | null,
  placement: PiecePlacement | null,
  placementValidation: ValidationReport | null,
): void {
  target.replaceChildren();
  if (placement !== null) {
    renderPiecePlacementGrid(target, placement, placementValidation);
    return;
  }
  if (geometry === null) {
    renderEmptySvg(target, 'No geometry or piece placement build artifact loaded.');
    return;
  }

  const plan = buildGridPlan(geometry);
  const margin = 24;
  const headerHeight = 54;
  const width = margin * 2 + plan.columns * plan.cellPixels;
  const height = margin * 2 + headerHeight + plan.rows * plan.cellPixels;
  target.setAttribute('viewBox', `0 0 ${width} ${height}`);

  const title = createSvg('text');
  title.setAttribute('class', 'intermediate-role-label');
  title.setAttribute('x', String(margin));
  title.setAttribute('y', '28');
  title.textContent = 'Geometry Build Grid';
  target.append(title);

  const stats = createSvg('text');
  stats.setAttribute('class', 'intermediate-region-detail');
  stats.setAttribute('x', String(margin));
  stats.setAttribute('y', '48');
  stats.textContent = `${plan.columns} x ${plan.rows} cells / ${geometry.rooms.length} rooms / ${geometry.corridors.length} corridors / ${geometry.contents.length} markers`;
  target.append(stats);

  const grid = createSvg('g');
  grid.setAttribute('transform', `translate(${margin} ${margin + headerHeight})`);
  target.append(grid);

  const background = createSvg('rect');
  background.setAttribute('x', '0');
  background.setAttribute('y', '0');
  background.setAttribute('width', String(plan.columns * plan.cellPixels));
  background.setAttribute('height', String(plan.rows * plan.cellPixels));
  background.setAttribute('fill', '#111820');
  grid.append(background);

  for (const [key, cell] of plan.cells) {
    const [column, row] = key.split(',').map(Number);
    const rect = createSvg('rect');
    rect.setAttribute('class', `build-cell ${cell.kind} ${slugClass(cell.role)}`);
    rect.setAttribute('x', String(column * plan.cellPixels));
    rect.setAttribute('y', String(row * plan.cellPixels));
    rect.setAttribute('width', String(plan.cellPixels));
    rect.setAttribute('height', String(plan.cellPixels));
    grid.append(rect);
  }

  for (let column = 0; column <= plan.columns; column += 1) {
    const line = createSvg('line');
    line.setAttribute('class', 'build-grid-line');
    line.setAttribute('x1', String(column * plan.cellPixels));
    line.setAttribute('y1', '0');
    line.setAttribute('x2', String(column * plan.cellPixels));
    line.setAttribute('y2', String(plan.rows * plan.cellPixels));
    grid.append(line);
  }
  for (let row = 0; row <= plan.rows; row += 1) {
    const line = createSvg('line');
    line.setAttribute('class', 'build-grid-line');
    line.setAttribute('x1', '0');
    line.setAttribute('y1', String(row * plan.cellPixels));
    line.setAttribute('x2', String(plan.columns * plan.cellPixels));
    line.setAttribute('y2', String(row * plan.cellPixels));
    grid.append(line);
  }

  for (const room of geometry.rooms) {
    const centerPoint = rectCenter(room.rect);
    const centerCell = pointToCell(centerPoint, plan.cellSize);
    const label = createSvg('text');
    label.setAttribute('class', 'build-label');
    label.setAttribute('x', String(centerCell.column * plan.cellPixels + 3));
    label.setAttribute('y', String(centerCell.row * plan.cellPixels + 12));
    label.textContent = buildRoomLabel(room);
    grid.append(label);
  }

  for (const [index, content] of geometry.contents.entries()) {
    const room = geometry.rooms.find((candidate) => candidate.id === content.roomId);
    if (room === undefined) {
      continue;
    }
    const centerPoint = rectCenter(room.rect);
    const centerCell = pointToCell(centerPoint, plan.cellSize);
    const markerX = centerCell.column * plan.cellPixels + 8 + (index % 3) * 12;
    const markerY = centerCell.row * plan.cellPixels + 25 + (index % 2) * 12;
    const marker = createSvg('circle');
    marker.setAttribute('class', `build-marker ${slugClass(content.kind)}`);
    marker.setAttribute('cx', String(markerX));
    marker.setAttribute('cy', String(markerY));
    marker.setAttribute('r', '7');
    grid.append(marker);

    const label = createSvg('text');
    label.setAttribute('class', 'build-marker-label');
    label.setAttribute('x', String(markerX));
    label.setAttribute('y', String(markerY + 3));
    label.textContent = contentSymbol(content.kind);
    grid.append(label);
  }
}

function renderPiecePlacementGrid(
  target: SVGSVGElement,
  placement: PiecePlacement,
  validation: ValidationReport | null,
): void {
  const plan = piecePlacementGridPlan(placement);
  const margin = 24;
  const headerHeight = 64;
  const width = margin * 2 + plan.columns * plan.cellPixels;
  const height = margin * 2 + headerHeight + plan.rows * plan.cellPixels;
  target.setAttribute('viewBox', `0 0 ${width} ${height}`);

  const title = createSvg('text');
  title.setAttribute('class', 'intermediate-role-label');
  title.setAttribute('x', String(margin));
  title.setAttribute('y', '28');
  title.textContent = 'Piece Placement Grid';
  target.append(title);

  const stats = createSvg('text');
  stats.setAttribute('class', 'intermediate-region-detail');
  stats.setAttribute('x', String(margin));
  stats.setAttribute('y', '50');
  const connectivity = placement.gridConnectivity.replace('_', '-');
  stats.textContent = `${placement.instances.length} pieces / ${placement.occupiedCells.length} occupied / ${placement.connectionCells.length} connection / ${connectivity} / ${validation?.ok === false ? `${validation.fatalCount} fatal` : 'valid'}`;
  target.append(stats);

  const grid = createSvg('g');
  grid.setAttribute('transform', `translate(${margin} ${margin + headerHeight})`);
  target.append(grid);

  const background = createSvg('rect');
  background.setAttribute('x', '0');
  background.setAttribute('y', '0');
  background.setAttribute('width', String(plan.columns * plan.cellPixels));
  background.setAttribute('height', String(plan.rows * plan.cellPixels));
  background.setAttribute('fill', '#111820');
  grid.append(background);

  const centers = new Map<string, { readonly x: number; readonly y: number }>();
  for (const instance of placement.instances) {
    const cells = instance.occupiedCells.map((cell) => normalizeCell(cell, plan));
    if (cells.length === 0) {
      continue;
    }
    const minColumn = Math.min(...cells.map((cell) => cell.column));
    const maxColumn = Math.max(...cells.map((cell) => cell.column));
    const minRow = Math.min(...cells.map((cell) => cell.row));
    const maxRow = Math.max(...cells.map((cell) => cell.row));
    centers.set(instance.instanceId, {
      x: ((minColumn + maxColumn + 1) / 2) * plan.cellPixels,
      y: ((minRow + maxRow + 1) / 2) * plan.cellPixels,
    });
  }

  for (const glued of placement.gluedExits) {
    const from = centers.get(glued.fromInstance);
    const to = centers.get(glued.toInstance);
    if (from === undefined || to === undefined) {
      continue;
    }
    const line = createSvg('line');
    line.setAttribute('class', `build-glue-link ${glueClass(glued)}`);
    line.setAttribute('x1', String(from.x));
    line.setAttribute('y1', String(from.y));
    line.setAttribute('x2', String(to.x));
    line.setAttribute('y2', String(to.y));
    grid.append(line);
  }

  for (const cell of placement.reservedCells) {
    const normalized = normalizeCell(cell, plan);
    const rect = createSvg('rect');
    rect.setAttribute('class', 'build-cell reserved');
    rect.setAttribute('x', String(normalized.column * plan.cellPixels));
    rect.setAttribute('y', String(normalized.row * plan.cellPixels));
    rect.setAttribute('width', String(plan.cellPixels));
    rect.setAttribute('height', String(plan.cellPixels));
    grid.append(rect);
  }

  for (const cell of placement.connectionCells) {
    const normalized = normalizeCell(cell, plan);
    const rect = createSvg('rect');
    rect.setAttribute('class', 'build-cell connection');
    rect.setAttribute('x', String(normalized.column * plan.cellPixels));
    rect.setAttribute('y', String(normalized.row * plan.cellPixels));
    rect.setAttribute('width', String(plan.cellPixels));
    rect.setAttribute('height', String(plan.cellPixels));
    const titleElement = createSvg('title');
    titleElement.textContent = cell.instanceId;
    rect.append(titleElement);
    grid.append(rect);
  }

  const instancesById = new Map(placement.instances.map((instance) => [instance.instanceId, instance]));
  for (const cell of placement.occupiedCells) {
    const normalized = normalizeCell(cell, plan);
    const instance = instancesById.get(cell.instanceId);
    const rect = createSvg('rect');
    rect.setAttribute(
      'class',
      `build-cell piece ${slugClass(instance?.requirementKind ?? 'piece')} ${slugClass(instance?.role ?? 'piece')}`,
    );
    rect.setAttribute('x', String(normalized.column * plan.cellPixels));
    rect.setAttribute('y', String(normalized.row * plan.cellPixels));
    rect.setAttribute('width', String(plan.cellPixels));
    rect.setAttribute('height', String(plan.cellPixels));
    const titleElement = createSvg('title');
    titleElement.textContent = instance === undefined
      ? cell.instanceId
      : `${instance.pieceId} / ${instance.shapeId} / ${instance.transform}`;
    rect.append(titleElement);
    grid.append(rect);
  }

  for (let column = 0; column <= plan.columns; column += 1) {
    const line = createSvg('line');
    line.setAttribute('class', 'build-grid-line');
    line.setAttribute('x1', String(column * plan.cellPixels));
    line.setAttribute('y1', '0');
    line.setAttribute('x2', String(column * plan.cellPixels));
    line.setAttribute('y2', String(plan.rows * plan.cellPixels));
    grid.append(line);
  }
  for (let row = 0; row <= plan.rows; row += 1) {
    const line = createSvg('line');
    line.setAttribute('class', 'build-grid-line');
    line.setAttribute('x1', '0');
    line.setAttribute('y1', String(row * plan.cellPixels));
    line.setAttribute('x2', String(plan.columns * plan.cellPixels));
    line.setAttribute('y2', String(row * plan.cellPixels));
    grid.append(line);
  }

  for (const instance of placement.instances) {
    const center = centers.get(instance.instanceId);
    if (center === undefined) {
      continue;
    }
    const label = createSvg('text');
    label.setAttribute('class', 'build-label piece-label');
    label.setAttribute('x', String(center.x - 8));
    label.setAttribute('y', String(center.y + 4));
    label.textContent = pieceLabel(instance);
    grid.append(label);

    instance.featurePlacements.forEach((feature, index) => {
      const marker = createSvg('circle');
      marker.setAttribute('class', `build-marker ${slugClass(feature.kind)}`);
      marker.setAttribute('cx', String(center.x + 10 + (index % 2) * 10));
      marker.setAttribute('cy', String(center.y - 10 + Math.floor(index / 2) * 10));
      marker.setAttribute('r', '6');
      grid.append(marker);

      const markerLabel = createSvg('text');
      markerLabel.setAttribute('class', 'build-marker-label');
      markerLabel.setAttribute('x', marker.getAttribute('cx') ?? String(center.x));
      markerLabel.setAttribute('y', String(Number(marker.getAttribute('cy') ?? center.y) + 3));
      markerLabel.textContent = contentSymbol(feature.kind);
      grid.append(markerLabel);
    });
  }

  for (const dangling of placement.danglingExits) {
    const center = centers.get(dangling.instanceId);
    if (center === undefined) {
      continue;
    }
    const marker = createSvg('rect');
    marker.setAttribute('class', 'build-dangling');
    marker.setAttribute('x', String(center.x - 6));
    marker.setAttribute('y', String(center.y - 6));
    marker.setAttribute('width', '12');
    marker.setAttribute('height', '12');
    marker.setAttribute('transform', `rotate(45 ${center.x} ${center.y})`);
    grid.append(marker);
  }
}

interface PiecePlacementGridPlan {
  readonly cellPixels: number;
  readonly minX: number;
  readonly minY: number;
  readonly columns: number;
  readonly rows: number;
}

function piecePlacementGridPlan(placement: PiecePlacement): PiecePlacementGridPlan {
  const allCells = [
    ...placement.occupiedCells,
    ...placement.connectionCells,
    ...placement.reservedCells,
  ];
  const minX = Math.min(...allCells.map((cell) => cell.x), 0);
  const minY = Math.min(...allCells.map((cell) => cell.y), 0);
  const maxX = Math.max(...allCells.map((cell) => cell.x), 1);
  const maxY = Math.max(...allCells.map((cell) => cell.y), 1);
  return {
    cellPixels: 14,
    minX,
    minY,
    columns: maxX - minX + 3,
    rows: maxY - minY + 3,
  };
}

function normalizeCell(
  cell: GridCell | PlacementCellRef,
  plan: PiecePlacementGridPlan,
): { readonly column: number; readonly row: number } {
  return {
    column: cell.x - plan.minX + 1,
    row: cell.y - plan.minY + 1,
  };
}

function pieceLabel(instance: PieceInstance): string {
  switch (instance.requirementKind) {
    case 'connector':
      return 'CON';
    case 'corridor':
      return 'COR';
    case 'threshold':
      return 'GATE';
    case 'hazard':
      return 'HAZ';
    case 'reward':
      return 'REW';
    case 'resource':
      return 'RES';
    case 'secret':
      return 'SEC';
    case 'shortcut':
      return 'SCT';
    default:
      return instance.requirementKind.slice(0, 4).toUpperCase();
  }
}

function glueClass(glued: GluedExit): string {
  if (glued.tags.some((tag) => tag.includes('hidden'))) {
    return 'hidden';
  }
  if (glued.tags.some((tag) => tag.includes('locked'))) {
    return 'locked';
  }
  if (glued.tags.some((tag) => tag.includes('shortcut'))) {
    return 'shortcut';
  }
  if (glued.tags.some((tag) => tag.includes('pressure'))) {
    return 'pressure';
  }
  return 'standard';
}

function buildGridPlan(geometry: Geometry2dArtifact): BuildPlan {
  const cellSize = 24;
  const cellPixels = 16;
  const columns = Math.ceil(geometry.bounds.width / cellSize) + 1;
  const rows = Math.ceil(geometry.bounds.height / cellSize) + 1;
  const cells = new Map<string, BuildCell>();

  for (const room of geometry.rooms) {
    const startColumn = Math.floor(room.rect.x / cellSize);
    const endColumn = Math.ceil((room.rect.x + room.rect.width) / cellSize);
    const startRow = Math.floor(room.rect.y / cellSize);
    const endRow = Math.ceil((room.rect.y + room.rect.height) / cellSize);
    for (let row = startRow; row < endRow; row += 1) {
      for (let column = startColumn; column < endColumn; column += 1) {
        setBuildCell(cells, column, row, { kind: 'room', role: room.role });
      }
    }
  }

  for (const corridor of geometry.corridors) {
    for (let index = 0; index < corridor.points.length - 1; index += 1) {
      digCorridorSegment(cells, corridor.points[index], corridor.points[index + 1], cellSize);
    }
  }

  return { cellSize, cellPixels, columns, rows, cells };
}

function digCorridorSegment(
  cells: Map<string, BuildCell>,
  start: GeometryPoint,
  end: GeometryPoint,
  cellSize: number,
): void {
  const dx = end.x - start.x;
  const dy = end.y - start.y;
  const steps = Math.max(1, Math.ceil(Math.max(Math.abs(dx), Math.abs(dy)) / cellSize));
  for (let step = 0; step <= steps; step += 1) {
    const ratio = step / steps;
    const point = {
      x: start.x + dx * ratio,
      y: start.y + dy * ratio,
    };
    const cell = pointToCell(point, cellSize);
    const key = cellKey(cell.column, cell.row);
    if (!cells.has(key)) {
      cells.set(key, { kind: 'corridor', role: 'corridor' });
    }
  }
}

function setBuildCell(
  cells: Map<string, BuildCell>,
  column: number,
  row: number,
  cell: BuildCell,
): void {
  cells.set(cellKey(column, row), cell);
}

function pointToCell(
  point: GeometryPoint,
  cellSize: number,
): { readonly column: number; readonly row: number } {
  return {
    column: Math.floor(point.x / cellSize),
    row: Math.floor(point.y / cellSize),
  };
}

function cellKey(column: number, row: number): string {
  return `${column},${row}`;
}

function rectCenter(rect: GeometryRect): GeometryPoint {
  return {
    x: rect.x + rect.width / 2,
    y: rect.y + rect.height / 2,
  };
}

function buildRoomLabel(room: GeometryRoom): string {
  const source = room.sourceNodes[0] ?? room.id;
  if (room.role === 'start' || room.role === 'goal') {
    return room.role.toUpperCase();
  }
  return source.replace('gate.', 'G.').replace('hazard.', 'H.').replace('treasure.', 'T.');
}

function contentSymbol(kind: string): string {
  switch (kind) {
    case 'key_pickup':
      return 'K';
    case 'locked_gate':
    case 'gate_line':
      return 'L';
    case 'boss_threshold':
    case 'boss_space':
      return 'B';
    case 'hazard':
    case 'hazard_zone':
      return '!';
    case 'reward_cache':
      return '$';
    case 'secret_route_marker':
    case 'secret_marker':
      return '?';
    case 'shortcut_marker':
      return 'S';
    case 'resource_clue':
      return 'R';
    case 'start_marker':
      return 'A';
    case 'goal_marker':
      return 'Z';
    default:
      return '*';
  }
}

function renderEmptySvg(target: SVGSVGElement, message: string): void {
  target.replaceChildren();
  target.setAttribute('viewBox', '0 0 900 620');
  const text = createSvg('text');
  text.setAttribute('class', 'empty-svg-label');
  text.setAttribute('x', '450');
  text.setAttribute('y', '310');
  text.textContent = message;
  target.append(text);
}

function center(room: LayoutRoom): { readonly x: number; readonly y: number } {
  return {
    x: room.x + room.width / 2,
    y: room.y + room.height / 2,
  };
}

function describeLink(link: LayoutLink): string | null {
  if (link.requiredItem !== null) {
    return `requires ${link.requiredItem.replace('item.', '')}`;
  }
  if (link.traversal === 'hidden') {
    return 'hidden';
  }
  if (link.traversal === 'one_way_return') {
    return 'one-way';
  }
  return null;
}

function connectorClass(connector: IntermediateConnector): string {
  const values = [...connector.intents, ...(connector.affordances ?? [])];
  if (values.some((value) => value.includes('hidden'))) {
    return 'hidden';
  }
  if (values.some((value) => value.includes('locked') || value.includes('gated'))) {
    return 'locked';
  }
  if (values.some((value) => value.includes('shortcut'))) {
    return 'shortcut';
  }
  if (values.some((value) => value.includes('pressure'))) {
    return 'pressure';
  }
  if (values.some((value) => value.includes('rejoin') || value.includes('return'))) {
    return 'rejoin';
  }
  return 'standard';
}

function connectorBadge(connector: IntermediateConnector): string {
  const affordances = connector.affordances ?? [];
  const labels = affordances.length > 0 ? affordances : connector.intents;
  const base = labels.slice(0, 2).map((value) => value.replaceAll('_', ' ')).join(' / ');
  const constraintCount = connector.constraintRefs?.length ?? 0;
  if (constraintCount > 0) {
    return `${base} (${constraintCount})`;
  }
  return base || connector.edgeId;
}

function regionLabel(region: IntermediateRegion): string {
  const node = region.nodeIds?.[0] ?? region.id.replace('region.', '');
  return node.replaceAll('_', '.');
}

function shortCandidate(candidateId: string): string {
  return candidateId.replace('candidate.first_slice.', '').replace('candidate.first-slice.', '');
}

function truncateText(value: string, maxLength: number): string {
  if (value.length <= maxLength) {
    return value;
  }
  return `${value.slice(0, Math.max(0, maxLength - 3))}...`;
}

function slugClass(value: string): string {
  return value.replaceAll('_', '-').replaceAll('.', '-');
}

function createSvg(name: string): SVGElement {
  return document.createElementNS('http://www.w3.org/2000/svg', name);
}
