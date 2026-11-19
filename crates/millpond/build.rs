//! What the binary says of itself: the revision it was built from and the
//! build that made it. The flake gives the revision as MILLPOND_REVISION,
//! since a nix build has no `.git`; a cargo build in a checkout reads
//! git's HEAD, and runs again whenever HEAD or the branch it names moves.

use std::path::Path;
use std::process::Command;

#[path = "build/revision.rs"]
mod revision;

fn main() {
    let checkout = Path::new(".");
    let revision = std::env::var("MILLPOND_REVISION")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| revision::head(checkout))
        .unwrap_or_else(|| "unknown".to_string());
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let compiler = Command::new(&rustc)
        .arg("-V")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "rustc".to_string());
    let target = std::env::var("TARGET").unwrap_or_default();
    println!("cargo:rustc-env=MILLPOND_REVISION={revision}");
    println!("cargo:rustc-env=MILLPOND_BUILD={compiler} for {target}");
    println!("cargo:rerun-if-env-changed=MILLPOND_REVISION");
    for path in revision::watched(checkout) {
        println!("cargo:rerun-if-changed={}", path.display());
    }
}
