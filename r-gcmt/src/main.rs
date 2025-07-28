use std::process::{ Command };
use clap::{ Parser, Subcommand};
use anyhow::{ Context, Result };
use dirs::cache_dir;

mod set_json;
mod handle_ticket;
mod read_env;
mod config;
use set_json::set_json;
use handle_ticket::handle_ticket;

#[derive(Parser)]
#[command(subcommand_negates_reqs = true)]
struct Cli {
    #[arg(required = true)]
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
    let cli = Cli::parse();
    // Handle subcommands
    match &cli.command {
        // If set-model: Set model value into JSON, then return
        Some(Commands::SetModel { model }) => {
            println!("model: {}", model);
            return Ok(())
        },
        // If set-ticket: Set ticket value into JSON, then return
        Some(Commands::SetTicket { ticket }) => {
            println!("ticket: {}", ticket);
            return Ok(())
        },
        None => {}
    };
    // Handle unset message
    let message = match &cli.message {
        Some(message) => message.to_string(),
        None => panic!("Message not set")
    };
    let env_var_key = "RGCMT_TICKET".to_string();
    // Define config.json_path
    let cache_dir_path = cache_dir()
        .with_context(|| format!("dirs::cache_dir() panicked"))?;
    let config_path = cache_dir_path.join(r"r-gcmt/config.json");

    if cli.debug { println!("set_env: {}, json_path: {}", cli.set_config, config_path.display()) };
    if cli.set_config {
        set_json(&cli.ticket, &config_path, &env_var_key, cli.debug)
            .with_context(|| format!("set_json failed"))?;
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
