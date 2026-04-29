use anyhow::Result;
use async_trait::async_trait;
use std::io::Write;
use std::process::Command;

use super::whisper_compat::WhisperCompatProvider;
use super::{SttConfig, SttProvider, TranscriptEvent};

/// Max audio buffer: ~6 MB PCM ≈ 3 min at 16kHz 16-bit mono.
/// Local whisper is slower than cloud, so we keep it short.
const MAX_AUDIO_BYTES: usize = 6 * 1024 * 1024;

/// Local whisper.cpp provider. Calls the whisper-cli binary bundled with the app.
/// Uses Metal GPU on Apple Silicon, falls back to CPU on other platforms.
/// Supports LoRA adapters for dialect fine-tuning.
pub struct LocalWhisperProvider {
    cli_path: String,
    model_path: String,
    lora_path: Option<String>,
    stt_config: Option<SttConfig>,
    audio_buffer: Vec<u8>,
}

impl LocalWhisperProvider {
    pub fn new(cli_path: String, model_path: String, lora_path: Option<String>) -> Self {
        Self {
            cli_path,
            model_path,
            lora_path,
            stt_config: None,
            audio_buffer: Vec::new(),
        }
    }
}

#[async_trait]
impl SttProvider for LocalWhisperProvider {
    async fn connect(&mut self, config: &SttConfig) -> Result<()> {
        self.stt_config = Some(config.clone());
        self.audio_buffer.clear();
        tracing::info!(
            "LocalWhisper model: {}, cli: {}",
            self.model_path,
            self.cli_path
        );
        Ok(())
    }

    async fn send_audio(&mut self, chunk: &[u8]) -> Result<()> {
        if self.audio_buffer.len() + chunk.len() > MAX_AUDIO_BYTES {
            anyhow::bail!("LocalWhisper: audio exceeds maximum length (~3 min)");
        }
        self.audio_buffer.extend_from_slice(chunk);
        Ok(())
    }

    async fn recv_transcript(&mut self) -> Result<Option<TranscriptEvent>> {
        // File-based transcription — partial results not supported in v1.
        // Streaming partials will be added in a future iteration.
        Ok(None)
    }

    async fn disconnect(&mut self) -> Result<Option<String>> {
        let config = match &self.stt_config {
            Some(c) => c.clone(),
            None => return Ok(None),
        };

        if self.audio_buffer.is_empty() {
            tracing::info!("LocalWhisper: no audio buffered, skipping");
            return Ok(None);
        }

        let audio_len_secs = self.audio_buffer.len() as f64 / (config.sample_rate as f64 * 2.0);
        tracing::info!(
            "LocalWhisper: transcribing {:.1}s of audio",
            audio_len_secs
        );

        let wav_data = WhisperCompatProvider::build_wav(&self.audio_buffer, config.sample_rate);
        self.audio_buffer.clear();

        let mut temp_file = tempfile::NamedTempFile::with_suffix(".wav")?;
        temp_file.write_all(&wav_data)?;
        let temp_path = temp_file.into_temp_path();

        let lang = match &config.language {
            Some(l) if l == "multi" => "auto",
            Some(l) => l.as_str(),
            None => "auto",
        };

        let mut cmd = Command::new(&self.cli_path);
        cmd.arg("-m")
            .arg(&self.model_path)
            .arg("-f")
            .arg(&temp_path)
            .arg("-l")
            .arg(lang)
            .arg("--no-timestamps")
            .arg("-nt")
            .arg("--no-fallback");

        if let Some(ref lora) = self.lora_path {
            if !lora.is_empty() && std::path::Path::new(lora).exists() {
                cmd.arg("--lora").arg(lora);
                tracing::info!("LocalWhisper: using LoRA adapter: {}", lora);
            }
        }

        let output = cmd.output()
            .map_err(|e| {
                // Temp file is cleaned up automatically when temp_path is dropped
                anyhow::anyhow!("whisper-cli not found or failed to start: {}", e)
            })?;

        // Explicitly keep the temp file alive until after the subprocess finishes,
        // then let it be deleted automatically on drop.
        drop(temp_path);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!("whisper-cli failed: {}", stderr);
            anyhow::bail!("whisper-cli error: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let text: String = stdout
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string();

        tracing::info!("LocalWhisper transcription: {} chars", text.len());

        if text.is_empty() {
            Ok(None)
        } else {
            Ok(Some(text))
        }
    }

    fn name(&self) -> &str {
        "LocalWhisper"
    }
}
