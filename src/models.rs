use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormFactor {
    Earbuds,
    OverEar,
    Neckband,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceModelInfo {
    pub model_id: String,
    pub display_name: String,
    pub form_factor: FormFactor,
    pub has_anc: bool,
    pub has_transparency: bool,
    pub has_adaptive: bool,
    pub has_in_ear: bool,
    pub has_tri_battery: bool,
    pub has_spatial_audio: bool,
    pub has_conversational_awareness: bool,
    pub has_one_bud_anc: bool,
    pub has_chime: bool,
}

impl DeviceModelInfo {
    pub fn new(
        model_id: &str,
        display_name: &str,
        form_factor: FormFactor,
        has_anc: bool,
        has_transparency: bool,
        has_adaptive: bool,
        has_in_ear: bool,
        has_tri_battery: bool,
        has_spatial_audio: bool,
        has_conversational_awareness: bool,
        has_one_bud_anc: bool,
        has_chime: bool,
    ) -> Self {
        Self {
            model_id: model_id.to_string(),
            display_name: display_name.to_string(),
            form_factor,
            has_anc,
            has_transparency,
            has_adaptive,
            has_in_ear,
            has_tri_battery,
            has_spatial_audio,
            has_conversational_awareness,
            has_one_bud_anc,
            has_chime,
        }
    }
}

pub fn get_known_beats_models() -> Vec<DeviceModelInfo> {
    vec![
        DeviceModelInfo::new("beats_fit_pro", "Beats Fit Pro", FormFactor::Earbuds, true, true, true, true, true, true, true, true, true),
        DeviceModelInfo::new("beats_studio_pro", "Beats Studio Pro", FormFactor::OverEar, true, true, false, false, false, true, false, false, true),
        DeviceModelInfo::new("beats_solo_4", "Beats Solo 4", FormFactor::OverEar, false, false, false, false, false, true, false, false, true),
        DeviceModelInfo::new("beats_studio_buds", "Beats Studio Buds", FormFactor::Earbuds, true, true, false, true, true, false, false, false, true),
        DeviceModelInfo::new("beats_studio_buds_plus", "Beats Studio Buds +", FormFactor::Earbuds, true, true, false, true, true, true, false, true, true),
        DeviceModelInfo::new("powerbeats_pro", "Powerbeats Pro", FormFactor::Earbuds, false, false, false, true, true, false, false, false, true),
        DeviceModelInfo::new("beats_solo_pro", "Beats Solo Pro", FormFactor::OverEar, true, true, false, false, false, false, false, false, false),
        DeviceModelInfo::new("beats_flex", "Beats Flex", FormFactor::Neckband, false, false, false, false, false, false, false, false, false),
        DeviceModelInfo::new("beats_studio_3", "Beats Studio 3 Wireless", FormFactor::OverEar, true, false, false, false, false, false, false, false, false),
        DeviceModelInfo::new("beats_solo_3", "Beats Solo 3 Wireless", FormFactor::OverEar, false, false, false, false, false, false, false, false, false),
        DeviceModelInfo::new("airpods_pro", "AirPods Pro", FormFactor::Earbuds, true, true, false, true, true, true, false, false, true),
        DeviceModelInfo::new("airpods_pro_2", "AirPods Pro 2", FormFactor::Earbuds, true, true, true, true, true, true, true, true, true),
        DeviceModelInfo::new("airpods_max", "AirPods Max", FormFactor::OverEar, true, true, false, false, false, true, false, false, false),
    ]
}

/// Resolves device model info from device name, alias, or modalias PID.
pub fn match_model(name_or_alias: &str, modalias: &str) -> DeviceModelInfo {
    let lower_name = name_or_alias.to_lowercase();
    let lower_modalias = modalias.to_lowercase();

    // Check by modalias PID first (exact match)
    if lower_modalias.contains("v004cp2012") {
        return find_model("beats_fit_pro");
    }
    if lower_modalias.contains("v004cp2015") {
        return find_model("beats_studio_pro");
    }
    if lower_modalias.contains("v004cp2016") {
        return find_model("beats_solo_4");
    }
    if lower_modalias.contains("v004cp2011") {
        return find_model("beats_studio_buds");
    }
    if lower_modalias.contains("v004cp2014") {
        return find_model("beats_studio_buds_plus");
    }
    if lower_modalias.contains("v004cp200b") {
        return find_model("powerbeats_pro");
    }
    if lower_modalias.contains("v004cp200c") {
        return find_model("beats_solo_pro");
    }
    if lower_modalias.contains("v004cp200e") {
        return find_model("beats_flex");
    }

    // Name substring matching
    if lower_name.contains("fit pro") || lower_name.contains("fit_pro") {
        return find_model("beats_fit_pro");
    }
    if lower_name.contains("studio pro") || lower_name.contains("studio_pro") {
        return find_model("beats_studio_pro");
    }
    if lower_name.contains("solo 4") || lower_name.contains("solo_4") {
        return find_model("beats_solo_4");
    }
    if lower_name.contains("studio buds +") || lower_name.contains("studio buds+") || lower_name.contains("studio_buds_plus") {
        return find_model("beats_studio_buds_plus");
    }
    if lower_name.contains("studio buds") || lower_name.contains("studio_buds") {
        return find_model("beats_studio_buds");
    }
    if lower_name.contains("powerbeats pro") || lower_name.contains("powerbeats_pro") {
        return find_model("powerbeats_pro");
    }
    if lower_name.contains("solo pro") || lower_name.contains("solo_pro") {
        return find_model("beats_solo_pro");
    }
    if lower_name.contains("beats flex") || lower_name.contains("beats_flex") {
        return find_model("beats_flex");
    }
    if lower_name.contains("studio 3") || lower_name.contains("studio_3") {
        return find_model("beats_studio_3");
    }
    if lower_name.contains("solo 3") || lower_name.contains("solo_3") {
        return find_model("beats_solo_3");
    }
    if lower_name.contains("airpods pro 2") || lower_name.contains("airpods_pro_2") {
        return find_model("airpods_pro_2");
    }
    if lower_name.contains("airpods pro") || lower_name.contains("airpods_pro") {
        return find_model("airpods_pro");
    }
    if lower_name.contains("airpods max") || lower_name.contains("airpods_max") {
        return find_model("airpods_max");
    }

    // Default fallback: Generic Beats Earbuds
    DeviceModelInfo::new(
        "beats_generic",
        if name_or_alias.is_empty() { "Beats Device" } else { "Beats Headset" },
        FormFactor::Earbuds,
        true,
        true,
        false,
        true,
        true,
        false,
        false,
        false,
        true,
    )
}

fn find_model(id: &str) -> DeviceModelInfo {
    let models = get_known_beats_models();
    models
        .into_iter()
        .find(|m| m.model_id == id)
        .unwrap_or_else(|| DeviceModelInfo::new(
            "unknown",
            "Beats Wireless",
            FormFactor::Unknown,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        ))
}
