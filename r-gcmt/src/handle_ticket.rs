use anyhow::{ Context, Result };
use std::path::PathBuf;
use std::fs::{ read_to_string };

use crate::{ config::Config, read_env::read_env };

pub fn read_json (json_path: &PathBuf, _debug: bool) -> Result<Config> {
    let json = read_to_string(json_path).with_context(|| format!("Error reading json_path '{}' to string", json_path.display()))?;
    
    let config: Config = serde_json::from_str(&json).with_context(|| format!("Error deserialising JSON"))?;

    Ok(config)
}

pub fn handle_ticket (json_path: &PathBuf, env_var_key: &str, debug: bool) -> Result<String> {
    if debug { println!("handle_ticket") };
    
    // Read value of ticket
    let read_json = match read_json(&json_path, debug) {
        Ok(config) => config, 
        Err(e) => {
            if debug { println!("read_json failed with error: {}, attempting to read from ENVs", e)}
            let config = Config {
                ticket_number: read_env(&env_var_key, debug).with_context(|| format!("Failed to read ticket from config.json and Shell Envs at handle_ticket. Has a ticket number been set anywhere?"))?, 
                model: String::new(),
            };
            config
        }
    };

    Ok(read_json.ticket_number)
}