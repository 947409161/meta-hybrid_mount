// Copyright 2025 Meta-Hybrid Mount Authors
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    collections::HashSet,
    path::Path,
    sync::{LazyLock, Mutex, OnceLock},
};

use anyhow::Result;
use ksu::{NukeExt4Sysfs, TryUmount};

pub static TMPFS: OnceLock<String> = OnceLock::new();
pub static LIST: LazyLock<Mutex<TryUmount>> = LazyLock::new(|| Mutex::new(TryUmount::new()));
static HISTORY: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

pub fn send_unmountable<P>(target: P) -> Result<()>
where
    P: AsRef<Path>,
{
    if !crate::utils::KSU.load(std::sync::atomic::Ordering::Relaxed) {
        return Ok(());
    }

    let path_str = target.as_ref().to_string_lossy().to_string();
    let mut history = HISTORY.lock().unwrap();

    if history.contains(&path_str) {
        tracing::debug!("Ignored duplicate unmount request: {}", path_str);
        return Ok(());
    }

    history.insert(path_str);
    LIST.lock().unwrap().add(target);
    Ok(())
}

pub fn commit() -> Result<()> {
    if !crate::utils::KSU.load(std::sync::atomic::Ordering::Relaxed) {
        return Ok(());
    }
    let mut list = LIST
        .lock()
        .map_err(|_| anyhow::anyhow!("Failed to lock unmount list"))?;

    // Attempt 1: Normal unmount (0)
    list.flags(0);
    if let Err(e0) = list.umount() {
        tracing::debug!("try_umount(0) failed: {:#}, retrying with flags(2)", e0);

        // Attempt 2: Detach/Lazy unmount (2)
        list.flags(2);
        if let Err(e2) = list.umount() {
            tracing::warn!("try_umount(2) failed: {:#}", e2);
        }
    }

    Ok(())
}

pub fn ksu_nuke_sysfs(target: &str) -> Result<()> {
    if !crate::utils::KSU.load(std::sync::atomic::Ordering::Relaxed) {
        return Ok(());
    }
    NukeExt4Sysfs::new().add(target).execute()?;
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "android")))]
pub fn ksu_nuke_sysfs(_target: &str) -> Result<()> {
    bail!("Not supported on this OS")
}
