use crate::model::{ConnectionConfig, KeyInfo, KeyValue, StreamEntry, TredisConfig, ServerConfig, ServerInfo, ServerType};
use crate::ui::splash::SplashState;
use crate::ui::server_dialog::ServerDialogState;
use anyhow::Result;
use redis::AsyncCommands;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Mode {
    Splash,
    Normal,
    Command,
    Describe,
    Confirm,
    Resources,
    ServerDialog,
}

#[derive(Debug, Clone)]
pub struct ResourceItem {
    pub name: String,
    pub command: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PendingActionType {
    DeleteKey,
    DeleteServer,
    DeletePattern,
}

pub struct PendingAction {
    pub key: String,
    pub action_type: PendingActionType,
    pub selected_yes: bool,
    pub matched_keys: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PaginationState {
    pub cursor: u64,
    pub next_cursor: u64,
    pub cursor_stack: Vec<u64>,
    pub total_keys: u64,
    pub page_size: usize,
}

impl Default for PaginationState {
    fn default() -> Self {
        Self {
            cursor: 0,
            next_cursor: 0,
            cursor_stack: Vec::new(),
            total_keys: 0,
            page_size: 100,
        }
    }
}

pub struct App {
    pub mode: Mode,
    pub active_resource: String,
    pub splash_state: SplashState,
    pub connection_config: ConnectionConfig,
    
    // Server configuration
    pub tredis_config: TredisConfig,
    pub current_server: Option<ServerConfig>,
    pub server_dialog_state: ServerDialogState,
    pub selected_server_index: usize,
    
    // Data - Keys
    pub all_keys: Vec<KeyInfo>,
    pub scan_result: Vec<KeyInfo>,
    pub filter_text: String,
    pub filter_active: bool,
    pub pagination: PaginationState,
    pub selected_key_index: usize,

    // Data - Clients
    pub clients: Vec<crate::model::ClientInfo>,
    pub selected_client_index: usize,

    // Data - Info
    pub info_data: Vec<(String, String)>,
    pub info_scroll: usize,
    pub info_search_active: bool,
    pub info_search_text: String,
    pub info_search_matches: Vec<usize>,  // Line indices that match
    pub info_search_current: usize,       // Current match index

    // Data - Slowlog
    pub slowlogs: Vec<crate::model::SlowlogEntry>,
    pub selected_slowlog_index: usize,

    // Data - Config
    pub configs: Vec<crate::model::ConfigEntry>,
    pub selected_config_index: usize,

    // Data - ACL
    pub acls: Vec<crate::model::AclUser>,
    pub selected_acl_index: usize,

    // Data - Monitor
    pub monitor_entries: Vec<crate::model::MonitorEntry>,
    pub selected_monitor_index: usize,
    pub monitor_scroll: usize,
    pub monitor_active: bool,
    pub monitor_task: Option<tokio::task::JoinHandle<()>>,

    // Data - Streams
    pub streams: Vec<crate::model::StreamInfo>,
    pub selected_stream_index: usize,
    pub stream_messages: Vec<crate::model::StreamEntry>,
    pub stream_scroll: usize,
    pub stream_active: bool,
    pub stream_task: Option<tokio::task::JoinHandle<()>>,
    pub stream_consumer_group: String,

    // Data - PubSub
    pub pubsub_channels: Vec<crate::model::PubSubChannel>,
    pub selected_pubsub_index: usize,
    pub pubsub_subscribe_mode: bool,
    pub pubsub_subscribe_channel: String,
    pub pubsub_subscribe_input: String,
    pub pubsub_messages: Vec<crate::model::PubSubMessage>,
    pub pubsub_task: Option<tokio::task::JoinHandle<()>>,
    
    pub should_quit: bool,
    
    // Resources Modal
    pub resources: Vec<ResourceItem>,
    pub command_text: String,
    pub command_suggestions: Vec<ResourceItem>,
    pub command_suggestion_selected: usize,
    pub command_preview: Option<String>,
    
    // Describe Data
    pub describe_data: KeyValue,
    pub describe_scroll: usize,

    // Confirm Action
    pub pending_action: Option<PendingAction>,
    pub last_key_press: Option<(crossterm::event::KeyCode, std::time::Instant)>,
    
