// src/command/character_cmd.rs

//! Comandi su `Character`: creazione, rimozione, nome/alias (scollegati
//! di proposito, vedi `CharacterCommand::Rename`), attore, note di
//! costume/entrata/uscita. Parte del raggruppamento per dominio
//! descritto in cima a `commands::command`.

use crate::core::Character;
use crate::errors::{CoreError, CoreResult};
use crate::lake::Document;
use crate::types::{CharacterAlias, ElementId, Timestamp};

use crate::commands::command::ChangeKind;

#[derive(Debug, PartialEq)]
pub enum CharacterCommand {
    Insert {
        id: ElementId,
        name: String,
        alias: CharacterAlias,
    },
    Remove {
        id: ElementId,
    },
    /// Cambia `Character.name` (il nome per esteso), non l'alias: i due
    /// campi sono deliberatamente scollegati — correggere un errore nel
    /// nome non deve forzare una decisione sull'alias, che resta una
    /// scorciatoia stabile scelta a mano. Un form che vuole cambiare
    /// entrambi in un colpo manda questo comando e `RenameAlias` nello
    /// stesso batch: l'atomicità arriva gratis dal bus, non serve un
    /// comando combinato.
    Rename {
        id: ElementId,
        new_name: String,
    },
    RenameAlias {
        id: ElementId,
        new_alias: CharacterAlias,
    },
    SetActor {
        id: ElementId,
        actor: Option<String>,
    },
    SetCostumeNotes {
        id: ElementId,
        notes: Option<String>,
    },
    SetEntranceNote {
        id: ElementId,
        note: Option<String>,
    },
    SetExitNote {
        id: ElementId,
        note: Option<String>,
    },
}

#[derive(Debug, PartialEq)]
pub(crate) enum CharacterUndo {
    Remove(ElementId),
    Reinsert(Character),
    Rename { id: ElementId, name: String },
    RenameAlias { id: ElementId, alias: CharacterAlias },
    SetActor { id: ElementId, actor: Option<String> },
    SetCostumeNotes { id: ElementId, notes: Option<String> },
    SetEntranceNote { id: ElementId, note: Option<String> },
    SetExitNote { id: ElementId, note: Option<String> },
}

pub(crate) fn classify(cmd: &CharacterCommand) -> (ElementId, ChangeKind) {
    match cmd {
        CharacterCommand::Insert { id, .. } => (*id, ChangeKind::Created),
        CharacterCommand::Remove { id } => (*id, ChangeKind::Removed),
        CharacterCommand::Rename { id, .. }
        | CharacterCommand::RenameAlias { id, .. }
        | CharacterCommand::SetActor { id, .. }
        | CharacterCommand::SetCostumeNotes { id, .. }
        | CharacterCommand::SetEntranceNote { id, .. }
        | CharacterCommand::SetExitNote { id, .. } => (*id, ChangeKind::Updated),
    }
}

pub(crate) fn cascaded_ids(_undo: &CharacterUndo) -> Vec<ElementId> {
    Vec::new()
}

pub(crate) fn apply(cmd: &CharacterCommand, doc: &mut Document, at: Timestamp) -> CoreResult<CharacterUndo> {
    match cmd {
        CharacterCommand::Insert { id, name, alias } => {
            let character = Character::with_id(*id, name.clone(), alias.clone(), at);
            doc.insert_character(character)?;
            Ok(CharacterUndo::Remove(*id))
        }
        CharacterCommand::Remove { id } => {
            let removed = doc.remove_character(*id).ok_or(CoreError::ElementNotFound(*id))?;
            Ok(CharacterUndo::Reinsert(removed))
        }
        CharacterCommand::Rename { id, new_name } => {
            let character = doc.character_mut(*id).ok_or(CoreError::ElementNotFound(*id))?;
            let previous = character.name.clone();
            character.name = new_name.clone();
            Ok(CharacterUndo::Rename { id: *id, name: previous })
        }
        CharacterCommand::RenameAlias { id, new_alias } => {
            let previous = doc
                .character(*id)
                .ok_or(CoreError::ElementNotFound(*id))?
                .alias()
                .clone();
            doc.rename_character_alias(*id, new_alias.clone())?;
            Ok(CharacterUndo::RenameAlias { id: *id, alias: previous })
        }
        CharacterCommand::SetActor { id, actor } => {
            let character = doc.character_mut(*id).ok_or(CoreError::ElementNotFound(*id))?;
            let previous = character.actor.clone();
            character.actor = actor.clone();
            Ok(CharacterUndo::SetActor { id: *id, actor: previous })
        }
        CharacterCommand::SetCostumeNotes { id, notes } => {
            let character = doc.character_mut(*id).ok_or(CoreError::ElementNotFound(*id))?;
            let previous = character.costume_notes.clone();
            character.costume_notes = notes.clone();
            Ok(CharacterUndo::SetCostumeNotes { id: *id, notes: previous })
        }
        CharacterCommand::SetEntranceNote { id, note } => {
            let character = doc.character_mut(*id).ok_or(CoreError::ElementNotFound(*id))?;
            let previous = character.entrance_notes.clone();
            character.entrance_notes = note.clone();
            Ok(CharacterUndo::SetEntranceNote { id: *id, note: previous })
        }
        CharacterCommand::SetExitNote { id, note } => {
            let character = doc.character_mut(*id).ok_or(CoreError::ElementNotFound(*id))?;
            let previous = character.exit_notes.clone();
            character.exit_notes = note.clone();
            Ok(CharacterUndo::SetExitNote { id: *id, note: previous })
        }
    }
}

pub(crate) fn revert(undo: &CharacterUndo, doc: &mut Document) -> CoreResult<()> {
    match undo {
        CharacterUndo::Remove(id) => {
            doc.remove_character(*id);
            Ok(())
        }
        CharacterUndo::Reinsert(character) => {
            doc.insert_character(character.clone())?;
            Ok(())
        }
        CharacterUndo::Rename { id, name } => {
            if let Some(character) = doc.character_mut(*id) {
                character.name = name.clone();
            }
            Ok(())
        }
        CharacterUndo::RenameAlias { id, alias } => {
            doc.rename_character_alias(*id, alias.clone())?;
            Ok(())
        }
        CharacterUndo::SetActor { id, actor } => {
            if let Some(character) = doc.character_mut(*id) {
                character.actor = actor.clone();
            }
            Ok(())
        }
        CharacterUndo::SetCostumeNotes { id, notes } => {
            if let Some(character) = doc.character_mut(*id) {
                character.costume_notes = notes.clone();
            }
            Ok(())
        }
        CharacterUndo::SetEntranceNote { id, note } => {
            if let Some(character) = doc.character_mut(*id) {
                character.entrance_notes = note.clone();
            }
            Ok(())
        }
        CharacterUndo::SetExitNote { id, note } => {
            if let Some(character) = doc.character_mut(*id) {
                character.exit_notes = note.clone();
            }
            Ok(())
        }
    }
}