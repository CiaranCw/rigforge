# POC-CORE-01 research fixture

This directory is **research-only / W0-P / non-production**.

`minimal_chain.fbx` is a project-authored ASCII FBX 7.5 skeleton chain.
It is not a Canonical asset, not a Level-3 corpus member, and not an
FBX-semantics oracle (that is POC-FBX-01).

Purpose: give both language candidates the same ufbx load input for
joint names, hierarchy, rest local transforms, and a diagnostic/error path.

`long_name.fbx` is a project-authored ASCII FBX 7.5 file whose Model name
is longer than 1023 bytes. `long_name.expected.txt` is the exact UTF-8
name payload. This fixture is **not** FBX semantic validation. It only
proves implementation equivalence for the string/lifetime boundary.
