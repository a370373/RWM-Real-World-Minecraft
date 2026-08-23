use std::fs;
use std::path::PathBuf;

fn repository_root() -> PathBuf {
    let relative = PathBuf::from("assets/3dmodels");

    if relative.is_dir() {
        return relative;
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join(&relative);

            if candidate.is_dir() {
                return candidate;
            }

            if let Some(project_root) = parent.parent() {
                let candidate = project_root.join(&relative);

                if candidate.is_dir() {
                    return candidate;
                }
            }
        }
    }

    relative
}

/// Read an RWM bundled 3D archetype entirely from the local repository.
///
/// Runtime only reads bundled local model assets.
pub(super) fn fetch_glb(filename: &str) -> Result<Vec<u8>, String> {
    let path = repository_root().join(filename);

    fs::read(&path).map_err(|e| format!("Local RWM 3D model read failed ({}): {e}", path.display()))
}
