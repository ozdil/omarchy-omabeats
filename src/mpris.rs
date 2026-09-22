use crate::security::spawn_isolated;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

/// Finds all active MPRIS media players on user session bus
fn get_active_players() -> Vec<String> {
    let mut players = Vec::new();
    let mut cmd = Command::new("/usr/bin/busctl");
    cmd.process_group(0);
    cmd.args(["--user", "list"]);
    cmd.stdin(Stdio::null());

    let output = match cmd.output() {
        Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
        Err(_) => return players,
    };

    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if let Some(name) = parts.first() {
            if name.starts_with("org.mpris.MediaPlayer2.") {
                players.push(name.to_string());
            }
        }
    }
    players
}

/// Triggers media pause via D-Bus MPRIS or playerctl
pub fn pause_media() {
    let players = get_active_players();
    for player in players {
        let mut cmd = Command::new("/usr/bin/busctl");
        cmd.args([
            "--user",
            "call",
            &player,
            "/org/mpris/MediaPlayer2",
            "org.mpris.MediaPlayer2.Player",
            "Pause",
        ]);
        if let Ok(mut guard) = spawn_isolated(cmd) {
            if let Some(mut child) = guard.take() {
                let _ = child.wait();
            }
        }
    }
}

/// Triggers media play/resume via D-Bus MPRIS or playerctl
pub fn resume_media() {
    let players = get_active_players();
    for player in players {
        let mut cmd = Command::new("/usr/bin/busctl");
        cmd.args([
            "--user",
            "call",
            &player,
            "/org/mpris/MediaPlayer2",
            "org.mpris.MediaPlayer2.Player",
            "Play",
        ]);
        if let Ok(mut guard) = spawn_isolated(cmd) {
            if let Some(mut child) = guard.take() {
                let _ = child.wait();
            }
        }
    }
}
