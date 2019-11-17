use directories::ProjectDirs;
use std::path::PathBuf;
use std::fs::create_dir_all;

pub struct Config {
    pub path: Option<PathBuf>,
}

impl Config {
    pub fn new() -> Config {
        let config_dir: Option<PathBuf> = ProjectDirs::from("app", "parametrik", "para")
            .map(|dir| dir.config_dir().to_path_buf());
        if let Some(dir) = &config_dir {
            create_dir_all(&dir).expect("Could not create project config directory");
        }
        Config { path: config_dir }
    }
}
