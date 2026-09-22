use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

static LOOPBACK_MODULE_ID: AtomicU32 = AtomicU32::new(0);
static TRANSPARENCY_LOCK: Mutex<()> = Mutex::new(());

/// Helper to configure secure isolated subprocess with process_group(0)
fn secure_pactl_cmd() -> Command {
    let mut cmd = Command::new("/usr/bin/pactl");
    cmd.process_group(0);
    cmd.stdin(Stdio::null());
    cmd
}

/// Finds any existing module-loopback instances in PipeWire/PulseAudio
fn find_existing_loopback_id() -> Option<u32> {
    let mut cmd = secure_pactl_cmd();
    cmd.args(["list", "modules", "short"]);

    let output = cmd.output().ok()?;
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
#[allow(dead_code)]
pub fn find_hardware_mic() -> Option<String> {
    let mut cmd = secure_pactl_cmd();
    cmd.args(["list", "sources", "short"]);

    let output = cmd.output().ok()?;
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

/// Dynamically locates the active Beats Bluetooth headphone audio sink.
/// STRICT: Never returns laptop speakers or generic fallback sinks.
#[allow(dead_code)]
pub fn find_headphone_sink() -> Option<String> {
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
    let mut cmd = secure_pactl_cmd();
    cmd.args(["list", "sink-inputs"]);

    let mut result = Vec::new();
    let output = cmd.output().ok();
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
        let mut cmd = secure_pactl_cmd();
        cmd.args(["unload-module", &current_id.to_string()]);
        let _ = cmd.status();
    }

    while let Some(lingering_id) = find_existing_loopback_id() {
        let mut cmd = secure_pactl_cmd();
        cmd.args(["unload-module", &lingering_id.to_string()]);
        let _ = cmd.status();
    }

    // Restore all active media streams to 100% full volume
    let inputs = get_sink_inputs();
    for (id, is_loopback) in inputs {
        if !is_loopback {
            let mut cmd = secure_pactl_cmd();
            cmd.args(["set-sink-input-volume", &id.to_string(), "100%"]);
            let _ = cmd.status();
        }
    }
}

/// Verifies that any active loopback is still safely connected to a Bluetooth headphone.
/// If headphone is disconnected, IMMEDIATELY unloads loopback to prevent speaker feedback!
pub fn verify_passthrough_safety() {
    if find_existing_loopback_id().is_some() {
        match find_headphone_sink() {
            Some(sink) if sink.starts_with("bluez_output") => {}
            _ => {
                disable_ambient_passthrough();
            }
        }
    }
}

/// Enables or updates low-latency ambient passthrough for Transparency Mode
/// level: 51 - 100 (where 100 is max transparency: ambient voice boosted, media ducked to 50%)
pub fn set_ambient_passthrough(level: u32) {
    let _guard = TRANSPARENCY_LOCK.lock().unwrap();

    if level <= 50 {
        drop(_guard);
        disable_ambient_passthrough();
        return;
    }

    // STRICT SAFETY CHECK: Bluetooth headphone sink MUST be present!
    // NEVER fall back to @DEFAULT_SINK@ or laptop speakers!
    let sink = match find_headphone_sink() {
        Some(s) if s.starts_with("bluez_output") => s,
        _ => {
            drop(_guard);
            disable_ambient_passthrough();
            return;
        }
    };

    let mic = match find_hardware_mic() {
        Some(m) => m,
        None => {
            drop(_guard);
            disable_ambient_passthrough();
            return;
        }
    };

    let mut current_id = LOOPBACK_MODULE_ID.load(Ordering::SeqCst);
    if current_id == 0 {
        if let Some(id) = find_existing_loopback_id() {
            current_id = id;
            LOOPBACK_MODULE_ID.store(id, Ordering::SeqCst);
        }
    }

    if current_id == 0 {
        let mut cmd = secure_pactl_cmd();
        cmd.args([
            "load-module",
            "module-loopback",
            "latency_msec=35",
            "sink_dont_move=true",
            "source_dont_move=true",
            &format!("source={}", mic),
            &format!("sink={}", sink),
        ]);

        if let Ok(out) = cmd.output() {
            if out.status.success() {
                let id_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if let Ok(id) = id_str.parse::<u32>() {
                    LOOPBACK_MODULE_ID.store(id, Ordering::SeqCst);
                }
            }
        }
    }

    // Ensure physical mic is unmuted and set to full volume
    let mut mute_cmd = secure_pactl_cmd();
    mute_cmd.args(["set-source-mute", &mic, "0"]);
    let _ = mute_cmd.status();

    let mut vol_cmd = secure_pactl_cmd();
    vol_cmd.args(["set-source-volume", &mic, "100%"]);
    let _ = vol_cmd.status();

    // Calculate transparency factor: t in [0.0, 1.0] for level in [51, 100]
    let t = ((level.saturating_sub(50)) as f32 / 50.0).clamp(0.0, 1.0);

    // Loopback ambient voice volume: from 70% up to 110%
    let loopback_vol = (70.0 + t * 40.0) as u32;

    // Media stream volume ducking: from 100% down to 50% (-18 dB duck)
    // Providing conversational awareness while playing media
    let media_vol = (100.0 - t * 50.0) as u32;

    let inputs = get_sink_inputs();
    for (id, is_loopback) in inputs {
        let mut set_vol = secure_pactl_cmd();
        if is_loopback {
            set_vol.args(["set-sink-input-volume", &id.to_string(), &format!("{}%", loopback_vol)]);
        } else {
            set_vol.args(["set-sink-input-volume", &id.to_string(), &format!("{}%", media_vol)]);
        }
        let _ = set_vol.status();
    }
}
