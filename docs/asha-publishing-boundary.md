# ASHA Publishing Boundary

`asha-procgen` is an offline authoring/compiler consumer. It owns dungeon
generation policy and publishes selected results into ASHA-owned contract
shapes; it does not become runtime authority by producing those artifacts.

## Ownership

| Surface | Owner | Procgen responsibility |
| --- | --- | --- |
| Shape catalog and placement | `asha-procgen` | Footprints, reserved cells, exits, sockets, matching constraints, transforms, seeded selection, and placement provenance. |
| Prefab registry | ASHA ProjectBundle contracts | Stable `PrefabId`, part identity, stable part roles, source asset references, variants, and overrides. Procgen supplies an explicit mapping into these generated types. |
| Scene and ProjectBundle inventory | ASHA ProjectBundle contracts | Procgen emits a typed manifest plus a durable scene-side prefab-instance artifact and source-asset references. Rust ProjectBundle load remains the acceptance authority. |
| Voxel geometry and accepted mutation | ASHA voxel/runtime authority | Procgen may reference voxel-object assets. The separate direct-command extrusion is only a bounded native authority smoke lane. |

The adapter in `src/prefab-publishing.ts` consumes the local shape catalog,
shape-match artifact, piece placement, and an explicit mapping fixture. It uses
generated `@asha/contracts` types and validates the constructed registry with
the public `@asha/game-workspace` source validator. It does not copy the ASHA
prefab schema.

## Reproducible proof

Run:

```bash
npm run publish:asha-smoke
```

The proof reads the representative mapping at
`fixtures/prefab-mappings/first-slice.json` and regenerates
`artifacts/evidence/prefab-project-bundle-publication.json`. It publishes two
placed pieces as stable prefab definitions and instance records, contributes a
generated `FlatSceneDocument` projection, preserves the
candidate -> shape match -> placement -> published-artifact chain, and records
the prefab registry, scene, asset lock, and voxel-object source artifacts in a
generated `ProjectBundleManifest`.

The smoke also proves fail-closed handling for:

- a selected shape without a prefab mapping;
- a missing or malformed stable part role;
- a source whose asset kind is incompatible with the ASHA prefab part;
- duplicate or missing stable prefab instance identities;
- invalid or unsupported placement transforms.

The output is authoring evidence. It does not claim live prefab instantiation,
Rust ProjectBundle load acceptance, rendering, navigation, or collision.

## Consumer role and distribution

Sibling-checkout development currently installs ASHA packages with `file:`
dependencies from `../asha-engine`. That is a development convenience, not a
reproducible external distribution contract. External consumption should use
the shared ASHA package/version/distribution mechanism once that work is
available; Procgen must not invent a private package-copying or tarball lane.

The upstream public-surface manifest currently lacks an offline authoring
consumer role. `package.json` therefore records the existing `asha-demo` role
as an explicit temporary compatibility policy and names the intended
`downstream-authoring` migration. Upstream ASHA task #5828 owns adding that
reusable role for the narrow package set used here. The local boundary checker
now fails closed when its configured role or the upstream manifest is missing;
after #5828 lands, change `ashaDownstream.consumerPolicy` to
`downstream-authoring` and remove the migration marker.

## Voxel command lane

`src/voxel-extrusion.ts` and `npm run voxel:asha-smoke` remain a focused proof
that bounded `VoxelCommand` batches are accepted by native Rust authority,
repeat deterministically, reject an unknown material with the exact generated
tagged DTO, and do not mutate state on rejection. They are not the canonical
generated-level publication format. The maintained 2D-extrusion and
non-mesh-fidelity limits remain documented in Den's `known-limitations` entry.

## Crate and code disposition

No current Procgen crate should move upstream wholesale:

- `asha-procgen-preflight` remains downstream authoring/generation tooling;
- graph grammar, scoring, repair, embedding, shape matching, placement, and
  HTML/viewer generation remain Procgen policy;
- `src/voxel-extrusion.ts` remains a dungeon-specific realization adapter.

Only a concrete missing public capability proven at this publishing border
should become an upstream ASHA task. Determinism or potential reuse alone is
not a reason to promote a local algorithm into engine authority.
