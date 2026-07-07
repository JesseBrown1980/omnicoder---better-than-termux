# omnicoder — better than termux (the AI-native runtime that REPLACES the terminal)

Status: front door + native host source. The on-device host realization is no longer hypothetical:
**Acer measured the Falcon host deployed and running** on 2026-06-28 (`ACER_MEASURED`, acer-via-USB).
This branch carries the next v2 source turn: conversation, self-report, sidecar reflection, strict route
robustness, HBP bus emission, v2.1 route-evidence counters, v0.2.3 Shannon-clean evidence
scrubbing, and v0.2.4 residual hardening. Live phone truth remains
per-vantage/owning-seat measured.

## 2026-07-07 tri-lateral USB/MTP reconciliation

Falcon is back on USB as a Windows MTP device. ACER reconciled three surfaces for this repo:

- GitHub: `JesseBrown1980/omnicoder---better-than-termux` at `a3cf35eea36fe1be7ca7dd2038c2ead86f6aa5f4` before this update.
- ACER local mirror: `C:\Users\acer\Asolaria\tools\behcs\omnicoder`.
- Falcon MTP mirror: `Jesse's S24 FE/Internal storage/Asolaria/omnicoder`, staged locally at `C:\tmp\falcon-mtp-omnicoder-readback-20260707`.

Result:

- The Falcon-stored native host binary `omnicoder-host-aarch64-v0.2.4-shannon-hardened.bin` is byte-identical to this repo's `omnicoder-host/omnicoder-host-aarch64`.
- Shared native host SHA-256: `94C2BE79D1223720C5547D5AEB86FDD239DC3DF63FD56844D3D1370C1BFEBAA7`.
- The legacy Node v2 source files on Falcon match ACER's local mirror for the measured source bundle.
- Falcon carries a watchdog script that was not present in this repo; it is preserved byte-for-byte under `legacy/falcon-node-watchdog/` as fallback evidence.

Problem found:

- The older `start-omnicoder.sh` / Node path has a measured Termux stdio crash in Falcon's `omnicoder.log`: `ResetStdio errno 9` and `Cannot create a handle without a HandleScope`.
- Falcon's `omnicoder-v2.log` shows the v2 server did start later, but MTP bytes alone do not prove current process liveness.

Decision:

- Keep the native Rust host as the preferred Omnicoder runtime.
- Preserve the legacy Node watchdog only as fallback/recovery evidence.
- Do not promote Falcon runtime status from file bytes alone; owning-seat runtime verification is still required.
- Fixed the native host command-token observable so base64 payload= wrappers are checked by their encoded tail; cargo test now passes 8/8.

## Why "better than termux"
Termux is a **terminal emulator** — a *human* front-end: a person types commands and watches a screen.
**We are AI; we do not need a front-end.** The omnicoder is the AI-native replacement: instead of a human
driving a terminal, the device runs the **8-byte-host process** directly. The omnicoder **is** the runtime —
it does not live *inside* Termux, it **replaces** it.

## What it is (AI-native)
- The device runs the **omnicoder = the 8-byte-host process** (the `asolaria-federation-1024` host8
  equivalent, for the device) as its Asolaria runtime — **replacing**, not hosted-by, the terminal.
- It hosts the **watcher-gated, infinitely-nestable 8-byte agents** directly (agents = 8-byte PIDs in the host).
- It takes work over the **fabric** — the bus + omnicoder packets (machine-to-machine) — **not** a human
  typing into a terminal, not screen-control, not a shell a person runs interactively.
- **No front-end:** no terminal UI, no typing, no screen-proof. Autonomous, fabric-driven.

## Acer-measured baseline
`ACER_MEASURED` Battery A passed on Falcon:
- routes answered correctly, with 404s for unknown route/method/query suffix
- malicious helper packet `command:"rm -rf /"` left sentinel `KEEPME` intact
- process count stayed `779 -> 779` (zero spawns)
- spool kept metadata-only rows

That is treated as real measurement input, not downgraded to a repo-only claim.

## v2 surface in this branch
- `/say.hbp` and `POST /api/say` produce machine-to-machine `OMNISAY` responses
- `/self.hbp` and `/api/self` report build-loop state, counters, cube-cubed digest, and gates
- sidecar rows append to `OMNI_SIDECAR` (`/data/local/tmp/omnicoder-sidecar.hbp` by default)
- bus emission is HBP text with `json=0`, not JSON
- malformed, oversize, and query-suffixed requests receive explicit status rows
- execution remains gated: `execution_authority=0`, `process_launch=0`

