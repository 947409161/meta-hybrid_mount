#[cfg(any(target_os = "linux", target_os = "android"))]
use std::fs;
#[cfg(any(target_os = "linux", target_os = "android"))]
use std::{os::unix::fs::PermissionsExt, path::Path, process::Command};

#[cfg(any(target_os = "linux", target_os = "android"))]
use anyhow::{Context, Result, anyhow};
#[cfg(any(target_os = "linux", target_os = "android"))]
use rustix::mount::{UnmountFlags, unmount};

pub struct AutoMountExt4 {
    target: String,
    auto_umount: bool,
}

impl AutoMountExt4 {
    #[cfg(any(target_os = "linux", target_os = "android"))]
    pub fn try_new(source: &str, target: &str, auto_umount: bool) -> Result<Self> {
        let path = Path::new(source);
        if !path.exists() {
            println!("Source path does not exist");
        } else {
            let metadata = fs::metadata(path)?;
            let permissions = metadata.permissions();
            let mode = permissions.mode();

            if permissions.readonly() {
                println!("File permissions: {:o} (octal)", mode & 0o777);
            }
        }

        mount_ext4(source, target)?;
        Ok(Self {
            target: target.to_string(),
            auto_umount,
        })
    }

    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    pub fn try_new(_src: &str, _mnt: &str, _auto_umount: bool) -> Result<Self> {
        unimplemented!()
    }

    #[cfg(any(target_os = "linux", target_os = "android"))]
    pub fn umount(&self) -> Result<()> {
        unmount(self.target.as_str(), UnmountFlags::DETACH)?;
        Ok(())
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Drop for AutoMountExt4 {
    fn drop(&mut self) {
        tracing::info!(
            "AutoMountExt4 drop: {}, auto_umount: {}",
            self.target,
            self.auto_umount
        );
        if self.auto_umount {
            let _ = self.umount();
        }
    }
}

#[allow(dead_code)]
#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn mount_image(src: &str, target: &str, _autodrop: bool) -> Result<()> {
    mount_ext4(src, target)?;
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn mount_ext4(source: impl AsRef<Path>, target: impl AsRef<Path>) -> Result<()> {
    // Fallback to std::process::Command to avoid 'loopdev' dependency issues
    let status = Command::new("mount")
        .args(["-t", "ext4", "-o", "loop,rw,noatime"])
        .arg(source.as_ref())
        .arg(target.as_ref())
        .status()
        .context("Failed to execute mount command")?;

    if !status.success() {
        return Err(anyhow!("Mount command failed"));
    }
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn umount_dir(src: impl AsRef<Path>) -> Result<()> {
    unmount(src.as_ref(), UnmountFlags::empty())
        .with_context(|| format!("Failed to umount {}", src.as_ref().display()))?;
    Ok(())
}
