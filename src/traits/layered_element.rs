//! src/traits/layered.rs

use crate::core::VisualLayer;

/// Interfaccia per i contenitori che gestiscono layer visivi, visibilità e geometrie.
pub trait LayeredElement {
    /// Restituisce un riferimento al layer visivo dell'oggetto
    fn visual_layer(&self) -> &VisualLayer;

    /// Restituisce un riferimento mutabile per modificare geometrie o visibilità
    fn visual_layer_mut(&mut self) -> &mut VisualLayer;

    /// Accende o spegne la visibilità di questo intero livello logico
    fn set_visible(&mut self, visible: bool) {
        self.visual_layer_mut().visible = visible;
    }
}