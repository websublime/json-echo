//! Error handling and result types for the JSON Echo application.
//!
//! This module provides comprehensive error handling for filesystem operations and
//! general application errors. It defines specific error types for different failure
//! scenarios and provides type-safe error propagation throughout the application.
//!
//! ## What
//!
//! The module defines two main error types:
//! - `FileSystemError`: Specific errors related to filesystem operations
//! - `Error`: General application errors that can wrap filesystem errors
//! - `FileSystemResult<T>`: Type alias for Results with FileSystemError
//!
//! ## How
//!
//! The error system works by:
//! 1. Categorizing errors into specific types with contextual information
//! 2. Providing automatic conversions from standard library errors
//! 3. Using thiserror for ergonomic error definitions and display formatting
//! 4. Maintaining error context such as file paths and error messages
//! 5. Supporting error chaining and cause tracking
//!
//! ## Why
//!
//! This design enables:
//! - Type-safe error handling with specific error categories
//! - Rich error context for debugging and user feedback
//! - Consistent error propagation across async operations
//! - Automatic error conversions from standard library types
//! - Clear error messages with relevant contextual information
//!
//! # Examples
//!
//! ```rust
//! use json_echo_core::{FileSystemError, FileSystemResult, Error};
//! use std::path::PathBuf;
//!
//! // Creating specific filesystem errors
//! let not_found = FileSystemError::NotFound {
//!     path: PathBuf::from("/missing/file.json"),
//! };
//!
//! // Using the result type
//! fn load_config() -> FileSystemResult<String> {
//!     // This would return an error if the file doesn't exist
//!     Err(FileSystemError::NotFound {
//!         path: PathBuf::from("config.json"),
//!     })
//! }
//!
//! // Handling errors with pattern matching
//! match load_config() {
//!     Ok(content) => println!("Loaded: {}", content),
//!     Err(FileSystemError::NotFound { path }) => {
//!         eprintln!("Configuration file not found: {}", path.display());
//!     },
//!     Err(e) => eprintln!("Other error: {}", e),
//! }
//! ```

use core::result::Result as CoreResult;
use std::{io, path::PathBuf};
use thiserror::Error as ThisError;

/// Type alias for Results that can contain FileSystemError.
///
/// This type alias provides a convenient way to return Results from filesystem
/// operations without repeatedly specifying the error type. It's used throughout
/// the application for any operation that might fail with a filesystem-related error.
///
/// # Examples
///
/// ```rust
/// use json_echo_core::{FileSystemResult, FileSystemError};
/// use std::path::PathBuf;
///
/// fn read_config_file() -> FileSystemResult<String> {
///     // Implementation would read and return file contents
///     Err(FileSystemError::NotFound {
///         path: PathBuf::from("config.json"),
///     })
/// }
/// ```
pub type FileSystemResult<T> = CoreResult<T, FileSystemError>;

/// Specific error types for filesystem operations.
///
/// `FileSystemError` represents various failure scenarios that can occur during
/// filesystem operations. Each variant contains relevant contextual information
/// such as the file path involved and specific error details.
///
/// This error type is designed to provide clear, actionable error messages
/// while maintaining type safety and enabling proper error handling patterns.
///
/// # Variants
///
/// * `NotFound` - The specified path does not exist
/// * `PermissionDenied` - Insufficient permissions to access the path
/// * `Io` - Generic I/O errors with contextual information
/// * `NotADirectory` - Expected a directory but found a file
/// * `NotAFile` - Expected a file but found a directory
/// * `Utf8Decode` - Failed to decode UTF-8 content from a file
/// * `Validation` - Path validation failed for security or format reasons
/// * `Operation` - General operation failures
///
/// # Examples
///
/// ```rust
/// use json_echo_core::FileSystemError;
/// use std::path::PathBuf;
///
/// // Creating specific error types
/// let permission_error = FileSystemError::PermissionDenied {
///     path: PathBuf::from("/protected/file.txt"),
/// };
///
/// let validation_error = FileSystemError::Validation {
///     path: PathBuf::from("../../../etc/passwd"),
///     reason: "Path traversal attempt detected".to_string(),
/// };
///
/// // Pattern matching on errors
/// match permission_error {
///     FileSystemError::PermissionDenied { path } => {
///         eprintln!("Access denied to: {}", path.display());
///     },
///     _ => eprintln!("Other filesystem error"),
/// }
/// ```
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

