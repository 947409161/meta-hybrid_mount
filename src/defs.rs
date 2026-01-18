// Copyright 2025 Meta-Hybrid Mount Authors
// SPDX-License-Identifier: GPL-3.0-or-later

pub const DEFAULT_HYBRID_MNT_DIR: &str = "/debug_ramdisk";
pub const MODULES_IMG_FILE: &str = "/data/adb/meta-hybrid/modules.img";
pub const RUN_DIR: &str = "/data/adb/meta-hybrid/run/";
pub const STATE_FILE: &str = "/data/adb/meta-hybrid/run/daemon_state.json";
pub const DISABLE_FILE_NAME: &str = "disable";
pub const REMOVE_FILE_NAME: &str = "remove";
pub const SKIP_MOUNT_FILE_NAME: &str = "skip_mount";
pub const SYSTEM_RW_DIR: &str = "/data/adb/meta-hybrid/rw";
pub const MODULE_PROP_FILE: &str = "/data/adb/modules/meta-hybrid/module.prop";
pub const MODULES_DIR: &str = "/data/adb/modules";
pub const BUILTIN_PARTITIONS: &[&str] =
    &["system", "vendor", "product", "system_ext", "odm", "oem"];
pub const REPLACE_DIR_FILE_NAME: &str = ".replace";
pub const REPLACE_DIR_XATTR: &str = "trusted.overlay.opaque";
pub const RECOVERY_COUNTER_FILE: &str = "/data/adb/meta-hybrid/boot_counter";
pub const RECOVERY_RESCUE_NOTICE: &str = "/data/adb/meta-hybrid/rescue_notice";
pub const BACKUP_DIR: &str = "/data/adb/meta-hybrid/backups";
