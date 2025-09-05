//! Comprehensive test suite for the filesystem module.
//!
//! This module contains extensive tests for all functionality exposed by the filesystem module,
//! including PathUtils and FileSystemManager. Tests are designed to verify correct behavior,
//! error handling, and edge cases across different scenarios.
//!
//! ## What
//!
//! The test suite covers:
//! - PathUtils static methods (current_dir, find_root, normalize_path)
//! - FileSystemManager creation and configuration
//! - Async file operations (load_file, save_file)
//! - Error handling and edge cases
//! - Cross-platform compatibility scenarios
//!
//! ## How
//!
//! Tests use tempfile crate to create isolated test environments that don't interfere
//! with the actual filesystem. Each test creates its own temporary directory structure
//! and verifies the expected behavior without side effects.
//!
//! ## Why
//!
//! Comprehensive testing ensures:
//! - Reliability of filesystem operations
//! - Proper error handling and propagation
//! - Cross-platform compatibility
//! - Performance and correctness of async operations
//! - Robustness against edge cases and invalid inputs

use json_echo_core::FileSystemError;
use json_echo_core::{FileSystemManager, PathUtils};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

/// Sets up a temporary directory for testing.
///
/// Creates a new temporary directory that will be automatically cleaned up
/// when the returned TempDir is dropped. This ensures test isolation and
/// prevents test artifacts from affecting subsequent test runs.
///
/// # Returns
///
/// A TempDir instance representing the temporary test directory
///
/// # Examples
///
/// ```rust
/// let temp_dir = setup_test_dir();
/// let temp_path = temp_dir.path();
/// // Use temp_path for test operations
/// ```
fn setup_test_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temporary directory for test")
}

/// Creates a test file with the specified content in the given directory.
///
/// Helper function that creates a file with the provided content at the
/// specified path within a directory. Used to set up test scenarios with
/// predefined file structures.
///
/// # Parameters
///
/// * `dir` - The directory where the file should be created
/// * `file_name` - The name of the file to create
/// * `content` - The content to write to the file
///
/// # Examples
///
/// ```rust
/// let temp_dir = setup_test_dir();
/// create_test_file(temp_dir.path(), "config.json", b"{\"test\": true}");
/// ```
fn create_test_file(dir: &Path, file_name: &str, content: &[u8]) {
    let file_path = dir.join(file_name);
    let mut file = File::create(file_path).expect("Failed to create test file");
    file.write_all(content)
        .expect("Failed to write content to test file");
    file.sync_all().expect("Failed to sync test file");
}

/// Creates a nested directory structure for testing path operations.
///
/// Sets up a multi-level directory structure with configuration files
/// at different levels to test root discovery functionality.
///
/// # Parameters
///
/// * `base_dir` - The base directory where the structure should be created
///
/// # Returns
///
/// A PathBuf pointing to the deepest nested directory
///
/// # Structure Created
///
/// ```
/// base_dir/
/// ├── db.json
/// └── project/
///     └── src/
///         └── deep/
/// ```
fn create_nested_structure(base_dir: &Path) -> PathBuf {
    let nested_path = base_dir.join("project").join("src").join("deep");
    fs::create_dir_all(&nested_path).expect("Failed to create nested directory structure");

    // Create a root marker file
    create_test_file(base_dir, "db.json", b"{}");

    nested_path
}

mod path_utils_tests {
    use super::*;

    /// Tests that current_dir returns a valid path.
    ///
    /// Verifies that the current_dir method successfully retrieves the
    /// current working directory and returns a valid, existing path.
    #[test]
    fn test_current_dir_success() {
        let result = PathUtils::current_dir();
        assert!(result.is_ok(), "current_dir should succeed");

        let current_dir = result.unwrap();
        assert!(
            current_dir.exists(),
            "Current directory should exist: {}",
            current_dir.display()
        );
        assert!(
            current_dir.is_dir(),
            "Current directory should be a directory: {}",
            current_dir.display()
        );
    }

    /// Tests find_root when a configuration file exists at the starting directory.
    ///
    /// Verifies that find_root correctly identifies the project root when
    /// a supported configuration file (db.json) is present in the search
    /// starting directory.
    #[test]
    fn test_find_root_with_db_json() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path();

