use crate::CliError;
use std::path::{Path, PathBuf};

/// A scratch directory for one build, removed when the build is over.
pub struct BuildDir(PathBuf);

impl BuildDir {
    pub fn create() -> Result<Self, CliError> {
        let dir = std::env::temp_dir().join(format!("roop-build-{}", std::process::id()));
        std::fs::create_dir_all(&dir).map_err(|e| CliError::Io("temp dir".into(), e))?;
        Ok(BuildDir(dir))
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for BuildDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
