use crate::errors::{FileSystemError, FileSystemResult};
use std::{
    env::current_dir,
    path::{Path, PathBuf},
};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

#[derive(Debug, Clone)]
pub struct PathUtils;

impl PathUtils {
    pub fn current_dir() -> FileSystemResult<PathBuf> {
        current_dir().map_err(std::convert::Into::into)
    }

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

    pub fn normalize_path(path: &Path) -> PathBuf {
        path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
    }
}

#[derive(Debug, Clone)]
pub struct FileSystemManager {
    pub root: PathBuf,
}

impl FileSystemManager {
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

    pub async fn load_file(&self, relative_file_path: &str) -> FileSystemResult<Vec<u8>> {
        let file_path = self.root.as_path().join(relative_file_path);
        let mut file = File::open(file_path).await.map_err(FileSystemError::from)?;

        let mut buffer = vec![];

        file.read_to_end(&mut buffer)
            .await
            .map_err(FileSystemError::from)?;
        Ok(buffer)
    }

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
