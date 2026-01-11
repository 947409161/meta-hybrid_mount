pub mod overlayfs;
pub mod utils;

use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use anyhow::{Result, bail};

use crate::defs;

pub fn mount_systemlessly(module_id: HashSet<String>, extra_partitions: &[String]) -> Result<()> {
    // construct overlay mount params
    let module_dir = Path::new(defs::MODULES_DIR);
    let dir = module_dir.read_dir();
    let Ok(dir) = dir else {
        bail!("open {} failed", defs::MODULES_DIR);
    };

    let mut system_lowerdir: Vec<String> = Vec::new();

    let partition = vec!["vendor", "product", "system_ext", "odm", "oem"];
    let mut partition_lowerdir: HashMap<String, Vec<String>> = HashMap::new();
    for ele in &partition {
        partition_lowerdir.insert((*ele).to_string(), Vec::new());
    }
    for p in extra_partitions {
        partition_lowerdir.insert(p.clone(), Vec::new());
    }

    for entry in dir.flatten() {
        let module = entry.path();
        if !module.is_dir() {
            continue;
        }
        if let Some(module_name) = module.file_name() {
            let real_module_path = module_dir.join(module_name);

            let disabled = real_module_path.join(defs::DISABLE_FILE_NAME).exists();

            if disabled {
                tracing::info!("module: {} is disabled, ignore!", module.display());
                continue;
            }
            if !module_id.contains(&module_name.to_string_lossy().to_string()) {
                continue;
            }
        }

        let skip_mount = module.join(defs::SKIP_MOUNT_FILE_NAME).exists();
        if skip_mount {
            tracing::info!("module: {} skip_mount exist, skip!", module.display());
            continue;
        }

        let module_system = Path::new(&module).join("system");
        if module_system.is_dir() {
            system_lowerdir.push(format!("{}", module_system.display()));
        }

        for part in &partition {
            // if /partition is a mountpoint, we would move it to $MODPATH/$partition when install
            // otherwise it must be a symlink and we don't need to overlay!
            let part_path = Path::new(&module).join(part);
            if part_path.is_dir() {
                if let Some(v) = partition_lowerdir.get_mut(*part) {
                    v.push(format!("{}", part_path.display()));
                }
            }
        }
    }

    // mount /system first
    if let Err(e) = mount_partition("system", &system_lowerdir) {
        tracing::warn!("mount system failed: {:#}", e);
    }

    // mount other partitions
    for (k, v) in partition_lowerdir {
        if let Err(e) = mount_partition(k.clone(), &v) {
            tracing::warn!("mount {k} failed: {:#}", e);
        }
    }

    Ok(())
}

fn mount_partition<S>(partition_name: S, lowerdir: &Vec<String>) -> Result<()>
where
    S: AsRef<str>,
{
    let partition_name = partition_name.as_ref();
    if lowerdir.is_empty() {
        tracing::warn!("partition: {partition_name} lowerdir is empty");
        return Ok(());
    }

    let partition = format!("/{partition_name}");

    // if /partition is a symlink and linked to /system/partition, then we don't need to overlay it separately
    if Path::new(&partition).read_link().is_ok() {
        tracing::warn!("partition: {partition} is a symlink");
        return Ok(());
    }

    let mut workdir = None;
    let mut upperdir = None;
    let system_rw_dir = Path::new(defs::SYSTEM_RW_DIR);
    if system_rw_dir.exists() {
        workdir = Some(system_rw_dir.join(partition_name).join("workdir"));
        upperdir = Some(system_rw_dir.join(partition_name).join("upperdir"));
    }

    overlayfs::mount_overlay(&partition, lowerdir, workdir, upperdir)
}
