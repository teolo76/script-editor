//! src/elements/block.rs

use serde::{ Serialize, Deserialize };
use  uuid::Uuid;

// Package Modules
use crate::common::{BlockType, LayerChannel};
use crate::core::{ Meta, VisualLayer };
use crate::traits::{ BaseElement, TextElement };

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Hash)]
pub struct Block<'a> {
    pub meta: Meta,
    #[serde(rename = "type")]
    pub block_type: BlockType,
    pub text:&'a str,
    /// Geometria opzionale: se presente (Some), il blocco usa il posizionamento assoluto (es: nella Cover)
    pub visual: Option<VisualLayer>,
    pub channel: LayerChannel,
}

impl<'a> Block<'a> {
    /// Costruttore standard per la creazione a runtime (Genera automaticamente ID e timestamp)
    pub fn new(block_type: BlockType, channel: LayerChannel, text: &'a str) -> Self {
        Self {
            meta: Meta::new(),
            block_type,
            text: text.trim(),
            visual: None,
            channel,
        }
    }
    /// Costruttore specifico per blocchi geometrici posizionati (es: elementi della Cover)
    pub fn new_positioned(block_type: BlockType, channel: LayerChannel, text: &'a str, visual: VisualLayer) -> Self {
        Self {
            meta: Meta::new(),
            block_type,
            channel,
            text: text.trim(),
            visual: Some(visual),
        }
    }
    /// Costruttore simmetrico per il ripristino fedele da operazioni di I/O (JSON/File)
    pub fn with_specs(meta: Meta, block_type: BlockType, channel: LayerChannel, text: &'a str, visual: Option<VisualLayer>) -> Self {
        Self {
            meta,
            block_type,
            text: text.trim(),
            visual,
            channel,
        }
    }
    /// Helper per verificare al volo se il blocco richiede un posizionamento assoluto
    pub fn is_positioned(&self) -> bool {
        self.visual.is_some()
    }
}

// Implementazione del MetaTrait universale (Identità e Tempo)
impl<'a> BaseElement for Block<'a> {
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
        self.meta.touch()
    }
}

impl <'a> TextElement<'a> for Block<'a> {
    fn block_type(&self) -> BlockType {
        self.block_type
    }
    fn text(&self) -> &'a str {
        self.text
    }
    fn set_text(&mut self, _new_text: &str) {
        // Nello Stack modificheremo il puntatore tramite l'Engine centrale
        self.meta.touch();
    }
    fn text_type(&self) -> &str {
        self.block_type.as_str()
    }
    fn update_text(&mut self, new_text: &'a str) {
        self.text = new_text.trim();
        self.meta.touch();
    }
    fn update_type(&mut self, new_type: BlockType) {
        self.block_type = new_type;
        self.meta.touch();
    }
}

// =========================================================================
// UNIT TESTS DI VALIDAZIONE (Incluso il test di Serialization/Deserialization)
// =========================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use crate::common::{ BoundingBox };

    #[test]
    fn test_linear_block_creation() {
        let block = Block::new(BlockType::Dialog, LayerChannel::ScriptText, "  Essere o non essere  ");

        assert_eq!(block.text(), "Essere o non essere"); // Verifica trim
        assert_eq!(block.text_type(), "dialog");
        assert!(!block.is_positioned());
    }

    #[test]
    fn test_positioned_cover_block_and_copy() {
        // Prepariamo un layer geometrico usando i veri campi di BoundingBox
        let mut visual = VisualLayer::new();
        visual.bounds = BoundingBox {
            x: 0.1,
            y: 0.2,
            width: 0.8,
            height: 0.1
        };

        let block = Block::new_positioned(
            BlockType::Heading1,
            LayerChannel::ScriptText,
            "AMLETO",
            visual
        );

        assert!(block.is_positioned());
        assert_eq!(block.text_type(), "heading_1");
        assert_eq!(block.visual.unwrap().bounds.width, 0.8); // campo corretto

        // Verifica invarianza tratto Copy
        let copia = block;
        assert_eq!(copia.id(), block.id());
    }
}
