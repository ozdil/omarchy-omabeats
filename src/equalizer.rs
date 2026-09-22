use crate::security::{atomic_write_secure, spawn_isolated};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

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
            EqBand { filter_type: "bq_lowshelf", freq: 80.0, q: 1.0, gain: 4.5 },
            EqBand { filter_type: "bq_peaking", freq: 250.0, q: 1.0, gain: 2.0 },
            EqBand { filter_type: "bq_peaking", freq: 1000.0, q: 1.0, gain: -2.0 },
            EqBand { filter_type: "bq_peaking", freq: 3000.0, q: 1.0, gain: 1.0 },
            EqBand { filter_type: "bq_peaking", freq: 6000.0, q: 1.0, gain: 3.5 },
            EqBand { filter_type: "bq_highshelf", freq: 10000.0, q: 1.0, gain: 2.0 },
        ],
    },
    EqProfile {
        name: "Bass Boost",
        bands: [
            EqBand { filter_type: "bq_lowshelf", freq: 80.0, q: 1.2, gain: 7.5 },
            EqBand { filter_type: "bq_peaking", freq: 200.0, q: 1.0, gain: 4.0 },
            EqBand { filter_type: "bq_peaking", freq: 800.0, q: 1.0, gain: -1.0 },
            EqBand { filter_type: "bq_peaking", freq: 2500.0, q: 1.0, gain: 0.0 },
            EqBand { filter_type: "bq_peaking", freq: 6000.0, q: 1.0, gain: 1.0 },
            EqBand { filter_type: "bq_highshelf", freq: 10000.0, q: 1.0, gain: -1.5 },
        ],
    },
    EqProfile {
        name: "Vocal Clarity",
        bands: [
            EqBand { filter_type: "bq_lowshelf", freq: 120.0, q: 0.9, gain: -3.0 },
            EqBand { filter_type: "bq_peaking", freq: 300.0, q: 1.0, gain: -1.0 },
            EqBand { filter_type: "bq_peaking", freq: 1200.0, q: 1.0, gain: 2.5 },
            EqBand { filter_type: "bq_peaking", freq: 2500.0, q: 1.2, gain: 5.5 },
            EqBand { filter_type: "bq_peaking", freq: 5000.0, q: 1.0, gain: 3.0 },
            EqBand { filter_type: "bq_highshelf", freq: 10000.0, q: 1.0, gain: 2.0 },
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
        if let Ok(content) = fs::read_to_string(&pid_file) {
            if let Ok(pid) = content.trim().parse::<i32>() {
                unsafe {
                    libc::kill(pid, libc::SIGTERM);
                }
            }
        }
        let _ = fs::remove_file(&pid_file);
    }

    // Also terminate any leftover omabeats pipewire filter-chain instances
    let _ = Command::new("/usr/bin/pkill")
        .args(["-f", "omabeats_eq/filter-chain.conf"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
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

    // If Flat, stop the filter chain to save CPU and route directly to device
    if profile.name == "Flat" {
        stop_equalizer();
        if let Some(m) = mac {
            let sink = format!("bluez_output.{}.1", m.replace(':', "_"));
            let _ = Command::new("/usr/bin/pactl")
                .args(["set-default-sink", &sink])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        return true;
    }

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
                node.passive     = true
            }}
        }}
    }}
]
"#,
        name = profile.name,
        nodes = nodes_json
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
            let _ = fs::write(get_pid_file(), pid.to_string());
            // Leak child ownership to let it run persistently in background
            std::mem::forget(child);
        }
    }

    // 4. Wait brief moment and set omabeats_eq as default sink
    std::thread::sleep(Duration::from_millis(150));
    let _ = Command::new("/usr/bin/pactl")
        .args(["set-default-sink", "omabeats_eq"])
        .stdin(Stdio::null())
        .status();

    true
}
