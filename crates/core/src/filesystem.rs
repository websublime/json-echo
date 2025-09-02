//! Filesystem utilities and management for the JSON Echo application.
//!
//! This module provides comprehensive filesystem operations and utilities for managing
//! file I/O operations, path resolution, and project root discovery. It serves as the
//! foundation for all file-related operations in the JSON Echo application, including
//! configuration loading, data persistence, and project structure navigation.
//!
//! ## What
//!
//! The module defines two main components:
//! - `PathUtils`: A utility struct providing static methods for path operations and root discovery
//! - `FileSystemManager`: A manager for performing async file I/O operations with proper error handling
//!
//! ## How
//!
//! The filesystem module works by:
//! 1. Providing path normalization and canonicalization utilities
//! 2. Automatically discovering project roots based on configuration file presence
//! 3. Managing async file operations with proper error propagation
//! 4. Abstracting filesystem operations behind a consistent interface
//! 5. Supporting both absolute and relative path operations
//!
//! ## Why
//!
//! This design enables:
//! - Consistent file operations across the application
//! - Automatic project root discovery for portable deployments
//! - Type-safe error handling for filesystem operations
//! - Async I/O support for non-blocking file operations
//! - Cross-platform path handling and normalization
//!
//! # Examples
//!
//! ```rust
//! use json_echo_core::{FileSystemManager, PathUtils};
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Discover project root automatically
//! let fs_manager = FileSystemManager::new(None)?;
//!
//! // Load a configuration file
//! let config_data = fs_manager.load_file("config.json").await?;
//! println!("Loaded {} bytes of configuration", config_data.len());
//!
//! // Save processed data
//! let output_data = b"processed content";
//! fs_manager.save_file("output.json", output_data.to_vec()).await?;
//!
//! // Use path utilities
//! let current = PathUtils::current_dir()?;
//! if let Some(root) = PathUtils::find_root(&current) {
//!     println!("Project root found at: {}", root.display());
//! }
//! # Ok(())
//! # }
//! ```

use crate::errors::{FileSystemError, FileSystemResult};
use std::{
    env::current_dir,
    path::{Path, PathBuf},
};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

/// Utility struct providing static methods for path operations and project navigation.
///
/// `PathUtils` offers a collection of static utility methods for common path operations
/// including current directory access, project root discovery, and path normalization.
/// All methods are stateless and can be called without instantiating the struct.
///
/// # Examples
///
/// ```rust
/// use json_echo_core::PathUtils;
/// use std::path::Path;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Get current working directory
/// let current = PathUtils::current_dir()?;
/// println!("Current directory: {}", current.display());
///
/// // Find project root
/// if let Some(root) = PathUtils::find_root(&current) {
///     println!("Project root: {}", root.display());
/// }
///
/// // Normalize a path
/// let normalized = PathUtils::normalize_path(Path::new("./some/../path"));
/// println!("Normalized path: {}", normalized.display());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct PathUtils;

