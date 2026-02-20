#![allow(dead_code)]

use std::os::fd::{FromRawFd, OwnedFd};

use anyhow::{Result, bail};
use nix::{ioctl_none, ioctl_read, ioctl_readwrite, ioctl_write_ptr};

pub const HYMO_MAGIC1: libc::c_ulong = 0x48594D4F;
pub const HYMO_MAGIC2: libc::c_ulong = 0x524F4F54;
pub const HYMO_PROTOCOL_VERSION: i32 = 12;
pub const HYMO_MAX_LEN_PATHNAME: usize = 256;
pub const HYMO_FAKE_CMDLINE_SIZE: usize = 4096;
pub const HYMO_CMD_GET_FD: libc::c_ulong = 0x48021;
pub const HYMO_PRCTL_GET_FD: libc::c_int = 0x48021;

pub const HYMO_IOC_MAGIC: u8 = b'H';

#[repr(C)]
#[derive(Debug, Clone)]
pub struct HymoSyscallArg {
    pub src: *const libc::c_char,
    pub target: *const libc::c_char,
    pub type_: libc::c_int,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct HymoSyscallListArg {
    pub buf: *mut libc::c_char,
    pub size: libc::size_t,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct HymoUidListArg {
    pub count: u32,
    pub reserved: u32,
    pub uids: u64,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct HymoSpoofKstat {
    pub target_ino: libc::c_ulong,
    pub target_pathname: [libc::c_char; HYMO_MAX_LEN_PATHNAME],
    pub spoofed_ino: libc::c_ulong,
    pub spoofed_dev: libc::c_ulong,
    pub spoofed_nlink: libc::c_uint,
    pub spoofed_size: libc::c_longlong,
    pub spoofed_atime_sec: libc::c_long,
    pub spoofed_atime_nsec: libc::c_long,
    pub spoofed_mtime_sec: libc::c_long,
    pub spoofed_mtime_nsec: libc::c_long,
    pub spoofed_ctime_sec: libc::c_long,
    pub spoofed_ctime_nsec: libc::c_long,
    pub spoofed_blksize: libc::c_ulong,
    pub spoofed_blocks: libc::c_ulonglong,
    pub is_static: libc::c_int,
    pub err: libc::c_int,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct HymoSpoofUname {
    pub sysname: [libc::c_char; 65],
    pub nodename: [libc::c_char; 65],
    pub release: [libc::c_char; 65],
    pub version: [libc::c_char; 65],
    pub machine: [libc::c_char; 65],
    pub domainname: [libc::c_char; 65],
    pub err: libc::c_int,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct HymoSpoofCmdline {
    pub cmdline: [libc::c_char; HYMO_FAKE_CMDLINE_SIZE],
    pub err: libc::c_int,
}

ioctl_write_ptr!(hymo_ioc_add_rule, HYMO_IOC_MAGIC, 1, HymoSyscallArg);
ioctl_write_ptr!(hymo_ioc_del_rule, HYMO_IOC_MAGIC, 2, HymoSyscallArg);
ioctl_write_ptr!(hymo_ioc_hide_rule, HYMO_IOC_MAGIC, 3, HymoSyscallArg);
ioctl_none!(hymo_ioc_clear_all, HYMO_IOC_MAGIC, 5);
ioctl_read!(hymo_ioc_get_version, HYMO_IOC_MAGIC, 6, libc::c_int);
ioctl_readwrite!(hymo_ioc_list_rules, HYMO_IOC_MAGIC, 7, HymoSyscallListArg);
ioctl_write_ptr!(hymo_ioc_set_debug, HYMO_IOC_MAGIC, 8, libc::c_int);
ioctl_none!(hymo_ioc_reorder_mnt_id, HYMO_IOC_MAGIC, 9);
ioctl_write_ptr!(hymo_ioc_set_stealth, HYMO_IOC_MAGIC, 10, libc::c_int);
ioctl_write_ptr!(
    hymo_ioc_hide_overlay_xattrs,
    HYMO_IOC_MAGIC,
    11,
    HymoSyscallArg
);
ioctl_write_ptr!(hymo_ioc_add_merge_rule, HYMO_IOC_MAGIC, 12, HymoSyscallArg);
ioctl_write_ptr!(hymo_ioc_set_mirror_path, HYMO_IOC_MAGIC, 14, HymoSyscallArg);
ioctl_write_ptr!(hymo_ioc_add_spoof_kstat, HYMO_IOC_MAGIC, 15, HymoSpoofKstat);
ioctl_write_ptr!(
    hymo_ioc_update_spoof_kstat,
    HYMO_IOC_MAGIC,
    16,
    HymoSpoofKstat
);
ioctl_write_ptr!(hymo_ioc_set_uname, HYMO_IOC_MAGIC, 17, HymoSpoofUname);
ioctl_write_ptr!(hymo_ioc_set_cmdline, HYMO_IOC_MAGIC, 18, HymoSpoofCmdline);
ioctl_read!(hymo_ioc_get_features, HYMO_IOC_MAGIC, 19, libc::c_int);
ioctl_write_ptr!(hymo_ioc_set_enabled, HYMO_IOC_MAGIC, 20, libc::c_int);
ioctl_write_ptr!(hymo_ioc_set_hide_uids, HYMO_IOC_MAGIC, 21, HymoUidListArg);

pub fn get_hymofs_fd(syscall_nr: libc::c_long) -> Result<OwnedFd> {
    let mut fd: libc::c_int = -1;
    unsafe {
        let ret = libc::syscall(syscall_nr, HYMO_MAGIC1, HYMO_MAGIC2, HYMO_CMD_GET_FD);
        if ret >= 0 {
            fd = ret as libc::c_int;
        } else {
            libc::prctl(
                HYMO_PRCTL_GET_FD,
                &mut fd as *mut libc::c_int as libc::c_ulong,
                0,
                0,
                0,
            );
        }
    }

    if fd >= 0 {
        Ok(unsafe { OwnedFd::from_raw_fd(fd) })
    } else {
        bail!("Failed to get HymoFS file descriptor")
    }
}
