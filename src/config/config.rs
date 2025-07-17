use std::collections::HashMap;
use std::env;
use std::fs;
use serde::{Deserialize, Serialize};

use crate::constants::CONFIG_FILE;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub api_key: Option<String>,
    pub default_team_id: Option<String>,
    #[serde(default)]
    pub saved_searches: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            api_key: None,
            default_team_id: None,
            saved_searches: HashMap::new(),
        }
    }
}

pub fn load_config() -> Config {
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let config_path = home_dir.join(CONFIG_FILE);

    if config_path.exists() {
        let config_str = fs::read_to_string(&config_path).expect("Failed to read config file");
        serde_json::from_str(&config_str).unwrap_or_default()
    } else {
        Config::default()
    }
}

pub fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let config_path = home_dir.join(CONFIG_FILE);

    let config_str = serde_json::to_string_pretty(config)?;
    fs::write(config_path, config_str)?;

    Ok(())
}

pub fn get_api_key() -> Result<String, Box<dyn std::error::Error>> {
    // First check environment variable
    if let Ok(key) = env::var("LINEAR_API_KEY") {
        return Ok(key);
    }

    // Then check config file
    let config = load_config();
    if let Some(key) = config.api_key {
        return Ok(key);
    }

    Err("No API key found. Set LINEAR_API_KEY environment variable or run 'linear auth' to configure.".into())
}