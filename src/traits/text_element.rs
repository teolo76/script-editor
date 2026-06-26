//! TextElement

use uuid::Uuid;
use serde::{ Serialize, Deserialize };

// Package Module
use crate::common::BlockType;
use crate::traits::BaseElement;

/// Definisce il comportamento di un atomo di testo manipolabile dall'editor.
pub trait TextElement<'a>: BaseElement + Serialize + Deserialize<'a> {
    /// Restituisce il tipo drammaturgico del blocco
    fn block_type(&self) -> BlockType;
    /// Restituisce la fetta di testo corrente
    fn text(&self) -> &'a str;
    /// Aggiorna il contenuto testuale e marca l'elemento come modificato (touch)
    fn set_text(&mut self, new_text: &str);
    /// Restituisce la stringa identificativa del tipo di blocco (es: "heading_1", "dialog")
    fn text_type(&self) -> &str;
    /// Sostituisce il testo puntato con una nuova fetta di memoria
    fn update_text(&mut self, new_text: &'a str);
    /// Cambia la natura drammaturgica del blocco (es. da Dialog a Parenthetical)
    fn update_type(&mut self, new_type: BlockType);
}