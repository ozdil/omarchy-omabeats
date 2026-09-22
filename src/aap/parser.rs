use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryEntry {
    pub level: i32,
    pub charging: bool,
    pub connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryReport {
    pub left: Option<BatteryEntry>,
    pub right: Option<BatteryEntry>,
    pub case: Option<BatteryEntry>,
    pub single: Option<BatteryEntry>, // For over-ear / neckband headsets
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarDetectionReport {
    pub left_in_ear: bool,
    pub right_in_ear: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfoReport {
    pub model_name: String,
    pub firmware: String,
    pub serial: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParsedAapEvent {
    HandshakeAck,
    FeaturesAck,
    Battery(BatteryReport),
    AncMode(AncMode),
    EarDetection(EarDetectionReport),
    ConversationalAwareness(bool),
    DeviceInfo(DeviceInfoReport),
    RawNotification(u8, Vec<u8>),
}

/// Parses an incoming AAP binary packet from the L2CAP socket
pub fn parse_packet(data: &[u8]) -> Option<ParsedAapEvent> {
    if data.len() < 4 {
        return None;
    }

    // Handshake ACK pattern: 01 00 04 00
    if data.len() >= 4 && data[0] == 0x01 && data[1] == 0x00 && data[2] == 0x04 && data[3] == 0x00 {
        return Some(ParsedAapEvent::HandshakeAck);
    }

    // Must have standard header [0x04, 0x00, 0x04, 0x00]
    if data[0..4] != HEADER {
        return None;
    }

    if data.len() < 6 {
        return None;
    }

    let cmd = data[4];
    let payload = &data[6..];

    match cmd {
        // Features ACK (0x2B)
        0x2B => Some(ParsedAapEvent::FeaturesAck),

        CMD_BATTERY => {
            let report = parse_battery_payload(payload);
            Some(ParsedAapEvent::Battery(report))
        }

        CMD_EAR_DETECTION => {
            let report = parse_ear_detection_payload(payload);
            Some(ParsedAapEvent::EarDetection(report))
        }

        CMD_CONTROL => {
            if payload.len() >= 2 {
                let sub_cmd = payload[0];
                let value = payload[1];
                if sub_cmd == SUB_ANC_MODE {
                    if let Some(mode) = AncMode::from_u8(value) {
                        return Some(ParsedAapEvent::AncMode(mode));
                    }
                } else if sub_cmd == SUB_EAR_DETECTION {
                    let enabled = value == 0x01;
                    return Some(ParsedAapEvent::ConversationalAwareness(enabled));
                }
            }
            Some(ParsedAapEvent::RawNotification(cmd, payload.to_vec()))
        }

        CMD_DEVICE_INFO => {
            let info = parse_device_info_payload(payload);
            Some(ParsedAapEvent::DeviceInfo(info))
        }

        _ => Some(ParsedAapEvent::RawNotification(cmd, payload.to_vec())),
    }
}

/// Parses the battery payload bytes into BatteryReport
pub fn parse_battery_payload(payload: &[u8]) -> BatteryReport {
    // Format variant A: 3 pairs of [level, status]
    // Level is 0-100, 255 = disconnected. Status bit 0 = charging.
    let mut left = None;
    let mut right = None;
    let mut case = None;
    let mut single = None;

    if payload.len() >= 6 {
        // Byte 0, 1: Left
        let left_lvl = payload[0];
        let left_chg = (payload[1] & 0x01) != 0;
        if left_lvl <= 100 {
            left = Some(BatteryEntry {
                level: left_lvl as i32,
                charging: left_chg,
                connected: true,
            });
        }

        // Byte 2, 3: Right
        let right_lvl = payload[2];
        let right_chg = (payload[3] & 0x01) != 0;
        if right_lvl <= 100 {
            right = Some(BatteryEntry {
                level: right_lvl as i32,
                charging: right_chg,
                connected: true,
            });
        }

        // Byte 4, 5: Case
        let case_lvl = payload[4];
        let case_chg = (payload[5] & 0x01) != 0;
        if case_lvl <= 100 {
            case = Some(BatteryEntry {
                level: case_lvl as i32,
                charging: case_chg,
                connected: true,
            });
        }
    } else if payload.len() >= 2 {
        // Single battery payload for over-ear / neckband models
        let lvl = payload[0];
        let chg = (payload[1] & 0x01) != 0;
        if lvl <= 100 {
            single = Some(BatteryEntry {
                level: lvl as i32,
                charging: chg,
                connected: true,
            });
        }
    }

    BatteryReport {
        left,
        right,
        case,
        single,
    }
}

/// Parses the ear detection payload bytes
pub fn parse_ear_detection_payload(payload: &[u8]) -> EarDetectionReport {
    let mut left_in = false;
    let mut right_in = false;

    if !payload.is_empty() {
        left_in = payload[0] == 0x01;
    }
    if payload.len() >= 2 {
        right_in = payload[1] == 0x01;
    }

    EarDetectionReport {
        left_in_ear: left_in,
        right_in_ear: right_in,
    }
}

/// Parses device info strings from payload
pub fn parse_device_info_payload(payload: &[u8]) -> DeviceInfoReport {
    // ASCII/UTF-8 null-terminated or length prefixed
    let s = String::from_utf8_lossy(payload);
    let parts: Vec<&str> = s.split('\0').filter(|p| !p.is_empty()).collect();

    let model_name = parts.get(0).copied().unwrap_or("Beats Headset").to_string();
    let firmware = parts.get(1).copied().unwrap_or("Unknown").to_string();
    let serial = parts.get(2).copied().unwrap_or("").to_string();

    DeviceInfoReport {
        model_name,
        firmware,
        serial,
    }
}
