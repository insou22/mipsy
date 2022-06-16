use mipsy_utils::MipsyConfig;
use serde::{Deserialize, Serialize};

#[derive(Default, PartialEq, Serialize, Deserialize)]
pub struct MipsyWebConfig {
    pub mipsy_config: MipsyConfig,
    pub ignore_breakpoints: bool,
}
