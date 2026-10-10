// src/core/annotation.rs

//! Annotation: nota fuori dal contenuto (di regia, tecnica, di traduzione,
//! ...) ancorata a un elemento o a un intervallo di testo, mai incorporata
//! nel testo del blocco. Vive su un layer come i blocchi, così si accende
//! e spegne insieme alle altre annotazioni dello stesso tipo.
//!
//! `Annotation` non implementa `TreeElement`: non è un nodo dell'albero,
//! punta a un elemento esistente tramite `anchor`.

use crate::core::{LayerId, Meta, Entity };
use crate::types::{ ElementId, Timestamp };

/// Cosa ancora un'annotazione: un intero elemento (Node o Block) oppure un
/// intervallo di testo dentro un Block.
///
/// Gli offset di `TextRange` sono indici di carattere (non byte, per via di
/// UTF-8) e vanno tenuti coerenti dai comandi di editing quando il testo
/// del blocco cambia prima o dentro l'intervallo ancorato — è compito del
/// command bus, non di questo tipo, mantenerli allineati.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnchorTarget {
    Element(ElementId),
    TextRange {
        block_id: ElementId,
        start: usize,
        end: usize,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Annotation {
    pub meta: Meta,
    pub layer: LayerId,
    pub anchor: AnchorTarget,
    pub body: String,
}

impl Annotation {
    pub fn new(layer: LayerId, target: AnchorTarget, body: impl Into<String>, at: Timestamp) -> Self {
        Self {
            meta: Meta::new(at),
            layer,
            anchor: target,
            body: body.into(),
        }
    }

    /// L'elemento a cui questa annotazione fa riferimento, sia che sia
    /// ancorata all'intero elemento sia a un intervallo al suo interno.
    /// Utile all'arena per rispondere a "quali annotazioni riguardano
    /// questo blocco", senza dover fare match sull'anchor a ogni query.
    pub fn anchored_element(&self) -> ElementId {
        match self.anchor {
            AnchorTarget::Element(element) => element,
            AnchorTarget::TextRange {block_id, .. } => block_id,
        }
    }
}

impl Entity for Annotation {
    fn meta(&self) -> &Meta { &self.meta }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ElementId;

    #[test]
    fn anchored_element_on_whole_element() {
        let target = ElementId::new();
        let annotation = Annotation::new(
            LayerId::new(),
            AnchorTarget::Element(target),
            "Nota di regia",
            Timestamp::from_unix_seconds(0),
        );
        assert_eq!(annotation.anchored_element(), target);
    }

    #[test]
    fn anchored_element_on_text_range() {
        let block_id = ElementId::new();
        let annotation = Annotation::new(
            LayerId::new(),
            AnchorTarget::TextRange {
                block_id,
                start: 3,
                end: 10,
            },
            "Enfasi qui",
            Timestamp::from_unix_seconds(0),
        );
        assert_eq!(annotation.anchored_element(), block_id);
    }
}