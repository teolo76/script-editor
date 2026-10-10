// src/lake/mod.rs

//! Namespace dedicato agli arena di storage: `EntityStore` (il "lake"
//! generico, uno per tipo di entità), `CharacterStore` (arena
//! specializzato con l'indice per alias) e `Document` (che li compone e
//! aggiunge le query di dominio). Separato da `core` perché `core` è il
//! modello puro — non sa nulla di come le entità vengono immagazzinate o
//! attraversate, questo lo sa `lake`.
mod character_store;

mod document;
mod store;

pub use store::EntityStore;
pub use character_store::CharacterStore;
pub use document::{Document, ElementRef};
