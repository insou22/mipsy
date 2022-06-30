use std::rc::Rc;

use bounce::prelude::*;
use mipsy_utils::MipsyConfig;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

#[derive(Clone, PartialEq, Atom, Serialize, Deserialize)]
pub struct PrimaryColor(pub String);
#[derive(Clone, PartialEq, Atom, Serialize, Deserialize)]
pub struct SecondaryColor(pub String);
#[derive(Clone, PartialEq, Atom, Serialize, Deserialize)]
pub struct TertiaryColor(pub String);

#[derive(Clone, PartialEq, Atom, Serialize, Deserialize)]
#[bounce(observed)]
pub struct MipsyWebConfig {
    pub mipsy_config: MipsyConfig,
    pub ignore_breakpoints: bool,
    pub primary_color: PrimaryColor,
    pub secondary_color: SecondaryColor,
    pub tertiary_color: TertiaryColor,
    pub tab_size: u32,
    pub font_size: u32,
}

impl Observed for MipsyWebConfig {
    fn changed(self: Rc<Self>) {
        crate::set_localstorage("mipsy_web_config", &serde_json::to_string(&self).unwrap());
    }
}

impl Default for PrimaryColor {
    fn default() -> Self {
        PrimaryColor("#fee2e2".to_string())
    }
}
impl From<std::string::String> for PrimaryColor {
    fn from(value: std::string::String) -> Self {
        PrimaryColor(value)
    }
}

impl Default for SecondaryColor {
    fn default() -> Self {
        SecondaryColor("#f0f0f0".to_string())
    }
}
impl From<std::string::String> for SecondaryColor {
    fn from(value: std::string::String) -> Self {
        SecondaryColor(value)
    }
}

impl Default for TertiaryColor {
    fn default() -> Self {
        TertiaryColor("#d19292".to_string())
    }
}

impl From<std::string::String> for TertiaryColor {
    fn from(value: std::string::String) -> Self {
        TertiaryColor(value)
    }
}

impl Default for MipsyWebConfig {
    fn default() -> Self {
        Self {
            mipsy_config: MipsyConfig::default(),
            ignore_breakpoints: false,
            primary_color: PrimaryColor::default(),
            secondary_color: SecondaryColor::default(),
            tertiary_color: TertiaryColor::default(),
            tab_size: 8,
            font_size: 14,
        }
    }
}

impl MipsyWebConfig {
    pub fn apply(&self) {
        #[derive(Serialize, Deserialize)]
        struct EditorModelOptions {
            #[serde(rename = "tabSize")]
            tab_size: u32,
        }
        #[derive(Serialize, Deserialize)]
        struct EditorOptions {
            #[serde(rename = "fontSize")]
            font_size: u32,
        }

        crate::update_primary_color(&self.primary_color.0);
        crate::update_secondary_color(&self.secondary_color.0);
        crate::update_tertiary_color(&self.tertiary_color.0);
        crate::update_editor_options(
            JsValue::from_serde(&EditorOptions {
                font_size: self.font_size,
            })
            .unwrap(),
        );
        crate::update_editor_model_options(
            JsValue::from_serde(&EditorModelOptions {
                tab_size: self.tab_size,
            })
            .unwrap(),
        );
    }
}
