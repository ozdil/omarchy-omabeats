use crate::security::{atomic_write_secure, safe_read_file_limited, spawn_isolated};
use std::fs;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpatialMode {
    Off,
    CinemaDolby,
    MusicSpatial,
}

impl SpatialMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            SpatialMode::Off => "off",
            SpatialMode::CinemaDolby => "cinema",
            SpatialMode::MusicSpatial => "music",
        }
    }

    pub fn from_str_name(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "off" | "kapali" | "disable" | "none" => Some(SpatialMode::Off),
            "cinema" | "sinema" | "dolby" | "movie" | "atmos" => Some(SpatialMode::CinemaDolby),
            "music" | "muzik" | "wide" | "spatial" => Some(SpatialMode::MusicSpatial),
            _ => None,
        }
    }
}

/// Helper for secure isolated pactl execution with process_group(0)
fn secure_pactl_cmd() -> Command {
    let mut cmd = Command::new("/usr/bin/pactl");
    cmd.process_group(0);
    cmd.stdin(Stdio::null());
    cmd
}

fn get_spatial_dir() -> PathBuf {
    let base = std::env::var("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/home/ozdil".to_string());
            PathBuf::from(home).join(".local/state")
        });
    base.join("omarchy").join("omabeats_spatial")
}

fn get_pid_file() -> PathBuf {
    get_spatial_dir().join("spatial_filter_chain.pid")
}

pub fn stop_spatial() {
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

    let mut cmd = Command::new("/usr/bin/pkill");
    cmd.process_group(0);
    cmd.args(["-f", "omabeats_spatial/filter-chain.conf"]);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());
    let _ = cmd.status();
}

pub fn apply_spatial_mode(mode: SpatialMode, mac: Option<&str>) -> bool {
    let bt_sink = crate::equalizer::get_bluetooth_sink_name(mac);

    if mode == SpatialMode::Off {
        if let Some(ref sink) = bt_sink {
            let cur_vol = crate::equalizer::get_sink_volume(sink)
                .or_else(|| crate::equalizer::get_sink_volume("@DEFAULT_SINK@"))
                .unwrap_or(50);
            let _ = secure_pactl_cmd()
                .args(["set-default-sink", sink])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            crate::equalizer::move_all_sink_inputs_to(sink);
            let _ = secure_pactl_cmd()
                .args(["set-sink-volume", sink, &format!("{}%", cur_vol)])
                .status();
            std::thread::sleep(Duration::from_millis(30));
        }
        stop_spatial();
        return true;
    }

    let target_sink = match bt_sink {
        Some(s) => s,
        None => {
            stop_spatial();
            return true;
        }
    };

    let cur_vol = crate::equalizer::get_sink_volume(&target_sink)
        .or_else(|| crate::equalizer::get_sink_volume("@DEFAULT_SINK@"))
        .unwrap_or(50);

    crate::equalizer::move_all_sink_inputs_to(&target_sink);
    std::thread::sleep(Duration::from_millis(30));

    stop_spatial();

    let spatial_dir = get_spatial_dir();
    let conf_d = spatial_dir.join("filter-chain.conf.d");
    if fs::create_dir_all(&conf_d).is_err() {
        return false;
    }

    let base_src = Path::new("/usr/share/pipewire/filter-chain.conf");
    let base_dst = spatial_dir.join("filter-chain.conf");
    if base_src.exists() {
        let _ = fs::copy(base_src, &base_dst);
    }

    let filter_conf = match mode {
        SpatialMode::CinemaDolby => generate_cinema_dolby_config(&target_sink),
        SpatialMode::MusicSpatial => generate_music_spatial_config(&target_sink),
        SpatialMode::Off => return true,
    };

    let conf_path = conf_d.join("omabeats-spatial.conf");
    if atomic_write_secure(&conf_path, &filter_conf).is_err() {
        return false;
    }

    let mut cmd = Command::new("/usr/bin/pipewire");
    cmd.env("PIPEWIRE_CONFIG_DIR", &spatial_dir);
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

    for _ in 0..10 {
        std::thread::sleep(Duration::from_millis(50));
        if let Ok(output) = secure_pactl_cmd().args(["list", "short", "sinks"]).output() {
            let text = String::from_utf8_lossy(&output.stdout);
            if text.contains("omabeats_spatial") {
                break;
            }
        }
    }

    let _ = secure_pactl_cmd()
        .args(["set-sink-volume", "omabeats_spatial", &format!("{}%", cur_vol)])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    let _ = secure_pactl_cmd()
        .args(["set-default-sink", "omabeats_spatial"])
        .status();

    crate::equalizer::move_all_sink_inputs_to("omabeats_spatial");

    true
}

