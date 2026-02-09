use std::path::Path;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum TranscriptionEvent {
    Stdout(String),
    Stderr(String),
    Completed(String),
    Error(String),
}

pub async fn run_transcription(
    app: AppHandle,
    audio_path: &Path,
    model_path: &Path,
    output_format: &str,
    language: Option<&str>,
) -> Result<mpsc::Receiver<TranscriptionEvent>, String> {
    let (tx, rx) = mpsc::channel(100);

    let mut args = vec![
        "-m".to_string(),
        model_path.to_string_lossy().to_string(),
        "-f".to_string(),
        audio_path.to_string_lossy().to_string(),
        "-o".to_string(),
        output_format.to_string(),
    ];

    if let Some(lang) = language {
        if lang != "auto" {
            args.push("-l".to_string());
            args.push(lang.to_string());
        }
    }

    let shell = app.shell();
    let command = shell
        .sidecar("binaries/whisper-cli")
        .map_err(|e| format!("Failed to create sidecar command: {}", e))?
        .args(&args);

    let (mut rx_cmd, _child) = command
        .spawn()
        .map_err(|e| format!("Failed to spawn whisper-cli: {}", e))?;

    let tx_clone = tx.clone();
    tokio::spawn(async move {
        let mut full_output = String::new();

        while let Some(event) = rx_cmd.recv().await {
            match event {
                CommandEvent::Stdout(line) => {
                    let line_str = String::from_utf8_lossy(&line).to_string();
                    full_output.push_str(&line_str);
                    full_output.push('\n');
                    let _ = tx_clone.send(TranscriptionEvent::Stdout(line_str)).await;
                }
                CommandEvent::Stderr(line) => {
                    let line_str = String::from_utf8_lossy(&line).to_string();
                    let _ = tx_clone.send(TranscriptionEvent::Stderr(line_str)).await;
                }
                CommandEvent::Terminated(payload) => {
                    if payload.code == Some(0) {
                        let _ = tx_clone
                            .send(TranscriptionEvent::Completed(full_output.clone()))
                            .await;
                    } else {
                        let _ = tx_clone
                            .send(TranscriptionEvent::Error(format!(
                                "Process exited with code: {:?}",
                                payload.code
                            )))
                            .await;
                    }
                    break;
                }
                _ => {}
            }
        }
    });

    Ok(rx)
}
