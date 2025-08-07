use core::result::Result as CoreResult;
use std::{io, path::PathBuf};
use thiserror::Error as ThisError;

pub type FileSystemResult<T> = CoreResult<T, FileSystemError>;

#[derive(ThisError, Debug, Clone)]
pub enum FileSystemError {
    /// Path not found.
    #[error("Path not found: {path}")]
    NotFound {
        /// The path that was not found
        path: PathBuf,
    },

    /// Permission denied for accessing the path.
    #[error("Permission denied for path: {path}")]
    PermissionDenied {
        /// The path for which permission was denied
        path: PathBuf,
    },

    /// Generic I/O error during filesystem operation.
    #[error("I/O error accessing path '{path}': {message}")]
    Io {
        /// The path where the I/O error occurred
        path: PathBuf,
        /// The I/O error message
        message: String,
    },

    /// Attempted an operation requiring a directory on a file.
    #[error("Expected a directory but found a file: {path}")]
    NotADirectory {
        /// The path that was expected to be a directory but wasn't
        path: PathBuf,
    },

    /// Attempted an operation requiring a file on a directory.
    #[error("Expected a file but found a directory: {path}")]
    NotAFile {
        /// The path that was expected to be a file but wasn't
        path: PathBuf,
    },

    /// Failed to decode UTF-8 content from a file.
    #[error("Failed to decode UTF-8 content in file: {path} - {message}")]
    Utf8Decode {
        /// The path to the file with invalid UTF-8 content
        path: PathBuf,
        /// The UTF-8 decoding error message
        message: String,
    },

    /// Path validation failed (e.g., contains '..', absolute path, symlink).
    #[error("Path validation failed for '{path}': {reason}")]
    Validation {
        /// The path that failed validation
        path: PathBuf,
        /// The reason why validation failed
        reason: String,
    },

    /// Operation failed (e.g., timeout, concurrency limit exceeded).
    #[error("Operation failed: {0}")]
    Operation(String),
}

#[derive(ThisError, Debug, Clone)]
pub enum Error {
    /// Filesystem-related error.
    #[error("FileSystem execution error")]
    FileSystem(#[from] FileSystemError),

    /// General purpose errors with a custom message.
    #[error("Operation error: {0}")]
    Operation(String),
}

impl Error {
    /// Creates a new operational error.
    pub fn operation(message: impl Into<String>) -> Self {
        Self::Operation(message.into())
    }
}

impl From<io::Error> for FileSystemError {
    fn from(error: io::Error) -> Self {
        // Create a dummy path or indicate unknown path
        let path = PathBuf::from("<unknown>");
        match error.kind() {
            io::ErrorKind::NotFound => Self::NotFound { path },
            io::ErrorKind::PermissionDenied => Self::PermissionDenied { path },
            _ => Self::Io {
                path,
                message: error.to_string(),
            },
        }
    }
}

impl From<serde_json::Error> for FileSystemError {
    fn from(error: serde_json::Error) -> Self {
        // Create a dummy path or indicate unknown path
        let path = PathBuf::from("<unknown>");
        Self::Utf8Decode {
            path,
            message: error.to_string(),
        }
    }
}

impl AsRef<str> for Error {
    fn as_ref(&self) -> &str {
        match self {
            Error::FileSystem(_) => "Error::FileSystem",
            Error::Operation(_) => "Error::Operation",
        }
    }
}
