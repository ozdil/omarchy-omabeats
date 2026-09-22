use crate::aap::{AncMode, MicMode};
use crate::models::{ConnectionType, DeviceModelInfo};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeatsState {
    pub connected: bool,
    pub test_mode: bool,
    pub mac: String,
    pub model: DeviceModelInfo,
    #[serde(default)]
    pub connection_type: Option<ConnectionType>,
    #[serde(default)]
    pub is_wired: bool,
    #[serde(default)]
    pub wired_model: Option<String>,
    pub codec: String,
    pub rssi: i32,
    pub battery_left: i32,
    pub charging_left: bool,
    pub battery_right: i32,
    pub charging_right: bool,
    pub battery_case: i32,
    pub charging_case: bool,
    pub battery_single: i32,
    pub charging_single: bool,
    pub in_ear_left: bool,
    pub in_ear_right: bool,
    pub anc_mode: AncMode,
    #[serde(default)]
    pub noise_control_level: i32,
    pub mic_mode: MicMode,
    pub one_bud_anc: bool,
    pub auto_pause_enabled: bool,
    #[serde(default)]
    pub paused_by_auto_pause: bool,
    pub conversational_awareness: bool,
    pub eq_profile: String,
    pub chime_active: Option<String>,
    pub volume: i32,
    pub muted: bool,
    pub firmware_version: String,
    pub serial_number: String,
    pub last_updated: u64,
}

impl Default for BeatsState {
    fn default() -> Self {
        let model = crate::models::match_model("Beats Fit Pro", "bluetooth:v004Cp2012dD408");
        BeatsState {
            connected: true,
            test_mode: false,
            mac: "04:9D:05:DD:08:62".to_string(),
            model,
            connection_type: Some(ConnectionType::BluetoothL2cap),
            is_wired: false,
            wired_model: None,
            codec: "AAC".to_string(),
            rssi: -58,
            battery_left: 85,
            charging_left: false,
            battery_right: 80,
            charging_right: false,
            battery_case: 95,
            charging_case: true,
            battery_single: -1,
            charging_single: false,
            in_ear_left: true,
            in_ear_right: true,
            anc_mode: AncMode::NoiseCancellation,
            noise_control_level: 0,
            mic_mode: MicMode::Auto,
            one_bud_anc: true,
            auto_pause_enabled: true,
            paused_by_auto_pause: false,
            conversational_awareness: false,
            eq_profile: "Flat".to_string(),
            chime_active: None,
            volume: 60,
            muted: false,
            firmware_version: "6F8".to_string(),
            serial_number: "CC2G4000P3V9".to_string(),
            last_updated: 0,
        }
    }
}

pub fn get_state_file_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    Path::new(&home)
        .join(".local/state/omarchy/omabeats_state.json")
}
