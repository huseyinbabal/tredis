use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Redis server deployment type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum ServerType {
    #[default]
    Standalone,
    Cluster,
    Sentinel,
}

impl ServerType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ServerType::Standalone => "Standalone",
            ServerType::Cluster => "Cluster",
            ServerType::Sentinel => "Sentinel",
        }
    }
}

impl std::fmt::Display for ServerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Redis server information detected on connection
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerInfo {
    pub server_type: ServerType,
    pub redis_version: String,
    pub os: String,
    pub cluster_size: Option<usize>,
    pub role: String, // master, slave, sentinel
}

/// A saved server connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub name: String,
    pub uri: String,
    #[serde(default)]
    pub info: Option<ServerInfo>,
}

/// The root config file structure stored in XDG config
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TredisConfig {
    #[serde(default)]
    pub servers: Vec<ServerConfig>,
}

impl TredisConfig {
    /// Get the config file path (XDG config dir / tredis / config.yaml)
    pub fn config_path() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("tredis").join("config.yaml")
        } else if let Some(home) = dirs::home_dir() {
            home.join(".config").join("tredis").join("config.yaml")
        } else {
            PathBuf::from("config.yaml")
        }
    }

    /// Load config from file, returns default if not exists
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(config) = serde_yaml::from_str(&content) {
                    return config;
                }
            }
        }
        Self::default()
    }

    /// Save config to file
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_yaml::to_string(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Add a new server and save
    pub fn add_server(&mut self, name: String, uri: String) -> anyhow::Result<()> {
        self.servers.push(ServerConfig {
            name,
            uri,
            info: None,
        });
        self.save()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInfo {
    pub key: String,
    pub key_type: String,
    pub ttl: i64,
    pub memory_usage: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyValue {
    String(String),
    List(Vec<String>),
    Set(Vec<String>),
    ZSet(Vec<(String, f64)>),
    Hash(HashMap<String, String>),
    Stream(Vec<StreamEntry>),
    None,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEntry {
    pub id: String,
    pub fields: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub id: String,
    pub addr: String,
    pub fd: String,
    pub name: String,
    pub age: String,
    pub idle: String,
    pub flags: String,
    pub db: String,
    pub sub: String,
    pub psub: String,
    pub multi: String,
    pub qbuf: String,
    pub qbuf_free: String,
    pub obl: String,
    pub oll: String,
    pub omem: String,
    pub events: String,
    pub cmd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlowlogEntry {
    pub id: i64,
    pub timestamp: i64,
    pub duration: i64,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AclUser {
    pub name: String,
    pub status: String,
    pub rules: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorEntry {
    pub timestamp: String,
    pub db: String,
    pub client: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamInfo {
    pub name: String,
    pub length: i64,
    pub first_entry_id: String,
    pub last_entry_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubSubChannel {
    pub name: String,
    pub subscribers: i64,
}

#[derive(Debug, Clone)]
pub struct PubSubMessage {
    pub timestamp: String,
    #[allow(dead_code)]
    pub channel: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    pub host: String,
    pub port: u16,
    pub db: i64,
    pub user: Option<String>,
    pub password: Option<String>,
    pub tls: bool,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 6379,
            db: 0,
            user: None,
            password: None,
            tls: false,
        }
    }
}
