use super::*;

/// Initial L2CAP handshake packet
pub const HANDSHAKE: [u8; 16] = [
    0x00, 0x00, 0x04, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// Host features capability advertisement packet
pub const SET_FEATURES: [u8; 14] = [
    0x04, 0x00, 0x04, 0x00, 0x4D, 0x00, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// Notification subscription packet for battery, ANC, ear detection, and status updates
pub const SUBSCRIBE_NOTIFICATIONS: [u8; 10] = [
    0x04, 0x00, 0x04, 0x00, 0x0F, 0x00, 0xFF, 0xFF, 0xFF, 0xFF,
];

/// Enables all listening modes in firmware rotation (Off + Noise + Transparency + Adaptive)
/// Bitmask: 0x01=Off, 0x02=Noise, 0x04=Transparency, 0x08=Adaptive (0x0F = all enabled)
pub const ENABLE_ALL_LISTENING_MODES: [u8; 11] = [
    0x04, 0x00, 0x04, 0x00, 0x09, 0x00, 0x1A, 0x0F, 0x00, 0x00, 0x00,
];

/// Builds an AAP control command packet
pub fn control_command(sub_cmd: u8, value: u8) -> Vec<u8> {
    vec![
        HEADER[0],
        HEADER[1],
        HEADER[2],
        HEADER[3],
        CMD_CONTROL,
        0x00,
        sub_cmd,
        value,
        0x00,
        0x00,
        0x00,
    ]
}

/// Builds packet to set Active Noise Cancellation (ANC) mode
pub fn set_anc_mode(mode: AncMode) -> Vec<u8> {
    control_command(SUB_ANC_MODE, mode as u8)
}

/// Builds packet to set microphone routing mode (Auto/Left/Right)
pub fn set_mic_mode(mode: MicMode) -> Vec<u8> {
    control_command(SUB_MIC_MODE, mode as u8)
}

/// Builds packet to toggle One-Bud ANC
pub fn set_one_bud_anc(enable: bool) -> Vec<u8> {
    let val = if enable { 0x01 } else { 0x02 };
    control_command(SUB_ONE_BUD_ANC, val)
}

/// Builds packet to toggle In-Ear Detection
pub fn set_in_ear_detection(enable: bool) -> Vec<u8> {
    let val = if enable { 0x01 } else { 0x02 };
    control_command(SUB_EAR_DETECTION, val)
}

/// Builds packet to trigger Chime / Find My sound
pub fn play_chime_command(target: &str) -> Vec<u8> {
    let val = match target.to_lowercase().as_str() {
        "left" | "sol" => 0x01,
        "right" | "sag" => 0x02,
        _ => 0x03, // Both
    };
    control_command(SUB_CHIME, val)
}

