use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fs,
    io::BufReader,
    path::{Path, PathBuf},
    process::Command,
};

pub fn get_config_path() -> PathBuf {
    let mut path = home_dir().unwrap();
    path.push(".config");
    path.push(env!("CARGO_PKG_NAME"));
    path.push("settings.json");
    path
}

#[derive(Serialize, Deserialize)]
struct ExecCommand {
    program: String,
    args: String,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    exec_command: Option<ExecCommand>,
    edit_command: Option<ExecCommand>,
}

impl Config {
    pub fn new(path: &Path) -> Result<Self, Box<dyn Error>> {
        let file = fs::File::open(path);
        if let Ok(file) = file {
            let reader = BufReader::new(file);
            let config: Self = serde_json::from_reader(reader)?;
            Ok(config)
        } else {
            Ok(Config::default_self())
        }
    }
    pub fn default() -> Result<Self, Box<dyn Error>> {
        Config::new(get_config_path().as_path())
    }

    #[cfg(target_os = "windows")]
    fn default_self() -> Self {
        Self {
            exec_command: Some(ExecCommand {
                program: "explorer".to_owned(),
                args: "%p".to_owned(),
            }),
            edit_command: Some(ExecCommand {
                program: "gvim".to_owned(),
                args: "%p".to_owned(),
            }),
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn default_self() -> Self {
        Self {
            exec_command: None,
            edit_command: None,
        }
    }

    pub fn exec(&self, path: &Path, dir: &Path) {
        if let Some(command) = &self.exec_command {
            let path = path.to_string_lossy().to_string();
            let args = command.args.replace("%p", &path);
            let _ = Command::new(&command.program)
                .current_dir(dir)
                .arg(args)
                .spawn();
        }
    }

    pub fn edit(&self, path: &Path, dir: &Path) {
        if let Some(command) = &self.edit_command {
            let path = path.to_string_lossy().to_string();
            let args = command.args.replace("%p", &path);
            let _ = Command::new(&command.program)
                .current_dir(dir)
                .arg(args)
                .spawn();
        }
    }
}
