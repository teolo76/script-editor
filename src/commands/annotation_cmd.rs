// src/commands/annotation.rs

//! Comandi su `Annotation`: creazione, rimozione, riancoraggio. Parte
//! del raggruppamento per dominio descritto in cima a
//! `commands::command`.
//!
//! `Insert`/`Remove` sono nuovi in questo raggruppamento: prima le
//! annotazioni si creavano chiamando `Document::insert_annotation`
//! direttamente, scavalcando il bus — quindi senza undo. Non c'era una
//! ragione di design per questa eccezione, solo che non era stata ancora
//! chiusa.

use crate::core::{AnchorTarget, Annotation, LayerId};
use crate::errors::{CoreError, CoreResult};
use crate::lake::Document;
use crate::types::{ElementId, Timestamp};

use crate::commands::command::ChangeKind;

#[derive(Debug, PartialEq)]
pub enum AnnotationCommand {
    Insert {
        id: ElementId,
        layer_id: LayerId,
        anchor: AnchorTarget,
        body: String,
    },
    Remove {
        id: ElementId,
    },
    /// Cambia l'ancora di un'annotazione (a cosa è agganciata), non la
    /// sua posizione in un elenco: nome diverso da "Move" apposta,
    /// perché la semantica è diversa da un riordino fra fratelli.
    Reanchor {
        id: ElementId,
        new_anchor: AnchorTarget,
    },
}

#[derive(Debug, PartialEq)]
pub(crate) enum AnnotationUndo {
    Remove(ElementId),
    Reinsert(Annotation),
    Reanchor { id: ElementId, anchor: AnchorTarget },
}

pub(crate) fn classify(cmd: &AnnotationCommand) -> (ElementId, ChangeKind) {
    match cmd {
        AnnotationCommand::Insert { id, .. } => (*id, ChangeKind::Created),
        AnnotationCommand::Remove { id } => (*id, ChangeKind::Removed),
        AnnotationCommand::Reanchor { id, .. } => (*id, ChangeKind::Updated),
    }
}

pub(crate) fn cascaded_ids(_undo: &AnnotationUndo) -> Vec<ElementId> {
    Vec::new()
}

pub(crate) fn apply(cmd: &AnnotationCommand, doc: &mut Document, at: Timestamp) -> CoreResult<AnnotationUndo> {
    match cmd {
        AnnotationCommand::Insert { id, layer_id, anchor, body } => {
            let annotation = Annotation::with_id(*id, *layer_id, *anchor, body.clone(), at);
            doc.insert_annotation(annotation);
            Ok(AnnotationUndo::Remove(*id))
        }
        AnnotationCommand::Remove { id } => {
            let removed = doc.remove_annotation(*id).ok_or(CoreError::ElementNotFound(*id))?;
            Ok(AnnotationUndo::Reinsert(removed))
        }
        AnnotationCommand::Reanchor { id, new_anchor } => {
            let annotation = doc.annotation_mut(*id).ok_or(CoreError::ElementNotFound(*id))?;
            let previous = annotation.anchor;
            annotation.anchor = *new_anchor;
            Ok(AnnotationUndo::Reanchor { id: *id, anchor: previous })
        }
    }
}

pub(crate) fn revert(undo: &AnnotationUndo, doc: &mut Document) -> CoreResult<()> {
    match undo {
        AnnotationUndo::Remove(id) => {
            doc.remove_annotation(*id);
            Ok(())
        }
        AnnotationUndo::Reinsert(annotation) => {
            doc.insert_annotation(annotation.clone());
            Ok(())
        }
        AnnotationUndo::Reanchor { id, anchor } => {
            if let Some(annotation) = doc.annotation_mut(*id) {
                annotation.anchor = *anchor;
            }
            Ok(())
        }
    }
}