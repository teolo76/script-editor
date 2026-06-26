//! src/elements/cover.rs

use uuid::Uuid;
use serde::{ Serialize, Deserialize };

// Package Files & Modules
use crate::core::VisualLayer;

/// Rappresenta un asset grafico posizionato sulla copertina (es: logo o poster).
/// Risiede interamente nello Stack e mantiene il tratto Copy.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CoverImage<'a> {
    pub visual: VisualLayer,
    pub asset_path: &'a str,
}

impl<'a> CoverImage<'a> {
    /// Costruttore standard per l'immagine di copertina
    pub fn new(asset_path: &'a str, visual: VisualLayer) -> Self {
        Self {
            visual,
            asset_path: asset_path.trim(),
        }
    }
}