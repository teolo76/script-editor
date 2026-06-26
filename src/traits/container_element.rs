//! src/traits/container_element.rs

use uuid::Uuid;
use serde::Serialize;

// Package Files & Modules
use crate::traits::BaseElement;

/// Unifica il comportamento anagrafico e strutturale di tutti i contenitori (Script, Act, Scene).
/// Richiede che l'oggetto sia tracciabile (Component) e serializzabile.
pub trait ContainerElement: BaseElement + Serialize {
    /// Restituisce il titolo del contenitore (es. Titolo Scena o Titolo Copione)
    fn title(&self) -> &str;
    /// Modifica il titolo e aggiorna il timestamp di modifica (`updated_at`)
    fn set_title(&mut self, new_title: &str);
    /// Restituisce il numero progressivo visivo (o la versione del file nel caso dello Script)
    fn number(&self) -> u32;
    /// Modifica la numerazione visiva o la versione del componente
    fn set_number(&mut self, new_number: u32);
    /// Restituisce la lista di UUID dei personaggi associati a questo specifico livello narrativo
    fn characters(&self) -> &[Uuid];
    /// Associa un nuovo personaggio al contenitore tramite il suo UUID
    fn add_character(&mut self, character_id: Uuid);
    /// Rimuove un personaggio dal contenitore
    fn remove_character(&mut self, character_id: &Uuid);
}