    // Redis
    pub client: Option<redis::Client>,
    pub connection: Option<redis::aio::MultiplexedConnection>,
}

impl App {
    pub fn new() -> Self {
        let resources = vec![
            ResourceItem { name: "Servers".to_string(), command: "servers".to_string(), description: "Manage server connections" .to_string()},
            ResourceItem { name: "Keys".to_string(), command: "keys".to_string(), description: "Browse all keys" .to_string()},
            ResourceItem { name: "Streams".to_string(), command: "streams".to_string(), description: "Redis Streams" .to_string()},
            ResourceItem { name: "PubSub".to_string(), command: "pubsub".to_string(), description: "Pub/Sub channels" .to_string()},
            ResourceItem { name: "Clients".to_string(), command: "clients".to_string(), description: "Connected clients" .to_string()},
            ResourceItem { name: "Monitor".to_string(), command: "monitor".to_string(), description: "Real-time command monitor" .to_string()},
            ResourceItem { name: "Info".to_string(), command: "info".to_string(), description: "Server information" .to_string()},
            ResourceItem { name: "Config".to_string(), command: "config".to_string(), description: "Redis configuration" .to_string()},
            ResourceItem { name: "Slowlog".to_string(), command: "slowlog".to_string(), description: "Slow query log" .to_string()},
            ResourceItem { name: "ACL".to_string(), command: "acl".to_string(), description: "Access Control List" .to_string()},
        ];

        // Load existing config
        let tredis_config = TredisConfig::load();

        Self {
            mode: Mode::Splash,
            active_resource: "keys".to_string(),
            splash_state: SplashState::new(),
            connection_config: ConnectionConfig::default(),
            tredis_config,
            current_server: None,
            server_dialog_state: ServerDialogState::new(),
            selected_server_index: 0,
            all_keys: Vec::new(),
            scan_result: Vec::new(),
            filter_text: String::new(),
            filter_active: false,
            pagination: PaginationState::default(),
            selected_key_index: 0,
            clients: Vec::new(),
            selected_client_index: 0,
            info_data: Vec::new(),
            info_scroll: 0,
            info_search_active: false,
            info_search_text: String::new(),
            info_search_matches: Vec::new(),
            info_search_current: 0,
            slowlogs: Vec::new(),
            selected_slowlog_index: 0,
            configs: Vec::new(),
            selected_config_index: 0,
            acls: Vec::new(),
            selected_acl_index: 0,
            monitor_entries: Vec::new(),
            selected_monitor_index: 0,
            monitor_scroll: 0,
            monitor_active: false,
            monitor_task: None,
            streams: Vec::new(),
            selected_stream_index: 0,
            stream_messages: Vec::new(),
            stream_scroll: 0,
            stream_active: false,
            stream_task: None,
            stream_consumer_group: {
                let hostname = hostname::get()
                    .ok()
                    .and_then(|h| h.into_string().ok())
                    .unwrap_or_else(|| "unknown".to_string());
                format!("tredis_{}", hostname)
            },
            pubsub_channels: Vec::new(),
            selected_pubsub_index: 0,
            pubsub_subscribe_mode: false,
            pubsub_subscribe_channel: String::new(),
            pubsub_subscribe_input: String::new(),
            pubsub_messages: Vec::new(),
            pubsub_task: None,
            should_quit: false,
            resources: resources.clone(),
            command_text: String::new(),
            command_suggestions: resources,
            command_suggestion_selected: 0,
            command_preview: None,
            describe_data: KeyValue::None,
            describe_scroll: 0,
            pending_action: None,
            last_key_press: None,
            client: None,
            connection: None,
        }
    }

    /// Check if we need to show the server dialog (no servers configured)
    pub fn needs_server_setup(&self) -> bool {
        self.tredis_config.servers.is_empty()
    }

