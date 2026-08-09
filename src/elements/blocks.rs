// src/elements/blocks.rs

use serde::{ Deserialize, Serialize };

// --- STRUCT MARCATORE PER I BLOCCHI ATOMICI (Zero-Sized Types) ---

/// Marcatore per il blocco Titolo
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HeadingBlock;

/// Marcatore per il blocco Nome Personaggio
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CharacterNameBlock;

/// Marcatore per il blocco Dialogo
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DialogBlock;

/// Marcatore per il blocco Parentesi / Intonazione (es. (sottovoce))
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ParentheticalBlock;

/// Marcatore per il blocco Didascalia / Azione
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StageDirectionBlock;

/// Marcatore per il blocco Transizione (es. STACCO SU:)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransitionBlock;

/// Marcatore per il blocco Note del Regista
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DirectorNoteBlock;

/// Marcatore per gli elementi specifici della Cover (Autore, Versione, Note Produzione)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CoverTextBlock;

// --- ENUM PER I FIGLI AMMESSI NELLE SCENE ---

use crate::core::Node;

/// Enumerativo fortemente tipizzato dei soli blocchi validi come figli di una Scena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SceneBlock {
    Heading(Node<HeadingBlock>),
    CharacterName(Node<CharacterNameBlock>),
    Dialog(Node<DialogBlock>),
    Parenthetical(Node<ParentheticalBlock>),
    StageDirection(Node<StageDirectionBlock>),
    Transition(Node<TransitionBlock>),
    DirectorNote(Node<DirectorNoteBlock>),
}

impl SceneBlock {
    /// Restituisce l'ID del nodo contenuto indipendentemente dalla variante.
    pub fn node_id(&self) -> uuid::Uuid {
        match self {
            Self::Heading(n) => n.id,
            Self::CharacterName(n) => n.id,
            Self::Dialog(n) => n.id,
            Self::Parenthetical(n) => n.id,
            Self::StageDirection(n) => n.id,
            Self::Transition(n) => n.id,
            Self::DirectorNote(n) => n.id,
        }
    }

    /// Restituisce il record_id nell'arena contenuto indipendentemente dalla variante.
    pub fn record_id(&self) -> uuid::Uuid {
        match self {
            Self::Heading(n) => n.record_id,
            Self::CharacterName(n) => n.record_id,
            Self::Dialog(n) => n.record_id,
            Self::Parenthetical(n) => n.record_id,
            Self::StageDirection(n) => n.record_id,
            Self::Transition(n) => n.record_id,
            Self::DirectorNote(n) => n.record_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_scene_block_helpers() {
        let rec_id = Uuid::new_v4();
        let dialog_node: Node<DialogBlock> = Node::new(rec_id, 1);
        let block = SceneBlock::Dialog(dialog_node);

        assert_eq!(block.node_id(), dialog_node.id);
        assert_eq!(block.record_id(), rec_id);
    }

    #[test]
    fn test_serde_serialization() {
        let rec_id = Uuid::new_v4();
        let node: Node<StageDirectionBlock> = Node::new(rec_id, 5);
        let block = SceneBlock::StageDirection(node);

        let json = serde_json::to_string(&block).expect("Serializzazione fallita");
        let deserialized: SceneBlock = serde_json::from_str(&json).expect("Deserializzazione fallita");

        assert_eq!(block, deserialized);
    }
}