pub mod commands;
pub mod db;
pub mod disk;
pub mod report;
pub mod test;

use db::Database;
use std::sync::Mutex;

pub struct AppState {
    pub db: Mutex<Database>,
}

pub fn run() {
    env_logger::init();

    let db = Database::new().expect("Failed to initialize database");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            db: Mutex::new(db),
        })
        .invoke_handler(tauri::generate_handler![
            commands::drive::list_drives,
            commands::drive::get_drive_info,
            commands::test::start_test,
            commands::test::stop_test,
            commands::report::get_report,
            commands::report::list_reports,
            commands::report::export_html,
            commands::report::delete_report,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
