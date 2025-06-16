use anyhow::{ Context, Result };

pub fn read_env (debug: bool) -> Result<String> {
    if debug { println!("read_env") };

    let ticket = std::env::var("RGCMT_TICKET")
        .with_context(|| format!("read_env panicked"))?;

    Ok(ticket)
}