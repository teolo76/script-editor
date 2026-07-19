// src/traits/block_behaviour.rs

/// Descrive le facoltà atomiche di gestione del testo e del comportamento geometrico.
pub trait BlockBehavior {
    /// Identifica se l'elemento segue il flusso lineare del testo o ha vincoli geometrici flottanti.
    fn is_linear(&self) -> bool;

    /// Restituisce il contenuto testuale grezzo del blocco.
    fn raw_text(&self) -> &str;

    /// Sostituisce o aggiorna il testo contenuto.
    fn set_raw_text(&mut self, text: String);
}