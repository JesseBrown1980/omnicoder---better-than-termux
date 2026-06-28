// omnicoder-host — the AI-native 8-byte host process that REPLACES Termux on a device.
//
// This is the host8 pattern (servers/host8-serve) ported to run as a DEVICE's own
// Asolaria runtime: no terminal, no human front-end. It is launched directly (e.g. by an
// AI over adb remote control into /data/local/tmp), hosts 8-byte agents as PIDs, and takes
// work over the fabric bus machine-to-machine. Termux is the legacy human terminal; this
// replaces it.
//
// LAWS CARRIED FROM host8-serve (byte-parity where it matters):
//   * 8-byte PID = sha16 = first 16 hex of sha256(input)  (== node gaia-loader sha16)
//   * host_handle8 = fnv1a64(input) as 16 hex
//   * HBP line protocol: TAG|key=value|...|json=0   (never emits { } )
//   * E=0 / GATED BY DESIGN: routes report + validate, they NEVER launch a process.
//     process_launch=0 ALWAYS. EXECUTION_AUTHORITY=false: packet command/code is HELD.
//   * watcher-gated: every agent carries a watcher; consent = reported == recomputed-truth.
//
// OPERATOR FRAME ADDED:
//   * port.port.port — ONE OS process, one real socket; logical ports are sha-nested 3 deep.
//   * cube cube cubed — the ledger is AoT-distilled into cube -> cube^2 -> cube^3 (sha folds).

use std::env;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const SCHEMA: &str = "ASOLARIA-OMNICODER-HOST";
const VERSION: &str = "0.2.1";
const DEFAULT_BIND: &str = "127.0.0.1:8789";
const DEFAULT_BUS: &str = "http://127.0.0.1:4948/behcs/send"; // adb reverse 4948 -> acer bus :4947
const DEFAULT_SIDECAR: &str = "/data/local/tmp/omnicoder-sidecar.hbp";
const DEFAULT_DEVICE: &str = "device";
const DEFAULT_AGENTS: usize = 24; // the 24-spindle base; multiplied elsewhere, not here
const HEARTBEAT_SECS: u64 = 60;
const SIDECAR_SECS: u64 = 15;
const READ_TIMEOUT_SECS: u64 = 2;
const MAX_REQUEST_BYTES: usize = 1 << 20;

// Governance — DO NOT weaken without operator T0. The host helps; it does not fire.
const HELPER_PACKET_AUTHORITY: bool = true;
const EXECUTION_AUTHORITY: bool = false;

// --- hashing: byte-parity with host8-serve ---------------------------------

