# Trilateral evidence for omnicoder device endpoints — 2026-07-11

Canonical doctrine:
[`HYPER-BECHS--the-third-set/TRILATERAL-REALITY-EVIDENCE-DOCTRINE-2026-07-11.md`](https://github.com/JesseBrown1980/HYPER-BECHS--the-third-set/blob/main/TRILATERAL-REALITY-EVIDENCE-DOCTRINE-2026-07-11.md)

## Preserve the measured endpoint

The Falcon deployment and Battery-A observations are Acer/owning-seat measurements. They are not
downgraded to source-only claims merely because another seat does not possess that phone or USB
vantage.

They also do not prove the same host build is currently deployed on every device.

## Evidence ladder

```text
1. Rust source and deterministic tests exist
2. independent third seat builds/tests the host on a supported target
3. owning seat deploys the artifact to a named device PID
4. device-local routes, counters, sentinel, process count, and spool are measured
5. cross-fabric governors consume and compare emitted evidence
```

Each rung is real in scope. A desktop build is not a phone deployment; a phone route response is not
a global fabric verdict.

## Observable, not self-verdict

The endpoint must emit evidence such as route/status/counter rows. Correctness, admission, fallback,
promotion, and authority belong to the external fabric governors.

```text
packet_received     -> observable
route_matched_known -> observable
status_code          -> observable
route_correct        -> external comparison, not endpoint self-verdict
admitted             -> external authority decision
```

This design is not a deflation of the endpoint. It is what makes its evidence independently
recomputable.

## Trilateral roles

- Acer/owning seat: deploys and measures the physical endpoint.
- Liris/peer seat: attacks route semantics, HBP rows, and governance integration.
- Third seat: builds/tests public source and validates deterministic rows without device secrets.
- CI: runs immutable-head tests and static checks.

A third seat does not receive private device identifiers, helper authority, or arbitrary command
execution.

## No-deflate / no-inflate

Reject both:

```text
"no human terminal, therefore no real runtime"               -> deflation
"source builds, therefore Falcon is currently deployed"      -> inflation
"command token seen, therefore malicious command executed"   -> inflation
"execution_authority=0, therefore endpoint is decorative"    -> deflation
"one phone measured, therefore every device surface is live" -> inflation
```

## Merge rule

Merge route hardening, deterministic evidence, tests, crash containment, and privacy-safe receipts
when clean. Hold arbitrary execution, helper authority, device secrets, deployment claims, scale/fire,
or cross-device generalization until owning-seat measurement and operator authorization exist.
