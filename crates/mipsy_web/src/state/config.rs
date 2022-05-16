use mipsy_utils::MipsyConfig;
use serde::{Serialize, Deserialize};

#[derive(Default, PartialEq, Serialize, Deserialize)]
pub struct MipsyWebConfig {
    pub mipsy_config: MipsyConfig,
    pub ignore_breakpoints: bool,
}
