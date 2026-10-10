// src/core/character.rs

//! Character: personaggio del copione. È un registro piatto come `Layer`
//! (niente `parent`/`sort_order`, non vive nell'albero) ma con un vincolo
//! di dominio in più: l'alias deve essere univoco fra tutti i personaggi.
//! Quel vincolo non vive qui — `Character` resta un semplice contenitore
//! di dati — ma in `lake::CharacterStore`, perché è un fatto sulla
//! collezione, non sul singolo personaggio.
use crate::core::{ Meta, Entity };
use crate::types::{CharacterAlias, ElementId, Timestamp};

#[derive(Debug, Clone, PartialEq)]
pub struct Character {
    meta: Meta,
    /// Privato apposta: l'unicità dell'alias è un vincolo sulla collezione,
    /// quindi ogni rinomina deve passare da
    /// `lake::CharacterStore::rename_alias`, mai da una mutazione diretta
    /// del campo (che qui, non essendo pubblico, non è nemmeno possibile
    /// fuori da questo modulo).
    alias: CharacterAlias,
    pub name: String,
    /// Nome dell'attore che lo interpreta. `None` segnala un personaggio
    /// "fantasma": presente nel copione (citato, con battute) ma senza un
    /// attore assegnato — utile per scovarli prima che sfuggano al
    /// casting.
    pub actor: Option<String>,
    pub costume_notes: Option<String>,
    pub entrance_notes: Option<String>,
    pub exit_notes: Option<String>,
}

impl Character {
    pub fn new(name: impl Into<String>, alias: CharacterAlias, at: Timestamp) -> Self {
        Self {
            meta: Meta::new(at),
            name: name.into(),
            alias,
            actor: None,
            costume_notes: None,
            entrance_notes: None,
            exit_notes: None,
        }
    }

    /// Come `new`, ma con id esplicito: usato dal command bus per il redo,
    /// che deve ricreare il personaggio con lo stesso id della prima
    /// applicazione.
    pub fn with_id(id: ElementId, name: impl Into<String>, alias: CharacterAlias, at: Timestamp) -> Self {
        Self {
            meta: Meta::with_id(id, at),
            name: name.into(),
            alias,
            actor: None,
            costume_notes: None,
            entrance_notes: None,
            exit_notes: None,
        }
    }

    pub fn alias(&self) -> &CharacterAlias {
        &self.alias
    }

    /// Cambia l'alias senza passare dal controllo di unicità. Visibile
    /// solo all'interno del crate: va chiamato esclusivamente da
    /// `lake::CharacterStore::rename_alias`, che aggiorna anche l'indice
    /// `alias -> ElementId`. Chiamarlo altrove disallineerebbe l'indice.
    pub(crate) fn set_alias(&mut self, alias: CharacterAlias) {
        self.alias = alias;
    }

    /// Un personaggio senza attore assegnato ("fantasma").
    pub fn is_ghost(&self) -> bool {
        self.actor.is_none()
    }
}

impl Entity for Character {
    fn meta(&self) -> &Meta {
        &self.meta
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CharacterAlias;

    fn alias(s: &str) -> CharacterAlias {
        CharacterAlias::new(s).unwrap()
    }

    #[test]
    fn new_character_is_ghost_until_actor_assigned() {
        let mut character = Character::new("Giulietta", alias("GIU"), Timestamp::from_unix_seconds(0));
        assert!(character.is_ghost());
        character.actor = Some("Attrice Reale".to_string());
        assert!(!character.is_ghost());
    }

    #[test]
    fn alias_getter_reflects_constructor() {
        let character = Character::new("Romeo", alias("ROM"), Timestamp::from_unix_seconds(0));
        assert_eq!(character.alias().as_str(), "ROM");
    }
}