mod aap;
mod bluez;
mod l2cap;
mod mock;
mod models;
mod mpris;
mod security;
mod state;

use aap::{AncMode, MicMode};
use bluez::{connect_device, detect_active_codec, disconnect_device, discover_beats_devices};
use l2cap::L2capConnection;
use mock::{apply_param_mutation, create_mock_state, load_state, save_state};
use state::BeatsState;
use std::env;

fn print_usage() {
    eprintln!(
        r#"OmaBeats Engine v1.0.0 - Omarchy Linux Beats Kulaklik Yonetim Motoru

KULLANIM:
    omabeats-engine <KOMUT> [ARGUMANLAR...]

KOMUTLAR:
    status                      Mevcut kulaklik durumunu JSON olarak dondurur
    sync                        BlueZ ve donanim durumunu tarayip durumu gunceller
    anc <MOD>                   ANC modunu ayarlar (off | noise | transparency | adaptive)
    mic <MOD>                   Mikrofon yonlendirmesini ayarlar (auto | left | right)
    eq <PROFIL>                 Ekolayzer profilini secer (Beats Signature | Bass Boost | Vocal Clarity | Flat)
    chime <sol|sag|ikisi|off>   Kayip kulakligi bulmak icin ses caldirir
    connect [MAC]               Beats kulakliga baglanir
    disconnect [MAC]            Kulaklik baglantisini keser
    toggle-pause                Medya oynatmayi duraklatir veya surdurur (MPRIS)
    mock <MODEL>                Test/Simulator modunu baslatir (or: beats_fit_pro, beats_studio_pro, beats_solo_4)
    set <PARAMETRE> <DEGER>     Test modunda parametre gunceller (or: bat_left 50, ear_left false)
    help                        Bu yardim iletisini gosterir
"#
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let command = args[1].to_lowercase();
    match command.as_str() {
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
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let state = match load_state() {
        Some(s) if !s.test_mode && now.saturating_sub(s.last_updated) < 5 => s,
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
    let output = match std::process::Command::new("/usr/bin/pactl")
        .args(["get-sink-volume", "@DEFAULT_SINK@"])
        .stdin(std::process::Stdio::null())
        .output()
    {
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

fn cmd_set_volume(val_str: &str) {
    let mut state = load_state().unwrap_or_default();
    let val = val_str.parse::<i32>().unwrap_or(state.volume).clamp(0, 100);
    state.volume = val;

    let _ = std::process::Command::new("/usr/bin/pactl")
        .args(["set-sink-volume", "@DEFAULT_SINK@", &format!("{}%", val)])
        .stdin(std::process::Stdio::null())
        .status();

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
                let packets = conn.read_all_notifications(std::time::Duration::from_millis(400));
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
                state.battery_case = -1;
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
            state.battery_single = -1;
        }
    }

    state.last_updated = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    state
}

fn cmd_set_anc(mode_str: &str) {
    let mode = match AncMode::from_str_name(mode_str) {
        Some(m) => m,
        None => {
            eprintln!("Gecersiz ANC modu: {}. Gecerli modlar: off, noise, transparency, adaptive", mode_str);
            std::process::exit(1);
        }
    };

    let mut state = load_state().unwrap_or_default();
    state.anc_mode = mode;

    if !state.mac.is_empty() {
        if let Ok(conn) = L2capConnection::connect(&state.mac) {
            conn.drain();
            let _ = conn.set_anc_mode(mode);
            let packets = conn.read_all_notifications(std::time::Duration::from_millis(200));
            for data in packets {
                if let Some(aap::parser::ParsedAapEvent::AncMode(m)) = aap::parser::parse_packet(&data) {
                    state.anc_mode = m;
                }
            }
        }
    }

    let _ = save_state(&state);
    println!("{{\"success\":true,\"anc_mode\":\"{}\"}}", state.anc_mode.as_str());
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

    match profile.to_lowercase().as_str() {
        "bass boost" | "bas+" => {
            let _ = std::process::Command::new("/usr/bin/pactl")
                .args(["set-sink-volume", "@DEFAULT_SINK@", "+4%"])
                .stdin(std::process::Stdio::null())
                .status();
        }
        "vocal clarity" | "vokal" => {
            let _ = std::process::Command::new("/usr/bin/pactl")
                .args(["set-sink-volume", "@DEFAULT_SINK@", "-2%"])
                .stdin(std::process::Stdio::null())
                .status();
        }
        "flat" => {
            let _ = std::process::Command::new("/usr/bin/pactl")
                .args(["set-sink-volume", "@DEFAULT_SINK@", "50%"])
                .stdin(std::process::Stdio::null())
                .status();
        }
        "beats signature" | "imza" => {
            let _ = std::process::Command::new("/usr/bin/pactl")
                .args(["set-sink-volume", "@DEFAULT_SINK@", "65%"])
                .stdin(std::process::Stdio::null())
                .status();
        }
        _ => {}
    }

    state.volume = get_system_volume();
    let _ = save_state(&state);
    println!("{{\"success\":true,\"eq_profile\":\"{}\",\"volume\":{}}}", profile, state.volume);
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

        let paths = [
            format!("/home/ozdil/.config/omarchy/plugins/ozdil.omabeats/resources/{}", wav_name),
            format!("/home/ozdil/Projects/omarchy/omarchy-omabeats/resources/{}", wav_name),
            format!("/tmp/{}", wav_name),
        ];

        if let Some(sound_path) = paths.iter().find(|p| std::path::Path::new(p).exists()) {
            let sink = format!("bluez_output.{}.1", state.mac.replace(':', "_"));
            let _ = std::process::Command::new("/usr/bin/pw-play")
                .args(["--target", &sink, sound_path])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
        }
    }
    let _ = save_state(&state);
    println!("{{\"success\":true,\"chime_active\":{:?}}}", state.chime_active);
}

fn cmd_connect(mac_opt: Option<&str>) {
    let mut state = load_state().unwrap_or_default();
    let mac = mac_opt.unwrap_or(&state.mac);

    match connect_device(mac) {
        Ok(_) => {
            state.connected = true;
            let _ = save_state(&state);
            println!("{{\"success\":true,\"connected\":true,\"mac\":\"{}\"}}", mac);
        }
        Err(e) => {
            eprintln!("{{\"success\":false,\"error\":\"{}\"}}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_disconnect(mac_opt: Option<&str>) {
    let mut state = load_state().unwrap_or_default();
    let mac = mac_opt.unwrap_or(&state.mac);

    match disconnect_device(mac) {
        Ok(_) => {
            state.connected = false;
            let _ = save_state(&state);
            println!("{{\"success\":true,\"connected\":false,\"mac\":\"{}\"}}", mac);
        }
        Err(e) => {
            eprintln!("{{\"success\":false,\"error\":\"{}\"}}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_mock(model_id: &str) {
    let state = create_mock_state(model_id);
    match save_state(&state) {
        Ok(_) => {
            println!("{{\"success\":true,\"mock_model\":\"{}\",\"connected\":true}}", state.model.display_name);
        }
        Err(e) => {
            eprintln!("{{\"success\":false,\"error\":\"{}\"}}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_set_param(key: &str, val: &str) {
    let mut state = load_state().unwrap_or_default();
    match apply_param_mutation(&mut state, key, val) {
        Ok(_) => {
            let _ = save_state(&state);
            println!("{{\"success\":true,\"key\":\"{}\",\"value\":\"{}\"}}", key, val);
        }
        Err(e) => {
            eprintln!("{{\"success\":false,\"error\":\"{}\"}}", e);
            std::process::exit(1);
        }
    }
}
