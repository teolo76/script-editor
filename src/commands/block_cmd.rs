// src/command/block_cmd.rs

//! Comandi su `Block`: creazione, rimozione (a cascata), spostamento
//! nell'albero, ed editing testuale (`InsertText`/`DeleteText`) con lo
//! spostamento coerente delle `Annotation` di tipo `TextRange` ancorate
//! al blocco. Parte del raggruppamento per dominio descritto in cima a
//! `commands::command`.

use crate::core::{AnchorTarget, Annotation, Block, BlockPayload, Entity, LayerId, TreeElement};
use crate::errors::{CoreError, CoreResult};
use crate::lake::Document;
use crate::types::{ElementId, SortKey, Timestamp};

use crate::commands::command::ChangeKind;

#[derive(Debug, PartialEq)]
pub enum BlockCommand {
    /// `parent` non è opzionale, stessa ragione di `NodeCommand::Insert`:
    /// un Block non è mai il Root, quindi ha sempre un parent concreto.
    Insert {
        id: ElementId,
        payload: BlockPayload,
        layer_id: LayerId,
        parent: ElementId,
        sort_order: SortKey,
    },
    Remove {
        id: ElementId,
    },
    Move {
        id: ElementId,
        new_parent: ElementId,
        new_sort_order: SortKey,
    },
    /// Inserisce testo in un `TextBlock` a un offset di *carattere* (non
    /// byte — coerente con `AnchorTarget::TextRange`). Ogni `Annotation`
    /// con ancora `TextRange` su questo blocco viene spostata di
    /// conseguenza: un bordo (`start` o `end`) che cade a `offset` o dopo
    /// avanza di `text.chars().count()`; un bordo prima di `offset` resta
    /// fermo. Inserire esattamente al bordo finale di un intervallo lo
    /// estende (il testo nuovo entra nell'annotazione); inserire
    /// esattamente al bordo iniziale lo spinge via (il testo nuovo resta
    /// fuori) — è una scelta di convenzione, non un vincolo del modello.
    InsertText {
        block_id: ElementId,
        offset: usize,
        text: String,
    },
    /// Rimuove il testo di un `TextBlock` nell'intervallo di carattere
    /// `[start, end)`. Le `Annotation` con ancora `TextRange` su questo
    /// blocco vengono ridotte di conseguenza; un'annotazione interamente
    /// contenuta nell'intervallo rimosso collassa a un punto invece di
    /// essere eliminata: il contenuto dell'annotazione (`body`) non è
    /// testo del copione, non va perso solo perché il testo a cui
    /// puntava è stato tagliato — tocca all'editor segnalarla come
    /// "ancora da rivedere", sullo stesso principio di
    /// `TextBlock::character_id` non risolto.
    DeleteText {
        block_id: ElementId,
        start: usize,
        end: usize,
    },
}

#[derive(Debug, PartialEq)]
pub(crate) enum BlockUndo {
    /// Inverso di `Insert`, stesso ragionamento di `NodeUndo::Remove`:
    /// nessuna cascata al momento dell'inserimento.
    Remove(ElementId),
    /// Inverso di `Remove`. Un Block non ha mai figli nel nostro modello,
    /// quindi qui basta l'elenco delle annotazioni: nessun sottoalbero
    /// da ripristinare.
    RemoveCascade {
        block: Block,
        annotations: Vec<Annotation>,
    },
    Move {
        id: ElementId,
        parent: Option<ElementId>,
        sort_order: SortKey,
    },
    /// Inverso di `InsertText`: rimuove esattamente i caratteri appena
    /// inseriti (`[offset, offset + len)`) e applica agli
    /// `AnchorTarget::TextRange` del blocco la stessa formula di
    /// riduzione usata da `DeleteText` su quell'intervallo. L'inserimento
    /// non perde mai informazione sui bordi (ogni bordo si limita a
    /// spostarsi, mai a collassare), quindi questa formula è l'inverso
    /// esatto — non serve un'istantanea delle annotazioni come per
    /// `DeleteText` qui sotto. `affected` è solo per `cascaded_ids`
    /// (riportare nel `Delta` quali annotazioni sono state toccate).
    InsertText {
        block_id: ElementId,
        offset: usize,
        len: usize,
        affected: Vec<ElementId>,
    },
    /// Inverso di `DeleteText`: a differenza di `InsertText`, qui serve
    /// un'istantanea completa — un'annotazione interamente dentro il
    /// testo rimosso collassa a un punto durante `apply`, e da un punto
    /// solo non si può ricostruire l'intervallo originale. `text` è il
    /// testo rimosso (da reinserire a `offset`), `annotations` sono le
    /// coppie `(id, start, end)` precedenti di ogni annotazione
    /// `TextRange` toccata su questo blocco, da ripristinare alla
    /// lettera.
    DeleteText {
        block_id: ElementId,
        offset: usize,
        text: String,
        annotations: Vec<(ElementId, usize, usize)>,
    },
}

