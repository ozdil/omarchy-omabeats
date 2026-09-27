mod aap;
mod bluez;
mod equalizer;
mod l2cap;
mod mock;
mod models;
mod mpris;
mod security;
mod state;
mod transparency;
mod wired;

use aap::{AncMode, MicMode};
use bluez::{connect_device, detect_active_codec, disconnect_device, discover_beats_devices};
use l2cap::L2capConnection;
use mock::{apply_param_mutation, create_mock_state, load_state, save_state};
use state::BeatsState;
use std::env;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn get_socket_path() -> PathBuf {
    let uid = unsafe { libc::getuid() };
    let runtime_dir = format!("/run/user/{}", uid);
    if Path::new(&runtime_dir).exists() {
        PathBuf::from(runtime_dir).join("omabeats.sock")
    } else {
        PathBuf::from(format!("/tmp/omabeats_{}.sock", uid))
    }
}

fn try_send_daemon_command(cmd_line: &str) -> Option<String> {
    let path = get_socket_path();
    if !path.exists() {
        return None;
    }

    let mut stream = UnixStream::connect(path).ok()?;
    stream.set_read_timeout(Some(Duration::from_millis(2000))).ok()?;
    stream.set_write_timeout(Some(Duration::from_millis(500))).ok()?;

    stream.write_all(cmd_line.as_bytes()).ok()?;
    stream.write_all(b"\n").ok()?;
    let _ = stream.shutdown(std::net::Shutdown::Write);

    let mut response = String::new();
    let mut take_stream = stream.take(1024 * 1024);
    take_stream.read_to_string(&mut response).ok()?;
    if response.trim().is_empty() {
        None
    } else {
        Some(response)
    }
}

fn print_usage() {
    eprintln!(
        r#"OmaBeats Engine v1.0.0 - Omarchy Linux Beats Kulaklik Yonetim Motoru

KULLANIM:
    omabeats-engine <KOMUT> [ARGUMANLAR...]

KOMUTLAR:
    daemon                      Arka plan donanim ve olay dinleme servisini baslatir
    status                      Mevcut kulaklik durumunu JSON olarak dondurur
    sync                        BlueZ, USB ve donanim durumunu tarayip gunceller
    models                      Desteklenen tum Beats modellerini listeler
    wired <MODEL|reset>         Kablolu Beats modelini secer (ornek: beats_ep, beats_pro, reset)
    anc <MOD>                   ANC modunu ayarlar (off | noise | transparency | adaptive)
    mic <MOD>                   Mikrofon yonlendirmesini ayarlar (auto | left | right)
    eq <PROFIL>                 Ekolayzer profilini secer (Beats Signature | Bass Boost | Vocal Clarity | Flat)
    volume <0-100>              Ses yuksekligini ayarlar
    auto-pause <true|false>     Kulak ici otomatik duraklatmayi etkinlestirir / kapatir
    chime <sol|sag|ikisi|off>   Kayip kulakligi bulmak icin ses caldirir
    connect [MAC]               Beats kulakliga baglanir
    disconnect [MAC]            Kulaklik baglantisini keser
    toggle-pause                Medya oynatmayi duraklatir veya surdurur (MPRIS)
    mock <MODEL>                Test/Simulator modunu baslatir
    set <PARAMETRE> <DEGER>     Test modunda parametre gunceller
    help                        Bu yardim iletisini gosterir
"#
    );
}

