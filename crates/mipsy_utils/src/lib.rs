//! Various utility functions used
//! within the mipsy workspace

mod config;

pub use config::{
    MipsyConfig,
    MipsyConfigError,
    read_config,
    config_path,
};
