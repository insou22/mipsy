use std::{fs::{self, File}, io::{Read, Write}, path::PathBuf, time::SystemTime};
use serde::{Serialize, Deserialize};

const MIPSY_DIR: &str = "mipsy";
const CONFIG_NAME: &str = "config.yaml";

/// # The user's mipsy congfiguration.
/// 
/// This usually comes from the `~/.config/mipsy/config.yaml` file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MipsyConfig {
    pub tab_size: u32,
    pub spim: bool,
}

/// # Errors arising from reading the mipsy configuration.
/// 
/// This is used to indicate that the configuration file
/// could not be read, or that the configuration file
/// is invalid.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum MipsyConfigError {
    InvalidConfig(PathBuf, MipsyConfig),
}

/// # The path of the user's mipsy configuration, if it exists.
pub fn config_path() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join(MIPSY_DIR).join(CONFIG_NAME))
}

/// # Reads the user's mipsy configuration.
/// 
/// This reads the mipsy config file,
/// and returns the configuration if it exists.
/// 
/// If the configuration file does not exist,
/// it will be created with the default configuration.
/// 
/// # Errors
/// 
/// This function may return an error if the configuration
/// file could not be read, or if the configuration file
/// is invalid.
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
