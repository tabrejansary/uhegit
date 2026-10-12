use crate::storage::config::{config_path, load_config, set_config_value};
use anyhow::Result;

pub fn run_show() -> Result<()> {
    let config = load_config()?;
    let json = serde_json::to_string_pretty(&config)?;
    println!("\n  Configuration:\n{}\n", json);
    Ok(())
}

pub fn run_path() -> Result<()> {
    let path = config_path()?;
    println!("{}", path.display());
    Ok(())
}

pub fn run_set(key: &str, value: &str) -> Result<()> {
    set_config_value(key, value)?;
    println!("  ✓ Set '{}' = '{}'", key, value);
    Ok(())
}
