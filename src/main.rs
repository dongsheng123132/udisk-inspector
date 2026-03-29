use std::sync::atomic::Ordering;
use udisk_inspector_lib::cli;

#[tokio::main]
async fn main() {
    env_logger::init();

    ctrlc::set_handler(move || {
        eprintln!("\nStopping test...");
        udisk_inspector_lib::STOP_FLAG.store(true, Ordering::Relaxed);
    })
    .ok();

    if let Err(e) = cli::run().await {
        let json_mode = std::env::args().any(|a| a == "--json");
        if json_mode {
            let envelope = serde_json::json!({
                "success": false,
                "error": e.to_string()
            });
            println!("{}", envelope);
        } else {
            eprintln!("Error: {}", e);
        }
        std::process::exit(1);
    }
}
