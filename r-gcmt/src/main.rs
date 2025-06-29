use std::process::{ Command };
use clap::Parser;
use anyhow::{ Context, Result };
use dirs::cache_dir;

mod set_json;
mod handle_ticket;
mod read_env;
mod config;
use set_json::set_json;
use handle_ticket::handle_ticket;

#[derive(Parser)]
struct Cli {
    message: String,
    #[arg( default_value(""))]
    ticket: String, 
    #[arg(long="set-env", short, action=clap::ArgAction::SetTrue, default_value_t=false)]
    set_config: bool,
    #[arg(long, short, action)]
    debug: bool,
}

// Supported cases
// CASE 1: gcmt "message" "ticket"
// CASE 2: gcmt -s "message" "ticket"
// CASE 3a || b: gcmt "message" + `export RGCMT_TICKET=1111` || -s "ticket"
fn main() -> Result<()> {
    let args = Cli::parse();
    let env_var_key = format!("RGCMT_TICKET");
    // Define config.json_path
    let cache_dir_path = cache_dir()
        .with_context(|| format!("dirs::cache_dir() panicked"))?;
    let config_path = cache_dir_path.join(r"r-gcmt/config.json");

    if args.debug { println!("set_env: {}, json_path: {}", args.set_config, config_path.display()) };
    if args.set_config {
        set_json(&args.ticket, &config_path, &env_var_key, args.debug)
            .with_context(|| format!("set_json failed"))?;
    }

    let ticket = if !args.ticket.is_empty() {
        args.ticket
    } else {
        handle_ticket(&config_path, &env_var_key, args.debug)?
    };

    let commit_message = format!("{}: {}", ticket, args.message);
    if args.debug {
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
