# The falcon phone interface (AI-native, machine-to-machine)

How the fabric talks to falcon — **not** a screen a human taps, not Termux typing. The phone runs the
`omnicoder-host` process; acer reaches it over the USB transport (adb) as a peer fabric node.

```
   ACER (desktop, build+control seat)                 FALCON (S24 FE, device host)
   ┌───────────────────────────────┐                 ┌──────────────────────────────┐
   │ omni-falcon-ctl.sh (this dir)  │                 │ omnicoder-host (pid, in init) │
   │   curl 127.0.0.1:18789  ───────┼── adb forward ──┼─▶ :8789  /health /agents      │
   │                                │   18789→8789    │        /ports /cube /say /self │
   │                                │                 │        /api/packet /api/say    │
   │ acer bus :4947  ◀──────────────┼── adb reverse ──┼── 127.0.0.1:4948 (heartbeat)  │
   └───────────────────────────────┘   4948→4947     └──────────────────────────────┘
```

Two bridges, both pure transport (no human in the loop):
- **read/control** — `adb forward tcp:18789 tcp:8789`: acer's `127.0.0.1:18789` *is* falcon's omnicoder.
  Poll health, list the 8-byte agents, read the cube-cubed digest, POST helper packets, ask `/self.hbp`,
  and talk machine-to-machine through `/api/say`.
- **bus/heartbeat** — `adb reverse tcp:4948 tcp:4947`: falcon's `127.0.0.1:4948` reaches the acer bus
  `:4947`, so the device announces online + pulses into the fabric machine-to-machine.

Per-vantage law: this is the **acer↔falcon** interface over USB. falcon's `:8789` is its own loopback
(owning seat = falcon); acer measures it only while USB-attached, tagged `acer-via-USB`. Starting/stopping
the host is device control = operator-T0 (authorized for this falcon upgrade).

Governance: the interface carries **helper packets**; `execution_authority=false` on the host means a
packet command-like tokens are observed as `cmd_token_seen`; nothing is executed. Scale/spawn/fire stay operator-gated.

## Acer-measured Battery A baseline
`ACER_MEASURED` on Falcon, 2026-06-28:
- route table passed, including correct 404s for unknown route/method/query suffix
- `command:"rm -rf /"` stayed helper-only; sentinel `KEEPME` remained intact
- process count stayed `779 -> 779`
- spool rows were metadata-only

## v2 routes
- `GET /say.hbp` — emits a minimal `OMNISAY` row
- `POST /api/say` — accepts machine-to-machine text and answers with `OMNISAY`
- `GET /self.hbp` / `GET /api/self` — reports counters, build-loop state, cube-cubed digest, and gates
- sidecar: rows append to `OMNI_SIDECAR` (default `/data/local/tmp/omnicoder-sidecar.hbp`)
- bus: emits HBP text rows to `OMNI_BUS`, with `json=0`

## v2.1 evidence endpoint rule
The phone host is not a duplicate Hilbra/Shannon/GNN controller. It reports:

- `OMNIROUTEEVIDENCE` for route status and route-match observables
- `OMNIROUTEGUARD` for bus endpoint success/failure observables
- `OMNIEVIDENCE` / `OMNISELFEVIDENCE` aggregate counters

The decision brain is explicitly upstairs: `decision_brain=external_fabric`.

## Shannon-clean evidence shape
The host must not assert correctness or admission. It emits observables only:

- `route_matched_known={0|1}` instead of `route_correct=1`
- `cmd_token_seen={0|1}` instead of `held=`
- `packet_received=1` instead of `accepted=1`
- `bus_post_ok` / `bus_post_failed` instead of `admitted`, `endpoint_bound`, or `fallback`

## Shannon residual hardening
- `cmd_token_seen` is still an observable, not an execution guarantee, but it now catches bracketed form keys,
  broader command-like key names, percent/unicode escaped keys, and bounded base64 payload probes.
- Active-connection accounting is guarded by `Drop`; release panic mode is unwind so a handler panic is
  contained to that connection and releases the connection slot.