/// Generates PipeWire Filter-Chain configuration for Cinema / Dolby Atmos Virtual Surround
/// Utilizes stereo upmixing with center dialogue clarity, LFE subwoofer boost,
/// and phase-shifted rear surround binaural channels (Hilbert transform convolver).
fn generate_cinema_dolby_config(target_sink: &str) -> String {
    format!(
        r#"context.modules = [
    {{ name = libpipewire-module-filter-chain
        args = {{
            node.description = "Beats Cinema Dolby Virtual Surround"
            media.name       = "Beats Cinema Dolby Virtual Surround"
            filter.graph = {{
                nodes = [
                    {{ type = builtin name = copyFL label = copy }}
                    {{ type = builtin name = copyFR label = copy }}
                    {{
                        name   = mixF
                        type   = builtin
                        label  = mixer
                        control = {{
                            "Gain 1" = 0.707
                            "Gain 2" = 0.707
                        }}
                    }}
                    {{
                        type = builtin
                        name = eq_FC_LFE
                        label = param_eq
                        config = {{
                            filters1 = [
                                {{ type = bq_peaking freq = 2500 q = 1.2 gain = 3.0 }}
                                {{ type = bq_lowpass freq = 12000 }}
                            ]
                            filters2 = [
                                {{ type = bq_lowshelf freq = 90 q = 1.0 gain = 4.5 }}
                                {{ type = bq_lowpass freq = 140 }}
                            ]
                        }}
                    }}
                    {{
                        name   = subR
                        type   = builtin
                        label  = mixer
                        control = {{
                            "Gain 1" = 0.707
                            "Gain 2" = -0.707
                        }}
                    }}
                    {{
                        type   = builtin
                        name   = convRL
                        label  = convolver
                        config = {{
                            gain = 1.0
                            delay = 0.015
                            filename = "/hilbert"
                            length = 33
                            latency = 0.0
                        }}
                    }}
                    {{
                        type   = builtin
                        name   = convRR
                        label  = convolver
                        config = {{
                            gain = -1.0
                            delay = 0.015
                            filename = "/hilbert"
                            length = 33
                            latency = 0.0
                        }}
                    }}
                    {{
                        name   = mixOutL
                        type   = builtin
                        label  = mixer
                        control = {{
                            "Gain 1" = 0.85
                            "Gain 2" = 0.50
                            "Gain 3" = 0.40
                            "Gain 4" = 0.45
                        }}
                    }}
                    {{
                        name   = mixOutR
                        type   = builtin
                        label  = mixer
                        control = {{
                            "Gain 1" = 0.85
                            "Gain 2" = 0.50
                            "Gain 3" = 0.40
                            "Gain 4" = 0.45
                        }}
                    }}
                ]
                links = [
                    {{ output = "copyFL:Out" input = "mixF:In 1" }}
                    {{ output = "copyFR:Out" input = "mixF:In 2" }}
                    {{ output = "mixF:Out" input = "eq_FC_LFE:In 1" }}
                    {{ output = "mixF:Out" input = "eq_FC_LFE:In 2" }}
                    {{ output = "copyFL:Out" input = "subR:In 1" }}
                    {{ output = "copyFR:Out" input = "subR:In 2" }}
                    {{ output = "subR:Out" input = "convRL:In" }}
                    {{ output = "subR:Out" input = "convRR:In" }}

                    {{ output = "copyFL:Out" input = "mixOutL:In 1" }}
                    {{ output = "eq_FC_LFE:Out 1" input = "mixOutL:In 2" }}
                    {{ output = "eq_FC_LFE:Out 2" input = "mixOutL:In 3" }}
                    {{ output = "convRL:Out" input = "mixOutL:In 4" }}

                    {{ output = "copyFR:Out" input = "mixOutR:In 1" }}
                    {{ output = "eq_FC_LFE:Out 1" input = "mixOutR:In 2" }}
                    {{ output = "eq_FC_LFE:Out 2" input = "mixOutR:In 3" }}
                    {{ output = "convRR:Out" input = "mixOutR:In 4" }}
                ]
                inputs = [ "copyFL:In" "copyFR:In" ]
                outputs = [ "mixOutL:Out" "mixOutR:Out" ]
            }}
            audio.channels = 2
            audio.position = [ FL FR ]
            capture.props = {{
                node.name        = "omabeats_spatial"
                media.class      = Audio/Sink
                node.description = "Beats Spatial Cinema (Dolby Virtual Surround)"
            }}
            playback.props = {{
                node.name        = "omabeats_spatial_out"
                node.passive     = true
                target.object    = "{target}"
            }}
        }}
    }}
]
"#,
        target = target_sink
    )
}

