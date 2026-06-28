# omnicoder — the "better than termux" on-device / remote coder

Status: docs-first / **E=0**. Front door for the omnicoder surface. Not a runtime fire; the on-device
source + live state are owning-seat (falcon). Per-vantage: `:8789` is a device loopback, not federation-global.

## What it is
The **omnicoder** is Asolaria's on-device coding surface — *better than termux* — the lane through which a
**device vantage does coding work remotely** in the every-device federation (acer + liris + falcon + aether).
It runs on the **falcon S24 FE phone** at **`http://127.0.0.1:8789`** (falcon's loopback — **dark from other
seats by design**; only falcon's vantage measures it live). PID-office seat: `agt-OMNICODER-PID-FALCON-8789`.

## falcon works remotely through it (helper-packet lane)
From the `falcon-omnicoder-helper-cube` (2026-05-15), the authority model is explicitly **gated**:
| dimension | value | meaning |
|---|---|---|
| `D_SURFACE` | `falcon_omnicoder_helper_packet_lane` | it's a helper-packet lane |
| `D_DEVICE` | `falcon-s24fe` | runs on the falcon phone |
| `D_BASE_URL` | `http://127.0.0.1:8789` | the omnicoder port |
| `D_HELPER_PACKET_AUTHORITY` | **true** | may assist via helper packets |
| `D_EXECUTION_AUTHORITY` | **false** | **no execution authority — gated** |
| `D_SCREEN_PROOF` / `D_ADB_CONTROL_PROOF` | false | no screen/ADB control claimed |
| `D_SCHEDULER_TASK` | `HYP-175` | scheduled lane |

So the omnicoder **assists** (helper packets) but does **not** hold execution authority — execution stays the
operator-gated step (`E=0` on the device). It has been through an `opencode-omnicoder-hardening` pass
(plan + requirements + safety-ledger + source-ledger) and carries a status-bridge.

## Place in the root
The omnicoder is a **per-device surface** instantiating the root: work it relays flows through the
watcher-gated 8-byte-agent root and the dispatcher/emitter substrate (see the system `MAP.md`,
`ROOT-PRIMITIVE`, and `FEDERATION-VANTAGE-MAP`). It is how a **phone vantage** contributes to the fabric —
the every-device-surface goal made concrete.

## Per-vantage + hard holds
- `:8789` = **falcon's loopback** — owning-seat (falcon) measures it live; not reachable/claimable from acer
  or any other seat (cross-host bus down → GitHub is the mediator).
- The on-device omnicoder **source** runs on the phone (Termux); it is **not vendored here** (owning-seat).
- **Hard holds (T0):** no execution authority (helper-packet only) · no agent spawn · no device control ·
  no fire. This repo is the **docs front door** only; the live surface is gated and owning-seat.