pub(crate) fn classify(cmd: &BlockCommand) -> (ElementId, ChangeKind) {
    match cmd {
        BlockCommand::Insert { id, .. } => (*id, ChangeKind::Created),
        BlockCommand::Remove { id } => (*id, ChangeKind::Removed),
        BlockCommand::Move { id, .. } => (*id, ChangeKind::Updated),
        BlockCommand::InsertText { block_id, .. } | BlockCommand::DeleteText { block_id, .. } => {
            (*block_id, ChangeKind::Updated)
        }
    }
}

pub(crate) fn cascaded_ids(undo: &BlockUndo) -> Vec<ElementId> {
    match undo {
        BlockUndo::RemoveCascade { annotations, .. } => annotations.iter().map(|a| a.id()).collect(),
        // `InsertText`/`DeleteText` sono `Updated` sul blocco (vedi
        // `classify`), e le annotazioni che toccano sono anch'esse solo
        // spostate, non create o rimosse: condividono lo stesso
        // `ChangeKind::Updated`.
        BlockUndo::DeleteText { annotations, .. } => annotations.iter().map(|(id, ..)| *id).collect(),
        BlockUndo::InsertText { affected, .. } => affected.clone(),
        _ => Vec::new(),
    }
}

/// Offset-byte del bordo di carattere numero `char_offset` in `s` (0 =
/// inizio stringa, `s.chars().count()` = fine stringa). `None` se
/// `char_offset` supera la lunghezza in caratteri. Unico punto di
/// conversione fra gli indici di carattere di `AnchorTarget::TextRange`
/// e l'indicizzazione per byte di `String`.
fn char_byte_offset(s: &str, char_offset: usize) -> Option<usize> {
    s.char_indices().map(|(b, _)| b).chain(std::iter::once(s.len())).nth(char_offset)
}

/// Inserisce `insert` in `text` all'offset di carattere `offset`.
/// `None` se `offset` supera la lunghezza in caratteri di `text`.
fn insert_text_at(text: &mut String, offset: usize, insert: &str) -> Option<()> {
    let byte_offset = char_byte_offset(text, offset)?;
    text.insert_str(byte_offset, insert);
    Some(())
}

/// Rimuove da `text` l'intervallo di carattere `[start, end)` e
/// restituisce il testo rimosso. `None` se `start > end` o se `end`
/// supera la lunghezza in caratteri di `text`.
fn delete_text_at(text: &mut String, start: usize, end: usize) -> Option<String> {
    if start > end {
        return None;
    }
    let byte_start = char_byte_offset(text, start)?;
    let byte_end = char_byte_offset(text, end)?;
    let removed = text[byte_start..byte_end].to_string();
    text.replace_range(byte_start..byte_end, "");
    Some(removed)
}

/// Bordo (`start` o `end`) di un `AnchorTarget::TextRange` dopo
/// l'inserimento di `inserted_len` caratteri all'offset `at`: un bordo a
/// `at` o dopo avanza; un bordo prima di `at` resta fermo.
fn shift_bound_on_insert(bound: usize, at: usize, inserted_len: usize) -> usize {
    if bound >= at {
        bound + inserted_len
    } else {
        bound
    }
}