/// General application error type that can contain various error categories.
///
/// `Error` serves as the top-level error type for the JSON Echo application,
/// capable of wrapping more specific error types like `FileSystemError` while
/// also providing general operation error handling.
///
/// This design allows for error propagation and handling at different levels
/// of the application while maintaining type safety and error context.
///
/// # Variants
///
/// * `FileSystem` - Wraps a FileSystemError with automatic conversion
/// * `Operation` - General operation errors with custom messages
///
/// # Examples
///
/// ```rust
/// use json_echo_core::{Error, FileSystemError};
/// use std::path::PathBuf;
///
/// // Automatic conversion from FileSystemError
/// let fs_error = FileSystemError::NotFound {
///     path: PathBuf::from("missing.json"),
/// };
/// let general_error: Error = fs_error.into();
///
/// // Creating operation errors
/// let op_error = Error::operation("Database connection failed");
///
/// // Pattern matching on general errors
/// match general_error {
///     Error::FileSystem(fs_err) => {
///         eprintln!("Filesystem issue: {}", fs_err);
///     },
///     Error::Operation(msg) => {
///         eprintln!("Operation failed: {}", msg);
///     },
/// }
/// ```
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
    /// Creates a new operational error with the provided message.
    ///
    /// This is a convenience method for creating `Error::Operation` variants
    /// with custom error messages. The message can be any type that implements
    /// `Into<String>` for flexible error message creation.
    ///
    /// # Parameters
    ///
    /// * `message` - The error message (can be &str, String, or other `Into<String>` types)
    ///
    /// # Returns
    ///
    /// A new `Error::Operation` instance with the provided message
    ///
    /// # Examples
    ///
    /// ```rust
    /// use json_echo_core::Error;
    ///
    /// // Create with string literal
    /// let error1 = Error::operation("Connection timeout");
    ///
    /// // Create with owned string
    /// let message = format!("Failed to process {} items", 42);
    /// let error2 = Error::operation(message);
    ///
    /// // Create with dynamic content
    /// let user_id = 123;
    /// let error3 = Error::operation(format!("User {} not found", user_id));
    /// ```
    pub fn operation(message: impl Into<String>) -> Self {
        Self::Operation(message.into())
    }
}

impl From<io::Error> for FileSystemError {
    /// Converts standard library I/O errors into FileSystemError.
    ///
    /// This implementation enables automatic conversion from `std::io::Error`
    /// to `FileSystemError`, making error handling more ergonomic when working
    /// with filesystem operations that use standard library functions.
    ///
    /// # Parameters
    ///
    /// * `error` - The I/O error to convert
    ///
    /// # Returns
    ///
    /// A corresponding `FileSystemError` variant based on the I/O error kind
    ///
    /// # Behavior
    ///
    /// Maps I/O error kinds to specific FileSystemError variants:
    /// - `NotFound` → `FileSystemError::NotFound`
    /// - `PermissionDenied` → `FileSystemError::PermissionDenied`
    /// - Other kinds → `FileSystemError::Io`
    ///
    /// Note: Uses a placeholder path since the original path context is not
    /// available in the I/O error. Applications should prefer creating
    /// FileSystemError variants directly when path context is available.
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
    /// Converts serde_json errors into FileSystemError::Utf8Decode.
    ///
    /// This implementation enables automatic conversion from JSON parsing
    /// errors to filesystem errors, treating JSON parsing failures as
    /// UTF-8 decoding issues since they often occur when reading files.
    ///
    /// # Parameters
    ///
    /// * `error` - The serde_json error to convert
    ///
    /// # Returns
    ///
    /// A `FileSystemError::Utf8Decode` variant containing the error details
    ///
    /// # Behavior
    ///
    /// All serde_json errors are mapped to `Utf8Decode` variants with:
    /// - A placeholder path (since path context is not available)
    /// - The original error message for debugging purposes
    ///
    /// Applications should prefer creating FileSystemError variants directly
    /// when file path context is available for better error reporting.
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
    /// Provides a string reference representing the error category.
    ///
    /// This implementation enables pattern matching and categorization of
    /// errors based on their type without needing to destructure the full
    /// error variants. Useful for logging, metrics, and error classification.
    ///
    /// # Returns
    ///
    /// A static string slice representing the error category:
    /// - `"Error::FileSystem"` for filesystem-related errors
    /// - `"Error::Operation"` for general operation errors
    ///
    /// # Examples
    ///
    /// ```rust
    /// use json_echo_core::{Error, FileSystemError};
    /// use std::path::PathBuf;
    ///
    /// let fs_error = Error::FileSystem(FileSystemError::NotFound {
    ///     path: PathBuf::from("test.json"),
    /// });
    /// let op_error = Error::operation("Something went wrong");
    ///
    /// assert_eq!(fs_error.as_ref(), "Error::FileSystem");
    /// assert_eq!(op_error.as_ref(), "Error::Operation");
    ///
    /// // Useful for logging or categorization
    /// match error.as_ref() {
    ///     "Error::FileSystem" => log::warn!("Filesystem issue detected"),
    ///     "Error::Operation" => log::error!("Operation failure occurred"),
    ///     _ => log::error!("Unknown error type"),
    /// }
    /// ```
    fn as_ref(&self) -> &str {
        match self {
            Error::FileSystem(_) => "Error::FileSystem",
            Error::Operation(_) => "Error::Operation",
        }
    }
}
