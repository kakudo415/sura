use std::{collections::HashMap, env, fs::File, io::BufReader};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "languageServers")]
    pub language_servers: HashMap<String, String>,
}

pub fn load() -> Result<Config> {
    let mut config = read()?;

    for (_, path) in config.language_servers.iter_mut() {
        *path = shellexpand::full(&path)?.to_string();
    }

    anyhow::Ok(config)
}

fn read() -> Result<Config> {
    let xdg_config_home = if let Ok(home) = env::var("XDG_CONFIG_HOME") {
        home
    } else {
        env::var("HOME")? + "/.config"
    };

    anyhow::Ok(serde_json::from_reader(BufReader::new(File::open(
        xdg_config_home + "/sura/config.json",
    )?))?)
}
