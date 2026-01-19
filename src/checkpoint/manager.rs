//! Checkpoint manager for coordinating atomic saves and loads.
//!
//! This module provides the `CheckpointManager` which handles all checkpoint
//! file operations with atomic writes to prevent corruption.

use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use thiserror::Error;

use super::Checkpoint;

/// Default name for the Ralph state directory.
const RALPH_DIR_NAME: &str = ".ralph";

/// Default name for the checkpoint file.
const CHECKPOINT_FILE_NAME: &str = "checkpoint.json";

/// Errors that can occur during checkpoint operations.
#[derive(Error, Debug)]
pub enum CheckpointError {
    /// IO error during file operations.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Checkpoint version mismatch.
    #[error("Version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: u32, found: u32 },

    /// Checkpoint data validation failed.
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

/// Result type for checkpoint operations.
pub type CheckpointResult<T> = Result<T, CheckpointError>;

/// Manager for checkpoint file operations.
///
/// The `CheckpointManager` coordinates saving and loading of execution state
/// with atomic writes to prevent corruption from interrupted operations.
#[derive(Debug, Clone)]
pub struct CheckpointManager {
    /// Path to the checkpoint file.
    checkpoint_path: PathBuf,
}

impl CheckpointManager {
    /// Create a new `CheckpointManager` for the given base directory.
    ///
    /// Creates the `.ralph` directory if it does not exist.
    ///
    /// # Arguments
    ///
    /// * `base_dir` - The base directory where `.ralph` directory will be created.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created.
    pub fn new(base_dir: impl Into<PathBuf>) -> CheckpointResult<Self> {
        let base = base_dir.into();
        let ralph_dir = base.join(RALPH_DIR_NAME);

        // Create .ralph directory if it doesn't exist
        fs::create_dir_all(&ralph_dir)?;

        let checkpoint_path = ralph_dir.join(CHECKPOINT_FILE_NAME);

        Ok(Self { checkpoint_path })
    }

    /// Save a checkpoint atomically.
    ///
    /// Writes to a temporary file first, then renames to the final location
    /// to ensure atomicity.
    ///
    /// # Arguments
    ///
    /// * `checkpoint` - The checkpoint to save.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or file operations fail.
    pub fn save(&self, checkpoint: &Checkpoint) -> CheckpointResult<()> {
        let json = serde_json::to_string_pretty(checkpoint)?;

        // Create temp file in the same directory to ensure atomic rename
        let temp_path = self.checkpoint_path.with_extension("json.tmp");

        // Write to temp file
        let mut file = fs::File::create(&temp_path)?;
        file.write_all(json.as_bytes())?;
        file.sync_all()?;

        // Atomic rename
        fs::rename(&temp_path, &self.checkpoint_path)?;

        Ok(())
    }

    /// Load a checkpoint from disk.
    ///
    /// # Errors
    ///
    /// Returns `Ok(None)` if the checkpoint file does not exist.
    /// Returns an error if reading or deserialization fails.
    pub fn load(&self) -> CheckpointResult<Option<Checkpoint>> {
        match fs::read_to_string(&self.checkpoint_path) {
            Ok(content) => {
                let checkpoint: Checkpoint = serde_json::from_str(&content)?;
                Ok(Some(checkpoint))
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(CheckpointError::Io(e)),
        }
    }

    /// Clear the checkpoint by removing the file.
    ///
    /// # Errors
    ///
    /// Returns an error if file removal fails (except for file not found).
    pub fn clear(&self) -> CheckpointResult<()> {
        match fs::remove_file(&self.checkpoint_path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(CheckpointError::Io(e)),
        }
    }

    /// Check if a checkpoint file exists.
    pub fn exists(&self) -> bool {
        self.checkpoint_path.exists()
    }

    /// Verify that a checkpoint is valid.
    ///
    /// Checks:
    /// - Version is compatible with current version
    /// - Required data is present and valid
    ///
    /// # Arguments
    ///
    /// * `checkpoint` - The checkpoint to verify.
    ///
    /// # Errors
    ///
    /// Returns an error if verification fails.
    pub fn verify(&self, checkpoint: &Checkpoint) -> CheckpointResult<()> {
        // Check version compatibility
        if checkpoint.version > Checkpoint::CURRENT_VERSION {
            return Err(CheckpointError::VersionMismatch {
                expected: Checkpoint::CURRENT_VERSION,
                found: checkpoint.version,
            });
        }

        // Validate story checkpoint if present
        if let Some(ref story) = checkpoint.current_story {
            if story.story_id.is_empty() {
                return Err(CheckpointError::ValidationFailed(
                    "story_id cannot be empty".to_string(),
                ));
            }
            if story.iteration > story.max_iterations {
                return Err(CheckpointError::ValidationFailed(format!(
                    "iteration {} exceeds max_iterations {}",
                    story.iteration, story.max_iterations
                )));
            }
        }

        Ok(())
    }

    /// Get the path to the checkpoint file.
    pub fn checkpoint_path(&self) -> &PathBuf {
        &self.checkpoint_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checkpoint::{PauseReason, StoryCheckpoint};
    use tempfile::TempDir;

    fn create_test_checkpoint() -> Checkpoint {
        Checkpoint::new(
            Some(StoryCheckpoint::new("US-001", 2, 5)),
            PauseReason::RateLimited,
            vec!["src/main.rs".to_string()],
        )
    }

    #[test]
    fn test_new_creates_ralph_directory() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        let manager = CheckpointManager::new(base_path).unwrap();

        let ralph_dir = base_path.join(".ralph");
        assert!(ralph_dir.exists());
        assert!(ralph_dir.is_dir());
        assert_eq!(
            manager.checkpoint_path(),
            &ralph_dir.join("checkpoint.json")
        );
    }

    #[test]
    fn test_new_with_existing_directory() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Pre-create the directory
        fs::create_dir_all(base_path.join(".ralph")).unwrap();

        // Should not fail
        let manager = CheckpointManager::new(base_path).unwrap();
        assert!(manager.checkpoint_path().parent().unwrap().exists());
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        let checkpoint = create_test_checkpoint();
        manager.save(&checkpoint).unwrap();

        let loaded = manager.load().unwrap().expect("checkpoint should exist");

        assert_eq!(loaded.version, checkpoint.version);
        assert_eq!(loaded.pause_reason, checkpoint.pause_reason);
        assert_eq!(loaded.current_story, checkpoint.current_story);
        assert_eq!(loaded.uncommitted_files, checkpoint.uncommitted_files);
    }

