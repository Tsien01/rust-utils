use clap::Parser;
use anyhow::{ Context, Result };
use std::process::{ Command };

mod read_env;

#[derive(Parser)]
// enum Cli {
//     Commit {

//     },
//     SetEnv {

//     }
// }
struct Cli {
    message: String,
    #[arg(long, short, default_value(""))]
    ticket: String, 
    #[arg(long, short, action)]
    debug: bool,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let mut ticket = args.ticket;
    if ticket.eq("") { ticket = read_env::read_env(args.debug)? }
    

    let commit_message = format!("{}: {}", ticket, args.message);
    if args.debug {
        println!("Commit Message, {}", commit_message);
        println!("__________________");
    }

    let command = Command::new("git")
        .args(["commit", "-m", &commit_message])
        .output()
        .with_context(|| format!("git commit panicked"))?;

    let git_message = String::from_utf8_lossy(&command.stdout).into_owned();
    println!("Git commit: {}", git_message);
    
    Ok(())
}