fn cmd_dashboard() {
    let mut cmd = std::process::Command::new("/usr/bin/quickshell");
    if !Path::new("/usr/bin/quickshell").exists() {
        cmd = std::process::Command::new("quickshell");
    }

    let candidate_paths = [
        "/home/ozdil/.config/omarchy/plugins/ozdil.omabeats/qml/shell.qml",
        "/home/ozdil/Projects/omarchy/omarchy-omabeats/qml/shell.qml",
        "/usr/share/omarchy/plugins/ozdil.omabeats/qml/shell.qml",
    ];

    if let Some(&path) = candidate_paths.iter().find(|p| Path::new(p).exists()) {
        cmd.process_group(0);
        cmd.args(["-p", path]);
        if let Ok(mut child) = cmd.spawn() {
            let _ = child.wait();
            return;
        }
    }

    cmd_status();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        if std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok() {
            cmd_dashboard();
            return;
        } else {
            print_usage();
            std::process::exit(0);
        }
    }

    let command = args[1].to_lowercase();

    if command == "daemon" {
        cmd_daemon();
        return;
    }

    if command == "dashboard" || command == "gui" || command == "app" {
        cmd_dashboard();
        return;
    }

    if command == "--help" || command == "-h" || command == "help" {
        print_usage();
        return;
    }

    // Try communicating via background daemon for instant response
    let full_cmd = args[1..].join(" ");
    if let Some(response) = try_send_daemon_command(&full_cmd) {
        print!("{}", response);
        return;
    }

    match command.as_str() {
        "dashboard" | "gui" | "app" => cmd_dashboard(),
        "status" => cmd_status(),
        "sync" => cmd_sync(),
        "anc" => {
            if args.len() < 3 {
                eprintln!("Hata: ANC modu belirtilmedi (off, noise, transparency, adaptive).");
                std::process::exit(1);
            }
            cmd_set_anc(&args[2]);
        }
        "mic" => {
            if args.len() < 3 {
                eprintln!("Hata: Mikrofon modu belirtilmedi (auto, left, right).");
                std::process::exit(1);
            }
            cmd_set_mic(&args[2]);
        }
        "eq" => {
            if args.len() < 3 {
                eprintln!("Hata: EQ profili belirtilmedi.");
                std::process::exit(1);
            }
            let profile = args[2..].join(" ");
            cmd_set_eq(&profile);
        }
        "chime" => {
            let target = if args.len() >= 3 { &args[2] } else { "both" };
            cmd_chime(target);
        }
        "volume" => {
            if args.len() < 3 {
                eprintln!("Hata: Ses duzeyi (0-100) belirtilmedi.");
                std::process::exit(1);
            }
            cmd_set_volume(&args[2]);
        }
        "connect" => {
            let mac = args.get(2).map(|s| s.as_str());
            cmd_connect(mac);
        }
        "disconnect" => {
            let mac = args.get(2).map(|s| s.as_str());
            cmd_disconnect(mac);
        }
        "toggle-pause" => {
            let state = load_state().unwrap_or_default();
            if state.auto_pause_enabled {
                mpris::pause_media();
            } else {
                mpris::resume_media();
            }
            println!("{{\"success\":true}}");
        }
        "auto-pause" => {
            let enable = if args.len() >= 3 {
                args[2].eq_ignore_ascii_case("true") || args[2] == "1"
            } else {
                let s = load_state().unwrap_or_default();
                !s.auto_pause_enabled
            };
            let mut state = load_state().unwrap_or_default();
            state.auto_pause_enabled = enable;
            if !state.mac.is_empty() {
                if let Ok(conn) = L2capConnection::connect(&state.mac) {
                    let _ = conn.set_in_ear_detection(enable);
                }
            }
            let _ = save_state(&state);
            println!("{{\"success\":true,\"auto_pause_enabled\":{}}}", enable);
        }
        "mock" => {
            let model = if args.len() >= 3 { &args[2] } else { "beats_fit_pro" };
            cmd_mock(model);
        }
        "set" => {
            if args.len() < 4 {
                eprintln!("Hata: set <parametre> <deger> biciminde arguman bekleniyor.");
                std::process::exit(1);
            }
            cmd_set_param(&args[2], &args[3]);
        }
        "wired" => {
            if args.len() < 3 {
                eprintln!("Hata: Kablolu model ID belirtilmedi (ornek: beats_ep, beats_pro, urbeats_3, reset).");
                std::process::exit(1);
            }
            if args[2] == "reset" || args[2] == "clear" {
                cmd_reset_wired();
            } else {
                cmd_set_wired(&args[2]);
            }
        }
        "models" => {
            cmd_models();
        }
        "--help" | "-h" | "help" => {
            print_usage();
        }
        _ => {
            eprintln!("Bilinmeyen komut: {}", command);
            print_usage();
            std::process::exit(1);
        }
    }
}

fn cmd_status() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let state = match load_state() {
        Some(s) if !s.test_mode && now.saturating_sub(s.last_updated) < 2 => s,
        _ => {
            let s = perform_sync();
            let _ = save_state(&s);
            s
        }
    };

    match serde_json::to_string_pretty(&state) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("{{\"error\":\"{}\"}}", e),
    }
}

fn cmd_sync() {
    let state = perform_sync();
    let _ = save_state(&state);
    if let Ok(json) = serde_json::to_string_pretty(&state) {
        println!("{}", json);
    }
}

fn get_system_volume() -> i32 {
    let mut cmd = std::process::Command::new("/usr/bin/pactl");
    cmd.process_group(0);
    cmd.args(["get-sink-volume", "@DEFAULT_SINK@"]);
    cmd.stdin(std::process::Stdio::null());

    let output = match cmd.output() {
        Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
        Err(_) => return 50,
    };

    for part in output.split_whitespace() {
        if part.ends_with('%') {
            if let Ok(val) = part.trim_end_matches('%').parse::<i32>() {
                return val.clamp(0, 100);
            }
        }
    }
    50
}

fn apply_system_volume(val: i32, mac: Option<&str>) {
    let mut cmd = std::process::Command::new("/usr/bin/pactl");
    cmd.process_group(0);
    cmd.args(["set-sink-volume", "@DEFAULT_SINK@", &format!("{}%", val)]);
    cmd.stdin(std::process::Stdio::null());
    let _ = cmd.status();

    if let Some(sink) = equalizer::get_bluetooth_sink_name(mac) {
        let mut bt_cmd = std::process::Command::new("/usr/bin/pactl");
        bt_cmd.process_group(0);
        bt_cmd.args(["set-sink-volume", &sink, &format!("{}%", val)]);
        bt_cmd.stdin(std::process::Stdio::null());
        let _ = bt_cmd.status();
    }
}

