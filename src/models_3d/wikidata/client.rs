//! RWM Wikidata model resolver.
//!
//! Resolution order:
//!
//! 1. Bundled RWM model
//! 2. Local runtime cache
//! 3. Network fallback from the model URL
//!
//! Wikidata/index data tells RWM which model belongs to a QID.
//! This module is responsible only for resolving that model locally,
//! with network fallback when the local copy is missing.

use fnv::FnvHasher;
use reqwest::blocking::{Client, ClientBuilder};
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;

const BUNDLED_SUBDIR: &str = "assets/3dmodels/wikidata";
const CACHE_SUBDIR: &str = "rwm/wikidata_models";

const MAX_MODEL_BYTES: u64 = 128 * 1024 * 1024;
const REQUEST_TIMEOUT_SECS: u64 = 30;

/// Locate the RWM repository root / runtime root.
fn repository_root() -> PathBuf {
    let relative = PathBuf::from(BUNDLED_SUBDIR);

    if relative.is_dir() {
        return PathBuf::from(".");
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join(&relative);

            if candidate.is_dir() {
                return parent.to_path_buf();
            }

            if let Some(project_root) = parent.parent() {
                let candidate = project_root.join(&relative);

                if candidate.is_dir() {
                    return project_root.to_path_buf();
                }
            }
        }
    }

    PathBuf::from(".")
}

/// Bundled RWM model directory.
pub(crate) fn bundled_root() -> PathBuf {
    repository_root().join(BUNDLED_SUBDIR)
}

/// Runtime cache directory.
pub(crate) fn cache_root() -> PathBuf {
    if let Some(dir) = dirs::cache_dir() {
        dir.join(CACHE_SUBDIR)
    } else {
        PathBuf::from("./.rwm_wikidata_cache")
    }
}

fn build_client() -> Result<Client, String> {
    ClientBuilder::new()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .user_agent(concat!("RWM/", env!("CARGO_PKG_VERSION"), " (+RWM)"))
        .build()
        .map_err(|e| e.to_string())
}

fn read_capped(mut resp: reqwest::blocking::Response, cap: u64) -> Result<Vec<u8>, String> {
    if let Some(len) = resp.content_length() {
        if len > cap {
            return Err(format!("model exceeds {cap}-byte cap (advertised {len})"));
        }
    }

    let mut buf = Vec::new();

    let mut taken = (&mut resp).take(cap + 1);

    taken.read_to_end(&mut buf).map_err(|e| e.to_string())?;

    if buf.len() as u64 > cap {
        return Err(format!("model exceeds {cap}-byte cap"));
    }

    Ok(buf)
}

fn url_hash(url: &str) -> String {
    let mut h = FnvHasher::default();
    url.hash(&mut h);
    format!("{:016x}", h.finish())
}

fn cache_path(url: &str) -> PathBuf {
    cache_root().join(format!("{}.bin", url_hash(url)))
}

/// Try to read a bundled model by filename.
///
/// This is checked before the runtime cache and before any network request.
pub fn read_bundled(filename: &str) -> Result<Option<Vec<u8>>, String> {
    let path = bundled_root().join(filename);

    match fs::read(&path) {
        Ok(bytes) => {
            if bytes.len() < 12 {
                return Err(format!(
                    "bundled Wikidata model is invalid or too small: {}",
                    path.display()
                ));
            }

            Ok(Some(bytes))
        }

        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),

        Err(e) => Err(format!(
            "failed reading bundled Wikidata model {}: {e}",
            path.display()
        )),
    }
}

/// Try to read a model from the RWM runtime cache.
fn read_cache(url: &str) -> Result<Option<Vec<u8>>, String> {
    let path = cache_path(url);

    match fs::read(&path) {
        Ok(bytes) => {
            if bytes.len() < 12 {
                let _ = fs::remove_file(&path);
                return Ok(None);
            }

            Ok(Some(bytes))
        }

        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),

        Err(e) => Err(format!(
            "failed reading Wikidata model cache {}: {e}",
            path.display()
        )),
    }
}

/// Download a missing Wikidata model and put it into the RWM cache.
fn download_to_cache(url: &str) -> Result<Vec<u8>, String> {
    eprintln!("  Wikidata model missing locally; downloading from {}", url);

    let client = build_client()?;

    let resp = client
        .get(url)
        .send()
        .map_err(|e| format!("Wikidata model download failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!(
            "Wikidata model download failed: HTTP {} ({})",
            resp.status(),
            url
        ));
    }

    let bytes = read_capped(resp, MAX_MODEL_BYTES)?;

    if bytes.len() < 12 {
        return Err(format!(
            "downloaded Wikidata model is invalid or too small: {}",
            url
        ));
    }

    let dir = cache_root();

    fs::create_dir_all(&dir).map_err(|e| format!("failed creating Wikidata model cache: {e}"))?;

    let path = cache_path(url);

    fs::write(&path, &bytes).map_err(|e| format!("failed writing Wikidata model cache: {e}"))?;

    Ok(bytes)
}

/// Resolve a Wikidata model.
///
/// Resolution order:
///
/// bundled model → runtime cache → network fallback
///
/// `bundled_filename` is optional. When present, RWM first checks
/// `assets/3dmodels/wikidata/<bundled_filename>`.
pub fn fetch_model(url: &str, bundled_filename: Option<&str>) -> Result<Vec<u8>, String> {
    if let Some(filename) = bundled_filename {
        if let Some(bytes) = read_bundled(filename)? {
            return Ok(bytes);
        }
    }

    if let Some(bytes) = read_cache(url)? {
        return Ok(bytes);
    }

    download_to_cache(url)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn url_hash_is_stable() {
        let a = url_hash("https://commons.wikimedia.org/wiki/Special:FilePath/X.stl");

        let b = url_hash("https://commons.wikimedia.org/wiki/Special:FilePath/X.stl");

        assert_eq!(a, b);
        assert_eq!(a.len(), 16);
    }

    #[test]
    fn url_hash_differs_for_different_urls() {
        let a = url_hash("https://commons.wikimedia.org/wiki/Special:FilePath/A.stl");

        let b = url_hash("https://commons.wikimedia.org/wiki/Special:FilePath/B.stl");

        assert_ne!(a, b);
    }

    #[test]
    fn cache_path_is_inside_cache_root() {
        let url = "https://commons.wikimedia.org/wiki/Special:FilePath/X.stl";
        let path = cache_path(url);

        assert!(path.starts_with(cache_root()));
    }

    #[test]
    fn bundled_root_is_defined() {
        let path = bundled_root();
        assert!(path.ends_with(Path::new(BUNDLED_SUBDIR)));
    }
}
