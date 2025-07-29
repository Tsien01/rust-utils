use std::{ process::Command, io };
use clap::{ Parser, Subcommand};
use anyhow::{ Context, Result };
use dirs::cache_dir;

mod set_ticket;
mod handle_ticket;
mod read_env;
mod config;
use set_ticket::set_ticket;
use handle_ticket::handle_ticket;

use crate::{handle_ticket::read_json, set_ticket::write_json};

#[derive(Parser)]
#[command(subcommand_negates_reqs = true)]
struct Cli {
    message: Option<String>,
    #[arg( default_value(""))]
    ticket: String, 
    #[arg(long="set-env", short, action=clap::ArgAction::SetTrue, default_value_t=false)]
    set_config: bool,
    #[arg(long, short, action)]
    debug: bool,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    SetModel {
        model: String,
    },
    SetTicket {
        ticket: String,
    },
}

// Supported cases
// CASE 1: gcmt "message" "ticket"
// CASE 2: gcmt -s "message" "ticket"
// CASE 3a || b: gcmt "message" + `export RGCMT_TICKET=1111` || -s "ticket"
// CASE 4: gcmt 
fn main() -> Result<()> {
    let env_var_key = "RGCMT_TICKET".to_string();
    let ollama_prompt = String::from("Given the following git diff output, generate a concise Git commit message summarising the changes. The message should follow conventional commit style when appropriate. Keep the message short (max 50 characters) and imperative mood (e.g., Add feature, Fix bug). git diff: ");

    let cli = Cli::parse();
    // Define config.json_path
    let cache_dir_path = cache_dir()
        .with_context(|| format!("dirs::cache_dir() panicked"))?;
    let config_path = cache_dir_path.join(r"r-gcmt/config.json");
    // Handle subcommands
    match &cli.command {
        // If set-model: Set model value into JSON, then return
        Some(Commands::SetModel { model }) => {
            let model_list = Command::new("ollama")
                .args(["show", model])
                .output();
            match model_list {
                Err(e) => panic!("ollama list panicked with error: {}", e),
                Ok(v) => {
                    if !v.stdout.is_empty() {
                        let output = String::from_utf8_lossy(&v.stdout).into_owned();
                        if cli.debug { println!("ollama show output: {}", output) }
                        write_json("model", model, &config_path, cli.debug)?;
                    } else if !v.stderr.is_empty() {
                        println!("ollama show panicked with error: {}", String::from_utf8_lossy(&v.stderr).into_owned());
                        panic!("Provided model name does not appear in ollama list output. Please ensure the provided value is a valid model for ollama run and is already downloaded onto your system via either ollama run or pull"); 
                    } else { panic!("stdout and stderr empty") };
                    println!("model: {}", model);
                },
            }
            return Ok(())
        },
        // If set-ticket: Set ticket value into JSON, then return
        Some(Commands::SetTicket { ticket }) => {
            println!("ticket: {}", ticket);
            set_ticket(ticket, &config_path, &env_var_key, cli.debug)?;
            return Ok(())
        },
        None => {}
    };
    // Handle unset message
    let message = match &cli.message {
        Some(message) => message.to_string(),
        None => {
            let diff = Command::new("git")
                .args(["diff", "--cached"])
                .output();
            let diff_output = match diff {
                Err(e) => { panic!("git diff failed with error: {}", e) },
                Ok(v) => {
                    let string_v = if !v.stdout.is_empty() {
                        String::from_utf8_lossy(&v.stdout).into_owned()
                    } else if !v.stderr.is_empty() {
                        panic!("git diff failed with error: {}", String::from_utf8_lossy(&v.stderr).into_owned());  
                    } else { panic!("stdout and stderr empty") };
                    string_v
                },
            };
            let model_name = read_json(&config_path, cli.debug).unwrap().model;
            let prompt = format!("{} {}",&ollama_prompt, &diff_output);
            println!("Message not provided. Attempting auto-generation of message via ollama...");
            let ollama_output = Command::new("ollama")
                .args(["run", &model_name, &prompt])
                .output();
            let commit_m = match ollama_output {
                Err(e) => panic!("ollama run failed with error: {}", e),
                Ok(v) => {
                    let string_v = if !v.stdout.is_empty() {
                        String::from_utf8_lossy(&v.stdout).into_owned()
                    } else if !v.stderr.is_empty() {
                        panic!("ollama run failed with error: {}", String::from_utf8_lossy(&v.stderr).into_owned());  
                    } else { panic!("stdout and stderr empty") };
                    string_v.trim().to_string()
                }
            };
            let mut input = String::new();
            println!("Generated commit message: [{}]", commit_m);
            println!("Proceed to commit with this message? Input 'y' to proceed, or 'n' to exit this command.");
            let io = io::stdin();
            
            loop {
                io.read_line(&mut input)?;
                if input.trim().eq("y") {
                    break;
                } else if input.trim().eq("n") {
                    panic!("Generated commit message rejected. Process exiting. If auto-generated messages don't line up with your changes correctly, consider changing the processing model or manually inputting.")
                } else { input.clear() }
            }
            commit_m
        }
    };

    if cli.debug { println!("set_env: {}, json_path: {}", cli.set_config, config_path.display()) };
    if cli.set_config {
        set_ticket(&cli.ticket, &config_path, &env_var_key, cli.debug)
            .with_context(|| format!("set_ticket failed"))?;
    }

    let ticket = if !cli.ticket.is_empty() {
        cli.ticket
    } else {
        handle_ticket(&config_path, &env_var_key, cli.debug)?
    };

    let commit_message = format!("{}: {}", ticket, message);
    if cli.debug {
        println!("Commit Message: '{}'", commit_message);
        println!("__________________");
    }

    let command = Command::new("git")
        .args(["commit", "-m", &commit_message])
        .output();

    match command {
        Ok(v) => {
            let commit_result = if !v.stdout.is_empty() {
                String::from_utf8_lossy(&v.stdout).into_owned()
            } else if !v.stderr.is_empty() {
                String::from_utf8_lossy(&v.stderr).into_owned()
            } else { String::from("stdout and stderr empty") };
            println!("Git commit: {}", commit_result);
        },
        Err(e) => panic!("git commit panicked with error: {}", e)
    }
    
    Ok(())
}
