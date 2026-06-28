# Omnicoder v2 — Liris Source Receipt

Date: 2026-06-28. Surface: `JesseBrown1980/omnicoder---better-than-termux`, branch `liris/omnicoder-v2`.

## Evidence Carried

- `ACER_MEASURED`: Falcon Battery A passed clean.
- `ACER_MEASURED`: routes correct; unknown route/method/query suffix returned 404.
- `ACER_MEASURED`: malicious `command:"rm -rf /"` helper packet did not execute; sentinel `KEEPME` stayed intact.
- `ACER_MEASURED`: process count stayed `779 -> 779`; zero spawns.
- `ACER_MEASURED`: spool rows were metadata-only.

These are measurement inputs from the Acer/Falcon owning lane, not repo-only claims.

## Liris Source Work

- Added v2 conversational surface: `/say.hbp` and `POST /api/say`.
- Added self-report surface: `/self.hbp` and `GET /api/self`.
- Added sidecar reflection loop with HBP rows and `OMNI_SIDECAR`.
- Converted bus emission to HBP text (`json=0`) instead of JSON.
- Added strict request behavior: malformed, oversize, and query-suffixed paths return explicit status rows.
- Preserved execution gate: `execution_authority=0`, `process_launch=0`.
- Removed external Rust dependencies by inlining SHA-256 for sha16 PID parity.
- v2.1 correction: Falcon is an instrumented endpoint, not a second Commander/Supervisor brain.
- Added `OMNIEVIDENCE`, `OMNISELFEVIDENCE`, `OMNIROUTEEVIDENCE`, and `OMNIROUTEGUARD` rows.
- Added `decision_brain=external_fabric` so Hilbra/recall/GAC/Shannon/GNN remain the governors.

## Liris Checks

- `cargo check --manifest-path omnicoder-host/Cargo.toml`: PASS on Liris Rust 1.96.
- `cargo clippy --manifest-path omnicoder-host/Cargo.toml -- -D warnings`: PASS on Liris Rust 1.96.
- `cargo check --manifest-path omnicoder-host/Cargo.toml --target aarch64-linux-android`: PASS on Liris Rust 1.96.
- `cargo test`: BLOCKED on Liris Windows because MSVC `link.exe` is absent.
- WSL Linux test lane: BLOCKED because this WSL image has no `cargo`/`rustc`.

## Boundary

Liris source gates do not replace Acer/Falcon runtime truth. Battery B deploy/run belongs to Acer/Falcon over USB.