    /// Add a new server from the dialog and connect to it
    pub fn add_server_from_dialog(&mut self) -> Result<()> {
        let name = self.server_dialog_state.name.trim().to_string();
        let uri = self.server_dialog_state.uri.trim().to_string();
        
        if name.is_empty() {
            self.server_dialog_state.set_error("Name cannot be empty".to_string());
            return Ok(());
        }
        
        if uri.is_empty() {
            self.server_dialog_state.set_error("URI cannot be empty".to_string());
            return Ok(());
        }

        // Add server to config and save
        self.tredis_config.add_server(name.clone(), uri.clone())?;
        
        // Set as current server
        self.current_server = Some(ServerConfig { name, uri, info: None });
        
        Ok(())
    }

    /// Parse redis URI and set connection config
    pub fn set_connection_from_uri(&mut self, uri: &str) -> Result<()> {
        // Parse the URI - supports redis:// or rediss:// (TLS)
        // Format: redis[s]://[user:password@]host[:port][/db]
        let uri = uri.trim();
        
        // Check for TLS (rediss://) vs plain (redis://)
        let (tls, rest) = if let Some(rest) = uri.strip_prefix("rediss://") {
            (true, rest)
        } else if let Some(rest) = uri.strip_prefix("redis://") {
            (false, rest)
        } else {
            // No prefix, assume plain
            (false, uri)
        };
        
        self.connection_config.tls = tls;
        
        // Check for auth (user:password@)
        let (auth_part, host_part) = if let Some(at_pos) = rest.rfind('@') {
            let (auth, host) = rest.split_at(at_pos);
            (Some(auth), &host[1..]) // Skip the '@'
        } else {
            (None, rest)
        };
        
        // Parse auth if present
        if let Some(auth) = auth_part {
            if let Some(colon_pos) = auth.find(':') {
                let (user, pass) = auth.split_at(colon_pos);
                self.connection_config.user = Some(user.to_string());
                self.connection_config.password = Some(pass[1..].to_string());
            } else {
                // Just password, no user
                self.connection_config.password = Some(auth.to_string());
            }
        }
        
        // Parse host:port/db
        let (host_port, db) = if let Some(slash_pos) = host_part.find('/') {
            let (hp, d) = host_part.split_at(slash_pos);
            (hp, d[1..].parse::<i64>().unwrap_or(0))
        } else {
            (host_part, 0)
        };
        
        // Parse host and port
        if let Some(colon_pos) = host_port.rfind(':') {
            let (host, port_str) = host_port.split_at(colon_pos);
            self.connection_config.host = host.to_string();
            self.connection_config.port = port_str[1..].parse().unwrap_or(6379);
        } else {
            self.connection_config.host = host_port.to_string();
            self.connection_config.port = 6379;
        }
        
        self.connection_config.db = db;
        
        Ok(())
    }

    /// Get the current server name for display in header
    pub fn current_server_name(&self) -> &str {
        self.current_server
            .as_ref()
            .map(|s| s.name.as_str())
            .unwrap_or("No Server")
    }

    pub async fn fetch_clients(&mut self) -> Result<()> {
        if let Some(con) = &mut self.connection {
            let client_list: String = redis::cmd("CLIENT").arg("LIST").query_async(con).await?;
            let mut clients = Vec::new();

            for line in client_list.lines() {
                let mut info_map = HashMap::new();
                for part in line.split_whitespace() {
                    if let Some((key, val)) = part.split_once('=') {
                        info_map.insert(key, val);
                    }
                }

                clients.push(crate::model::ClientInfo {
                    id: info_map.get("id").unwrap_or(&"").to_string(),
                    addr: info_map.get("addr").unwrap_or(&"").to_string(),
                    fd: info_map.get("fd").unwrap_or(&"").to_string(),
                    name: info_map.get("name").unwrap_or(&"").to_string(),
                    age: info_map.get("age").unwrap_or(&"").to_string(),
                    idle: info_map.get("idle").unwrap_or(&"").to_string(),
                    flags: info_map.get("flags").unwrap_or(&"").to_string(),
                    db: info_map.get("db").unwrap_or(&"").to_string(),
                    sub: info_map.get("sub").unwrap_or(&"").to_string(),
                    psub: info_map.get("psub").unwrap_or(&"").to_string(),
                    multi: info_map.get("multi").unwrap_or(&"").to_string(),
                    qbuf: info_map.get("qbuf").unwrap_or(&"").to_string(),
                    qbuf_free: info_map.get("qbuf-free").unwrap_or(&"").to_string(),
                    obl: info_map.get("obl").unwrap_or(&"").to_string(),
                    oll: info_map.get("oll").unwrap_or(&"").to_string(),
                    omem: info_map.get("omem").unwrap_or(&"").to_string(),
                    events: info_map.get("events").unwrap_or(&"").to_string(),
                    cmd: info_map.get("cmd").unwrap_or(&"").to_string(),
                });
            }
            self.clients = clients;
        }
        Ok(())
    }

