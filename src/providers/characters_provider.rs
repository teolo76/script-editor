//! src/providers/characters_provider.rs

use uuid::Uuid;

// Package Files & Modules
use crate::elements::Character;

/// Interfaccia astratta per la risoluzione dinamica dei personaggi e dei loro interpreti.
pub trait CharactersProvider<'a> {
    /// Riceve un UUID e restituisce un riferimento immutabile all'oggetto Character.
    fn resolve_character(&self, id: &Uuid) -> Option<&Character<'a>>;
    /// Riceve un UUID e restituisce un riferimento mutabile per aggiornare note o attori.
    fn resolve_character_mut(&mut self, id: &Uuid) -> Option<&mut Character<'a>>;
}