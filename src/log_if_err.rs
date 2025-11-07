use core::fmt::Debug;

pub fn log_err<T, E: Debug>(result: Result<T, E>, msg: &str) {
    if let Err(e) = result {
        log::error!("{}: {:?}", msg, e);
    }
}
