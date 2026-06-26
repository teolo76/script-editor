//! src/core/meta.rs

use serde::{ Serialize, Deserialize };
use uuid::Uuid;

// Package Files & Modules
use crate::common::utils::current_timestamp;

/// Contenitore atomico di metadati di sistema puri (Identità e Tempo).
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Debug, Hash)]
pub struct Meta {
    pub id: Uuid,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Meta {
    pub fn new() -> Self {
        let now = current_timestamp();
        Self { id: Uuid::new_v4(), created_at: now, updated_at: now }
    }

    pub fn with_specs(id: Uuid, created_at: u64, updated_at: u64) -> Self {
        Self { id, created_at, updated_at }
    }

    pub fn touch(&mut self) {
        self.updated_at = current_timestamp();
    }
}