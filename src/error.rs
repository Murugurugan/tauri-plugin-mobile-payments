use serde::{ser::Serializer, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error(transparent)]
  Io(#[from] std::io::Error),

  #[error(transparent)]
  Tauri(#[from] tauri::Error),

  #[error(transparent)]
  Anyhow(#[from] anyhow::Error),

  #[cfg(mobile)]
  #[error(transparent)]
  MobileInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),

  #[cfg(not(any(target_os = "android", target_os = "ios")))]
  #[error(transparent)]
  Keyring(#[from] keyring::Error),

  // Generic string error (for machine-uid or custom messages)
  #[error("{0}")]
  PluginError(String),
}

impl Serialize for Error {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.to_string().as_ref())
  }
}