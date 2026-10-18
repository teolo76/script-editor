// src/command/node.rs

//! Comandi su `Node`: creazione, rimozione (a cascata), titolo,
//! spostamento nell'albero. Parte del raggruppamento per dominio
//! descritto in cima a `commands::command` — qui vive solo la logica,
//! `Command`/`Undo` restano i soli tipi pubblici del modulo `commands`.

use crate::core::{Annotation, Block, Entity, Node, TreeElement};
use crate::errors::{CoreError, CoreResult};
use crate::lake::Document;
use crate::types::{ElementId, NodeKind, SortKey, Timestamp, ROOT_ID};

use crate::commands::command::ChangeKind;

#[derive(Debug, PartialEq)]
pub enum NodeCommand {
    /// `parent` non è opzionale: nessuna chiamata legittima a questo
    /// comando crea il Root (esiste già, creato una volta sola da
    /// `Document::new`, mai tramite comando), quindi ogni nodo inserito
    /// dal bus ha sempre un parent concreto. Stessa logica di `Move`.
    Insert {
        id: ElementId,
        kind: NodeKind,
        parent: ElementId,
        sort_order: SortKey,
    },
    Remove {
        id: ElementId,
    },
    SetTitle {
        id: ElementId,
        title: Option<String>,
    },
    /// Cambia parent e/o posizione di un Node. `new_parent` non è
    /// opzionale: solo il Root ha parent `None`, il Root non si può
    /// spostare (rifiutato da `apply`), quindi ogni bersaglio valido di
    /// una move è un id concreto.
    Move {
        id: ElementId,
        new_parent: ElementId,
        new_sort_order: SortKey,
    },
}

#[derive(Debug, PartialEq)]
pub(crate) enum NodeUndo {
    /// Inverso di `Insert`: rimuove semplicemente il nodo appena creato.
    /// Nessuna cascata necessaria — al momento dell'inserimento il nodo
    /// non può ancora avere figli né annotazioni ancorate.
    Remove(ElementId),
    /// Inverso di `Remove`: rimozione a cascata, quindi ripristino a
    /// cascata. Oltre al nodo stesso, tutto il suo sottoalbero (Node e
    /// Block discendenti) e tutte le `Annotation` ancorate a uno
    /// qualunque di questi elementi.
    RemoveCascade {
        node: Node,
        child_nodes: Vec<Node>,
        child_blocks: Vec<Block>,
        annotations: Vec<Annotation>,
    },
    SetTitle {
        id: ElementId,
        title: Option<String>,
    },
    Move {
        id: ElementId,
        parent: Option<ElementId>,
        sort_order: SortKey,
    },
}

pub(crate) fn classify(cmd: &NodeCommand) -> (ElementId, ChangeKind) {
    match cmd {
        NodeCommand::Insert { id, .. } => (*id, ChangeKind::Created),
        NodeCommand::Remove { id } => (*id, ChangeKind::Removed),
        NodeCommand::SetTitle { id, .. } | NodeCommand::Move { id, .. } => (*id, ChangeKind::Updated),
    }
}

pub(crate) fn cascaded_ids(undo: &NodeUndo) -> Vec<ElementId> {
    match undo {
        NodeUndo::RemoveCascade { child_nodes, child_blocks, annotations, .. } => child_nodes
            .iter()
            .map(|n| n.id())
            .chain(child_blocks.iter().map(|b| b.id()))
            .chain(annotations.iter().map(|a| a.id()))
            .collect(),
        _ => Vec::new(),
    }
}

pub(crate) fn apply(cmd: &NodeCommand, doc: &mut Document, at: Timestamp) -> CoreResult<NodeUndo> {
    match cmd {
        NodeCommand::Insert { id, kind, parent, sort_order } => {
            // `Node::with_id` richiede `Option<ElementId>` perché deve
            // ospitare anche il caso del Root (parent `None`); qui il
            // comando garantisce già un parent concreto, quindi
            // l'avvolgimento in `Some` è solo al confine fra le due
            // rappresentazioni.
            let node = Node::with_id(*id, kind.clone(), Some(*parent), sort_order.clone(), at);
            doc.insert_node(node);
            Ok(NodeUndo::Remove(*id))
        }
        NodeCommand::Remove { id } => {
            if *id == ROOT_ID {
                return Err(CoreError::CannotRemoveRoot);
            }
            let descendant_ids = doc.descendant_ids(*id);
            let mut all_ids = descendant_ids.clone();
            all_ids.push(*id);

            let mut removed_annotations = Vec::new();
            for eid in &all_ids {
                removed_annotations.extend(doc.annotations_for(*eid).into_iter().cloned());
            }
            for ann in &removed_annotations {
                doc.remove_annotation(ann.id());
            }

            let mut child_nodes = Vec::new();
            let mut child_blocks = Vec::new();
            for eid in &descendant_ids {
                if let Some(n) = doc.remove_node(*eid) {
                    child_nodes.push(n);
                } else if let Some(b) = doc.remove_block(*eid) {
                    child_blocks.push(b);
                }
            }

            let node = doc.remove_node(*id).ok_or(CoreError::ElementNotFound(*id))?;
            Ok(NodeUndo::RemoveCascade { node, child_nodes, child_blocks, annotations: removed_annotations })
        }
        NodeCommand::SetTitle { id, title } => {
            let node = doc.node_mut(*id).ok_or(CoreError::ElementNotFound(*id))?;
            let previous = node.title.clone();
            node.title = title.clone();
            Ok(NodeUndo::SetTitle { id: *id, title: previous })
        }
        NodeCommand::Move { id, new_parent, new_sort_order } => {
            if *id == ROOT_ID {
                return Err(CoreError::CannotMoveRoot);
            }
            if new_parent == id {
                return Err(CoreError::CyclicMove(*id));
            }
            if doc.node(*new_parent).is_none() {
                return Err(CoreError::ElementNotFound(*new_parent));
            }
            if doc.is_descendant_of(*new_parent, *id) {
                return Err(CoreError::CyclicMove(*id));
            }
            let node = doc.node(*id).ok_or(CoreError::ElementNotFound(*id))?;
            let previous_parent = node.parent();
            let previous_sort_order = node.sort_order().clone();
            doc.node_mut(*id)
                .expect("verificato sopra")
                .reparent(Some(*new_parent), new_sort_order.clone());
            Ok(NodeUndo::Move { id: *id, parent: previous_parent, sort_order: previous_sort_order })
        }
    }
}

pub(crate) fn revert(undo: &NodeUndo, doc: &mut Document) -> CoreResult<()> {
    match undo {
        NodeUndo::Remove(id) => {
            doc.remove_node(*id);
            Ok(())
        }
        NodeUndo::RemoveCascade { node, child_nodes, child_blocks, annotations } => {
            doc.insert_node(node.clone());
            for n in child_nodes {
                doc.insert_node(n.clone());
            }
            for b in child_blocks {
                doc.insert_block(b.clone());
            }
            for a in annotations {
                doc.insert_annotation(a.clone());
            }
            Ok(())
        }
        NodeUndo::SetTitle { id, title } => {
            if let Some(node) = doc.node_mut(*id) {
                node.title = title.clone();
            }
            Ok(())
        }
        NodeUndo::Move { id, parent, sort_order } => {
            if let Some(node) = doc.node_mut(*id) {
                node.reparent(*parent, sort_order.clone());
            }
            Ok(())
        }
    }
}