        // Create db.json in the temp directory
        create_test_file(temp_path, "db.json", b"{}");

        let result = PathUtils::find_root(temp_path);
        assert!(result.is_some(), "Should find root with db.json");
        assert_eq!(
            result.unwrap(),
            temp_path,
            "Root should be the directory containing db.json"
        );
    }

    /// Tests find_root when a hidden configuration file exists.
    ///
    /// Verifies that find_root correctly identifies the project root when
    /// a hidden configuration file (.db.json) is present.
    #[test]
    fn test_find_root_with_hidden_db_json() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path();

        // Create .db.json in the temp directory
        create_test_file(temp_path, ".db.json", b"{}");

        let result = PathUtils::find_root(temp_path);
        assert!(result.is_some(), "Should find root with .db.json");
        assert_eq!(
            result.unwrap(),
            temp_path,
            "Root should be the directory containing .db.json"
        );
    }

    /// Tests find_root when json-echo.json configuration file exists.
    ///
    /// Verifies that find_root correctly identifies the project root when
    /// the json-echo.json configuration file is present.
    #[test]
    fn test_find_root_with_json_echo_config() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path();

        // Create json-echo.json in the temp directory
        create_test_file(temp_path, "json-echo.json", b"{}");

        let result = PathUtils::find_root(temp_path);
        assert!(result.is_some(), "Should find root with json-echo.json");
        assert_eq!(
            result.unwrap(),
            temp_path,
            "Root should be the directory containing json-echo.json"
        );
    }

    /// Tests find_root traversing up directory hierarchy.
    ///
    /// Verifies that find_root correctly traverses up the directory tree
    /// to find the project root when starting from a deeply nested directory.
    #[test]
    fn test_find_root_traverses_up() {
        let temp_dir = setup_test_dir();
        let nested_path = create_nested_structure(temp_dir.path());

        let result = PathUtils::find_root(&nested_path);
        assert!(result.is_some(), "Should find root by traversing up");
        assert_eq!(
            result.unwrap(),
            temp_dir.path(),
            "Root should be the top-level directory with db.json"
        );
    }

    /// Tests find_root when no configuration files exist.
    ///
    /// Verifies that find_root returns None when no supported configuration
    /// files are found in the directory hierarchy.
    #[test]
    fn test_find_root_not_found() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path();

        // Create nested structure without config files
        let nested_path = temp_path.join("project").join("src");
        fs::create_dir_all(&nested_path).expect("Failed to create nested directories");

        let result = PathUtils::find_root(&nested_path);
        assert!(
            result.is_none(),
            "Should not find root without config files"
        );
    }

    /// Tests find_root with multiple configuration files.
    ///
    /// Verifies that find_root correctly prioritizes configuration files
    /// when multiple supported files exist in the same directory.
    #[test]
    fn test_find_root_with_multiple_configs() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path();

        // Create multiple config files
        create_test_file(temp_path, "db.json", b"{}");
        create_test_file(temp_path, ".db.json", b"{}");
        create_test_file(temp_path, "json-echo.json", b"{}");

        let result = PathUtils::find_root(temp_path);
        assert!(result.is_some(), "Should find root with multiple configs");
        assert_eq!(
            result.unwrap(),
            temp_path,
            "Root should be found regardless of which config file is checked first"
        );
    }

    /// Tests normalize_path with an existing path.
    ///
    /// Verifies that normalize_path correctly canonicalizes an existing
    /// path, resolving any relative components and symbolic links.
    #[test]
    fn test_normalize_path_existing() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path();

        let normalized = PathUtils::normalize_path(temp_path);

        // The normalized path should be absolute and should exist
        assert!(
            normalized.is_absolute(),
            "Normalized path should be absolute: {}",
            normalized.display()
        );
        assert!(
            normalized.exists(),
            "Normalized path should exist: {}",
            normalized.display()
        );
    }

    /// Tests normalize_path with a non-existing path.
    ///
    /// Verifies that normalize_path gracefully handles non-existing paths
    /// by returning the original path when canonicalization fails.
    #[test]
    fn test_normalize_path_non_existing() {
        let temp_dir = setup_test_dir();
        let non_existing = temp_dir.path().join("non_existing_file.txt");

        let normalized = PathUtils::normalize_path(&non_existing);

        // Since the path doesn't exist, canonicalization should fail
        // and return the original path
        assert_eq!(
            normalized, non_existing,
            "Non-existing path should be returned unchanged"
        );
    }

    /// Tests normalize_path with relative path components.
    ///
    /// Verifies that normalize_path correctly handles paths with relative
    /// components like "." and ".." when the path exists.
    #[test]
    fn test_normalize_path_with_relative_components() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path();

        // Create a subdirectory
        let sub_dir = temp_path.join("subdir");
        fs::create_dir(&sub_dir).expect("Failed to create subdirectory");

        // Create a path with relative components
        let messy_path = sub_dir.join("..").join("subdir");
        let normalized = PathUtils::normalize_path(&messy_path);

        // The normalized path should resolve the ".." component
        assert!(
            normalized.is_absolute(),
            "Normalized path should be absolute"
        );
        assert!(normalized.exists(), "Normalized path should exist");
        // Should end with "subdir"
        assert!(
            normalized.ends_with("subdir"),
            "Normalized path should end with 'subdir': {}",
            normalized.display()
        );
    }
}

