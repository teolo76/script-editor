// src/elements/cast.rs

use serde::{ Deserialize, Serialize };

// Package Modules
use crate::core::{ Container, Node };

/// Marcatore semantico per il nodo Foglio del Cast (Zero-Sized Type).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CastSheet;

/// Marcatore semantico per il nodo Contenitore del Personaggio (Zero-Sized Type).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CharacterBlock;

// --- MARCATORI FOGLIA DEDICATI AGLI ELEMENTI DEL PERSONAGGIO ---

/// Marcatore per la foglia contenente il Nome del Personaggio
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CharacterNameChildBlock;

/// Marcatore per la foglia contenente il Nome dell'Attore
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActorNameBlock;

/// Marcatore per la foglia contenente una Nota di Reparto/Produzione
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DepartmentNoteBlock;

// --- ENUM PER I FIGLI AMMESSI ALL'INTERNO DI UN PERSONAGGIO ---

/// Enumerativo dei soli nodi figli ammessi dentro un contenitore `CharacterBlock`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CharacterChildNode {
    Name(Node<CharacterNameChildBlock>),
    Actor(Node<ActorNameBlock>),
    DepartmentNote(Node<DepartmentNoteBlock>),
}

impl CharacterChildNode {
    /// Restituisce l'ID del nodo contenuto indipendentemente dalla variante.
    pub fn node_id(&self) -> uuid::Uuid {
        match self {
            Self::Name(n) => n.id,
            Self::Actor(n) => n.id,
            Self::DepartmentNote(n) => n.id,
        }
    }

    /// Restituisce il record_id nell'arena contenuto indipendentemente dalla variante.
    pub fn record_id(&self) -> uuid::Uuid {
        match self {
            Self::Name(n) => n.record_id,
            Self::Actor(n) => n.record_id,
            Self::DepartmentNote(n) => n.record_id,
        }
    }
}

// --- CONTENITORI TIPIZZATI PER IL CAST ---

/// Il singolo personaggio è a sua volta un `Container` che ospita i suoi dettagli come nodi figli!
pub type CharacterContainer = Container<CharacterBlock, CharacterChildNode>;

impl CharacterContainer {
    /// Helper per creare un nuovo contenitore Personaggio.
    pub fn new_character(record_id: uuid::Uuid, sort_order: u32) -> Self {
        let node = Node::<CharacterBlock>::new(record_id, sort_order);
        Container::new(node)
    }
}

/// Il Foglio del Cast è il contenitore che ospita i singoli personaggi (`Node<CharacterBlock>`).
pub type CastContainer = Container<CastSheet, Node<CharacterBlock>>;

impl CastContainer {
    /// Helper per creare un nuovo Foglio del Cast vuoto.
    pub fn new_cast_sheet(record_id: uuid::Uuid, sort_order: u32) -> Self {
        let node = Node::<CastSheet>::new(record_id, sort_order);
        Container::new(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_character_container_with_child_nodes() {
        let char_rec_id = Uuid::new_v4();
        let mut character = CharacterContainer::new_character(char_rec_id, 1);

        // Creiamo i nodi figli dell'anagrafica personaggio
        let name_node = Node::<CharacterNameChildBlock>::new(Uuid::new_v4(), 0);
        let actor_node = Node::<ActorNameBlock>::new(Uuid::new_v4(), 1);
        let costume_note_node = Node::<DepartmentNoteBlock>::new(Uuid::new_v4(), 2);

        character.push_child(CharacterChildNode::Name(name_node));
        character.push_child(CharacterChildNode::Actor(actor_node));
        character.push_child(CharacterChildNode::DepartmentNote(costume_note_node));

        assert_eq!(character.len(), 3);
        assert_eq!(character.children()[0], CharacterChildNode::Name(name_node));
        assert_eq!(character.children()[2], CharacterChildNode::DepartmentNote(costume_note_node));
    }

    #[test]
    fn test_cast_sheet_container_hierarchy() {
        let cast_rec_id = Uuid::new_v4();
        let mut cast = CastContainer::new_cast_sheet(cast_rec_id, 0);

        let char1_node = Node::<CharacterBlock>::new(Uuid::new_v4(), 1);
        let char2_node = Node::<CharacterBlock>::new(Uuid::new_v4(), 2);

        cast.push_child(char1_node);
        cast.push_child(char2_node);

        assert_eq!(cast.len(), 2);
        assert_eq!(cast.children()[0], char1_node);
    }

    #[test]
    fn test_serde_serialization() {
        let char_rec_id = Uuid::new_v4();
        let mut character = CharacterContainer::new_character(char_rec_id, 1);
        let name_node = Node::<CharacterNameChildBlock>::new(Uuid::new_v4(), 0);
        character.push_child(CharacterChildNode::Name(name_node));

        let json = serde_json::to_string(&character).expect("Serializzazione fallita");
        let deserialized: CharacterContainer = serde_json::from_str(&json).expect("Deserializzazione fallita");

        assert_eq!(character, deserialized);
    }
}