    pub fn go_to_top(&mut self) {
        self.selected_key_index = 0;
    }

    pub fn go_to_bottom(&mut self) {
        if !self.scan_result.is_empty() {
            self.selected_key_index = self.scan_result.len() - 1;
        }
    }

    pub fn describe_go_to_top(&mut self) {
        self.describe_scroll = 0;
    }

    pub fn describe_go_to_bottom(&mut self, _visible_lines: usize) {
        self.describe_scroll = 999999; 
    }

    pub async fn fetch_info(&mut self) -> Result<()> {
        if let Some(con) = &mut self.connection {
            let info: String = redis::cmd("INFO").query_async(con).await?;
            let mut info_data = Vec::new();
            for line in info.lines() {
                if line.is_empty() {
                    continue;
                }
                if line.starts_with('#') {
                    info_data.push((line.to_string(), String::new()));
                } else if let Some((key, val)) = line.split_once(':') {
                    info_data.push((key.to_string(), val.to_string()));
                }
            }
            self.info_data = info_data;
        }
        Ok(())
    }

    pub async fn fetch_slowlog(&mut self) -> Result<()> {
        if let Some(con) = &mut self.connection {
            let raw_logs: Vec<(i64, i64, i64, Vec<String>)> = redis::cmd("SLOWLOG").arg("GET").arg(100).query_async(con).await?;
            let mut slowlogs = Vec::new();

            for (id, timestamp, duration, cmd_parts) in raw_logs {
                slowlogs.push(crate::model::SlowlogEntry {
                    id,
                    timestamp,
                    duration,
                    command: cmd_parts.join(" "),
                });
            }
            self.slowlogs = slowlogs;
        }
        Ok(())
    }

    pub async fn fetch_configs(&mut self) -> Result<()> {
        if let Some(con) = &mut self.connection {
            let config_map: HashMap<String, String> = redis::cmd("CONFIG").arg("GET").arg("*").query_async(con).await?;
            let mut configs: Vec<_> = config_map.into_iter().map(|(k, v)| crate::model::ConfigEntry { key: k, value: v }).collect();
            configs.sort_by(|a, b| a.key.cmp(&b.key));
            self.configs = configs;
        }
        Ok(())
    }

    pub async fn fetch_acls(&mut self) -> Result<()> {
        if let Some(con) = &mut self.connection {
            let acl_list: Vec<String> = redis::cmd("ACL").arg("LIST").query_async(con).await?;
            let mut acls = Vec::new();

            for line in acl_list {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 && parts[0] == "user" {
                    let name = parts[1].to_string();
                    let status = parts[2].to_string();
                    let rules = parts[3..].join(" ");
                    acls.push(crate::model::AclUser { name, status, rules });
                }
            }
            self.acls = acls;
        }
        Ok(())
    }

