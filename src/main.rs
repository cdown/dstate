extern crate failure;
#[macro_use]
extern crate failure_derive;

use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::string;

static BUF_SIZE: usize = 1024 * 64; // a "large enough" buffer to do one read()

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

#[derive(Fail, Debug)]
enum DStateError {
    #[fail(display = "{}", _0)]
    Utf8(#[fail(cause)] string::FromUtf8Error),
    #[fail(display = "{}", _0)]
    Io(#[fail(cause)] io::Error),
    #[fail(display = "Invalid stat file")]
    InvalidStatFile,
}

impl From<io::Error> for DStateError {
    fn from(err: io::Error) -> DStateError {
        DStateError::Io(err)
    }
}

impl From<string::FromUtf8Error> for DStateError {
    fn from(err: string::FromUtf8Error) -> DStateError {
        DStateError::Utf8(err)
    }
}

/// read_to_string() uses an expanding buffer, so is dangerous with {proc,kern,sys}fs.
fn read_to_string_single<T: AsRef<Path>>(path: T) -> Result<String, DStateError> {
    let mut file = File::open(path)?;
    let mut buf: Vec<u8> = vec![0; BUF_SIZE];
    let len_read = file.read(&mut buf[..])?;
    buf.truncate(len_read);
    let out = String::from_utf8(buf)?;
    Ok(out)
}

fn get_state(path: &PathBuf) -> Result<String, DStateError> {
    let mut stat_path = path.clone();
    stat_path.push("stat");
    let line = read_to_string_single(stat_path)?;
    let fields: Vec<&str> = line.split_whitespace().collect();
    Ok(fields
        .get(2)
        .ok_or(DStateError::InvalidStatFile)?
        .to_string())
}

fn get_stack(pid: u64) -> Result<String, DStateError> {
    let stack_path: PathBuf = [r"/proc", &pid.to_string(), "stack"].iter().collect();
    let stack = read_to_string_single(stack_path)?;
    Ok(stack)
}

fn get_d_state_pids() -> HashSet<u64> {
    let dentries = fs::read_dir("/proc").expect("Can't read /proc");
    let mut pids = HashSet::new();

    for dentry in dentries {
        let path = cont_on_err!(dentry).path();
        if !path.is_dir() {
            continue;
        }
        if get_state(&path).unwrap_or_else(|_| "".to_string()) != "D" {
            continue;
        }
        let dir_name = cont_on_none!(cont_on_none!(path.file_name()).to_str());
        let pid = cont_on_err!(dir_name.parse::<u64>());
        pids.insert(pid);
    }

    pids
}

fn get_pid_to_stack() -> HashMap<u64, String> {
    let mut out = HashMap::new();
    for pid in get_d_state_pids() {
        out.insert(
            pid,
            get_stack(pid).unwrap_or_else(|_| "unavailable".to_string()),
        );
    }
    out
}

fn main() {
    println!("{:?}", get_pid_to_stack());
}
