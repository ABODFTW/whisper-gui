mod commands;
mod downloader;
mod whisper;

use commands::{
    delete_model, download_model_command, get_model_path_command, list_models, transcribe_audio,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            list_models,
            download_model_command,
            get_model_path_command,
            delete_model,
            transcribe_audio,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
