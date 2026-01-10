mod app;
mod model;
mod ui;

use app::{App, Mode, PendingAction, PendingActionType};
use model::{ServerConfig, KeyValue, ServerInfo};
use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, path::PathBuf, sync::OnceLock, time::{Duration, Instant}};
use tokio::sync::mpsc;
use futures::StreamExt;

pub const VERSION: &str = match option_env!("TREDIS_VERSION") {
    Some(v) => v,
    None => env!("CARGO_PKG_VERSION"),
};

/// Log level for TRedis
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum LogLevel {
    /// No logging
    Off,
    /// Only errors
    Error,
    /// Errors and warnings
    Warn,
    /// Info, warnings, and errors
    Info,
    /// All logs including debug
    Debug,
}

static LOG_LEVEL: OnceLock<LogLevel> = OnceLock::new();

pub fn get_log_level() -> LogLevel {
    *LOG_LEVEL.get().unwrap_or(&LogLevel::Off)
}

/// TRedis - Terminal UI for Redis
#[derive(Parser, Debug)]
#[command(name = "tredis", version = VERSION, about = "Terminal UI for Redis")]
pub struct Args {
    /// Redis host
    #[arg(short = 'H', long, default_value = "localhost")]
    pub host: String,

    /// Redis port
    #[arg(short, long, default_value = "6379")]
    pub port: u16,

    /// Redis database
    #[arg(short, long, default_value = "0")]
    pub db: i64,

    /// Log level (off, error, warn, info, debug)
    #[arg(short, long, default_value = "off", value_enum)]
    pub log_level: LogLevel,
}

pub fn get_log_path() -> PathBuf {
    if let Some(config_dir) = dirs::config_dir() {
        return config_dir.join("tredis").join("tredis.log");
    }
    if let Some(home) = dirs::home_dir() {
        return home.join(".tredis").join("tredis.log");
    }
    PathBuf::from("tredis.log")
}

#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => {
        {
            if $crate::get_log_level() >= $level {
                use std::io::Write;
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open($crate::get_log_path())
                {
                    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
                    let level_str = match $level {
                        $crate::LogLevel::Error => "ERROR",
                        $crate::LogLevel::Warn => "WARN",
                        $crate::LogLevel::Info => "INFO",
                        $crate::LogLevel::Debug => "DEBUG",
                        _ => "LOG",
                    };
                    let _ = writeln!(file, "[{}] [{}] {}", timestamp, level_str, format!($($arg)*));
                }
            }
        }
    };
}