    pub async fn connect(&mut self) -> Result<()> {
        use std::time::Duration;
        use tokio::time::timeout;
        
        // Close existing connection first (should already be closed, but just in case)
        drop(self.connection.take());
        drop(self.client.take());
        
        // Use the original URI from current_server if available (preserves auth, TLS, etc.)
        let url = if let Some(ref server) = self.current_server {
            server.uri.clone()
        } else {
            // Fallback: Build URL from connection config
            let scheme = if self.connection_config.tls { "rediss" } else { "redis" };
            
            if let Some(ref password) = self.connection_config.password {
                if let Some(ref user) = self.connection_config.user {
                    format!(
                        "{}://{}:{}@{}:{}/{}",
                        scheme, user, password,
                        self.connection_config.host,
                        self.connection_config.port,
                        self.connection_config.db
                    )
                } else {
                    format!(
                        "{}://:{}@{}:{}/{}",
                        scheme, password,
                        self.connection_config.host,
                        self.connection_config.port,
                        self.connection_config.db
                    )
                }
            } else {
                format!(
                    "{}://{}:{}/{}",
                    scheme,
                    self.connection_config.host,
                    self.connection_config.port,
                    self.connection_config.db
                )
            }
        };
        
        let client = redis::Client::open(url)?;
        
        // Use timeout for connection (30 seconds for TLS connections which can be slow)
        let connection = timeout(
            Duration::from_secs(30),
            client.get_multiplexed_async_connection()
        )
        .await
        .map_err(|_| anyhow::anyhow!("Connection timed out after 30 seconds"))??;
        
        self.client = Some(client);
        self.connection = Some(connection);
        Ok(())
    }

    /// Detect server type and info by connecting and running INFO/CLUSTER commands
    pub async fn detect_server_info(uri: &str) -> Result<ServerInfo> {
        use std::time::Duration;
        use tokio::time::timeout;
        
        let client = redis::Client::open(uri)?;
        let mut con = timeout(
            Duration::from_secs(30),
            client.get_multiplexed_async_connection()
        )
        .await
        .map_err(|_| anyhow::anyhow!("Connection timed out after 30 seconds"))??;
        
        let mut info = ServerInfo::default();
        
        // Get basic INFO
        let info_str: String = redis::cmd("INFO").query_async(&mut con).await?;
        
        for line in info_str.lines() {
            if let Some((key, val)) = line.split_once(':') {
                match key {
                    "redis_version" => info.redis_version = val.to_string(),
                    "os" => info.os = val.to_string(),
                    "role" => info.role = val.to_string(),
                    "redis_mode" => {
                        if val == "sentinel" {
                            info.server_type = ServerType::Sentinel;
                        }
                    }
                    _ => {}
                }
            }
        }
        
        // If already detected as Sentinel from INFO, return early
        if info.server_type == ServerType::Sentinel {
            return Ok(info);
        }
        
        // Check if it's a Sentinel by command (fallback)
        let sentinel_check: Result<String, _> = redis::cmd("SENTINEL")
            .arg("MASTERS")
            .query_async(&mut con)
            .await;
        
        if sentinel_check.is_ok() {
            info.server_type = ServerType::Sentinel;
            info.role = "sentinel".to_string();
            return Ok(info);
        }
        
        // Check if it's a Cluster
        let cluster_info: Result<String, _> = redis::cmd("CLUSTER")
            .arg("INFO")
            .query_async(&mut con)
            .await;
        
        if let Ok(cluster_str) = cluster_info {
            // Parse cluster info
            let mut cluster_enabled = false;
            let mut cluster_size = 0usize;
            
            for line in cluster_str.lines() {
                if let Some((key, val)) = line.split_once(':') {
                    match key {
                        "cluster_state" => {
                            if val == "ok" {
                                cluster_enabled = true;
                            }
                        }
                        "cluster_size" => {
                            cluster_size = val.parse().unwrap_or(0);
                        }
                        _ => {}
                    }
                }
            }
            
            if cluster_enabled {
                info.server_type = ServerType::Cluster;
                info.cluster_size = Some(cluster_size);
                return Ok(info);
            }
        }
        
        // Default to Standalone
        info.server_type = ServerType::Standalone;
        Ok(info)
    }

    /// Update server info in config for a specific server
    pub fn update_server_info(&mut self, server_name: &str, server_info: ServerInfo) -> Result<()> {
        if let Some(server) = self.tredis_config.servers.iter_mut().find(|s| s.name == server_name) {
            server.info = Some(server_info);
        }
        self.tredis_config.save()?;
        Ok(())
    }

    /// Delete a server from config
    pub fn delete_server(&mut self, server_name: &str) -> Result<()> {
        self.tredis_config.servers.retain(|s| s.name != server_name);
        self.tredis_config.save()?;
        Ok(())
    }

