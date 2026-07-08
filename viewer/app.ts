interface AcceptedArtifact {
  readonly artifactId: string;
  readonly candidateHash: string;
  readonly layoutHash: string;
  readonly validationRef: string;
  readonly scoreRef: string;
  readonly layout: LayoutArtifact;
  readonly scoreSummary: ScoreReport;
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
  readonly requestedCount: number;
  readonly generatedCount: number;
  readonly accepted: readonly SelectionEntry[];
  readonly rejected: readonly SelectionRejection[];
}

interface SelectionEntry {
  readonly candidateId: string;
  readonly artifactRef: string;
  readonly validationRef: string;
  readonly scoreRef: string;
  readonly layoutRef: string;
  readonly overall: number;
  readonly metrics: Record<string, number>;
  readonly tags: readonly string[];
}

interface SelectionRejection {
  readonly candidateId: string;
  readonly candidateRef: string;
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

const svg = document.querySelector<SVGSVGElement>('#layout');
const summary = document.querySelector<HTMLElement>('#summary');
const batchList = document.querySelector<HTMLElement>('#batch-list');
const diagnostics = document.querySelector<HTMLElement>('#diagnostics');

if (svg === null || summary === null || batchList === null || diagnostics === null) {
  throw new Error('viewer mount elements are missing');
}

const layoutSvg = svg;
const summaryPanel = summary;
const batchPanel = batchList;
const diagnosticsPanel = diagnostics;
const batch = await fetchBatch();
const initialSelection = batch.accepted[0] ?? null;

if (initialSelection === null) {
  const artifact = await fetchArtifact('/api/artifacts/first-run');
  renderBatchList(batchPanel, batch, null, selectEntry);
  renderSummary(summaryPanel, artifact, null, batch);
  renderDiagnostics(diagnosticsPanel, batch);
  renderLayout(layoutSvg, artifact.layout);
} else {
  await selectEntry(initialSelection);
}

async function selectEntry(entry: SelectionEntry): Promise<void> {
  const artifact = await fetchArtifact(artifactUrl(entry.artifactRef));
  renderBatchList(batchPanel, batch, entry.candidateId, selectEntry);
  renderSummary(summaryPanel, artifact, entry, batch);
  renderDiagnostics(diagnosticsPanel, batch);
  renderLayout(layoutSvg, artifact.layout);
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
    metric('Rejected', String(report.rejected.length)),
    metric('Tags', topTags),
  );
}

function renderDiagnostics(target: HTMLElement, report: SelectionReport): void {
  if (report.rejected.length === 0) {
    const empty = document.createElement('p');
    empty.className = 'diagnostic-empty';
    empty.textContent = 'No rejected candidates in the current sample batch.';
    target.replaceChildren(empty);
    return;
  }

  const items = report.rejected.flatMap((rejection) =>
    rejection.diagnostics.map((diagnostic) => {
      const item = document.createElement('li');
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
    }),
  );
  const list = document.createElement('ul');
  list.className = 'diagnostic-list';
  list.append(...items);
  target.replaceChildren(list);
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

function shortCandidate(candidateId: string): string {
  return candidateId.replace('candidate.first_slice.', '').replace('candidate.first-slice.', '');
}

function createSvg(name: string): SVGElement {
  return document.createElementNS('http://www.w3.org/2000/svg', name);
}
