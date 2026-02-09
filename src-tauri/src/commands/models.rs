use crate::downloader::{
    download_model, get_available_models, get_model_path, is_model_downloaded, ModelInfo,
};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStatus {
    pub info: ModelInfo,
    pub downloaded: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DownloadProgress {
    pub model_name: String,
    pub downloaded: u64,
    pub total: u64,
    pub percent: f64,
}

#[tauri::command]
pub async fn list_models() -> Result<Vec<ModelStatus>, String> {
    let models = get_available_models();
    let mut result = Vec::new();

    for model in models {
        let downloaded = is_model_downloaded(&model.name).await;
        result.push(ModelStatus {
            info: model,
            downloaded,
        });
    }

    Ok(result)
}

#[tauri::command]
pub async fn download_model_command(
    app: AppHandle,
    model_name: String,
) -> Result<String, String> {
    let downloaded = Arc::new(AtomicU64::new(0));
    let total = Arc::new(AtomicU64::new(0));
    let model_name_clone = model_name.clone();
    let app_clone = app.clone();

    let downloaded_clone = downloaded.clone();
    let total_clone = total.clone();

    let progress_callback = move |dl: u64, tot: u64| {
        downloaded_clone.store(dl, Ordering::Relaxed);
        total_clone.store(tot, Ordering::Relaxed);

        let percent = if tot > 0 {
            (dl as f64 / tot as f64) * 100.0
        } else {
            0.0
        };

        let _ = app_clone.emit(
            "download-progress",
            DownloadProgress {
                model_name: model_name_clone.clone(),
                downloaded: dl,
                total: tot,
                percent,
            },
        );
    };

    let path = download_model(&model_name, progress_callback).await?;

    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn get_model_path_command(model_name: String) -> Result<String, String> {
    let path = get_model_path(&model_name);
    if path.exists() {
        Ok(path.to_string_lossy().to_string())
    } else {
        Err(format!("Model '{}' not downloaded", model_name))
    }
}

#[tauri::command]
pub async fn delete_model(model_name: String) -> Result<(), String> {
    let path = get_model_path(&model_name);
    if path.exists() {
        tokio::fs::remove_file(&path)
            .await
            .map_err(|e| format!("Failed to delete model: {}", e))?;
    }
    Ok(())
}
