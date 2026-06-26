//! src/common/bounding_box.rs
use serde::{ Serialize, Deserialize };
use std::hash::{Hash, Hasher};

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

// Implementazione manuale di Hash per aggirare il blocco dei float
impl Hash for BoundingBox {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Trasformiamo i bit del float in u32 (operazione a costo zero)
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
        self.width.to_bits().hash(state);
        self.height.to_bits().hash(state);
    }
}