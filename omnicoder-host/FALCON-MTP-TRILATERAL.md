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

The Falcon-stored native host binary matched the binary tracked in this repo before the runtime payload-fix rebuild.

```text
repo_path: omnicoder-host/omnicoder-host-aarch64
falcon_mtp_file: C:\tmp\falcon-mtp-omnicoder-readback-20260707\omnicoder-host-aarch64-v0.2.4-shannon-hardened.bin
sha256: 94C2BE79D1223720C5547D5AEB86FDD239DC3DF63FD56844D3D1370C1BFEBAA7
result: MATCH
```

A native host binary replacement is now included in this PR because the later ACER runtime probe found and fixed the `payload=<base64>` observable gap.

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


## Runtime Payload-fix Update

After the tri-lateral file reconciliation, ACER started the native host on Falcon through the USB/OmniScrcpy lane and tested the exact unit-test failure shape against the live phone process.

Before fix:

```text
payload=eyJDT01NQU5EIjoiaWQifQ==
cmd_token_seen=0
```

Fix:

```text
base64_command_key_seen checks the encoded tail after `payload=`.
```

After fix on Falcon:

```text
path: /data/local/tmp/omnicoder-host
pid: 7628
sha256: 10AB18E65459BEC12C2EAD10976A131175F91B9252BF163B3D692D29E0EF0372
probe: payload=eyJDT01NQU5EIjoiaWQifQ==
result: cmd_token_seen=1, executed=0, execution_authority=0, process_launch=0
```

The previous binary remains backed up on Falcon:

```text
path: /data/local/tmp/omnicoder-host.pre-payloadfix-20260707
sha256: 94C2BE79D1223720C5547D5AEB86FDD239DC3DF63FD56844D3D1370C1BFEBAA7
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