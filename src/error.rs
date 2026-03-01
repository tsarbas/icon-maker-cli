use std::io;
use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum IconMakerError {
    #[error("OpenAI API key is missing (set OPENAI_API_KEY or config openai_key)")]
    MissingApiKey,
    #[error("output directory already exists: {0}. Use --force to overwrite.")]
    OutputExists(PathBuf),
    #[error(transparent)]
    Io(#[from] io::Error),
}
