use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{str, string};

static BUF_SIZE: usize = 1024 * 64; // a "large enough" buffer to do one read() on {proc,sys,kern}fs

macro_rules! cont_on_err {
    ($res:expr) => {
        match $res {
            Ok(val) => val,
            Err(_) => continue,
        }
    };
}

macro_rules! cont_on_none {
    ($res:expr) => {
        match $res {
            Some(val) => val,
            None => continue,
        }
    };
}

#[derive(Debug)]
enum DStateError {
    StringUtf8(string::FromUtf8Error),
    StrUtf8(str::Utf8Error),
    Io(io::Error),
    InvalidStatFile,
}

#[derive(Hash, Eq, PartialEq, Debug)]
enum StackType {
    Kernel,
    User,
}

impl From<io::Error> for DStateError {
    fn from(err: io::Error) -> DStateError {
        DStateError::Io(err)
    }
}

impl From<string::FromUtf8Error> for DStateError {
    fn from(err: string::FromUtf8Error) -> DStateError {
        DStateError::StringUtf8(err)
    }
}

impl From<str::Utf8Error> for DStateError {
    fn from(err: str::Utf8Error) -> DStateError {
        DStateError::StrUtf8(err)
    }
}

/// read_to_string() uses an expanding buffer, so is dangerous with {proc,kern,sys}fs.
fn read_to_string_single<T: AsRef<Path>>(path: T) -> Result<String, DStateError> {
    let mut file = File::open(path)?;
    let mut buf: Vec<_> = vec![0; BUF_SIZE];
    let len_read = file.read(&mut buf[..])?;
    buf.truncate(len_read);
    let out = String::from_utf8(buf)?;
    Ok(out)
}

fn get_state(path: &PathBuf) -> Result<String, DStateError> {
    let mut stat_path = path.clone();
    stat_path.push("stat");
    let line = read_to_string_single(stat_path)?;
    let fields: Vec<_> = line.split_whitespace().collect();
    Ok(fields
        .get(2)
        .ok_or(DStateError::InvalidStatFile)?
        .to_string())
}

fn get_proc_pid_file(path: &PathBuf, filename: &str) -> Result<String, DStateError> {
    let mut file_path = path.clone();
    file_path.push(filename);
    let stack = read_to_string_single(file_path)?;
    Ok(stack)
}

fn get_user_stack(pid: u64) -> Result<String, DStateError> {
    let raw_out = Command::new("quickstack")
        .args(&["-d0", "-p", &pid.to_string()])
        .output()?
        .stdout;
    Ok(str::from_utf8(&raw_out)?.trim().to_string())
}

fn get_d_state_stacks() -> HashMap<u64, HashMap<StackType, String>> {
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
            get_proc_pid_file(&path, "stack").unwrap_or_else(|e| format!("unavailable: {:?}", e)),
        );
        stack_map.insert(
            StackType::User,
            get_user_stack(pid).unwrap_or_else(|e| format!("unavailable: {:?}", e)),
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
    Ok(get_proc_pid_file(&get_proc_pid_path(pid), "cmdline")?)
}

fn get_pid_comm(pid: u64) -> Result<String, DStateError> {
    Ok(get_proc_pid_file(&get_proc_pid_path(pid), "comm")?
        .trim()
        .to_string())
}

fn main() {
    for (pid, stacks) in get_d_state_stacks() {
        println!(
            "---\n\n# {} (comm: {}) (cmd: {}):\n\nKernel stack:\n\n{}\nUserspace stack:\n\n{}\n\n",
            pid,
            get_pid_comm(pid).unwrap_or_else(|_| "unknown".to_string()),
            get_pid_cmdline(pid).unwrap_or_else(|_| "unknown".to_string()),
            stacks[&StackType::Kernel],
            stacks[&StackType::User]
        );
    }
}
