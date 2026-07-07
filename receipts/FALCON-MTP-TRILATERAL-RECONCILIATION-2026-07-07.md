# Falcon MTP Tri-lateral Omnicoder Reconciliation - 2026-07-07

```yaml
receipt_id: FALCON-MTP-TRILATERAL-RECONCILIATION-2026-07-07
repo: JesseBrown1980/omnicoder---better-than-termux
recording_node: ACER
claim_class: MEASURED_ACER_USB_MTP_AND_GITHUB_COMPARE
status: TRILATERAL_RECONCILIATION_PUBLISHED_FOR_LIRIS_AND_RELIC_REVIEW
```

## Purpose

Record the 2026-07-07 reconciliation between:

- GitHub repo bytes;
- ACER local Omnicoder mirror bytes;
- Falcon Windows MTP file-manager bytes.

## GitHub Surface

```text
repository: JesseBrown1980/omnicoder---better-than-termux
base_head_before_update: a3cf35eea36fe1be7ca7dd2038c2ead86f6aa5f4
branch: codex/falcon-mtp-trilateral-omnicoder
```

## Falcon MTP Surface

```text
device: Jesse's S24 FE
path: Internal storage/Asolaria/omnicoder
local_stage: C:\tmp\falcon-mtp-omnicoder-readback-20260707
```

## Native Host Result

The Falcon-stored native host binary and the GitHub-tracked native host binary are byte-identical.

```text
repo_path: omnicoder-host/omnicoder-host-aarch64
falcon_mtp_file: omnicoder-host-aarch64-v0.2.4-shannon-hardened.bin
sha256: 94C2BE79D1223720C5547D5AEB86FDD239DC3DF63FD56844D3D1370C1BFEBAA7
result: MATCH
```

No native binary replacement was required.

## Legacy Node Mirror Result

The measured Falcon legacy Node source bundle matches ACER's local mirror for the shared files.

| File | SHA-256 | Result |
| --- | --- | --- |
| `omnicoder-server-v2.mjs` | `18478A7E7B28E74A3256EEC59289969417238FC0EA94FEB9310F4BF061070713` | ACER == FALCON |
| `omnicoder-server.mjs` | `54C40078E5A4CFCB0E7A57888ED15BA1653D44874E74B0FB36DEE1F17AA894C5` | ACER == FALCON |
| `start-omnicoder.sh` | `258E2A11CA8BA26990FA1D6DFF46D139FEB894210377034BB55F3ADF0E23112B` | ACER == FALCON |
| `public/index.html` | `E9F56482B6AD5011DB8F5A2E32842A44F089AEBA9BE073E9DC82571465C12599` | ACER == FALCON |
| `lib/hyperbehcs-core.cjs` | `668A938A8B0281694797D64A6FF31921031B9AF58FD7AFB77A50DD3E0EE31E08` | ACER == FALCON |
| `lib/zeta-process.mjs` | `77D6A53AA27BF21945BED6F8254F69E3E8E480FD57498A4701C60109EEAF3A26` | ACER == FALCON |
| `lib/hrm-slow-fast.mjs` | `A5EC3B7B9176AADCD6D213D474128642A76CBA3B799594B697B0BDCC609141BA` | ACER == FALCON |
| `lib/mtp-heads.mjs` | `63559E1C605E626FFCE08CA7456ECA54653B12CE5641B2618355B89461E79490` | ACER == FALCON |
| `lib/primes.mjs` | `1284CE7AFF8844321D462C0E9975AE1598779B95419F74295FD3E85C0DCE6291` | ACER == FALCON |
| `lib/hilbert.mjs` | `61A4C4AE6BED569576BD7EDF2EF15F7E804490F5A40F8F9B56D6E8BAD839EC0A` | ACER == FALCON |

Falcon also carried `falcon-omnicoder-persistent.sh`, which was absent from the GitHub repo and ACER local mirror. It is now preserved under:

```text
legacy/falcon-node-watchdog/falcon-omnicoder-persistent.sh
sha256: 056F2F022D718F4ED8F5C394872DBC6C454FBD096330F9DC9BF04D63DC27EAB1
```

## Problem Found

Falcon's legacy `omnicoder.log` records the older Node/Termux path crashing with:

```text
ResetStdio errno 9
Cannot create a handle without a HandleScope
```

Falcon's `omnicoder-v2.log` records a later v2 listener line, but MTP file bytes alone do not prove current runtime liveness.


## Native Host Test Fix

During ACER validation, `cargo test --manifest-path omnicoder-host/Cargo.toml` initially exposed one failing test: the command-token observable did not inspect the encoded tail of `payload=<base64>` wrappers.

The fix is intentionally narrow:

```text
base64_command_key_seen now checks both the raw token and the assignment tail after the first '='.
```

Validation after the fix:

```text
cargo test --manifest-path omnicoder-host/Cargo.toml
result: 8 passed / 0 failed
```
## Decision

```text
KEEP_NATIVE_RUST_HOST_AS_PRIMARY
PRESERVE_LEGACY_NODE_WATCHDOG_AS_FALLBACK_EVIDENCE
DO_NOT_PROMOTE_RUNTIME_ON_MTP_BYTES_ALONE
```

## Boundary

- MTP bytes are file evidence, not runtime liveness.
- Legacy packets and logs are evidence, not execution authority.
- Runtime-online status requires owning-seat runtime verification.
- This reconciliation does not create Canon promotion.
- This update does not start Termux, ADB, USB write, deploy, QEMU, or boot work.

## Result

```text
TRILATERAL_RECONCILIATION_PUBLISHED_FOR_LIRIS_AND_RELIC_REVIEW
```