// src/traits/base_element.rs

use uuid::Uuid;

/// Il MetaTrait universale del copione. Garantisce che ogni elemento
/// abbia un'identità tracciabile e metadati temporali uniformi.
pub trait BaseElement {
    /// Restituisce l'UUID univoco dell'elemento
    fn id(&self) -> Uuid;

    /// Restituisce il timestamp di creazione (Unix Epoch ms)
    fn created_at(&self) -> u64;

    /// Restituisce il timestamp dell'ultima modifica (Unix Epoch ms)
    fn updated_at(&self) -> u64;

    /// Aggiorna il timestamp di modifica al millisecondo attuale
    fn touch(&mut self);
}