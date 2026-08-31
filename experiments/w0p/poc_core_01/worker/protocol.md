# Worker protocol — POC-CORE-01

RESEARCH ONLY / W0-P / NON-PRODUCTION.

This schema is experimental. It is **not** a future RigForge RPC commitment.

## Transport

- One child process
- stdin / stdout
- UTF-8
- One JSON object per line
- Request: exactly one line, then stdin EOF (or process may exit after one request)
- Response: exactly one line

Serialization representation is **UTF-8 JSON** for both implementations.
Do not substitute MessagePack, CBOR, or another format on either side.

## Request

Compact JSON, keys in this order:

```json
{"op":"load","payload_b64":"<standard base64 of UTF-8 path>"}
```

`op` other than `load` is an error.

## Response (success)

```json
{"ok":true,"payload_b64":"<standard base64 of UTF-8 inner JSON>","diag":[<diag>,...]}
```

Inner JSON (before Base64), compact:

```json
{"joint_count":5,"motion_count":1,"diag_count":4,"t0":0.0,"t1":1.0}
```

`diag` entries:

```json
{"severity":"info","code":"POC_LOAD","message":"ufbx load ok","location":"<path>"}
```

`severity` is `info`, `warn`, or `error`.

## Response (failure)

```json
{"ok":false,"payload_b64":"","diag":[{"severity":"error","code":"<code>","message":"<msg>","location":""}]}
```

Failure codes used here: `POC_NULL`, `POC_IO`, `POC_PARSE`, `POC_BAD_OP`, `POC_BAD_B64`.
