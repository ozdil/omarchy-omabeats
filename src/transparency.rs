use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

static LOOPBACK_MODULE_ID: AtomicU32 = AtomicU32::new(0);
static TRANSPARENCY_LOCK: Mutex<()> = Mutex::new(());

/// Finds any existing module-loopback instances in PipeWire/PulseAudio
fn find_existing_loopback_id() -> Option<u32> {
    let output = Command::new("pactl")
        .args(["list", "modules", "short"])
        .stdin(Stdio::null())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[1] == "module-loopback" {
            if let Ok(id) = parts[0].parse::<u32>() {
                return Some(id);
            }
        }
    }
    None
}

/// Dynamically locates the active physical hardware microphone (laptop internal mic / USB mic),
/// specifically avoiding monitor sinks and avoiding the silent Bluetooth A2DP bluez_input source.
pub fn find_hardware_mic() -> Option<String> {
    let output = Command::new("pactl")
        .args(["list", "sources", "short"])
        .stdin(Stdio::null())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut candidates = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let name = parts[1];
            if name.ends_with(".monitor") || name.starts_with("bluez_input") {
                continue;
            }
            if name.starts_with("alsa_input") || name.starts_with("usb_input") {
                candidates.push(name.to_string());
            }
        }
    }

    // Prioritize built-in digital / internal microphone array
    for c in &candidates {
        let lower = c.to_lowercase();
        if lower.contains("mic1") || lower.contains("digital") || lower.contains("internal") {
            return Some(c.clone());
        }
    }

    // Fallback to first non-bluez hardware candidate
    if let Some(first) = candidates.into_iter().next() {
        return Some(first);
    }

    None
}

/// Dynamically locates the Beats headphone audio sink
pub fn find_headphone_sink() -> Option<String> {
    let output = Command::new("pactl")
        .args(["list", "sinks", "short"])
        .stdin(Stdio::null())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let name = parts[1];
            if name.starts_with("bluez_output") {
                return Some(name.to_string());
            }
        }
    }

    None
}

/// Returns a list of active sink inputs: (id, is_loopback)
pub fn get_sink_inputs() -> Vec<(u32, bool)> {
    let output = Command::new("pactl")
        .args(["list", "sink-inputs"])
        .stdin(Stdio::null())
        .output()
        .ok();

    let mut result = Vec::new();
    let Some(out) = output else { return result; };
    if !out.status.success() {
        return result;
    }

    let text = String::from_utf8_lossy(&out.stdout);
    for block in text.split("Sink Input #") {
        let lines: Vec<&str> = block.lines().collect();
        if lines.is_empty() {
            continue;
        }
        let id_str = lines[0].trim();
        if let Ok(id) = id_str.parse::<u32>() {
            let is_loopback = block.to_lowercase().contains("loopback");
            result.push((id, is_loopback));
        }
    }

    result
}

/// Disables ambient sound passthrough, unloads loopback, and restores media volume to 100%
pub fn disable_ambient_passthrough() {
    let _guard = TRANSPARENCY_LOCK.lock().unwrap();

    let current_id = LOOPBACK_MODULE_ID.swap(0, Ordering::SeqCst);
    if current_id > 0 {
        let _ = Command::new("pactl")
            .args(["unload-module", &current_id.to_string()])
            .stdin(Stdio::null())
            .status();
    }

    while let Some(lingering_id) = find_existing_loopback_id() {
        let _ = Command::new("pactl")
            .args(["unload-module", &lingering_id.to_string()])
            .stdin(Stdio::null())
            .status();
    }

    // Restore all active media streams to 100% full volume
    let inputs = get_sink_inputs();
    for (id, is_loopback) in inputs {
        if !is_loopback {
            let _ = Command::new("pactl")
                .args(["set-sink-input-volume", &id.to_string(), "100%"])
                .stdin(Stdio::null())
                .status();
        }
    }
}

/// Enables or updates low-latency ambient passthrough for Transparency Mode
/// level: 51 - 100 (where 100 is max transparency: ambient boosted to 140%, media ducked to 50%)
pub fn set_ambient_passthrough(level: u32) {
    let _guard = TRANSPARENCY_LOCK.lock().unwrap();

    if level <= 50 {
        drop(_guard);
        disable_ambient_passthrough();
        return;
    }

    let mut current_id = LOOPBACK_MODULE_ID.load(Ordering::SeqCst);
    if current_id == 0 {
        if let Some(id) = find_existing_loopback_id() {
            current_id = id;
            LOOPBACK_MODULE_ID.store(id, Ordering::SeqCst);
        }
    }

    let mic = find_hardware_mic().unwrap_or_else(|| "@DEFAULT_SOURCE@".to_string());
    let sink = find_headphone_sink().unwrap_or_else(|| "@DEFAULT_SINK@".to_string());

    if current_id == 0 {
        let output = Command::new("pactl")
            .args([
                "load-module",
                "module-loopback",
                "latency_msec=5",
                &format!("source={}", mic),
                &format!("sink={}", sink),
            ])
            .stdin(Stdio::null())
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                let id_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if let Ok(id) = id_str.parse::<u32>() {
                    LOOPBACK_MODULE_ID.store(id, Ordering::SeqCst);
                }
            }
        }
    }

    // Ensure physical mic is unmuted and set to full volume
    let _ = Command::new("pactl")
        .args(["set-source-mute", &mic, "0"])
        .stdin(Stdio::null())
        .status();
    let _ = Command::new("pactl")
        .args(["set-source-volume", &mic, "100%"])
        .stdin(Stdio::null())
        .status();

    // Calculate transparency factor: t in [0.0, 1.0] for level in [51, 100]
    let t = ((level.saturating_sub(50)) as f32 / 50.0).clamp(0.0, 1.0);

    // Loopback ambient voice volume: from 70% up to 140% (+8.77 dB active boost)
    let loopback_vol = (70.0 + t * 70.0) as u32;

    // Media stream volume ducking: from 100% down to 50% (-18 dB duck)
    // Providing exactly the 50%-50% balance requested by user
    let media_vol = (100.0 - t * 50.0) as u32;

    let inputs = get_sink_inputs();
    for (id, is_loopback) in inputs {
        if is_loopback {
            let _ = Command::new("pactl")
                .args(["set-sink-input-volume", &id.to_string(), &format!("{}%", loopback_vol)])
                .stdin(Stdio::null())
                .status();
        } else {
            let _ = Command::new("pactl")
                .args(["set-sink-input-volume", &id.to_string(), &format!("{}%", media_vol)])
                .stdin(Stdio::null())
                .status();
        }
    }
}
