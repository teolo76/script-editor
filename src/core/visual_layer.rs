//! src/core/visual_layer.rs
use serde::{ Serialize, Deserialize };

// Package Files & Modules
use crate::common::{ BoundingBox, LayerChannel };

/// Il motore spaziale e logico applicabile ai Container del copione.
/// Gestisce la profondità, la visibilità globale e i filtri dei canali attivi.
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct VisualLayer {
    pub z_index: i32,
    pub visible: bool,
    pub bounds: BoundingBox,

    // Matrice fissa di visibilità per i singoli canali (Stile Photoshop/AutoCAD)
    // Indicizziamo i canali in ordine: [ScriptText, DirectorNotes, ProductionMeta, Revision]
    pub channel_mask: [bool; 4],
}

impl VisualLayer {
    /// Inizializza un layer standard con tutti i canali attivi di default
    pub fn new() -> Self {
        Self {
            z_index: 0,
            visible: true,
            bounds: BoundingBox::full_page(),
            channel_mask: [true, true, true, true],
        }
    }
    /// Configura il layer per un posizionamento geometrico specifico (es. elementi grafici della Cover)
    pub fn with_bounds(z_index: i32, bounds: BoundingBox) -> Self {
        Self {
            z_index,
            visible: true,
            bounds,
            channel_mask: [true, false, false, false], // Solo testo base attivo per elementi geometrici
        }
    }
    /// Accende o spegne un singolo canale logico (es. nasconde le note di regia)
    pub fn set_channel(&mut self, channel: LayerChannel, is_enabled: bool) {
        self.channel_mask[channel.to_index()] = is_enabled;
    }
    /// Interroga lo stato di un canale per capire se deve entrare nella pipeline di impaginazione
    pub fn is_channel_enabled(&self, channel: LayerChannel) -> bool {
        self.channel_mask[channel.to_index()]
    }
}

// =========================================================================
// UNIT TESTS DI VALIDAZIONE
// =========================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual_layer_preserves_copy_and_defaults() {
        let layer_1 = VisualLayer::new();

        // Verifica del tratto Copy puro nello Stack
        let mut layer_2 = layer_1;

        assert!(layer_1.visible);
        assert_eq!(layer_1.bounds.width, 1.0);

        // Modificando la copia, l'originale resta blindato
        layer_2.visible = false;
        assert!(layer_1.visible);
        assert!(!layer_2.visible);
    }

    #[test]
    fn test_channel_mask_filtering() {
        let mut layer = VisualLayer::new();

        // All'inizio tutti i canali sono attivi
        assert!(layer.is_channel_enabled(LayerChannel::DirectorNotes));
        assert!(layer.is_channel_enabled(LayerChannel::ScriptText));

        // Spegniamo il canale delle note di regia (editing pulito attivo)
        layer.set_channel(LayerChannel::DirectorNotes, false);

        // Verifichiamo che il filtro sia isolato correttamente
        assert!(!layer.is_channel_enabled(LayerChannel::DirectorNotes));
        assert!(layer.is_channel_enabled(LayerChannel::ScriptText)); // Il testo resta intatto
    }

    #[test]
    fn test_proportional_geometry() {
        let custom_bounds = BoundingBox { x: 0.1, y: 0.2, width: 0.8, height: 0.1 };
        let layer = VisualLayer::with_bounds(10, custom_bounds);

        assert_eq!(layer.z_index, 10);
        assert_eq!(layer.bounds.x, 0.1);
        assert_eq!(layer.bounds.height, 0.1);

        // Gli elementi geometrici puri nascono con i canali extra spenti
        assert!(!layer.is_channel_enabled(LayerChannel::DirectorNotes));
    }
}
