#[path = "../src/aap/mod.rs"]
mod aap;
#[path = "../src/models.rs"]
mod models;
#[path = "../src/security.rs"]
mod security;
#[path = "../src/state.rs"]
mod state;
#[path = "../src/mock.rs"]
mod mock;

use aap::{commands, parser, AncMode, MicMode};
use models::{match_model, FormFactor};
use security::{sanitize_alphanumeric, validate_mac_address};

#[test]
fn test_anc_command_encoding() {
    let off_cmd = commands::set_anc_mode(AncMode::Off);
    assert_eq!(off_cmd[4], aap::CMD_CONTROL);
    assert_eq!(off_cmd[6], aap::SUB_ANC_MODE);
    assert_eq!(off_cmd[7], 1);

    let anc_cmd = commands::set_anc_mode(AncMode::NoiseCancellation);
    assert_eq!(anc_cmd[7], 2);

    let transp_cmd = commands::set_anc_mode(AncMode::Transparency);
    assert_eq!(transp_cmd[7], 3);

    let adapt_cmd = commands::set_anc_mode(AncMode::Adaptive);
    assert_eq!(adapt_cmd[7], 4);
}

#[test]
fn test_mic_command_encoding() {
    let auto_cmd = commands::set_mic_mode(MicMode::Auto);
    assert_eq!(auto_cmd[6], aap::SUB_MIC_MODE);
    assert_eq!(auto_cmd[7], 0);

    let right_cmd = commands::set_mic_mode(MicMode::Right);
    assert_eq!(right_cmd[7], 1);

    let left_cmd = commands::set_mic_mode(MicMode::Left);
    assert_eq!(left_cmd[7], 2);
}

#[test]
fn test_chime_command_encoding() {
    let left = commands::play_chime_command("left");
    assert_eq!(left[6], aap::SUB_CHIME);
    assert_eq!(left[7], 1);

    let right = commands::play_chime_command("right");
    assert_eq!(right[7], 2);

    let both = commands::play_chime_command("both");
    assert_eq!(both[7], 3);
}

#[test]
fn test_battery_payload_parsing() {
    // Left: 85% (not charging), Right: 90% (charging), Case: 100% (charging)
    let payload = vec![85, 0, 90, 1, 100, 1];
    let report = parser::parse_battery_payload(&payload);

    let left = report.left.expect("Left battery missing");
    assert_eq!(left.level, 85);
    assert!(!left.charging);

    let right = report.right.expect("Right battery missing");
    assert_eq!(right.level, 90);
    assert!(right.charging);

    let case = report.case.expect("Case battery missing");
    assert_eq!(case.level, 100);
    assert!(case.charging);
}

#[test]
fn test_single_battery_payload_parsing() {
    // Over-ear Beats Studio Pro: 74% (charging)
    let payload = vec![74, 1];
    let report = parser::parse_battery_payload(&payload);

    let single = report.single.expect("Single battery missing");
    assert_eq!(single.level, 74);
    assert!(single.charging);
}

#[test]
fn test_ear_detection_parsing() {
    // AAP Standard: Left in-ear (0x00), Right out-of-ear (0x01)
    let payload = vec![0x00, 0x01];
    let report = parser::parse_ear_detection_payload(&payload);
    assert!(report.left_in_ear);
    assert!(!report.right_in_ear);
}

#[test]
fn test_model_matching() {
    let fit_pro = match_model("Beats Fit Pro", "bluetooth:v004Cp2012dD408");
    assert_eq!(fit_pro.model_id, "beats_fit_pro");
    assert_eq!(fit_pro.form_factor, FormFactor::Earbuds);
    assert!(fit_pro.has_anc);
    assert!(fit_pro.has_tri_battery);

    let studio_pro = match_model("Beats Studio Pro", "");
    assert_eq!(studio_pro.model_id, "beats_studio_pro");
    assert_eq!(studio_pro.form_factor, FormFactor::OverEar);
    assert!(!studio_pro.has_tri_battery);

    let solo_4 = match_model("Beats Solo 4", "");
    assert_eq!(solo_4.model_id, "beats_solo_4");
    assert!(!solo_4.has_anc);
}