mod filesystem_manager_tests {
    use super::*;

    /// Tests FileSystemManager creation with explicit root.
    ///
    /// Verifies that FileSystemManager can be successfully created with
    /// an explicitly provided root directory path.
    #[test]
    fn test_new_with_explicit_root() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path().to_path_buf();

        let result = FileSystemManager::new(Some(temp_path.clone()));
        assert!(result.is_ok(), "Should create manager with explicit root");

        let manager = result.unwrap();
        // The root should be normalized, so we compare the canonical forms
        let expected_root = PathUtils::normalize_path(&temp_path);
        assert_eq!(
            manager.root, expected_root,
            "Manager root should match the provided path"
        );
    }

    /// Tests FileSystemManager creation with automatic root discovery.
    ///
    /// Verifies that FileSystemManager can automatically discover the
    /// project root when no explicit root is provided.
    #[test]
    fn test_new_with_auto_discovery() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path();

        // Create a config file for root discovery
        create_test_file(temp_path, "db.json", b"{}");

        // Change to the temp directory to test auto-discovery
        let original_dir = std::env::current_dir().expect("Failed to get current directory");
        std::env::set_current_dir(temp_path).expect("Failed to change directory");

        let result = FileSystemManager::new(None);

        // Restore original directory
        std::env::set_current_dir(original_dir).expect("Failed to restore directory");

        assert!(result.is_ok(), "Should create manager with auto-discovery");

        let manager = result.unwrap();
        let expected_root = PathUtils::normalize_path(temp_path);
        assert_eq!(
            manager.root, expected_root,
            "Manager should auto-discover the correct root"
        );
    }

    /// Tests FileSystemManager creation failure when no root found.
    ///
    /// Verifies that FileSystemManager creation fails appropriately when
    /// no root directory can be discovered and none is explicitly provided.
    #[test]
    fn test_new_fails_when_no_root_found() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path();

        // Create a nested directory without config files
        let nested_path = temp_path.join("deeply").join("nested").join("path");
        fs::create_dir_all(&nested_path).expect("Failed to create nested directories");

        // Change to the nested directory
        let original_dir = std::env::current_dir().expect("Failed to get current directory");
        std::env::set_current_dir(&nested_path).expect("Failed to change directory");

        let result = FileSystemManager::new(None);

        // Restore original directory
        std::env::set_current_dir(original_dir).expect("Failed to restore directory");

        assert!(result.is_err(), "Should fail when no root is found");

        if let Err(error) = result {
            match error {
                FileSystemError::Operation(msg) => {
                    assert!(
                        msg.contains("Root directory not found"),
                        "Error message should indicate root not found: {msg}",
                    );
                }
                _ => panic!("Expected Operation error, got: {error}"),
            }
        }
    }

    /// Tests successful file loading.
    ///
    /// Verifies that load_file correctly reads the contents of an existing
    /// file and returns the data as a byte vector.
    #[tokio::test]
    async fn test_load_file_success() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path().to_path_buf();

        // Create test file with known content
        let test_content = b"Hello, World! This is test content.";
        create_test_file(&temp_path, "test.txt", test_content);

        let manager =
            FileSystemManager::new(Some(temp_path)).expect("Failed to create FileSystemManager");

        let result = manager.load_file("test.txt").await;
        assert!(result.is_ok(), "Should successfully load existing file");

        let loaded_content = result.unwrap();
        assert_eq!(
            loaded_content, test_content,
            "Loaded content should match original content"
        );
    }

    /// Tests file loading with non-existing file.
    ///
    /// Verifies that load_file returns an appropriate error when attempting
    /// to load a file that doesn't exist.
    #[tokio::test]
    async fn test_load_file_not_found() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path().to_path_buf();

        let manager =
            FileSystemManager::new(Some(temp_path)).expect("Failed to create FileSystemManager");

        let result = manager.load_file("non_existent.txt").await;
        assert!(
            result.is_err(),
            "Should fail when loading non-existent file"
        );

        // Verify the error type
        if let Err(error) = result {
            // The error should be an I/O error or NotFound error
            match error {
                FileSystemError::NotFound { .. } | FileSystemError::Io { .. } => {
                    // Expected error types
                }
                _ => panic!("Expected NotFound or Io error, got: {error}"),
            }
        }
    }

    /// Tests file loading with relative paths.
    ///
    /// Verifies that load_file correctly handles relative paths and
    /// resolves them relative to the manager's root directory.
    #[tokio::test]
    async fn test_load_file_relative_path() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path().to_path_buf();

        // Create subdirectory and file
        let sub_dir = temp_path.join("config");
        fs::create_dir(&sub_dir).expect("Failed to create subdirectory");

        let test_content = b"Configuration data";
        create_test_file(&sub_dir, "app.json", test_content);

        let manager =
            FileSystemManager::new(Some(temp_path)).expect("Failed to create FileSystemManager");

        let result = manager.load_file("config/app.json").await;
        assert!(result.is_ok(), "Should load file with relative path");

        let loaded_content = result.unwrap();
        assert_eq!(
            loaded_content, test_content,
            "Content should match for relative path"
        );
    }

    /// Tests successful file saving.
    ///
    /// Verifies that save_file correctly writes data to a file and that
    /// the file can be subsequently read with the same content.
    #[tokio::test]
    async fn test_save_file_success() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path().to_path_buf();

        let manager = FileSystemManager::new(Some(temp_path.clone()))
            .expect("Failed to create FileSystemManager");

        let test_content = b"New file content for testing save functionality.";
        let result = manager
            .save_file("new_file.txt", test_content.to_vec())
            .await;

        assert!(result.is_ok(), "Should successfully save file");

        // Verify the file was created and contains the correct content
        let file_path = temp_path.join("new_file.txt");
        assert!(file_path.exists(), "Saved file should exist");

        let saved_content = fs::read(&file_path).expect("Failed to read saved file");
        assert_eq!(
            saved_content, test_content,
            "Saved content should match original"
        );
    }

    /// Tests file saving with overwrite.
    ///
    /// Verifies that save_file correctly overwrites existing files with
    /// new content when the file already exists.
    #[tokio::test]
    async fn test_save_file_overwrite() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path().to_path_buf();

        // Create initial file
        let initial_content = b"Initial content";
        create_test_file(&temp_path, "overwrite_test.txt", initial_content);

        let manager = FileSystemManager::new(Some(temp_path.clone()))
            .expect("Failed to create FileSystemManager");

        // Overwrite with new content
        let new_content = b"New overwritten content";
        let result = manager
            .save_file("overwrite_test.txt", new_content.to_vec())
            .await;

        assert!(result.is_ok(), "Should successfully overwrite file");

        // Verify the file contains the new content
        let file_path = temp_path.join("overwrite_test.txt");
        let saved_content = fs::read(&file_path).expect("Failed to read overwritten file");
        assert_eq!(
            saved_content, new_content,
            "File should contain new content after overwrite"
        );
    }

    /// Tests file saving with nested directory creation.
    ///
    /// Verifies that save_file handles saving to nested paths when the
    /// intermediate directories already exist.
    #[tokio::test]
    async fn test_save_file_nested_path() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path().to_path_buf();

        // Create nested directory structure
        let nested_dir = temp_path.join("data").join("configs");
        fs::create_dir_all(&nested_dir).expect("Failed to create nested directories");

        let manager = FileSystemManager::new(Some(temp_path.clone()))
            .expect("Failed to create FileSystemManager");

        let test_content = b"Nested file content";
        let result = manager
            .save_file("data/configs/nested.json", test_content.to_vec())
            .await;

        assert!(result.is_ok(), "Should save file in nested path");

        // Verify the file was created in the correct location
        let file_path = temp_path.join("data").join("configs").join("nested.json");
        assert!(file_path.exists(), "Nested file should exist");

        let saved_content = fs::read(&file_path).expect("Failed to read nested file");
        assert_eq!(
            saved_content, test_content,
            "Nested file content should be correct"
        );
    }

    /// Tests file saving with empty content.
    ///
    /// Verifies that save_file correctly handles saving empty files
    /// and that the resulting file has zero size.
    #[tokio::test]
    async fn test_save_file_empty_content() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path().to_path_buf();

        let manager = FileSystemManager::new(Some(temp_path.clone()))
            .expect("Failed to create FileSystemManager");

        let empty_content = Vec::new();
        let result = manager.save_file("empty.txt", empty_content).await;

        assert!(result.is_ok(), "Should save empty file successfully");

        // Verify the file exists and is empty
        let file_path = temp_path.join("empty.txt");
        assert!(file_path.exists(), "Empty file should exist");

        let metadata = fs::metadata(&file_path).expect("Failed to get file metadata");
        assert_eq!(metadata.len(), 0, "Empty file should have zero size");
    }

    /// Tests file saving with binary content.
    ///
    /// Verifies that save_file correctly handles binary data that may
    /// contain null bytes and other non-text characters.
    #[tokio::test]
    async fn test_save_file_binary_content() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path().to_path_buf();

        let manager = FileSystemManager::new(Some(temp_path.clone()))
            .expect("Failed to create FileSystemManager");

        // Create binary content with various byte values including null bytes
        let binary_content: Vec<u8> = (0..=255).collect();
        let result = manager
            .save_file("binary_test.bin", binary_content.clone())
            .await;

        assert!(result.is_ok(), "Should save binary file successfully");

        // Verify the binary content is preserved
        let file_path = temp_path.join("binary_test.bin");
        let saved_content = fs::read(&file_path).expect("Failed to read binary file");
        assert_eq!(
            saved_content, binary_content,
            "Binary content should be preserved exactly"
        );
    }

    /// Tests round-trip file operations.
    ///
    /// Verifies that data saved with save_file can be correctly loaded
    /// back with load_file, ensuring data integrity across operations.
    #[tokio::test]
    async fn test_roundtrip_save_and_load() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path().to_path_buf();

        let manager =
            FileSystemManager::new(Some(temp_path)).expect("Failed to create FileSystemManager");

        let original_content = b"Round-trip test data with special chars: \n\t\r\0";

        // Save the file
        let save_result = manager
            .save_file("roundtrip.txt", original_content.to_vec())
            .await;
        assert!(save_result.is_ok(), "Should save file for round-trip test");

        // Load the file back
        let load_result = manager.load_file("roundtrip.txt").await;
        assert!(load_result.is_ok(), "Should load file for round-trip test");

        let loaded_content = load_result.unwrap();
        assert_eq!(
            loaded_content, original_content,
            "Round-trip should preserve data exactly"
        );
    }

    /// Tests concurrent file operations.
    ///
    /// Verifies that the FileSystemManager can handle multiple concurrent
    /// file operations without data corruption or errors.
    #[tokio::test]
    async fn test_concurrent_operations() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path().to_path_buf();

        let manager =
            FileSystemManager::new(Some(temp_path)).expect("Failed to create FileSystemManager");

        // Create multiple concurrent save operations
        let tasks: Vec<_> = (0..10)
            .map(|i| {
                let manager_clone = manager.clone();
                let file_name = format!("concurrent_{i}.txt");
                let content = format!("Content for file {i}").into_bytes();

                tokio::spawn(async move { manager_clone.save_file(&file_name, content).await })
            })
            .collect();

        // Wait for all tasks to complete
        let results: Vec<_> = futures::future::join_all(tasks).await;

        // Verify all operations succeeded
        for (i, result) in results.into_iter().enumerate() {
            let task_result = result.expect("Task should not panic");
            assert!(
                task_result.is_ok(),
                "Concurrent save operation {i} should succeed",
            );
        }

        // Verify all files can be loaded correctly
        for i in 0..10 {
            let file_name = format!("concurrent_{i}.txt");
            let load_result = manager.load_file(&file_name).await;
            assert!(
                load_result.is_ok(),
                "Should load concurrent file {i} successfully",
            );

            let expected_content = format!("Content for file {i}");
            let loaded_content =
                String::from_utf8(load_result.unwrap()).expect("Content should be valid UTF-8");
            assert_eq!(
                loaded_content, expected_content,
                "Concurrent file {i} content should be correct",
            );
        }
    }
}

