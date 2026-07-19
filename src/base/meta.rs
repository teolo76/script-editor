// src/base/meta.rs

use serde::{ Serialize, Deserialize };
use uuid::Uuid;

// Package Modules
use crate::traits::BaseElement;
use crate::common::utils::current_timestamp;

/// Metadati comuni a tutti gli elementi (Identità e Tempo).
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Debug, Hash)]
pub struct Meta {
    pub id: Uuid,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Meta {
    /// Crea una nuova istanza di Meta e assegna un nuovo id
    pub fn new() -> Self {
        let now = current_timestamp();
        Self { id: Uuid::new_v4(), created_at: now, updated_at: now }
    }

    /// Crea una istanza di Meta con dati già esistenti
    pub fn with_specs(id: Uuid, created_at: u64, updated_at: u64) -> Self {
        Self { id, created_at, updated_at }
    }

    /// Aggiorna il timestamp di updated_at
    pub fn touch(&mut self) {
        self.updated_at = current_timestamp();
    }
}

impl BaseElement for Meta {
    fn id(&self) -> Uuid {
        self.id
    }

    fn created_at(&self) -> u64 {
        self.created_at
    }

    fn updated_at(&self) -> u64 {
        self.updated_at
    }

    fn touch(&mut self) {
        self.touch();
    }
}

// Unit Test

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    /// Verifica che il costruttore di default generi correttamente
    /// istanze con UUID univoci e timestamp coerenti.
    #[test]
    fn test_meta_new_initialization() {
        let meta_1 = Meta::new();
        let meta_2 = Meta::new();

        // Gli UUID devono essere assolutamente unici (prevenzione collisioni)
        assert_ne!(meta_1.id, meta_2.id);

        // I timestamp di creazione e aggiornamento iniziali devono coincidere perfettamente
        assert_eq!(meta_1.created_at, meta_1.updated_at);
        assert!(meta_1.created_at > 0);
    }

    /// Verifica che il costruttore esplicito con specifiche manuali
    /// assegni esattamente i dati forniti senza alterazioni.
    #[test]
    fn test_meta_with_specs() {
        let custom_id = Uuid::new_v4();
        let created = 1000_u64;
        let updated = 2000_u64;

        let meta = Meta::with_specs(custom_id, created, updated);

        assert_eq!(meta.id, custom_id);
        assert_eq!(meta.created_at, created);
        assert_eq!(meta.updated_at, updated);
    }

    /// Verifica che il metodo touch aggiorni correttamente il timestamp
    /// di modifica lasciando intatto quello di creazione.
    #[test]
    fn test_meta_touch_update() {
        let mut meta = Meta::new();
        let original_created = meta.created_at;
        let original_updated = meta.updated_at;

        // Introduciamo una piccolissima pausa artificiale per far avanzare il clock di sistema
        thread::sleep(Duration::from_millis(2));

        // Aggiorniamo il timestamp di modifica
        meta.touch();

        // La data di creazione DEVE rimanere immutata
        assert_eq!(meta.created_at, original_created);

        // La data di modifica DEVE essere avanzata rispetto a quella originale
        assert!(
            meta.updated_at > original_updated,
            "Il metodo touch non ha incrementato il timestamp. Vecchio: {}, Nuovo: {}",
            original_updated, meta.updated_at
        );
    }
}