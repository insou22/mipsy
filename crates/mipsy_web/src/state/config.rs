use mipsy_utils::MipsyConfig;
use serde::{Deserialize, Serialize};
use bounce::Atom;

#[derive(Clone, PartialEq, Atom, Serialize, Deserialize)]
pub struct MipsyWebConfig {
    pub mipsy_config: MipsyConfig,
    pub ignore_breakpoints: bool,
    pub primary_color: String,
    pub secondary_color: String,
    pub tab_color: String,
    pub tab_size: u32,
    pub font_size: u32,
}

impl Default for MipsyWebConfig {
    fn default() -> Self {
        Self {
            mipsy_config: MipsyConfig::default(),
            ignore_breakpoints: false,
            primary_color: "fee2e2".to_string(),
            secondary_color: "f0f0f0".to_string(),
            tab_color: "d19292".to_string(),
            tab_size: 8,
            font_size: 14,
        }
    }
}
