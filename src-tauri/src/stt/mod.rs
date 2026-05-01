pub mod assemblyai;
pub mod cloud;
pub mod deepgram;
pub mod local_whisper;
pub mod whisper_compat;
pub mod xiaomi_mimo;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use whisper_compat::{WhisperCompatConfig, WhisperCompatProvider};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttConfig {
    pub api_key: String,
    pub language: Option<String>,
    pub smart_format: bool,
    pub sample_rate: u32,
}

impl Default for SttConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            language: None,
            smart_format: true,
            sample_rate: 16000,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TranscriptEvent {
    Partial { text: String },
    Final { text: String, confidence: f32 },
    SpeechStarted,
    SpeechEnded,
    Error { message: String },
}

#[async_trait]
pub trait SttProvider: Send + Sync {
    async fn connect(&mut self, config: &SttConfig) -> Result<()>;
    async fn send_audio(&mut self, chunk: &[u8]) -> Result<()>;
    async fn recv_transcript(&mut self) -> Result<Option<TranscriptEvent>>;
    /// Disconnect and optionally return a final transcript (for file-based providers).
    async fn disconnect(&mut self) -> Result<Option<String>>;
    fn name(&self) -> &str;
}

pub fn create_provider(
    provider_name: &str,
    client: Option<reqwest::Client>,
    custom_stt_base_url: Option<&str>,
    custom_stt_model: Option<&str>,
    whisper_cli_path: Option<&str>,
    whisper_model_path: Option<&str>,
    whisper_lora_path: Option<&str>,
) -> Box<dyn SttProvider> {
    let make = |cfg: WhisperCompatConfig| -> Box<dyn SttProvider> {
        match client {
            Some(ref c) => Box::new(WhisperCompatProvider::with_client(cfg, c.clone())),
            None => Box::new(WhisperCompatProvider::new(cfg)),
        }
    };
    match provider_name {
        "cloud" => {
            let api_base_url = crate::api_base_url();
            match client {
                Some(ref c) => Box::new(cloud::CloudSttProvider::with_client(
                    api_base_url,
                    c.clone(),
                )),
                None => Box::new(cloud::CloudSttProvider::new(api_base_url)),
            }
        }
        "assemblyai" => Box::new(assemblyai::AssemblyAiProvider::new()),
        "glm-asr" => make(WhisperCompatConfig {
            provider_name: "GLM-ASR".to_string(),
            endpoint: "https://open.bigmodel.cn/api/paas/v4/audio/transcriptions".to_string(),
            model: "glm-asr-2512".to_string(),
            extra_fields: vec![("stream".to_string(), "false".to_string())],
        }),
        "openai-whisper" => make(WhisperCompatConfig {
            provider_name: "OpenAI Whisper".to_string(),
            endpoint: "https://api.openai.com/v1/audio/transcriptions".to_string(),
            model: "whisper-1".to_string(),
            extra_fields: vec![],
        }),
        "groq-whisper" => make(WhisperCompatConfig {
            provider_name: "Groq Whisper".to_string(),
            endpoint: "https://api.groq.com/openai/v1/audio/transcriptions".to_string(),
            model: "whisper-large-v3-turbo".to_string(),
            extra_fields: vec![],
        }),
        "siliconflow" => make(WhisperCompatConfig {
            provider_name: "SiliconFlow".to_string(),
            endpoint: "https://api.siliconflow.cn/v1/audio/transcriptions".to_string(),
            model: "FunAudioLLM/SenseVoiceSmall".to_string(),
            extra_fields: vec![],
        }),
        "custom-whisper" => make(WhisperCompatConfig {
            provider_name: "Custom Whisper-Compatible".to_string(),
            endpoint: custom_stt_base_url
                .unwrap_or("https://api.openai.com/v1/audio/transcriptions")
                .to_string(),
            model: custom_stt_model.unwrap_or("whisper-1").to_string(),
            extra_fields: vec![],
        }),
        "xiaomi-mimo" => {
            let endpoint = xiaomi_mimo::normalize_xiaomi_endpoint(
                custom_stt_base_url
                .filter(|v| !v.trim().is_empty())
                .unwrap_or("https://token-plan-cn.xiaomimimo.com/v1"),
            );
            let model = custom_stt_model
                .filter(|v| !v.trim().is_empty())
                .unwrap_or("mimo-v2.5")
                .to_string();
            match client {
                Some(ref c) => Box::new(xiaomi_mimo::XiaomiMimoProvider::with_client(
                    endpoint,
                    model,
                    c.clone(),
                )),
                None => Box::new(xiaomi_mimo::XiaomiMimoProvider::new(endpoint, model)),
            }
        }
        "local-whisper" => {
            let cli = whisper_cli_path.unwrap_or("whisper-cli");
            let model = whisper_model_path.unwrap_or("models/ggml-tiny.bin");
            let lora = whisper_lora_path
                .filter(|p| !p.is_empty())
                .map(|p| p.to_string());
            Box::new(local_whisper::LocalWhisperProvider::new(
                cli.to_string(),
                model.to_string(),
                lora,
            ))
        }
        _ => make(WhisperCompatConfig {
            provider_name: "GLM-ASR".to_string(),
            endpoint: "https://open.bigmodel.cn/api/paas/v4/audio/transcriptions".to_string(),
            model: "glm-asr-2512".to_string(),
            extra_fields: vec![("stream".to_string(), "false".to_string())],
        }),
    }
}
