use std::{fs::{ write, create_dir_all }, path::{ PathBuf }};
use anyhow::{ Context, Result };
use crate::{ config::Config, read_env::read_env, handle_ticket::read_json };

pub fn write_json (key: &str, value: &str, json_path: &PathBuf, debug: bool) -> Result<()> {
    // Check if utility directory already exists, and create it if not
    let json_dir_path: PathBuf = json_path
        .components()
        .take(json_path.components().count() - 1)
        .collect();
    if !json_dir_path.is_dir() {
        create_dir_all(&json_dir_path).with_context(|| format!("create_dir_all failed"))?;
        if debug { println!("create_dir_all for path: {} succeeded", &json_dir_path.display()) }
    };
    let json = match read_json(json_path, debug) {
        Ok(config) => config, 
        Err(e) => {
            if debug { println!("read_json failed with error: {}", e)}
            let new_config = Config {
                ticket_number: String::new(), 
                model: String::new()
            };
            new_config
        }
    };
    let new_json = match key {
        "ticket_number" => {
            let new_config = Config {
                ticket_number: String::from(value), 
                model: String::from(&json.model),
            };
            new_config
        }, 
        "model" => {
            let new_config = Config {
                ticket_number: String::from(&json.ticket_number), 
                model: String::from(value),
            };
            new_config
        }, 
        _ => { panic!("Invalid key provided. Provided key is not a valid key of type Config.")},
    };
    // Serialise json contents
    let serialised = serde_json::to_string(&new_json).unwrap();
    // Write to file
    write(json_path, serialised)
        .with_context(|| format!("fs::write panicked"))?;
    Ok(())
}

pub fn set_ticket (ticket_arg: &str, json_path: &PathBuf, env_var_key:&str, debug: bool) -> Result<()> {
    if debug { println!("set_json") };
    // if ticket_arg is "", attempt to read from env
    let ticket: String = if !ticket_arg.is_empty() {
        ticket_arg.to_string()
    } else {
        println!("INFO: This command has been run with the --set-config flag while an exported RGCMT_TICKET variable is active in your current shell. This will cause your saved config to be continually overridden by the Env value. To avoid this behaviour, close this shell and open a new session.");
        read_env(env_var_key, debug).with_context(|| format!("read_env failed at set_json. Is a ticket number set anywhere?"))?
    };
    write_json("ticket_number", &ticket, json_path, debug)?;
    Ok(())
}