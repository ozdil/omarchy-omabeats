use crate::security::{atomic_write_secure, safe_read_file_limited, spawn_isolated};
use std::fs;
use std::path::{Path, PathBuf};
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::time::Duration;

/// Helper for secure isolated pactl execution with process_group(0)
fn secure_pactl_cmd() -> Command {
    let mut cmd = Command::new("/usr/bin/pactl");
    cmd.process_group(0);
    cmd.stdin(Stdio::null());
    cmd
}

#[derive(Debug, Clone)]
pub struct EqBand {
    pub filter_type: &'static str,
    pub freq: f32,
    pub q: f32,
    pub gain: f32,
}

pub struct EqProfile {
    pub name: &'static str,
    pub bands: [EqBand; 6],
}

pub const PROFILES: &[EqProfile] = &[
    EqProfile {
        name: "Beats Signature",
        bands: [
            EqBand { filter_type: "bq_lowshelf", freq: 80.0, q: 1.0, gain: 4.0 },
            EqBand { filter_type: "bq_peaking", freq: 250.0, q: 1.0, gain: 2.0 },
            EqBand { filter_type: "bq_peaking", freq: 1000.0, q: 1.0, gain: -2.0 },
            EqBand { filter_type: "bq_peaking", freq: 3000.0, q: 1.0, gain: 1.0 },
            EqBand { filter_type: "bq_peaking", freq: 6000.0, q: 1.0, gain: 3.0 },
            EqBand { filter_type: "bq_highshelf", freq: 10000.0, q: 1.0, gain: 2.5 },
        ],
    },
    EqProfile {
        name: "Bass Boost",
        bands: [
            EqBand { filter_type: "bq_lowshelf", freq: 70.0, q: 1.2, gain: 6.5 },
            EqBand { filter_type: "bq_peaking", freq: 160.0, q: 1.0, gain: 3.5 },
            EqBand { filter_type: "bq_peaking", freq: 600.0, q: 1.0, gain: -1.5 },
            EqBand { filter_type: "bq_peaking", freq: 2000.0, q: 1.0, gain: -2.0 },
            EqBand { filter_type: "bq_peaking", freq: 5000.0, q: 1.0, gain: -1.0 },
            EqBand { filter_type: "bq_highshelf", freq: 10000.0, q: 1.0, gain: -2.0 },
        ],
    },
    EqProfile {
        name: "Vocal Clarity",
        bands: [
            EqBand { filter_type: "bq_lowshelf", freq: 100.0, q: 0.9, gain: -4.0 },
            EqBand { filter_type: "bq_peaking", freq: 300.0, q: 1.0, gain: -2.0 },
            EqBand { filter_type: "bq_peaking", freq: 1200.0, q: 1.1, gain: 3.0 },
            EqBand { filter_type: "bq_peaking", freq: 3000.0, q: 1.2, gain: 5.0 },
            EqBand { filter_type: "bq_peaking", freq: 6000.0, q: 1.0, gain: 2.5 },
            EqBand { filter_type: "bq_highshelf", freq: 10000.0, q: 1.0, gain: 1.0 },
        ],
    },
    EqProfile {
        name: "Flat",
        bands: [
            EqBand { filter_type: "bq_lowshelf", freq: 80.0, q: 1.0, gain: 0.0 },
            EqBand { filter_type: "bq_peaking", freq: 250.0, q: 1.0, gain: 0.0 },
            EqBand { filter_type: "bq_peaking", freq: 1000.0, q: 1.0, gain: 0.0 },
            EqBand { filter_type: "bq_peaking", freq: 3000.0, q: 1.0, gain: 0.0 },
            EqBand { filter_type: "bq_peaking", freq: 6000.0, q: 1.0, gain: 0.0 },
            EqBand { filter_type: "bq_highshelf", freq: 10000.0, q: 1.0, gain: 0.0 },
        ],
    },
];

fn get_eq_dir() -> PathBuf {
    let base = std::env::var("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/home/ozdil".to_string());
            PathBuf::from(home).join(".local/state")
        });
    base.join("omarchy").join("omabeats_eq")
}

fn get_pid_file() -> PathBuf {
    get_eq_dir().join("filter_chain.pid")
}

