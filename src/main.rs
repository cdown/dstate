use std::collections::HashSet;
use std::fs;

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

fn get_d_state_pids() -> HashSet<u64> {
    let dentries = fs::read_dir("/proc").expect("Can't read /proc");
    let mut pids = HashSet::new();

    for dentry in dentries {
        let path = cont_on_err!(dentry).path();
        if !path.is_dir() {
            continue;
        }
        let dir_name = cont_on_none!(cont_on_none!(path.file_name()).to_str());
        let pid = cont_on_err!(dir_name.parse::<u64>());
        pids.insert(pid);
    }

    pids
}

fn main() {
    println!("{:?}", get_d_state_pids());
}
