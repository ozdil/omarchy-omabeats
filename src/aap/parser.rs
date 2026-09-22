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

    // Must have standard AAP framing header [0x04, 0x00]
    if data[0] != 0x04 || data[1] != 0x00 {
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
                    let report = parse_ear_detection_payload(&payload[1..]);
                    return Some(ParsedAapEvent::EarDetection(report));
                }
            }
            Some(ParsedAapEvent::RawNotification(cmd, payload.to_vec()))
        }

        CMD_NOISE_CONTROL_STATUS => {
            if !payload.is_empty() {
                if let Some(mode) = AncMode::from_u8(payload[0]) {
                    return Some(ParsedAapEvent::AncMode(mode));
                }
            }
            Some(ParsedAapEvent::RawNotification(cmd, payload.to_vec()))
        }

        CMD_CA_ACTIVITY => {
            if !payload.is_empty() {
                let enabled = payload[0] == 0x01;
                return Some(ParsedAapEvent::ConversationalAwareness(enabled));
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
    let mut left = None;
    let mut right = None;
    let mut case = None;
    let mut single = None;

    if payload.is_empty() {
        return BatteryReport { left, right, case, single };
    }

    let count = payload[0] as usize;
    // Check if it matches AAP TLV format: payload[0] = count, followed by count * 5 bytes
    if count > 0 && payload.len() >= 1 + count * 5 {
        let mut offset = 1;
        for _ in 0..count {
            if offset + 5 > payload.len() {
                break;
            }
            let comp_id = payload[offset];
            // payload[offset + 1] is length/type (usually 0x01)
            let level = payload[offset + 2];
            let status = payload[offset + 3];
            let charging = (status & 0x01) != 0;
            let disconnected = (status & 0x04) != 0;

            let entry = if !disconnected && level <= 100 && (comp_id != 0x08 || level > 0) {
                Some(BatteryEntry {
                    level: level as i32,
                    charging,
                    connected: true,
                })
            } else {
                None
            };

            match comp_id {
                0x04 => left = entry,
                0x02 => right = entry,
                0x08 => case = entry,
                0x01 => single = entry,
                _ => {}
            }
            offset += 5;
        }
    } else if payload.len() >= 6 {
        // Fallback variant A: 3 pairs of [level, status]
        let left_lvl = payload[0];
        let left_chg = (payload[1] & 0x01) != 0;
        if left_lvl <= 100 {
            left = Some(BatteryEntry {
                level: left_lvl as i32,
                charging: left_chg,
                connected: true,
            });
        }

        let right_lvl = payload[2];
        let right_chg = (payload[3] & 0x01) != 0;
        if right_lvl <= 100 {
            right = Some(BatteryEntry {
                level: right_lvl as i32,
                charging: right_chg,
                connected: true,
            });
        }

        let case_lvl = payload[4];
        let case_chg = (payload[5] & 0x01) != 0;
        let case_disconn = (payload[5] & 0x04) != 0;
        if !case_disconn && case_lvl <= 100 && case_lvl > 0 {
            case = Some(BatteryEntry {
                level: case_lvl as i32,
                charging: case_chg,
                connected: true,
            });
        }
    } else if payload.len() >= 2 {
        // Fallback single battery payload for over-ear / neckband models
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

/// Parses the ear detection payload bytes according to AAP standard:
/// 0x00 = InEar, 0x01 = OutOfEar, 0x02 = InCase, 0x03 = Disconnected
pub fn parse_ear_detection_payload(payload: &[u8]) -> EarDetectionReport {
    let mut left_in = false;
    let mut right_in = false;

    if !payload.is_empty() {
        left_in = payload[0] == 0x00;
    }
    if payload.len() >= 2 {
        right_in = payload[1] == 0x00;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_real_beats_fit_pro_battery() {
        // Real payload captured from Beats Fit Pro:
        // count=3, item0=(id=04, len=01, val=5f [95%], status=02, 01)
        //          item1=(id=02, len=01, val=5d [93%], status=02, 01)
        //          item2=(id=08, len=01, val=00 [0%],  status=04, 01)
        let payload = [
            0x03, 0x04, 0x01, 0x5f, 0x02, 0x01,
            0x02, 0x01, 0x5d, 0x02, 0x01,
            0x08, 0x01, 0x00, 0x04, 0x01,
        ];
        let rep = parse_battery_payload(&payload);
        assert_eq!(rep.left.as_ref().unwrap().level, 95);
        assert_eq!(rep.left.as_ref().unwrap().charging, false);
        assert_eq!(rep.right.as_ref().unwrap().level, 93);
        assert_eq!(rep.right.as_ref().unwrap().charging, false);
        // Case status 0x04 indicates disconnected (closed case); level should be None to preserve cached charge
        assert!(rep.case.is_none());

        // Test connected case (status 0x02, 94%):
        let connected_case_payload = [0x01, 0x08, 0x01, 0x5e, 0x02, 0x01];
        let rep_conn = parse_battery_payload(&connected_case_payload);
        assert_eq!(rep_conn.case.as_ref().unwrap().level, 94);
        assert_eq!(rep_conn.case.as_ref().unwrap().charging, false);
    }

    #[test]
    fn test_parse_ear_detection_payload() {
        // Both in ear: 0x00, 0x00
        let in_both = parse_ear_detection_payload(&[0x00, 0x00]);
        assert_eq!(in_both.left_in_ear, true);
        assert_eq!(in_both.right_in_ear, true);

        // Left in ear, right out of ear: 0x00, 0x01
        let left_only = parse_ear_detection_payload(&[0x00, 0x01]);
        assert_eq!(left_only.left_in_ear, true);
        assert_eq!(left_only.right_in_ear, false);

        // Both out of ear / in case: 0x01, 0x02
        let out_both = parse_ear_detection_payload(&[0x01, 0x02]);
        assert_eq!(out_both.left_in_ear, false);
        assert_eq!(out_both.right_in_ear, false);
    }
}