pub fn stop_equalizer() {
    let pid_file = get_pid_file();
    if pid_file.exists() {
        if let Ok(content) = safe_read_file_limited(&pid_file) {
            if let Ok(pid) = content.trim().parse::<i32>() {
                if pid > 1 {
                    unsafe {
                        libc::kill(pid, libc::SIGTERM);
                    }
                }
            }
        }
        let _ = fs::remove_file(&pid_file);
    }

    // Also terminate any leftover omabeats pipewire filter-chain instances
    let mut cmd = Command::new("/usr/bin/pkill");
    cmd.process_group(0);
    cmd.args(["-f", "omabeats_eq/filter-chain.conf"]);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());
    let _ = cmd.status();
}

/// Dynamically locates the active Bluetooth Pulse/PipeWire sink for given MAC address
pub fn get_bluetooth_sink_name(mac_opt: Option<&str>) -> Option<String> {
    let output = secure_pactl_cmd()
        .args(["list", "short", "sinks"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);

    if let Some(mac) = mac_opt {
        let mac_clean = mac.replace(':', "_").to_lowercase();
        for line in text.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[1];
                if name.to_lowercase().contains(&mac_clean) {
                    return Some(name.to_string());
                }
            }
        }
    }

    // Fallback: any bluez_output sink
    for line in text.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[1].starts_with("bluez_output") {
            return Some(parts[1].to_string());
        }
    }

    None
}

/// Retrieves the current volume percentage of a sink (or default sink)
pub fn get_sink_volume(sink_name: &str) -> Option<i32> {
    let output = secure_pactl_cmd()
        .args(["get-sink-volume", sink_name])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    for word in text.split_whitespace() {
        if word.ends_with('%') {
            if let Ok(val) = word.trim_end_matches('%').parse::<i32>() {
                return Some(val);
            }
        }
    }
    None
}

/// Moves all active playing media streams (sink-inputs) to specified sink
pub fn move_all_sink_inputs_to(target_sink: &str) {
    if let Ok(output) = secure_pactl_cmd()
        .args(["list", "short", "sink-inputs"])
        .output()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if let Some(id) = line.split_whitespace().next() {
                let _ = secure_pactl_cmd()
                    .args(["move-sink-input", id, target_sink])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }
        }
    }
}

