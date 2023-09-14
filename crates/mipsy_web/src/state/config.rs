use std::rc::Rc;

use bounce::prelude::*;
use gloo_utils::format::JsValueSerdeExt;
use mipsy_utils::MipsyConfig;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

// i should probably write a macro for this
#[derive(Clone, PartialEq, Atom, Serialize, Deserialize, Debug)]
pub struct PrimaryColor(pub String);
#[derive(Clone, PartialEq, Atom, Serialize, Deserialize, Debug)]
pub struct SecondaryColor(pub String);
#[derive(Clone, PartialEq, Atom, Serialize, Deserialize, Debug)]
pub struct TertiaryColor(pub String);
#[derive(Clone, PartialEq, Atom, Serialize, Deserialize, Debug)]
pub struct HighlightColor(pub String);
#[derive(Clone, PartialEq, Atom, Serialize, Deserialize, Debug)]
pub struct FontColor(pub String);

#[derive(Clone, PartialEq, Atom, Serialize, Deserialize, Debug)]
#[bounce(observed)]
pub struct MipsyWebConfig {
    pub mipsy_config: MipsyConfig,
    pub ignore_breakpoints: bool,
    pub primary_color: PrimaryColor,
    pub secondary_color: SecondaryColor,
    pub tertiary_color: TertiaryColor,
    pub highlight_color: HighlightColor,
    pub font_color: FontColor,
    pub tab_size: u32,
    pub font_size: u32,
    pub monaco_theme: String,
    pub register_base: RegisterBase,
    // TODO: should this be abstracted to hide N registers?
    pub hide_uncommon_registers: bool,
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

impl Default for HighlightColor {
    fn default() -> Self {
        HighlightColor("#34d399".to_string())
    }
}

impl From<std::string::String> for HighlightColor {
    fn from(value: std::string::String) -> Self {
        HighlightColor(value)
    }
}

impl Default for FontColor {
    fn default() -> Self {
        FontColor("#000000".to_string())
    }
}

impl From<std::string::String> for FontColor {
    fn from(value: std::string::String) -> Self {
        FontColor(value)
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum RegisterBase {
    Hexadecimal,
    Decimal,
    Binary,
    Mixed,
}

impl std::fmt::Display for RegisterBase {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RegisterBase::Hexadecimal => write!(f, "Hexadecimal"),
            RegisterBase::Decimal => write!(f, "Decimal"),
            RegisterBase::Binary => write!(f, "Binary"),
            RegisterBase::Mixed => write!(f, "Mixed"),
        }
    }
}

impl From<std::string::String> for RegisterBase {
    fn from(value: std::string::String) -> Self {
        match value.as_str() {
            "Hexadecimal" => RegisterBase::Hexadecimal,
            "Decimal" => RegisterBase::Decimal,
            "Binary" => RegisterBase::Binary,
            "Mixed" => RegisterBase::Mixed,
            _ => RegisterBase::default(),
        }
    }
}

impl Default for RegisterBase {
    fn default() -> Self {
        RegisterBase::Mixed
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
            highlight_color: HighlightColor::default(),
            font_color: FontColor::default(),
            tab_size: 8,
            font_size: 14,
            monaco_theme: "vs".to_string(),
            register_base: RegisterBase::default(),
            // This is true as most CS1521 students don't need to see them
            hide_uncommon_registers: true,
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
            theme: String,
        }

        crate::update_font_color(&self.font_color.0);
        crate::update_primary_color(&self.primary_color.0);
        crate::update_secondary_color(&self.secondary_color.0);
        crate::update_tertiary_color(&self.tertiary_color.0);
        crate::update_highlight_color(&self.highlight_color.0);
        crate::update_editor_options(
            JsValue::from_serde(&EditorOptions {
                font_size: self.font_size,
                theme: self.monaco_theme.clone(),
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

#[derive(Atom, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonacoCursor {
    #[serde(rename = "lineNumber")]
    pub line: u32,
    pub column: u32,
}

impl From<JsValue> for MonacoCursor {
    fn from(value: JsValue) -> Self {
        let position = value.into_serde::<MonacoCursor>().unwrap();
        MonacoCursor {
            line: position.line,
            column: position.column,
        }
    }
}