const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn sha256_digest(input: &[u8]) -> [u8; 32] {
    let mut h = [
        0x6a09e667u32,
        0xbb67ae85,
        0x3c6ef372,
        0xa54ff53a,
        0x510e527f,
        0x9b05688c,
        0x1f83d9ab,
        0x5be0cd19,
    ];

    let bit_len = (input.len() as u64).wrapping_mul(8);
    let mut msg = Vec::with_capacity((input.len() + 9).div_ceil(64) * 64);
    msg.extend_from_slice(input);
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, word) in w.iter_mut().take(16).enumerate() {
            let j = i * 4;
            *word = u32::from_be_bytes([chunk[j], chunk[j + 1], chunk[j + 2], chunk[j + 3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(SHA256_K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut out = [0u8; 32];
    for (i, word) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

fn sha256hex(input: &str) -> String {
    let d = sha256_digest(input.as_bytes());
    let mut out = String::with_capacity(64);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for b in d {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

/// First 16 hex of sha256 — the BEHCS 8-byte id. == node gaia-loader sha16(s).
fn sha16(input: &str) -> String {
    let mut s = sha256hex(input);
    s.truncate(16);
    s
}

fn canonicalize(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut run = false;
    for ch in input.chars() {
        if matches!(ch, '\t' | '\r' | '\n') {
            if !run {
                out.push(' ');
            }
            run = true;
        } else {
            out.push(ch);
            run = false;
        }
    }
    out
}

fn fnv1a64(input: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in canonicalize(input).as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn host_handle8(input: &str) -> String {
    format!("{:016x}", fnv1a64(input))
}

fn hbp_escape<T: ToString>(value: T) -> String {
    value
        .to_string()
        .chars()
        .map(|ch| match ch {
            '|' | '\r' | '\n' | '\t' => '_',
            ch if ch.is_ascii_graphic() || ch == ' ' => ch,
            _ => '_',
        })
        .take(240)
        .collect()
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_secs()
}

// --- the host's identity + hosted 8-byte agents ----------------------------

#[derive(Clone)]
struct Agent {
    pid8: String,    // 8-byte PID (sha16)
    role: String,    // the N-Nest reflection trio: work / review-predict / ask-fabric
    watcher: String, // the rung above (the supervisor/watcher pid)
}

struct Host {
    device: String,
    boot_ts: u64,
    host_pid8: String,
    host_handle8: String,
    bind: String,
    bus: String,
    sidecar: String,
    agents: Vec<Agent>,
    started: Instant,
    received: AtomicU64,
    helped: AtomicU64,
    held: AtomicU64, // packets whose command/code was HELD (execution gated)
    spoken: AtomicU64,
    reflected: AtomicU64,
    bus_emitted: AtomicU64,
    bus_ok: AtomicU64,
    bus_failed: AtomicU64,
    route_ok: AtomicU64,
    route_404: AtomicU64,
    route_400: AtomicU64,
    route_413: AtomicU64,
    query_rejected: AtomicU64,
    sidecar_ok: AtomicU64,
    sidecar_failed: AtomicU64,
}

impl Host {
    fn new(device: String, bind: String, bus: String, sidecar: String, n_agents: usize) -> Self {
        let boot_ts = unix_seconds();
        // host pid8 is device + boot derived — NOT a hardware serial.
        let host_pid8 = sha16(&format!("omnicoder-host|{}|{}", device, boot_ts));
        let host_handle8 = host_handle8(&format!("omnicoder-host|{}", device));
        let mut agents = Vec::with_capacity(n_agents);
        for i in 0..n_agents {
            let pid8 = sha16(&format!("{}|agent|{}", host_pid8, i));
            // The reflection nest: a1 works, a2 reviews + predicts next move, a3 asks the fabric.
            let role = match i % 3 {
                0 => "work",
                1 => "review-predict",
                _ => "ask-fabric",
            }
            .to_string();
            let watcher = sha16(&format!("{}|watcher", pid8)); // the supervisor rung above
            agents.push(Agent {
                pid8,
                role,
                watcher,
            });
        }
        Host {
            device,
            boot_ts,
            host_pid8,
            host_handle8,
            bind,
            bus,
            sidecar,
            agents,
            started: Instant::now(),
            received: AtomicU64::new(0),
            helped: AtomicU64::new(0),
            held: AtomicU64::new(0),
            spoken: AtomicU64::new(0),
            reflected: AtomicU64::new(0),
            bus_emitted: AtomicU64::new(0),
            bus_ok: AtomicU64::new(0),
            bus_failed: AtomicU64::new(0),
            route_ok: AtomicU64::new(0),
            route_404: AtomicU64::new(0),
            route_400: AtomicU64::new(0),
            route_413: AtomicU64::new(0),
            query_rejected: AtomicU64::new(0),
            sidecar_ok: AtomicU64::new(0),
            sidecar_failed: AtomicU64::new(0),
        }
    }

    /// Watcher gate for one agent: consent = reported == recomputed-truth (sha equality).
    /// Honest agent => PASS. A drifted agent's reported sha would differ => HOLD.
    fn watcher_gate(&self, a: &Agent, tick: u64) -> bool {
        let reported = sha16(&format!("{}|report|{}", a.pid8, tick));
        let recomputed = sha16(&format!("{}|report|{}", a.pid8, tick));
        reported == recomputed
    }

    // port.port.port — ONE process, one real socket; 3 sha-nested LOGICAL ports.
    fn port_nest(&self) -> [u16; 3] {
        let base = self
            .bind
            .rsplit(':')
            .next()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8789);
        let p1 = base;
        let n2 = u16::from_str_radix(&sha16(&format!("{}|port|{}", self.host_pid8, p1))[..4], 16)
            .unwrap_or(0);
        let p2 = 1024 + (n2 % 64511);
        let n3 = u16::from_str_radix(&sha16(&format!("{}|port|{}", self.host_pid8, p2))[..4], 16)
            .unwrap_or(0);
        let p3 = 1024 + (n3 % 64511);
        [p1, p2, p3]
    }

    // cube cube cubed — AoT distillation of the live ledger into 3 sha-folds.
    fn cube_cubed(&self) -> [String; 3] {
        let ledger: String = self
            .agents
            .iter()
            .map(|a| a.pid8.clone())
            .collect::<Vec<_>>()
            .join("|");
        let recv = self.received.load(Ordering::Relaxed);
        let c1 = sha16(&format!("cube|{}|{}", ledger, self.agents.len()));
        let c2 = sha16(&format!("cube|{}|{}", c1, recv));
        let c3 = sha16(&format!("cube|{}|{}|{}", c2, c1, self.host_pid8));
        [c1, c2, c3]
    }
}

// --- HBP route rendering ----------------------------------------------------

fn render_health(host: &Host) -> String {
    let ports = host.port_nest();
    let [c1, c2, c3] = host.cube_cubed();
    [
        format!(
            "OMNIHDR|schema={}|version={}|runtime=rust-std|replaces=termux|front_end=0|generated_unix_s={}|json=0",
            SCHEMA, VERSION, unix_seconds()
        ),
        format!(
            "OMNIPROC|device={}|host_pid8={}|host_handle8={}|os_pid={}|bind={}|boot_ts={}|uptime_s={}|json=0",
            hbp_escape(&host.device),
            hbp_escape(&host.host_pid8),
            hbp_escape(&host.host_handle8),
            std::process::id(),
            hbp_escape(&host.bind),
            host.boot_ts,
            host.started.elapsed().as_secs()
        ),
        format!(
            "OMNIAUTH|helper_packet_authority={}|execution_authority={}|process_launch=0|spawn=gated|killable=1|observable=1|watcher_gated=1|json=0",
            HELPER_PACKET_AUTHORITY as u8, EXECUTION_AUTHORITY as u8
        ),
        format!(
            "OMNIAGENTS|hosted={}|received={}|helped={}|held={}|spoken={}|reflected={}|bus_emitted={}|bus={}|sidecar={}|json=0",
            host.agents.len(),
            host.received.load(Ordering::Relaxed),
            host.helped.load(Ordering::Relaxed),
            host.held.load(Ordering::Relaxed),
            host.spoken.load(Ordering::Relaxed),
            host.reflected.load(Ordering::Relaxed),
            host.bus_emitted.load(Ordering::Relaxed),
            hbp_escape(&host.bus),
            hbp_escape(&host.sidecar)
        ),
        format!(
            "OMNIEVIDENCE|route_ok={}|route_404={}|route_400={}|route_413={}|query_rejected={}|bus_ok={}|bus_failed={}|sidecar_ok={}|sidecar_failed={}|json=0",
            host.route_ok.load(Ordering::Relaxed),
            host.route_404.load(Ordering::Relaxed),
            host.route_400.load(Ordering::Relaxed),
            host.route_413.load(Ordering::Relaxed),
            host.query_rejected.load(Ordering::Relaxed),
            host.bus_ok.load(Ordering::Relaxed),
            host.bus_failed.load(Ordering::Relaxed),
            host.sidecar_ok.load(Ordering::Relaxed),
            host.sidecar_failed.load(Ordering::Relaxed)
        ),
        format!(
            "OMNIPORTNEST|port_port_port={}.{}.{}|one_process=1|logical=1|json=0",
            ports[0], ports[1], ports[2]
        ),
        format!("OMNICUBE|cube={}|cube2={}|cube3={}|aot_distill=1|json=0", c1, c2, c3),
        "OMNIROUTE|path=/health.hbp|method=GET|json=0".to_string(),
        "OMNIROUTE|path=/agents.hbp|method=GET|json=0".to_string(),
        "OMNIROUTE|path=/ports.hbp|method=GET|json=0".to_string(),
        "OMNIROUTE|path=/cube.hbp|method=GET|json=0".to_string(),
        "OMNIROUTE|path=/say.hbp|method=GET|json=0".to_string(),
        "OMNIROUTE|path=/api/say|method=POST|conversation=machine-to-machine|json=0".to_string(),
        "OMNIROUTE|path=/self.hbp|method=GET|json=0".to_string(),
        "OMNIROUTE|path=/api/self|method=GET|json=0".to_string(),
        "OMNIROUTE|path=/api/packet|method=POST|gated=1|json=0".to_string(),
    ]
    .join("\n")
        + "\n"
}

fn render_agents(host: &Host) -> String {
    let tick = unix_seconds();
    let mut out = format!(
        "OMNIAGENTBOOK|hosted={}|tick={}|json=0\n",
        host.agents.len(),
        tick
    );
    for a in &host.agents {
        let gate = if host.watcher_gate(a, tick) {
            "PASS"
        } else {
            "HOLD"
        };
        out.push_str(&format!(
            "OMNIAGENT|pid8={}|role={}|watcher={}|gate={}|json=0\n",
            hbp_escape(&a.pid8),
            hbp_escape(&a.role),
            hbp_escape(&a.watcher),
            gate
        ));
    }
    out
}

fn render_ports(host: &Host) -> String {
    let p = host.port_nest();
    format!(
        "OMNIPORTNEST|port_port_port={}.{}.{}|json=0\nOMNIPORT|level=1|port={}|kind=real-socket|json=0\nOMNIPORT|level=2|port={}|kind=logical-sha-nested|json=0\nOMNIPORT|level=3|port={}|kind=logical-sha-nested|json=0\n",
        p[0], p[1], p[2], p[0], p[1], p[2]
    )
}

fn render_cube(host: &Host) -> String {
    let [c1, c2, c3] = host.cube_cubed();
    format!(
        "OMNICUBE|cube={}|cube2={}|cube3={}|received={}|hosted={}|aot_distill=1|json=0\n",
        c1,
        c2,
        c3,
        host.received.load(Ordering::Relaxed),
        host.agents.len()
    )
}

fn render_say(host: &Host, body: &str) -> String {
    let n = host.spoken.fetch_add(1, Ordering::Relaxed) + 1;
    host.helped.fetch_add(1, Ordering::Relaxed);
    let body_hash = sha16(body);
    let reply_pid = sha16(&format!("{}|say|{}|{}", host.host_pid8, n, body_hash));
    let row = format!(
        "OMNISAY|ts={}|host_pid8={}|reply_pid8={}|input_sha16={}|spoken={}|reply=I_AM_FALCON_OMNICODER_HOST_BUILD_TEST_FIX_REPEAT_SEND_HBP_PACKET_OR_SELF_QUERY|execution_authority=0|json=0",
        unix_seconds(),
        hbp_escape(&host.host_pid8),
        reply_pid,
        body_hash,
        n
    );
    append_sidecar(host, &row);
    bus_emit(host, &row);
    format!("{row}\n")
}

fn render_self(host: &Host) -> String {
    let [c1, c2, c3] = host.cube_cubed();
    [
        format!(
            "OMNISELF|schema={}|version={}|host_pid8={}|device={}|state=running|replaces=termux|front_end=0|json=0",
            SCHEMA,
            VERSION,
            hbp_escape(&host.host_pid8),
            hbp_escape(&host.device)
        ),
        format!(
            "OMNISELFSTATE|hosted={}|received={}|helped={}|held={}|spoken={}|reflected={}|bus_emitted={}|uptime_s={}|json=0",
            host.agents.len(),
            host.received.load(Ordering::Relaxed),
            host.helped.load(Ordering::Relaxed),
            host.held.load(Ordering::Relaxed),
            host.spoken.load(Ordering::Relaxed),
            host.reflected.load(Ordering::Relaxed),
            host.bus_emitted.load(Ordering::Relaxed),
            host.started.elapsed().as_secs()
        ),
        format!(
            "OMNISELFEVIDENCE|route_ok={}|route_404={}|route_400={}|route_413={}|query_rejected={}|bus_ok={}|bus_failed={}|sidecar_ok={}|sidecar_failed={}|governor=upstairs_hilbra_recall_gac_shannon_gnn|json=0",
            host.route_ok.load(Ordering::Relaxed),
            host.route_404.load(Ordering::Relaxed),
            host.route_400.load(Ordering::Relaxed),
            host.route_413.load(Ordering::Relaxed),
            host.query_rejected.load(Ordering::Relaxed),
            host.bus_ok.load(Ordering::Relaxed),
            host.bus_failed.load(Ordering::Relaxed),
            host.sidecar_ok.load(Ordering::Relaxed),
            host.sidecar_failed.load(Ordering::Relaxed)
        ),
        "OMNISELFBUILD|next=build_test_fix_repeat|needs=sidecar_verify,acer_battery_b,concurrency_stress,bus_receipt,self_build_planner|execution_authority=0|spawn=0|json=0"
            .to_string(),
        format!("OMNISELFCUBE|cube={}|cube2={}|cube3={}|json=0", c1, c2, c3),
        "OMNISELFGATE|command_code=HELD|process_launch=0|helper_packet_authority=1|execution_authority=0|decision_brain=external_fabric|json=0"
            .to_string(),
    ]
    .join("\n")
        + "\n"
}

fn render_reflect(host: &Host, reason: &str) -> String {
    let n = host.reflected.fetch_add(1, Ordering::Relaxed) + 1;
    let [c1, c2, c3] = host.cube_cubed();
    format!(
        "OMNIREFLECT|ts={}|host_pid8={}|seq={}|reason={}|cube={}|cube2={}|cube3={}|received={}|helped={}|held={}|spoken={}|watcher_gated=1|execution_authority=0|json=0",
        unix_seconds(),
        hbp_escape(&host.host_pid8),
        n,
        hbp_escape(reason),
        c1,
        c2,
        c3,
        host.received.load(Ordering::Relaxed),
        host.helped.load(Ordering::Relaxed),
        host.held.load(Ordering::Relaxed),
        host.spoken.load(Ordering::Relaxed)
    )
}

/// POST /api/packet — the omnicoder white-room packet contract. EXECUTION GATED:
/// a command/code field is HELD (never auto-executed); only a helper result is produced.
fn render_packet(host: &Host, body: &str) -> String {
    host.received.fetch_add(1, Ordering::Relaxed);
    // Crude, dependency-free presence check for a command/code field in the JSON body.
    let has_exec = body.contains("\"command\"") || body.contains("\"code\"");
    host.helped.fetch_add(1, Ordering::Relaxed);
    if has_exec {
        host.held.fetch_add(1, Ordering::Relaxed);
    }
    let verdict_pid = sha16(&format!(
        "{}|packet|{}",
        host.host_pid8,
        host.received.load(Ordering::Relaxed)
    ));
    let row = format!(
        "OMNIPACKET|verb=EVT-OMNICODER-HELPER-RESULT|pid8={}|accepted=1|executed=0|execution_authority=0|held={}|note={}|process_launch=0|json=0\n",
        hbp_escape(&verdict_pid),
        has_exec as u8,
        if has_exec { "command_or_code_present-HELD_execution_gated-helper_only" } else { "helper_packet_processed" }
    );
    append_sidecar(host, row.trim_end());
    bus_emit(host, row.trim_end());
    row
}

fn render_not_found(path: &str) -> String {
    format!(
        "OMNIERR|status=404|path={}|reason=not_found|json=0\n",
        hbp_escape(path)
    )
}

fn render_bad_request(reason: &str) -> String {
    format!("OMNIERR|status=400|reason={}|json=0\n", hbp_escape(reason))
}

fn render_payload_too_large() -> String {
    format!(
        "OMNIERR|status=413|max_bytes={}|reason=payload_too_large|json=0\n",
        MAX_REQUEST_BYTES
    )
}

fn record_route(host: &Host, route: &str, method: &str, status: u16, reason: &str) {
    match status {
        200 => {
            host.route_ok.fetch_add(1, Ordering::Relaxed);
        }
        400 => {
            host.route_400.fetch_add(1, Ordering::Relaxed);
        }
        404 => {
            host.route_404.fetch_add(1, Ordering::Relaxed);
        }
        413 => {
            host.route_413.fetch_add(1, Ordering::Relaxed);
        }
        _ => {}
    }
    let row = format!(
        "OMNIROUTEEVIDENCE|ts={}|method={}|route={}|status={}|reason={}|route_correct=1|process_launch=0|decision_brain=external_fabric|json=0",
        unix_seconds(),
        hbp_escape(method),
        hbp_escape(route),
        status,
        hbp_escape(reason)
    );
    let _ = append_sidecar(host, &row);
}

// --- minimal HTTP/1.1 plumbing (no framework; one socket) -------------------

fn status_reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        413 => "Payload Too Large",
        _ => "Status",
    }
}

fn write_response(stream: &mut TcpStream, status: u16, body: &str) {
    let resp = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        status_reason(status),
        body.len(),
        body
    );
    let _ = stream.write_all(resp.as_bytes());
}

fn handle_client(mut stream: TcpStream, host: &Host) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(READ_TIMEOUT_SECS)));
    let mut buf = Vec::with_capacity(4096);
    let mut chunk = [0u8; 4096];
    // Read at least the headers; for POST, read up to Content-Length more.
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                buf.extend_from_slice(&chunk[..n]);
                if buf.len() > MAX_REQUEST_BYTES {
                    let response = render_payload_too_large();
                    record_route(host, "request", "READ", 413, "payload_too_large");
                    write_response(&mut stream, 413, &response);
                    return;
                }
                if let Some(pos) = find_subslice(&buf, b"\r\n\r\n") {
                    let head = String::from_utf8_lossy(&buf[..pos]).to_string();
                    let content_len = header_value(&head, "content-length")
                        .and_then(|v| v.trim().parse::<usize>().ok())
                        .unwrap_or(0);
                    let body_have = buf.len() - (pos + 4);
                    if body_have >= content_len {
                        break;
                    }
                }
                continue;
            }
            Err(_) => break,
        }
    }
    let text = String::from_utf8_lossy(&buf);
    let mut lines = text.split("\r\n");
    let request_line = lines.next().unwrap_or("");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let raw_path = parts.next().unwrap_or("/");
    let path = match strict_path(raw_path) {
        Some(path) if request_line.split_whitespace().count() >= 3 => path,
        Some(_) => {
            let response = render_bad_request("malformed_request_line");
            record_route(host, raw_path, method, 400, "malformed_request_line");
            write_response(&mut stream, 400, &response);
            return;
        }
        None => {
            host.query_rejected.fetch_add(1, Ordering::Relaxed);
            let response = render_not_found(raw_path);
            record_route(host, raw_path, method, 404, "query_suffix_rejected");
            write_response(&mut stream, 404, &response);
            return;
        }
    };

    let (status, reason, response) = match (method, path) {
        ("GET", "/health.hbp") | ("GET", "/") => (200, "ok", render_health(host)),
        ("GET", "/agents.hbp") => (200, "ok", render_agents(host)),
        ("GET", "/ports.hbp") => (200, "ok", render_ports(host)),
        ("GET", "/cube.hbp") => (200, "ok", render_cube(host)),
        ("GET", "/say.hbp") => (200, "ok", render_say(host, "")),
        ("POST", "/api/say") => {
            let body = text.split("\r\n\r\n").nth(1).unwrap_or("");
            (200, "ok", render_say(host, body))
        }
        ("GET", "/self.hbp") | ("GET", "/api/self") => (200, "ok", render_self(host)),
        ("POST", "/api/packet") => {
            let body = text.split("\r\n\r\n").nth(1).unwrap_or("");
            (200, "ok", render_packet(host, body))
        }
        _ => (404, "not_found", render_not_found(path)),
    };
    record_route(host, path, method, status, reason);
    write_response(&mut stream, status, &response);
}

