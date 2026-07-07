# Falcon MTP Tri-lateral Native Host Reconciliation

Date: 2026-07-07
Recording node: ACER
Repo: `JesseBrown1980/omnicoder---better-than-termux`

## Scope

This note records the native-host side of the tri-lateral reconciliation between:

- GitHub repo bytes;
- ACER local Omnicoder mirror bytes;
- Falcon Windows MTP file-manager bytes.

The Falcon MTP path was visible through Windows Explorer/File Manager as:

```text
Jesse's S24 FE/Internal storage/Asolaria/omnicoder
```

## Native Host Result

The Falcon-stored native host binary matches the binary already tracked in this repo.

```text
repo_path: omnicoder-host/omnicoder-host-aarch64
falcon_mtp_file: C:\tmp\falcon-mtp-omnicoder-readback-20260707\omnicoder-host-aarch64-v0.2.4-shannon-hardened.bin
sha256: 94C2BE79D1223720C5547D5AEB86FDD239DC3DF63FD56844D3D1370C1BFEBAA7
result: MATCH
```

No native host binary replacement is needed.

## Legacy Node Result

Falcon also carries the older Node/Termux Omnicoder tree. Shared source files match ACER's local mirror, but this lane is not primary.

Measured problem in Falcon `omnicoder.log`:

```text
ResetStdio errno 9
Cannot create a handle without a HandleScope
```

The fallback watchdog recovered by launching Node with detached stdio:

```sh
setsid "$NODE" "$SERVER" </dev/null >> "$LOG" 2>&1 &
```

That watchdog is preserved byte-for-byte at:

```text
legacy/falcon-node-watchdog/falcon-omnicoder-persistent.sh
sha256: 056F2F022D718F4ED8F5C394872DBC6C454FBD096330F9DC9BF04D63DC27EAB1
```

## Decision

```text
KEEP_NATIVE_RUST_HOST_AS_PRIMARY
PRESERVE_LEGACY_NODE_WATCHDOG_AS_FALLBACK_EVIDENCE
DO_NOT_PROMOTE_RUNTIME_ON_MTP_BYTES_ALONE
```

## Boundary

- MTP file bytes are evidence of files, not proof of live runtime state.
- Native host source and binary equality do not grant execution authority.
- Legacy Node watchdog preservation does not re-promote Termux as primary.
- Owning-seat runtime verification is required before reporting Falcon as live.