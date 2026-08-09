// src/core/record.rs

use serde::{ Deserialize, Serialize };
use uuid::Uuid;

// Package Modules
use crate::common::utils::current_timestamp;
use crate::geometry::BlockGeometry;

/// Contenitore atomico di metadati di sistema (Identità e Tracciamento Temporale).
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct RecordMeta {
    pub id: Uuid,
    pub created_at: u64,
    pub updated_at: u64,
    pub version: u32,
}

impl RecordMeta {
    pub fn new() -> Self {
        let now = current_timestamp();
        Self { id: Uuid::new_v4(), created_at: now, updated_at: now, version: 1, }
    }

    pub fn with_specs(id: Uuid, created_at: u64, updated_at: u64, version: u32) -> Self {
        Self { id, created_at, updated_at, version, }
    }

    /// Aggiorna il timestamp di modifica al millisecondo attuale e incrementa la versione interna.
    pub fn touch(&mut self) {
        self.updated_at = current_timestamp();
        self.version = self.version.saturating_add(1);
    }
}

impl Default for RecordMeta {
    fn default() -> Self {
        Self::new()
    }
}

/// Involucro universale per i dati reali mantenuti nello storage piatto dell'arena.
///
/// `P` rappresenta il payload informativo contenuto nel record
/// (es. testo, attributi personaggio, configurazioni).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentRecord<P> {
    /// Metadati di sistema, identificativo univoco e timestamp
    pub meta: RecordMeta,
    /// Geometria di margine e spaziatura millimetrica
    pub geometry: BlockGeometry,
    /// Il contenuto informativo reale
    pub payload: P,
}

impl<P> DocumentRecord<P> {
    /// Inizializza un nuovo record assegnandogli un ID univoco, una geometria neutra e il payload dato.
    pub fn new(payload: P) -> Self {
        Self {
            meta: RecordMeta::new(),
            geometry: BlockGeometry::zero(),
            payload,
        }
    }

    /// Inizializza un nuovo record fornendo sia la geometria sia il payload.
    pub fn with_geometry(geometry: BlockGeometry, payload: P) -> Self {
        Self {
            meta: RecordMeta::new(),
            geometry,
            payload,
        }
    }

    /// Costruttore completo per il ripristino esplicito da operazioni di I/O.
    pub fn with_specs(meta: RecordMeta, geometry: BlockGeometry, payload: P) -> Self {
        Self {
            meta,
            geometry,
            payload,
        }
    }

    /// Restituisce l'UUID del record.
    pub fn id(&self) -> Uuid {
        self.meta.id
    }

    /// Notifica una modifica al record, aggiornando il timestamp `updated_at` e incrementando la versione.
    pub fn touch(&mut self) {
        self.meta.touch();
    }

    /// Restituisce un riferimento mutabile al payload, invocando automaticamente `touch()`.
    pub fn payload_mut(&mut self) -> &mut P {
        self.touch();
        &mut self.payload
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct MockPayload {
        pub text: String,
    }

    #[test]
    fn test_record_meta_creation_and_touch() {
        let mut meta = RecordMeta::new();
        let initial_updated = meta.updated_at;
        let initial_version = meta.version;

        assert_eq!(initial_version, 1);
        assert_ne!(meta.id, Uuid::nil());

        sleep(Duration::from_millis(5));
        meta.touch();

        assert!(meta.updated_at >= initial_updated);
        assert_eq!(meta.version, 2);
    }

    #[test]
    fn test_document_record_creation() {
        let payload = MockPayload {
            text: "Hello World".to_string(),
        };
        let record = DocumentRecord::new(payload.clone());

        assert_eq!(record.payload, payload);
        assert_eq!(record.geometry, BlockGeometry::zero());
        assert_eq!(record.meta.version, 1);
    }

    #[test]
    fn test_payload_mut_auto_touch() {
        let payload = MockPayload {
            text: "Initial Text".to_string(),
        };
        let mut record = DocumentRecord::new(payload);
        let initial_version = record.meta.version;

        sleep(Duration::from_millis(2));
        {
            let p = record.payload_mut();
            p.text = "Updated Text".to_string();
        }

        assert_eq!(record.payload.text, "Updated Text");
        assert_eq!(record.meta.version, initial_version + 1);
    }

    #[test]
    fn test_serde_serialization() {
        let payload = MockPayload {
            text: "Test Dialog".to_string(),
        };
        let record = DocumentRecord::with_geometry(
            BlockGeometry::new(10.0, 10.0, 2.0, 2.0),
            payload,
        );

        let json = serde_json::to_string(&record).expect("Serializzazione fallita");
        let deserialized: DocumentRecord<MockPayload> =
            serde_json::from_str(&json).expect("Deserializzazione fallita");

        assert_eq!(record, deserialized);
    }
}