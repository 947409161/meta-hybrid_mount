use std::{ffi::CString, os::fd::AsRawFd, path::Path};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::{
    conf::{
        cli::{Cli, HymofsAction},
        config::{self, Config},
    },
    core::{inventory, inventory::model as modules, ops::planner},
    defs,
    mount::hymofs::{
        driver::check_hymofs_status,
        ioctl::{
            HymoSyscallArg, HymoSyscallListArg, get_hymofs_fd, hymo_ioc_add_merge_rule,
            hymo_ioc_add_rule, hymo_ioc_clear_all, hymo_ioc_del_rule, hymo_ioc_hide_overlay_xattrs,
            hymo_ioc_hide_rule, hymo_ioc_list_rules, hymo_ioc_set_debug, hymo_ioc_set_enabled,
            hymo_ioc_set_stealth,
        },
    },
    utils,
};

#[derive(Serialize)]
struct DiagnosticIssueJson {
    level: String,
    context: String,
    message: String,
}

fn load_config(cli: &Cli) -> Result<Config> {
    if let Some(config_path) = &cli.config {
        return Config::from_file(config_path).with_context(|| {
            format!(
                "Failed to load config from custom path: {}",
                config_path.display()
            )
        });
    }

    match Config::load_default() {
        Ok(config) => Ok(config),
        Err(e) => {
            let is_not_found = e
                .root_cause()
                .downcast_ref::<std::io::Error>()
                .map(|io_err| io_err.kind() == std::io::ErrorKind::NotFound)
                .unwrap_or(false);

            if is_not_found {
                Ok(Config::default())
            } else {
                Err(e).context(format!(
                    "Failed to load default config from {}",
                    defs::CONFIG_FILE
                ))
            }
        }
    }
}

pub fn handle_gen_config(output: &Path) -> Result<()> {
    Config::default()
        .save_to_file(output)
        .with_context(|| format!("Failed to save generated config to {}", output.display()))
}

pub fn handle_show_config(cli: &Cli) -> Result<()> {
    let config = load_config(cli)?;

    let json = serde_json::to_string(&config).context("Failed to serialize config to JSON")?;

    println!("{}", json);

    Ok(())
}

pub fn handle_save_config(payload: &str) -> Result<()> {
    let json_bytes = (0..payload.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&payload[i..i + 2], 16))
        .collect::<Result<Vec<u8>, _>>()
        .context("Failed to decode hex payload")?;

    let config: Config =
        serde_json::from_slice(&json_bytes).context("Failed to parse config JSON payload")?;

    config
        .save_to_file(defs::CONFIG_FILE)
        .context("Failed to save config file")?;

    println!("Configuration saved successfully.");

    Ok(())
}

pub fn handle_save_module_rules(module_id: &str, payload: &str) -> Result<()> {
    utils::validate_module_id(module_id)?;
    let json_bytes = (0..payload.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&payload[i..i + 2], 16))
        .collect::<Result<Vec<u8>, _>>()
        .context("Failed to decode hex payload")?;

    let new_rules: config::ModuleRules =
        serde_json::from_slice(&json_bytes).context("Failed to parse module rules JSON")?;
    let mut config = Config::load_default().unwrap_or_default();

    config.rules.insert(module_id.to_string(), new_rules);

    config
        .save_to_file(defs::CONFIG_FILE)
        .context("Failed to update config file with new rules")?;

    println!("Module rules saved for {} into config.toml", module_id);

    Ok(())
}

pub fn handle_modules(cli: &Cli) -> Result<()> {
    let config = load_config(cli)?;

    modules::print_list(&config).context("Failed to list modules")
}

pub fn handle_conflicts(cli: &Cli) -> Result<()> {
    let config = load_config(cli)?;

    let module_list = inventory::scan(&config.moduledir, &config)
        .context("Failed to scan modules for conflict analysis")?;

    let plan = planner::generate(&config, &module_list, &config.moduledir)
        .context("Failed to generate plan for conflict analysis")?;

    let report = plan.analyze();

    let json =
        serde_json::to_string(&report.conflicts).context("Failed to serialize conflict report")?;

    println!("{}", json);

    Ok(())
}

pub fn handle_diagnostics(cli: &Cli) -> Result<()> {
    let config = load_config(cli)?;

    let module_list = inventory::scan(&config.moduledir, &config)
        .context("Failed to scan modules for diagnostics")?;

    let plan = planner::generate(&config, &module_list, &config.moduledir)
        .context("Failed to generate plan for diagnostics")?;

    let report = plan.analyze();

    let json_issues: Vec<DiagnosticIssueJson> = report
        .diagnostics
        .into_iter()
        .map(|i| DiagnosticIssueJson {
            level: match i.level {
                planner::DiagnosticLevel::Warning => "Warning".to_string(),
                planner::DiagnosticLevel::Critical => "Critical".to_string(),
            },
            context: i.context,
            message: i.message,
        })
        .collect();

    let json =
        serde_json::to_string(&json_issues).context("Failed to serialize diagnostics report")?;

    println!("{}", json);

    Ok(())
}

