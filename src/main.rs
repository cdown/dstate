#[macro_use]
mod macros;
mod errors;

use errors::DStateError;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

static BUF_SIZE: usize = 1024 * 64; // a "large enough" buffer to do one read() on {proc,sys,kern}fs

#[derive(Hash, Eq, PartialEq, Debug)]
enum StackType {
    Kernel,
    User,
}

/// read_to_string() uses an expanding buffer, so is dangerous with {proc,kern,sys}fs.
fn read_to_string_single(path: &PathBuf) -> Result<String, DStateError> {
    let mut file = File::open(path)?;
    let mut buf: Vec<_> = vec![0; BUF_SIZE];
    let len_read = file.read(&mut buf[..])?;
    buf.truncate(len_read);
    let out = String::from_utf8(buf)?;
    Ok(out)
}

fn get_state(path: &Path) -> Result<String, DStateError> {
    let mut stat_path = path.to_path_buf();
    stat_path.push("stat");
    let line = read_to_string_single(&stat_path)?;
    let fields: Vec<_> = line.split_whitespace().collect();
    Ok(fields
        .get(2)
        .ok_or(DStateError::InvalidStatFile)?
        .to_string())
}

fn get_proc_pid_file(path: &Path, filename: &str) -> Result<String, DStateError> {
    let mut file_path = path.to_path_buf();
    file_path.push(filename);
    let stack = read_to_string_single(&file_path)?;
    Ok(stack)
}

fn get_kernel_stack(path: &Path) -> Result<Option<String>, DStateError> {
    let stack = get_proc_pid_file(path, "stack")?;

    // If there's only one line (address -1), there's no current kernel stack
    if stack.lines().count() < 2 {
        Ok(None)
    } else {
        Ok(Some(stack))
    }
}

fn get_user_stack(pid: u64) -> Result<Option<String>, DStateError> {
    let raw_out = Command::new("quickstack")
        .args(["-d0", "-p", &pid.to_string()])
        .output()?
        .stdout;

    let stack = str::from_utf8(&raw_out)?.trim().to_string();

    if !stack.contains("0x") {
        // This stack is empty and probably a kernel thread
        Ok(None)
    } else {
        Ok(Some(stack))
    }
}

fn get_d_state_stacks() -> HashMap<u64, HashMap<StackType, Option<String>>> {
    let dentries = fs::read_dir("/proc").expect("Can't read /proc");
    let mut out = HashMap::new();

    for dentry in dentries {
        let path = cont_on_err!(dentry).path();
        if !path.is_dir() {
            continue;
        }
        if get_state(&path).unwrap_or_else(|_| "".to_string()) != "D" {
            continue;
        }
        let dir_name = cont_on_none!(cont_on_none!(path.file_name()).to_str());
        let pid = cont_on_err!(dir_name.parse());
        let mut stack_map = HashMap::new();
        stack_map.insert(
            StackType::Kernel,
            get_kernel_stack(&path).unwrap_or_else(|e| Some(format!("unavailable: {:?}", e))),
        );
        stack_map.insert(
            StackType::User,
            get_user_stack(pid).unwrap_or_else(|e| Some(format!("unavailable: {:?}", e))),
        );
        out.insert(pid, stack_map);
    }

    out
}

fn get_proc_pid_path(pid: u64) -> PathBuf {
    let mut path = PathBuf::from("/proc");
    path.push(pid.to_string());
    path
}

fn get_pid_cmdline(pid: u64) -> Result<String, DStateError> {
    get_proc_pid_file(&get_proc_pid_path(pid), "cmdline")
}

fn get_pid_comm(pid: u64) -> Result<String, DStateError> {
    Ok(get_proc_pid_file(&get_proc_pid_path(pid), "comm")?
        .trim()
        .to_string())
}

fn main() {
    for (pid, mut stacks) in get_d_state_stacks() {
        println!(
            "---\n\n# {} (comm: {}) (cmd: {}):\n\n",
            pid,
            get_pid_comm(pid).unwrap_or_else(|_| "unknown".to_string()),
            get_pid_cmdline(pid).unwrap_or_else(|_| "unknown".to_string()),
        );

        for (stype, print_name) in &[(StackType::Kernel, "Kernel"), (StackType::User, "User")] {
            if let Some(stack) = stacks.remove(stype).unwrap_or(None) {
                println!("{} stack:\n\n{}\n", print_name, stack);
            }
        }
    }
}
