# POC-FBX-01 controlled fixtures

**RESEARCH ONLY / W0-P / NON-PRODUCTION**

These files are project-authored. They are not Canonical assets and not
Level-3 corpus members.

| File | Question |
| --- | --- |
| `../../../poc_core_01/fixture/minimal_chain.fbx` (reused, not copied) | Hierarchy / names |
| `ordinary_anim.fbx` | Ordinary animation stack + two layers (second layer empty) |
| `authored_local.fbx` | Non-plain-TRS authored local recipe; two children of the implicit root; Null helper |

`authored_local.fbx` GlobalSettings use **Z-up** and **UnitScaleFactor 2.54** so the
harness cannot silently assume Y-up centimetres.

Do not treat evaluated `local_transform` as the authored pivot/pre/post recipe.
