import { execFile } from 'node:child_process';
import { mkdir, readFile, stat, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { promisify } from 'node:util';
import { spawn } from 'node:child_process';

const execFileAsync = promisify(execFile);
const host = '127.0.0.1';
const port = Number(process.env.VIEWER_SMOKE_PORT ?? 5194);
const baseUrl = `http://${host}:${port}`;
const outDir = process.env.VIEWER_SMOKE_OUT ?? join(tmpdir(), 'asha-procgen-viewer-smoke');

await mkdir(outDir, { recursive: true });

const server = spawn(process.execPath, ['scripts/serve-viewer.mjs', '--host', host, '--port', String(port)], {
  cwd: process.cwd(),
  stdio: ['ignore', 'pipe', 'pipe'],
});

let serverLog = '';
server.stdout.on('data', (chunk) => {
  serverLog += chunk.toString();
});
server.stderr.on('data', (chunk) => {
  serverLog += chunk.toString();
});

try {
  await waitForHealth();
  const batch = await fetchJson('/api/batches/v2');
  if (!Array.isArray(batch.accepted) || batch.accepted.length === 0) {
    throw new Error('sample batch has no accepted candidates');
  }
  const top = batch.accepted[0];
  if (typeof top.intermediateBreakdownRef !== 'string') {
    throw new Error('top selection is missing intermediateBreakdownRef');
  }
  if (typeof top.htmlRef !== 'string') {
    throw new Error('top selection is missing htmlRef');
  }
  const breakdown = await fetchArtifact(top.intermediateBreakdownRef);
  if (breakdown.kind !== 'asha_procgen.intermediate_breakdown.v1') {
    throw new Error(`unexpected intermediate kind: ${breakdown.kind}`);
  }
  if (!Array.isArray(breakdown.regions) || breakdown.regions.length === 0) {
    throw new Error('intermediate breakdown has no regions');
  }
  if (!Array.isArray(breakdown.connectors) || breakdown.connectors.length === 0) {
    throw new Error('intermediate breakdown has no connectors');
  }
  const css = await fetchText('/viewer/styles.css');
  if (!css.includes('color-scheme: dark') || !css.includes('#11161d')) {
    throw new Error('viewer dark theme CSS was not found');
  }
  const previewHtml = await fetchText(`/api/artifacts/by-path?path=${encodeURIComponent(top.htmlRef)}`);
  const previewRoomCount = countOccurrences(previewHtml, '<rect ');
  const previewCorridorCount = countOccurrences(previewHtml, '<polyline ');
  if (!previewHtml.includes('background: #0b0d10')) {
    throw new Error('standalone preview dark background was not found');
  }
  if (previewRoomCount < 2 || previewCorridorCount < 1) {
    throw new Error(`standalone preview SVG looks sparse: rooms=${previewRoomCount}, corridors=${previewCorridorCount}`);
  }
  for (const label of ['Key Pickup', 'Boss Threshold']) {
    if (!previewHtml.includes(label)) {
      throw new Error(`standalone preview missing content label: ${label}`);
    }
  }

  const chromium = await findChromium();
  const previewUrl = `${baseUrl}/api/artifacts/by-path?path=${encodeURIComponent(top.htmlRef)}`;
  const buildDom = await dumpDom(chromium, `${baseUrl}/#build`);
  const buildCellCount = countOccurrences(buildDom, 'class="build-cell');
  const buildMarkerCount = countOccurrences(buildDom, 'class="build-marker');
  if (buildCellCount < 20 || buildMarkerCount < 2) {
    throw new Error(`build tab rendered too little grid detail: cells=${buildCellCount}, markers=${buildMarkerCount}`);
  }
  const screenshots = [
    {
      name: 'layout-desktop.png',
      url: `${baseUrl}/#layout`,
      size: '1000,760',
    },
    {
      name: 'intermediate-desktop.png',
      url: `${baseUrl}/#intermediate`,
      size: '1000,760',
    },
    {
      name: 'intermediate-mobile.png',
      url: `${baseUrl}/#intermediate`,
      size: '390,800',
    },
    {
      name: 'build-desktop.png',
      url: `${baseUrl}/#build`,
      size: '1100,780',
    },
    {
      name: 'standalone-preview-desktop.png',
      url: previewUrl,
      size: '1100,780',
    },
    {
      name: 'standalone-preview-mobile.png',
      url: previewUrl,
      size: '390,820',
    },
  ];
  for (const screenshot of screenshots) {
    const out = join(outDir, screenshot.name);
    await execFileAsync(chromium, [
      '--headless',
      '--no-sandbox',
      '--disable-gpu',
      '--run-all-compositor-stages-before-draw',
      '--virtual-time-budget=3000',
      `--window-size=${screenshot.size}`,
      `--screenshot=${out}`,
      screenshot.url,
    ]);
    const file = await stat(out);
    if (file.size < 10_000) {
      throw new Error(`${screenshot.name} looks too small to be a useful screenshot`);
    }
  }

  const report = {
    ok: true,
    baseUrl,
    batchId: batch.batchId,
    candidateId: top.candidateId,
    regions: breakdown.regions.length,
    connectors: breakdown.connectors.length,
    standalonePreview: {
      htmlRef: top.htmlRef,
      rooms: previewRoomCount,
      corridors: previewCorridorCount,
      hasDarkBackground: true,
      requiredLabels: ['Key Pickup', 'Boss Threshold'],
    },
    buildTab: {
      cells: buildCellCount,
      markers: buildMarkerCount,
    },
    screenshots: screenshots.map((screenshot) => join(outDir, screenshot.name)),
  };
  await writeFile(join(outDir, 'viewer-smoke-report.json'), `${JSON.stringify(report, null, 2)}\n`);
  console.log(`viewer smoke passed; evidence written to ${outDir}`);
} finally {
  server.kill('SIGTERM');
}

async function waitForHealth() {
  const started = Date.now();
  while (Date.now() - started < 10_000) {
    try {
      const response = await fetch(`${baseUrl}/health`);
      if (response.ok) {
        return;
      }
    } catch {
      // Server is still starting.
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error(`viewer server did not start:\n${serverLog}`);
}

async function fetchJson(path) {
  const response = await fetch(`${baseUrl}${path}`);
  if (!response.ok) {
    throw new Error(`failed to fetch ${path}: ${response.status}`);
  }
  return await response.json();
}

async function fetchText(path) {
  const response = await fetch(`${baseUrl}${path}`);
  if (!response.ok) {
    throw new Error(`failed to fetch ${path}: ${response.status}`);
  }
  return await response.text();
}

async function fetchArtifact(path) {
  return await fetchJson(`/api/artifacts/by-path?path=${encodeURIComponent(path)}`);
}

function countOccurrences(text, pattern) {
  return text.split(pattern).length - 1;
}

async function dumpDom(chromium, url) {
  const { stdout } = await execFileAsync(chromium, [
    '--headless',
    '--no-sandbox',
    '--disable-gpu',
    '--run-all-compositor-stages-before-draw',
    '--virtual-time-budget=3000',
    '--dump-dom',
    url,
  ]);
  return stdout;
}

async function findChromium() {
  for (const command of ['chromium', 'chromium-browser', 'google-chrome']) {
    try {
      const { stdout } = await execFileAsync('sh', ['-lc', `command -v ${command}`]);
      const resolved = stdout.trim();
      if (resolved.length > 0) {
        return resolved;
      }
    } catch {
      // Try next candidate.
    }
  }
  const hint = await readFile('/etc/os-release', 'utf8').catch(() => '');
  throw new Error(`chromium executable not found; install chromium to run viewer smoke\n${hint}`);
}