/// Bordo (`start` o `end`) di un `AnchorTarget::TextRange` dopo la
/// rimozione dell'intervallo di carattere `[removed_start, removed_end)`:
/// un bordo prima della rimozione resta fermo, uno dopo retrocede della
/// lunghezza rimossa, uno all'interno collassa sull'inizio della
/// rimozione. Usata sia da `DeleteText` in avanti sia da `InsertText`
/// all'indietro.
fn shift_bound_on_delete(bound: usize, removed_start: usize, removed_end: usize) -> usize {
    if bound <= removed_start {
        bound
    } else if bound >= removed_end {
        bound - (removed_end - removed_start)
    } else {
        removed_start
    }
}

pub(crate) fn apply(cmd: &BlockCommand, doc: &mut Document, at: Timestamp) -> CoreResult<BlockUndo> {
    match cmd {
        BlockCommand::Insert { id, payload, layer_id, parent, sort_order } => {
            let block = Block::with_id(*id, payload.clone(), *layer_id, Some(*parent), sort_order.clone(), at);
            doc.insert_block(block);
            Ok(BlockUndo::Remove(*id))
        }
        BlockCommand::Remove { id } => {
            let removed_annotations: Vec<Annotation> =
                doc.annotations_for(*id).into_iter().cloned().collect();
            for ann in &removed_annotations {
                doc.remove_annotation(ann.id());
            }
            let block = doc.remove_block(*id).ok_or(CoreError::ElementNotFound(*id))?;
            Ok(BlockUndo::RemoveCascade { block, annotations: removed_annotations })
        }
        BlockCommand::Move { id, new_parent, new_sort_order } => {
            if doc.node(*new_parent).is_none() {
                return Err(CoreError::ElementNotFound(*new_parent));
            }
            let block = doc.block(*id).ok_or(CoreError::ElementNotFound(*id))?;
            let previous_parent = block.parent();
            let previous_sort_order = block.sort_order().clone();
            doc.block_mut(*id)
                .expect("verificato sopra")
                .reparent(Some(*new_parent), new_sort_order.clone());
            Ok(BlockUndo::Move { id: *id, parent: previous_parent, sort_order: previous_sort_order })
        }
        BlockCommand::InsertText { block_id, offset, text } => {
            let block = doc.block_mut(*block_id).ok_or(CoreError::ElementNotFound(*block_id))?;
            let text_block = match &mut block.payload {
                BlockPayload::Text(t) => t,
                BlockPayload::Asset(_) => return Err(CoreError::NotATextBlock(*block_id)),
            };
            let len_before = text_block.text.chars().count();
            insert_text_at(&mut text_block.text, *offset, text).ok_or(CoreError::TextOffsetOutOfBounds {
                block_id: *block_id,
                offset: *offset,
                len: len_before,
            })?;
            let inserted_len = text.chars().count();

            // Nessuna istantanea necessaria: lo spostamento dei bordi non
            // perde mai informazione (vedi `BlockUndo::InsertText`),
            // quindi l'inverso si ricalcola da `offset`/`inserted_len`.
            let affected: Vec<ElementId> = doc
                .annotations_for(*block_id)
                .iter()
                .filter(|a| matches!(a.anchor, AnchorTarget::TextRange { block_id: b, .. } if b == *block_id))
                .map(|a| a.id())
                .collect();
            for ann_id in affected.iter().copied() {
                let annotation = doc.annotation_mut(ann_id).expect("appena trovata sopra");
                if let AnchorTarget::TextRange { block_id: b, start, end } = annotation.anchor {
                    annotation.anchor = AnchorTarget::TextRange {
                        block_id: b,
                        start: shift_bound_on_insert(start, *offset, inserted_len),
                        end: shift_bound_on_insert(end, *offset, inserted_len),
                    };
                }
            }

            Ok(BlockUndo::InsertText { block_id: *block_id, offset: *offset, len: inserted_len, affected })
        }
        BlockCommand::DeleteText { block_id, start, end } => {
            let block = doc.block_mut(*block_id).ok_or(CoreError::ElementNotFound(*block_id))?;
            let text_block = match &mut block.payload {
                BlockPayload::Text(t) => t,
                BlockPayload::Asset(_) => return Err(CoreError::NotATextBlock(*block_id)),
            };
            let len_before = text_block.text.chars().count();
            let removed = delete_text_at(&mut text_block.text, *start, *end).ok_or(CoreError::InvalidTextRange {
                block_id: *block_id,
                start: *start,
                end: *end,
                len: len_before,
            })?;

            // Qui serve un'istantanea (vedi `BlockUndo::DeleteText`): una
            // riduzione può far collassare due bordi distinti sullo
            // stesso punto, e da lì non si torna indietro senza aver
            // registrato i valori originali.
            let affected: Vec<ElementId> = doc
                .annotations_for(*block_id)
                .iter()
                .filter(|a| matches!(a.anchor, AnchorTarget::TextRange { block_id: b, .. } if b == *block_id))
                .map(|a| a.id())
                .collect();
            let mut previous_ranges = Vec::with_capacity(affected.len());
            for ann_id in affected {
                let annotation = doc.annotation_mut(ann_id).expect("appena trovata sopra");
                if let AnchorTarget::TextRange { block_id: b, start: prev_start, end: prev_end } =
                    annotation.anchor
                {
                    previous_ranges.push((ann_id, prev_start, prev_end));
                    annotation.anchor = AnchorTarget::TextRange {
                        block_id: b,
                        start: shift_bound_on_delete(prev_start, *start, *end),
                        end: shift_bound_on_delete(prev_end, *start, *end),
                    };
                }
            }

            Ok(BlockUndo::DeleteText { block_id: *block_id, offset: *start, text: removed, annotations: previous_ranges })
        }
    }
}

