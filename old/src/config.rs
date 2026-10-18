use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::editor::Mode;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ConfigKeyMaps {
    pub normal: HashMap<String, String>,
    pub insert: HashMap<String, String>,
    pub command: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigRaw {
    pub keymaps: ConfigKeyMaps,
}

const DEFAULT_CONFIG_LOCATION: &'static str = "~/.config/rv/config.toml";

impl Default for ConfigRaw {
    fn default() -> Self {
        Self {
            keymaps: Default::default(),
        }
    }
}

impl ConfigRaw {
    fn from_default() -> Result<Option<ConfigRaw>> {
        let config_dir = match dirs::config_dir() {
            Some(c) => c,
            None => return Ok(None),
        };

        let path = config_dir.join("rv/config.toml");

        if !path.exists() {
            warn!("{} does not exist", DEFAULT_CONFIG_LOCATION);
            return Ok(None);
        }

        let content = std::fs::read_to_string(path)?;
        let config: ConfigRaw = toml::from_str(&content)?;

        Ok(Some(config))
    }

    pub fn get() -> ConfigRaw {
        if let Ok(Some(config)) = ConfigRaw::from_default() {
            config
        } else {
            ConfigRaw::default()
        }
    }

    pub fn set(&mut self, mode: Mode, binding: &str, action: &str) {
        let map = match mode {
            Mode::Normal => &mut self.keymaps.normal,
            Mode::Insert => &mut self.keymaps.insert,
            Mode::Command => &mut self.keymaps.command,
        };
        map.insert(binding.to_string(), action.to_string());
    }

    pub fn debug_to_string(&self) -> Result<String> {
        Ok(toml::to_string_pretty(self)?)
    }
}