/// Generates PipeWire Filter-Chain configuration for Music Spatial Audio (Binaural Wide Soundstage)
/// Creates an expansive, natural acoustic stage similar to Apple Spatial Audio
/// without coloration or phase cancellation.
fn generate_music_spatial_config(target_sink: &str) -> String {
    format!(
        r#"context.modules = [
    {{ name = libpipewire-module-filter-chain
        args = {{
            node.description = "Beats Spatial Music Stage"
            media.name       = "Beats Spatial Music Stage"
            filter.graph = {{
                nodes = [
                    {{ type = builtin name = copyFL label = copy }}
                    {{ type = builtin name = copyFR label = copy }}
                    {{
                        name   = diffMid
                        type   = builtin
                        label  = mixer
                        control = {{
                            "Gain 1" = 0.50
                            "Gain 2" = 0.50
                        }}
                    }}
                    {{
                        name   = diffSide
                        type   = builtin
                        label  = mixer
                        control = {{
                            "Gain 1" = 0.50
                            "Gain 2" = -0.50
                        }}
                    }}
                    {{
                        type   = builtin
                        name   = sideSpatialL
                        label  = delay
                        config = {{ "max-delay" = 1 }}
                        control = {{ "Delay (s)" = 0.0006 }}
                    }}
                    {{
                        type   = builtin
                        name   = sideSpatialR
                        label  = delay
                        config = {{ "max-delay" = 1 }}
                        control = {{ "Delay (s)" = 0.0006 }}
                    }}
                    {{
                        name   = mixOutL
                        type   = builtin
                        label  = mixer
                        control = {{
                            "Gain 1" = 0.95
                            "Gain 2" = 0.45
                        }}
                    }}
                    {{
                        name   = mixOutR
                        type   = builtin
                        label  = mixer
                        control = {{
                            "Gain 1" = 0.95
                            "Gain 2" = -0.45
                        }}
                    }}
                ]
                links = [
                    {{ output = "copyFL:Out" input = "diffMid:In 1" }}
                    {{ output = "copyFR:Out" input = "diffMid:In 2" }}
                    {{ output = "copyFL:Out" input = "diffSide:In 1" }}
                    {{ output = "copyFR:Out" input = "diffSide:In 2" }}

                    {{ output = "diffSide:Out" input = "sideSpatialL:In" }}
                    {{ output = "diffSide:Out" input = "sideSpatialR:In" }}

                    {{ output = "diffMid:Out" input = "mixOutL:In 1" }}
                    {{ output = "sideSpatialL:Out" input = "mixOutL:In 2" }}

                    {{ output = "diffMid:Out" input = "mixOutR:In 1" }}
                    {{ output = "sideSpatialR:Out" input = "mixOutR:In 2" }}
                ]
                inputs = [ "copyFL:In" "copyFR:In" ]
                outputs = [ "mixOutL:Out" "mixOutR:Out" ]
            }}
            audio.channels = 2
            audio.position = [ FL FR ]
            capture.props = {{
                node.name        = "omabeats_spatial"
                media.class      = Audio/Sink
                node.description = "Beats Spatial Audio (Music Soundstage)"
            }}
            playback.props = {{
                node.name        = "omabeats_spatial_out"
                node.passive     = true
                target.object    = "{target}"
            }}
        }}
    }}
]
"#,
        target = target_sink
    )
}