    #[test]
    fn test_load_nonexistent_returns_none() {
        let temp_dir = TempDir::new().unwrap();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        let result = manager.load().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_clear_removes_file() {
        let temp_dir = TempDir::new().unwrap();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        let checkpoint = create_test_checkpoint();
        manager.save(&checkpoint).unwrap();
        assert!(manager.exists());

        manager.clear().unwrap();
        assert!(!manager.exists());
    }

    #[test]
    fn test_clear_nonexistent_succeeds() {
        let temp_dir = TempDir::new().unwrap();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        // Should not fail even if file doesn't exist
        manager.clear().unwrap();
    }

    #[test]
    fn test_exists() {
        let temp_dir = TempDir::new().unwrap();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        assert!(!manager.exists());

        let checkpoint = create_test_checkpoint();
        manager.save(&checkpoint).unwrap();

        assert!(manager.exists());
    }

    #[test]
    fn test_verify_valid_checkpoint() {
        let temp_dir = TempDir::new().unwrap();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        let checkpoint = create_test_checkpoint();
        manager.verify(&checkpoint).unwrap();
    }

    #[test]
    fn test_verify_future_version_fails() {
        let temp_dir = TempDir::new().unwrap();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        let mut checkpoint = create_test_checkpoint();
        checkpoint.version = Checkpoint::CURRENT_VERSION + 1;

        let result = manager.verify(&checkpoint);
        assert!(matches!(
            result,
            Err(CheckpointError::VersionMismatch { .. })
        ));
    }

    #[test]
    fn test_verify_empty_story_id_fails() {
        let temp_dir = TempDir::new().unwrap();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        let checkpoint = Checkpoint::new(
            Some(StoryCheckpoint::new("", 1, 5)),
            PauseReason::UserRequested,
            vec![],
        );

        let result = manager.verify(&checkpoint);
        assert!(matches!(result, Err(CheckpointError::ValidationFailed(_))));
    }

    #[test]
    fn test_verify_iteration_exceeds_max_fails() {
        let temp_dir = TempDir::new().unwrap();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        let checkpoint = Checkpoint::new(
            Some(StoryCheckpoint::new("US-001", 10, 5)),
            PauseReason::UserRequested,
            vec![],
        );

        let result = manager.verify(&checkpoint);
        assert!(matches!(result, Err(CheckpointError::ValidationFailed(_))));
    }

    #[test]
    fn test_verify_no_story_succeeds() {
        let temp_dir = TempDir::new().unwrap();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        let checkpoint = Checkpoint::new(None, PauseReason::Timeout, vec![]);

        manager.verify(&checkpoint).unwrap();
    }

    #[test]
    fn test_atomic_save_cleans_up_temp_file() {
        let temp_dir = TempDir::new().unwrap();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        let checkpoint = create_test_checkpoint();
        manager.save(&checkpoint).unwrap();

        // Temp file should not exist after successful save
        let temp_path = manager.checkpoint_path().with_extension("json.tmp");
        assert!(!temp_path.exists());
    }

    #[test]
    fn test_save_overwrites_existing() {
        let temp_dir = TempDir::new().unwrap();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        // Save first checkpoint
        let checkpoint1 = Checkpoint::new(
            Some(StoryCheckpoint::new("US-001", 1, 5)),
            PauseReason::RateLimited,
            vec![],
        );
        manager.save(&checkpoint1).unwrap();

        // Save second checkpoint
        let checkpoint2 = Checkpoint::new(
            Some(StoryCheckpoint::new("US-002", 3, 10)),
            PauseReason::Timeout,
            vec!["file.rs".to_string()],
        );
        manager.save(&checkpoint2).unwrap();

        // Load should return second checkpoint
        let loaded = manager.load().unwrap().unwrap();
        assert_eq!(loaded.current_story.as_ref().unwrap().story_id, "US-002");
        assert_eq!(loaded.pause_reason, PauseReason::Timeout);
    }

    #[test]
    fn test_load_invalid_json_returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let manager = CheckpointManager::new(temp_dir.path()).unwrap();

        // Write invalid JSON
        fs::write(manager.checkpoint_path(), "{ invalid json }").unwrap();

        let result = manager.load();
        assert!(matches!(result, Err(CheckpointError::Json(_))));
    }
}
