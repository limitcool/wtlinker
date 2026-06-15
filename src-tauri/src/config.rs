use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// AI 程序类型
#[derive(Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AiProgram {
    Claude,
    Codex,
    Opencode,
}

impl Default for AiProgram {
    fn default() -> Self {
        AiProgram::Codex
    }
}

impl AiProgram {
    pub fn as_str(&self) -> &'static str {
        match self {
            AiProgram::Claude => "claude",
            AiProgram::Codex => "codex",
            AiProgram::Opencode => "opencode",
        }
    }
}

/// 单条配置
#[derive(Clone, Serialize, Deserialize)]
pub struct Entry {
    pub name: String,
    pub dir: String,
    pub session: String,
    pub enabled: bool,
    pub ai: AiProgram,
}

impl Default for Entry {
    fn default() -> Self {
        Entry {
            name: String::new(),
            dir: String::new(),
            session: String::new(),
            enabled: true,
            ai: AiProgram::Codex,
        }
    }
}

/// 整个配置
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub default_flags: String,
    pub append_mode: bool,
    pub default_ai: AiProgram,
    pub entries: Vec<Entry>,
    pub shell: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            default_flags: "--dangerously-bypass-approvals-and-sandbox".to_string(),
            append_mode: true,
            default_ai: AiProgram::Codex,
            entries: Vec::new(),
            shell: "pwsh".to_string(),
        }
    }
}

/// 配置文件路径
pub fn config_path() -> PathBuf {
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    let dir = PathBuf::from(appdata).join("wtlinker");
    fs::create_dir_all(&dir).ok();
    dir.join("config.yaml")
}

pub fn load_config_file() -> Config {
    let path = config_path();
    match fs::read_to_string(&path) {
        Ok(text) => serde_yaml::from_str(&text).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

pub fn save_config_file(cfg: &Config) {
    let path = config_path();
    if let Ok(text) = serde_yaml::to_string(cfg) {
        let _ = fs::write(path, text);
    }
}