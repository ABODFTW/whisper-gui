use crate::downloader::get_model_path;
use crate::whisper::{run_transcription, TranscriptionEvent};
use serde::Serialize;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize)]
pub struct TranscriptionOutput {
    pub line: String,
    pub is_error: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TranscriptionComplete {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn transcribe_audio(
    app: AppHandle,
    audio_path: String,
    model_name: String,
    output_format: String,
    language: Option<String>,
) -> Result<(), String> {
    let audio_path = PathBuf::from(&audio_path);
    if !audio_path.exists() {
        return Err(format!("Audio file not found: {}", audio_path.display()));
    }

    let model_path = get_model_path(&model_name);
    if !model_path.exists() {
        return Err(format!("Model '{}' not downloaded", model_name));
    }

    let mut rx = run_transcription(
        app.clone(),
        &audio_path,
        &model_path,
        &output_format,
        language.as_deref(),
    )
    .await?;

    let app_clone = app.clone();
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                TranscriptionEvent::Stdout(line) => {
                    let _ = app_clone.emit(
                        "transcription-output",
                        TranscriptionOutput {
                            line,
                            is_error: false,
                        },
                    );
                }
                TranscriptionEvent::Stderr(line) => {
                    let _ = app_clone.emit(
                        "transcription-output",
                        TranscriptionOutput {
                            line,
                            is_error: true,
                        },
                    );
                }
                TranscriptionEvent::Completed(output) => {
                    let _ = app_clone.emit(
                        "transcription-complete",
                        TranscriptionComplete {
                            success: true,
                            output,
                            error: None,
                        },
                    );
                }
                TranscriptionEvent::Error(err) => {
                    let _ = app_clone.emit(
                        "transcription-complete",
                        TranscriptionComplete {
                            success: false,
                            output: String::new(),
                            error: Some(err),
                        },
                    );
                }
            }
        }
    });

    Ok(())
}
