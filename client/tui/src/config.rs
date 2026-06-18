use std::path::PathBuf;
use std::sync::OnceLock;

pub static ROOT_DIR: OnceLock<PathBuf> = OnceLock::new();
pub static ASSETS_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn init_config() {
    let root_env = std::env::var("ROOT_DIR").unwrap_or_else(|_| ".".to_string());
    let assets_env = std::env::var("ASSETS_DIR").unwrap_or_else(|_| "assets".to_string());

    let absolute_root = std::fs::canonicalize(&root_env).unwrap_or_else(|_| PathBuf::from(&root_env));
    let absolute_assets = std::fs::canonicalize(&assets_env).unwrap_or_else(|_| PathBuf::from(&assets_env));

    let _ = ROOT_DIR.set(absolute_root);
    let _ = ASSETS_DIR.set(absolute_assets);
}
