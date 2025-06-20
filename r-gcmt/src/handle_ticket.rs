use anyhow::{ Context, Result };
use std::path::PathBuf;
use std::fs::{ read_to_string };

use crate::{ config::Config, read_env::read_env };

fn read_json (json_path: &PathBuf, _debug: bool) -> Result<String> {
    let json = read_to_string(json_path).with_context(|| format!("Error reading json_path '{}' to string", json_path.display()))?;
    
    let config: Config = serde_json::from_str(&json).with_context(|| format!("Error deserialising JSON"))?;

    Ok(config.ticket_number)
}

pub fn handle_ticket (json_path: &PathBuf, env_var_key: &str, debug: bool) -> Result<String> {
    if debug { println!("handle_ticket") };
    
    // Read value of ticket
    let read_json = read_json(&json_path, debug);
    let ticket = if let Ok(read_json_string) = read_json {
        read_json_string
    } else {
        read_env(&env_var_key, debug).with_context(|| format!("Failed to read ticket from config.json and Shell Envs at handle_ticket. Has a ticket number been set anywhere?"))?
    };

    Ok(ticket)
}