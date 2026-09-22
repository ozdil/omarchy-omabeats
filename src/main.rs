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
    let state = load_state().unwrap_or_else(|| {
        // If state file does not exist yet, sync from BlueZ
        perform_sync()
    });

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

fn perform_sync() -> BeatsState {
    let mut state = load_state().unwrap_or_default();

    // If currently in test/mock mode, maintain mock unless explicit real device sync requested
    if state.test_mode {
        return state;
    }

    let devices = discover_beats_devices();
    if let Some(dev) = devices.iter().find(|d| d.connected).or_else(|| devices.first()) {
        state.connected = dev.connected;
        state.mac = dev.mac.clone();
        state.model = dev.model.clone();
        state.rssi = dev.rssi.unwrap_or(-60);

        if let Some(bat) = dev.battery_level {
            if dev.model.has_tri_battery {
                state.battery_left = bat;
                state.battery_right = bat;
            } else {
                state.battery_single = bat;
            }
        }

        if dev.connected {
            state.codec = detect_active_codec(&dev.mac);

            // Attempt L2CAP read if possible
            if let Ok(conn) = L2capConnection::connect(&dev.mac) {
                if let Ok(data) = conn.read_packet() {
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
                            _ => {}
                        }
                    }
                }
            }
        }
    } else {
        // No beats devices found in system
        state.connected = false;
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

    if state.connected && !state.test_mode {
        if let Ok(conn) = L2capConnection::connect(&state.mac) {
            let _ = conn.set_anc_mode(mode);
        }
    }

    let _ = save_state(&state);
    println!("{{\"success\":true,\"anc_mode\":\"{}\"}}", mode.as_str());
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

    if state.connected && !state.test_mode {
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
    let _ = save_state(&state);
    println!("{{\"success\":true,\"eq_profile\":\"{}\"}}", profile);
}

fn cmd_chime(target: &str) {
    let mut state = load_state().unwrap_or_default();
    if target == "off" || target == "none" {
        state.chime_active = None;
    } else {
        state.chime_active = Some(target.to_string());
        if state.connected && !state.test_mode {
            if let Ok(conn) = L2capConnection::connect(&state.mac) {
                let _ = conn.play_chime(target);
            }
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