pub(crate) fn revert(undo: &BlockUndo, doc: &mut Document) -> CoreResult<()> {
    match undo {
        BlockUndo::Remove(id) => {
            doc.remove_block(*id);
            Ok(())
        }
        BlockUndo::RemoveCascade { block, annotations } => {
            doc.insert_block(block.clone());
            for a in annotations {
                doc.insert_annotation(a.clone());
            }
            Ok(())
        }
        BlockUndo::Move { id, parent, sort_order } => {
            if let Some(block) = doc.block_mut(*id) {
                block.reparent(*parent, sort_order.clone());
            }
            Ok(())
        }
        BlockUndo::InsertText { block_id, offset, len, .. } => {
            if let Some(block) = doc.block_mut(*block_id) {
                if let BlockPayload::Text(text_block) = &mut block.payload {
                    delete_text_at(&mut text_block.text, *offset, *offset + *len);
                }
            }
            let affected: Vec<ElementId> = doc
                .annotations_for(*block_id)
                .iter()
                .filter(|a| matches!(a.anchor, AnchorTarget::TextRange { block_id: b, .. } if b == *block_id))
                .map(|a| a.id())
                .collect();
            for ann_id in affected {
                if let Some(annotation) = doc.annotation_mut(ann_id) {
                    if let AnchorTarget::TextRange { block_id: b, start, end } = annotation.anchor {
                        annotation.anchor = AnchorTarget::TextRange {
                            block_id: b,
                            start: shift_bound_on_delete(start, *offset, *offset + *len),
                            end: shift_bound_on_delete(end, *offset, *offset + *len),
                        };
                    }
                }
            }
            Ok(())
        }
        BlockUndo::DeleteText { block_id, offset, text, annotations } => {
            if let Some(block) = doc.block_mut(*block_id) {
                if let BlockPayload::Text(text_block) = &mut block.payload {
                    insert_text_at(&mut text_block.text, *offset, text);
                }
            }
            for (ann_id, start, end) in annotations {
                if let Some(annotation) = doc.annotation_mut(*ann_id) {
                    if let AnchorTarget::TextRange { block_id: b, .. } = annotation.anchor {
                        annotation.anchor = AnchorTarget::TextRange { block_id: b, start: *start, end: *end };
                    }
                }
            }
            Ok(())
        }
    }
}