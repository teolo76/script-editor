// src/traits/layered_element.rs

use crate::base::VisualLayer;

/// Interfaccia agnostica per gli elementi che gestiscono layer visivi, geometrie e visibilità.
pub trait LayeredElement {
    /// Restituisce un riferimento immutabile al layer visivo dell'oggetto.
    fn visual_layer(&self) -> &VisualLayer;

    /// Restituisce un riferimento mutabile per modificare geometrie, maschere o profondità.
    fn visual_layer_mut(&mut self) -> &mut VisualLayer;

    /// Accende o spegne la visibilità globale di questo livello logico.
    fn set_visible(&mut self, visible: bool) {
        self.visual_layer_mut().visible = visible;
    }
}