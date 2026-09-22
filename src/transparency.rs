use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

static LOOPBACK_MODULE_ID: AtomicU32 = AtomicU32::new(0);
static TRANSPARENCY_LOCK: Mutex<()> = Mutex::new(());

/// Finds any existing module-loopback instances loaded by OmaBeats or system
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

/// Disables ambient sound passthrough and unloads any loopback modules
pub fn disable_ambient_passthrough() {
    let _guard = TRANSPARENCY_LOCK.lock().unwrap();
    let current_id = LOOPBACK_MODULE_ID.swap(0, Ordering::SeqCst);
    if current_id > 0 {
        let _ = Command::new("pactl")
            .args(["unload-module", &current_id.to_string()])
            .stdin(Stdio::null())
            .status();
    }

    // Also clean up any lingering loopback modules if present
    while let Some(lingering_id) = find_existing_loopback_id() {
        let _ = Command::new("pactl")
            .args(["unload-module", &lingering_id.to_string()])
            .stdin(Stdio::null())
            .status();
    }
}

/// Enables or updates low-latency ambient passthrough for Transparency Mode
pub fn set_ambient_passthrough(level_percent: u32) {
    let _guard = TRANSPARENCY_LOCK.lock().unwrap();

    if level_percent == 0 {
        let current_id = LOOPBACK_MODULE_ID.swap(0, Ordering::SeqCst);
        if current_id > 0 {
            let _ = Command::new("pactl")
                .args(["unload-module", &current_id.to_string()])
                .stdin(Stdio::null())
                .status();
        }
        return;
    }

    let mut current_id = LOOPBACK_MODULE_ID.load(Ordering::SeqCst);

    // If ID is 0, check if one already exists
    if current_id == 0 {
        if let Some(id) = find_existing_loopback_id() {
            current_id = id;
            LOOPBACK_MODULE_ID.store(id, Ordering::SeqCst);
        }
    }

    // If still no module, load a new low-latency loopback from default source to default sink
    if current_id == 0 {
        let output = Command::new("pactl")
            .args([
                "load-module",
                "module-loopback",
                "latency_msec=5",
                "source=@DEFAULT_SOURCE@",
                "sink=@DEFAULT_SINK@",
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

    // Calculate gain scale (e.g. 50% slider -> reasonable ambient volume, 100% -> full gain)
    // Scale 1-100 level into source-output volume if possible
    let gain = level_percent.min(100);
    let _ = Command::new("pactl")
        .args(["set-source-volume", "@DEFAULT_SOURCE@", &format!("{}%", gain)])
        .stdin(Stdio::null())
        .status();
}