    pub async fn fetch_keys(&mut self, pattern: Option<String>) -> Result<()> {
        if let Some(con) = &mut self.connection {
            let total: u64 = redis::cmd("DBSIZE").query_async(con).await.unwrap_or(0);
            self.pagination.total_keys = total;

            let mut cmd = redis::cmd("SCAN");
            cmd.arg(self.pagination.cursor);
            
            if let Some(p) = &pattern {
                cmd.arg("MATCH").arg(format!("*{}*", p));
            }
            
            cmd.arg("COUNT").arg(self.pagination.page_size);

            let (next_cursor, keys): (u64, Vec<String>) = cmd.query_async(con).await?;
            self.pagination.next_cursor = next_cursor;

            let mut key_infos = Vec::new();
            for key in keys {
                let key_type: String = con.key_type(&key).await.unwrap_or("unknown".to_string());
                let ttl: i64 = con.ttl(&key).await.unwrap_or(-1);
                let memory = 0; 

                key_infos.push(KeyInfo {
                    key,
                    key_type,
                    ttl,
                    memory_usage: memory,
                });
            }
            
            self.all_keys = key_infos;
            self.apply_filter();
        }
        Ok(())
    }

    pub async fn next_page(&mut self) -> Result<()> {
        if self.pagination.next_cursor != 0 {
            self.pagination.cursor_stack.push(self.pagination.cursor);
            self.pagination.cursor = self.pagination.next_cursor;
            
            let pattern = if self.filter_text.is_empty() {
                None
            } else {
                Some(self.filter_text.clone())
            };
            self.fetch_keys(pattern).await?;
        }
        Ok(())
    }

    pub async fn prev_page(&mut self) -> Result<()> {
        if let Some(prev_cursor) = self.pagination.cursor_stack.pop() {
            self.pagination.cursor = prev_cursor;
            
            let pattern = if self.filter_text.is_empty() {
                None
            } else {
                Some(self.filter_text.clone())
            };
            self.fetch_keys(pattern).await?;
        }
        Ok(())
    }

    pub fn apply_filter(&mut self) {
        if self.filter_text.is_empty() {
            self.scan_result = self.all_keys.clone();
        } else {
            let filter = self.filter_text.to_lowercase();
            self.scan_result = self.all_keys
                .iter()
                .filter(|k| k.key.to_lowercase().contains(&filter))
                .cloned()
                .collect();
        }

        if self.selected_key_index >= self.scan_result.len() {
             if !self.scan_result.is_empty() {
                 self.selected_key_index = self.scan_result.len() - 1;
             } else {
                 self.selected_key_index = 0;
             }
        }
    }

    pub async fn delete_key(&mut self) -> Result<()> {
        if let Some(pending) = &self.pending_action {
             if let Some(con) = &mut self.connection {
                 let _: () = con.del(&pending.key).await?;
             }
        }
        Ok(())
    }

