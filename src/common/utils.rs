// src/common/utils.rs

use std::time::{ UNIX_EPOCH, SystemTime };

/// Helper per ottenere il timestamp attuale in millisecondi
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}