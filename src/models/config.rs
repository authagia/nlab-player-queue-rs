use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config{
    pub data_dir: PathBuf,
    pub ytdlp_path: String, // for Command
    pub ytdlp_option: Vec<String>,
} 