impl PathUtils {
    /// Returns the current working directory.
    ///
    /// Retrieves the current working directory of the process and wraps any
    /// errors in the application's error type for consistent error handling.
    ///
    /// # Returns
    ///
    /// * `Ok(PathBuf)` - The current working directory path
    /// * `Err(FileSystemError)` - If the current directory cannot be determined
    ///
    /// # Errors
    ///
    /// This method can fail if:
    /// - The current directory has been deleted
    /// - Insufficient permissions to access the current directory
    /// - The current directory path contains invalid Unicode
    ///
    /// # Examples
    ///
    /// ```rust
    /// use json_echo_core::PathUtils;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// match PathUtils::current_dir() {
    ///     Ok(dir) => println!("Working in: {}", dir.display()),
    ///     Err(e) => eprintln!("Cannot determine current directory: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn current_dir() -> FileSystemResult<PathBuf> {
        current_dir().map_err(std::convert::Into::into)
    }

    /// Searches for the project root directory by looking for configuration files.
    ///
    /// Traverses up the directory tree from the starting path, looking for common
    /// configuration files that indicate the project root. This enables automatic
    /// project discovery regardless of where the application is executed from.
    ///
    /// # Parameters
    ///
    /// * `start` - The starting path for the search (typically current directory)
    ///
    /// # Returns
    ///
    /// * `Some(PathBuf)` - The project root directory if found
    /// * `None` - If no project root is found in the directory hierarchy
    ///
    /// # Behavior
    ///
    /// Searches for these configuration files in order:
    /// 1. `db.json`
    /// 2. `.db.json`
    /// 3. `json-echo.json`
    ///
    /// The search starts from the given path and moves up the directory tree
    /// until one of these files is found or the filesystem root is reached.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use json_echo_core::PathUtils;
    /// use std::path::Path;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let current_dir = PathUtils::current_dir()?;
    ///
    /// match PathUtils::find_root(&current_dir) {
    ///     Some(root) => {
    ///         println!("Project root found: {}", root.display());
    ///         // Use root for file operations
    ///     },
    ///     None => {
    ///         println!("No project root found, using current directory");
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn find_root(start: &Path) -> Option<PathBuf> {
        let mut current = Some(start);

        let mock_files = ["db.json", ".db.json", "json-echo.json"];

        while let Some(path) = current {
            for mock_file in &mock_files {
                if path.join(mock_file).exists() {
                    return Some(path.to_path_buf());
                }
            }

            current = path.parent();
        }

        None
    }

    /// Normalizes a path by resolving symbolic links and relative components.
    ///
    /// Attempts to canonicalize the path to resolve symbolic links, `.` and `..`
    /// components, and convert to an absolute path. If canonicalization fails,
    /// returns the original path unchanged for graceful degradation.
    ///
    /// # Parameters
    ///
    /// * `path` - The path to normalize
    ///
    /// # Returns
    ///
    /// A `PathBuf` containing the normalized path, or the original path if normalization fails
    ///
    /// # Behavior
    ///
    /// - Attempts to canonicalize the path using the filesystem
    /// - Falls back to the original path if canonicalization fails
    /// - Resolves symbolic links and relative path components
    /// - Converts relative paths to absolute paths when possible
    ///
    /// # Examples
    ///
    /// ```rust
    /// use json_echo_core::PathUtils;
    /// use std::path::Path;
    ///
    /// let messy_path = Path::new("./some/../complex/./path");
    /// let clean_path = PathUtils::normalize_path(messy_path);
    /// println!("Normalized: {}", clean_path.display());
    ///
    /// // Works with symbolic links too
    /// let symlink_path = Path::new("/some/symlink/path");
    /// let resolved_path = PathUtils::normalize_path(symlink_path);
    /// println!("Resolved: {}", resolved_path.display());
    /// ```
    pub fn normalize_path(path: &Path) -> PathBuf {
        path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
    }
}

/// Manager for performing async filesystem operations with error handling.
///
/// `FileSystemManager` provides a high-level interface for file I/O operations
/// within the context of a specific root directory. It handles path resolution,
/// async file operations, and proper error propagation for robust file handling.
///
/// # Fields
///
/// * `root` - The root directory for all file operations performed by this manager
///
/// # Examples
///
/// ```rust
/// use json_echo_core::FileSystemManager;
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create manager with automatic root discovery
/// let fs_manager = FileSystemManager::new(None)?;
///
/// // Create manager with specific root
/// let custom_root = PathBuf::from("/custom/project/root");
/// let custom_manager = FileSystemManager::new(Some(custom_root))?;
///
/// // Perform file operations
/// let data = fs_manager.load_file("config.json").await?;
/// fs_manager.save_file("backup.json", data).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct FileSystemManager {
    /// The root directory path for all file operations
    pub root: PathBuf,
}

impl FileSystemManager {
    /// Creates a new FileSystemManager with the specified or discovered root directory.
    ///
    /// Initializes a new filesystem manager with either a provided root directory
    /// or automatically discovers the project root by searching for configuration files.
    /// The root path is normalized to ensure consistent path handling.
    ///
    /// # Parameters
    ///
    /// * `root` - Optional root directory path. If None, attempts automatic discovery
    ///
    /// # Returns
    ///
    /// * `Ok(FileSystemManager)` - A new manager instance with the determined root
    /// * `Err(FileSystemError)` - If root discovery fails or paths cannot be resolved
    ///
    /// # Errors
    ///
    /// This method can fail if:
    /// - No root directory can be determined when `root` is None
    /// - The current directory cannot be accessed
    /// - Path normalization fails due to filesystem issues
    ///
    /// # Examples
    ///
    /// ```rust
    /// use json_echo_core::FileSystemManager;
    /// use std::path::PathBuf;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Automatic root discovery
    /// let auto_manager = FileSystemManager::new(None)?;
    /// println!("Auto-discovered root: {}", auto_manager.root.display());
    ///
    /// // Explicit root specification
    /// let explicit_root = PathBuf::from("/path/to/project");
    /// let explicit_manager = FileSystemManager::new(Some(explicit_root))?;
    /// println!("Explicit root: {}", explicit_manager.root.display());
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(root: Option<PathBuf>) -> FileSystemResult<Self> {
        let root = match root {
            Some(path) => PathUtils::normalize_path(&path),
            None => {
                let current_dir = PathUtils::current_dir()?;
                let path = PathUtils::find_root(&current_dir)
                    .ok_or_else(|| FileSystemError::Operation("Root directory not found".into()))?;
                PathUtils::normalize_path(&path)
            }
        };

        Ok(Self { root })
    }

