use crate::aap::{AncMode, MicMode};
use crate::models::{match_model, FormFactor};
use crate::security::{atomic_write_secure, safe_read_file_limited};
use crate::state::{get_state_file_path, BeatsState};
use std::time::{SystemTime, UNIX_EPOCH};

/// Generates a mock Beats state for the specified model for test and development mode
pub fn create_mock_state(model_id_or_name: &str) -> BeatsState {
    let model = match_model(model_id_or_name, "");
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

    let is_over_ear = model.form_factor == FormFactor::OverEar;
    let is_neckband = model.form_factor == FormFactor::Neckband;

    let (left, right, case, single) = if is_over_ear || is_neckband {
        (-1, -1, -1, 78)
    } else {
        (85, 80, 95, -1)
    };

    let anc = if model.has_anc {
        AncMode::NoiseCancellation
    } else {
        AncMode::Off
    };

    BeatsState {
        connected: true,
        test_mode: true,
        mac: "04:9D:05:DD:08:62".to_string(),
        connection_type: Some(model.default_connection),
        is_wired: model.is_wired_only,
        wired_model: if model.is_wired_only { Some(model.model_id.clone()) } else { None },
        model,
        codec: "AAC".to_string(),
        rssi: -54,
        battery_left: left,
        charging_left: false,
        battery_right: right,
        charging_right: false,
        battery_case: case,
        charging_case: true,
        battery_single: single,
        charging_single: false,
        in_ear_left: !is_over_ear,
        in_ear_right: !is_over_ear,
        anc_mode: anc,
        noise_control_level: if anc == AncMode::NoiseCancellation { 0 } else { 50 },
        mic_mode: MicMode::Auto,
        one_bud_anc: true,
        auto_pause_enabled: true,
        paused_by_auto_pause: false,
        conversational_awareness: false,
        eq_profile: "Flat".to_string(),
        chime_active: None,
        volume: 65,
        muted: false,
        firmware_version: "6F8".to_string(),
        serial_number: "CC2G4000P3V9".to_string(),
        last_updated: now,
    }
}

/// Modifies a state parameter based on key and value strings
pub fn apply_param_mutation(state: &mut BeatsState, key: &str, value: &str) -> Result<(), String> {
    match key.to_lowercase().as_str() {
        "model" => {
            state.model = match_model(value, "");
            let is_over_ear = state.model.form_factor == FormFactor::OverEar;
            let is_neckband = state.model.form_factor == FormFactor::Neckband;
            if is_over_ear || is_neckband {
                state.battery_single = 80;
                state.battery_left = -1;
                state.battery_right = -1;
                state.battery_case = -1;
                state.in_ear_left = false;
                state.in_ear_right = false;
            } else {
                state.battery_single = -1;
                if state.battery_left < 0 { state.battery_left = 85; }
                if state.battery_right < 0 { state.battery_right = 80; }
                if state.battery_case < 0 { state.battery_case = 95; }
                state.in_ear_left = true;
                state.in_ear_right = true;
            }
        }
        "anc" | "anc_mode" => {
            if let Some(mode) = AncMode::from_str_name(value) {
                state.anc_mode = mode;
            } else {
                return Err(format!("Unknown ANC mode: {}", value));
            }
        }
        "mic" | "mic_mode" => {
            if let Some(mode) = MicMode::from_str_name(value) {
                state.mic_mode = mode;
            } else {
                return Err(format!("Unknown mic mode: {}", value));
            }
        }
        "battery_left" | "bat_left" => {
            if let Ok(v) = value.parse::<i32>() {
                state.battery_left = v.clamp(0, 100);
            }
        }
        "battery_right" | "bat_right" => {
            if let Ok(v) = value.parse::<i32>() {
                state.battery_right = v.clamp(0, 100);
            }
        }
        "battery_case" | "bat_case" => {
            if let Ok(v) = value.parse::<i32>() {
                state.battery_case = v.clamp(0, 100);
            }
        }
        "battery_single" | "bat_single" => {
            if let Ok(v) = value.parse::<i32>() {
                state.battery_single = v.clamp(0, 100);
            }
        }
        "charging_left" => {
            state.charging_left = value.eq_ignore_ascii_case("true") || value == "1";
        }
        "charging_right" => {
            state.charging_right = value.eq_ignore_ascii_case("true") || value == "1";
        }
        "charging_case" => {
            state.charging_case = value.eq_ignore_ascii_case("true") || value == "1";
        }
        "in_ear_left" | "ear_left" => {
            state.in_ear_left = value.eq_ignore_ascii_case("true") || value == "1";
        }
        "in_ear_right" | "ear_right" => {
            state.in_ear_right = value.eq_ignore_ascii_case("true") || value == "1";
        }
        "connected" => {
            state.connected = value.eq_ignore_ascii_case("true") || value == "1";
        }
        "test_mode" => {
            state.test_mode = value.eq_ignore_ascii_case("true") || value == "1";
        }
        "one_bud_anc" => {
            state.one_bud_anc = value.eq_ignore_ascii_case("true") || value == "1";
        }
        "auto_pause" => {
            state.auto_pause_enabled = value.eq_ignore_ascii_case("true") || value == "1";
        }
        "ca" | "conversational_awareness" => {
            state.conversational_awareness = value.eq_ignore_ascii_case("true") || value == "1";
        }
        "eq" | "eq_profile" => {
            state.eq_profile = value.to_string();
        }
        "chime" => {
            if value == "none" || value == "off" || value.is_empty() {
                state.chime_active = None;
            } else {
                state.chime_active = Some(value.to_string());
            }
        }
        _ => return Err(format!("Unknown configuration key: {}", key)),
    }

    state.last_updated = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    Ok(())
}

/// Atomically persists the current state to the Omarchy runtime state directory
pub fn save_state(state: &BeatsState) -> Result<(), String> {
    let path = get_state_file_path();
    let json = serde_json::to_string_pretty(state).map_err(|e| format!("JSON error: {}", e))?;
    atomic_write_secure(&path, &json).map_err(|e| format!("Write error: {}", e))
}

/// Safely reads the persistent Beats state from the Omarchy runtime state directory
pub fn load_state() -> Option<BeatsState> {
    let path = get_state_file_path();
    if !path.exists() {
        return None;
    }

    match safe_read_file_limited(&path) {
        Ok(content) => serde_json::from_str(&content).ok(),
        Err(_) => None,
    }
}
