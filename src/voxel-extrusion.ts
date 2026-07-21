import type { VoxelCommand, VoxelCoord } from '@asha/contracts';

export interface PlacementCell {
  readonly x: number;
  readonly y: number;
}

export interface PlacementOwnedCell extends PlacementCell {
  readonly instanceId: string;
}

export interface PiecePlacementPolicy {
  readonly schemaVersion: 1;
  readonly minimumClearanceCells: number;
  readonly contactPolicy: 'glued_exits_only';
  readonly wallThicknessCells: number;
  readonly doorwayWidthCells: number;
  readonly preservePieceBoundaries: boolean;
}

export interface PlacementGluedExit {
  readonly id: string;
  readonly fromInstance: string;
  readonly toInstance: string;
}

export interface PiecePlacementForExtrusion {
  readonly kind: string;
  readonly placementId: string;
  readonly gridConnectivity: 'four_way' | 'eight_way';
  readonly placementPolicy: PiecePlacementPolicy;
  readonly occupiedCells: readonly PlacementOwnedCell[];
  readonly connectionCells: readonly PlacementOwnedCell[];
  readonly gluedExits: readonly PlacementGluedExit[];
}

export interface VoxelExtrusionOptions {
  readonly grid: number;
  readonly chunkSize: number;
  readonly floorY: number;
  readonly wallMinY: number;
  readonly wallMaxY: number;
  readonly ceilingY: number;
  readonly floorMaterial: number;
  readonly wallMaterial: number;
  readonly ceilingMaterial: number;
  readonly generatorSeed: number;
  readonly generatorVersion: number;
}

export interface VoxelExtrusionPlan {
  readonly placementId: string;
  readonly coordinateMapping: 'placement_x_y_to_voxel_x_z';
  readonly commands: readonly VoxelCommand[];
  readonly solidVoxels: readonly {
    readonly coord: VoxelCoord;
    readonly material: number;
  }[];
  readonly walkableCellCount: number;
  readonly openingCellCount: number;
  readonly boundaryCellCount: number;
  readonly solidVoxelCount: number;
  readonly residentChunkCount: number;
  readonly buildBounds: {
    readonly min: VoxelCoord;
    readonly maxExclusive: VoxelCoord;
  };
}

const DEFAULT_OPTIONS: VoxelExtrusionOptions = {
  grid: 1,
  chunkSize: 2,
  floorY: 0,
  wallMinY: 1,
  wallMaxY: 3,
  ceilingY: 4,
  floorMaterial: 2,
  wallMaterial: 1,
  ceilingMaterial: 3,
  generatorSeed: 0,
  generatorVersion: 1,
};

interface MutableVoxel {
  readonly x: number;
  readonly y: number;
  readonly z: number;
  readonly material: number;
}

