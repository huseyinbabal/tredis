pub mod acls_table;
pub mod channels_table;
pub mod clients_table;
pub mod configs_table;
pub mod describe;
pub mod dialog;
pub mod header;
pub mod info_view;
pub mod keys_table;
pub mod monitor_table;
pub mod pubsub_table;
pub mod resources;
pub mod server_dialog;
pub mod servers_table;
pub mod slowlog_table;
pub mod splash;
pub mod streams_table;

use crate::app::{App, Mode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    if app.mode == Mode::Splash {
        splash::render(f, &app.splash_state);
        return;
    }

    // Server dialog is shown as a full-screen overlay when no servers exist
    if app.mode == Mode::ServerDialog {
        server_dialog::render(f, &app.server_dialog_state);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Header
            Constraint::Min(1),    // Main content
            Constraint::Length(1), // Footer
        ])
        .split(f.area());

    header::render(f, app, chunks[0]);

    match app.mode {
        Mode::Describe => {
            describe::render(f, app, chunks[1]);
        }
        _ => match app.active_resource.as_str() {
            "servers" => servers_table::render(f, app, chunks[1]),
            "clients" => clients_table::render(f, app, chunks[1]),
            "info" => info_view::render(f, app, chunks[1]),
            "slowlog" => slowlog_table::render(f, app, chunks[1]),
            "config" => configs_table::render(f, app, chunks[1]),
            "acl" => acls_table::render(f, app, chunks[1]),
            "monitor" => monitor_table::render(f, app, chunks[1]),
            "streams" => streams_table::render(f, app, chunks[1]),
            "channels" => channels_table::render(f, app, chunks[1]),
            "pubsub" => pubsub_table::render(f, app, chunks[1]),
            _ => keys_table::render(f, app, chunks[1]),
        },
    }

    // Render overlays
    if app.mode == Mode::Confirm {
        dialog::render(f, app);
    }

    if app.mode == Mode::Resources {
        resources::render(f, app);
    }
}
