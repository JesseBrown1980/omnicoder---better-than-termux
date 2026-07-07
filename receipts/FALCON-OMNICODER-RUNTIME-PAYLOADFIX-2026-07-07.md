# Falcon Omnicoder Runtime Payload Fix - 2026-07-07

```yaml
receipt_id: FALCON-OMNICODER-RUNTIME-PAYLOADFIX-2026-07-07
recording_node: ACER
repo: JesseBrown1980/omnicoder---better-than-termux
branch: codex/falcon-mtp-trilateral-omnicoder
claim_class: ACER_MEASURED_USB_OMNICODER_RUNTIME_FIX
status: FIX_DEPLOYED_AND_VERIFIED_ON_FALCON
```

## Purpose

Deploy the ACER source fix for the native Omnicoder command-token observable to Falcon and verify the exact failing live payload shape.

## Pre-fix State

ACER measured Falcon native Omnicoder running with the old binary:

```text
old_binary_sha256: 94C2BE79D1223720C5547D5AEB86FDD239DC3DF63FD56844D3D1370C1BFEBAA7
old_live_probe: payload=eyJDT01NQU5EIjoiaWQifQ==
old_live_result: cmd_token_seen=0
```

The old result was safe from execution, but incomplete as an observable:

```text
executed=0
execution_authority=0
process_launch=0
```

## Fix

The source fix checks the encoded assignment tail in `base64_command_key_seen`:

```text
payload=<base64-json> -> decode <base64-json>, not the raw payload= prefix
```

## Build

```text
build_host: ACER
rust_target: aarch64-unknown-linux-musl
RUSTFLAGS: -C linker=rust-lld -C link-self-contained=yes
cargo_build: release OK
cargo_test: 8 passed / 0 failed
```

Fixed binary:

```text
repo_path: omnicoder-host/omnicoder-host-aarch64
sha256: 10AB18E65459BEC12C2EAD10976A131175F91B9252BF163B3D692D29E0EF0372
bytes: 460472
```

## Falcon Deployment

```text
serial: R5CXA4MGQXV
model: SM-S721U1
runtime_path: /data/local/tmp/omnicoder-host
review_copy: /sdcard/Download/omnicoder-host-aarch64-v0.2.4-shannon-hardened-payloadfix-20260707.bin
old_backup: /data/local/tmp/omnicoder-host.pre-payloadfix-20260707
old_backup_sha256: 94C2BE79D1223720C5547D5AEB86FDD239DC3DF63FD56844D3D1370C1BFEBAA7
new_runtime_sha256: 10AB18E65459BEC12C2EAD10976A131175F91B9252BF163B3D692D29E0EF0372
pid_at_verification: 7628
adb_forward: ACER 127.0.0.1:18789 -> Falcon 127.0.0.1:8789
adb_reverse: Falcon 127.0.0.1:4948 -> ACER 127.0.0.1:4947
```

## Live Verification

ACER sent the exact payload shape that previously failed:

```text
POST /api/packet
body: payload=eyJDT01NQU5EIjoiaWQifQ==
```

Falcon returned:

```text
OMNIPACKET|verb=EVT-OMNICODER-HELPER-RESULT|pid8=9231ac7795301027|packet_received=1|executed=0|execution_authority=0|cmd_token_seen=1|note=cmd_token_seen-best_effort_shape_casefold-executed=0_structural_no_exec_path|process_launch=0|json=0
```

Self check after restart:

```text
OMNISELF|schema=ASOLARIA-OMNICODER-HOST|version=0.2.4-shannon-hardened|host_pid8=3fea2d9a3c634ac0|device=falcon-s24fe|state=running|replaces=termux|front_end=0|json=0
```

## Boundary

- This is an ACER-via-USB runtime fix and measurement.
- Omnicoder remains execution-gated: `execution_authority=0`, `process_launch=0`.
- The fix improves the observable, not execution capability.
- No Termux/Node path was promoted.
- No Canon promotion is claimed.
- Liris and Relic should verify through this PR and/or their own byte bridge before treating it as independent verification.

## Result

```text
FALCON_OMNICODER_PAYLOADFIX_DEPLOYED_AND_VERIFIED_BY_ACER
```