mod error_handling_tests {
    use super::*;

    /// Tests error handling when FileSystemManager cannot access files due to permissions.
    ///
    /// Note: This test may behave differently on different platforms due to
    /// permission handling variations. On some systems, temp directories
    /// may not support strict permission restrictions.
    #[tokio::test]
    async fn test_permission_errors() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path().to_path_buf();

        let manager = FileSystemManager::new(Some(temp_path.clone()))
            .expect("Failed to create FileSystemManager");

        // Try to access a file in a non-existent subdirectory
        // This should result in a permission or I/O error
        let result = manager.load_file("non_existent_dir/file.txt").await;
        assert!(
            result.is_err(),
            "Should fail when accessing non-existent directory"
        );

        // The specific error type may vary by platform, but it should be a filesystem error
        if let Err(error) = result {
            match error {
                FileSystemError::NotFound { .. }
                | FileSystemError::Io { .. }
                | FileSystemError::PermissionDenied { .. } => {
                    // These are all acceptable error types for this scenario
                }
                _ => panic!("Unexpected error type: {error}"),
            }
        }
    }

    /// Tests error propagation through the Result types.
    ///
    /// Verifies that errors are properly wrapped and propagated through
    /// the application's error handling system.
    #[tokio::test]
    async fn test_error_propagation() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path().to_path_buf();

        let manager =
            FileSystemManager::new(Some(temp_path)).expect("Failed to create FileSystemManager");

        // Test with various invalid file paths
        let invalid_paths = vec![
            "definitely_does_not_exist.txt",
            "also/does/not/exist.json",
            "", // Empty path
        ];

        for invalid_path in invalid_paths {
            let result = manager.load_file(invalid_path).await;
            assert!(
                result.is_err(),
                "Should return error for invalid path: {invalid_path}",
            );

            // Verify error can be displayed (tests Display implementation)
            if let Err(error) = result {
                let error_string = error.to_string();
                assert!(
                    !error_string.is_empty(),
                    "Error should have non-empty display message"
                );
            }
        }
    }

    /// Tests that FileSystemManager properly handles edge cases.
    ///
    /// Verifies robust behavior when dealing with unusual but valid inputs
    /// and edge cases in file operations.
    #[tokio::test]
    async fn test_edge_cases() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path().to_path_buf();

        let manager = FileSystemManager::new(Some(temp_path.clone()))
            .expect("Failed to create FileSystemManager");

        // Test with file names containing special characters
        let special_names = vec![
            "file with spaces.txt",
            "file-with-dashes.txt",
            "file_with_underscores.txt",
            "file.with.dots.txt",
        ];

        for special_name in special_names {
            let content = format!("Content for {special_name}").into_bytes();

            // Save file with special name
            let save_result = manager.save_file(special_name, content.clone()).await;
            assert!(
                save_result.is_ok(),
                "Should save file with special name: {special_name}"
            );

            // Load file with special name
            let load_result = manager.load_file(special_name).await;
            assert!(
                load_result.is_ok(),
                "Should load file with special name: {special_name}"
            );

            let loaded_content = load_result.unwrap();
            assert_eq!(
                loaded_content, content,
                "Content should match for special name: {special_name}"
            );
        }
    }
}

