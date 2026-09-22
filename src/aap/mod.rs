#![allow(dead_code)]

pub mod commands;
pub mod parser;

use serde::{Deserialize, Serialize};

/// Transport: L2CAP, PSM 0x1001 (4097)
pub const AAP_PSM: u16 = 0x1001;

/// Apple / Beats Accessory Protocol service UUID
pub const BEATS_AAP_UUID: &str = "74ec2172-0bad-4d01-8f77-997b2be0722a";

/// Standard 4-byte control packet header
pub const HEADER: [u8; 4] = [0x04, 0x00, 0x04, 0x00];

/// Command IDs (byte 4 of packet)
pub const CMD_BATTERY: u8 = 0x04;
pub const CMD_EAR_DETECTION: u8 = 0x06;
pub const CMD_CONTROL: u8 = 0x09;
pub const CMD_NOISE_CONTROL_STATUS: u8 = 0x0D;
pub const CMD_AUDIO_SOURCE: u8 = 0x0E;
pub const CMD_NOTIFICATION_SUBSCRIBE: u8 = 0x0F;
pub const CMD_HEAD_TRACKING: u8 = 0x17;
pub const CMD_STEM_PRESS: u8 = 0x19;
pub const CMD_DEVICE_INFO: u8 = 0x1D;
pub const CMD_CONNECTED_DEVICES: u8 = 0x2E;
pub const CMD_CA_ACTIVITY: u8 = 0x4B;

/// Control sub-commands (byte 6 of packet when cmd is CMD_CONTROL)
pub const SUB_MIC_MODE: u8 = 0x01;
pub const SUB_BUTTON_SEND_MODE: u8 = 0x05;
pub const SUB_EAR_DETECTION: u8 = 0x0A;
pub const SUB_ANC_MODE: u8 = 0x0D;
pub const SUB_VOICE_TRIGGER_SIRI: u8 = 0x12;
pub const SUB_ONE_BUD_ANC: u8 = 0x1B;
pub const SUB_CHIME: u8 = 0x1E;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AncMode {
    Off = 1,
    NoiseCancellation = 2,
    Transparency = 3,
    Adaptive = 4,
}

impl AncMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            AncMode::Off => "off",
            AncMode::NoiseCancellation => "noise_cancellation",
            AncMode::Transparency => "transparency",
            AncMode::Adaptive => "adaptive",
        }
    }

    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            1 => Some(AncMode::Off),
            2 => Some(AncMode::NoiseCancellation),
            3 => Some(AncMode::Transparency),
            4 => Some(AncMode::Adaptive),
            _ => None,
        }
    }

    pub fn from_str_name(name: &str) -> Option<Self> {
        let clean = name.to_lowercase().replace(['_', '-', ' '], "");
        match clean.as_str() {
            "off" | "kapali" => Some(AncMode::Off),
            "anc" | "noise" | "noisecancellation" | "cancellation" | "gurultu" => Some(AncMode::NoiseCancellation),
            "transparency" | "seffaf" | "ambient" => Some(AncMode::Transparency),
            "adaptive" | "uyumlu" => Some(AncMode::Adaptive),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MicMode {
    Auto = 0,
    Right = 1,
    Left = 2,
}

impl MicMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            MicMode::Auto => "auto",
            MicMode::Right => "right",
            MicMode::Left => "left",
        }
    }

    pub fn from_str_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "auto" | "otomatik" => Some(MicMode::Auto),
            "right" | "sag" => Some(MicMode::Right),
            "left" | "sol" => Some(MicMode::Left),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EarStatus {
    OutOfEar = 0,
    InEar = 1,
    Unknown = 2,
}
