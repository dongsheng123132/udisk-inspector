pub mod cli;
pub mod db;
pub mod disk;
pub mod report;
pub mod test;
pub mod types;

use std::sync::atomic::AtomicBool;

pub static STOP_FLAG: AtomicBool = AtomicBool::new(false);
