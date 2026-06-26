//! src/elements/character.rs

use uuid::Uuid;
use serde::{ Serialize, Deserialize };

// Package Files & Modules
use crate::core::Meta;
use crate::traits::BaseElement;

/// Rappresenta un Personaggio all'interno del copione.
/// Include i metadati di produzione per attori e note di reparto (Costumi, Trucco, ecc.).
/// Risiede interamente nello Stack grazie a &'a str e implementa Copy.
#[derive(Serialize,Deserialize,Debug, Copy, Clone, PartialEq, Hash)]
pub struct Character<'a> {
    pub meta: Meta,
    /// Il nome del personaggio come appare nel copione (es. "AMLETO")
    pub name: &'a str,
    /// Il nome e cognome dell'attore associato (es. "Laurence Olivier")
    pub actor_name: &'a str,
    /// Note di produzione specifiche (costumi, trucco, indicazioni di reparto)
    pub production_notes: &'a str,
}

impl <'a> Character<'a> {
    /// Costruttore standard per la creazione a runtime nella UI
    pub fn new(name: &'a str) -> Self {
        Self { meta: Meta::new(), name: name.trim(), actor_name: "", production_notes: "", }
    }
    /// Costruttore simmetrico completo per il ripristino da I/O (JSON/File) o clonazioni avanzate
    pub fn with_specs(meta: Meta, name: &'a str, actor_name: &'a str, production_notes: &'a str) -> Self {
        Self {
            meta,
            name: name.trim(),
            actor_name: actor_name.trim(),
            production_notes: production_notes.trim(),
        }
    }
    /// Assegna o aggiorna l'attore interprete
    pub fn set_actor(&mut self, actor_name: &'a str) {
        self.actor_name = actor_name.trim();
        self.meta.touch();
    }
    /// Aggiorna le note di produzione/costume
    pub fn set_production_notes(&mut self, notes: &'a str) {
        self.production_notes = notes.trim();
        self.meta.touch();
    }
    /// Aggiorna il nome del personaggio nel copione
    pub fn set_name(&mut self, new_name: &'a str) {
        self.name = new_name.trim();
        self.meta.touch();
    }
}

// Implementazione del MetaTrait universale per il tracciamento di Identità e Tempo
impl <'a> BaseElement for Character<'a> {
    fn id(&self) -> Uuid {
        self.meta.id
    }
    fn created_at(&self) -> u64 {
        self.meta.created_at
    }
    fn updated_at(&self) -> u64 {
        self.meta.updated_at
    }
    fn touch(&mut self) {
        self.meta.touch();
    }
}

// =========================================================================
// UNIT TESTS DI VALIDAZIONE
// =========================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_character_creation_and_copy() {
        let mut char_1 = Character::new(" ORAZIO ");

        // Verifica trim nativo
        assert_eq!(char_1.name, "ORAZIO");
        assert_eq!(char_1.actor_name, "");

        // Verifica del tratto Copy puro nello Stack
        let char_2 = char_1;
        assert_eq!(char_1.id(), char_2.id());

        char_1.set_actor("John Doe");
        assert_eq!(char_1.actor_name, "John Doe");
        assert_eq!(char_2.actor_name, ""); // La copia originale rimane intatta
    }

    #[test]
    fn test_character_serialization_roundtrip() {
        let custom_meta = Meta::with_specs(Uuid::new_v4(), 2000, 2000);
        let character = Character::with_specs(
            custom_meta,
            "AMLETO",
            "Laurence Olivier",
            "Costume nero, anello con sigillo"
        );

        // Serializzazione
        let json_output = serde_json::to_string(&character).unwrap();
        assert!(json_output.contains("\"name\":\"AMLETO\""));
        assert!(json_output.contains("\"actor_name\":\"Laurence Olivier\""));

        // Deserializzazione Zero-Copy
        let deserialized: Character = serde_json::from_str(&json_output).unwrap();
        assert_eq!(deserialized.id(), character.id());
        assert_eq!(deserialized.production_notes, "Costume nero, anello con sigillo");
    }
}