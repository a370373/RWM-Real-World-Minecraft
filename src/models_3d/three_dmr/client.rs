use reqwest::blocking::{Client, ClientBuilder};
use serde::{Deserialize, Deserializer};
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;

const API_BASE: &str = "https://3dmr.eu/api";
const CACHE_SUBDIR: &str = "rwm/3dmr";
const MAX_GLB_BYTES: u64 = 64 * 1024 * 1024;
const REQUEST_TIMEOUT_SECS: u64 = 20;

#[derive(Debug, Deserialize, Clone)]
pub struct ModelInfo {
    #[allow(dead_code)]
    pub id: u64,
    pub title: Option<String>,
    pub author: Option<String>,
    #[allow(dead_code)]
    pub lat: Option<f64>,
    #[allow(dead_code)]
    pub lon: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_license")]
    pub license: Option<String>,

    #[serde(default)]
    pub rotation: f64,

    #[serde(default = "default_scale")]
    pub scale: f64,

    #[serde(default)]
    pub translation: [f64; 3],
}

fn default_scale() -> f64 {
    1.0
}

fn deserialize_license<'de, D: Deserializer<'de>>(de: D) -> Result<Option<String>, D::Error> {
    let v = serde_json::Value::deserialize(de)?;

    Ok(match v {
        serde_json::Value::Null => None,
        serde_json::Value::String(s) => Some(s),
        serde_json::Value::Number(n) => Some(n.to_string()),
        other => Some(other.to_string()),
    })
}

/// Locate the RWM bundled 3DMR repository.
fn bundled_root() -> PathBuf {
    let relative = PathBuf::from("assets/3dmodels/3dmr");

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

/// User-level persistent cache used only when the bundled repository
/// does not contain the requested model.
pub(crate) fn cache_root() -> PathBuf {
    if let Some(dir) = dirs::cache_dir() {
        dir.join(CACHE_SUBDIR)
    } else {
        PathBuf::from("./.rwm_3dmr_cache")
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

/// Load metadata from the bundled RWM repository first,
/// then user cache, and finally 3DMR over HTTP.
pub fn fetch_info(id: u64) -> Result<ModelInfo, String> {
    // 1. Bundled RWM metadata.
    let bundled_path = bundled_root().join("info.json");

    if let Ok(bytes) = fs::read(&bundled_path) {
        if let Ok(models) =
            serde_json::from_slice::<std::collections::HashMap<String, ModelInfo>>(&bytes)
        {
            if let Some(info) = models.get(&id.to_string()) {
                println!("  3DMR info {id}: using bundled RWM data");
                return Ok(info.clone());
            }
        }
    }

    // 2. User cache.
    let dir = cache_root();
    let cache_path = dir.join(format!("{id}.json"));

    if let Ok(bytes) = fs::read(&cache_path) {
        if let Ok(info) = serde_json::from_slice::<ModelInfo>(&bytes) {
            println!("  3DMR info {id}: using local cache");
            return Ok(info);
        }
    }

    // 3. Network fallback.
    println!("  3DMR info {id}: not found locally; downloading...");

    let client = build_client()?;
    let url = format!("{API_BASE}/info/{id}");

    let resp = client.get(&url).send().map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("3DMR info {id}: HTTP {}", resp.status()));
    }

    let bytes = resp.bytes().map_err(|e| e.to_string())?;

    let info: ModelInfo =
        serde_json::from_slice(&bytes).map_err(|e| format!("3DMR info {id} parse: {e}"))?;

    let _ = fs::create_dir_all(&dir);
    let _ = fs::write(&cache_path, &bytes);

    Ok(info)
}

/// Load the GLB from the bundled RWM repository first,
/// then user cache, and finally 3DMR over HTTP.
pub fn fetch_glb(id: u64) -> Result<Vec<u8>, String> {
    // 1. Bundled RWM model.
    let bundled_path = bundled_root().join("models").join(format!("{id}.glb"));

    if let Ok(bytes) = fs::read(&bundled_path) {
        if !bytes.is_empty() {
            println!("  3DMR model {id}: using bundled RWM model");
            return Ok(bytes);
        }
    }

    // 2. User cache.
    let dir = cache_root();
    let cache_path = dir.join(format!("{id}.glb"));

    if let Ok(bytes) = fs::read(&cache_path) {
        if !bytes.is_empty() {
            println!("  3DMR model {id}: using local cache");
            return Ok(bytes);
        }
    }

    // 3. Network fallback.
    println!("  3DMR model {id}: not found locally; downloading...");

    let client = build_client()?;
    let url = format!("{API_BASE}/model/{id}");

    let resp = client.get(&url).send().map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("3DMR model {id}: HTTP {}", resp.status()));
    }

    let bytes = read_capped(resp, MAX_GLB_BYTES)?;

    let _ = fs::create_dir_all(&dir);
    let _ = fs::write(&cache_path, &bytes);

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_license_as_integer() {
        let json =
            r#"{"id": 1, "title": "x", "author": "a", "lat": null, "lon": null, "license": 0}"#;

        let info: ModelInfo = serde_json::from_str(json).unwrap();

        assert_eq!(info.license.as_deref(), Some("0"));
    }

    #[test]
    fn parses_license_as_string() {
        let json = r#"{"id": 1, "license": "CC-BY-SA"}"#;

        let info: ModelInfo = serde_json::from_str(json).unwrap();

        assert_eq!(info.license.as_deref(), Some("CC-BY-SA"));
    }

    #[test]
    fn parses_missing_license() {
        let json = r#"{"id": 1}"#;

        let info: ModelInfo = serde_json::from_str(json).unwrap();

        assert!(info.license.is_none());
    }
}
