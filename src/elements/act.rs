//! src/elements/act.rs

use uuid::Uuid;
use serde::{ Serialize, Deserialize };

// Package Modules
use crate::core::{ Meta, VisualLayer };
use crate::traits:: {BaseElement, ContainerElement, LayeredElement};
use crate::common::constants::{
    DEFAULT_ACT_TITLE,
    MAX_SCENE_PER_ACT,
};

// Funzioni helper necessarie a Serde per inizializzare gli array fissi quando salta il JSON
fn default_scenes() -> [Uuid; MAX_SCENE_PER_ACT] { [Uuid::nil(); MAX_SCENE_PER_ACT] }

/// Rappresenta un macro-blocco narrativo (Atto) basato su indici.
/// Raccoglie le relazioni con le scene figlie e il cast dell'atto tramite UUID.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Hash)]
pub struct Act<'a> {
    pub meta: Meta,
    pub visual: VisualLayer,
    pub number: u32,
    pub title: &'a str,

    // Array statici per preservare il tratto Copy nello Stack
    #[serde(skip, default = "default_scenes")]
    pub scenes: [Uuid;MAX_SCENE_PER_ACT],
    pub scenes_count: usize,
}

impl<'a> Act<'a> {
    /// Costruttore standard per la creazione a runtime
    pub fn new(number: u32, title: &'a str) -> Self {
        // Calcoliamo il titolo direttamente qui per evitare conflitti di lifetime
        let mut final_title = title.trim();
        if final_title.is_empty() {
            if number == 0 {
                final_title = DEFAULT_ACT_TITLE;
            } else {
                final_title = "Atto"; // Verrà formattato dallo StyleManager (es. Atto I)
            }
        }
        Self {
            meta: Meta::new(),
            visual: VisualLayer::new(),
            number,
            title: final_title,
            scenes: [Uuid::nil(); MAX_SCENE_PER_ACT],
            scenes_count: 0,
        }
    }
    /// Costruttore simmetrico per il ripristino da I/O o database
    pub fn with_specs(meta: Meta, visual: VisualLayer, number: u32, title: &'a str) -> Self {
        // Calcoliamo il titolo direttamente qui per evitare conflitti di lifetime
        let mut final_title = title.trim();
        if final_title.is_empty() {
            if number == 0 {
                final_title = DEFAULT_ACT_TITLE;
            } else {
                final_title = "Atto"; // Verrà formattato dallo StyleManager (es. Atto I)
            }
        }
        Self {
            meta,
            visual,
            number,
            title: title.trim(),
            scenes: [Uuid::nil(); MAX_SCENE_PER_ACT],
            scenes_count: 0,
        }
    }
    /// Calcola il titolo di default in base alla semantica del numero di Atto
    fn resolve_default_title(number: u32, title: &'a str) -> &'a str {
        let trimmed = title.trim();

        // Se l'utente ha inserito un titolo valido, usiamo quello
        if !trimmed.is_empty() {
            return title;
        }
        // Altrimenti scatta la logica dei default di sistema
        if number == 0 {
            DEFAULT_ACT_TITLE
        } else {
            "Atto" // Lo StyleManager farà l'append del numero (es. Atto I)
        }
    }
    /// Aggancia una scena a questo atto tramite il suo UUID
    pub fn add_scene(&mut self, scene_id: Uuid) -> Result<(), &'static str> {
        if self.scenes_count >= 64 {
            return Err("Capacità massima di scene per questo atto raggiunta (64)");
        }
        self.scenes[self.scenes_count] = scene_id;
        self.scenes_count += 1;
        self.meta.touch();
        Ok(())
    }
    /// Restituisce gli ID delle scene figlie
    pub fn scene_ids(&self) -> &[Uuid] {
        &self.scenes[..self.scenes_count]
    }
}

// 1. Implementazione di BaseElement (Identità e Tempo)
impl<'a> BaseElement for Act<'a> {
    fn id(&self) -> Uuid { self.meta.id }
    fn created_at(&self) -> u64 { self.meta.created_at }
    fn updated_at(&self) -> u64 { self.meta.updated_at }
    fn touch(&mut self) { self.meta.touch(); }
}
// 2. Implementazione di ContainerElement (Gestione Anagrafica e Cast)
impl<'a> ContainerElement for Act<'a> {
    fn title(&self) -> &str { self.title }
    fn set_title(&mut self, _new_title: &str) { self.meta.touch(); }

    fn number(&self) -> u32 { self.number }
    fn set_number(&mut self, new_number: u32) {
        self.number = new_number;
        self.meta.touch();
    }
    /// L'Atto non memorizza più direttamente i personaggi per evitare disallineamenti.
    /// Restituisce una slice vuota. Il calcolo reale sarà delegato alle query dell'Engine.
    fn characters(&self) -> &[Uuid] { &[] }
    fn add_character(&mut self, _character_id: Uuid) {}
    fn remove_character(&mut self, _character_id: &Uuid) {}
}
// 3. Implementazione di LayeredElement  (Filtri Visualizzazione ed Ereditarietà Canali)
impl<'a> LayeredElement for Act<'a> {
    fn visual_layer(&self) -> &VisualLayer { &self.visual }
    fn visual_layer_mut(&mut self) -> &mut VisualLayer { &mut self.visual }
}

// =========================================================================
// UNIT TESTS DI VALIDAZIONE
// =========================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_act_clean_architecture_and_copy() {
        // Test logica Atto Unico con stringa vuota
        let mut act_unico = Act::new(0, "");
        assert_eq!(act_unico.title(), DEFAULT_ACT_TITLE);
        assert_eq!(act_unico.number(), 0);

        let scene_id = Uuid::new_v4();
        assert!(act_unico.add_scene(scene_id).is_ok());
        assert_eq!(act_unico.scene_ids().len(), 1);

        // Verifica che i personaggi restituiscano sempre una slice vuota a questo livello
        assert_eq!(act_unico.characters().len(), 0);

        // Verifica invarianza tratto Copy
        let act_copia = act_unico;
        assert_eq!(act_copia.id(), act_unico.id());
    }
}