fn cmd_set_volume(val_str: &str) {
    let mut state = load_state().unwrap_or_default();
    let val = val_str.parse::<i32>().unwrap_or(state.volume).clamp(0, 100);
    state.volume = val;

    let mac = if !state.mac.is_empty() { Some(state.mac.as_str()) } else { None };
    apply_system_volume(val, mac);

    let _ = save_state(&state);
    println!("{{\"success\":true,\"volume\":{}}}", val);
}

fn perform_sync() -> BeatsState {
    let mut state = load_state().unwrap_or_default();
    let prev_in_ear = state.in_ear_left && state.in_ear_right;

    let devices = discover_beats_devices();
    if let Some(dev) = devices.iter().find(|d| d.connected).or_else(|| devices.first()) {
        if dev.connected {
            state.test_mode = false;
            state.connected = true;
            state.is_wired = false;
            state.wired_model = None;
            state.connection_type = Some(dev.model.default_connection);
            state.mac = dev.mac.clone();
            state.model = dev.model.clone();
            state.rssi = dev.rssi.unwrap_or(-60);
            state.codec = detect_active_codec(&dev.mac);

            if let Some(bat) = dev.battery_level {
                if dev.model.has_tri_battery {
                    state.battery_left = bat;
                    state.battery_right = bat;
                } else {
                    state.battery_single = bat;
                }
            }

            // Connect L2CAP and read incoming AAP notification stream
            if let Ok(conn) = L2capConnection::connect(&dev.mac) {
                let packets = conn.read_all_notifications(Duration::from_millis(250));
                for data in packets {
                    if let Some(event) = aap::parser::parse_packet(&data) {
                        match event {
                            aap::parser::ParsedAapEvent::Battery(rep) => {
                                if let Some(l) = rep.left {
                                    state.battery_left = l.level;
                                    state.charging_left = l.charging;
                                }
                                if let Some(r) = rep.right {
                                    state.battery_right = r.level;
                                    state.charging_right = r.charging;
                                }
                                if let Some(c) = rep.case {
                                    state.battery_case = c.level;
                                    state.charging_case = c.charging;
                                } else {
                                    state.charging_case = false;
                                }
                                if let Some(s) = rep.single {
                                    state.battery_single = s.level;
                                    state.charging_single = s.charging;
                                }
                            }
                            aap::parser::ParsedAapEvent::AncMode(m) => {
                                state.anc_mode = m;
                            }
                            aap::parser::ParsedAapEvent::EarDetection(ear) => {
                                state.in_ear_left = ear.left_in_ear;
                                state.in_ear_right = ear.right_in_ear;
                            }
                            aap::parser::ParsedAapEvent::DeviceInfo(info) => {
                                if !info.firmware.is_empty() {
                                    state.firmware_version = info.firmware;
                                }
                                if !info.serial.is_empty() {
                                    state.serial_number = info.serial;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            // In-Ear Auto-Pause detection
            let new_in_ear = state.in_ear_left && state.in_ear_right;
            if state.auto_pause_enabled {
                if prev_in_ear && !new_in_ear {
                    mpris::pause_media();
                } else if !prev_in_ear && new_in_ear {
                    mpris::resume_media();
                }
            }

            state.volume = get_system_volume();
        } else {
            // Device paired in BlueZ but not currently connected
            if !state.test_mode {
                state.connected = false;
                state.mac = dev.mac.clone();
                state.model = dev.model.clone();
                state.battery_left = -1;
                state.battery_right = -1;
                if state.battery_case <= 0 {
                    state.battery_case = -1;
                }
                state.charging_case = false;
                state.battery_single = -1;
            }
        }
    } else {
        // No Beats devices found in BlueZ
        if !state.test_mode {
            state.connected = false;
            state.battery_left = -1;
            state.battery_right = -1;
            state.battery_case = -1;
            state.charging_case = false;
            state.battery_single = -1;
        }
    }

    // Check for USB Lossless Beats Device or active wired selection if not connected via Bluetooth
    if !state.connected && !state.test_mode {
        if let Some(usb_dev) = wired::detect_usb_beats_device() {
            state.connected = true;
            state.is_wired = true;
            state.connection_type = Some(usb_dev.connection_type);
            state.model = usb_dev.model;
            state.codec = "USB 24-bit Lossless".to_string();
            state.battery_left = -1;
            state.battery_right = -1;
            state.battery_case = -1;
        } else if state.is_wired && state.wired_model.is_some() {
            state.connected = true;
        }
    }

    state.last_updated = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    state
}

fn resolve_anc_mode_and_level(input: &str) -> Option<(AncMode, i32)> {
    let clean = input.trim().to_lowercase();
    if let Ok(num) = clean.parse::<i32>() {
        let level = num.clamp(0, 100);
        let mode = if level <= 30 {
            AncMode::NoiseCancellation
        } else if level <= 50 {
            AncMode::Off
        } else {
            AncMode::Transparency
        };
        return Some((mode, level));
    }

    match clean.as_str() {
        "noise" | "anc" | "noisecancellation" | "cancellation" => Some((AncMode::NoiseCancellation, 0)),
        "off" | "kapali" => Some((AncMode::Off, 50)),
        "transparency" | "seffaf" | "ambient" => Some((AncMode::Transparency, 100)),
        "adaptive" | "uyumlu" => Some((AncMode::Adaptive, 75)),
        _ => None,
    }
}

fn cmd_set_anc(mode_str: &str) {
    let (mode, level) = match resolve_anc_mode_and_level(mode_str) {
        Some(res) => res,
        None => {
            eprintln!("Gecersiz ANC modu: {}. Gecerli modlar: 0-100, off, noise, transparency, adaptive", mode_str);
            std::process::exit(1);
        }
    };

    let mut state = load_state().unwrap_or_default();
    state.anc_mode = mode;
    state.noise_control_level = level;

    if !state.mac.is_empty() {
        if let Ok(conn) = L2capConnection::connect(&state.mac) {
            conn.drain();
            let _ = conn.set_anc_mode(mode);
            let packets = conn.read_all_notifications(Duration::from_millis(150));
            for data in packets {
                if let Some(aap::parser::ParsedAapEvent::AncMode(m)) = aap::parser::parse_packet(&data) {
                    state.anc_mode = m;
                }
            }
        }
    }

    if mode == AncMode::Transparency && level > 50 {
        transparency::set_ambient_passthrough(level as u32);
    } else {
        transparency::disable_ambient_passthrough();
    }

    let _ = save_state(&state);
    println!("{{\"success\":true,\"anc_mode\":\"{}\",\"noise_control_level\":{}}}", state.anc_mode.as_str(), state.noise_control_level);
}

fn cmd_set_mic(mode_str: &str) {
    let mode = match MicMode::from_str_name(mode_str) {
        Some(m) => m,
        None => {
            eprintln!("Gecersiz mikrofon modu: {}. Gecerli modlar: auto, left, right", mode_str);
            std::process::exit(1);
        }
    };

    let mut state = load_state().unwrap_or_default();
    state.mic_mode = mode;

    if !state.mac.is_empty() {
        if let Ok(conn) = L2capConnection::connect(&state.mac) {
            let _ = conn.set_mic_mode(mode);
        }
    }

    let _ = save_state(&state);
    println!("{{\"success\":true,\"mic_mode\":\"{}\"}}", mode.as_str());
}

fn cmd_set_eq(profile: &str) {
    let mut state = load_state().unwrap_or_default();
    state.eq_profile = profile.to_string();

    let mac = if !state.mac.is_empty() { Some(state.mac.as_str()) } else { None };
    let success = equalizer::apply_profile(profile, mac);

    state.volume = get_system_volume();
    let _ = save_state(&state);
    println!("{{\"success\":{},\"eq_profile\":\"{}\",\"volume\":{}}}", success, profile, state.volume);
}

fn cmd_set_wired(model_id: &str) {
    let mut state = load_state().unwrap_or_default();
    let model = models::find_model(model_id);
    state.connected = true;
    state.is_wired = true;
    state.wired_model = Some(model_id.to_string());
    state.model = model;
    state.connection_type = Some(models::ConnectionType::AnalogJack);
    let eq = wired::get_wired_model_eq_preset(model_id);
    state.eq_profile = eq.to_string();
    let mac = if !state.mac.is_empty() { Some(state.mac.as_str()) } else { None };
    let _ = equalizer::apply_profile(eq, mac);
    let _ = save_state(&state);
    println!("{{\"success\":true,\"wired_model\":\"{}\",\"eq\":\"{}\"}}", model_id, eq);
}

fn cmd_reset_wired() {
    let mut state = load_state().unwrap_or_default();
    state.is_wired = false;
    state.wired_model = None;
    equalizer::stop_equalizer();
    let _ = save_state(&state);
    println!("{{\"success\":true,\"reset\":true}}");
}

fn cmd_models() {
    let list = models::get_known_beats_models();
    if let Ok(json) = serde_json::to_string_pretty(&list) {
        println!("{}", json);
    }
}

fn cmd_chime(target: &str) {
    let mut state = load_state().unwrap_or_default();
    if target == "off" || target == "none" {
        state.chime_active = None;
    } else {
        state.chime_active = Some(target.to_string());
        let wav_name = match target.to_lowercase().as_str() {
            "left" | "sol" => "chime_left.wav",
            "right" | "sag" => "chime_right.wav",
            _ => "chime_both.wav",
        };

        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let paths = [
            format!("{}/.config/omarchy/plugins/ozdil.omabeats/resources/{}", home, wav_name),
            format!("{}/Projects/omarchy/omarchy-omabeats/resources/{}", home, wav_name),
            format!("/tmp/{}", wav_name),
        ];

        if let Some(sound_path) = paths.iter().find(|p| Path::new(p).exists()) {
            let sink = equalizer::get_bluetooth_sink_name(Some(&state.mac))
                .unwrap_or_else(|| format!("bluez_output.{}.1", state.mac.replace(':', "_")));
            let mut play_cmd = std::process::Command::new("/usr/bin/pw-play");
            play_cmd.process_group(0);
            play_cmd.args(["--target", &sink, sound_path]);
            play_cmd.stdin(std::process::Stdio::null());
            play_cmd.stdout(std::process::Stdio::null());
            play_cmd.stderr(std::process::Stdio::null());
            std::thread::spawn(move || {
                if let Ok(mut guard) = crate::security::spawn_isolated(play_cmd) {
                    if let Some(mut child) = guard.take() {
                        let _ = child.wait();
                    }
                }
            });
        }
    }
    let _ = save_state(&state);
    println!("{{\"success\":true,\"chime_active\":{:?}}}", state.chime_active);
}

fn cmd_connect(mac_opt: Option<&str>) {
    let mut state = load_state().unwrap_or_default();
    let target_mac = mac_opt
        .map(|s| s.to_string())
        .or_else(|| {
            let devices = discover_beats_devices();
            devices.first().map(|d| d.mac.clone())
        })
        .or_else(|| if !state.mac.is_empty() { Some(state.mac.clone()) } else { None });

    match target_mac {
        Some(mac) => match connect_device(&mac) {
            Ok(_) => {
                state.connected = true;
                state.mac = mac.clone();
                let _ = save_state(&state);
                println!("{{\"success\":true,\"connected\":true,\"mac\":\"{}\"}}", mac);
            }
            Err(e) => {
                eprintln!("{{\"success\":false,\"error\":\"{}\"}}", e);
                std::process::exit(1);
            }
        },
        None => {
            eprintln!("{{\"success\":false,\"error\":\"Baglanilacak cihaz bulunamadi\"}}");
            std::process::exit(1);
        }
    }
}

fn cmd_disconnect(mac_opt: Option<&str>) {
    let mut state = load_state().unwrap_or_default();
    let target_mac = mac_opt.map(|s| s.to_string()).or_else(|| {
        if !state.mac.is_empty() {
            Some(state.mac.clone())
        } else {
            let devices = discover_beats_devices();
            devices.iter().find(|d| d.connected).map(|d| d.mac.clone())
        }
    });

    match target_mac {
        Some(mac) => match disconnect_device(&mac) {
            Ok(_) => {
                state.connected = false;
                state.battery_left = -1;
                state.battery_right = -1;
                state.battery_case = -1;
                state.battery_single = -1;
                let _ = save_state(&state);
                println!("{{\"success\":true,\"connected\":false}}");
            }
            Err(e) => {
                eprintln!("{{\"success\":false,\"error\":\"{}\"}}", e);
                std::process::exit(1);
            }
        },
        None => {
            state.connected = false;
            let _ = save_state(&state);
            println!("{{\"success\":true,\"connected\":false}}");
        }
    }
}

fn cmd_mock(model_name: &str) {
    let mock_state = create_mock_state(model_name);
    let _ = save_state(&mock_state);
    if let Ok(json) = serde_json::to_string_pretty(&mock_state) {
        println!("{}", json);
    }
}

fn cmd_set_param(key: &str, value: &str) {
    let mut state = load_state().unwrap_or_default();
    match apply_param_mutation(&mut state, key, value) {
        Ok(()) => {
            let _ = save_state(&state);
            if let Ok(json) = serde_json::to_string_pretty(&state) {
                println!("{}", json);
            }
        }
        Err(e) => {
            eprintln!("{{\"success\":false,\"error\":\"{}\"}}", e);
            std::process::exit(1);
        }
    }
}

/// OmaBeats background hardware listener & Unix Domain Socket Server
fn cmd_daemon() {
    let socket_path = get_socket_path();
    if let Some(parent) = socket_path.parent() {
        if !parent.exists() {
            let _ = std::fs::create_dir_all(parent);
            let _ = std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700));
        }
    }

    if let Ok(meta) = std::fs::symlink_metadata(&socket_path) {
        if meta.file_type().is_symlink() {
            let _ = std::fs::remove_file(&socket_path);
        } else if socket_path.exists() {
            if UnixStream::connect(&socket_path).is_ok() {
                eprintln!("OmaBeats daemon zaten calisiyor.");
                return;
            }
            let _ = std::fs::remove_file(&socket_path);
        }
    }

    let listener = match UnixListener::bind(&socket_path) {
        Ok(l) => {
            // Set 0600 file permissions
            let _ = std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o600));
            l
        }
        Err(e) => {
            eprintln!("Daemon unix socket baglanamadi ({}): {}", socket_path.display(), e);
            std::process::exit(1);
        }
    };

    let active_conn: Arc<Mutex<Option<Arc<L2capConnection>>>> = Arc::new(Mutex::new(None));
    let active_mac: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));

    // Thread 1: Unix Domain Socket Server for instant CLI commands
    let conn_for_socket = Arc::clone(&active_conn);
    let mac_for_socket = Arc::clone(&active_mac);
    thread::spawn(move || {
        for stream in listener.incoming() {
            if let Ok(mut s) = stream {
                // Enforce Zero-Trust peer credential verification (SO_PEERCRED)
                if let Err(e) = security::verify_socket_peer_credentials(&s) {
                    eprintln!("OmaBeats Daemon Security Warning: {}", e);
                    let _ = s.write_all(b"{\"error\":\"Zero-Trust Access Denied: Unauthorized peer UID\"}\n");
                    let _ = s.shutdown(std::net::Shutdown::Both);
                    continue;
                }

                let Ok(cloned) = s.try_clone() else {
                    continue;
                };
                let mut reader = BufReader::new(cloned);
                let mut line = String::new();
                if reader.read_line(&mut line).is_ok() {
                    let trimmed = line.trim();
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.is_empty() {
                        continue;
                    }

                    let response = match parts[0].to_lowercase().as_str() {
                        "status" => {
                            let mut state = load_state().unwrap_or_default();
                            state.volume = get_system_volume();
                            serde_json::to_string(&state).unwrap_or_default()
                        }
                        "anc" => {
                            if parts.len() >= 2 {
                                if let Some((m, level)) = resolve_anc_mode_and_level(parts[1]) {
                                    let mut state = load_state().unwrap_or_default();
                                    state.anc_mode = m;
                                    state.noise_control_level = level;
                                    let mut sent_ok = false;
                                    let conn_opt = {
                                        let mut guard = conn_for_socket.lock().unwrap();
                                        if guard.is_none() {
                                            let mac = if !state.mac.is_empty() {
                                                state.mac.clone()
                                            } else {
                                                mac_for_socket.lock().unwrap().clone()
                                            };
                                            if !mac.is_empty() {
                                                if let Ok(new_conn) = L2capConnection::connect(&mac) {
                                                    *guard = Some(Arc::new(new_conn));
                                                }
                                            }
                                        }
                                        guard.clone()
                                    };
                                    if let Some(ref conn) = conn_opt {
                                        if conn.set_anc_mode(m).is_ok() {
                                            sent_ok = true;
                                        }
                                    }
                                    if !sent_ok {
                                        let mac = if !state.mac.is_empty() {
                                            state.mac.clone()
                                        } else {
                                            mac_for_socket.lock().unwrap().clone()
                                        };
                                        if !mac.is_empty() {
                                            if let Ok(new_conn) = L2capConnection::connect(&mac) {
                                                let _ = new_conn.set_anc_mode(m);
                                                let mut guard = conn_for_socket.lock().unwrap();
                                                *guard = Some(Arc::new(new_conn));
                                            }
                                        }
                                    }

                                    if m == AncMode::Transparency && level > 50 {
                                        transparency::set_ambient_passthrough(level as u32);
                                    } else {
                                        transparency::disable_ambient_passthrough();
                                    }

                                    let _ = save_state(&state);
                                    format!("{{\"success\":true,\"anc_mode\":\"{}\",\"noise_control_level\":{}}}", m.as_str(), level)
                                } else {
                                    format!("{{\"success\":false,\"error\":\"Invalid ANC mode {}\"}}", parts[1])
                                }
                            } else {
                                "{\"success\":false,\"error\":\"Missing ANC mode\"}".to_string()
                            }
                        }
                        "eq" => {
                            let mut state = load_state().unwrap_or_default();
                            state.eq_profile = "Flat".to_string();
                            let _ = save_state(&state);
                            equalizer::stop_equalizer();
                            "{\"success\":true,\"eq_profile\":\"Flat\"}".to_string()
                        }
                        "volume" => {
                            if parts.len() >= 2 {
                                let mut state = load_state().unwrap_or_default();
                                let val = parts[1].parse::<i32>().unwrap_or(state.volume).clamp(0, 100);
                                state.volume = val;
                                let mac_guard = mac_for_socket.lock().unwrap();
                                let mac_opt = if !mac_guard.is_empty() {
                                    Some(mac_guard.as_str())
                                } else if !state.mac.is_empty() {
                                    Some(state.mac.as_str())
                                } else {
                                    None
                                };
                                apply_system_volume(val, mac_opt);
                                let _ = save_state(&state);
                                format!("{{\"success\":true,\"volume\":{}}}", val)
                            } else {
                                "{\"success\":false,\"error\":\"Missing volume\"}".to_string()
                            }
                        }
                        "auto-pause" => {
                            if parts.len() >= 2 {
                                let enable = parts[1].eq_ignore_ascii_case("true") || parts[1] == "1";
                                let mut state = load_state().unwrap_or_default();
                                state.auto_pause_enabled = enable;
                                let conn_opt = {
                                    let mut guard = conn_for_socket.lock().unwrap();
                                    if guard.is_none() {
                                        let mac = if !state.mac.is_empty() {
                                            state.mac.clone()
                                        } else {
                                            mac_for_socket.lock().unwrap().clone()
                                        };
                                        if !mac.is_empty() {
                                            if let Ok(new_conn) = L2capConnection::connect(&mac) {
                                                *guard = Some(Arc::new(new_conn));
                                            }
                                        }
                                    }
                                    guard.clone()
                                };
                                if let Some(ref conn) = conn_opt {
                                    let _ = conn.set_in_ear_detection(enable);
                                }
                                let _ = save_state(&state);
                                format!("{{\"success\":true,\"auto_pause_enabled\":{}}}", enable)
                            } else {
                                "{\"success\":false,\"error\":\"Missing argument\"}".to_string()
                            }
                        }
                        "mic" => {
                            if parts.len() >= 2 {
                                if let Some(m) = MicMode::from_str_name(parts[1]) {
                                    let mut state = load_state().unwrap_or_default();
                                    state.mic_mode = m;
                                    let conn_opt = {
                                        let mut guard = conn_for_socket.lock().unwrap();
                                        if guard.is_none() {
                                            let mac = if !state.mac.is_empty() {
                                                state.mac.clone()
                                            } else {
                                                mac_for_socket.lock().unwrap().clone()
                                            };
                                            if !mac.is_empty() {
                                                if let Ok(new_conn) = L2capConnection::connect(&mac) {
                                                    *guard = Some(Arc::new(new_conn));
                                                }
                                            }
                                        }
                                        guard.clone()
                                    };
                                    if let Some(ref conn) = conn_opt {
                                        let _ = conn.set_mic_mode(m);
                                    }
                                    let _ = save_state(&state);
                                    format!("{{\"success\":true,\"mic_mode\":\"{}\"}}", m.as_str())
                                } else {
                                    format!("{{\"success\":false,\"error\":\"Invalid mic mode {}\"}}", parts[1])
                                }
                            } else {
                                "{\"success\":false,\"error\":\"Missing mic mode\"}".to_string()
                            }
                        }
                        "chime" => {
                            let target = if parts.len() >= 2 { parts[1] } else { "both" };
                            cmd_chime(target);
                            "{\"success\":true}".to_string()
                        }
                        "sync" => {
                            let s = perform_sync();
                            let _ = save_state(&s);
                            serde_json::to_string(&s).unwrap_or_default()
                        }
                        "wired" => {
                            if parts.len() >= 2 {
                                let model_id = parts[1];
                                if model_id == "reset" || model_id == "clear" {
                                    let mut state = load_state().unwrap_or_default();
                                    state.is_wired = false;
                                    state.wired_model = None;
                                    equalizer::stop_equalizer();
                                    let _ = save_state(&state);
                                    "{\"success\":true,\"reset\":true}".to_string()
                                } else {
                                    let mut state = load_state().unwrap_or_default();
                                    let model = models::find_model(model_id);
                                    state.connected = true;
                                    state.is_wired = true;
                                    state.wired_model = Some(model_id.to_string());
                                    state.model = model;
                                    state.connection_type = Some(models::ConnectionType::AnalogJack);
                                    let eq = wired::get_wired_model_eq_preset(model_id);
                                    state.eq_profile = eq.to_string();
                                    let mac = if !state.mac.is_empty() { Some(state.mac.as_str()) } else { None };
                                    let _ = equalizer::apply_profile(eq, mac);
                                    let _ = save_state(&state);
                                    format!("{{\"success\":true,\"wired_model\":\"{}\",\"eq\":\"{}\"}}", model_id, eq)
                                }
                            } else {
                                "{\"success\":false,\"error\":\"Missing model id\"}".to_string()
                            }
                        }
                        "models" => {
                            let list = models::get_known_beats_models();
                            serde_json::to_string(&list).unwrap_or_default()
                        }
                        _ => "{\"error\":\"Unknown daemon command\"}".to_string(),
                    };

                    let _ = s.write_all(response.as_bytes());
                    let _ = s.write_all(b"\n");
                }
            }
        }
    });

    // Thread 2 (Main Thread): Hardware L2CAP & In-Ear / Battery Event Monitoring Loop
    let mut last_discovery = std::time::Instant::now() - Duration::from_secs(10);
    let mut cached_dev: Option<bluez::DiscoveredDevice> = None;

    loop {
        transparency::verify_passthrough_safety();

        let conn_is_active = {
            let guard = active_conn.lock().unwrap();
            guard.is_some()
        };

        // When connected, throttle bluetoothctl discovery to once every 3s to minimize CPU
        // When disconnected, discover every 1.5s
        let interval = if conn_is_active {
            Duration::from_secs(3)
        } else {
            Duration::from_millis(1500)
        };

        if last_discovery.elapsed() >= interval {
            last_discovery = std::time::Instant::now();
            let devices = discover_beats_devices();
            cached_dev = devices.into_iter().find(|d| d.connected);
        }

        if let Some(ref dev) = cached_dev {
            let mut need_connect = false;
            {
                let mut mac_guard = active_mac.lock().unwrap();
                if *mac_guard != dev.mac {
                    *mac_guard = dev.mac.clone();
                    need_connect = true;
                }
            }

            {
                let guard = active_conn.lock().unwrap();
                if guard.is_none() {
                    need_connect = true;
                }
            }

            if need_connect {
                match L2capConnection::connect(&dev.mac) {
                    Ok(conn) => {
                        let mut state = load_state().unwrap_or_default();
                        state.connected = true;
                        state.is_wired = false;
                        state.wired_model = None;
                        state.connection_type = Some(dev.model.default_connection);
                        state.mac = dev.mac.clone();
                        state.model = dev.model.clone();
                        state.codec = detect_active_codec(&dev.mac);
                        let _ = save_state(&state);

                        let mut guard = active_conn.lock().unwrap();
                        *guard = Some(Arc::new(conn));

                        equalizer::stop_equalizer();
                    }
                    Err(_) => {
                        thread::sleep(Duration::from_millis(1500));
                        continue;
                    }
                }
            }

            // Read packets from active L2CAP connection using non-blocking poll (100ms timeout)
            let conn_opt = {
                let guard = active_conn.lock().unwrap();
                guard.clone()
            };

            if let Some(ref conn) = conn_opt {
                match conn.poll_read_packet(100) {
                    Ok(Some(data)) => {
                        eprintln!("[l2cap-recv] {:02x?}", data);
                        if let Some(event) = aap::parser::parse_packet(&data) {
                            let mut state = load_state().unwrap_or_default();
                            let prev_left = state.in_ear_left;
                            let prev_right = state.in_ear_right;
                            let prev_count = (prev_left as usize) + (prev_right as usize);

                            match event {
                                aap::parser::ParsedAapEvent::Battery(rep) => {
                                    if let Some(l) = rep.left {
                                        state.battery_left = l.level;
                                        state.charging_left = l.charging;
                                        if l.charging {
                                            state.in_ear_left = false;
                                        }
                                    }
                                    if let Some(r) = rep.right {
                                        state.battery_right = r.level;
                                        state.charging_right = r.charging;
                                        if r.charging {
                                            state.in_ear_right = false;
                                        }
                                    }
                                    if let Some(c) = rep.case {
                                        state.battery_case = c.level;
                                        state.charging_case = c.charging;
                                    } else {
                                        state.charging_case = false;
                                    }
                                    if let Some(s) = rep.single {
                                        state.battery_single = s.level;
                                        state.charging_single = s.charging;
                                    }

                                    // SAFETY: If both earbuds are charging in the case, instantly disable passthrough loopback
                                    if state.charging_left && state.charging_right {
                                        transparency::disable_ambient_passthrough();
                                    }

                                    let curr_count = (state.in_ear_left as usize) + (state.in_ear_right as usize);
                                    if state.auto_pause_enabled && prev_count > 0 && curr_count < prev_count {
                                        mpris::pause_media();
                                        state.paused_by_auto_pause = true;
                                    }
                                }
                                aap::parser::ParsedAapEvent::AncMode(m) => {
                                    state.anc_mode = m;
                                }
                                aap::parser::ParsedAapEvent::EarDetection(ear) => {
                                    state.in_ear_left = ear.left_in_ear;
                                    state.in_ear_right = ear.right_in_ear;
                                    let curr_count = (state.in_ear_left as usize) + (state.in_ear_right as usize);

                                    if state.auto_pause_enabled {
                                        if curr_count < prev_count && prev_count > 0 {
                                            mpris::pause_media();
                                            state.paused_by_auto_pause = true;
                                        } else if curr_count > prev_count && state.paused_by_auto_pause {
                                            mpris::resume_media();
                                            state.paused_by_auto_pause = false;
                                        }
                                    }
                                }
                                _ => {}
                            }

                            state.last_updated = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                            let _ = save_state(&state);
                        }
                    }
                    Ok(None) => {
                        // 100ms elapsed with no pending packet; loop again
                    }
                    Err(_) => {
                        // Socket reset or remote disconnected
                        let mut guard = active_conn.lock().unwrap();
                        *guard = None;
                        let mut mac_guard = active_mac.lock().unwrap();
                        mac_guard.clear();
                        let mut state = load_state().unwrap_or_default();
                        state.connected = false;
                        let _ = save_state(&state);
                        transparency::disable_ambient_passthrough();
                        equalizer::stop_equalizer();
                        cached_dev = None;
                        thread::sleep(Duration::from_millis(1000));
                    }
                }
            }
        } else {
            // Headset disconnected in BlueZ -> check for USB Lossless or Wired Beats
            let mut was_connected = false;
            {
                let mut guard = active_conn.lock().unwrap();
                if guard.is_some() {
                    *guard = None;
                    was_connected = true;
                }
            }
            if was_connected {
                let mut mac_guard = active_mac.lock().unwrap();
                mac_guard.clear();
                let mut state = load_state().unwrap_or_default();
                state.connected = false;
                let _ = save_state(&state);
                transparency::disable_ambient_passthrough();
                equalizer::stop_equalizer();
            }

            // USB-C Beats Detection (Beats Studio Pro, Beats Solo 4, Beats Pill 2024 Lossless Mode)
            if let Some(usb_dev) = wired::detect_usb_beats_device() {
                let mut state = load_state().unwrap_or_default();
                if !state.connected || !state.is_wired {
                    state.connected = true;
                    state.is_wired = true;
                    state.connection_type = Some(usb_dev.connection_type);
                    state.model = usb_dev.model;
                    state.codec = "USB 24-bit Lossless".to_string();
                    let _ = save_state(&state);
                }
            } else {
                let mut state = load_state().unwrap_or_default();
                if state.is_wired && state.wired_model.is_none() {
                    state.connected = false;
                    state.is_wired = false;
                    let _ = save_state(&state);
                }
            }

            transparency::verify_passthrough_safety();
            thread::sleep(Duration::from_millis(1000));
        }
    }
}