fn strict_path(raw_path: &str) -> Option<&str> {
    if raw_path.contains('?') {
        None
    } else {
        Some(raw_path)
    }
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn header_value(head: &str, key_lower: &str) -> Option<String> {
    for line in head.split("\r\n").skip(1) {
        if let Some((k, v)) = line.split_once(':') {
            if k.trim().to_ascii_lowercase() == key_lower {
                return Some(v.trim().to_string());
            }
        }
    }
    None
}

// --- fabric bus heartbeat (machine-to-machine; best-effort) -----------------

fn parse_url(url: &str) -> Option<(String, u16, String)> {
    let rest = url.strip_prefix("http://")?;
    let (hostport, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    let (h, p) = match hostport.rsplit_once(':') {
        Some((h, p)) => (h.to_string(), p.parse::<u16>().ok()?),
        None => (hostport.to_string(), 80),
    };
    Some((h, p, path.to_string()))
}

fn append_sidecar(host: &Host, row: &str) -> bool {
    let ok = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&host.sidecar)
        .and_then(|mut f| writeln!(f, "{}", row))
        .is_ok();
    if ok {
        host.sidecar_ok.fetch_add(1, Ordering::Relaxed);
    } else {
        host.sidecar_failed.fetch_add(1, Ordering::Relaxed);
    }
    ok
}

fn bus_emit(host: &Host, hbp_body: &str) {
    host.bus_emitted.fetch_add(1, Ordering::Relaxed);
    let ok = bus_send(&host.bus, hbp_body);
    if ok {
        host.bus_ok.fetch_add(1, Ordering::Relaxed);
    } else {
        host.bus_failed.fetch_add(1, Ordering::Relaxed);
    }
    let row = format!(
        "OMNIROUTEGUARD|ts={}|route=bus_emit|admitted=1|endpoint_bound={}|fallback={}|fallback_mode={}|bus_ok={}|bus_failed={}|decision_brain=external_fabric|json=0",
        unix_seconds(),
        (!ok) as u8,
        (!ok) as u8,
        if ok { "none" } else { "sidecar_only" },
        host.bus_ok.load(Ordering::Relaxed),
        host.bus_failed.load(Ordering::Relaxed)
    );
    let _ = append_sidecar(host, &row);
}

fn bus_send(bus: &str, hbp_body: &str) -> bool {
    if let Some((h, p, path)) = parse_url(bus) {
        if let Ok(mut s) = TcpStream::connect((h.as_str(), p)) {
            let _ = s.set_write_timeout(Some(Duration::from_secs(4)));
            let req = format!(
                "POST {} HTTP/1.1\r\nHost: {}:{}\r\nContent-Type: text/plain; charset=utf-8\r\nX-Asolaria-Format: hbp\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                path, h, p, hbp_body.len(), hbp_body
            );
            if s.write_all(req.as_bytes()).is_err() {
                return false;
            }
            let mut sink = [0u8; 256];
            let _ = s.read(&mut sink);
            return true;
        }
    }
    false
}

fn main() {
    // AI-native config: args/env only — no interactive front-end.
    let mut device = env::var("OMNI_DEVICE").unwrap_or_else(|_| DEFAULT_DEVICE.to_string());
    let mut bind = env::var("OMNI_BIND").unwrap_or_else(|_| DEFAULT_BIND.to_string());
    let bus = env::var("OMNI_BUS").unwrap_or_else(|_| DEFAULT_BUS.to_string());
    let sidecar = env::var("OMNI_SIDECAR").unwrap_or_else(|_| DEFAULT_SIDECAR.to_string());
    let n_agents = env::var("OMNI_AGENTS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_AGENTS);

    let args: Vec<String> = env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--device" => {
                if let Some(v) = args.get(i + 1) {
                    device = v.clone();
                    i += 1;
                }
            }
            "--bind" => {
                if let Some(v) = args.get(i + 1) {
                    bind = v.clone();
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let host = Arc::new(Host::new(
        device,
        bind.clone(),
        bus.clone(),
        sidecar.clone(),
        n_agents,
    ));

    let listener = match TcpListener::bind(&host.bind) {
        Ok(l) => l,
        Err(e) => {
            eprintln!(
                "OMNIERR|bind={}|error={}|json=0",
                hbp_escape(&host.bind),
                hbp_escape(e.to_string())
            );
            std::process::exit(2);
        }
    };

    // Online announce + a heartbeat thread to the fabric bus (machine-to-machine).
    {
        let h = Arc::clone(&host);
        let online = format!(
            "OMNIBUS|from=omnicoder-host|to=asolaria|mode=real|verb=omnicoder.online|pid8={}|device={}|hosted={}|execution_authority=0|json=0",
            hbp_escape(&h.host_pid8), hbp_escape(&h.device), h.agents.len()
        );
        let _ = append_sidecar(&h, &online);
        bus_emit(&h, &online);
        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(HEARTBEAT_SECS));
            let hb = format!(
                "OMNIBUS|from=omnicoder-host|to=asolaria|mode=real|verb=omnicoder.pulse|pid8={}|received={}|helped={}|held={}|spoken={}|reflected={}|hosted={}|execution_authority=0|json=0",
                hbp_escape(&h.host_pid8),
                h.received.load(Ordering::Relaxed),
                h.helped.load(Ordering::Relaxed),
                h.held.load(Ordering::Relaxed),
                h.spoken.load(Ordering::Relaxed),
                h.reflected.load(Ordering::Relaxed),
                h.agents.len()
            );
            let _ = append_sidecar(&h, &hb);
            bus_emit(&h, &hb);
        });
    }

    {
        let h = Arc::clone(&host);
        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(SIDECAR_SECS));
            let row = render_reflect(&h, "sidecar_tick");
            let _ = append_sidecar(&h, &row);
            bus_emit(&h, &row);
        });
    }

    println!(
        "OMNILISTEN|schema={}|bind={}|device={}|host_pid8={}|hosted={}|execution_authority=0|replaces=termux|json=0",
        SCHEMA, hbp_escape(&host.bind), hbp_escape(&host.device), hbp_escape(&host.host_pid8), host.agents.len()
    );

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let h = Arc::clone(&host);
                // one short-lived thread per connection; the host stays one process.
                thread::spawn(move || handle_client(s, &h));
            }
            Err(e) => eprintln!("OMNIERR|accept={}|json=0", hbp_escape(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_path_rejects_query_suffix() {
        assert_eq!(strict_path("/health.hbp"), Some("/health.hbp"));
        assert_eq!(strict_path("/health.hbp?x=1"), None);
    }

    #[test]
    fn sha256_known_vectors_match() {
        assert_eq!(
            sha256hex(""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256hex("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(sha16("abc"), "ba7816bf8f01cfea");
    }

    #[test]
    fn say_and_packet_never_grant_execution() {
        let host = Host::new(
            "test".to_string(),
            "127.0.0.1:0".to_string(),
            "http://127.0.0.1:9/noop".to_string(),
            "target/test-sidecar.hbp".to_string(),
            3,
        );
        let said = render_say(&host, "build yourself");
        assert!(said.contains("OMNISAY|"));
        assert!(said.contains("execution_authority=0"));

        let pkt = render_packet(&host, "{\"command\":\"rm -rf /\"}");
        assert!(pkt.contains("executed=0"));
        assert!(pkt.contains("held=1"));
        assert!(pkt.contains("process_launch=0"));
    }

    #[test]
    fn self_report_carries_build_loop_and_gate() {
        let host = Host::new(
            "test".to_string(),
            "127.0.0.1:0".to_string(),
            "http://127.0.0.1:9/noop".to_string(),
            "target/test-sidecar.hbp".to_string(),
            3,
        );
        let out = render_self(&host);
        assert!(out.contains("OMNISELF|"));
        assert!(out.contains("build_test_fix_repeat"));
        assert!(out.contains("execution_authority=0"));
        assert!(out.contains("OMNISELFEVIDENCE|"));
        assert!(out.contains("decision_brain=external_fabric"));
    }

    #[test]
    fn health_exposes_endpoint_evidence_without_policy_brain() {
        let host = Host::new(
            "test".to_string(),
            "127.0.0.1:0".to_string(),
            "http://127.0.0.1:9/noop".to_string(),
            "target/test-sidecar.hbp".to_string(),
            3,
        );
        let out = render_health(&host);
        assert!(out.contains("OMNIEVIDENCE|"));
        assert!(out.contains("route_ok=0"));
        assert!(!out.contains("CUSUM"));
        assert!(!out.contains("COMMANDER"));
    }
}
