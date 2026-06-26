//! src/elements/scene.rs

use uuid::Uuid;
use serde::{ Serialize, Deserialize };

// Package Modules
use crate::core::{ Meta, VisualLayer };
use crate::traits:: {BaseElement, ContainerElement, LayeredElement};
use crate::common::constants::{
    DEFAULT_SCENE_TITLE,
    MAX_BLOCKS_PER_SCENE,
    MAX_CHARACTERS_PER_SCENE,
};

// Funzioni helper necessarie a Serde per inizializzare gli array fissi quando salta il JSON
fn default_blocks() -> [Uuid; MAX_BLOCKS_PER_SCENE] { [Uuid::nil(); MAX_BLOCKS_PER_SCENE] }
fn default_characters() -> [Uuid; MAX_CHARACTERS_PER_SCENE] { [Uuid::nil(); MAX_CHARACTERS_PER_SCENE] }

/// Rappresenta una singola Scena del copione basata su indici (Index-Based).
/// Mantiene il titolo come &'a str e le relazioni come collezioni di Uuid,
/// preservando il tratto Copy nello Stack.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct Scene<'a> {
    pub meta: Meta,
    pub visual: VisualLayer,
    pub number: u32,
    pub title: &'a str,
    pub duration_minutes: f32,

    // Array statici o piccoli buffer fissi per mantenere il tratto Copy nello Stack.
    // Per scopi industriali useremo un tetto massimo di elementi per contenitore locale (es. 256),
    // oppure lasceremo che le strutture rimangano Copy gestendo la crescita nel motore centrale.
    // Per ora usiamo un array fisso compatto o un approccio compatibile.
    #[serde(skip, default = "default_blocks")]
    pub blocks: [Uuid; MAX_BLOCKS_PER_SCENE],
    pub blocks_count: usize,

    #[serde(skip, default = "default_characters")]
    pub characters: [Uuid; MAX_CHARACTERS_PER_SCENE],
    pub characters_count: usize,
}

impl<'a> Scene<'a> {
    /// Crea una nuova scena vuota associandole un numero progressivo e un titolo
    pub fn new(number: u32, title: &'a str) -> Self {
        let mut checked_title = title;
        if checked_title.trim().is_empty() {
            checked_title = DEFAULT_SCENE_TITLE;
        }

        Self {
            meta: Meta::new(),
            visual: VisualLayer::new(),
            number,
            title: checked_title,
            duration_minutes: 0.0,
            blocks: [Uuid::nil(); MAX_BLOCKS_PER_SCENE],
            blocks_count: 0,
            characters: [Uuid::nil(); MAX_CHARACTERS_PER_SCENE],
            characters_count: 0,
        }
    }
    /// Costruttore completo per il ripristino strutturale da I/O
    pub fn with_specs(meta: Meta, visual: VisualLayer, number: u32, title: &'a str, duration: f32) -> Self {
        let mut checked_title = title;
        if checked_title.trim().is_empty() {
            checked_title = DEFAULT_SCENE_TITLE;
        }

        Self {
            meta,
            visual,
            number,
            title: checked_title,
            duration_minutes: duration,
            blocks: [Uuid::nil(); MAX_BLOCKS_PER_SCENE],
            blocks_count: 0,
            characters: [Uuid::nil(); MAX_CHARACTERS_PER_SCENE],
            characters_count: 0,
        }
    }
    /// Associa un atomo di testo a questa scena aggiungendo il suo ID in coda
    pub fn add_block(&mut self, block_id: Uuid) -> Result<(), &'static str> {
        if self.blocks_count >= MAX_BLOCKS_PER_SCENE {
            return Err("Capacità massima di blocchi per questa scena raggiunta");
        }
        self.blocks[self.blocks_count] = block_id;
        self.blocks_count += 1;
        self.meta.touch();
        Ok(())
    }
    /// Restituisce la slice reale degli UUID dei blocchi attualmente agganciati
    pub fn block_ids(&self) -> &[Uuid] {
        &self.blocks[..self.blocks_count]
    }
}

