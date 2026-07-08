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

const svg = document.querySelector<SVGSVGElement>('#layout');
const summary = document.querySelector<HTMLElement>('#summary');
const batchList = document.querySelector<HTMLElement>('#batch-list');
const diagnostics = document.querySelector<HTMLElement>('#diagnostics');
const viewTabs = document.querySelectorAll<HTMLButtonElement>('[data-view]');

if (svg === null || summary === null || batchList === null || diagnostics === null) {
  throw new Error('viewer mount elements are missing');
}

type ViewMode = 'layout' | 'intermediate';

const layoutSvg = svg;
const summaryPanel = summary;
const batchPanel = batchList;
const diagnosticsPanel = diagnostics;
const batch = await fetchBatch();
const initialSelection = batch.accepted[0] ?? null;
let activeView: ViewMode = location.hash === '#intermediate' ? 'intermediate' : 'layout';
let currentLayout: LayoutArtifact | null = null;
let currentIntermediate: IntermediateContext = emptyIntermediateContext();

for (const tab of viewTabs) {
  tab.addEventListener('click', () => {
    const nextView = tab.dataset.view;
    if (nextView === 'layout' || nextView === 'intermediate') {
      activeView = nextView;
      history.replaceState(null, '', activeView === 'intermediate' ? '#intermediate' : '#layout');
      renderActiveView();
    }
  });
}

if (initialSelection === null) {
  const artifact = await fetchArtifact('/api/artifacts/first-run');
  const validation = await fetchValidation(artifactUrl(artifact.validationRef));
  currentLayout = artifact.layout;
  currentIntermediate = emptyIntermediateContext();
  renderBatchList(batchPanel, batch, null, selectEntry);
  renderSummary(summaryPanel, artifact, null, batch);
  renderContext(diagnosticsPanel, artifact, null, batch, validation, emptyIntermediateContext());
  renderActiveView();
} else {
  await selectEntry(initialSelection);
}

async function selectEntry(entry: SelectionEntry): Promise<void> {
  const artifact = await fetchArtifact(artifactUrl(entry.artifactRef));
  const validation = await fetchValidation(artifactUrl(entry.validationRef));
  const intermediate = await fetchIntermediateContext(entry);
  currentLayout = artifact.layout;
  currentIntermediate = intermediate;
  renderBatchList(batchPanel, batch, entry.candidateId, selectEntry);
  renderSummary(summaryPanel, artifact, entry, batch);
  renderContext(diagnosticsPanel, artifact, entry, batch, validation, intermediate);
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

function emptyIntermediateContext(): IntermediateContext {
  return {
    spatialIntent: null,
    breakdown: null,
    validation: null,
  };
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
    contextSection('Intermediate', intermediateLines(intermediate)),
    contextSection('Validation', validationLines(validation)),
    contextSection('Provenance', provenanceLines(artifact.candidate.provenance)),
    contextSection('Batch Rejections', rejectionLines(report)),
  );
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

function slugClass(value: string): string {
  return value.replaceAll('_', '-').replaceAll('.', '-');
}

function createSvg(name: string): SVGElement {
  return document.createElementNS('http://www.w3.org/2000/svg', name);
}
