// src/lake/store.rs

//! Arena generico: memorizza entità indicizzandole per il loro stesso
//! `ElementId`. È il mattone di base su cui `Document` costruisce un
//! "lake" per ciascun tipo di entità (Node, Block, Annotation, Layer).
//! `Character` usa invece `CharacterStore`, che compone questo arena
//! aggiungendo l'indice per alias — la regola di unicità dell'alias non
//! appartiene a questo tipo generico.
//!
//! Deliberatamente non sa nulla di alberi, layer o annotazioni: quella
//! conoscenza di dominio vive in `document.rs`, che compone più
//! `EntityStore` e aggiunge le query che li attraversano insieme.
use std::collections::HashMap;

// Package Modules
use crate::core::Entity;
use crate::types::ElementId;

pub struct EntityStore<T> {
    items: HashMap<ElementId, T>,
}

impl<T> EntityStore<T> {
    pub fn new() -> Self {
        Self { items: HashMap::new() }
    }

    pub fn get(&self, id: ElementId) -> Option<&T> {
        self.items.get(&id)
    }

    pub fn get_mut(&mut self, id: ElementId) -> Option<&mut T> {
        self.items.get_mut(&id)
    }

    pub fn remove(&mut self, id: ElementId) -> Option<T> {
        self.items.remove(&id)
    }

    pub fn contains(&self, id: ElementId) -> bool {
        self.items.contains_key(&id)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.values()
    }
}

impl<T: Entity> EntityStore<T> {
    pub fn insert(&mut self, item: T) -> Option<T> {
        self.items.insert(item.id(), item)
    }
}

impl<T>Default for EntityStore<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Node;
    use crate::types::{NodeKind, SortKey, Timestamp};

    #[test]
    fn insert_and_get_roundtrip() {
        let mut store: EntityStore<Node> = EntityStore::new();
        let node = Node::new(NodeKind::Act, None, SortKey::initial(), Timestamp::from_unix_seconds(0));
        let id = node.id();
        assert!(store.insert(node).is_none());
        assert!(store.get(id).is_some());
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn insert_replaces_existing_id() {
        let mut store: EntityStore<Node> = EntityStore::new();
        let node = Node::new(NodeKind::Act, None, SortKey::initial(), Timestamp::from_unix_seconds(0))
            .with_title("Primo titolo");
        let id = node.id();
        store.insert(node);

        let mut updated = store.get(id).unwrap().clone();
        updated.title = Some("Titolo aggiornato".to_string());
        let previous = store.insert(updated);

        assert_eq!(previous.unwrap().title.as_deref(), Some("Primo titolo"));
        assert_eq!(store.get(id).unwrap().title.as_deref(), Some("Titolo aggiornato"));
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn remove_returns_item() {
        let mut store: EntityStore<Node> = EntityStore::new();
        let node = Node::new(NodeKind::Scene, None, SortKey::initial(), Timestamp::from_unix_seconds(0));
        let id = node.id();
        store.insert(node);
        assert!(store.remove(id).is_some());
        assert!(store.get(id).is_none());
        assert!(store.is_empty());
    }
}