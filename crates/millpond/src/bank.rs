//! The raw bank on this host: each capture's raw files under
//! `<bank>/<key>/`, byte for byte, with a cut excerpt's whole text beside
//! them as `_excerpt` (14 to 16, 19, 83 to 85; C.3). millpond never deletes
//! what a taken capture names (103); `discard` removes only what was
//! banked for a capture that was not written.

use std::fs;
use std::io;
use std::path::PathBuf;

use weir_capture::{Bank, Key, RawFile, Refusal};

/// The file a cut excerpt's whole text is banked in, beside the raw files.
pub const EXCERPT: &str = "_excerpt";

pub struct FolderBank {
    pub root: PathBuf,
}

impl Bank for FolderBank {
    fn bank(
        &mut self,
        key: &Key,
        files: &[RawFile],
        whole_excerpt: Option<&str>,
    ) -> Result<String, Refusal> {
        let dir = self.root.join(key.as_str());
        let fail = |error: io::Error| Refusal(format!("{}: {error}", dir.display()));
        fs::create_dir_all(&dir).map_err(fail)?;
        for file in files {
            let path = dir.join(&file.path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(fail)?;
            }
            fs::write(&path, &file.bytes).map_err(fail)?;
        }
        if let Some(text) = whole_excerpt {
            fs::write(dir.join(EXCERPT), text).map_err(fail)?;
        }
        Ok(dir.to_string_lossy().into_owned())
    }

    fn discard(&mut self, key: &Key) {
        let _ = fs::remove_dir_all(self.root.join(key.as_str()));
    }
}
