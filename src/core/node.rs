// src/core/node.rs

//! Nodo strutturale dell'albero del copione: Root, Atto, Scena, e tipi
//! custom definiti a livello di progetto (`NodeKind::Custom`).
//!
//! Un `Node` non tiene una lista di figli: l'appartenenza è espressa dal
//! `parent` di ciascun figlio (in `BaseElement`), e l'elenco/ordine dei
//! figli di un dato nodo si ottiene interrogando l'arena filtrando per
//! `parent` e ordinando per `sort_order`. Tenere qui una lista
//! duplicherebbe lo stato e andrebbe risincronizzata a ogni spostamento —
//! meglio una fonte di verità sola.

use crate::core::{ BaseElement, Meta, Entity, TreeElement };
use crate::types::{ ElementId, NodeKind, SortKey, Timestamp, ROOT_ID };

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    base: BaseElement,
    pub kind: NodeKind,
    /// Titolo visualizzato (es. "Atto I", "Scena 3 – Il giardino"). Per il
    /// nodo Root resta `None`: il titolo dell'opera vive nei metadati di
    /// progetto, non sul nodo Root.
    pub title: Option<String>,
}

impl Node {
    pub fn new(kind: NodeKind, parent: Option<ElementId>,  order: SortKey, at: Timestamp,) -> Self {
        Self {
            base: BaseElement::new(parent, order, at),
            kind,
            title: None,
        }
    }

    /// Come `new`, ma con id esplicito invece che generato. Usato per il
    /// nodo Root e dal command bus, che deve poter ricreare un nodo con lo
    /// stesso id sia alla prima applicazione di un comando sia a ogni suo
    /// redo.
    pub fn with_id(id: ElementId, kind: NodeKind, parent: Option<ElementId>, order: SortKey, at: Timestamp) -> Self {
        Self {
            base: BaseElement::with_id(id, parent, order, at),
            kind,
            title: None,
        }
    }

    /// Il nodo Root: id fisso (`ROOT_ID`), nessun parent. Ce n'è uno solo
    /// per documento, creato all'inizializzazione dell'arena.
    pub fn root(at: Timestamp) -> Self {
        Self{
            base: BaseElement::with_id(ROOT_ID, None, SortKey::initial(), at),
            kind: NodeKind::Root,
            title: None,
        }
    }

    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Cambia parent e posizione fra fratelli. Visibile solo all'interno
    /// del crate: va chiamato esclusivamente da `commands::Command::MoveNode`
    /// dopo aver validato l'assenza di cicli (vedi
    /// `lake::Document::is_descendant_of`) — mai direttamente, altrimenti
    /// si bypassano sia quella validazione sia la cronologia undo/redo.
    /// Stesso principio di `Character::set_alias` per l'unicità
    /// dell'alias: la validazione è un vincolo che riguarda l'intero
    /// documento, non il singolo `Node`, quindi non vive qui.
    pub(crate) fn reparent(&mut self, parent: Option<ElementId>, order: SortKey) {
        self.base.parent = parent;
        self.base.sort_order = order;
    }
}

impl Entity for Node {
    fn meta(&self) -> &Meta {
        self.base.meta()
    }
}

impl TreeElement for Node {
    fn parent(&self) -> Option<ElementId> {
        self.base.parent()
    }

    fn sort_order(&self) -> &SortKey {
        self.base.sort_order()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_has_fixed_id_and_no_parent() {
        let root = Node::root(Timestamp::from_unix_seconds(0));
        assert_eq!(root.id(), ROOT_ID);
        assert_eq!(root.parent(), None);
    }

    #[test]
    fn new_node_has_generated_id_and_declared_parent() {
        let root = Node::root(Timestamp::from_unix_seconds(0));
        let act = Node::new(
            NodeKind::Act,
            Some(root.id()),
            SortKey::initial(),
            Timestamp::from_unix_seconds(0),
        );
        assert_ne!(act.id(), ROOT_ID);
        assert_eq!(act.parent(), Some(root.id()));
    }

    #[test]
    fn with_title_sets_title() {
        let node = Node::new(
            NodeKind::Scene,
            None,
            SortKey::initial(),
            Timestamp::from_unix_seconds(0),
        )
            .with_title("Il giardino");
        assert_eq!(node.title.as_deref(), Some("Il giardino"));
    }
}