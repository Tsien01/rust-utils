use std::env::VarError;

use anyhow::{ Result };

pub fn read_env (env_var_key: &str, debug: bool) -> Result<String> {
    if debug { println!("read_env") }
    let env_ticket_result = std::env::var(env_var_key);
    let env_ticket = match env_ticket_result {
        Ok(v) => { v },
        Err(e) => {
            match e {
                VarError::NotPresent => { panic!("Ticket value not found. Have you provided a ticket value anywhere?") },
                VarError::NotUnicode(os) => { panic!("Non-unicode values found in ticket value. OsString:{}", os.display()) }, 
            } 
        }
    };
    Ok(env_ticket)
}