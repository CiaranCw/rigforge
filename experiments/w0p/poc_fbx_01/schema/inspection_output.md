# POC-FBX-01 inspection JSON (research-only)

Not a Canonical contract. Not a production schema.

Harness: `harness/inspect_fbx.c` (C99 around the ufbx C API).

C is the test harness language only. This is not evidence selecting the
RigForge Core language.

## Layers (mandatory distinction)

| JSON region | Layer |
| --- | --- |
| `nodes[].authored_source_props` | RAW SOURCE STRUCTURE |
| `nodes[].ufbx_evaluated_default` | UFBX EVALUATED RESULT (load / default pose) |
| `evaluated_motion_samples` | UFBX EVALUATED RESULT (`ufbx_evaluate_scene` / `ufbx_evaluate_transform`) |
| `derived_test_view` | RIGFORGE DERIVED TEST INTERPRETATION — this harness leaves it `NOT_CREATED` |
| `*.role_label` | structural sample subset only, not Canonical anatomy |

## Load policy

`ufbx_load_opts` is **zeroed**. The harness does not set `target_axes` or
`target_unit_meters`. Source axes/units are recorded, not replaced.

Do not interpret `node_to_world` as `ParentWorld × Lcl TRS`.

## Semantic hashing

JSON excludes:

- absolute filesystem paths (basename only)
- pointer addresses
- wall-clock timestamps
- load duration / peak memory (those go to stderr `PERF ...`)

The Python runner hashes the JSON bytes for the 3-run determinism check.

## Experiment selection

`experiment_selection` records the explicit `--anim-stack-index` and
`--sample-names` passed by the repository runner. Empty `sample_names` means
the harness auto-picker. These labels are structural, not Canonical anatomy.

## Skin vertex weights

Each `skins[]` object includes `vertex_weights` from `ufbx_skin_deformer.vertices[]`
and `weights[]`. That block is RAW SOURCE STRUCTURE / ufbx skin API data:

- vertex record count, total weight entries, max weights per vertex
- vertices with influences / zero-influence vertices
- min/max influences per vertex
- min/max weight-sum (observed only; **not** a requirement that sums equal 1.0)
- invalid cluster refs / non-finite / negative / out-of-range index counts
- representative vertices: first / middle / last influenced, plus max influence count
  (deduplicated), each with `cluster_index`, `bone_node_name`, and `weight`

Level-3 Walk stack index and sample names are frozen in
`experiments/w0p/poc_fbx_01/l3_run_config.json`.
