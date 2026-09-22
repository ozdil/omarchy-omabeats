use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormFactor {
    Earbuds,
    OverEar,
    OnEar,
    Neckband,
    Speaker,
    Unknown,
}

impl FormFactor {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            FormFactor::Earbuds => "Earbuds",
            FormFactor::OverEar => "OverEar",
            FormFactor::OnEar => "OnEar",
            FormFactor::Neckband => "Neckband",
            FormFactor::Speaker => "Speaker",
            FormFactor::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionType {
    BluetoothL2cap,
    BluetoothClassic,
    UsbLossless,
    AnalogJack,
}

impl ConnectionType {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            ConnectionType::BluetoothL2cap => "Bluetooth (AAP/L2CAP)",
            ConnectionType::BluetoothClassic => "Bluetooth (Classic A2DP)",
            ConnectionType::UsbLossless => "USB-C (Lossless Audio 24-bit/48kHz)",
            ConnectionType::AnalogJack => "3.5mm Analog Kablolu",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceModelInfo {
    pub model_id: String,
    pub display_name: String,
    pub form_factor: FormFactor,
    pub default_connection: ConnectionType,
    pub has_anc: bool,
    pub has_transparency: bool,
    pub has_adaptive: bool,
    pub has_in_ear: bool,
    pub has_tri_battery: bool,
    pub has_spatial_audio: bool,
    pub has_conversational_awareness: bool,
    pub has_one_bud_anc: bool,
    pub has_chime: bool,
    pub has_usb_audio: bool,
    pub has_analog_jack: bool,
    pub is_wired_only: bool,
    pub release_year: Option<u16>,
}

impl DeviceModelInfo {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        model_id: &str,
        display_name: &str,
        form_factor: FormFactor,
        default_connection: ConnectionType,
        has_anc: bool,
        has_transparency: bool,
        has_adaptive: bool,
        has_in_ear: bool,
        has_tri_battery: bool,
        has_spatial_audio: bool,
        has_conversational_awareness: bool,
        has_one_bud_anc: bool,
        has_chime: bool,
        has_usb_audio: bool,
        has_analog_jack: bool,
        is_wired_only: bool,
        release_year: Option<u16>,
    ) -> Self {
        Self {
            model_id: model_id.to_string(),
            display_name: display_name.to_string(),
            form_factor,
            default_connection,
            has_anc,
            has_transparency,
            has_adaptive,
            has_in_ear,
            has_tri_battery,
            has_spatial_audio,
            has_conversational_awareness,
            has_one_bud_anc,
            has_chime,
            has_usb_audio,
            has_analog_jack,
            is_wired_only,
            release_year,
        }
    }
}

/// Returns the comprehensive master catalog of all Beats headphones, earphones, and speakers ever produced.
pub fn get_known_beats_models() -> Vec<DeviceModelInfo> {
    vec![
        // ==========================================
        // MODERN BLUETOOTH & WIRELESS EARBUDS
        // ==========================================
        DeviceModelInfo::new(
            "beats_fit_pro", "Beats Fit Pro", FormFactor::Earbuds,
            ConnectionType::BluetoothL2cap, true, true, true, true, true, true, false, true, true, false, false, false, Some(2021),
        ),
        DeviceModelInfo::new(
            "beats_studio_buds_plus", "Beats Studio Buds +", FormFactor::Earbuds,
            ConnectionType::BluetoothL2cap, true, true, false, true, true, true, false, true, true, false, false, false, Some(2023),
        ),
        DeviceModelInfo::new(
            "beats_studio_buds", "Beats Studio Buds", FormFactor::Earbuds,
            ConnectionType::BluetoothL2cap, true, true, false, true, true, false, false, false, true, false, false, false, Some(2021),
        ),
        DeviceModelInfo::new(
            "beats_solo_buds", "Beats Solo Buds", FormFactor::Earbuds,
            ConnectionType::BluetoothL2cap, false, false, false, true, false, true, false, false, true, false, false, false, Some(2024),
        ),
        DeviceModelInfo::new(
            "powerbeats_pro", "Powerbeats Pro", FormFactor::Earbuds,
            ConnectionType::BluetoothL2cap, false, false, false, true, true, false, false, false, true, false, false, false, Some(2019),
        ),
        DeviceModelInfo::new(
            "powerbeats_pro_2", "Powerbeats Pro 2", FormFactor::Earbuds,
            ConnectionType::BluetoothL2cap, true, true, true, true, true, true, false, true, true, false, false, false, Some(2025),
        ),

        // ==========================================
        // WIRELESS NECKBANDS & SPORT EARPHONES
        // ==========================================
        DeviceModelInfo::new(
            "beats_flex", "Beats Flex", FormFactor::Neckband,
            ConnectionType::BluetoothL2cap, false, false, false, false, false, false, false, false, false, false, false, false, Some(2020),
        ),
        DeviceModelInfo::new(
            "beats_x", "BeatsX", FormFactor::Neckband,
            ConnectionType::BluetoothL2cap, false, false, false, false, false, false, false, false, false, false, false, false, Some(2017),
        ),
        DeviceModelInfo::new(
            "powerbeats_4", "Powerbeats", FormFactor::Neckband,
            ConnectionType::BluetoothL2cap, false, false, false, false, false, false, false, false, false, false, false, false, Some(2020),
        ),
        DeviceModelInfo::new(
            "powerbeats_3", "Powerbeats 3 Wireless", FormFactor::Neckband,
            ConnectionType::BluetoothL2cap, false, false, false, false, false, false, false, false, false, false, false, false, Some(2016),
        ),
        DeviceModelInfo::new(
            "powerbeats_2_wireless", "Powerbeats 2 Wireless", FormFactor::Neckband,
            ConnectionType::BluetoothClassic, false, false, false, false, false, false, false, false, false, false, false, false, Some(2014),
        ),

        // ==========================================
        // WIRELESS & HYBRID OVER-EAR / ON-EAR HEADPHONES
        // ==========================================
        DeviceModelInfo::new(
            "beats_studio_pro", "Beats Studio Pro", FormFactor::OverEar,
            ConnectionType::BluetoothL2cap, true, true, false, false, false, true, false, false, true, true, true, false, Some(2023),
        ),
        DeviceModelInfo::new(
            "beats_studio_3", "Beats Studio 3 Wireless", FormFactor::OverEar,
            ConnectionType::BluetoothL2cap, true, false, false, false, false, false, false, false, false, false, true, false, Some(2017),
        ),
        DeviceModelInfo::new(
            "beats_studio_2_wireless", "Beats Studio 2.0 Wireless", FormFactor::OverEar,
            ConnectionType::BluetoothClassic, true, false, false, false, false, false, false, false, false, false, true, false, Some(2013),
        ),
        DeviceModelInfo::new(
            "beats_solo_4", "Beats Solo 4", FormFactor::OnEar,
            ConnectionType::BluetoothL2cap, false, false, false, false, false, true, false, false, true, true, true, false, Some(2024),
        ),
        DeviceModelInfo::new(
            "beats_solo_pro", "Beats Solo Pro", FormFactor::OnEar,
            ConnectionType::BluetoothL2cap, true, true, false, false, false, false, false, false, false, false, false, false, Some(2019),
        ),
        DeviceModelInfo::new(
            "beats_solo_3", "Beats Solo 3 Wireless", FormFactor::OnEar,
            ConnectionType::BluetoothL2cap, false, false, false, false, false, false, false, false, false, false, true, false, Some(2016),
        ),
        DeviceModelInfo::new(
            "beats_solo_2_wireless", "Beats Solo 2 Wireless", FormFactor::OnEar,
            ConnectionType::BluetoothClassic, false, false, false, false, false, false, false, false, false, false, true, false, Some(2014),
        ),
        DeviceModelInfo::new(
            "beats_wireless_1", "Beats Wireless", FormFactor::OnEar,
            ConnectionType::BluetoothClassic, false, false, false, false, false, false, false, false, false, false, true, false, Some(2012),
        ),

        // ==========================================
        // HISTORICAL WIRED HEADPHONES & EARPHONES (3.5mm / Lightning / Jack)
        // ==========================================
        DeviceModelInfo::new(
            "beats_ep", "Beats EP", FormFactor::OnEar,
            ConnectionType::AnalogJack, false, false, false, false, false, false, false, false, false, false, true, true, Some(2016),
        ),
        DeviceModelInfo::new(
            "beats_pro", "Beats Pro", FormFactor::OverEar,
            ConnectionType::AnalogJack, false, false, false, false, false, false, false, false, false, false, true, true, Some(2010),
        ),
        DeviceModelInfo::new(
            "beats_executive", "Beats Executive", FormFactor::OverEar,
            ConnectionType::AnalogJack, true, false, false, false, false, false, false, false, false, false, true, true, Some(2012),
        ),
        DeviceModelInfo::new(
            "beats_mixr", "Beats Mixr", FormFactor::OnEar,
            ConnectionType::AnalogJack, false, false, false, false, false, false, false, false, false, false, true, true, Some(2011),
        ),
        DeviceModelInfo::new(
            "beats_studio_1", "Beats Studio 1.0 (Original)", FormFactor::OverEar,
            ConnectionType::AnalogJack, true, false, false, false, false, false, false, false, false, false, true, true, Some(2008),
        ),
        DeviceModelInfo::new(
            "beats_studio_2_wired", "Beats Studio 2.0 Wired", FormFactor::OverEar,
            ConnectionType::AnalogJack, true, false, false, false, false, false, false, false, false, false, true, true, Some(2013),
        ),
        DeviceModelInfo::new(
            "beats_solo_hd", "Beats Solo HD", FormFactor::OnEar,
            ConnectionType::AnalogJack, false, false, false, false, false, false, false, false, false, false, true, true, Some(2009),
        ),
        DeviceModelInfo::new(
            "beats_solo_2_wired", "Beats Solo 2 Wired", FormFactor::OnEar,
            ConnectionType::AnalogJack, false, false, false, false, false, false, false, false, false, false, true, true, Some(2014),
        ),
        DeviceModelInfo::new(
            "urbeats_3", "urBeats 3", FormFactor::Earbuds,
            ConnectionType::AnalogJack, false, false, false, false, false, false, false, false, false, false, true, true, Some(2017),
        ),
        DeviceModelInfo::new(
            "urbeats_2", "urBeats 2", FormFactor::Earbuds,
            ConnectionType::AnalogJack, false, false, false, false, false, false, false, false, false, false, true, true, Some(2013),
        ),
        DeviceModelInfo::new(
            "urbeats_1", "urBeats (Original)", FormFactor::Earbuds,
            ConnectionType::AnalogJack, false, false, false, false, false, false, false, false, false, false, true, true, Some(2012),
        ),
        DeviceModelInfo::new(
            "beats_tour_2", "Beats Tour 2.0", FormFactor::Earbuds,
            ConnectionType::AnalogJack, false, false, false, false, false, false, false, false, false, false, true, true, Some(2015),
        ),
        DeviceModelInfo::new(
            "beats_tour_1", "Beats Tour (Original)", FormFactor::Earbuds,
            ConnectionType::AnalogJack, false, false, false, false, false, false, false, false, false, false, true, true, Some(2008),
        ),
        DeviceModelInfo::new(
            "powerbeats_wired", "Powerbeats Wired", FormFactor::Earbuds,
            ConnectionType::AnalogJack, false, false, false, false, false, false, false, false, false, false, true, true, Some(2010),
        ),
        DeviceModelInfo::new(
            "heartbeats_lady_gaga", "Heartbeats by Lady Gaga", FormFactor::Earbuds,
            ConnectionType::AnalogJack, false, false, false, false, false, false, false, false, false, false, true, true, Some(2009),
        ),
        DeviceModelInfo::new(
            "diddybeats", "Diddybeats", FormFactor::Earbuds,
            ConnectionType::AnalogJack, false, false, false, false, false, false, false, false, false, false, true, true, Some(2010),
        ),

        // ==========================================
        // BEATS PILL SPEAKERS
        // ==========================================
        DeviceModelInfo::new(
            "beats_pill_2024", "Beats Pill (2024)", FormFactor::Speaker,
            ConnectionType::BluetoothL2cap, false, false, false, false, false, false, false, false, false, true, false, false, Some(2024),
        ),
        DeviceModelInfo::new(
            "beats_pill_plus", "Beats Pill+", FormFactor::Speaker,
            ConnectionType::BluetoothClassic, false, false, false, false, false, false, false, false, false, false, true, false, Some(2015),
        ),
        DeviceModelInfo::new(
            "beats_pill_2", "Beats Pill 2.0", FormFactor::Speaker,
            ConnectionType::BluetoothClassic, false, false, false, false, false, false, false, false, false, false, true, false, Some(2013),
        ),
        DeviceModelInfo::new(
            "beats_pill_1", "Beats Pill 1.0", FormFactor::Speaker,
            ConnectionType::BluetoothClassic, false, false, false, false, false, false, false, false, false, false, true, false, Some(2012),
        ),
        DeviceModelInfo::new(
            "beats_pill_xl", "Beats Pill XL", FormFactor::Speaker,
            ConnectionType::BluetoothClassic, false, false, false, false, false, false, false, false, false, false, true, false, Some(2013),
        ),

        // ==========================================
        // APPLE AIRPODS FAMILY (SHARED AAP PROTOCOL)
        // ==========================================
        DeviceModelInfo::new(
            "airpods_pro", "AirPods Pro", FormFactor::Earbuds,
            ConnectionType::BluetoothL2cap, true, true, false, true, true, true, false, false, true, false, false, false, Some(2019),
        ),
        DeviceModelInfo::new(
            "airpods_pro_2", "AirPods Pro 2", FormFactor::Earbuds,
            ConnectionType::BluetoothL2cap, true, true, true, true, true, true, true, true, true, false, false, false, Some(2022),
        ),
        DeviceModelInfo::new(
            "airpods_max", "AirPods Max", FormFactor::OverEar,
            ConnectionType::BluetoothL2cap, true, true, false, false, false, true, false, false, false, false, true, false, Some(2020),
        ),
        DeviceModelInfo::new(
            "airpods_4_anc", "AirPods 4 with ANC", FormFactor::Earbuds,
            ConnectionType::BluetoothL2cap, true, true, true, true, true, true, true, false, true, false, false, false, Some(2024),
        ),
        DeviceModelInfo::new(
            "airpods_4", "AirPods 4", FormFactor::Earbuds,
            ConnectionType::BluetoothL2cap, false, false, false, true, true, true, false, false, true, false, false, false, Some(2024),
        ),
        DeviceModelInfo::new(
            "airpods_3", "AirPods 3", FormFactor::Earbuds,
            ConnectionType::BluetoothL2cap, false, false, false, true, true, true, false, false, true, false, false, false, Some(2021),
        ),
    ]
}

/// Matches a device by modalias PID, USB ID, or device name.
/// If not in the hardcoded list, applies a smart future-proof discovery engine.
pub fn match_model(name_or_alias: &str, modalias: &str) -> DeviceModelInfo {
    let lower_name = name_or_alias.to_lowercase();
    let lower_modalias = modalias.to_lowercase();

    // 1. Modalias Product ID matching (Apple Vendor ID: 0x004C)
    let pid_map = [
        ("v004cp2012", "beats_fit_pro"),
        ("v004cp2015", "beats_studio_pro"),
        ("v004cp2016", "beats_solo_4"),
        ("v004cp2011", "beats_studio_buds"),
        ("v004cp2014", "beats_studio_buds_plus"),
        ("v004cp2017", "beats_solo_buds"),
        ("v004cp200b", "powerbeats_pro"),
        ("v004cp200c", "beats_solo_pro"),
        ("v004cp200e", "beats_flex"),
        ("v004cp2006", "beats_solo_3"),
        ("v004cp2009", "beats_studio_3"),
        ("v004cp2002", "beats_x"),
        ("v004cp200d", "powerbeats_4"),
        ("v004cp2018", "beats_pill_2024"),
    ];

    for (pattern, id) in pid_map {
        if lower_modalias.contains(pattern) {
            return find_model(id);
        }
    }

    // 2. Exact Name or Model ID Matching across master catalog
    let catalog = get_known_beats_models();
    let normalized_input = lower_name.replace(['_', '-'], " ");
    for model in &catalog {
        let model_clean = model.display_name.to_lowercase();
        if lower_name == model.model_id || lower_name == model_clean || normalized_input == model_clean {
            return model.clone();
        }
    }

    // 3. Substring Name Matching (Ordered from specific to general)
    let name_rules = [
        ("beats fit pro", "beats_fit_pro"),
        ("fit pro", "beats_fit_pro"),
        ("studio pro", "beats_studio_pro"),
        ("studio buds +", "beats_studio_buds_plus"),
        ("studio buds+", "beats_studio_buds_plus"),
        ("studio buds", "beats_studio_buds"),
        ("solo buds", "beats_solo_buds"),
        ("powerbeats pro 2", "powerbeats_pro_2"),
        ("powerbeats pro", "powerbeats_pro"),
        ("powerbeats 4", "powerbeats_4"),
        ("powerbeats 3", "powerbeats_3"),
        ("powerbeats 2", "powerbeats_2_wireless"),
        ("powerbeats", "powerbeats_4"),
        ("solo pro", "beats_solo_pro"),
        ("solo 4", "beats_solo_4"),
        ("solo 3", "beats_solo_3"),
        ("solo 2", "beats_solo_2_wireless"),
        ("solo hd", "beats_solo_hd"),
        ("studio 3", "beats_studio_3"),
        ("studio 2", "beats_studio_2_wireless"),
        ("studio 1", "beats_studio_1"),
        ("beats ep", "beats_ep"),
        ("beats pro", "beats_pro"),
        ("beats executive", "beats_executive"),
        ("beats mixr", "beats_mixr"),
        ("beats flex", "beats_flex"),
        ("beatsx", "beats_x"),
        ("beats x", "beats_x"),
        ("urbeats 3", "urbeats_3"),
        ("urbeats3", "urbeats_3"),
        ("urbeats 2", "urbeats_2"),
        ("urbeats", "urbeats_1"),
        ("beats tour 2", "beats_tour_2"),
        ("beats tour", "beats_tour_1"),
        ("heartbeats", "heartbeats_lady_gaga"),
        ("diddybeats", "diddybeats"),
        ("beats pill 2024", "beats_pill_2024"),
        ("beats pill+", "beats_pill_plus"),
        ("beats pill 2", "beats_pill_2"),
        ("beats pill", "beats_pill_2024"),
        ("airpods pro 2", "airpods_pro_2"),
        ("airpods pro", "airpods_pro"),
        ("airpods max", "airpods_max"),
        ("airpods 4", "airpods_4_anc"),
        ("airpods 3", "airpods_3"),
    ];

    for (token, id) in name_rules {
        if lower_name.contains(token) || normalized_input.contains(token) {
            return find_model(id);
        }
    }

    // 4. FUTURE-PROOF DYNAMIC ENGINE
    // For unreleased, newly launched, or custom Beats models (Apple Vendor ID 004c or name containing Beats)
    let is_apple_or_beats = lower_modalias.contains("v004c")
        || lower_name.contains("beats")
        || lower_name.contains("powerbeats");

    if is_apple_or_beats {
        return discover_future_beats_model(name_or_alias);
    }

    // Generic fallback for non-Beats audio device
    DeviceModelInfo::new(
        "generic_audio",
        if name_or_alias.is_empty() { "Audio Device" } else { name_or_alias },
        FormFactor::Unknown,
        ConnectionType::BluetoothClassic,
        false, false, false, false, false, false, false, false, false, false, true, false, None,
    )
}

/// Dynamically infers form-factor and capabilities for any future unreleased Beats device.
pub fn discover_future_beats_model(raw_name: &str) -> DeviceModelInfo {
    let lower = raw_name.to_lowercase();

    let form_factor = if lower.contains("pill") || lower.contains("speaker") {
        FormFactor::Speaker
    } else if lower.contains("buds") || lower.contains("fit") || lower.contains("in-ear") || lower.contains("earbuds") {
        FormFactor::Earbuds
    } else if lower.contains("solo") || lower.contains("on-ear") || lower.contains("mixr") || lower.contains("ep") {
        FormFactor::OnEar
    } else if lower.contains("studio") || lower.contains("over-ear") || lower.contains("max") || lower.contains("pro") {
        FormFactor::OverEar
    } else if lower.contains("flex") || lower.contains("neckband") || lower.contains("beatsx") || lower.contains("beats x") {
        FormFactor::Neckband
    } else {
        FormFactor::Earbuds
    };

    let has_anc = !lower.contains("flex") && !lower.contains("ep") && !lower.contains("solo buds");
    let has_transparency = has_anc;
    let has_in_ear = matches!(form_factor, FormFactor::Earbuds);
    let has_tri_battery = matches!(form_factor, FormFactor::Earbuds) && !lower.contains("solo buds");
    let has_spatial = true;
    let has_chime = matches!(form_factor, FormFactor::Earbuds | FormFactor::OverEar);
    let has_usb_audio = matches!(form_factor, FormFactor::OverEar | FormFactor::OnEar | FormFactor::Speaker);
    let has_analog_jack = matches!(form_factor, FormFactor::OverEar | FormFactor::OnEar);

    let clean_title = if raw_name.is_empty() {
        "Next-Gen Beats Device".to_string()
    } else {
        raw_name.to_string()
    };

    DeviceModelInfo::new(
        "beats_future_device",
        &clean_title,
        form_factor,
        ConnectionType::BluetoothL2cap,
        has_anc,
        has_transparency,
        false,
        has_in_ear,
        has_tri_battery,
        has_spatial,
        false,
        has_anc,
        has_chime,
        has_usb_audio,
        has_analog_jack,
        false,
        None,
    )
}

pub fn find_model(id: &str) -> DeviceModelInfo {
    let models = get_known_beats_models();
    models
        .into_iter()
        .find(|m| m.model_id == id)
        .unwrap_or_else(|| DeviceModelInfo::new(
            "unknown",
            "Beats Headset",
            FormFactor::Unknown,
            ConnectionType::BluetoothClassic,
            false, false, false, false, false, false, false, false, false, false, false, false, None,
        ))
}
