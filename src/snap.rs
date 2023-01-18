use std::env;
use std::path::PathBuf;

/// Checks to see whether we are operating within a snap
pub(crate) fn check_in_snap() -> (bool, Option<PathBuf>) {
    if env::var("SNAP_NAME").is_ok() {
        if let Ok(host_home_str) = env::var("SNAP_REAL_HOME") {
            (true, Some(PathBuf::from(host_home_str.as_str())))
        } else {
            (true, None)
        }
    } else {
        (false, None)
    }
}
