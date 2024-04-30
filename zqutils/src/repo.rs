use anyhow::Result;
use std::path::Path;
use std::path::PathBuf;

/// Find the lowest directory above this one containing a .git directory and return its absolute path.
pub fn find_enclosing_git_repo_base_from_string(start_at: &str) -> Result<Option<Box<PathBuf>>> {
    let here = std::fs::canonicalize(start_at)?;
    find_enclosing_git_repo_base(&here)
}

pub fn find_enclosing_git_repo_base(path: &Path) -> Result<Option<Box<PathBuf>>> {
    let mut scan_path = PathBuf::new();
    let root_path = PathBuf::new();
    scan_path.push(path);
    while scan_path != root_path {
        let mut git_path = PathBuf::new();
        git_path.push(scan_path.clone());
        git_path.push(".git");
        if git_path.is_dir() {
            // Found it!
            return Ok(Some(Box::new(scan_path)));
        }
        // Otherwise
        scan_path.pop();
    }
    Ok(None)
}
