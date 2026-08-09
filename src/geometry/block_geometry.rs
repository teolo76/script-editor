// src/geometry/block_geometry.rs

use serde::{ Deserialize, Serialize };

/// Geometria di rientro e spaziatura specifica per un singolo elemento del copione.
/// Espressa interamente in millimetri (mm) per l'indipendenza dalla densità di pixel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockGeometry {
    /// Rientro dal margine sinistro della colonna stampabile (in mm)
    pub margin_left: f32,
    /// Rientro dal margine destro della colonna stampabile (in mm)
    pub margin_right: f32,
    /// Spazio vuoto verticale da lasciare SOPRA il blocco (in mm)
    pub space_before: f32,
    /// Spazio vuoto verticale da lasciare SOTTO il blocco (in mm)
    pub space_after: f32,
}

impl BlockGeometry {
    /// Crea una nuova geometria con i parametri specificati (espressi in mm).
    pub fn new(margin_left: f32, margin_right: f32, space_before: f32, space_after: f32) -> Self {
        Self {
            margin_left,
            margin_right,
            space_before,
            space_after,
        }
    }

    /// Inizializza una geometria neutra priva di spaziature e rientri (0 mm).
    pub fn zero() -> Self {
        Self {
            margin_left: 0.0,
            margin_right: 0.0,
            space_before: 0.0,
            space_after: 0.0,
        }
    }

    /// Calcola l'ingombro verticale totale (overhead) introdotto dalle spaziature prima e dopo il blocco.
    pub fn vertical_overhead(&self) -> f32 {
        self.space_before + self.space_after
    }

    /// Calcola l'ingombro orizzontale totale introdotto dai margini sinistro e destro.
    pub fn horizontal_margins(&self) -> f32 {
        self.margin_left + self.margin_right
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_geometry_new_and_zero() {
        let geom = BlockGeometry::new(10.0, 15.0, 2.0, 4.0);
        assert_eq!(geom.margin_left, 10.0);
        assert_eq!(geom.margin_right, 15.0);
        assert_eq!(geom.space_before, 2.0);
        assert_eq!(geom.space_after, 4.0);

        let zero_geom = BlockGeometry::zero();
        assert_eq!(zero_geom.margin_left, 0.0);
        assert_eq!(zero_geom.margin_right, 0.0);
        assert_eq!(zero_geom.space_before, 0.0);
        assert_eq!(zero_geom.space_after, 0.0);
    }

    #[test]
    fn test_geometry_calculations() {
        let geom = BlockGeometry::new(12.5, 25.0, 5.0, 3.0);
        assert_eq!(geom.vertical_overhead(), 8.0);
        assert_eq!(geom.horizontal_margins(), 37.5);
    }

    #[test]
    fn test_serde_serialization() {
        let geom = BlockGeometry::new(5.0, 10.0, 1.5, 2.5);
        let json = serde_json::to_string(&geom).expect("Serializzazione fallita");

        let deserialized: BlockGeometry = serde_json::from_str(&json).expect("Deserializzazione fallita");
        assert_eq!(deserialized, geom);
    }
}