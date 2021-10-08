use std::{fs::{self, File}, io::{Read, Write}, path::PathBuf, time::SystemTime};
use serde::{Serialize, Deserialize};

const MIPSY_DIR: &str = "mipsy";
const CONFIG_NAME: &str = "config.yaml";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct MipsyConfig {
    pub tab_size: u32,
    pub spim: bool,
}

pub enum MipsyConfigError {
    InvalidConfig(PathBuf, MipsyConfig),
}

pub fn config_path() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join(MIPSY_DIR).join(CONFIG_NAME))
}

pub fn read_config() -> Result<MipsyConfig, MipsyConfigError> {
    use DeserialiseConfigError::*;

    match try_deserialise() {
        Ok(config) => Ok(config),
        Err(NotUsingConfig) => Ok(MipsyConfig::default()),
        Err(ConfigBroken(to_path, config)) => Err(MipsyConfigError::InvalidConfig(to_path, config)),
    }
}

impl Default for MipsyConfig {
    fn default() -> Self {
        Self {
            tab_size: 8,
            spim: false,
        }
    }
}

#[derive(Debug)]
enum DeserialiseConfigError {
    NotUsingConfig,
    ConfigBroken(PathBuf, MipsyConfig),
}

fn write_default_config(path: &PathBuf) -> Result<MipsyConfig, DeserialiseConfigError> {
    use DeserialiseConfigError::*;

    let mut file = File::create(path).map_err(|_| NotUsingConfig)?;
    let default_config = MipsyConfig::default();

    let mut lines = serde_yaml::to_string(&default_config)
        .expect("cannot fail to serialise default mipsy config")
        .lines()
        .skip(1)
        .map(String::from)
        .collect::<Vec<_>>();
    lines.push(String::new());
        
    let yaml = lines.join("\n");

    file.write_all(yaml.as_bytes())
        .map_err(|_| NotUsingConfig)?;

    Ok(default_config)
}

fn try_deserialise() -> Result<MipsyConfig, DeserialiseConfigError> {
    use DeserialiseConfigError::*;

    let config_dir = dirs::config_dir().ok_or(NotUsingConfig)?;
    let mipsy_dir = config_dir.join(MIPSY_DIR);
    fs::create_dir_all(&mipsy_dir).map_err(|_| NotUsingConfig)?;

    let config_path = mipsy_dir.join(CONFIG_NAME);

    if !config_path.exists() {
        write_default_config(&config_path)?;
    }

    let mut file = File::open(&config_path).map_err(|_| NotUsingConfig)?;
    let mut contents = String::new();

    file.read_to_string(&mut contents).map_err(|_| NotUsingConfig)?;

    let config: Result<MipsyConfig, _> = serde_yaml::from_str(&contents);
    
    match config {
        Ok(config) => Ok(config),
        Err(_) => {
            let epoch = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("can you please set your system clock to *some time* after the 1970's?")
                .as_secs();

            let to_path = mipsy_dir.join(&format!("{}.{}", CONFIG_NAME, epoch));

            fs::rename(&config_path, &to_path)
                .expect("cannot rename broken config file");
            
            Err(ConfigBroken(to_path, write_default_config(&config_path)?))
        }
    }
}