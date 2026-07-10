import type { VoxelCommand, VoxelCoord } from '@asha/contracts';

export interface PlacementCell {
  readonly x: number;
  readonly y: number;
}

export interface PiecePlacementForExtrusion {
  readonly kind: string;
  readonly placementId: string;
  readonly gridConnectivity: 'four_way' | 'eight_way';
  readonly occupiedCells: readonly PlacementCell[];
  readonly connectionCells: readonly PlacementCell[];
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

  const walkable = new Map<string, PlacementCell>();
  for (const cell of [...placement.occupiedCells, ...placement.connectionCells]) {
    walkable.set(cellKey(cell.x, cell.y), cell);
  }
  if (walkable.size === 0) {
    throw new Error('piece placement has no occupied or connection cells to extrude');
  }

  const boundary = new Map<string, PlacementCell>();
  for (const cell of walkable.values()) {
    for (const neighbor of cardinalNeighbors(cell)) {
      const key = cellKey(neighbor.x, neighbor.y);
      if (!walkable.has(key)) {
        boundary.set(key, neighbor);
      }
    }
  }

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
