// src/command/layer_cmd.rs

//! Comandi su `Layer`: creazione, rimozione, e i tre switch
//! visibilità/lock/printable più il riordino. Parte del raggruppamento
//! per dominio descritto in cima a `commands::command`.

use crate::core::Layer;
use crate::errors::{CoreError, CoreResult};
use crate::lake::Document;
use crate::types::{Color, ElementId, SortKey, Timestamp};

use crate::commands::command::ChangeKind;

#[derive(Debug, PartialEq)]
pub enum LayerCommand {
    Insert {
        id: ElementId,
        name: String,
        color: Color,
        order: SortKey,
    },
    Remove {
        id: ElementId,
    },
    SetVisible {
        id: ElementId,
        visible: bool,
    },
    SetLocked {
        id: ElementId,
        locked: bool,
    },
    SetPrintable {
        id: ElementId,
        printable: bool,
    },
    SetOrder {
        id: ElementId,
        new_order: SortKey,
    },
}

#[derive(Debug, PartialEq)]
pub(crate) enum LayerUndo {
    Remove(ElementId),
    Reinsert(Layer),
    SetVisible { id: ElementId, visible: bool },
    SetLocked { id: ElementId, locked: bool },
    SetPrintable { id: ElementId, printable: bool },
    SetOrder { id: ElementId, order: SortKey },
}

pub(crate) fn classify(cmd: &LayerCommand) -> (ElementId, ChangeKind) {
    match cmd {
        LayerCommand::Insert { id, .. } => (*id, ChangeKind::Created),
        LayerCommand::Remove { id } => (*id, ChangeKind::Removed),
        LayerCommand::SetVisible { id, .. }
        | LayerCommand::SetLocked { id, .. }
        | LayerCommand::SetPrintable { id, .. }
        | LayerCommand::SetOrder { id, .. } => (*id, ChangeKind::Updated),
    }
}

pub(crate) fn cascaded_ids(_undo: &LayerUndo) -> Vec<ElementId> {
    Vec::new()
}

pub(crate) fn apply(cmd: &LayerCommand, doc: &mut Document, at: Timestamp) -> CoreResult<LayerUndo> {
    match cmd {
        LayerCommand::Insert { id, name, color, order } => {
            let layer = Layer::with_id(*id, name.clone(), *color, order.clone(), at);
            doc.insert_layer(layer);
            Ok(LayerUndo::Remove(*id))
        }
        LayerCommand::Remove { id } => {
            let removed = doc.remove_layer(*id).ok_or(CoreError::ElementNotFound(*id))?;
            Ok(LayerUndo::Reinsert(removed))
        }
        LayerCommand::SetVisible { id, visible } => {
            let layer = doc.layer_mut(*id).ok_or(CoreError::ElementNotFound(*id))?;
            let previous = layer.visible;
            layer.visible = *visible;
            Ok(LayerUndo::SetVisible { id: *id, visible: previous })
        }
        LayerCommand::SetLocked { id, locked } => {
            let layer = doc.layer_mut(*id).ok_or(CoreError::ElementNotFound(*id))?;
            let previous = layer.locked;
            layer.locked = *locked;
            Ok(LayerUndo::SetLocked { id: *id, locked: previous })
        }
        LayerCommand::SetPrintable { id, printable } => {
            let layer = doc.layer_mut(*id).ok_or(CoreError::ElementNotFound(*id))?;
            let previous = layer.printable;
            layer.printable = *printable;
            Ok(LayerUndo::SetPrintable { id: *id, printable: previous })
        }
        LayerCommand::SetOrder { id, new_order } => {
            let layer = doc.layer_mut(*id).ok_or(CoreError::ElementNotFound(*id))?;
            let previous = layer.order.clone();
            layer.order = new_order.clone();
            Ok(LayerUndo::SetOrder { id: *id, order: previous })
        }
    }
}

pub(crate) fn revert(undo: &LayerUndo, doc: &mut Document) -> CoreResult<()> {
    match undo {
        LayerUndo::Remove(id) => {
            doc.remove_layer(*id);
            Ok(())
        }
        LayerUndo::Reinsert(layer) => {
            doc.insert_layer(layer.clone());
            Ok(())
        }
        LayerUndo::SetVisible { id, visible } => {
            if let Some(layer) = doc.layer_mut(*id) {
                layer.visible = *visible;
            }
            Ok(())
        }
        LayerUndo::SetLocked { id, locked } => {
            if let Some(layer) = doc.layer_mut(*id) {
                layer.locked = *locked;
            }
            Ok(())
        }
        LayerUndo::SetPrintable { id, printable } => {
            if let Some(layer) = doc.layer_mut(*id) {
                layer.printable = *printable;
            }
            Ok(())
        }
        LayerUndo::SetOrder { id, order } => {
            if let Some(layer) = doc.layer_mut(*id) {
                layer.order = order.clone();
            }
            Ok(())
        }
    }
}