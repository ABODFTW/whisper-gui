import { useEffect, useState, useCallback } from "react";
import {
  listModels,
  downloadModel,
  deleteModel,
  transcribeAudio,
  selectAudioFile,
  onDownloadProgress,
  onTranscriptionOutput,
  onTranscriptionComplete,
  ModelStatus,
  DownloadProgress,
} from "./lib/tauri";

const OUTPUT_FORMATS = [
  { value: "txt", label: "Text (.txt)" },
  { value: "srt", label: "Subtitles (.srt)" },
  { value: "vtt", label: "WebVTT (.vtt)" },
  { value: "json", label: "JSON (.json)" },
];

const LANGUAGES = [
  { value: "auto", label: "Auto-detect" },
  { value: "en", label: "English" },
  { value: "es", label: "Spanish" },
  { value: "fr", label: "French" },
  { value: "de", label: "German" },
  { value: "it", label: "Italian" },
  { value: "pt", label: "Portuguese" },
  { value: "ru", label: "Russian" },
  { value: "ja", label: "Japanese" },
  { value: "ko", label: "Korean" },
  { value: "zh", label: "Chinese" },
  { value: "ar", label: "Arabic" },
];

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

export default function App() {
  const [models, setModels] = useState<ModelStatus[]>([]);
  const [selectedModel, setSelectedModel] = useState<string>("");
  const [audioPath, setAudioPath] = useState<string>("");
  const [outputFormat, setOutputFormat] = useState<string>("txt");
  const [language, setLanguage] = useState<string>("auto");
  const [output, setOutput] = useState<string>("");
  const [isTranscribing, setIsTranscribing] = useState(false);
  const [downloadingModel, setDownloadingModel] = useState<string | null>(null);
  const [downloadProgress, setDownloadProgress] = useState<DownloadProgress | null>(null);
  const [error, setError] = useState<string | null>(null);

  const loadModels = useCallback(async () => {
    try {
      const modelList = await listModels();
      setModels(modelList);

      const downloaded = modelList.find((m) => m.downloaded);
      if (downloaded && !selectedModel) {
        setSelectedModel(downloaded.info.name);
      }
    } catch (err) {
      setError(`Failed to load models: ${err}`);
    }
  }, [selectedModel]);

  useEffect(() => {
    loadModels();
  }, [loadModels]);

  useEffect(() => {
    const unlistenProgress = onDownloadProgress((progress) => {
      setDownloadProgress(progress);
    });

    const unlistenOutput = onTranscriptionOutput((out) => {
      setOutput((prev) => prev + out.line + "\n");
    });

    const unlistenComplete = onTranscriptionComplete((result) => {
      setIsTranscribing(false);
      if (!result.success && result.error) {
        setError(result.error);
      }
    });

    return () => {
      unlistenProgress.then((fn) => fn());
      unlistenOutput.then((fn) => fn());
      unlistenComplete.then((fn) => fn());
    };
  }, []);

  const handleSelectFile = async () => {
    const path = await selectAudioFile();
    if (path) {
      setAudioPath(path);
      setError(null);
    }
  };

  const handleDownloadModel = async (modelName: string) => {
    setDownloadingModel(modelName);
    setDownloadProgress(null);
    setError(null);

    try {
      await downloadModel(modelName);
      await loadModels();
      setSelectedModel(modelName);
    } catch (err) {
      setError(`Download failed: ${err}`);
    } finally {
      setDownloadingModel(null);
      setDownloadProgress(null);
    }
  };

  const handleDeleteModel = async (modelName: string) => {
    try {
      await deleteModel(modelName);
      await loadModels();
      if (selectedModel === modelName) {
        const remaining = models.find(
          (m) => m.downloaded && m.info.name !== modelName
        );
        setSelectedModel(remaining?.info.name || "");
      }
    } catch (err) {
      setError(`Delete failed: ${err}`);
    }
  };

  const handleTranscribe = async () => {
    if (!audioPath || !selectedModel) {
      setError("Please select an audio file and model");
      return;
    }

    setOutput("");
    setError(null);
    setIsTranscribing(true);

    try {
      await transcribeAudio(
        audioPath,
        selectedModel,
        outputFormat,
        language === "auto" ? null : language
      );
    } catch (err) {
      setError(`Transcription failed: ${err}`);
      setIsTranscribing(false);
    }
  };

  return (
    <div className="min-h-screen bg-gray-900 text-gray-100 p-6">
      <div className="max-w-4xl mx-auto space-y-6">
        <h1 className="text-3xl font-bold text-center mb-8">Whisper GUI</h1>

        {error && (
          <div className="bg-red-900/50 border border-red-500 text-red-200 px-4 py-3 rounded">
            {error}
            <button
              onClick={() => setError(null)}
              className="float-right text-red-400 hover:text-red-200"
            >
              x
            </button>
          </div>
        )}

        {/* Model Selection */}
        <section className="bg-gray-800 rounded-lg p-4">
          <h2 className="text-xl font-semibold mb-4">Select Model</h2>
          <div className="grid gap-3">
            {models.map((model) => (
              <div
                key={model.info.name}
                className={`flex items-center justify-between p-3 rounded border ${
                  selectedModel === model.info.name
                    ? "border-blue-500 bg-blue-900/30"
                    : "border-gray-700 bg-gray-700/30"
                }`}
              >
                <label className="flex items-center gap-3 cursor-pointer flex-1">
                  <input
                    type="radio"
                    name="model"
                    value={model.info.name}
                    checked={selectedModel === model.info.name}
                    onChange={(e) => setSelectedModel(e.target.value)}
                    disabled={!model.downloaded || isTranscribing}
                    className="w-4 h-4"
                  />
                  <div>
                    <div className="font-medium">
                      {model.info.display_name}
                      <span className="text-gray-400 text-sm ml-2">
                        ({model.info.size_mb} MB)
                      </span>
                    </div>
                    <div className="text-gray-400 text-sm">
                      {model.info.description}
                    </div>
                  </div>
                </label>

                <div className="flex gap-2">
                  {model.downloaded ? (
                    <button
                      onClick={() => handleDeleteModel(model.info.name)}
                      disabled={isTranscribing}
                      className="px-3 py-1 text-sm bg-red-600 hover:bg-red-700 disabled:opacity-50 rounded"
                    >
                      Delete
                    </button>
                  ) : downloadingModel === model.info.name ? (
                    <div className="text-sm text-blue-400">
                      {downloadProgress
                        ? `${downloadProgress.percent.toFixed(1)}% (${formatBytes(
                            downloadProgress.downloaded
                          )})`
                        : "Starting..."}
                    </div>
                  ) : (
                    <button
                      onClick={() => handleDownloadModel(model.info.name)}
                      disabled={downloadingModel !== null || isTranscribing}
                      className="px-3 py-1 text-sm bg-blue-600 hover:bg-blue-700 disabled:opacity-50 rounded"
                    >
                      Download
                    </button>
                  )}
                </div>
              </div>
            ))}
          </div>
        </section>

        {/* Audio File Selection */}
        <section className="bg-gray-800 rounded-lg p-4">
          <h2 className="text-xl font-semibold mb-4">Audio File</h2>
          <div className="flex gap-3">
            <input
              type="text"
              value={audioPath}
              readOnly
              placeholder="Select an audio file..."
              className="flex-1 bg-gray-700 border border-gray-600 rounded px-3 py-2 text-gray-200"
            />
            <button
              onClick={handleSelectFile}
              disabled={isTranscribing}
              className="px-4 py-2 bg-gray-600 hover:bg-gray-500 disabled:opacity-50 rounded"
            >
              Browse
            </button>
          </div>
        </section>

        {/* Options */}
        <section className="bg-gray-800 rounded-lg p-4">
          <h2 className="text-xl font-semibold mb-4">Options</h2>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm text-gray-400 mb-1">
                Output Format
              </label>
              <select
                value={outputFormat}
                onChange={(e) => setOutputFormat(e.target.value)}
                disabled={isTranscribing}
                className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2"
              >
                {OUTPUT_FORMATS.map((format) => (
                  <option key={format.value} value={format.value}>
                    {format.label}
                  </option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-1">
                Language
              </label>
              <select
                value={language}
                onChange={(e) => setLanguage(e.target.value)}
                disabled={isTranscribing}
                className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2"
              >
                {LANGUAGES.map((lang) => (
                  <option key={lang.value} value={lang.value}>
                    {lang.label}
                  </option>
                ))}
              </select>
            </div>
          </div>
        </section>

        {/* Transcribe Button */}
        <button
          onClick={handleTranscribe}
          disabled={!audioPath || !selectedModel || isTranscribing}
          className="w-full py-3 bg-green-600 hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg text-lg font-semibold"
        >
          {isTranscribing ? "Transcribing..." : "Transcribe"}
        </button>

        {/* Output */}
        <section className="bg-gray-800 rounded-lg p-4">
          <h2 className="text-xl font-semibold mb-4">Output</h2>
          <textarea
            value={output}
            readOnly
            placeholder="Transcription output will appear here..."
            className="w-full h-64 bg-gray-700 border border-gray-600 rounded px-3 py-2 font-mono text-sm resize-none"
          />
        </section>
      </div>
    </div>
  );
}
