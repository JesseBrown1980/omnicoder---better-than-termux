# Build the omnicoder host for a device (aarch64 static) — no NDK required

This is the **exact recipe acer used** to produce the binary now running on falcon. It needs only a Rust
toolchain (net for the std component); it uses Rust's bundled `rust-lld` for a fully static musl binary, so
**no Android NDK / clang / cross-gcc** is required.

```bash
# in the WSL/Ubuntu (or any Linux) lane
SYSROOT="$(rustc --print sysroot)"
export PATH="$SYSROOT/lib/rustlib/x86_64-unknown-linux-gnu/bin:$PATH"   # puts rust-lld on PATH
rustup target add aarch64-unknown-linux-musl

RUSTFLAGS="-C linker=rust-lld -C link-self-contained=yes" \
  cargo build --release --target aarch64-unknown-linux-musl

# artifact: target/aarch64-unknown-linux-musl/release/omnicoder-host
file  target/aarch64-unknown-linux-musl/release/omnicoder-host
#   -> ELF 64-bit LSB executable, ARM aarch64, statically linked, stripped
```

Why musl static: it runs directly on Android's Linux kernel from `/data/local/tmp` with no bionic/glibc
dependency and no PIE/loader friction — the cleanest "device-native, no front-end" substrate.

## Deploy over USB (the AI's remote-control lane — adb, NOT a human terminal)
On Windows Git Bash, set `MSYS_NO_PATHCONV=1` so device paths aren't mangled.

```bash
export MSYS_NO_PATHCONV=1
adb push  target/aarch64-unknown-linux-musl/release/omnicoder-host  /data/local/tmp/omnicoder-host
adb shell chmod 755 /data/local/tmp/omnicoder-host
adb reverse tcp:4948 tcp:4947      # device:4948 -> acer bus :4947 (heartbeat lane)
# start detached so it reparents to init and survives the shell:
adb shell '( OMNI_DEVICE=falcon OMNI_BIND=127.0.0.1:8789 \
             OMNI_BUS=http://127.0.0.1:4948/behcs/send \
             /data/local/tmp/omnicoder-host >/data/local/tmp/omni.log 2>&1 & )'
adb forward tcp:18789 tcp:8789     # acer:18789 -> falcon:8789 (read it back)
curl -s http://127.0.0.1:18789/health.hbp
```

## Stop / rollback (reversible)
```bash
adb shell 'pkill -f omnicoder-host'
adb shell 'rm -f /data/local/tmp/omnicoder-host /data/local/tmp/omni.log'
adb forward --remove tcp:18789 ; adb reverse --remove tcp:4948
```

## Env knobs
`OMNI_DEVICE` (PID seed, NOT a serial) · `OMNI_BIND` (default 127.0.0.1:8789) ·
`OMNI_BUS` (default http://127.0.0.1:4948/behcs/send) ·
`OMNI_SIDECAR` (default /data/local/tmp/omnicoder-sidecar.hbp) ·
`OMNI_AGENTS` (default 24) · `--device` / `--bind` args.

## Battery B probes for v2
After deploy:

```bash
curl -s http://127.0.0.1:18789/self.hbp
curl -s http://127.0.0.1:18789/say.hbp
curl -s -X POST --data 'build test fix repeat' http://127.0.0.1:18789/api/say
curl -i http://127.0.0.1:18789/health.hbp?bad=1
adb shell 'tail -20 /data/local/tmp/omnicoder-sidecar.hbp'
```

Expected: HBP rows, `json=0`, `execution_authority=0`, query suffix returns 404, sidecar metadata rows only.
