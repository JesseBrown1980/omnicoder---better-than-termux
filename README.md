# omnicoder — better than termux (the AI-native runtime that REPLACES the terminal)

Status: docs-first / **E=0**. Front door + frame. The on-device host realization is the **build target** —
not claimed running until observed (per-vantage, owning-seat). Corrects a prior human-framed draft.

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

## Status / build target (honest, no overclaim)
- This repo = the front door + frame.
- The **device-native 8-byte-host realization** — *how* host8 runs as the device's native runtime,
  replacing the terminal (cross-compiled native binary / daemon vs other substrate) — is the **build
  target**, to be specified with the operator + the running system, then deployed and **observed**
  (MEASURED only once the device serves it).
- Per-vantage: each device's omnicoder is **owning-seat-measured**; no serials / fingerprints / PII published.
- Hard holds (operator T0): device-native deploy/run · execution authority · scale.