export function compilePlacementExtrusion(
  placement: PiecePlacementForExtrusion,
  overrides: Partial<VoxelExtrusionOptions> = {},
): VoxelExtrusionPlan {
  validatePlacement(placement);
  const options = { ...DEFAULT_OPTIONS, ...overrides };
  validateOptions(options);

  const occupiedByCell = ownedCellsByPosition(placement.occupiedCells);
  validateOwnedClearance(occupiedByCell, placement.placementPolicy);
  const opening = declaredOpeningCells(placement, occupiedByCell);
  const walkable = new Map<string, PlacementCell>();
  for (const cell of placement.occupiedCells) {
    walkable.set(cellKey(cell.x, cell.y), cell);
  }
  for (const cell of opening.values()) {
    walkable.set(cellKey(cell.x, cell.y), cell);
  }
  if (walkable.size === 0) {
    throw new Error('piece placement has no occupied or connection cells to extrude');
  }

  const boundary = buildWallShell(
    walkable,
    placement.placementPolicy.wallThicknessCells,
  );

  const solids = new Map<string, MutableVoxel>();
  for (const cell of walkable.values()) {
    setSolid(solids, cell.x, options.floorY, cell.y, options.floorMaterial);
    setSolid(solids, cell.x, options.ceilingY, cell.y, options.ceilingMaterial);
  }
  for (const cell of boundary.values()) {
    for (let y = options.wallMinY; y <= options.wallMaxY; y += 1) {
      setSolid(solids, cell.x, y, cell.y, options.wallMaterial);
    }
  }

  const sortedSolids = [...solids.values()].sort(compareVoxel);
  const chunks = requiredChunks(sortedSolids, options.chunkSize);
  const commands: VoxelCommand[] = [];
  for (const chunk of chunks) {
    commands.push({
      op: 'generateChunk',
      grid: options.grid,
      chunk,
      seed: options.generatorSeed,
      generatorVersion: options.generatorVersion,
    });
  }
  for (const chunk of chunks) {
    const min = {
      x: chunk.x * options.chunkSize,
      y: chunk.y * options.chunkSize,
      z: chunk.z * options.chunkSize,
    };
    commands.push({
      op: 'fillRegion',
      grid: options.grid,
      min,
      max: {
        x: min.x + options.chunkSize,
        y: min.y + options.chunkSize,
        z: min.z + options.chunkSize,
      },
      value: { kind: 'empty' },
    });
  }
  for (const voxel of sortedSolids) {
    commands.push({
      op: 'setVoxel',
      grid: options.grid,
      coord: { x: voxel.x, y: voxel.y, z: voxel.z },
      value: { kind: 'solid', material: voxel.material },
    });
  }

  return {
    placementId: placement.placementId,
    coordinateMapping: 'placement_x_y_to_voxel_x_z',
    commands,
    solidVoxels: sortedSolids.map((voxel) => ({
      coord: { x: voxel.x, y: voxel.y, z: voxel.z },
      material: voxel.material,
    })),
    walkableCellCount: walkable.size,
    openingCellCount: opening.size,
    boundaryCellCount: boundary.size,
    solidVoxelCount: sortedSolids.length,
    residentChunkCount: chunks.length,
    buildBounds: boundsFor(sortedSolids),
  };
}

function validatePlacement(placement: PiecePlacementForExtrusion): void {
  if (placement.kind !== 'asha_procgen.piece_placement.v1') {
    throw new Error(`unsupported placement kind: ${placement.kind}`);
  }
  if (placement.gridConnectivity !== 'four_way') {
    throw new Error(`first voxel extrusion proof requires four_way connectivity, got ${placement.gridConnectivity}`);
  }
  validatePlacementPolicy(placement.placementPolicy);
  if (!Array.isArray(placement.gluedExits)) {
    throw new Error('piece placement gluedExits must be an array');
  }
}

function validatePlacementPolicy(policy: PiecePlacementPolicy): void {
  if (policy?.schemaVersion !== 1) {
    throw new Error(`unsupported placement policy schema: ${String(policy?.schemaVersion)}`);
  }
  if (!Number.isInteger(policy.minimumClearanceCells) || policy.minimumClearanceCells < 0) {
    throw new Error('placement policy minimumClearanceCells must be a non-negative integer');
  }
  if (policy.contactPolicy !== 'glued_exits_only') {
    throw new Error(`unsupported placement contact policy: ${String(policy.contactPolicy)}`);
  }
  if (!Number.isInteger(policy.wallThicknessCells) || policy.wallThicknessCells <= 0) {
    throw new Error('placement policy wallThicknessCells must be a positive integer');
  }
  if (policy.minimumClearanceCells < policy.wallThicknessCells * 2 + 1) {
    throw new Error(
      'placement policy minimumClearanceCells must be at least twice wallThicknessCells plus one',
    );
  }
  if (
    !Number.isInteger(policy.doorwayWidthCells)
    || policy.doorwayWidthCells <= 0
    || policy.doorwayWidthCells % 2 === 0
  ) {
    throw new Error('placement policy doorwayWidthCells must be a positive odd integer');
  }
  if (policy.preservePieceBoundaries !== true) {
    throw new Error('placement policy schema 1 requires preservePieceBoundaries=true');
  }
}

function validateOptions(options: VoxelExtrusionOptions): void {
  if (!Number.isInteger(options.chunkSize) || options.chunkSize <= 0) {
    throw new Error('chunkSize must be a positive integer');
  }
  if (options.wallMinY > options.wallMaxY) {
    throw new Error('wallMinY must be less than or equal to wallMaxY');
  }
  if (options.floorY >= options.wallMinY || options.ceilingY <= options.wallMaxY) {
    throw new Error('floor, wall, and ceiling heights must form a non-overlapping enclosure');
  }
}

