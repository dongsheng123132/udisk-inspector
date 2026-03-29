use serde::Serialize;

#[derive(Clone, Copy)]
pub enum OutputMode {
    Human,
    Json,
}

#[derive(Serialize)]
struct JsonEnvelope<T: Serialize> {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

pub fn print_success<T: Serialize>(mode: OutputMode, data: &T) {
    match mode {
        OutputMode::Human => {}
        OutputMode::Json => {
            let envelope = JsonEnvelope {
                success: true,
                data: Some(data),
                error: None,
            };
            println!("{}", serde_json::to_string(&envelope).unwrap());
        }
    }
}

pub fn print_error(mode: OutputMode, msg: &str) {
    match mode {
        OutputMode::Human => {
            eprintln!("Error: {}", msg);
        }
        OutputMode::Json => {
            let envelope: JsonEnvelope<()> = JsonEnvelope {
                success: false,
                data: None,
                error: Some(msg.to_string()),
            };
            println!("{}", serde_json::to_string(&envelope).unwrap());
        }
    }
}
