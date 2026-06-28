# The falcon phone interface (AI-native, machine-to-machine)

How the fabric talks to falcon — **not** a screen a human taps, not Termux typing. The phone runs the
`omnicoder-host` process; acer reaches it over the USB transport (adb) as a peer fabric node.

```
   ACER (desktop, build+control seat)                 FALCON (S24 FE, device host)
   ┌───────────────────────────────┐                 ┌──────────────────────────────┐
   │ omni-falcon-ctl.sh (this dir)  │                 │ omnicoder-host (pid, in init) │
   │   curl 127.0.0.1:18789  ───────┼── adb forward ──┼─▶ :8789  /health /agents      │
   │                                │   18789→8789    │        /ports /cube /api/packet│
   │ acer bus :4947  ◀──────────────┼── adb reverse ──┼── 127.0.0.1:4948 (heartbeat)  │
   └───────────────────────────────┘   4948→4947     └──────────────────────────────┘
```

Two bridges, both pure transport (no human in the loop):
- **read/control** — `adb forward tcp:18789 tcp:8789`: acer's `127.0.0.1:18789` *is* falcon's omnicoder.
  Poll health, list the 8-byte agents, read the cube-cubed digest, POST helper packets.
- **bus/heartbeat** — `adb reverse tcp:4948 tcp:4947`: falcon's `127.0.0.1:4948` reaches the acer bus
  `:4947`, so the device announces online + pulses into the fabric machine-to-machine.

Per-vantage law: this is the **acer↔falcon** interface over USB. falcon's `:8789` is its own loopback
(owning seat = falcon); acer measures it only while USB-attached, tagged `acer-via-USB`. Starting/stopping
the host is device control = operator-T0 (authorized for this falcon upgrade).

Governance: the interface carries **helper packets**; `execution_authority=false` on the host means a
packet's `command`/`code` is **HELD**, never executed. Scale/spawn/fire stay operator-gated.