function ownedCellsByPosition(
  cells: readonly PlacementOwnedCell[],
): ReadonlyMap<string, PlacementOwnedCell> {
  const byCell = new Map<string, PlacementOwnedCell>();
  for (const cell of cells) {
    if (!Number.isInteger(cell.x) || !Number.isInteger(cell.y)) {
      throw new Error('piece placement occupied coordinates must be integers');
    }
    if (typeof cell.instanceId !== 'string' || cell.instanceId.length === 0) {
      throw new Error(`piece placement occupied cell ${cell.x},${cell.y} has no instance owner`);
    }
    const key = cellKey(cell.x, cell.y);
    const existing = byCell.get(key);
    if (existing !== undefined) {
      throw new Error(
        `piece placement occupied cell ${key} is shared by ${existing.instanceId} and ${cell.instanceId}`,
      );
    }
    byCell.set(key, cell);
  }
  return byCell;
}

function validateOwnedClearance(
  occupiedByCell: ReadonlyMap<string, PlacementOwnedCell>,
  policy: PiecePlacementPolicy,
): void {
  const clearance = policy.minimumClearanceCells;
  for (const cell of occupiedByCell.values()) {
    for (let dy = -clearance; dy <= clearance; dy += 1) {
      for (let dx = -clearance; dx <= clearance; dx += 1) {
        const distance = Math.abs(dx) + Math.abs(dy);
        if (distance === 0 || distance > clearance) {
          continue;
        }
        const other = occupiedByCell.get(cellKey(cell.x + dx, cell.y + dy));
        if (other !== undefined && other.instanceId !== cell.instanceId) {
          throw new Error(
            `piece boundary clearance ${clearance} violated by ${cell.instanceId} and ${other.instanceId}`,
          );
        }
      }
    }
  }
}

function declaredOpeningCells(
  placement: PiecePlacementForExtrusion,
  occupiedByCell: ReadonlyMap<string, PlacementOwnedCell>,
): ReadonlyMap<string, PlacementCell> {
  const openingsByOwner = new Map<string, PlacementGluedExit>();
  for (const glued of placement.gluedExits) {
    if (
      typeof glued.id !== 'string'
      || glued.id.length === 0
      || typeof glued.fromInstance !== 'string'
      || glued.fromInstance.length === 0
      || typeof glued.toInstance !== 'string'
      || glued.toInstance.length === 0
    ) {
      throw new Error('piece placement glued exits require non-empty ids and endpoint instances');
    }
    const owner = `connection.${slugifyLabel(glued.id)}`;
    if (openingsByOwner.has(owner)) {
      throw new Error(`piece placement has duplicate routed opening owner ${owner}`);
    }
    openingsByOwner.set(owner, glued);
  }

  const seenOwners = new Set<string>();
  const openings = new Map<string, PlacementCell>();
  const openingRadius = Math.floor(placement.placementPolicy.doorwayWidthCells / 2);
  for (const cell of placement.connectionCells) {
    if (!Number.isInteger(cell.x) || !Number.isInteger(cell.y)) {
      throw new Error('piece placement connection coordinates must be integers');
    }
    const glued = openingsByOwner.get(cell.instanceId);
    if (glued === undefined) {
      throw new Error(`connection cell ${cell.x},${cell.y} is not owned by a declared glued exit`);
    }
    seenOwners.add(cell.instanceId);
    for (let dy = -openingRadius; dy <= openingRadius; dy += 1) {
      for (let dx = -openingRadius; dx <= openingRadius; dx += 1) {
        const opened = { x: cell.x + dx, y: cell.y + dy };
        const occupied = occupiedByCell.get(cellKey(opened.x, opened.y));
        if (
          occupied !== undefined
          && occupied.instanceId !== glued.fromInstance
          && occupied.instanceId !== glued.toInstance
        ) {
          throw new Error(
            `doorway ${glued.id} would open unrelated piece ${occupied.instanceId} at ${opened.x},${opened.y}`,
          );
        }
        validateOpeningWallClearance(
          opened,
          glued,
          occupiedByCell,
          placement.placementPolicy.wallThicknessCells,
        );
        openings.set(cellKey(opened.x, opened.y), opened);
      }
    }
  }
  for (const owner of openingsByOwner.keys()) {
    if (!seenOwners.has(owner)) {
      throw new Error(`declared glued exit ${owner} has no routed connection cells`);
    }
  }
  return openings;
}

