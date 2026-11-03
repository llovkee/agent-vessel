use std::collections::HashMap;
use std::path::{Path, PathBuf};
use sha2::{Sha256, Digest};
use walkdir::WalkDir;

pub struct WorkspaceMonitor {
    workspace_root: PathBuf,
}
impl WorkspaceMonitor {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            workspace_root: root.as_ref().to_path_buf()
        }
    }

    pub fn snapshot(&self) -> Result<HashMap<PathBuf, String>, std::io::Error> {
        let mut state_map = HashMap::new();
        for entry in WalkDir::new(&self.workspace_root).into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.into_path();
            let hash = self.hash_file(&path)?;
            let rel_path = path.strip_prefix(&self.workspace_root).unwrap().to_path_buf();
            state_map.insert(rel_path, hash);
        }
        Ok(state_map)
    }
    fn hash_file(&self, path: &Path) -> Result<String, std::io::Error> {
        let bytes = std::fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let res = hasher.finalize();
        Ok(hex::encode(res))
    }
}