    /// Scan all keys matching a pattern using SCAN command
    pub async fn scan_keys_by_pattern(&mut self, pattern: &str) -> Result<Vec<String>> {
        let mut matched_keys = Vec::new();

        if let Some(con) = &mut self.connection {
            let mut cursor: u64 = 0;
            loop {
                let (next_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                    .arg(cursor)
                    .arg("MATCH")
                    .arg(pattern)
                    .arg("COUNT")
                    .arg(1000)
                    .query_async(con)
                    .await?;

                matched_keys.extend(keys);
                cursor = next_cursor;

                if cursor == 0 {
                    break;
                }
            }
        }

        Ok(matched_keys)
    }

    /// Delete all keys matching the pattern stored in pending_action
    pub async fn delete_keys_by_pattern(&mut self) -> Result<u64> {
        let mut deleted_count: u64 = 0;

        if let Some(pending) = &self.pending_action {
            if let Some(con) = &mut self.connection {
                // Delete in batches to avoid blocking Redis for too long
                for chunk in pending.matched_keys.chunks(100) {
                    if !chunk.is_empty() {
                        let count: u64 = con.del(chunk).await?;
                        deleted_count += count;
                    }
                }
            }
        }

        Ok(deleted_count)
    }

    pub async fn fetch_key_value(&mut self) -> Result<()> {
        if self.scan_result.is_empty() {
            return Ok(());
        }
        
        let key_info = &self.scan_result[self.selected_key_index];
        let key = &key_info.key;
        let key_type = &key_info.key_type;

        if let Some(con) = &mut self.connection {
            self.describe_data = match key_type.as_str() {
                "string" => {
                    let val: String = con.get(key).await.unwrap_or_else(|e| format!("Error: {}", e));
                    KeyValue::String(val)
                },
                "list" => {
                    let val: Vec<String> = con.lrange(key, 0, -1).await.unwrap_or_default();
                    KeyValue::List(val)
                },
                "set" => {
                    let val: Vec<String> = con.smembers(key).await.unwrap_or_default();
                    KeyValue::Set(val)
                },
                "zset" => {
                    let val: Vec<(String, f64)> = con.zrange_withscores(key, 0, -1).await.unwrap_or_default();
                    KeyValue::ZSet(val)
                },
                "hash" => {
                    let val: HashMap<String, String> = con.hgetall(key).await.unwrap_or_default();
                    KeyValue::Hash(val)
                },
                "stream" => {
                    let entries: Vec<(String, Vec<(String, String)>)> = 
                        redis::cmd("XRANGE").arg(key).arg("-").arg("+")
                        .query_async(con).await.unwrap_or_default();
                    
                    let stream_entries: Vec<StreamEntry> = entries.into_iter().map(|(id, fields)| {
                        let mut field_map = HashMap::new();
                        for (k, v) in fields {
                            field_map.insert(k, v);
                        }
                        StreamEntry { id, fields: field_map }
                    }).collect();
                    
                    KeyValue::Stream(stream_entries)
                },
                _ => KeyValue::Error(format!("Unsupported type: {}", key_type)),
            };
        }
        Ok(())
    }

    pub async fn fetch_stream_entries(&mut self) -> Result<()> {
        if self.streams.is_empty() {
            return Ok(());
        }
        
        let stream = &self.streams[self.selected_stream_index];
        let stream_name = &stream.name;

        if let Some(con) = &mut self.connection {
            let entries: Vec<(String, Vec<(String, String)>)> = 
                redis::cmd("XRANGE").arg(stream_name).arg("-").arg("+")
                .query_async(con).await.unwrap_or_default();
            
            let stream_entries: Vec<StreamEntry> = entries.into_iter().map(|(id, fields)| {
                let mut field_map = HashMap::new();
                for (k, v) in fields {
                    field_map.insert(k, v);
                }
                StreamEntry { id, fields: field_map }
            }).collect();
            
            self.describe_data = KeyValue::Stream(stream_entries);
        }
        Ok(())
    }

    pub fn stop_stream_consumer(&mut self) {
        self.stream_active = false;
        if let Some(task) = self.stream_task.take() {
            task.abort();
        }
        self.stream_messages.clear();
    }

    pub fn update_command_suggestions(&mut self) {
        let typed = self.command_text.to_lowercase();
        self.command_suggestions = self.resources
            .iter()
            .filter(|r| r.command.to_lowercase().contains(&typed))
            .cloned()
            .collect();
        
        if self.command_suggestion_selected >= self.command_suggestions.len() {
            self.command_suggestion_selected = 0;
        }

        self.command_preview = self.command_suggestions.first().map(|r| r.command.clone());
    }

    pub fn on_tick(&mut self) {
        if self.mode == Mode::Splash {
            self.splash_state.spinner_frame = (self.splash_state.spinner_frame + 1) % 4;
        }
    }

    /// Update info search matches based on current search text
    pub fn update_info_search(&mut self) {
        self.info_search_matches.clear();
        self.info_search_current = 0;

        if self.info_search_text.is_empty() {
            return;
        }

        let search_lower = self.info_search_text.to_lowercase();

        for (idx, (key, value)) in self.info_data.iter().enumerate() {
            if key.to_lowercase().contains(&search_lower)
                || value.to_lowercase().contains(&search_lower)
            {
                self.info_search_matches.push(idx);
            }
        }

        // Scroll to first match
        if !self.info_search_matches.is_empty() {
            self.info_scroll = self.info_search_matches[0];
        }
    }

    /// Go to next search match
    pub fn info_search_next(&mut self) {
        if self.info_search_matches.is_empty() {
            return;
        }

        self.info_search_current = (self.info_search_current + 1) % self.info_search_matches.len();
        self.info_scroll = self.info_search_matches[self.info_search_current];
    }

    /// Go to previous search match
    pub fn info_search_prev(&mut self) {
        if self.info_search_matches.is_empty() {
            return;
        }

        if self.info_search_current == 0 {
            self.info_search_current = self.info_search_matches.len() - 1;
        } else {
            self.info_search_current -= 1;
        }
        self.info_scroll = self.info_search_matches[self.info_search_current];
    }

    /// Clear info search
    pub fn clear_info_search(&mut self) {
        self.info_search_active = false;
        self.info_search_text.clear();
        self.info_search_matches.clear();
        self.info_search_current = 0;
    }

    pub fn next(&mut self) {
        if !self.scan_result.is_empty() {
            if self.selected_key_index < self.scan_result.len() - 1 {
                self.selected_key_index += 1;
            }
        }
    }

    pub fn previous(&mut self) {
        if !self.scan_result.is_empty() {
            if self.selected_key_index > 0 {
                self.selected_key_index -= 1;
            }
        }
    }

    pub fn stop_monitor(&mut self) {
        self.monitor_active = false;
        if let Some(task) = self.monitor_task.take() {
            task.abort();
        }
        self.monitor_entries.clear();
    }

    pub async fn fetch_streams(&mut self) -> Result<()> {
        if let Some(con) = &mut self.connection {
            // Get all keys that are streams
            let keys: Vec<String> = redis::cmd("KEYS").arg("*").query_async(con).await?;
            let mut streams = Vec::new();

            for key in keys {
                let key_type: String = redis::cmd("TYPE").arg(&key).query_async(con).await?;
                if key_type == "stream" {
                    let length: i64 = redis::cmd("XLEN").arg(&key).query_async(con).await.unwrap_or(0);
                    
                    // Get first and last entry IDs
                    let first: Vec<(String, Vec<(String, String)>)> = 
                        redis::cmd("XRANGE").arg(&key).arg("-").arg("+").arg("COUNT").arg(1)
                        .query_async(con).await.unwrap_or_default();
                    let last: Vec<(String, Vec<(String, String)>)> = 
                        redis::cmd("XREVRANGE").arg(&key).arg("+").arg("-").arg("COUNT").arg(1)
                        .query_async(con).await.unwrap_or_default();

                    let first_entry_id = first.get(0).map(|e| e.0.clone()).unwrap_or_else(|| "-".to_string());
                    let last_entry_id = last.get(0).map(|e| e.0.clone()).unwrap_or_else(|| "-".to_string());

                    streams.push(crate::model::StreamInfo {
                        name: key,
                        length,
                        first_entry_id,
                        last_entry_id,
                    });
                }
            }
            self.streams = streams;
        }
        Ok(())
    }

    pub async fn fetch_pubsub_channels(&mut self) -> Result<()> {
        if let Some(con) = &mut self.connection {
            // PUBSUB CHANNELS returns only channels with active subscribers
            let channels: Vec<String> = redis::cmd("PUBSUB")
                .arg("CHANNELS")
                .arg("*")  // Pattern to match all channels
                .query_async(con)
                .await
                .unwrap_or_default();
            
            let mut pubsub_channels = Vec::new();

            for channel in channels {
                // Get subscriber count for each channel
                let numsub: Vec<redis::Value> = redis::cmd("PUBSUB")
                    .arg("NUMSUB")
                    .arg(&channel)
                    .query_async(con)
                    .await
                    .unwrap_or_default();
                
                let subscribers = if numsub.len() >= 2 {
                    match &numsub[1] {
                        redis::Value::Int(n) => *n,
                        redis::Value::BulkString(s) => {
                            String::from_utf8_lossy(s).parse::<i64>().unwrap_or(0)
                        }
                        _ => 0,
                    }
                } else {
                    0
                };

                pubsub_channels.push(crate::model::PubSubChannel {
                    name: channel,
                    subscribers,
                });
            }
            self.pubsub_channels = pubsub_channels;
        }
        Ok(())
    }

}
