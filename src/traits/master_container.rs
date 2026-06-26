//! src/traits/master_container.rs

use uuid::Uuid;
use serde::Serialize;

// Package Modules & Files
use crate::core::SwitchBoard;
use crate::traits::BaseElement;

/// Contratto speciale e polimorfico che governa l'elemento radice dell'opera (Script).
/// Gestisce l'anagrafe del cast, il controllo di versione e l'ambiente globale.
pub trait MasterContainer: BaseElement + Serialize {
    /// Restituisce la Major Version corrente
    fn major_version(&self) -> u32;
    /// Restituisce la Minor Version corrente
    fn minor_version(&self) -> u32;
    /// Incrementa il numero di revisione globale del documento
    fn increment_version(&mut self);
    /// Incrementa il numero di versione minore del documento
    fn increment_minor_version(&mut self);
    /// Restituisce il nome dell'autore o dei co-autori detentori del progetto
    fn author(&self) -> &str;
    /// Imposta l'autore o aggiorna i crediti del progetto
    fn set_author(&mut self, author: &str);
    /// Fornisce un riferimento immutabile allo SwitchBoard d'ambiente
    fn switchboard(&self) -> &SwitchBoard;
    /// Fornisce un riferimento mutabile allo SwitchBoard per attivare/disattivare canali
    fn switchboard_mut(&mut self) -> &mut SwitchBoard;
    /// Restituisce l'elenco (slice) degli UUID di tutti i personaggi censiti ufficialmente nel Cast
    fn cast(&self) -> &[Uuid];
    /// Registra formalmente un nuovo personaggio nell'anagrafe centrale del Cast
    fn register_character(&mut self, character_id: Uuid) -> Result<(), &'static str>;
    /// Rimuove un personaggio dall'anagrafe centrale del Cast
    fn unregister_character(&mut self, character_id: &Uuid);
    /// Interroga la presenza di una copertina associata (predisposizione futura)
    fn has_cover(&self) -> bool;
}