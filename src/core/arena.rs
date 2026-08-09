// src/core/arena.rs

use std::collections::HashMap;
use serde::{ Deserialize, Serialize };
use uuid::Uuid;

// Package Modules
use crate::core::DocumentRecord;

/// Storage piatto e centralizzato per tutti i `DocumentRecord<P>` dell'applicazione.
///
/// Disaccoppia la gerarchia dell'albero dei nodi dai dati reali mantenuti nella heap.
/// Permette lookup, aggiornamenti e rimozioni in $O(1)$ tramite `Uuid`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Arena<P> {
    records: HashMap<Uuid, DocumentRecord<P>>,
}

impl<P> Arena<P> {
    /// Inizializza un'arena vuota.
    pub fn new() -> Self {
        Self { records: HashMap::new(), }
    }

    /// Inizializza un'arena con una capacità iniziale preallocata per ridurre le riallocazioni.
    pub fn with_capacity(capacity: usize) -> Self {
        Self { records: HashMap::with_capacity(capacity), }
    }

    /// Inserisce un `DocumentRecord<P>` nell'arena, restituendone l'UUID.
    pub fn insert(&mut self, record: DocumentRecord<P>) -> Uuid {
        let id = record.id();
        self.records.insert(id, record);
        id
    }

    /// Recupera un riferimento immutabile al `DocumentRecord<P>` associato all'ID specificato.
    pub fn get(&self, id: &Uuid) -> Option<&DocumentRecord<P>> {
        self.records.get(id)
    }

    /// Recupera un riferimento mutabile al `DocumentRecord<P>` associato all'ID.
    /// Invocando questo metodo, il record viene automaticamente marcato come modificato (`touch()`).
    pub fn get_mut(&mut self, id: &Uuid) -> Option<&mut DocumentRecord<P>> {
        if let Some(record) = self.records.get_mut(id) {
            record.touch();
            Some(record)
        } else {
            None
        }
    }

    /// Rimuove e restituisce il `DocumentRecord<P>` dall'arena se presente.
    pub fn remove(&mut self, id: &Uuid) -> Option<DocumentRecord<P>> {
        self.records.remove(id)
    }

    /// Verifica se l'arena contiene un record associato all'ID dato.
    pub fn contains(&self) -> impl Fn(&Uuid) -> bool + '_ {
        move |id| self.records.contains_key(id)
    }

    /// Restituisce il numero totale di record memorizzati nell'arena.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Verifica se l'arena è completamente vuota.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Svuota completamente l'arena rimuovendo tutti i record.
    pub fn clear(&mut self) {
        self.records.clear();
    }

    /// Restituisce un iteratore sui riferimenti immutabili di tutti i record presenti.
    pub fn values(&self) -> impl Iterator<Item = &DocumentRecord<P>> {
        self.records.values()
    }

    /// Restituisce un iteratore sulle coppie (UUID, DocumentRecord<P>) per la serializzazione.
    pub fn iter(&self) -> impl Iterator<Item = (&Uuid, &DocumentRecord<P>)> {
        self.records.iter()
    }
}

impl<P> Default for Arena<P> {
    fn default() -> Self {
        Self::new()
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
    fn test_arena_insert_and_get() {
        let mut arena = Arena::new();
        let payload = MockPayload {
            text: "Dialogo 1".to_string(),
        };
        let record = DocumentRecord::new(payload.clone());
        let id = record.id();

        arena.insert(record);

        assert_eq!(arena.len(), 1);
        assert!(!arena.is_empty());
        assert!(arena.contains()(&id));

        let retrieved = arena.get(&id).expect("Record deve essere presente");
        assert_eq!(retrieved.payload, payload);
    }

    #[test]
    fn test_arena_get_mut_auto_touch() {
        let mut arena = Arena::new();
        let record = DocumentRecord::new(MockPayload {
            text: "Original Text".to_string(),
        });
        let id = arena.insert(record);

        let initial_version = arena.get(&id).unwrap().meta.version;

        sleep(Duration::from_millis(2));
        {
            let rec_mut = arena.get_mut(&id).expect("Record deve essere presente");
            rec_mut.payload.text = "Modified Text".to_string();
        }

        let updated_rec = arena.get(&id).unwrap();
        assert_eq!(updated_rec.payload.text, "Modified Text");
        assert_eq!(updated_rec.meta.version, initial_version + 1);
    }

    #[test]
    fn test_arena_remove_and_clear() {
        let mut arena = Arena::new();
        let rec1 = DocumentRecord::new(MockPayload { text: "1".to_string() });
        let rec2 = DocumentRecord::new(MockPayload { text: "2".to_string() });

        let id1 = arena.insert(rec1);
        let id2 = arena.insert(rec2);

        assert_eq!(arena.len(), 2);

        let removed = arena.remove(&id1);
        assert!(removed.is_some());
        assert_eq!(arena.len(), 1);
        assert!(!arena.contains()(&id1));
        assert!(arena.contains()(&id2));

        arena.clear();
        assert_eq!(arena.len(), 0);
        assert!(arena.is_empty());
    }

    #[test]
    fn test_serde_serialization() {
        let mut arena = Arena::new();
        let rec = DocumentRecord::new(MockPayload {
            text: "Serialized Item".to_string(),
        });
        arena.insert(rec);

        let json = serde_json::to_string(&arena).expect("Serializzazione fallita");
        let deserialized: Arena<MockPayload> =
            serde_json::from_str(&json).expect("Deserializzazione fallita");

        assert_eq!(arena, deserialized);
    }
}