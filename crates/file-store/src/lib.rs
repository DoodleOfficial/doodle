/**
 * @file lib.rs
 * @author Krisna Pranav
 * @brief lib
 * @version 1.0
 * @date 2024-11-25
 *
 * @copyright Copyright (c) 2024 Doodle Developers, Krisna Pranav
 *
 */

pub type Result<T> = std::result::Result<T, anyhow::Error>;

pub mod const_serializable;
pub mod iterable;
pub mod peekable;
pub mod random_lookup;
pub mod temp;

pub use const_serializable::ConstSerializable;
pub use peekable::Peekable;
use temp::TempDir;

pub fn gen_temp_path() -> std::path::PathBuf {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::SystemTime;

    static SALT_COUNTER: AtomicUsize = AtomicUsize::new(0);

    let seed = SALT_COUNTER.fetch_add(1, Ordering::SeqCst) as u128;

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        << 48;

    let pid = u128::from(std::process::id());

    let salt = (pid << 16) + now + seed;

    if cfg!(target_os = "linux") {
        format!("/dev/shm/pagecache.tmp.{salt}").into()
    } else {
        std::env::temp_dir().join(format!("pagecache.tmp.{salt}"))
    }
}

pub fn gen_temp_dir() -> Result<TempDir> {
    TempDir::new()
}