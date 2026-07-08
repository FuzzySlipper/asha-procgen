import { createReadStream } from 'node:fs';
import { stat } from 'node:fs/promises';
import { createServer } from 'node:http';
import { dirname, extname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const args = parseArgs(process.argv.slice(2));
const host = args.host ?? process.env.HOST ?? process.env.npm_config_host ?? '0.0.0.0';
const port = Number(args.port ?? process.env.PORT ?? process.env.npm_config_port ?? 5183);

const routes = new Map([
  ['/', join(repoRoot, 'viewer/index.html')],
  ['/viewer/index.html', join(repoRoot, 'viewer/index.html')],
  ['/viewer/styles.css', join(repoRoot, 'viewer/styles.css')],
  ['/viewer/app.js', join(repoRoot, 'dist/ts/viewer/app.js')],
  ['/api/artifacts/first-run', join(repoRoot, 'artifacts/samples/first-run/accepted.json')],
]);

const server = createServer(async (request, response) => {
  response.setHeader('X-Den-Project', 'asha-procgen');
  if (request.url === '/health') {
    sendJson(response, 200, { ok: true, project: 'asha-procgen' });
    return;
  }

  const pathname = new URL(request.url ?? '/', `http://${request.headers.host ?? 'localhost'}`).pathname;
  const filePath = routes.get(pathname);
  if (filePath === undefined) {
    response.writeHead(404);
    response.end('Not found');
    return;
  }
  await sendFile(response, filePath);
});

server.listen(port, host, () => {
  const address = server.address();
  const selectedPort = typeof address === 'object' && address !== null ? address.port : port;
  console.log(`asha-procgen viewer listening at http://${host}:${selectedPort}`);
  console.log('"project": "asha-procgen"');
});

process.on('SIGTERM', () => server.close(() => process.exit(0)));
process.on('SIGINT', () => server.close(() => process.exit(0)));

async function sendFile(response, filePath) {
  try {
    const fileStat = await stat(filePath);
    if (!fileStat.isFile()) {
      throw new Error('not a file');
    }
    response.writeHead(200, { 'Content-Type': contentType(filePath) });
    createReadStream(filePath).pipe(response);
  } catch {
    response.writeHead(404);
    response.end('Not found');
  }
}

function sendJson(response, statusCode, value) {
  response.writeHead(statusCode, { 'Content-Type': 'application/json; charset=utf-8' });
  response.end(`${JSON.stringify(value, null, 2)}\n`);
}

function contentType(filePath) {
  switch (extname(filePath)) {
    case '.css':
      return 'text/css; charset=utf-8';
    case '.html':
      return 'text/html; charset=utf-8';
    case '.js':
      return 'text/javascript; charset=utf-8';
    case '.json':
      return 'application/json; charset=utf-8';
    default:
      return 'application/octet-stream';
  }
}

function parseArgs(argv) {
  const parsed = {};
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--host') {
      parsed.host = argv[index + 1];
      index += 1;
    } else if (arg === '--port') {
      parsed.port = argv[index + 1];
      index += 1;
    }
  }
  return parsed;
}
