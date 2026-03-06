use serde::{ser::Serializer, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error(transparent)]
  Io(#[from] std::io::Error),

  // Generic Tauri errors (like spawn_blocking failures)
  #[error(transparent)]
  Tauri(#[from] tauri::Error),

  // Errors coming specifically from the Kotlin/Swift side
  #[error(transparent)]
  MobileInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),

  // 👇 DESKTOP ONLY: Handles Keyring/Credential Manager errors
  #[cfg(not(any(target_os = "android", target_os = "ios")))]
  #[error(transparent)]
  Keyring(#[from] keyring::Error),

  // 👇 YOUR REQUEST: A generic string error for custom logic or machine-uid
  #[error("{0}")]
  PluginError(String),
}

// This ensures that when an error happens, the Frontend receives just the string message
impl Serialize for Error {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.to_string().as_ref())
  }
}