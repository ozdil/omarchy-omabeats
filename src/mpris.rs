use crate::security::spawn_isolated;
use std::process::Command;

/// Triggers media pause via MPRIS (playerctl or dbus)
pub fn pause_media() {
    let mut cmd = Command::new("/usr/bin/playerctl");
    cmd.arg("pause");
    if let Ok(mut guard) = spawn_isolated(cmd) {
        if let Some(mut child) = guard.take() {
            let _ = child.wait();
        }
    }
}

/// Triggers media play/resume via MPRIS (playerctl or dbus)
pub fn resume_media() {
    let mut cmd = Command::new("/usr/bin/playerctl");
    cmd.arg("play");
    if let Ok(mut guard) = spawn_isolated(cmd) {
        if let Some(mut child) = guard.take() {
            let _ = child.wait();
        }
    }
}