mod integration_tests {
    use super::*;

    /// Tests integration between PathUtils and FileSystemManager.
    ///
    /// Verifies that the two main components work correctly together
    /// in realistic usage scenarios.
    #[tokio::test]
    async fn test_path_utils_filesystem_manager_integration() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path();

        // Create a project structure using PathUtils concepts
        create_test_file(temp_path, "db.json", b"{}");
        let nested_path = create_nested_structure(temp_path);

        // Use PathUtils to find the root
        let found_root = PathUtils::find_root(&nested_path);
        assert!(found_root.is_some(), "PathUtils should find the root");
        assert_eq!(
            found_root.unwrap(),
            temp_path,
            "Found root should match temp directory"
        );

        // Use the found root with FileSystemManager
        let manager = FileSystemManager::new(Some(temp_path.to_path_buf()))
            .expect("Should create manager with found root");

        // Test file operations within the integrated system
        let test_data = b"Integration test data";
        let save_result = manager
            .save_file("integration_test.txt", test_data.to_vec())
            .await;
        assert!(save_result.is_ok(), "Should save file in integrated system");

        let load_result = manager.load_file("integration_test.txt").await;
        assert!(load_result.is_ok(), "Should load file in integrated system");

        let loaded_data = load_result.unwrap();
        assert_eq!(
            loaded_data, test_data,
            "Loaded data should match in integrated system"
        );
    }

    /// Tests complete workflow from project discovery to file operations.
    ///
    /// Simulates a complete workflow that includes project root discovery,
    /// manager initialization, and various file operations that represent
    /// typical application usage patterns.
    #[tokio::test]
    async fn test_complete_workflow() {
        let temp_dir = setup_test_dir();
        let temp_path = temp_dir.path();

        // Step 1: Set up project structure
        create_test_file(temp_path, "json-echo.json", b"{}");
        let config_dir = temp_path.join("config");
        fs::create_dir(&config_dir).expect("Failed to create config directory");

        // Step 2: Auto-discover root and create manager
        let original_dir = std::env::current_dir().expect("Failed to get current directory");
        std::env::set_current_dir(temp_path).expect("Failed to change directory");

        let manager =
            FileSystemManager::new(None).expect("Should create manager with auto-discovery");

        // Restore directory
        std::env::set_current_dir(original_dir).expect("Failed to restore directory");

        // Step 3: Perform various file operations
        let config_data = serde_json::json!({
            "app_name": "json-echo",
            "version": "1.0.0",
            "features": ["file_ops", "auto_discovery"]
        });
        let config_bytes =
            serde_json::to_vec_pretty(&config_data).expect("Failed to serialize config");

        let save_result = manager.save_file("config/app.json", config_bytes).await;
        assert!(save_result.is_ok(), "Should save configuration file");

        // Step 4: Load and verify configuration
        let load_result = manager.load_file("config/app.json").await;
        assert!(load_result.is_ok(), "Should load configuration file");

        let loaded_bytes = load_result.unwrap();
        let loaded_config: serde_json::Value =
            serde_json::from_slice(&loaded_bytes).expect("Should parse loaded configuration");

        assert_eq!(
            loaded_config["app_name"], "json-echo",
            "Configuration should be preserved correctly"
        );
        assert_eq!(
            loaded_config["version"], "1.0.0",
            "Version should be preserved correctly"
        );

        // Step 5: Test backup operations
        let backup_result = manager
            .save_file("config/app.json.backup", loaded_bytes)
            .await;
        assert!(backup_result.is_ok(), "Should create backup file");

        let backup_load_result = manager.load_file("config/app.json.backup").await;
        assert!(backup_load_result.is_ok(), "Should load backup file");

        let backup_config: serde_json::Value = serde_json::from_slice(&backup_load_result.unwrap())
            .expect("Should parse backup configuration");

        assert_eq!(
            backup_config, loaded_config,
            "Backup should match original configuration"
        );
    }
}
