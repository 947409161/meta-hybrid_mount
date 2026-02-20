use std::{
    collections::HashSet,
    ffi::CString,
    fs::File,
    os::fd::{AsFd, AsRawFd},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use rustix::system::finit_module;
use walkdir::WalkDir;

use crate::{
    conf::config,
    mount::hymofs::ioctl::{
        HymoSyscallArg, get_hymofs_fd, hymo_ioc_add_merge_rule, hymo_ioc_add_rule,
        hymo_ioc_set_enabled,
    },
};

pub fn load_kernel_module() -> Result<()> {
    let ko_path = Path::new("/data/adb/modules/hybrid_mount/hymofs_lkm.ko");
    if !ko_path.exists() {
        return Ok(());
    }

    let file = File::open(ko_path)?;
    let args = CString::new("hymo_syscall_nr=142")?;

    finit_module(file.as_fd(), &args, 0);

    Ok(())
}

pub fn apply_hymofs_rules(
    ids: &HashSet<String>,
    _config: &config::Config,
    storage_root: &Path,
) -> Result<Vec<String>> {
    let fd = get_hymofs_fd(142).context("Failed to get hymofs fd")?;
    let raw_fd = fd.as_raw_fd();

    let mut applied_ids = Vec::new();

    for id in ids {
        let module_dir = storage_root.join(id);
        if !module_dir.exists() {
            continue;
        }

        let mut success = true;

        for entry in WalkDir::new(&module_dir).min_depth(1).into_iter().flatten() {
            let path = entry.path();
            let rel_path = match path.strip_prefix(&module_dir) {
                Ok(p) => p,
                Err(_) => continue,
            };
            let target_path = PathBuf::from("/").join(rel_path);

            let src_c = match CString::new(path.to_string_lossy().as_bytes()) {
                Ok(c) => c,
                Err(_) => {
                    success = false;
                    continue;
                }
            };
            let target_c = match CString::new(target_path.to_string_lossy().as_bytes()) {
                Ok(c) => c,
                Err(_) => {
                    success = false;
                    continue;
                }
            };

            let arg = HymoSyscallArg {
                src: src_c.as_ptr(),
                target: target_c.as_ptr(),
                type_: if path.is_dir() { 1 } else { 0 },
            };

            unsafe {
                if path.is_dir() {
                    if hymo_ioc_add_merge_rule(raw_fd, &arg).is_err() {
                        success = false;
                    }
                } else if hymo_ioc_add_rule(raw_fd, &arg).is_err() {
                    success = false;
                }
            }
        }

        if success {
            applied_ids.push(id.clone());
        }
    }

    unsafe {
        let _ = hymo_ioc_set_enabled(raw_fd, &1);
    }

    Ok(applied_ids)
}

