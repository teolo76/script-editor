// src/lake/character_store.rs

//! Arena specializzato per `Character`: oltre alla memorizzazione per id
//! (come `EntityStore`), mantiene un indice `alias -> ElementId` e ne fa
//! rispettare l'unicità. È il motivo per cui non basta un
//! `EntityStore<Character>` semplice: l'unicità dell'alias è una regola
//! sulla collezione, non sul singolo personaggio, quindi deve viverci
//! sopra, non dentro `Character`.
use std::collections::HashMap;

// Modules Package
use crate::core::{ Character, Entity};
use crate::errors::{ CoreError, CoreResult };
use crate::lake::EntityStore;
use crate::types::{ CharacterAlias, ElementId };

#[derive(Default)]
pub struct CharacterStore {
    items: EntityStore<Character>,
    by_alias: HashMap<CharacterAlias, ElementId>,
}

impl CharacterStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserisce un nuovo personaggio. Fallisce se l'alias è già in uso da
    /// un personaggio diverso: l'unicità è quello che rende l'alias
    /// utilizzabile in fase di stesura senza ambiguità.
    pub fn insert(&mut self, character: Character) -> CoreResult<ElementId> {
        if let Some(&existing_id) = self.by_alias.get(character.alias()) {
            if existing_id != character.id() {
                return Err(CoreError::DuplicateAlias(character.alias().clone()));
            }
        }

        let id = character.id();
        let alias = character.alias().clone();

        self.items.insert(character);
        self.by_alias.insert(alias, id);
        Ok(id)
    }

    pub fn get(&self, id: ElementId) -> Option<&Character> {
        self.items.get(id)
    }

    /// Accesso mutabile a un personaggio. Non permette di cambiare
    /// l'alias (il campo è privato fuori da `core::character`): per
    /// rinominarlo serve `rename_alias`, che aggiorna anche l'indice.
    pub fn get_mut(&mut self, id: ElementId) -> Option<&mut Character> {
        self.items.get_mut(id)
    }

    pub fn remove(&mut self, id: ElementId) -> Option<Character> {
        let removed = self.items.remove(id)?;
        self.by_alias.remove(removed.alias());
        Some(removed)
    }

    /// Rinomina l'alias di un personaggio già inserito, mantenendo
    /// l'indice coerente. Fallisce se il nuovo alias è già usato da un
    /// altro personaggio; l'`ElementId` del blocco `CharacterName` che lo
    /// referenzia resta valido in ogni caso, perché punta all'id, non
    /// all'alias.
    pub fn rename_alias(&mut self, id: ElementId, new: CharacterAlias) -> CoreResult<()> {
        if let Some(&existing_id) = self.by_alias.get(&new) {
            if existing_id != id {
                return Err(CoreError::DuplicateAlias(new));
            }
        }
        let character = self
            .items
            .get_mut(id)
            .ok_or(CoreError::ElementNotFound(id))?;

        let old = character.alias().clone();
        character.set_alias(new.clone());
        self.by_alias.remove(&old);
        self.by_alias.insert(new, id);

        Ok(())
    }

    /// Risolve un alias digitato in fase di stesura nell'id del
    /// personaggio corrispondente.
    pub fn resolve_alias(&self, alias: &CharacterAlias) -> Option<ElementId> {
        self.by_alias.get(alias).copied()
    }

    pub fn iter(&self) -> impl Iterator<Item=&Character> {
        self.items.iter()
    }

    /// Personaggi senza attore assegnato.
    pub fn ghosts(&self) -> impl Iterator<Item=&Character> {
        self.items.iter().filter(|c| c.is_ghost())
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Timestamp;

    fn alias(s: &str) -> CharacterAlias {
        CharacterAlias::new(s).unwrap()
    }

    fn t() -> Timestamp {
        Timestamp::from_unix_seconds(0)
    }

    #[test]
    fn insert_and_resolve_alias() {
        let mut store = CharacterStore::new();
        let character = Character::new("Giulietta", alias("GIU"), t());
        let id = store.insert(character).unwrap();
        assert_eq!(store.resolve_alias(&alias("GIU")), Some(id));
        assert_eq!(store.resolve_alias(&alias("giu")), Some(id)); // normalizzato in maiuscolo
    }

    #[test]
    fn insert_rejects_duplicate_alias() {
        let mut store = CharacterStore::new();
        store.insert(Character::new("Giulietta", alias("GIU"), t())).unwrap();
        let err = store
            .insert(Character::new("Giuseppe", alias("GIU"), t()))
            .unwrap_err();
        assert_eq!(err, CoreError::DuplicateAlias(alias("GIU")));
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn rename_alias_updates_index() {
        let mut store = CharacterStore::new();
        let id = store.insert(Character::new("Romeo", alias("ROM"), t())).unwrap();

        store.rename_alias(id, alias("ROX")).unwrap();

        assert_eq!(store.resolve_alias(&alias("ROM")), None);
        assert_eq!(store.resolve_alias(&alias("ROX")), Some(id));
        assert_eq!(store.get(id).unwrap().alias().as_str(), "ROX");
    }

    #[test]
    fn rename_alias_rejects_collision_with_another_character() {
        let mut store = CharacterStore::new();
        let romeo = store.insert(Character::new("Romeo", alias("ROM"), t())).unwrap();
        store.insert(Character::new("Rosalina", alias("ROS"), t())).unwrap();

        let err = store.rename_alias(romeo, alias("ROS")).unwrap_err();
        assert_eq!(err, CoreError::DuplicateAlias(alias("ROS")));
        // L'alias originale di Romeo deve restare intatto dopo il fallimento.
        assert_eq!(store.get(romeo).unwrap().alias().as_str(), "ROM");
    }

    #[test]
    fn remove_clears_alias_index() {
        let mut store = CharacterStore::new();
        let id = store.insert(Character::new("Romeo", alias("ROM"), t())).unwrap();
        store.remove(id);
        assert_eq!(store.resolve_alias(&alias("ROM")), None);
    }

    #[test]
    fn ghosts_filters_characters_without_actor() {
        let mut store = CharacterStore::new();
        let mut with_actor = Character::new("Romeo", alias("ROM"), t());
        with_actor.actor = Some("Mario Rossi".to_string());
        store.insert(with_actor).unwrap();
        store.insert(Character::new("Voce fuori campo", alias("VOC"), t())).unwrap();

        let ghosts: Vec<&str> = store.ghosts().map(|c| c.name.as_str()).collect();
        assert_eq!(ghosts, vec!["Voce fuori campo"]);
    }
}