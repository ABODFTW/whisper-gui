use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub display_name: String,
    pub size_mb: u64,
    pub description: String,
    pub url: String,
}

pub fn get_available_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            name: "tiny".to_string(),
            display_name: "Tiny".to_string(),
            size_mb: 75,
            description: "Fastest, lowest accuracy".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin".to_string(),
        },
        ModelInfo {
            name: "base".to_string(),
            display_name: "Base".to_string(),
            size_mb: 148,
            description: "Fast, good for simple audio".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin".to_string(),
        },
        ModelInfo {
            name: "small".to_string(),
            display_name: "Small".to_string(),
            size_mb: 488,
            description: "Balanced speed and accuracy".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin".to_string(),
        },
        ModelInfo {
            name: "medium".to_string(),
            display_name: "Medium".to_string(),
            size_mb: 1500,
            description: "High accuracy, slower".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin".to_string(),
        },
        ModelInfo {
            name: "large-v3".to_string(),
            display_name: "Large v3".to_string(),
            size_mb: 3000,
            description: "Best accuracy, slowest".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin".to_string(),
        },
        ModelInfo {
            name: "large-v3-turbo".to_string(),
            display_name: "Large v3 Turbo".to_string(),
            size_mb: 1600,
            description: "Fast and accurate".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin".to_string(),
        },
    ]
}

pub fn get_models_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.whisper-gui.app")
        .join("models")
}

pub fn get_model_path(model_name: &str) -> PathBuf {
    get_models_dir().join(format!("ggml-{}.bin", model_name))
}

pub async fn is_model_downloaded(model_name: &str) -> bool {
    let path = get_model_path(model_name);
    path.exists()
}

pub async fn download_model<F>(
    model_name: &str,
    progress_callback: F,
) -> Result<PathBuf, String>
where
    F: Fn(u64, u64) + Send + 'static,
{
    let models = get_available_models();
    let model = models
        .iter()
        .find(|m| m.name == model_name)
        .ok_or_else(|| format!("Model '{}' not found", model_name))?;

    let models_dir = get_models_dir();
    fs::create_dir_all(&models_dir)
        .await
        .map_err(|e| format!("Failed to create models directory: {}", e))?;

    let model_path = get_model_path(model_name);
    let temp_path = model_path.with_extension("bin.tmp");

    let client = Client::new();
    let response = client
        .get(&model.url)
        .send()
        .await
        .map_err(|e| format!("Failed to start download: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let mut file = fs::File::create(&temp_path)
        .await
        .map_err(|e| format!("Failed to create file: {}", e))?;

    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Error downloading: {}", e))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("Error writing file: {}", e))?;

        downloaded += chunk.len() as u64;
        progress_callback(downloaded, total_size);
    }

    file.flush()
        .await
        .map_err(|e| format!("Error flushing file: {}", e))?;

    fs::rename(&temp_path, &model_path)
        .await
        .map_err(|e| format!("Error finalizing download: {}", e))?;

    Ok(model_path)
}
