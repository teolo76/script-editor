// src/geometry/bounding_box.rs

use serde::{ Deserialize, Serialize };
use std::hash::{ Hash, Hasher };

/// Rappresenta il rettangolo di posizionamento proporzionale (da 0.0 a 1.0)
/// sulla superficie del foglio di lavoro (es. A4) [cite: 1, 2].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl BoundingBox {
    /// Inizializza una bounding box neutra a superficie zero[cite: 2].
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0, width: 0.0, height: 0.0, }
    }
    /// Costruttore esplicito con coordinate e dimensioni proporzionali[cite: 2].
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height, }
    }
    /// Genera una geometria che satura l'intera superficie del foglio (0.0 a 1.0).
    pub fn full_page() -> Self {
        Self { x: 0.0, y: 0.0, width: 1.0, height: 1.0, }
    }
}

// Implementazione manuale di Hash per aggirare il blocco dei tipi float in Rust.
impl Hash for BoundingBox {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Trasformiamo i bit del float in u32 a costo zero di CPU
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
        self.width.to_bits().hash(state);
        self.height.to_bits().hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn test_bounding_box_constructors() {
        let bbox = BoundingBox::new(0.1, 0.2, 0.5, 0.3);
        assert_eq!(bbox.x, 0.1);
        assert_eq!(bbox.y, 0.2);
        assert_eq!(bbox.width, 0.5);
        assert_eq!(bbox.height, 0.3);

        let zero_box = BoundingBox::zero();
        assert_eq!(zero_box.x, 0.0);
        assert_eq!(zero_box.height, 0.0);

        let full_box = BoundingBox::full_page();
        assert_eq!(full_box.width, 1.0);
        assert_eq!(full_box.height, 1.0);
    }

    #[test]
    fn test_hash_implementation() {
        let bbox1 = BoundingBox::new(0.1, 0.2, 0.3, 0.4);
        let bbox2 = BoundingBox::new(0.1, 0.2, 0.3, 0.4);

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        bbox1.hash(&mut hasher1);
        bbox2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish(), "I due BoundingBox identici devono produrre lo stesso hash");
    }

    #[test]
    fn test_serde_serialization() {
        let bbox = BoundingBox::new(0.25, 0.50, 0.75, 1.00);
        let json = serde_json::to_string(&bbox).expect("Serializzazione fallita");

        let deserialized: BoundingBox = serde_json::from_str(&json).expect("Deserializzazione fallita");
        assert_eq!(deserialized, bbox);
    }
}