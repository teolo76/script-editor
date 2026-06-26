//! src/common/bounding_box.rs
use serde::{ Serialize, Deserialize };

/// Rappresenta il rettangolo di posizionamento proporzionale (da 0.0 a 1.0) sul foglio A4.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl BoundingBox {
    pub fn full_page() -> Self {
        Self { x: 0.0, y: 0.0, width: 1.0, height: 1.0 }
    }
}