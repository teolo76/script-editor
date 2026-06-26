//! src/elements/script.rs

use uuid::Uuid;
use serde::{ Serialize, Deserialize };

// Package Files & Modules
use crate::core::{ Meta, SwitchBoard };
use crate::traits::{ BaseElement, MasterContainer };

use crate::common::constants::{
    MAX_ACTS_PER_SCRIPT,
    MAX_CHARACTERS_PER_SCRIPT
};

// Funzioni helper per permettere a Serde di inizializzare gli array fissi nello Stack
fn default_acts() -> [Uuid; MAX_ACTS_PER_SCRIPT] { [Uuid::nil(); MAX_ACTS_PER_SCRIPT] }
fn default_cast() -> [Uuid; MAX_CHARACTERS_PER_SCRIPT] { [Uuid::nil(); MAX_CHARACTERS_PER_SCRIPT] }

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Hash)]
pub struct Script<'a>{
    pub meta: Meta,
    pub switchboard: SwitchBoard,
    pub title: &'a str,
    pub author: &'a str,
    pub version_major: u32,
    pub version_minor: u32,

    // Array statici per preservare il tratto Copy nello Stack
    #[serde(skip, default = "default_acts")]
    pub acts: [Uuid; MAX_ACTS_PER_SCRIPT],
    pub acts_count: usize,

    #[serde(skip, default = "default_cast")]
    pub cast: [Uuid; MAX_CHARACTERS_PER_SCRIPT],
    pub cast_count: usize,

    pub has_cover_page: bool,
}

impl<'a> Script<'a>{
    /// Costruttore standard per un nuovo progetto
    pub fn new(title: &'a str, author: &'a str) -> Self {
        Self {
            meta: Meta::new(), // Inizializza id e timestamp, major version (meta.number) parte da 1
            switchboard: SwitchBoard::new(),
            title: title.trim(),
            author: author.trim(),
            version_major: 1,
            version_minor: 0,
            acts: [Uuid::nil(); MAX_ACTS_PER_SCRIPT],
            acts_count: 0,
            cast: [Uuid::nil(); MAX_CHARACTERS_PER_SCRIPT],
            cast_count: 0,
            has_cover_page: false,
        }
    }
    /// Costruttore completo simmetrico per il ripristino da I/O (JSON/File)
    pub fn with_specs(
        meta: Meta,
        switchboard: SwitchBoard,
        title: &'a str,
        author: &'a str,
        version_major: u32,
        version_minor: u32,
        has_cover_page: bool,
    ) -> Self {
        Self {
            meta,
            switchboard,
            title: title.trim(),
            author: author.trim(),
            version_major,
            version_minor,
            acts: [Uuid::nil(); MAX_ACTS_PER_SCRIPT],
            acts_count: 0,
            cast: [Uuid::nil();MAX_CHARACTERS_PER_SCRIPT],
            cast_count: 0,
            has_cover_page,
        }
    }
    /// Restituisce la versione formattata in modo leggibile senza allocare nella Heap a lungo termine
    pub fn formatted_version(&self) -> String {
        format!("{}.{}", self.version_major, self.version_minor)
    }
    /// Aggancia un Atto allo Script inserendo il suo ID in coda sequenziale
    pub fn add_act(&mut self, act_id: Uuid) -> Result<(), &'static str> {
        if self.acts_count >= MAX_ACTS_PER_SCRIPT {
            return Err("Capacità massima di atti per questo copione raggiunta (32)");
        }
        self.acts[self.acts_count] = act_id;
        self.acts_count += 1;
        self.meta.touch();
        Ok(())
    }
    /// Restituisce la slice ordinata degli UUID degli atti componenti
    pub fn act_ids(&self) -> &[Uuid] {
        &self.acts[..self.acts_count]
    }
}

// 1. Implementazione di BaseElement (Identità, Tempo e Versioning strutturale)
impl<'a> BaseElement for Script<'a> {
    fn id(&self) -> Uuid { self.meta.id }
    fn created_at(&self) -> u64 { self.meta.created_at }
    fn updated_at(&self) -> u64 { self.meta.updated_at }
    fn touch(&mut self) { self.meta.touch(); }
}

