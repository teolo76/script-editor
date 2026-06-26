//! src/providers/text_provider.rs

use uuid::Uuid;

// Package Files & Modules
use crate::elements::Block;

/// Interfaccia astratta per la risoluzione dinamica dei blocchi di testo.
/// Disaccoppia la struttura lineare dei container dallo storage centrale
pub trait TextProvider<'a> {
    /// Riceve un UUID e restituisce un riferimento immutabile al blocco reale, se esistente.
    fn resolve_block(&self, id: &Uuid) -> Option<&Block<'a>>;
    /// Riceve un UUID e restituisce un riferimento mutabile per manipolazioni dirette.
    fn resolve_block_mut(&mut self, id: &Uuid) -> Option<&mut Block<'a>>;
}