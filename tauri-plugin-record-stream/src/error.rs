use serde::{ser::Serializer, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error(transparent)]
  Io(#[from] std::io::Error),
  #[cfg(mobile)]
  #[error(transparent)]
  PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),
  #[error("Video writer error: {0}")]
  VideoWriterError(String),
  #[error("Image processing error: {0}")]
  ImageError(String),
  #[error("Base64 decoding error")]
  Base64Error(#[from] base64::DecodeError),
  #[cfg(not(target_os = "android"))]
  #[error("OpenCV error")]
  OpenCvError(#[from] opencv::Error),
  #[error("Image load error")]
  ImageLoadError(#[from] image::ImageError),
  #[error("Mutex lock error")]
  MutexError,
}

impl Serialize for Error {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.to_string().as_ref())
  }
}
