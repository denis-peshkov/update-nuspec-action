use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum LibError {
    #[error("failed to read file {path}: {source}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to write file {path}: {source}")]
    WriteFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid XML in {path}: {message}")]
    InvalidXml { path: PathBuf, message: String },

    #[error("nuspec metadata is missing <dependencies> in {path}")]
    MissingDependencies { path: PathBuf },
}
