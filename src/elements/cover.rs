// src/elements/cover.rs

use serde::{ Deserialize, Serialize };

// Package Modules
use crate::core::{ Container, Node };
use crate::elements::blocks::CoverTextBlock;

/// Marcatore semantico per il nodo Copertina (Zero-Sized Type)[cite: 2].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Cover;

/// Marcatore per un asset grafico posizionato sulla copertina (es. logo)[cite: 1, 2].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CoverAssetBlock;

/// Enumerativo dei soli elementi validi come figli della Copertina[cite: 1, 2].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoverBlock {
    Text(Node<CoverTextBlock>),
    Asset(Node<CoverAssetBlock>),
}

impl CoverBlock {
    /// Restituisce l'ID del nodo contenuto indipendentemente dalla variante[cite: 1].
    pub fn node_id(&self) -> uuid::Uuid {
        match self {
            Self::Text(n) => n.id,
            Self::Asset(n) => n.id,
        }
    }

    /// Restituisce il record_id nell'arena contenuto indipendentemente dalla variante[cite: 1].
    pub fn record_id(&self) -> uuid::Uuid {
        match self {
            Self::Text(n) => n.record_id,
            Self::Asset(n) => n.record_id,
        }
    }
}

/// Un contenitore Copertina tipizzato, vincolato ad accogliere solo varianti di `CoverBlock`[cite: 1, 2].
pub type CoverContainer = Container<Cover, CoverBlock>;

impl CoverContainer {
    /// Helper per creare una nuova Copertina vuota assegnandole un `record_id` per l'arena e un `sort_order`[cite: 1, 2].
    pub fn new_cover(record_id: uuid::Uuid, sort_order: u32) -> Self {
        let node = Node::<Cover>::new(record_id, sort_order);
        Container::new(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_cover_container_creation_and_block_push() {
        let cover_rec_id = Uuid::new_v4();
        let mut cover = CoverContainer::new_cover(cover_rec_id, 0);

        assert!(cover.is_empty());
        assert_eq!(cover.node.record_id, cover_rec_id);

        let title_text_node = Node::<CoverTextBlock>::new(Uuid::new_v4(), 1);
        let logo_asset_node = Node::<CoverAssetBlock>::new(Uuid::new_v4(), 2);

        cover.push_child(CoverBlock::Text(title_text_node));
        cover.push_child(CoverBlock::Asset(logo_asset_node));

        assert_eq!(cover.len(), 2);
        assert_eq!(cover.children()[0], CoverBlock::Text(title_text_node));
        assert_eq!(cover.children()[1], CoverBlock::Asset(logo_asset_node));
    }

    #[test]
    fn test_cover_serde_serialization() {
        let cover_rec_id = Uuid::new_v4();
        let mut cover = CoverContainer::new_cover(cover_rec_id, 0);

        let text_node = Node::<CoverTextBlock>::new(Uuid::new_v4(), 1);
        cover.push_child(CoverBlock::Text(text_node));

        let json = serde_json::to_string(&cover).expect("Serializzazione fallita");
        let deserialized: CoverContainer = serde_json::from_str(&json).expect("Deserializzazione fallita");

        assert_eq!(cover, deserialized);
    }
}