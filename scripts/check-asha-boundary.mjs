import { readFileSync, readdirSync } from 'node:fs';
import { dirname, extname, join, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const packageJsonPath = join(repoRoot, 'package.json');
const packageJson = JSON.parse(readFileSync(packageJsonPath, 'utf8'));
const projectName = packageJson.name ?? 'asha-procgen';
const engineSource = packageJson.ashaDownstream?.engineSource ?? '../asha-engine';
const consumerPolicyName = packageJson.ashaDownstream?.consumerPolicy;
const engineSurfaceManifestPath = resolve(repoRoot, engineSource, 'harness/public-surface/ts-packages.json');
const dependencySections = ['dependencies', 'devDependencies', 'peerDependencies', 'optionalDependencies'];
const scannedExtensions = new Set(['.cjs', '.cts', '.js', '.json', '.jsx', '.mjs', '.mts', '.rs', '.toml', '.ts', '.tsx']);
const ignoredDirectories = new Set(['.git', 'dist', 'node_modules', 'target']);
const ignoredFiles = new Set(['package-lock.json', 'scripts/check-asha-boundary.mjs']);
const errors = [];
const { packageRoots: allowedPackageRoots, specifiers: allowedSpecifiers } = loadAllowedAshaSpecifiers();

checkPackageJson();
scanRepoFiles(repoRoot);

if (errors.length > 0) {
  console.error(`${projectName} ASHA boundary check failed:`);
  for (const error of errors) {
    console.error(`- ${error}`);
  }
  process.exit(1);
}

console.log(`${projectName} ASHA boundary check passed (${allowedPackageRoots.size} approved ASHA package roots).`);

function loadAllowedAshaSpecifiers() {
  if (typeof consumerPolicyName !== 'string' || consumerPolicyName.length === 0) {
    errors.push('package.json ashaDownstream.consumerPolicy must name an explicit upstream consumer role');
    return buildSpecifierSets([], []);
  }
  try {
    const manifest = JSON.parse(readFileSync(engineSurfaceManifestPath, 'utf8'));
    const consumerPolicy = (manifest.consumerPolicies ?? []).find((entry) => entry.consumerRole === consumerPolicyName);
    if (consumerPolicy === undefined) {
      errors.push(`${engineSurfaceManifestPath} does not define configured consumer role ${consumerPolicyName}`);
      return buildSpecifierSets([], []);
    }
    return buildSpecifierSets(consumerPolicy.approvedPackageRoots ?? [], consumerPolicy.approvedPackageSubpaths ?? []);
  } catch (error) {
    errors.push(`cannot read required ASHA package policy ${engineSurfaceManifestPath}: ${error.message}`);
    return buildSpecifierSets([], []);
  }
}

function buildSpecifierSets(packageRoots, subpaths) {
  const roots = new Set();
  const specifiers = new Set();
  for (const packageRoot of packageRoots) {
    roots.add(packageRoot);
    specifiers.add(packageRoot);
  }
  for (const subpath of subpaths) {
    if (typeof subpath === 'string') {
      specifiers.add(subpath);
    }
  }
  return { packageRoots: roots, specifiers };
}

function checkPackageJson() {
  for (const section of dependencySections) {
    const dependencies = packageJson[section] ?? {};
    for (const dependencyName of Object.keys(dependencies)) {
      if (!dependencyName.startsWith('@asha/')) {
        continue;
      }
      if (!allowedPackageRoots.has(dependencyName)) {
        errors.push(`${section}.${dependencyName} is not approved for ${consumerPolicyName} by ${engineSurfaceManifestPath}`);
      }
    }
  }
}

function scanRepoFiles(directory) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    if (entry.isDirectory()) {
      if (!ignoredDirectories.has(entry.name)) {
        scanRepoFiles(join(directory, entry.name));
      }
      continue;
    }
    if (!entry.isFile()) {
      continue;
    }
    if (!scannedExtensions.has(extname(entry.name))) {
      continue;
    }
    const filePath = join(directory, entry.name);
    const displayPath = relative(repoRoot, filePath).split(sep).join('/');
    if (ignoredFiles.has(displayPath) || displayPath.endsWith('/crates/preflight/src/main.rs')) {
      continue;
    }
    checkTextFile(filePath, readFileSync(filePath, 'utf8'));
  }
}

function checkTextFile(filePath, text) {
  const displayPath = relative(repoRoot, filePath).split(sep).join('/');
  const ashaReferences = text.match(/@asha\/[A-Za-z0-9_-]+(?:\/[A-Za-z0-9_./-]+)?/g) ?? [];
  for (const reference of ashaReferences) {
    const packageRoot = reference.split('/').slice(0, 2).join('/');
    if (!allowedPackageRoots.has(packageRoot)) {
      errors.push(`${displayPath} references ${reference}, which is not approved for ${consumerPolicyName}`);
      continue;
    }
    if (!allowedSpecifiers.has(reference)) {
      errors.push(`${displayPath} references ${reference}; import ASHA packages from approved package exports only`);
    }
  }

  const forbiddenPathPatterns = [
    /\.\.\/(?:asha-engine|asha)\/engine-rs\b/,
    /\.\.\/(?:asha-engine|asha)\/ts\/packages\/[^"'\s]+\/src\b/,
    /\bengine-rs\/crates\b/,
    /\bts\/packages\/contracts\/src\/generated\b/,
    /\bcontracts\/src\/generated\b/,
    /\bdist\/generated\b/,
    /\bsrc\/generated\b/,
  ];
  for (const pattern of forbiddenPathPatterns) {
    if (pattern.test(text)) {
      errors.push(`${displayPath} contains forbidden ASHA internal/generated path pattern ${pattern}`);
    }
  }
}
