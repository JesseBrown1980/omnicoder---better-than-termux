# Falcon Orbital Link

This is the no-USB operating pattern for turning Falcon into an orbital Asolaria
worker. The phone does not wait for a host to reach through a cable. The phone
calls the fabric over the LAN, and the host exposes only the Shannon-clean
omnicoder endpoint.

## Measured State

Evidence class: `OPERATOR_OBSERVED_ACER`

- Acer LAN: `192.168.1.9`
- Falcon LAN: `192.168.1.6`
- Falcon omnicoder bind: `0.0.0.0:8789`
- Falcon health endpoint: `http://192.168.1.6:8789/health.hbp`
- Acer bus target used by Falcon: `http://192.168.1.9:4947/behcs/send`
- Recall/HBI hot path exposed on Acer LAN: `:4796`
- Falcon host PID8 observed by Acer: `4c7e27b6bfb76666`
- Version served on Falcon: `0.2.4-shannon-hardened`
- USB requirement after launch: `0`

## Launch Shape

Falcon runs the omnicoder host with explicit LAN bind and Acer bus target:

```bash
./omnicoder-host-aarch64 \
  --bind 0.0.0.0:8789 \
  --bus http://192.168.1.9:4947/behcs/send \
  --sidecar /data/local/tmp/omnicoder-sidecar.hbp
```

The host is allowed to emit receipts and local sidecar rows. It has no execution
authority. The execution gate remains structural:

```text
execution_authority=0|process_launch=0|json=0
```

## Fabric Contract

Falcon sends observables, not verdicts:

```text
ORBITALCALL|from=falcon|to=acer|verb=omnicoder.heartbeat|host_pid8=4c7e27b6bfb76666|usb_required=0|json=0
ORBITALENDPOINT|host=falcon|endpoint=http://192.168.1.6:8789|bind=0.0.0.0:8789|bus=http://192.168.1.9:4947/behcs/send|json=0
```

Hilbra, recall, GAC, Shannon, OmniShannon, and the GNN lanes own correctness
judgments above this endpoint.

## Key Boundary

The existing Acer-side registry indicates Falcon enrollment already exists.
Do not create or publish new secret keys from this repo. Public receipts may name
handles, endpoints, host PID8, timestamps, and SHA values. Private key material,
device serials, and vault paths stay in the owning backend/vault only.

## Liris Attach

The same orbital technique can be repeated from Liris once Falcon is brought to
that lane. Liris should register only public handles and receipts, then let the
backend key method bind the route through Hilbra/atlas/recall. USB is a temporary
transport for the first attack/attach, not a runtime dependency.