// 2. Implementazione del contratto speciale MasterContainer
impl<'a> MasterContainer for Script<'a> {
    fn major_version(&self) -> u32 {
        self.version_major
    }
    fn minor_version(&self) -> u32 {
        self.version_minor
    }
    fn increment_version(&mut self) {
        self.version_major += 1;
        self.version_minor = 0; // Il reset della minor al salto di major è lo standard
        self.meta.touch();
    }
    /// Incrementa la sotto-versione (minor) per modifiche correnti
    fn increment_minor_version(&mut self) {
        self.version_minor += 1;
        self.meta.touch();
    }
    fn author(&self) -> &str { self.author }
    fn set_author(&mut self, _author: &str) { self.meta.touch(); }
    fn switchboard(&self) -> &SwitchBoard { &self.switchboard }
    fn switchboard_mut(&mut self) -> &mut SwitchBoard { &mut self.switchboard }

    fn cast(&self) -> &[Uuid] {
        &self.cast[..self.cast_count]
    }

    fn register_character(&mut self, character_id: Uuid) -> Result<(), &'static str> {
        if self.cast_count >= 256 {
            return Err("Capacità massima del cast raggiunta (256)");
        }
        if !self.cast().contains(&character_id) {
            self.cast[self.cast_count] = character_id;
            self.cast_count += 1;
            self.meta.touch();
        }
        Ok(())
    }

    fn unregister_character(&mut self, character_id: &Uuid) {
        let mut found_idx = None;
        for i in 0..self.cast_count {
            if self.cast[i] == *character_id {
                found_idx = Some(i);
                break;
            }
        }
        if let Some(idx) = found_idx {
            for i in idx..self.cast_count - 1 {
                self.cast[i] = self.cast[i + 1];
            }
            self.cast_count -= 1;
            self.cast[self.cast_count] = Uuid::nil();
            self.meta.touch();
        }
    }

    fn has_cover(&self) -> bool { self.has_cover_page }
}

// =========================================================================
// UNIT TESTS DI VALIDAZIONE
// =========================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::LayerChannel;

    #[test]
    fn test_script_master_logic_and_copy() {
        let mut script = Script::new(" Amleto ", " William Shakespeare ");

        assert_eq!(script.title, "Amleto");
        assert_eq!(script.author(), "William Shakespeare");
        assert_eq!(script.major_version(), 1); // Parte da versione 1

        // Test evoluzione versione
        script.increment_version();
        assert_eq!(script.major_version(), 2);
        assert_eq!(script.minor_version(), 0);

        // Test Anagrafe del Cast Ufficiale
        let char_id_1 = Uuid::new_v4();
        let char_id_2 = Uuid::new_v4();
        assert!(script.register_character(char_id_1).is_ok());
        assert!(script.register_character(char_id_2).is_ok());
        assert_eq!(script.cast().len(), 2);

        // Test rimozione dal cast
        script.unregister_character(&char_id_1);
        assert_eq!(script.cast().len(), 1);
        assert_eq!(script.cast()[0], char_id_2);

        // Verifica invarianza tratto Copy nello Stack
        let script_copia = script;
        assert_eq!(script_copia.id(), script.id());
    }

    #[test]
    fn test_script_master_switchboard_integration() {
        let mut script = Script::new("Titolo", "Autore");

        // Di default i canali sono attivi
        assert!(script.switchboard().is_channel_enabled(LayerChannel::DirectorNotes));

        // Spegnimento globale (Veto Supremo)
        script.switchboard_mut().disable_channel(LayerChannel::DirectorNotes);
        assert!(!script.switchboard().is_channel_enabled(LayerChannel::DirectorNotes));
    }

    #[test]
    fn test_script_subversioning_and_with_specs() {
        let mut script = Script::new("Inception", "Nolan");
        assert_eq!(script.version_major, 1);
        assert_eq!(script.version_minor, 0);

        // Test incremento minor
        script.increment_minor_version();
        assert_eq!(script.version_major, 1);
        assert_eq!(script.version_minor, 1);

        // Test passaggio di Major (messa in produzione)
        script.increment_version();
        assert_eq!(script.version_major, 2);
        assert_eq!(script.version_minor, 0); // Reset automatico

        // Test costruttore speculare con specs esterne
        let copia_strutturale = Script::with_specs(
            script.meta,
            script.switchboard,
            script.title,
            script.author,
            script.version_major,
            script.version_minor,
            script.has_cover_page
        );

        assert_eq!(copia_strutturale.id(), script.id());
        assert_eq!(copia_strutturale.version_major, 2);
    }
}