# Worker Result Envelope (research)

Backend-neutral execution report.

Includes status, backend identity/version, input hashes, mapping/policy/job
hashes, clip/time facts, diagnostics, artifact hashes, and QC measurements.
Quaternion Policy execution is reported as a structural measurement, not as
Blender API identity.

Transient Blender pointers are forbidden as product truth. Adapter diagnostics
may exist under `adapter_diagnostics` only.