/// Mask password in a Redis URI for safe logging
fn mask_uri(uri: &str) -> String {
    // Match pattern: redis[s]://[user:password@]host:port[/db]
    if let Some(at_pos) = uri.rfind('@') {
        // Has auth part
        let prefix_end = if uri.starts_with("rediss://") {
            9
        } else if uri.starts_with("redis://") {
            8
        } else {
            0
        };
        
        let scheme = &uri[..prefix_end];
        let auth_part = &uri[prefix_end..at_pos];
        let host_part = &uri[at_pos..]; // includes @
        
        // Mask password in auth part (user:password or just password)
        let masked_auth = if let Some(colon_pos) = auth_part.find(':') {
            let user = &auth_part[..colon_pos];
            format!("{}:****", user)
        } else {
            "****".to_string()
        };
        
        format!("{}{}{}", scheme, masked_auth, host_part)
    } else {
        // No auth, return as-is
        uri.to_string()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();
    
    // Set log level globally
    let _ = LOG_LEVEL.set(args.log_level);
    
    // Ensure log directory exists
    if let Some(parent) = get_log_path().parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    log!(LogLevel::Info, "TRedis v{} started", VERSION);
    log!(LogLevel::Info, "Log level: {:?}", args.log_level);
    
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    // Don't capture mouse events so users can select text normally
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    
    // Create a channel for async events (like connection success)
    let (tx, mut rx) = mpsc::channel(100);
    
    // Channel for monitor task control
    let (monitor_tx, mut monitor_rx) = mpsc::channel::<bool>(1);

    // Check if we need to show server dialog (no servers configured and no CLI args override)
    let has_cli_override = args.host != "localhost" || args.port != 6379 || args.db != 0;
    
    if has_cli_override {
        // User provided CLI args, use them directly
        app.connection_config.host = args.host;
        app.connection_config.port = args.port;
        app.connection_config.db = args.db;
        app.current_server = Some(ServerConfig {
            name: format!("{}:{}", app.connection_config.host, app.connection_config.port),
            uri: format!("redis://{}:{}/{}", app.connection_config.host, app.connection_config.port, app.connection_config.db),
            info: None,
        });
        
        // Spawn connection task
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(500)).await;
            let _ = tx_clone.send(AppEvent::Progress("Connecting to Redis...".to_string())).await;
            let _ = tx_clone.send(AppEvent::Connect).await;
        });
    } else if app.needs_server_setup() {
        // No servers configured, show dialog
        app.mode = Mode::ServerDialog;
    } else {
        // Use first saved server
        let server = app.tredis_config.servers[0].clone();
        app.current_server = Some(server.clone());
        let _ = app.set_connection_from_uri(&server.uri);
        
        // Spawn connection task
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(500)).await;
            let _ = tx_clone.send(AppEvent::Progress("Connecting to Redis...".to_string())).await;
            let _ = tx_clone.send(AppEvent::Connect).await;
        });
    }

    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = crossterm::event::read()? {
                // Global quit
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    app.should_quit = true;
                }
                
                // Mode specific key handling
                match app.mode {
                    Mode::Normal => {
                        if app.info_search_active {
                            // Info search mode - typing search query
                            match key.code {
                                KeyCode::Enter | KeyCode::Esc => {
                                    // Close search input but keep results highlighted
                                    app.info_search_active = false;
                                }
                                KeyCode::Backspace => {
                                    app.info_search_text.pop();
                                    app.update_info_search();
                                }
                                KeyCode::Char(c) => {
                                    app.info_search_text.push(c);
                                    app.update_info_search();
                                }
                                _ => {}
                            }
                        } else if app.filter_active {
                            match key.code {
                                KeyCode::Enter => {
                                    app.filter_active = false;
                                    // Search on Enter - Reset pagination
                                    app.pagination.cursor = 0;
                                    app.pagination.cursor_stack.clear();
                                    if let Err(e) = app.fetch_keys(Some(app.filter_text.clone())).await {
                                        eprintln!("Search error: {}", e);
                                    }
                                }
                                KeyCode::Esc => {
                                    app.filter_text.clear();
                                    app.filter_active = false;
                                    // Reset to default view - Reset pagination
                                    app.pagination.cursor = 0;
                                    app.pagination.cursor_stack.clear();
                                    if let Err(e) = app.fetch_keys(None).await {
                                        eprintln!("Error fetching keys: {}", e);
                                    }
                                }
                                KeyCode::Backspace => {
                                    app.filter_text.pop();
                                    app.apply_filter();
                                }
                                KeyCode::Char(c) => {
                                    app.filter_text.push(c);
                                    app.apply_filter();
                                }
                                _ => {}
                            }
                        } else if app.pubsub_subscribe_mode {
                            // PubSub subscribe mode - input or listening
                            match key.code {
                                KeyCode::Esc | KeyCode::Char('q') => {
                                    // Stop subscription
                                    if let Some(task) = app.pubsub_task.take() {
                                        task.abort();
                                    }
                                    app.pubsub_subscribe_mode = false;
                                    app.pubsub_subscribe_channel.clear();
                                    app.pubsub_subscribe_input.clear();
                                    app.pubsub_messages.clear();
                                }
                                KeyCode::Enter => {
                                    if app.pubsub_subscribe_channel.is_empty() && !app.pubsub_subscribe_input.is_empty() {
                                        // Start subscription
                                        let channel = app.pubsub_subscribe_input.clone();
                                        app.pubsub_subscribe_channel = channel.clone();
                                        app.pubsub_subscribe_input.clear();
                                        app.pubsub_messages.clear();
                                        
                                        // Start pubsub listener task
                                        let uri = if let Some(ref server) = app.current_server {
                                            server.uri.clone()
                                        } else {
                                            let scheme = if app.connection_config.tls { "rediss" } else { "redis" };
                                            format!("{}://{}:{}/{}", scheme, app.connection_config.host, app.connection_config.port, app.connection_config.db)
                                        };
                                        let tx_clone = tx.clone();
                                        
                                        let task = tokio::spawn(async move {
                                            if let Ok(client) = redis::Client::open(uri) {
                                                if let Ok(mut pubsub) = client.get_async_pubsub().await {
                                                    let _ = pubsub.subscribe(&channel).await;
                                                    let mut pubsub_stream = pubsub.on_message();
                                                    
                                                    while let Some(msg) = pubsub_stream.next().await {
                                                        let payload: String = msg.get_payload().unwrap_or_default();
                                                        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                                                        
                                                        let entry = model::PubSubMessage {
                                                            timestamp,
                                                            channel: channel.clone(),
                                                            message: payload,
                                                        };
                                                        let _ = tx_clone.send(AppEvent::PubSubMessage(entry)).await;
                                                    }
                                                }
                                            }
                                        });
                                        
                                        app.pubsub_task = Some(task);
                                    }
                                }
                                KeyCode::Backspace => {
                                    if app.pubsub_subscribe_channel.is_empty() {
                                        app.pubsub_subscribe_input.pop();
                                    }
                                }
                                KeyCode::Char(c) => {
                                    if app.pubsub_subscribe_channel.is_empty() {
                                        app.pubsub_subscribe_input.push(c);
                                    }
                                }
                                _ => {}
                            }
                        } else {
                             let mut handled_g = false;
                             match key.code {
                                KeyCode::Esc => {
                                    // Clear info search if active
                                    if app.active_resource == "info" && !app.info_search_text.is_empty() {
                                        app.clear_info_search();
                                    }
                                    // Stop stream consumer if active
                                    else if app.stream_active {
                                        app.stop_stream_consumer();
                                    }
                                }
                                KeyCode::Char('q') => app.should_quit = true,
                                KeyCode::Char('j') | KeyCode::Down => {
                                    match app.active_resource.as_str() {
                                        "servers" => {
                                            if !app.tredis_config.servers.is_empty() && app.selected_server_index < app.tredis_config.servers.len() - 1 {
                                                app.selected_server_index += 1;
                                            }
                                        }
                                        "clients" => {
                                            if !app.clients.is_empty() && app.selected_client_index < app.clients.len() - 1 {
                                                app.selected_client_index += 1;
                                            }
                                        }
                                        "info" => {
                                            app.info_scroll = app.info_scroll.saturating_add(1);
                                        }
                                        "slowlog" => {
                                            if !app.slowlogs.is_empty() && app.selected_slowlog_index < app.slowlogs.len() - 1 {
                                                app.selected_slowlog_index += 1;
                                            }
                                        }
                                        "config" => {
                                            if !app.configs.is_empty() && app.selected_config_index < app.configs.len() - 1 {
                                                app.selected_config_index += 1;
                                            }
                                        }
                                        "acl" => {
                                            if !app.acls.is_empty() && app.selected_acl_index < app.acls.len() - 1 {
                                                app.selected_acl_index += 1;
                                            }
                                        }
                                        "monitor" => {
                                            if !app.monitor_entries.is_empty() && app.selected_monitor_index < app.monitor_entries.len() - 1 {
                                                app.selected_monitor_index += 1;
                                                // Only scroll if needed (selected item goes out of view)
                                                if app.selected_monitor_index >= app.monitor_scroll + 10 {
                                                    app.monitor_scroll = app.monitor_scroll.saturating_add(1);
                                                }
                                            }
                                        }
                                        "streams" => {
                                            if !app.streams.is_empty() && app.selected_stream_index < app.streams.len() - 1 {
                                                app.selected_stream_index += 1;
                                            }
                                        }
                                        "pubsub" => {
                                            if !app.pubsub_channels.is_empty() && app.selected_pubsub_index < app.pubsub_channels.len() - 1 {
                                                app.selected_pubsub_index += 1;
                                            }
                                        }
                                        _ => app.next(),
                                    }
                                }
                                KeyCode::Char('k') | KeyCode::Up => {
                                    match app.active_resource.as_str() {
                                        "servers" => {
                                            if app.selected_server_index > 0 {
                                                app.selected_server_index -= 1;
                                            }
                                        }
                                        "clients" => {
                                            if app.selected_client_index > 0 {
                                                app.selected_client_index -= 1;
                                            }
                                        }
                                        "info" => {
                                            app.info_scroll = app.info_scroll.saturating_sub(1);
                                        }
                                        "slowlog" => {
                                            if app.selected_slowlog_index > 0 {
                                                app.selected_slowlog_index -= 1;
                                            }
                                        }
                                        "config" => {
                                            if app.selected_config_index > 0 {
                                                app.selected_config_index -= 1;
                                            }
                                        }
                                        "acl" => {
                                            if app.selected_acl_index > 0 {
                                                app.selected_acl_index -= 1;
                                            }
                                        }
                                        "monitor" => {
                                            if app.selected_monitor_index > 0 {
                                                app.selected_monitor_index -= 1;
                                                // Only scroll if needed (selected item goes out of view)
                                                if app.selected_monitor_index < app.monitor_scroll {
                                                    app.monitor_scroll = app.monitor_scroll.saturating_sub(1);
                                                }
                                            }
                                        }
                                        "streams" => {
                                            if app.selected_stream_index > 0 {
                                                app.selected_stream_index -= 1;
                                            }
                                        }
                                        "pubsub" => {
                                            if app.selected_pubsub_index > 0 {
                                                app.selected_pubsub_index -= 1;
                                            }
                                        }
                                        _ => app.previous(),
                                    }
                                }
                                KeyCode::Char('g') => {
                                    if let Some((KeyCode::Char('g'), last_time)) = app.last_key_press {
                                        if last_time.elapsed() < Duration::from_millis(250) {
                                            app.go_to_top();
                                            app.last_key_press = None;
                                        } else {
                                            app.last_key_press = Some((KeyCode::Char('g'), Instant::now()));
                                        }
                                    } else {
                                        app.last_key_press = Some((KeyCode::Char('g'), Instant::now()));
                                    }
                                    handled_g = true;
                                }
                                KeyCode::Char('G') | KeyCode::End => app.go_to_bottom(),
                                KeyCode::Home => app.go_to_top(),
                                KeyCode::Char(']') => {
                                    if let Err(e) = app.next_page().await {
                                        eprintln!("Error next page: {}", e);
                                    }
                                }
                                KeyCode::Char('[') => {
                                    if let Err(e) = app.prev_page().await {
                                        eprintln!("Error prev page: {}", e);
                                    }
                                }
                                KeyCode::Char('R') => {
                                    match app.active_resource.as_str() {
                                        "clients" => { let _ = app.fetch_clients().await; }
                                        "info" => { let _ = app.fetch_info().await; }
                                        "slowlog" => { let _ = app.fetch_slowlog().await; }
                                        "config" => { let _ = app.fetch_configs().await; }
                                        "acl" => { let _ = app.fetch_acls().await; }
                                        "monitor" => { /* Monitor is real-time, cleared on refresh */ app.monitor_entries.clear(); }
                                        "streams" => { let _ = app.fetch_streams().await; }
                                        "pubsub" => { let _ = app.fetch_pubsub_channels().await; }
                                        _ => { let _ = app.fetch_keys(None).await; }
                                    }
                                }
                                KeyCode::Char(':') => {
                                    app.mode = Mode::Resources;
                                    app.command_text.clear();
                                    app.command_suggestion_selected = 0;
                                    app.update_command_suggestions();
                                }
                                KeyCode::Char('/') => {
                                    if app.active_resource == "keys" {
                                        // Filter for keys resource
                                        app.filter_active = true;
                                        app.filter_text.clear();
                                        app.apply_filter();
                                    } else if app.active_resource == "info" {
                                        // Search for info resource
                                        app.info_search_active = true;
                                        app.info_search_text.clear();
                                        app.info_search_matches.clear();
                                        app.info_search_current = 0;
                                    }
                                }
                                KeyCode::Char('n') => {
                                    // Next search match (vim-style) - only for info
                                    if app.active_resource == "info" && !app.info_search_text.is_empty() {
                                        app.info_search_next();
                                    }
                                }
                                KeyCode::Char('N') => {
                                    // Previous search match (vim-style with Shift) - only for info
                                    if app.active_resource == "info" && !app.info_search_text.is_empty() {
                                        app.info_search_prev();
                                    }
                                }
                                KeyCode::Char('s') => {
                                    // Subscribe to channel (in pubsub view)
                                    if app.active_resource == "pubsub" {
                                        app.pubsub_subscribe_mode = true;
                                        app.pubsub_subscribe_input.clear();
                                        app.pubsub_subscribe_channel.clear();
                                        app.pubsub_messages.clear();
                                    }
                                }
                                KeyCode::Char('c') => {
                                    // Connect to server (in servers view)
                                    if app.active_resource == "servers" && !app.tredis_config.servers.is_empty() {
                                        log!(LogLevel::Info, "[CONNECT] 'c' pressed in servers view");
                                        log!(LogLevel::Info, "[CONNECT] Selected server index: {}", app.selected_server_index);
                                        log!(LogLevel::Info, "[CONNECT] Total servers: {}", app.tredis_config.servers.len());
                                        
                                        let server = app.tredis_config.servers[app.selected_server_index].clone();
                                        log!(LogLevel::Info, "[CONNECT] Server name: {}, URI: {}", server.name, mask_uri(&server.uri));
                                        
                                        app.current_server = Some(server.clone());
                                        log!(LogLevel::Info, "[CONNECT] Set current_server");
                                        
                                        if let Err(e) = app.set_connection_from_uri(&server.uri) {
                                            log!(LogLevel::Error, "[CONNECT] Invalid URI error: {}", e);
                                            eprintln!("Invalid URI: {}", e);
                                        } else {
                                            log!(LogLevel::Info, "[CONNECT] URI parsed successfully");
                                            log!(LogLevel::Info, "[CONNECT] Host: {}, Port: {}, DB: {}", 
                                                app.connection_config.host, 
                                                app.connection_config.port, 
                                                app.connection_config.db);
                                            
                                            // Close existing connection properly before switching servers
                                            if let Some(conn) = app.connection.take() {
                                                drop(conn);
                                                log!(LogLevel::Info, "[CONNECT] Dropped existing connection");
                                            }
                                            if let Some(client) = app.client.take() {
                                                drop(client);
                                                log!(LogLevel::Info, "[CONNECT] Dropped existing client");
                                            }
                                            // Small delay to ensure connection is fully closed
                                            tokio::time::sleep(Duration::from_millis(100)).await;
                                            log!(LogLevel::Info, "[CONNECT] Reset client and connection");
                                            
                                            app.mode = Mode::Splash;
                                            app.splash_state = crate::ui::splash::SplashState::new();
                                            log!(LogLevel::Info, "[CONNECT] Set mode to Splash");
                                            
                                            // Spawn connection task with server info detection
                                            let tx_clone = tx.clone();
                                            let uri_clone = server.uri.clone();
                                            let name_clone = server.name.clone();
                                            let needs_detection = server.info.is_none();
                                            log!(LogLevel::Info, "[CONNECT] Spawning connection task, needs_detection={}", needs_detection);
                                            tokio::spawn(async move {
                                                log!(LogLevel::Info, "[CONNECT-TASK] Task started");
                                                tokio::time::sleep(Duration::from_millis(500)).await;
                                                
                                                // Detect server info if not already known
                                                if needs_detection {
                                                    let _ = tx_clone.send(AppEvent::Progress("Detecting server type...".to_string())).await;
                                                    if let Ok(info) = App::detect_server_info(&uri_clone).await {
                                                        let _ = tx_clone.send(AppEvent::ServerInfoDetected { 
                                                            server_name: name_clone, 
                                                            info 
                                                        }).await;
                                                    }
                                                }
                                                
                                                log!(LogLevel::Info, "[CONNECT-TASK] Sending Progress event");
                                                let _ = tx_clone.send(AppEvent::Progress("Connecting to Redis...".to_string())).await;
                                                log!(LogLevel::Info, "[CONNECT-TASK] Sending Connect event");
                                                let _ = tx_clone.send(AppEvent::Connect).await;
                                                log!(LogLevel::Info, "[CONNECT-TASK] Task completed");
                                            });
                                        }
                                    }
                                    // Start stream consumer
                                    else if app.active_resource == "streams" && !app.streams.is_empty() {
                                        eprintln!("[MAIN] Starting stream consumer...");
                                        app.stream_active = true;
                                        app.stream_messages.clear();
                                        
                                        let stream = app.streams[app.selected_stream_index].clone();
                                        let stream_name = stream.name.clone();
                                        let consumer_group = app.stream_consumer_group.clone();
                                        let config = app.connection_config.clone();
                                        let tx_clone = tx.clone();
                                        
                                        log!(LogLevel::Debug, "[MAIN] Spawning consumer task for stream: {}", stream_name);
                                        let task = tokio::spawn(async move {
                                            log!(LogLevel::Debug, "[TASK] Consumer task started for stream: {}", stream_name);
                                            use redis::AsyncCommands;
                                            
                                            log!(LogLevel::Debug, "[TASK] Connecting to Redis...");
                                            if let Ok(client) = redis::Client::open(format!("redis://{}:{}/{}", config.host, config.port, config.db)) {
                                                log!(LogLevel::Debug, "[TASK] Client created, getting connection...");
                                                if let Ok(mut con) = client.get_multiplexed_async_connection().await {
                                                    log!(LogLevel::Info, "[TASK] *** Connection established! ***");
                                                    // Create consumer group (ignore error if exists)
                                                    log!(LogLevel::Debug, "[TASK] Creating consumer group: {}", consumer_group);
                                                    let result: Result<String, _> = redis::cmd("XGROUP")
                                                        .arg("CREATE")
                                                        .arg(&stream_name)
                                                        .arg(&consumer_group)
                                                        .arg("0")
                                                        .arg("MKSTREAM")
                                                        .query_async(&mut con)
                                                        .await;
                                                    log!(LogLevel::Debug, "[TASK] XGROUP CREATE result: {:?}", result);
                                                    
                                                    // Get hostname for consumer name
                                                    let hostname = hostname::get()
                                                        .ok()
                                                        .and_then(|h| h.into_string().ok())
                                                        .unwrap_or_else(|| "unknown".to_string());
                                                    let consumer_name = format!("tredis_{}", hostname);
                                                    
                                                    log!(LogLevel::Info, "[TASK] *** Starting XREADGROUP loop with consumer: {} ***", consumer_name);
                                                    
                                                    // Start consuming messages (polling mode - no BLOCK)
                                                    loop {
                                                        let result: Result<Vec<(String, Vec<(String, Vec<(String, String)>)>)>, _> = 
                                                            redis::cmd("XREADGROUP")
                                                            .arg("GROUP")
                                                            .arg(&consumer_group)
                                                            .arg(&consumer_name)
                                                            .arg("COUNT")
                                                            .arg(10) // Read up to 10 messages at a time
                                                            .arg("STREAMS")
                                                            .arg(&stream_name)
                                                            .arg(">")
                                                            .query_async(&mut con)
                                                            .await;
                                                        
                                                        // Sleep 500ms between polls to avoid busy loop
                                                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                                                        
                                                        match result {
                                                            Ok(streams) => {
                                                                if !streams.is_empty() {
                                                                    log!(LogLevel::Info, "[CONSUMER] *** Received {} streams ***", streams.len());
                                                                }
                                                                for (stream_key, messages) in streams {
                                                                    if !messages.is_empty() {
                                                                        log!(LogLevel::Info, "[CONSUMER] Stream: {}, Messages: {}", stream_key, messages.len());
                                                                    }
                                                                for (entry_id, fields) in messages {
                                                                    let mut field_map = std::collections::HashMap::new();
                                                                    for (k, v) in fields {
                                                                        field_map.insert(k, v);
                                                                    }
                                                                    
                                                                    log!(LogLevel::Info, "[CONSUMER] Entry ID: {}, Fields: {:?}", entry_id, field_map);
                                                                    
                                                                    let entry = model::StreamEntry {
                                                                        id: entry_id.clone(),
                                                                        fields: field_map,
                                                                    };
                                                                    
                                                                    log!(LogLevel::Info, "[CONSUMER] Sending StreamMessage event to channel");
                                                                    let _ = tx_clone.send(AppEvent::StreamMessage(entry)).await;
                                                                    
                                                                    // ACK the message
                                                                    let _: Result<i64, _> = redis::cmd("XACK")
                                                                        .arg(&stream_name)
                                                                        .arg(&consumer_group)
                                                                        .arg(&entry_id)
                                                                        .query_async(&mut con)
                                                                        .await;
                                                                    }
                                                                }
                                                            }
                                                            Err(e) => {
                                                                // Timeout is normal - it means no new messages
                                                                let err_str = format!("{:?}", e);
                                                                if !err_str.contains("timed out") {
                                                                    log!(LogLevel::Error, "[CONSUMER] *** XREADGROUP error (breaking loop): {:?} ***", e);
                                                                    // Only break on real errors, not timeout
                                                                    break;
                                                                }
                                                                // Timeout is normal, continue silently
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        });
                                        
                                        app.stream_task = Some(task);
                                    }
                                }
                                KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                    // Delete server (in servers view)
                                    if app.active_resource == "servers" && !app.tredis_config.servers.is_empty() {
                                        let server = &app.tredis_config.servers[app.selected_server_index];
                                        app.pending_action = Some(PendingAction {
                                            key: server.name.clone(),
                                            action_type: PendingActionType::DeleteServer,
                                            selected_yes: false,
                                        });
                                        app.mode = Mode::Confirm;
                                    }
                                    // Delete key (in keys view)
                                    else if !app.scan_result.is_empty() {
                                        let key_info = &app.scan_result[app.selected_key_index];
                                        app.pending_action = Some(PendingAction {
                                            key: key_info.key.clone(),
                                            action_type: PendingActionType::DeleteKey,
                                            selected_yes: false,
                                        });
                                        app.mode = Mode::Confirm;
                                    }
                                }
                                KeyCode::Char('a') => {
                                    // Add new server (only in servers view)
                                    if app.active_resource == "servers" {
                                        app.server_dialog_state = crate::ui::server_dialog::ServerDialogState::new();
                                        app.mode = Mode::ServerDialog;
                                    }
                                }
                                KeyCode::Char('d') => {
                                    // Describe for servers shows connection details
                                    if app.active_resource == "servers" && !app.tredis_config.servers.is_empty() {
                                        let server = &app.tredis_config.servers[app.selected_server_index];
                                        app.describe_data = KeyValue::String(format_server_details(server));
                                        app.mode = Mode::Describe;
                                        app.describe_scroll = 0;
                                    } else if app.active_resource == "keys" && !app.scan_result.is_empty() {
                                        if let Err(e) = app.fetch_key_value().await {
                                             eprintln!("Error fetching value: {}", e);
                                        } else {
                                             app.mode = Mode::Describe;
                                             app.describe_scroll = 0;
                                        }
                                    } else if app.active_resource == "streams" && !app.streams.is_empty() {
                                        if let Err(e) = app.fetch_stream_entries().await {
                                             eprintln!("Error fetching stream entries: {}", e);
                                        } else {
                                             app.mode = Mode::Describe;
                                             app.describe_scroll = 0;
                                        }
                                    }
                                }
                                KeyCode::Enter => {
                                    // Handle Enter based on active resource
                                    if app.active_resource == "servers" && !app.tredis_config.servers.is_empty() {
                                        // Connect to selected server
                                        let server = app.tredis_config.servers[app.selected_server_index].clone();
                                        app.current_server = Some(server.clone());
                                        if let Err(e) = app.set_connection_from_uri(&server.uri) {
                                            eprintln!("Invalid URI: {}", e);
                                        } else {
                                            // Reset connection and go to splash
                                            app.client = None;
                                            app.connection = None;
                                            app.mode = Mode::Splash;
                                            app.splash_state = crate::ui::splash::SplashState::new();
                                            
                                            // Spawn connection task
                                            let tx_clone = tx.clone();
                                            tokio::spawn(async move {
                                                tokio::time::sleep(Duration::from_millis(500)).await;
                                                let _ = tx_clone.send(AppEvent::Progress("Connecting to Redis...".to_string())).await;
                                                let _ = tx_clone.send(AppEvent::Connect).await;
                                            });
                                        }
                                    } else if app.active_resource == "keys" && !app.scan_result.is_empty() {
                                        if let Err(e) = app.fetch_key_value().await {
                                             eprintln!("Error fetching value: {}", e);
                                        } else {
                                             app.mode = Mode::Describe;
                                             app.describe_scroll = 0;
                                        }
                                    } else if app.active_resource == "streams" && !app.streams.is_empty() {
                                        if let Err(e) = app.fetch_stream_entries().await {
                                             eprintln!("Error fetching stream entries: {}", e);
                                        } else {
                                             app.mode = Mode::Describe;
                                             app.describe_scroll = 0;
                                        }
                                    }
                                }
                                _ => {}
                             }
                             if !handled_g {
                                 app.last_key_press = None;
                             }
                        }
                    }
                    Mode::Confirm => {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                                app.pending_action = None;
                                app.mode = Mode::Normal;
                            }
                            KeyCode::Left | KeyCode::Right | KeyCode::Tab | KeyCode::Char('h') | KeyCode::Char('l') => {
                                if let Some(ref mut pending) = app.pending_action {
                                    pending.selected_yes = !pending.selected_yes;
                                }
                            }
                            KeyCode::Enter => {
                                if let Some(ref pending) = app.pending_action {
                                    if pending.selected_yes {
                                        match pending.action_type {
                                            PendingActionType::DeleteKey => {
                                                if let Err(e) = app.delete_key().await {
                                                    eprintln!("Error deleting key: {}", e);
                                                }
                                                // Refresh keys
                                                let _ = app.fetch_keys(None).await;
                                            }
                                            PendingActionType::DeleteServer => {
                                                let server_name = pending.key.clone();
                                                if let Err(e) = app.delete_server(&server_name) {
                                                    eprintln!("Error deleting server: {}", e);
                                                }
                                                // Reset selection if needed
                                                if app.selected_server_index >= app.tredis_config.servers.len() && app.selected_server_index > 0 {
                                                    app.selected_server_index -= 1;
                                                }
                                                // If deleted server was current, clear current
                                                if let Some(ref current) = app.current_server {
                                                    if current.name == server_name {
                                                        app.current_server = None;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                app.pending_action = None;
                                app.mode = Mode::Normal;
                            }
                            _ => {}
                        }
                    }
                    Mode::Describe => {
                        let mut handled_g = false;
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
                                app.mode = Mode::Normal;
                            }
                            KeyCode::Char('j') | KeyCode::Down => {
                                app.describe_scroll = app.describe_scroll.saturating_add(1);
                            }
                            KeyCode::Char('k') | KeyCode::Up => {
                                app.describe_scroll = app.describe_scroll.saturating_sub(1);
                            }
                            KeyCode::Char('g') => {
                                if let Some((KeyCode::Char('g'), last_time)) = app.last_key_press {
                                    if last_time.elapsed() < Duration::from_millis(250) {
                                        app.describe_go_to_top();
                                        app.last_key_press = None;
                                    } else {
                                        app.last_key_press = Some((KeyCode::Char('g'), Instant::now()));
                                    }
                                } else {
                                    app.last_key_press = Some((KeyCode::Char('g'), Instant::now()));
                                }
                                handled_g = true;
                            }
                            KeyCode::Char('G') | KeyCode::End => {
                                app.describe_go_to_bottom(0);
                            }
                            KeyCode::Home => app.describe_go_to_top(),
                            _ => {}
                        }
                        if !handled_g {
                            app.last_key_press = None;
                        }
                    }
                    Mode::Resources => {
                        match key.code {
                            KeyCode::Esc => {
                                app.mode = Mode::Normal;
                                app.command_text.clear();
                                app.update_command_suggestions();
                            }
                            KeyCode::Backspace => {
                                app.command_text.pop();
                                app.update_command_suggestions();
                            }
                            KeyCode::Down => {
                                if !app.command_suggestions.is_empty() {
                                    app.command_suggestion_selected = (app.command_suggestion_selected + 1) % app.command_suggestions.len();
                                }
                            }
                            KeyCode::Up => {
                                if !app.command_suggestions.is_empty() {
                                    if app.command_suggestion_selected > 0 {
                                        app.command_suggestion_selected -= 1;
                                    } else {
                                        app.command_suggestion_selected = app.command_suggestions.len() - 1;
                                    }
                                }
                            }
                            KeyCode::Right | KeyCode::Tab => {
                                if !app.command_suggestions.is_empty() {
                                    if let Some(selected) = app.command_suggestions.get(app.command_suggestion_selected) {
                                        app.command_text = selected.command.clone();
                                        app.update_command_suggestions();
                                    }
                                }
                            }
                            KeyCode::Char(c) => {
                                app.command_text.push(c);
                                app.update_command_suggestions();
                            }
                            KeyCode::Enter => {
                                if let Some(selected) = app.command_suggestions.get(app.command_suggestion_selected).cloned() {
                                    // Stop monitor/pubsub/stream consumers if switching away from them
                                    if app.active_resource == "monitor" && selected.command != "monitor" {
                                        app.stop_monitor();
                                    }
                                    if app.active_resource == "pubsub" && selected.command != "pubsub" {
                                        // Stop pubsub subscription if switching away
                                        if let Some(task) = app.pubsub_task.take() {
                                            task.abort();
                                        }
                                        app.pubsub_subscribe_mode = false;
                                        app.pubsub_subscribe_channel.clear();
                                        app.pubsub_messages.clear();
                                    }
                                    if app.active_resource == "streams" && selected.command != "streams" {
                                        app.stop_stream_consumer();
                                    }
                                    
                                    app.active_resource = selected.command.clone();
                                    app.mode = Mode::Normal;
                                    app.command_text.clear();
                                    app.update_command_suggestions();
                                    
                                    // Trigger fetch based on resource
                                    match app.active_resource.as_str() {
                                        "keys" => { let _ = app.fetch_keys(None).await; }
                                        "clients" => { let _ = app.fetch_clients().await; }
                                        "info" => { let _ = app.fetch_info().await; }
                                        "slowlog" => { let _ = app.fetch_slowlog().await; }
                                        "config" => { let _ = app.fetch_configs().await; }
                                        "acl" => { let _ = app.fetch_acls().await; }
                                        "monitor" => { 
                                            // Start monitor task using raw TCP connection
                                            app.monitor_active = true;
                                            app.monitor_entries.clear();
                                            let config = app.connection_config.clone();
                                            let tx_clone = tx.clone();
                                            
                                            let task = tokio::spawn(async move {
                                                use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
                                                use tokio::net::TcpStream;
                                                
                                                let addr = format!("{}:{}", config.host, config.port);
                                                
                                                if let Ok(stream) = TcpStream::connect(&addr).await {
                                                    let (reader, mut writer) = stream.into_split();
                                                    let mut reader = BufReader::new(reader);
                                                    
                                                    // Send MONITOR command using Redis protocol
                                                    let monitor_cmd = "*1\r\n$7\r\nMONITOR\r\n";
                                                    if writer.write_all(monitor_cmd.as_bytes()).await.is_ok() {
                                                        // Read first response (+OK)
                                                        let mut response = String::new();
                                                        let _ = reader.read_line(&mut response).await;
                                                        
                                                        // Now read monitor stream
                                                        loop {
                                                            let mut line = String::new();
                                                            match reader.read_line(&mut line).await {
                                                                Ok(0) => break, // Connection closed
                                                                Ok(_) => {
                                                                    // Remove the leading '+' and trim
                                                                    let line = line.trim();
                                                                    if line.starts_with('+') {
                                                                        let line = &line[1..];
                                                                        if let Some(entry) = parse_monitor_output(line) {
                                                                            let _ = tx_clone.send(AppEvent::MonitorCommand(entry)).await;
                                                                        }
                                                                    }
                                                                }
                                                                Err(_) => break,
                                                            }
                                                        }
                                                    }
                                                }
                                            });
                                            
                                            app.monitor_task = Some(task);
                                        }
                                        "streams" => { let _ = app.fetch_streams().await; }
                                        "pubsub" => { let _ = app.fetch_pubsub_channels().await; }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Mode::ServerDialog => {
                        match key.code {
                            KeyCode::Esc => {
                                // If no servers exist, quit - can't escape without a server
                                if app.tredis_config.servers.is_empty() {
                                    app.should_quit = true;
                                } else {
                                    // Go back to normal mode if servers exist
                                    app.mode = Mode::Splash;
                                }
                            }
                            KeyCode::Tab | KeyCode::Down | KeyCode::Up => {
                                app.server_dialog_state.toggle_field();
                            }
                            KeyCode::Backspace => {
                                app.server_dialog_state.pop_char();
                            }
                            KeyCode::Char(c) => {
                                app.server_dialog_state.push_char(c);
                            }
                            KeyCode::Enter => {
                                if app.server_dialog_state.is_valid() {
                                    // Try to add server
                                    if let Err(e) = app.add_server_from_dialog() {
                                        app.server_dialog_state.set_error(format!("Error: {}", e));
                                    } else {
                                        // Parse URI and set connection config
                                        let uri = app.current_server.as_ref().map(|s| s.uri.clone());
                                        let server_name = app.current_server.as_ref().map(|s| s.name.clone());
                                        if let (Some(uri), Some(name)) = (uri, server_name) {
                                            if let Err(e) = app.set_connection_from_uri(&uri) {
                                                app.server_dialog_state.set_error(format!("Invalid URI: {}", e));
                                            } else {
                                                // Success! Switch to splash and connect
                                                app.mode = Mode::Splash;
                                                app.splash_state.set_message("Detecting server type...");
                                                
                                                // Spawn task to detect server info then connect
                                                let tx_clone = tx.clone();
                                                let uri_clone = uri.clone();
                                                let name_clone = name.clone();
                                                tokio::spawn(async move {
                                                    tokio::time::sleep(Duration::from_millis(300)).await;
                                                    
                                                    // Detect server info
                                                    let _ = tx_clone.send(AppEvent::Progress("Detecting server type...".to_string())).await;
                                                    if let Ok(info) = App::detect_server_info(&uri_clone).await {
                                                        let _ = tx_clone.send(AppEvent::ServerInfoDetected { 
                                                            server_name: name_clone, 
                                                            info 
                                                        }).await;
                                                    }
                                                    
                                                    let _ = tx_clone.send(AppEvent::Progress("Connecting to Redis...".to_string())).await;
                                                    let _ = tx_clone.send(AppEvent::Connect).await;
                                                });
                                            }
                                        }
                                    }
                                } else {
                                    app.server_dialog_state.set_error("Please fill in all fields".to_string());
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
        
        // Handle async events - process ALL pending events (non-blocking)
        loop {
            match rx.try_recv() {
                Ok(event) => match event {
                AppEvent::Progress(msg) => {
                     app.splash_state.set_message(&msg);
                     app.splash_state.complete_step();
                }
                AppEvent::Connect => {
                    log!(LogLevel::Info, "[EVENT-CONNECT] Connect event received");
                    log!(LogLevel::Info, "[EVENT-CONNECT] Connection config - Host: {}, Port: {}, DB: {}", 
                        app.connection_config.host, 
                        app.connection_config.port, 
                        app.connection_config.db);
                    
                    app.splash_state.set_message("Connected! Fetching keys...");
                    app.splash_state.complete_step();
                    
                    // Do the actual connection here in the main thread (tokio runtime)
                    log!(LogLevel::Info, "[EVENT-CONNECT] Calling app.connect()...");
                    if let Err(e) = app.connect().await {
                         log!(LogLevel::Error, "[EVENT-CONNECT] Connection error: {}", e);
                         app.splash_state.set_message(&format!("Connection failed: {}", e));
                         tokio::time::sleep(Duration::from_secs(2)).await;
                         // Go back to servers list instead of quitting
                         app.mode = Mode::Normal;
                         app.active_resource = "servers".to_string();
                         app.current_server = None;
                    } else {
                        log!(LogLevel::Info, "[EVENT-CONNECT] Connected successfully, fetching keys...");
                        // Fetch keys
                        if let Err(e) = app.fetch_keys(None).await {
                            log!(LogLevel::Error, "[EVENT-CONNECT] Error fetching keys: {}", e);
                            app.splash_state.set_message(&format!("Error fetching keys: {}", e));
                            tokio::time::sleep(Duration::from_secs(2)).await;
                            // Go back to servers list
                            app.mode = Mode::Normal;
                            app.active_resource = "servers".to_string();
                            app.current_server = None;
                        } else {
                             log!(LogLevel::Info, "[EVENT-CONNECT] Keys fetched, switching to Normal mode");
                             app.splash_state.complete_step();
                             tokio::time::sleep(Duration::from_millis(500)).await;
                             app.mode = Mode::Normal;
                        }
                    }
                }
                AppEvent::DetectServerInfo { uri, server_name } => {
                    // Detect server info in background
                    let tx_clone = tx.clone();
                    tokio::spawn(async move {
                        if let Ok(info) = App::detect_server_info(&uri).await {
                            let _ = tx_clone.send(AppEvent::ServerInfoDetected { server_name, info }).await;
                        }
                    });
                }
                AppEvent::ServerInfoDetected { server_name, info } => {
                    // Update server info in config
                    if let Err(e) = app.update_server_info(&server_name, info.clone()) {
                        log!(LogLevel::Error, "Failed to save server info: {}", e);
                    }
                    // Also update current_server if it matches
                    if let Some(ref mut current) = app.current_server {
                        if current.name == server_name {
                            current.info = Some(info);
                        }
                    }
                }
                AppEvent::MonitorCommand(entry) => {
                    if app.monitor_active {
                        // Prepend to beginning of list (newest first)
                        app.monitor_entries.insert(0, entry);
                        // Keep only last 1000 entries
                        if app.monitor_entries.len() > 1000 {
                            app.monitor_entries.pop();
                        }
                        // Only auto-scroll if user is at the top (viewing latest entries)
                        // If user scrolled down, don't interrupt them
                        if app.selected_monitor_index == 0 && app.monitor_scroll == 0 {
                            // User is at top, keep them there to see new entries
                            app.selected_monitor_index = 0;
                            app.monitor_scroll = 0;
                        } else {
                            // User scrolled down, increment their position to keep viewing same entries
                            app.selected_monitor_index += 1;
                        }
                    }
                }

                AppEvent::PubSubMessage(entry) => {
                    if app.pubsub_subscribe_mode && !app.pubsub_subscribe_channel.is_empty() {
                        // Prepend to beginning of list (newest first)
                        app.pubsub_messages.insert(0, entry);
                        // Keep only last 1000 entries
                        if app.pubsub_messages.len() > 1000 {
                            app.pubsub_messages.pop();
                        }
                    }
                }
                AppEvent::StreamMessage(entry) => {
                    log!(LogLevel::Info, "[HANDLER] ========================================");
                    log!(LogLevel::Info, "[HANDLER] StreamMessage received!");
                    log!(LogLevel::Info, "[HANDLER]   stream_active: {}", app.stream_active);
                    log!(LogLevel::Info, "[HANDLER]   Entry ID: {}", entry.id);
                    log!(LogLevel::Info, "[HANDLER]   Fields: {:?}", entry.fields);
                    if app.stream_active {
                        log!(LogLevel::Info, "[HANDLER] Adding message to stream_messages");
                        log!(LogLevel::Info, "[HANDLER]   Current count: {}", app.stream_messages.len());
                        // Prepend to beginning of list (newest first)
                        app.stream_messages.insert(0, entry);
                        log!(LogLevel::Info, "[HANDLER]   New count: {}", app.stream_messages.len());
                        log!(LogLevel::Info, "[HANDLER] Message successfully added!");
                        // Keep only last 1000 entries
                        if app.stream_messages.len() > 1000 {
                            app.stream_messages.pop();
                        }
                        log!(LogLevel::Info, "[HANDLER] ========================================");
                    } else {
                        log!(LogLevel::Warn, "[HANDLER] Message IGNORED - stream_active is FALSE!");
                        log!(LogLevel::Info, "[HANDLER] ========================================");
                    }
                }
                }
                Err(_) => break, // No more events
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Cleanup
    app.stop_monitor();
    app.stop_stream_consumer();
    if let Some(task) = app.pubsub_task.take() {
        task.abort();
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

enum AppEvent {
    Progress(String),
    Connect,
    DetectServerInfo { uri: String, server_name: String },
    ServerInfoDetected { server_name: String, info: ServerInfo },
    MonitorCommand(model::MonitorEntry),
    PubSubMessage(model::PubSubMessage),
    StreamMessage(model::StreamEntry),
}

fn parse_monitor_output(line: &str) -> Option<model::MonitorEntry> {
    use chrono::{DateTime, TimeZone, Utc};
    
    // MONITOR output format: 1234567890.123456 [0 127.0.0.1:12345] "command" "arg1" "arg2"
    if line.is_empty() {
        return None;
    }
    
    let parts: Vec<&str> = line.splitn(3, ' ').collect();
    if parts.len() < 3 {
        return None;
    }
    
    // Parse timestamp and convert to human-readable format with date
    let timestamp_raw = parts[0];
    let timestamp = if let Ok(ts_float) = timestamp_raw.parse::<f64>() {
        let secs = ts_float as i64;
        let nanos = ((ts_float - secs as f64) * 1_000_000_000.0) as u32;
        if let Some(dt) = Utc.timestamp_opt(secs, nanos).single() {
            // Convert to local time
            let local = dt.with_timezone(&chrono::Local);
            local.format("%Y-%m-%d %H:%M:%S").to_string()
        } else {
            timestamp_raw.to_string()
        }
    } else {
        timestamp_raw.to_string()
    };
    
    let client_db = parts[1].trim_matches(|c| c == '[' || c == ']');
    let command = parts[2].to_string();
    
    // Parse [db client:port]
    let client_parts: Vec<&str> = client_db.splitn(2, ' ').collect();
    let db = client_parts.get(0).unwrap_or(&"0").to_string();
    let client = client_parts.get(1).unwrap_or(&"unknown").to_string();
    
    Some(model::MonitorEntry {
        timestamp,
        db,
        client,
        command,
    })
}

fn parse_uri_details(uri: &str) -> String {
    let uri = uri.trim();
    let rest = uri.strip_prefix("redis://").unwrap_or(uri);
    
    let mut host = "localhost".to_string();
    let mut port = "6379".to_string();
    let mut db = "0".to_string();
    let mut user = "None".to_string();
    let mut password = "None".to_string();
    
    // Check for auth (user:password@)
    let (auth_part, host_part) = if let Some(at_pos) = rest.rfind('@') {
        let (auth, h) = rest.split_at(at_pos);
        (Some(auth), &h[1..])
    } else {
        (None, rest)
    };
    
    // Parse auth if present
    if let Some(auth) = auth_part {
        if let Some(colon_pos) = auth.find(':') {
            let (u, p) = auth.split_at(colon_pos);
            user = u.to_string();
            password = "*".repeat(p.len() - 1); // Hide password
        } else {
            password = "*".repeat(auth.len());
        }
    }
    
    // Parse host:port/db
    let (host_port, db_str) = if let Some(slash_pos) = host_part.find('/') {
        let (hp, d) = host_part.split_at(slash_pos);
        (hp, &d[1..])
    } else {
        (host_part, "0")
    };
    db = db_str.to_string();
    
    // Parse host and port
    if let Some(colon_pos) = host_port.rfind(':') {
        let (h, p) = host_port.split_at(colon_pos);
        host = h.to_string();
        port = p[1..].to_string();
    } else {
        host = host_port.to_string();
    }
    
    format!(
        "  Host:     {}\n  Port:     {}\n  Database: {}\n  User:     {}\n  Password: {}",
        host, port, db, user, password
    )
}

fn format_server_details(server: &model::ServerConfig) -> String {
    // Parse URI for details
    let uri = server.uri.trim();
    let rest = uri.strip_prefix("rediss://").or_else(|| uri.strip_prefix("redis://")).unwrap_or(uri);
    let is_tls = uri.starts_with("rediss://");
    
    let (auth_part, host_part) = if let Some(at_pos) = rest.rfind('@') {
        let (auth, h) = rest.split_at(at_pos);
        (Some(auth), &h[1..])
    } else {
        (None, rest)
    };
    
    let mut host = "localhost".to_string();
    let mut port = "6379".to_string();
    let mut db = "0".to_string();
    let has_auth = auth_part.is_some();
    
    let (host_port, db_str) = if let Some(slash_pos) = host_part.find('/') {
        let (hp, d) = host_part.split_at(slash_pos);
        (hp, &d[1..])
    } else {
        (host_part, "0")
    };
    db = db_str.to_string();
    
    if let Some(colon_pos) = host_port.rfind(':') {
        let (h, p) = host_port.split_at(colon_pos);
        host = h.to_string();
        port = p[1..].to_string();
    } else {
        host = host_port.to_string();
    }
    
    // Build JSON object
    let mut json_obj = serde_json::Map::new();
    
    json_obj.insert("name".to_string(), serde_json::Value::String(server.name.clone()));
    
    // Server info
    if let Some(ref info) = server.info {
        let mut server_info = serde_json::Map::new();
        server_info.insert("type".to_string(), serde_json::Value::String(info.server_type.as_str().to_string()));
        server_info.insert("version".to_string(), serde_json::Value::String(info.redis_version.clone()));
        server_info.insert("role".to_string(), serde_json::Value::String(info.role.clone()));
        if !info.os.is_empty() {
            server_info.insert("os".to_string(), serde_json::Value::String(info.os.clone()));
        }
        if let Some(cluster_size) = info.cluster_size {
            server_info.insert("cluster_size".to_string(), serde_json::Value::Number(serde_json::Number::from(cluster_size)));
        }
        json_obj.insert("server_info".to_string(), serde_json::Value::Object(server_info));
    } else {
        json_obj.insert("server_info".to_string(), serde_json::Value::Null);
    }
    
    // Connection details
    let mut connection = serde_json::Map::new();
    connection.insert("uri".to_string(), serde_json::Value::String(mask_uri(&server.uri)));
    connection.insert("host".to_string(), serde_json::Value::String(host));
    connection.insert("port".to_string(), serde_json::Value::String(port));
    connection.insert("database".to_string(), serde_json::Value::String(db));
    connection.insert("tls".to_string(), serde_json::Value::Bool(is_tls));
    connection.insert("auth".to_string(), serde_json::Value::Bool(has_auth));
    json_obj.insert("connection".to_string(), serde_json::Value::Object(connection));
    
    serde_json::to_string_pretty(&json_obj).unwrap_or_else(|_| "{}".to_string())
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
