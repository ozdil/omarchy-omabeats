use crate::models::{match_model, DeviceModelInfo};
use crate::security::{reap_process_group, validate_mac_address};
use serde::{Deserialize, Serialize};
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredDevice {
    pub mac: String,
    pub name: String,
    pub alias: String,
    pub connected: bool,
    pub paired: bool,
    pub trusted: bool,
    pub rssi: Option<i32>,
    pub battery_level: Option<i32>,
    pub modalias: String,
    pub model: DeviceModelInfo,
}

/// Lists all paired Beats devices detected via BlueZ / bluetoothctl
pub fn discover_beats_devices() -> Vec<DiscoveredDevice> {
    let mut devices = Vec::new();

    let mut cmd = Command::new("/usr/bin/bluetoothctl");
    cmd.process_group(0);
    cmd.arg("devices");
    cmd.stdin(Stdio::null());

    let output = match cmd.output() {
        Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
        Err(_) => return devices,
    };

    for line in output.lines() {
        // Format: Device 04:9D:05:DD:08:62 Beats Fit Pro
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 && parts[0] == "Device" {
            let mac = parts[1];
            if !validate_mac_address(mac) {
                continue;
            }

            if let Some(dev) = inspect_device(mac) {
                let name_lower = dev.name.to_lowercase();
                let alias_lower = dev.alias.to_lowercase();
                let is_beats = name_lower.contains("beats")
                    || alias_lower.contains("beats")
                    || name_lower.contains("powerbeats")
                    || alias_lower.contains("powerbeats")
                    || name_lower.contains("urbeats")
                    || alias_lower.contains("urbeats")
                    || dev.modalias.to_lowercase().contains("v004c")
                    || (dev.model.model_id != "unknown" && dev.model.model_id != "generic_audio");

                if is_beats {
                    devices.push(dev);
                }
            }
        }
    }

    devices
}

/// Inspects a specific Bluetooth device by MAC address using bluetoothctl info
pub fn inspect_device(mac: &str) -> Option<DiscoveredDevice> {
    if !validate_mac_address(mac) {
        return None;
    }

    let mut cmd = Command::new("/usr/bin/bluetoothctl");
    cmd.process_group(0);
    cmd.args(["info", mac]);
    cmd.stdin(Stdio::null());

    let output = match cmd.output() {
        Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
        Err(_) => return None,
    };

    let mut name = String::new();
    let mut alias = String::new();
    let mut connected = false;
    let mut paired = false;
    let mut trusted = false;
    let mut rssi = None;
    let mut battery_level = None;
    let mut modalias = String::new();

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Name:") {
            name = trimmed["Name:".len()..].trim().to_string();
        } else if trimmed.starts_with("Alias:") {
            alias = trimmed["Alias:".len()..].trim().to_string();
        } else if trimmed.starts_with("Connected:") {
            connected = trimmed["Connected:".len()..].trim().eq_ignore_ascii_case("yes");
        } else if trimmed.starts_with("Paired:") {
            paired = trimmed["Paired:".len()..].trim().eq_ignore_ascii_case("yes");
        } else if trimmed.starts_with("Trusted:") {
            trusted = trimmed["Trusted:".len()..].trim().eq_ignore_ascii_case("yes");
        } else if trimmed.starts_with("RSSI:") {
            let val = trimmed["RSSI:".len()..].trim();
            rssi = val.parse::<i32>().ok();
        } else if trimmed.starts_with("Battery Percentage:") {
            let val_str = trimmed["Battery Percentage:".len()..].trim();
            if let Some(open) = val_str.find('(') {
                if let Some(close) = val_str.find(')') {
                    battery_level = val_str[open + 1..close].trim().parse::<i32>().ok();
                }
            } else {
                let cleaned = val_str.trim_end_matches('%');
                battery_level = cleaned.parse::<i32>().ok();
            }
        } else if trimmed.starts_with("Modalias:") {
            modalias = trimmed["Modalias:".len()..].trim().to_string();
        }
    }

    if name.is_empty() && alias.is_empty() {
        return None;
    }

    let dev_name = if !alias.is_empty() { &alias } else { &name };
    let model = match_model(dev_name, &modalias);

    Some(DiscoveredDevice {
        mac: mac.to_string(),
        name,
        alias,
        connected,
        paired,
        trusted,
        rssi,
        battery_level,
        modalias,
        model,
    })
}

/// Connects to a device via bluetoothctl
pub fn connect_device(mac: &str) -> Result<(), String> {
    if !validate_mac_address(mac) {
        return Err("Invalid MAC address".to_string());
    }

    let mut cmd = Command::new("/usr/bin/bluetoothctl");
    cmd.process_group(0);
    cmd.args(["connect", mac]);
    cmd.stdin(Stdio::null());

    let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn bluetoothctl: {}", e))?;
    let status = child.wait().map_err(|e| format!("Process error: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        reap_process_group(&mut child);
        Err("Failed to connect device".to_string())
    }
}

/// Disconnects a device via bluetoothctl
pub fn disconnect_device(mac: &str) -> Result<(), String> {
    if !validate_mac_address(mac) {
        return Err("Invalid MAC address".to_string());
    }

    let mut cmd = Command::new("/usr/bin/bluetoothctl");
    cmd.process_group(0);
    cmd.args(["disconnect", mac]);
    cmd.stdin(Stdio::null());

    let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn bluetoothctl: {}", e))?;
    let status = child.wait().map_err(|e| format!("Process error: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        reap_process_group(&mut child);
        Err("Failed to disconnect device".to_string())
    }
}

/// Detects the active audio codec for the Bluetooth device via pactl / PipeWire
pub fn detect_active_codec(mac: &str) -> String {
    let sink_name_fragment = mac.replace(':', "_");

    let mut cmd = Command::new("/usr/bin/pactl");
    cmd.process_group(0);
    cmd.args(["list", "sinks"]);
    cmd.stdin(Stdio::null());

    let output = match cmd.output() {
        Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
        Err(_) => return "AAC".to_string(),
    };

    let mut in_target_sink = false;
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Name:") {
            in_target_sink = trimmed.contains(&sink_name_fragment);
        } else if in_target_sink && trimmed.starts_with("bluetooth.codec =") {
            let parts: Vec<&str> = trimmed.split('=').collect();
            if parts.len() >= 2 {
                let codec = parts[1].trim().trim_matches('"').to_uppercase();
                return codec;
            }
        }
    }

    "AAC".to_string()
}
