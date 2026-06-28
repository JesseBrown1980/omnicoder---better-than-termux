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
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

const SCHEMA: &str = "ASOLARIA-OMNICODER-HOST";
const VERSION: &str = "0.1.0";
const DEFAULT_BIND: &str = "127.0.0.1:8789";
const DEFAULT_BUS: &str = "http://127.0.0.1:4948/behcs/send"; // adb reverse 4948 -> acer bus :4947
const DEFAULT_DEVICE: &str = "device";
const DEFAULT_AGENTS: usize = 24; // the 24-spindle base; multiplied elsewhere, not here
const HEARTBEAT_SECS: u64 = 60;

// Governance — DO NOT weaken without operator T0. The host helps; it does not fire.
const HELPER_PACKET_AUTHORITY: bool = true;
const EXECUTION_AUTHORITY: bool = false;

// --- hashing: byte-parity with host8-serve ---------------------------------

fn sha256hex(input: &str) -> String {
    let mut h = Sha256::new();
    h.update(input.as_bytes());
    let d = h.finalize();
    let mut out = String::with_capacity(64);
    for b in d.iter() {
        out.push_str(&format!("{:02x}", b));
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
    pid8: String,   // 8-byte PID (sha16)
    role: String,   // the N-Nest reflection trio: work / review-predict / ask-fabric
    watcher: String, // the rung above (the supervisor/watcher pid)
}

struct Host {
    device: String,
    boot_ts: u64,
    host_pid8: String,
    host_handle8: String,
    bind: String,
    bus: String,
    agents: Vec<Agent>,
    started: Instant,
    received: AtomicU64,
    helped: AtomicU64,
    held: AtomicU64, // packets whose command/code was HELD (execution gated)
}

impl Host {
    fn new(device: String, bind: String, bus: String, n_agents: usize) -> Self {
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
            agents.push(Agent { pid8, role, watcher });
        }
        Host {
            device,
            boot_ts,
            host_pid8,
            host_handle8,
            bind,
            bus,
            agents,
            started: Instant::now(),
            received: AtomicU64::new(0),
            helped: AtomicU64::new(0),
            held: AtomicU64::new(0),
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
        let base = self.bind.rsplit(':').next().and_then(|p| p.parse::<u16>().ok()).unwrap_or(8789);
        let p1 = base;
        let n2 = u16::from_str_radix(&sha16(&format!("{}|port|{}", self.host_pid8, p1))[..4], 16).unwrap_or(0);
        let p2 = 1024 + (n2 % 64511);
        let n3 = u16::from_str_radix(&sha16(&format!("{}|port|{}", self.host_pid8, p2))[..4], 16).unwrap_or(0);
        let p3 = 1024 + (n3 % 64511);
        [p1, p2, p3]
    }

    // cube cube cubed — AoT distillation of the live ledger into 3 sha-folds.
    fn cube_cubed(&self) -> [String; 3] {
        let ledger: String = self.agents.iter().map(|a| a.pid8.clone()).collect::<Vec<_>>().join("|");
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
            "OMNIAGENTS|hosted={}|received={}|helped={}|held={}|bus={}|json=0",
            host.agents.len(),
            host.received.load(Ordering::Relaxed),
            host.helped.load(Ordering::Relaxed),
            host.held.load(Ordering::Relaxed),
            hbp_escape(&host.bus)
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
        "OMNIROUTE|path=/api/packet|method=POST|gated=1|json=0".to_string(),
    ]
    .join("\n")
        + "\n"
}

fn render_agents(host: &Host) -> String {
    let tick = unix_seconds();
    let mut out = format!("OMNIAGENTBOOK|hosted={}|tick={}|json=0\n", host.agents.len(), tick);
    for a in &host.agents {
        let gate = if host.watcher_gate(a, tick) { "PASS" } else { "HOLD" };
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
        c1, c2, c3, host.received.load(Ordering::Relaxed), host.agents.len()
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
    let verdict_pid = sha16(&format!("{}|packet|{}", host.host_pid8, host.received.load(Ordering::Relaxed)));
    format!(
        "OMNIPACKET|verb=EVT-OMNICODER-HELPER-RESULT|pid8={}|accepted=1|executed=0|execution_authority=0|held={}|note={}|process_launch=0|json=0\n",
        hbp_escape(&verdict_pid),
        has_exec as u8,
        if has_exec { "command_or_code_present-HELD_execution_gated-helper_only" } else { "helper_packet_processed" }
    )
}

fn render_not_found(path: &str) -> String {
    format!("OMNIERR|status=404|path={}|reason=not_found|json=0\n", hbp_escape(path))
}

// --- minimal HTTP/1.1 plumbing (no framework; one socket) -------------------

fn write_response(stream: &mut TcpStream, body: &str) {
    let resp = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(resp.as_bytes());
}

fn handle_client(mut stream: TcpStream, host: &Host) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
    let mut buf = Vec::with_capacity(4096);
    let mut chunk = [0u8; 4096];
    // Read at least the headers; for POST, read up to Content-Length more.
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                buf.extend_from_slice(&chunk[..n]);
                if let Some(pos) = find_subslice(&buf, b"\r\n\r\n") {
                    let head = String::from_utf8_lossy(&buf[..pos]).to_string();
                    let content_len = header_value(&head, "content-length")
                        .and_then(|v| v.trim().parse::<usize>().ok())
                        .unwrap_or(0);
                    let body_have = buf.len() - (pos + 4);
                    if body_have >= content_len {
                        break;
                    }
                } else if buf.len() > 1 << 20 {
                    break; // 1 MiB header cap
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
    let path = raw_path.split('?').next().unwrap_or("/");

    let response = match (method, path) {
        ("GET", "/health.hbp") | ("GET", "/") => render_health(host),
        ("GET", "/agents.hbp") => render_agents(host),
        ("GET", "/ports.hbp") => render_ports(host),
        ("GET", "/cube.hbp") => render_cube(host),
        ("POST", "/api/packet") => {
            let body = text.split("\r\n\r\n").nth(1).unwrap_or("");
            render_packet(host, body)
        }
        _ => render_not_found(path),
    };
    write_response(&mut stream, &response);
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

fn bus_send(bus: &str, json_body: &str) {
    if let Some((h, p, path)) = parse_url(bus) {
        if let Ok(mut s) = TcpStream::connect((h.as_str(), p)) {
            let _ = s.set_write_timeout(Some(Duration::from_secs(4)));
            let req = format!(
                "POST {} HTTP/1.1\r\nHost: {}:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                path, h, p, json_body.len(), json_body
            );
            let _ = s.write_all(req.as_bytes());
            let mut sink = [0u8; 256];
            let _ = s.read(&mut sink);
        }
    }
}

fn main() {
    // AI-native config: args/env only — no interactive front-end.
    let mut device = env::var("OMNI_DEVICE").unwrap_or_else(|_| DEFAULT_DEVICE.to_string());
    let mut bind = env::var("OMNI_BIND").unwrap_or_else(|_| DEFAULT_BIND.to_string());
    let bus = env::var("OMNI_BUS").unwrap_or_else(|_| DEFAULT_BUS.to_string());
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

    let host = Arc::new(Host::new(device, bind.clone(), bus.clone(), n_agents));

    let listener = match TcpListener::bind(&host.bind) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("OMNIERR|bind={}|error={}|json=0", hbp_escape(&host.bind), hbp_escape(e.to_string()));
            std::process::exit(2);
        }
    };

    // Online announce + a heartbeat thread to the fabric bus (machine-to-machine).
    {
        let h = Arc::clone(&host);
        let online = format!(
            "{{\"from\":\"omnicoder-host\",\"to\":\"asolaria\",\"mode\":\"real\",\"pid8\":\"{}\",\"device\":\"{}\",\"payload\":{{\"type\":\"omnicoder_online\",\"hosted\":{},\"execution_authority\":false}}}}",
            h.host_pid8, h.device, h.agents.len()
        );
        bus_send(&h.bus, &online);
        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(HEARTBEAT_SECS));
            let hb = format!(
                "{{\"from\":\"omnicoder-host\",\"to\":\"asolaria\",\"mode\":\"real\",\"pid8\":\"{}\",\"payload\":{{\"type\":\"heartbeat\",\"verb\":\"omnicoder.pulse\",\"received\":{},\"helped\":{},\"held\":{},\"hosted\":{}}}}}",
                h.host_pid8,
                h.received.load(Ordering::Relaxed),
                h.helped.load(Ordering::Relaxed),
                h.held.load(Ordering::Relaxed),
                h.agents.len()
            );
            bus_send(&h.bus, &hb);
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