function validateOpeningWallClearance(
  opened: PlacementCell,
  glued: PlacementGluedExit,
  occupiedByCell: ReadonlyMap<string, PlacementOwnedCell>,
  wallThickness: number,
): void {
  for (let dy = -wallThickness; dy <= wallThickness; dy += 1) {
    for (let dx = -wallThickness; dx <= wallThickness; dx += 1) {
      if (Math.abs(dx) + Math.abs(dy) > wallThickness) {
        continue;
      }
      const occupied = occupiedByCell.get(cellKey(opened.x + dx, opened.y + dy));
      if (
        occupied !== undefined
        && occupied.instanceId !== glued.fromInstance
        && occupied.instanceId !== glued.toInstance
      ) {
        throw new Error(
          `doorway ${glued.id} enters wall clearance of unrelated piece ${occupied.instanceId}`,
        );
      }
    }
  }
}

function buildWallShell(
  walkable: ReadonlyMap<string, PlacementCell>,
  thickness: number,
): ReadonlyMap<string, PlacementCell> {
  const boundary = new Map<string, PlacementCell>();
  let frontier = [...walkable.values()];
  for (let layer = 0; layer < thickness; layer += 1) {
    const next = new Map<string, PlacementCell>();
    for (const cell of frontier) {
      for (const neighbor of cardinalNeighbors(cell)) {
        const key = cellKey(neighbor.x, neighbor.y);
        if (!walkable.has(key) && !boundary.has(key)) {
          boundary.set(key, neighbor);
          next.set(key, neighbor);
        }
      }
    }
    frontier = [...next.values()];
  }
  return boundary;
}

function slugifyLabel(label: string): string {
  const slug = label
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '_')
    .replace(/^_+|_+$/g, '');
  return slug.length === 0 ? 'fork' : slug;
}

function cardinalNeighbors(cell: PlacementCell): readonly PlacementCell[] {
  return [
    { x: cell.x + 1, y: cell.y },
    { x: cell.x - 1, y: cell.y },
    { x: cell.x, y: cell.y + 1 },
    { x: cell.x, y: cell.y - 1 },
  ];
}

function setSolid(
  solids: Map<string, MutableVoxel>,
  x: number,
  y: number,
  z: number,
  material: number,
): void {
  solids.set(voxelKey(x, y, z), { x, y, z, material });
}

function requiredChunks(solids: readonly MutableVoxel[], chunkSize: number): VoxelCoord[] {
  const chunks = new Map<string, VoxelCoord>();
  for (const voxel of solids) {
    const chunk = {
      x: floorDiv(voxel.x, chunkSize),
      y: floorDiv(voxel.y, chunkSize),
      z: floorDiv(voxel.z, chunkSize),
    };
    chunks.set(voxelKey(chunk.x, chunk.y, chunk.z), chunk);
  }
  return [...chunks.values()].sort(compareVoxel);
}

function boundsFor(solids: readonly MutableVoxel[]): VoxelExtrusionPlan['buildBounds'] {
  return {
    min: {
      x: Math.min(...solids.map((voxel) => voxel.x)),
      y: Math.min(...solids.map((voxel) => voxel.y)),
      z: Math.min(...solids.map((voxel) => voxel.z)),
    },
    maxExclusive: {
      x: Math.max(...solids.map((voxel) => voxel.x)) + 1,
      y: Math.max(...solids.map((voxel) => voxel.y)) + 1,
      z: Math.max(...solids.map((voxel) => voxel.z)) + 1,
    },
  };
}

function floorDiv(value: number, divisor: number): number {
  return Math.floor(value / divisor);
}

function compareVoxel(left: VoxelCoord, right: VoxelCoord): number {
  return left.x - right.x || left.y - right.y || left.z - right.z;
}

function cellKey(x: number, y: number): string {
  return `${x},${y}`;
}

function voxelKey(x: number, y: number, z: number): string {
  return `${x},${y},${z}`;
}