pub fn handle_hymofs(action: &HymofsAction) -> Result<()> {
    let abi = rustix::system::uname()
        .machine()
        .to_string_lossy()
        .into_owned();
    if abi == "x86_64" || abi == "x86-64" {
        anyhow::bail!("HymoFS operations are not supported on x86_64 architecture.");
    }

    if matches!(action, HymofsAction::Status) {
        let status = check_hymofs_status();
        let json = serde_json::to_string_pretty(&status).context("Failed to serialize status")?;
        println!("{}", json);
        return Ok(());
    }

    let fd = get_hymofs_fd(142).context("Failed to get hymofs fd")?;
    let raw_fd = fd.as_raw_fd();

    match action {
        HymofsAction::Status => unreachable!(),
        HymofsAction::Add {
            src,
            target,
            is_dir,
        } => {
            let src_c = CString::new(src.clone())?;
            let target_c = CString::new(target.clone())?;
            let arg = HymoSyscallArg {
                src: src_c.as_ptr(),
                target: target_c.as_ptr(),
                type_: if *is_dir { 1 } else { 0 },
            };
            unsafe { hymo_ioc_add_rule(raw_fd, &arg).context("ioctl hymo_ioc_add_rule failed")? };
            println!("Rule added successfully.");
        }
        HymofsAction::AddMerge { src, target } => {
            let src_c = CString::new(src.clone())?;
            let target_c = CString::new(target.clone())?;
            let arg = HymoSyscallArg {
                src: src_c.as_ptr(),
                target: target_c.as_ptr(),
                type_: 1,
            };
            unsafe {
                hymo_ioc_add_merge_rule(raw_fd, &arg)
                    .context("ioctl hymo_ioc_add_merge_rule failed")?
            };
            println!("Merge rule added successfully.");
        }
        HymofsAction::Del { src } => {
            let src_c = CString::new(src.clone())?;
            let arg = HymoSyscallArg {
                src: src_c.as_ptr(),
                target: std::ptr::null(),
                type_: 0,
            };
            unsafe { hymo_ioc_del_rule(raw_fd, &arg).context("ioctl hymo_ioc_del_rule failed")? };
            println!("Rule deleted successfully.");
        }
        HymofsAction::Hide { src } => {
            let src_c = CString::new(src.clone())?;
            let arg = HymoSyscallArg {
                src: src_c.as_ptr(),
                target: std::ptr::null(),
                type_: 0,
            };
            unsafe { hymo_ioc_hide_rule(raw_fd, &arg).context("ioctl hymo_ioc_hide_rule failed")? };
            println!("Hide rule added successfully.");
        }
        HymofsAction::HideXattr { src } => {
            let src_c = CString::new(src.clone())?;
            let arg = HymoSyscallArg {
                src: src_c.as_ptr(),
                target: std::ptr::null(),
                type_: 0,
            };
            unsafe {
                hymo_ioc_hide_overlay_xattrs(raw_fd, &arg)
                    .context("ioctl hymo_ioc_hide_overlay_xattrs failed")?
            };
            println!("Hide xattr rule added successfully.");
        }
        HymofsAction::Clear => {
            unsafe { hymo_ioc_clear_all(raw_fd).context("ioctl hymo_ioc_clear_all failed")? };
            println!("All rules cleared successfully.");
        }
        HymofsAction::List => {
            let mut buf = vec![0u8; 64 * 1024];
            let mut arg = HymoSyscallListArg {
                buf: buf.as_mut_ptr() as *mut libc::c_char,
                size: buf.len() as libc::size_t,
            };
            unsafe {
                hymo_ioc_list_rules(raw_fd, &mut arg).context("ioctl hymo_ioc_list_rules failed")?
            };
            let s = String::from_utf8_lossy(&buf[..arg.size]);
            println!("{}", s);
        }
        HymofsAction::Debug { enable } => {
            let val = if *enable { 1 } else { 0 };
            unsafe { hymo_ioc_set_debug(raw_fd, &val).context("ioctl hymo_ioc_set_debug failed")? };
            println!("Debug mode set to {}.", enable);
        }
        HymofsAction::Stealth { enable } => {
            let val = if *enable { 1 } else { 0 };
            unsafe {
                hymo_ioc_set_stealth(raw_fd, &val).context("ioctl hymo_ioc_set_stealth failed")?
            };
            println!("Stealth mode set to {}.", enable);
        }
        HymofsAction::Enable { enable } => {
            let val = if *enable { 1 } else { 0 };
            unsafe {
                hymo_ioc_set_enabled(raw_fd, &val).context("ioctl hymo_ioc_set_enabled failed")?
            };
            println!("Enabled state set to {}.", enable);
        }
    }

    Ok(())
}
