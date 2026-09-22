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

/// Verifies that no rogue software loopback is running in the audio graph.
/// Unloads any lingering loopback to prevent speaker feedback and buffer overflows.
pub fn verify_passthrough_safety() {
    if find_existing_loopback_id().is_some() {
        disable_ambient_passthrough();
    }
}

/// For Beats headphones (Beats Fit Pro, Studio Pro, Solo Pro, etc.),
/// Transparency mode is handled entirely by hardware DSP on the Apple H1 / Beats chip via AAP.
/// Software microphone loopback from the laptop's internal mic is strictly prevented to avoid
/// acoustic feedback loops, buffer overflows, and Bluetooth A2DP audio dropouts.
pub fn set_ambient_passthrough(_level: u32) {
    let _guard = TRANSPARENCY_LOCK.lock().unwrap();

    // Ensure any stray software loopback is cleaned up immediately
    while let Some(lingering_id) = find_existing_loopback_id() {
        let mut cmd = secure_pactl_cmd();
        cmd.args(["unload-module", &lingering_id.to_string()]);
        let _ = cmd.status();
    }

    // Ensure all media streams (Spotify, etc.) maintain 100% volume
    let inputs = get_sink_inputs();
    for (id, is_loopback) in inputs {
        if !is_loopback {
            let mut cmd = secure_pactl_cmd();
            cmd.args(["set-sink-input-volume", &id.to_string(), "100%"]);
            let _ = cmd.status();
        }
    }
}