    /// Asynchronously loads the contents of a file as a byte vector.
    ///
    /// Reads the entire contents of the specified file into memory as a byte vector.
    /// The file path is resolved relative to the manager's root directory, enabling
    /// portable file access regardless of the current working directory.
    ///
    /// # Parameters
    ///
    /// * `relative_file_path` - Path to the file relative to the root directory
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<u8>)` - The complete file contents as bytes
    /// * `Err(FileSystemError)` - If the file cannot be read
    ///
    /// # Errors
    ///
    /// This method can fail if:
    /// - The file does not exist
    /// - Insufficient permissions to read the file
    /// - I/O errors occur during reading
    /// - The file path is invalid or contains invalid characters
    ///
    /// # Examples
    ///
    /// ```rust
    /// use json_echo_core::FileSystemManager;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs_manager = FileSystemManager::new(None)?;
    ///
    /// // Load configuration file
    /// match fs_manager.load_file("config.json").await {
    ///     Ok(data) => {
    ///         println!("Loaded {} bytes", data.len());
    ///         // Process the data
    ///         let text = String::from_utf8_lossy(&data);
    ///         println!("Content preview: {}", &text[..100.min(text.len())]);
    ///     },
    ///     Err(e) => eprintln!("Failed to load file: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn load_file(&self, relative_file_path: &str) -> FileSystemResult<Vec<u8>> {
        let file_path = self.root.as_path().join(relative_file_path);
        let mut file = File::open(file_path).await.map_err(FileSystemError::from)?;

        let mut buffer = vec![];

        file.read_to_end(&mut buffer)
            .await
            .map_err(FileSystemError::from)?;
        Ok(buffer)
    }

    /// Asynchronously saves byte data to a file.
    ///
    /// Writes the provided byte vector to the specified file, creating the file
    /// if it doesn't exist or overwriting it if it does. The file path is resolved
    /// relative to the manager's root directory for consistent file operations.
    ///
    /// # Parameters
    ///
    /// * `relative_file_path` - Path where the file should be saved, relative to root
    /// * `content` - The byte data to write to the file
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the file was written successfully
    /// * `Err(FileSystemError)` - If the file cannot be written
    ///
    /// # Errors
    ///
    /// This method can fail if:
    /// - Insufficient permissions to create or write the file
    /// - The parent directory does not exist
    /// - Disk space is insufficient
    /// - I/O errors occur during writing
    /// - The file path is invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// use json_echo_core::FileSystemManager;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let fs_manager = FileSystemManager::new(None)?;
    ///
    /// // Save JSON configuration
    /// let config_json = r#"{"port": 3000, "host": "localhost"}"#;
    /// let config_bytes = config_json.as_bytes().to_vec();
    ///
    /// match fs_manager.save_file("new-config.json", config_bytes).await {
    ///     Ok(()) => println!("Configuration saved successfully"),
    ///     Err(e) => eprintln!("Failed to save configuration: {}", e),
    /// }
    ///
    /// // Save binary data
    /// let binary_data = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]; // "Hello" in ASCII
    /// fs_manager.save_file("data.bin", binary_data).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn save_file(
        &self,
        relative_file_path: &str,
        content: Vec<u8>,
    ) -> FileSystemResult<()> {
        let file_path = self.root.as_path().join(relative_file_path);
        let mut file = File::create(file_path)
            .await
            .map_err(FileSystemError::from)?;

        file.write_all(&content)
            .await
            .map_err(FileSystemError::from)?;
        Ok(())
    }
}