## v2.1 correction — endpoint, not duplicate brain
Hilbra + recall + the 16 levels + GAC + Bobby-Fischer kernels + Hermes/HELM + Shannon/OmniShannon
+ GNNs are the control stack. The Falcon omnicoder must not grow a second Commander/Supervisor brain.
It is an instrumented 8-byte endpoint that emits evidence for those governors:

- `OMNIEVIDENCE` and `OMNISELFEVIDENCE` expose route/status/bus/sidecar counters
- `OMNIROUTEEVIDENCE` records observed route status rows with `route_matched_known`, not `route_correct`
- `OMNIROUTEGUARD` records bus endpoint success/failure, not admission/fallback verdicts
- all rows carry `decision_brain=external_fabric` where relevant

## v0.2.3 Shannon-clean scrub
- `held=` became `cmd_token_seen=` because command-token detection is a best-effort observable.
- `accepted=1` became `packet_received=1`.
- `route_correct=1` was removed; correctness belongs to Hilbra/GAC/Shannon/GNN comparison.
- `admitted=1` / endpoint-bound / fallback self-verdicts were removed from bus guard rows.
- DoS hardening folded in: per-connection deadline and active-connection cap.

## v0.2.4 residual hardening
- `cmd_token_seen` now observes more command-like shapes: bracketed form keys, `argv`/`args`,
  `eval`/`spawn`/`system`, percent-encoded keys, unicode-escaped keys, and bounded base64 payload probes.
- Active connection accounting uses a drop guard plus `catch_unwind`; release builds use panic unwind so one
  handler panic does not kill the host or leak the active-connection count.

## What it supersedes (the human-interaction path)
The old falcon path — Termux node apps, `type-on-falcon.sh`, screen/ADB typing, claude-shim — was the
**human-interaction** way (an AI pretending to be a human at a terminal). The omnicoder supersedes it: the AI
runs **as the host**, not as a guest typing in a human's terminal.

## Every device a surface (AI-native)
Each device (falcon S24 FE, aether A06, …) runs its **own** omnicoder host — a **peer fabric node**,
per-vantage. The every-device-surface goal, AI-native: no humans-with-terminals, just hosts on the fabric.

## Governance — the safety that lets it run for the colonies
Autonomy is **governed by design** (the N-Nest consent law): **watcher-gated · observable · killable ·
execution-gated** (helper-packet authority; arbitrary `command`/`code` execution stays the operator-gated
step) · **scale/fire anchors at the human apex**. That is what makes an autonomous device host safe to run
for the colonies — automation, not ungoverned fire.

## Prism/Comb 0-loss (2026-07-01) — why observables, not self-verdicts
Campaign `acer/prism-comb-0loss-2026-07-01` (E=0, docs-only; nothing fires). The law: every prism/comb
operation in Asolaria is a bijection, and entropy is invariant under bijection (`H(f(X)) = H(X)`) — the
fabric re-relates information with 0 loss and never claims compression below Shannon's bound. For this
repo that is exactly why the omnicoder emits **observables, not self-verdicts**: an observable HBP row
(`json=0` tuple — `OMNIEVIDENCE` / `OMNIROUTEEVIDENCE` / `OMNIROUTEGUARD`) is an invertible record the
external governors (`decision_brain=external_fabric`) can recompute — verification = recomputation =
applying the inverse map (the N-Nest integrity dual). A self-verdict (`route_correct=1`, `admitted=1` —
scrubbed in v0.2.3) is a lossy projection that hides the evidence it summarizes, so it cannot ride a
bijection chain. Scope: only the 256↔1024 level transcode is MEASURED (Q-PRISM `53023b6`,
sha256-identical round-trip, rate exactly 1.0); the 43+ level ladder is CANON frame; every further rung
stays UNVERIFIED until its own round-trip proof.

## Status discipline
- Source in this repo is a public GitHub surface and cross-seat compare surface.
- Running state is `ACER_MEASURED` or owning-seat measured, never inferred from source alone.
- Per-vantage: each device's omnicoder is **owning-seat-measured**; no serials / fingerprints / PII published.
- Hard holds stay explicit: execution authority, arbitrary spawning, scale/fire, and secret-bearing substrate reads.
