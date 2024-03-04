use std::{
    error::Error,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

#[derive(Debug, Default, serde::Deserialize)]
pub struct Config {
    pub format: FormatConfig,
}

#[derive(Debug, serde::Deserialize)]
pub struct FormatConfig {
    pub line_width: usize,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self { line_width: 100 }
    }
}

pub fn config_of_path(file_path: &Path) -> Result<Config, Box<dyn Error>> {
    match file_path.parent() {
        Some(dir_path) => config_of_dir(dir_path),
        None => Ok(Config::default()),
    }
}

pub fn config_of_dir(dir_path: &Path) -> Result<Config, Box<dyn Error>> {
    let config_path = find_config_file_path(dir_path);
    let config = match config_path {
        Some(config_path) => {
            let config_file = File::open(config_path)?;
            let reader = BufReader::new(config_file);
            serde_yaml::from_reader(reader)?
        }
        None => Config::default(),
    };
    Ok(config)
}

fn find_config_file_path(base: &Path) -> Option<PathBuf> {
    let config_path = base.join(".rbfmt.yml");
    if config_path.exists() {
        return Some(config_path);
    }
    if let Some(parent) = base.parent() {
        return find_config_file_path(parent);
    }
    None
}
