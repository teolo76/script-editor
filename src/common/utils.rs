//! src/common/utils.rs
use std::time::{ SystemTime, UNIX_EPOCH};

/// Helper interno per ottenere il timestamp attuale in millisecondi
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}