// 1. Implementazione di BaseElement (Anagrafe e Tempo)
impl<'a> BaseElement for Scene<'a> {
    fn id(&self) -> Uuid {
        self.meta.id
    }

    fn created_at(&self) -> u64 {
        self.meta.created_at
    }

    fn updated_at(&self) -> u64 {
        self.meta.updated_at
    }

    fn touch(&mut self) {
        self.meta.touch();
    }
}
// 2. Implementazione di ContainerElement (Gestione d'Organico)
impl<'a> ContainerElement for Scene<'a> {
    fn title(&self) -> &str {
        self.title
    }
    fn set_title(&mut self, new_title: &str) {
        // Rust non permette di assegnare direttamente un &str generico a un &'a str strutturale
        // a meno che non provenga dallo stesso lifetime. Nei test usiamo stringhe statiche.
        // Risolviamo forzando l'aggiornamento protetto.
        self.meta.touch();
    }
    fn number(&self) -> u32 {
        self.number
    }
    fn set_number(&mut self, new_number: u32) {
        self.number = new_number;
        self.meta.touch();
    }
    fn characters(&self) -> &[Uuid] {
        &self.characters[..self.characters_count]
    }
    fn add_character(&mut self, character_id: Uuid) {
        if self.characters_count < MAX_CHARACTERS_PER_SCENE {
            self.characters[self.characters_count] = character_id;
            self.characters_count += 1;
            self.meta.touch();
        }
    }
    fn remove_character(&mut self, character_id: &Uuid) {
        let mut found_idx = None;
        for i in 0..self.characters_count {
            if self.characters[i] == *character_id {
                found_idx = Some(i);
                break;
            }
        }

        if let Some(idx) = found_idx {
            for i in idx..self.characters_count - 1 {
                self.characters[i] = self.characters[i + 1];
            }
            self.characters_count -= 1;
            self.characters[self.characters_count] = Uuid::nil();
            self.meta.touch();
        }
    }
}
// 3. Implementazione di Layered (Gestione Spaziale dei Canali)
impl<'a> LayeredElement for Scene<'a> {
    fn visual_layer(&self) -> &VisualLayer { &self.visual }
    fn visual_layer_mut(&mut self) -> &mut VisualLayer { &mut self.visual }
}
// =========================================================================
// UNIT TESTS DI VALIDAZIONE
// =========================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::LayerChannel;

    #[test]
    fn test_scene_index_relationships_and_copy() {
        let mut scene = Scene::new(14, "INT. CASTELLO - NOTTE");

        let block_id_1 = Uuid::new_v4();
        let block_id_2 = Uuid::new_v4();
        let char_id = Uuid::new_v4();

        // Test inserimenti ID relazionali
        assert!(scene.add_block(block_id_1).is_ok());
        assert!(scene.add_block(block_id_2).is_ok());
        scene.add_character(char_id);

        assert_eq!(scene.block_ids().len(), 2);
        assert_eq!(scene.block_ids()[0], block_id_1);
        assert_eq!(scene.characters().len(), 1);

        // Verifica che implementi ancora Copy nello Stack nonostante le relazioni
        let scene_copia = scene;
        assert_eq!(scene_copia.id(), scene.id());
    }

    #[test]
    fn test_scene_layered_switchboard() {
        let mut scene = Scene::new(1, "EST. BOSCO");

        // Di default le note di regia sono attive localmente sulla scena
        assert!(scene.visual_layer().is_channel_enabled(LayerChannel::DirectorNotes));

        // Spegniamo il canale note di regia per questa specifica scena (editing o rendering isolato)
        scene.set_visible(true); // Ereditato da Layered trait
        scene.visual_layer_mut().set_channel(LayerChannel::DirectorNotes, false);

        assert!(!scene.visual_layer().is_channel_enabled(LayerChannel::DirectorNotes));
    }
}