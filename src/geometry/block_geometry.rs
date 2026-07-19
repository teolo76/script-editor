// src/geometry/block_geometry.rs

use serde::{ Serialize, Deserialize };

/// Geometria di rientro e spaziatura specifica per un singolo elemento del copione.
/// Espressa interamente in millimetri (mm) per l'indipendenza dalla densità di pixel.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlockGeometry {
    /// Rientro dal margine sinistro della colonna stampabile
    pub margin_left: f32,
    /// Rientro dal margine destro della colonna stampabile
    pub margin_right: f32,
    /// Spazio vuoto verticale da lasciare SOPRA il blocco
    pub space_before: f32,
    /// Spazio vuoto verticale da lasciare SOTTO il blocco
    pub space_after: f32,
}

impl BlockGeometry {
    pub fn new(margin_left: f32, margin_right: f32, space_before: f32, space_after: f32) -> Self {
        Self { margin_left, margin_right, space_before, space_after, }
    }

    /// Inizializza una geometria senza spaziature né rientri.
    pub fn zero() -> Self {
        Self { margin_left: 0.0, margin_right: 0.0, space_before: 0.0, space_after: 0.0 }
    }
}
