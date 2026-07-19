// src/base/visual.rs

use serde::{ Serialize, Deserialize };

// Package Modules
use crate::common::LayerType;
use crate::geometry::BoundingBox;

/// Il motore spaziale e logico applicabile ai "Container" del copione.
/// Gestisce la profondità, la visibilità globale e i filtri dei canali attivi.
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize, Hash)]
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
    pub fn set_channel(&mut self, channel: LayerType, is_enabled: bool) {
        self.channel_mask[channel.to_index()] = is_enabled;
    }

    /// Controlla la visibilità globale e l'abilitazione del canale specifico.
    pub fn is_active_in_channel(&self, channel_index: usize) -> bool {
        self.visible && channel_index < 4 && self.channel_mask[channel_index]
    }

    /// Interroga lo stato di un canale per capire se deve entrare nella pipeline di impaginazione
    pub fn is_channel_enabled(&self, channel: LayerType) -> bool {
        self.channel_mask[channel.to_index()]
    }
}

// Unit Test

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual_layer_default_new() {
        let layer = VisualLayer::new();

        assert_eq!(layer.z_index, 0);
        assert!(layer.visible);
        // Verifica che occupi tutta la pagina di default
        assert_eq!(layer.bounds, BoundingBox::full_page());
        // Tutti i canali devono essere accesi
        assert_eq!(layer.channel_mask, [true, true, true, true]);
    }

    #[test]
    fn test_visual_layer_with_bounds() {
        let custom_bounds = BoundingBox::new(0.1, 0.1, 0.5, 0.2);
        let layer = VisualLayer::with_bounds(10, custom_bounds);

        assert_eq!(layer.z_index, 10);
        assert_eq!(layer.bounds, custom_bounds);
        // Deve avere solo il primo canale (ScriptText) attivo
        assert!(layer.is_channel_enabled(LayerType::ScriptText));
        assert!(!layer.is_channel_enabled(LayerType::DirectorNotes));
    }

    #[test]
    fn test_set_and_query_channels() {
        let mut layer = VisualLayer::new();

        // Spegniamo le note di produzione
        layer.set_channel(LayerType::ProductionMeta, false);

        assert!(layer.is_channel_enabled(LayerType::ScriptText));
        assert!(!layer.is_channel_enabled(LayerType::ProductionMeta));

        // Verifica la corrispondenza con l'indice numerico grezzo
        assert!(!layer.is_active_in_channel(LayerType::ProductionMeta.to_index()));
    }
}