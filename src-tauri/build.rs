fn main() {
    tauri_build::build();

    // Best-effort: embed git commit hash for About panel / audit.
    // This is local-only metadata and does not require network access.
    if let Ok(out) = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
    {
        if out.status.success() {
            let hash = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !hash.is_empty() {
                println!("cargo:rustc-env=GIT_COMMIT_HASH={hash}");
            }
        }
    }
}
