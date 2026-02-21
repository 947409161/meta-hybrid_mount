use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{defs, sys::fs::xattr};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct HymofsState {
    pub enabled: bool,
    pub loaded: bool,
    pub version: i32,
    pub active_features: Vec<String>,
    pub error_msg: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RuntimeState {
    pub timestamp: u64,
    pub pid: u32,
    #[serde(default)]
    pub abi: String,
    pub storage_mode: String,
    pub mount_point: PathBuf,
    pub overlay_modules: Vec<String>,
    pub magic_modules: Vec<String>,
    #[serde(default)]
    pub hymofs_modules: Vec<String>,
    #[serde(default)]
    pub active_mounts: Vec<String>,
    #[serde(default)]
    pub tmpfs_xattr_supported: bool,
    #[serde(default)]
    pub hymofs_state: HymofsState,
}

impl RuntimeState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        storage_mode: String,
        mount_point: PathBuf,
        overlay_modules: Vec<String>,
        magic_modules: Vec<String>,
        hymofs_modules: Vec<String>,
        active_mounts: Vec<String>,
        hymofs_state: HymofsState,
    ) -> Self {
        let start = SystemTime::now();

        let timestamp = start
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let pid = std::process::id();

        let abi = rustix::system::uname()
            .machine()
            .to_string_lossy()
            .into_owned();

        let tmpfs_xattr_supported = xattr::is_overlay_xattr_supported().unwrap_or(false);

        Self {
            timestamp,
            pid,
            abi,
            storage_mode,
            mount_point,
            overlay_modules,
            magic_modules,
            hymofs_modules,
            active_mounts,
            tmpfs_xattr_supported,
            hymofs_state,
        }
    }

    pub fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(defs::STATE_FILE, json)?;
        Ok(())
    }

    pub fn load() -> Result<Self> {
        if !std::path::Path::new(defs::STATE_FILE).exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(defs::STATE_FILE)?;
        let state = serde_json::from_str(&content)?;
        Ok(state)
    }
}
