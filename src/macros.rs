macro_rules! cont_on_err {
    ($res:expr) => {
        match $res {
            Ok(val) => val,
            Err(_) => continue,
        }
    };
}
pub(crate) use cont_on_err;

macro_rules! cont_on_none {
    ($res:expr) => {
        match $res {
            Some(val) => val,
            None => continue,
        }
    };
}
pub(crate) use cont_on_none;