#[test]
fn test_security_validations() {
    assert!(validate_mac_address("04:9D:05:DD:08:62"));
    assert!(validate_mac_address("AA:BB:CC:11:22:33"));
    assert!(!validate_mac_address("04:9D:05:DD:08:6Z")); // Non-hex
    assert!(!validate_mac_address("04:9D:05:DD:08")); // Too short
    assert!(!validate_mac_address("04:9D:05:DD:08:62; rm -rf /")); // Injection attempt

    let clean = sanitize_alphanumeric("Beats-Fit_Pro.1:test <script>", 64);
    assert_eq!(clean, "Beats-Fit_Pro.1:test script");
}

#[test]
fn test_mock_simulator_mutation() {
    let mut state = mock::create_mock_state("beats_fit_pro");
    assert_eq!(state.model.model_id, "beats_fit_pro");
    assert!(state.connected);

    mock::apply_param_mutation(&mut state, "anc", "transparency").unwrap();
    assert_eq!(state.anc_mode, AncMode::Transparency);

    mock::apply_param_mutation(&mut state, "bat_left", "42").unwrap();
    assert_eq!(state.battery_left, 42);

    mock::apply_param_mutation(&mut state, "ear_left", "false").unwrap();
    assert!(!state.in_ear_left);

    mock::apply_param_mutation(&mut state, "model", "beats_studio_pro").unwrap();
    assert_eq!(state.model.model_id, "beats_studio_pro");
    assert_eq!(state.model.form_factor, FormFactor::OverEar);
}

#[test]
fn test_all_historical_and_modern_beats_models() {
    // 1. Historical Wired Models
    let ep = models::match_model("Beats EP", "");
    assert_eq!(ep.model_id, "beats_ep");
    assert_eq!(ep.default_connection, models::ConnectionType::AnalogJack);
    assert!(ep.is_wired_only);

    let pro = models::match_model("Beats Pro", "");
    assert_eq!(pro.model_id, "beats_pro");
    assert_eq!(pro.form_factor, FormFactor::OverEar);

    let urbeats = models::match_model("urBeats 3", "");
    assert_eq!(urbeats.model_id, "urbeats_3");

    let solo_hd = models::match_model("Beats Solo HD", "");
    assert_eq!(solo_hd.model_id, "beats_solo_hd");

    let mixr = models::match_model("Beats Mixr", "");
    assert_eq!(mixr.model_id, "beats_mixr");

    // 2. Modern 2023-2024 Models
    let studio_pro = models::match_model("Beats Studio Pro", "");
    assert_eq!(studio_pro.model_id, "beats_studio_pro");
    assert!(studio_pro.has_usb_audio);
    assert!(studio_pro.has_anc);

    let solo_4 = models::match_model("Beats Solo 4", "");
    assert_eq!(solo_4.model_id, "beats_solo_4");
    assert!(solo_4.has_usb_audio);
    assert_eq!(solo_4.form_factor, FormFactor::OnEar);

    let solo_buds = models::match_model("Beats Solo Buds", "");
    assert_eq!(solo_buds.model_id, "beats_solo_buds");
    assert_eq!(solo_buds.form_factor, FormFactor::Earbuds);

    let pill = models::match_model("Beats Pill", "");
    assert_eq!(pill.model_id, "beats_pill_2024");
    assert_eq!(pill.form_factor, FormFactor::Speaker);
}

#[test]
fn test_future_proof_beats_discovery() {
    // Unreleased / Future models
    let future_buds = models::match_model("Beats Apex Buds Ultra 2028", "bluetooth:v004Cp9999");
    assert_eq!(future_buds.model_id, "beats_future_device");
    assert_eq!(future_buds.form_factor, FormFactor::Earbuds);
    assert!(future_buds.has_anc);
    assert!(future_buds.has_in_ear);

    let future_headphones = models::match_model("Beats Solo 6 Wireless Over-Ear", "");
    assert_eq!(future_headphones.model_id, "beats_future_device");
    assert_eq!(future_headphones.form_factor, FormFactor::OnEar);

    let future_speaker = models::match_model("Beats Mega Pill Extreme Speaker", "");
    assert_eq!(future_speaker.model_id, "beats_future_device");
    assert_eq!(future_speaker.form_factor, FormFactor::Speaker);
}
