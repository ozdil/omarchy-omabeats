use crate::models::{find_model, ConnectionType, DeviceModelInfo};
use std::fs::File;
use std::io::Read;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WiredDeviceInfo {
    pub model: DeviceModelInfo,
    pub connection_type: ConnectionType,
    pub sink_name: String,
    pub is_usb: bool,
}

/// Helper for secure isolated pactl execution
fn secure_pactl_cmd() -> Command {
    let mut cmd = Command::new("/usr/bin/pactl");
    cmd.process_group(0);
    cmd.stdin(Stdio::null());
    cmd
}

/// Scans PipeWire / ALSA sinks for USB-connected Apple/Beats devices (Lossless Audio)
pub fn detect_usb_beats_device() -> Option<WiredDeviceInfo> {
    let mut cmd = secure_pactl_cmd();
    cmd.args(["list", "sinks", "short"]);

    let output = cmd.output().ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let sink_name = parts[1];
            let lower = sink_name.to_lowercase();

            // Check if it is an Apple/Beats USB Audio device
            if lower.contains("usb") && (lower.contains("beats") || lower.contains("apple")) {
                let model = if lower.contains("studio_pro") || lower.contains("studiopro") {
                    find_model("beats_studio_pro")
                } else if lower.contains("solo_4") || lower.contains("solo4") {
                    find_model("beats_solo_4")
                } else if lower.contains("pill") {
                    find_model("beats_pill_2024")
                } else {
                    crate::models::discover_future_beats_model("Beats USB Audio")
                };

                return Some(WiredDeviceInfo {
                    model,
                    connection_type: ConnectionType::UsbLossless,
                    sink_name: sink_name.to_string(),
                    is_usb: true,
                });
            }
        }
    }

    // Also inspect Linux USB bus directly (/sys/bus/usb/devices) for Apple Inc (05ac) Beats devices
    scan_sysfs_usb_beats()
}

/// Inspects /sys/bus/usb/devices for Apple Vendor ID 05ac and Beats product strings
fn scan_sysfs_usb_beats() -> Option<WiredDeviceInfo> {
    let usb_dir = Path::new("/sys/bus/usb/devices");
    if !usb_dir.exists() {
        return None;
    }

    let entries = std::fs::read_dir(usb_dir).ok()?;
    for entry in entries.flatten() {
        let dev_path = entry.path();
        let vendor_file = dev_path.join("idVendor");
        let _product_file = dev_path.join("idProduct");
        let name_file = dev_path.join("product");

        if vendor_file.exists() && name_file.exists() {
            let mut vendor_str = String::new();
            if let Ok(mut f) = File::open(&vendor_file) {
                let _ = (&mut f).take(32).read_to_string(&mut vendor_str);
            }

            let mut prod_name = String::new();
            if let Ok(mut f) = File::open(&name_file) {
                let _ = (&mut f).take(256).read_to_string(&mut prod_name);
            }

            let trimmed_vendor = vendor_str.trim().to_lowercase();
            let trimmed_name = prod_name.trim();
            let lower_name = trimmed_name.to_lowercase();

            // 05ac is Apple Inc.
            if trimmed_vendor == "05ac" && (lower_name.contains("beats") || lower_name.contains("studio") || lower_name.contains("solo") || lower_name.contains("pill")) {
                let model = crate::models::match_model(trimmed_name, "");
                return Some(WiredDeviceInfo {
                    model,
                    connection_type: ConnectionType::UsbLossless,
                    sink_name: "usb_apple_beats".to_string(),
                    is_usb: true,
                });
            }
        }
    }

    None
}

/// Checks if an analog 3.5mm headphone jack is currently plugged into the computer
#[allow(dead_code)]
pub fn is_analog_headphone_plugged() -> bool {
    let mut cmd = secure_pactl_cmd();
    cmd.args(["list", "sinks"]);

    let output = match cmd.output() {
        Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
        Err(_) => return false,
    };

    // Look for active port: [Out] Headphones or analog-output-headphones with status available: yes
    let lower = output.to_lowercase();
    if lower.contains("analog-output-headphones") || lower.contains("[out] headphones") {
        for block in output.split("Ports:") {
            if block.contains("analog-output-headphones") || block.contains("[Out] Headphones") {
                if block.contains("available: yes") || block.contains("priority") {
                    return true;
                }
            }
        }
    }

    false
}

/// Recommended acoustic EQ presets tailored specifically for classic/wired Beats models
pub fn get_wired_model_eq_preset(model_id: &str) -> &'static str {
    match model_id {
        "beats_ep" => "Bass Boost",
        "beats_pro" => "Studio",
        "beats_solo_hd" => "Bass Boost",
        "beats_solo_2_wired" => "Rock",
        "urbeats_3" | "urbeats_2" | "urbeats_1" => "Vocal",
        "beats_studio_1" | "beats_studio_2_wired" => "Club",
        "beats_tour_1" | "beats_tour_2" => "Electronic",
        "beats_mixr" => "Bass Boost",
        _ => "Flat",
    }
}
