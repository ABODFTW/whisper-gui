import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";

export interface ModelInfo {
  name: string;
  display_name: string;
  size_mb: number;
  description: string;
  url: string;
}

export interface ModelStatus {
  info: ModelInfo;
  downloaded: boolean;
}

export interface DownloadProgress {
  model_name: string;
  downloaded: number;
  total: number;
  percent: number;
}

export interface TranscriptionOutput {
  line: string;
  is_error: boolean;
}

export interface TranscriptionComplete {
  success: boolean;
  output: string;
  error: string | null;
}

export async function listModels(): Promise<ModelStatus[]> {
  return invoke<ModelStatus[]>("list_models");
}

export async function downloadModel(modelName: string): Promise<string> {
  return invoke<string>("download_model_command", { modelName });
}

export async function getModelPath(modelName: string): Promise<string> {
  return invoke<string>("get_model_path_command", { modelName });
}

export async function deleteModel(modelName: string): Promise<void> {
  return invoke<void>("delete_model", { modelName });
}

export async function transcribeAudio(
  audioPath: string,
  modelName: string,
  outputFormat: string,
  language: string | null
): Promise<void> {
  return invoke<void>("transcribe_audio", {
    audioPath,
    modelName,
    outputFormat,
    language,
  });
}

export async function selectAudioFile(): Promise<string | null> {
  const result = await open({
    multiple: false,
    filters: [
      {
        name: "Audio Files",
        extensions: ["wav", "mp3", "m4a", "flac", "ogg", "wma", "aac"],
      },
      {
        name: "All Files",
        extensions: ["*"],
      },
    ],
  });

  if (result && typeof result === "string") {
    return result;
  }
  return null;
}

export function onDownloadProgress(
  callback: (progress: DownloadProgress) => void
): Promise<UnlistenFn> {
  return listen<DownloadProgress>("download-progress", (event) => {
    callback(event.payload);
  });
}

export function onTranscriptionOutput(
  callback: (output: TranscriptionOutput) => void
): Promise<UnlistenFn> {
  return listen<TranscriptionOutput>("transcription-output", (event) => {
    callback(event.payload);
  });
}

export function onTranscriptionComplete(
  callback: (result: TranscriptionComplete) => void
): Promise<UnlistenFn> {
  return listen<TranscriptionComplete>("transcription-complete", (event) => {
    callback(event.payload);
  });
}
