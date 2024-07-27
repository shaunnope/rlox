// use std::sync::atomic::{AtomicBool, Ordering};

pub type Error = Box<dyn std::error::Error + 'static>;

// static HAD_ERROR: AtomicBool = AtomicBool::new(false);

// pub fn set() {
//   HAD_ERROR.store(true, Ordering::Relaxed);
// }

// pub fn unset() {
//   HAD_ERROR.store(false, Ordering::Relaxed);
// }

// pub fn get() -> bool {
//   HAD_ERROR.load(Ordering::Relaxed)
// }
