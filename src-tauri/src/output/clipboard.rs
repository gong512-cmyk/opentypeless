use anyhow::Result;
use async_trait::async_trait;

use super::{OutputMode, TextOutput};

/// Delay after writing to clipboard before simulating paste.
const CLIPBOARD_SETTLE_MS: u64 = 20;

pub struct ClipboardOutput;

impl Default for ClipboardOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardOutput {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl TextOutput for ClipboardOutput {
    async fn type_text(&self, text: &str) -> Result<()> {
        let text = text.to_string();
        tokio::task::spawn_blocking(move || {
            let mut clipboard = arboard::Clipboard::new()
                .map_err(|e| anyhow::anyhow!("Failed to access clipboard: {}", e))?;

            clipboard
                .set_text(&text)
                .map_err(|e| anyhow::anyhow!("Failed to set clipboard: {}", e))?;

            std::thread::sleep(std::time::Duration::from_millis(CLIPBOARD_SETTLE_MS));

            // On macOS: try osascript Cmd+V (needs Accessibility permission).
            // If that fails, silently fall back — text is already on clipboard.
            #[cfg(target_os = "macos")]
            {
                let status = std::process::Command::new("osascript")
                    .args([
                        "-e",
                        r#"tell application "System Events" to keystroke "v" using command down"#,
                    ])
                    .status();
                match status {
                    Ok(s) if s.success() => {}
                    _ => tracing::debug!("osascript paste unavailable — text on clipboard, user can Cmd+V"),
                }
            }

            #[cfg(not(target_os = "macos"))]
            {
                use enigo::{Direction, Enigo, Key, Keyboard, Settings};
                if let Ok(mut enigo) = Enigo::new(&Settings::default()) {
                    let _ = enigo.key(Key::Control, Direction::Press);
                    let _ = enigo.key(Key::Unicode('v'), Direction::Click);
                    let _ = enigo.key(Key::Control, Direction::Release);
                }
            }

            Ok(())
        })
        .await?
    }

    fn mode(&self) -> OutputMode {
        OutputMode::Clipboard
    }
}