pub fn apply_profile(profile_name: &str, mac: Option<&str>) -> bool {
    let profile = PROFILES
        .iter()
        .find(|p| p.name.eq_ignore_ascii_case(profile_name))
        .or_else(|| {
            match profile_name.to_lowercase().as_str() {
                "bass" | "bass+" | "bas+" => PROFILES.iter().find(|p| p.name == "Bass Boost"),
                "vocal" | "vokal" => PROFILES.iter().find(|p| p.name == "Vocal Clarity"),
                "signature" | "imza" => PROFILES.iter().find(|p| p.name == "Beats Signature"),
                _ => None,
            }
        })
        .unwrap_or(&PROFILES[3]); // Default to Flat

    let bt_sink = get_bluetooth_sink_name(mac);

    // If Flat, FIRST safely migrate streams to the Bluetooth sink, then stop filter-chain
    if profile.name == "Flat" {
        if let Some(ref sink) = bt_sink {
            let cur_vol = get_sink_volume(sink)
                .or_else(|| get_sink_volume("@DEFAULT_SINK@"))
                .unwrap_or(50);
            let _ = secure_pactl_cmd()
                .args(["set-default-sink", sink])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            move_all_sink_inputs_to(sink);
            let _ = secure_pactl_cmd()
                .args(["set-sink-volume", sink, &format!("{}%", cur_vol)])
                .status();
            std::thread::sleep(Duration::from_millis(30));
        }
        stop_equalizer();
        return true;
    }

    // Beats profile requires the Bluetooth audio sink. Do NOT hijack laptop speakers!
    let target_sink = match bt_sink {
        Some(s) => s,
        None => {
            // Headset sink not ready in PipeWire; stop EQ and defer activation
            stop_equalizer();
            return true;
        }
    };

    let cur_vol = get_sink_volume(&target_sink)
        .or_else(|| get_sink_volume("@DEFAULT_SINK@"))
        .unwrap_or(50);

    // CRITICAL: Move playing media streams to target_sink BEFORE stopping the old filter-chain!
    // This prevents Spotify/players from receiving a broken pipe/kill signal and pausing playback.
    move_all_sink_inputs_to(&target_sink);
    std::thread::sleep(Duration::from_millis(30));

    stop_equalizer();

    let eq_dir = get_eq_dir();
    let conf_d = eq_dir.join("filter-chain.conf.d");
    if fs::create_dir_all(&conf_d).is_err() {
        return false;
    }

    // 1. Copy base filter-chain.conf
    let base_src = Path::new("/usr/share/pipewire/filter-chain.conf");
    let base_dst = eq_dir.join("filter-chain.conf");
    if base_src.exists() {
        let _ = fs::copy(base_src, &base_dst);
    }

    // 2. Generate filter-chain snippet
    let mut nodes_json = String::new();
    for (i, b) in profile.bands.iter().enumerate() {
        let node_str = format!(
            r#"                    {{
                        type  = builtin
                        name  = eq_band_{idx}
                        label = {lbl}
                        control = {{ "Freq" = {freq:.1} "Q" = {q:.2} "Gain" = {gain:.1} }}
                    }}
"#,
            idx = i + 1,
            lbl = b.filter_type,
            freq = b.freq,
            q = b.q,
            gain = b.gain
        );
        nodes_json.push_str(&node_str);
    }

    let target_obj_prop = format!("\n                target.object    = \"{}\"", target_sink);

    let filter_conf = format!(
        r#"context.modules = [
    {{ name = libpipewire-module-filter-chain
        args = {{
            node.description = "Beats {name} Equalizer"
            media.name       = "Beats {name} Equalizer"
            filter.graph = {{
                nodes = [
{nodes}                ]
                links = [
                    {{ output = "eq_band_1:Out" input = "eq_band_2:In" }}
                    {{ output = "eq_band_2:Out" input = "eq_band_3:In" }}
                    {{ output = "eq_band_3:Out" input = "eq_band_4:In" }}
                    {{ output = "eq_band_4:Out" input = "eq_band_5:In" }}
                    {{ output = "eq_band_5:Out" input = "eq_band_6:In" }}
                ]
            }}
            audio.channels = 2
            audio.position = [ FL FR ]
            capture.props = {{
                node.name        = "omabeats_eq"
                media.class      = Audio/Sink
                node.description = "Beats Studio Equalizer ({name})"
            }}
            playback.props = {{
                node.name        = "omabeats_eq_out"
                node.passive     = true{target_prop}
            }}
        }}
    }}
]
"#,
        name = profile.name,
        nodes = nodes_json,
        target_prop = target_obj_prop
    );

    let conf_path = conf_d.join("omabeats-eq.conf");
    if atomic_write_secure(&conf_path, &filter_conf).is_err() {
        return false;
    }

    // 3. Spawn pipewire filter-chain
    let mut cmd = Command::new("/usr/bin/pipewire");
    cmd.env("PIPEWIRE_CONFIG_DIR", &eq_dir);
    cmd.args(["-c", "filter-chain.conf"]);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());

    if let Ok(mut guard) = spawn_isolated(cmd) {
        if let Some(child) = guard.take() {
            let pid = child.id();
            let _ = atomic_write_secure(&get_pid_file(), &pid.to_string());
            std::mem::forget(child);
        }
    }

    // 4. Wait for omabeats_eq to appear in pactl sinks (up to 500ms)
    for _ in 0..10 {
        std::thread::sleep(Duration::from_millis(50));
        if let Ok(output) = secure_pactl_cmd().args(["list", "short", "sinks"]).output() {
            let text = String::from_utf8_lossy(&output.stdout);
            if text.contains("omabeats_eq") {
                break;
            }
        }
    }

    // 5. Synchronize volume BEFORE switching to prevent audio burst
    let _ = secure_pactl_cmd()
        .args(["set-sink-volume", "omabeats_eq", &format!("{}%", cur_vol)])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    // 6. Set omabeats_eq as default sink
    let _ = secure_pactl_cmd()
        .args(["set-default-sink", "omabeats_eq"])
        .status();

    // 7. Seamlessly move active playback streams to omabeats_eq
    move_all_sink_inputs_to("omabeats_eq");

    true
}
