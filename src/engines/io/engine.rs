//! src/engines/io/engine.rs

use uuid::Uuid;
use serde::{ Serialize, Deserialize };
use std::collections::HashMap;

// Package Modules & Files
use crate::elements::{ Script, Act, Scene, Block};
use crate::engines::io::error::IoError;
use crate::engines::io::rules::{ PaginationRules, ThemeRules };

// Architettura del file su disco

/// Il macro-contenitore eterogeneo dei Payload della UI e della Produzione.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProjectPayload {
    UiState { theme_mode: String, sidebar_open: bool, zoom_level: f32 },
    Settings { theme: ThemeRules, pagination: PaginationRules },
    Casting { assignments: HashMap<Uuid, String> },
}

/// La sezione "doc" del file. Un database relazionale piatto e serializzabile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocSection<'a> {
    #[serde(borrow)]
    pub script: Script<'a>,
    pub acts_pool: HashMap<Uuid, Act<'a>>,
    pub scenes_pool: HashMap<Uuid, Scene<'a>>,
    #[serde(borrow)]
    pub blocks_pool: HashMap<Uuid, Block<'a>>,
    pub text_arena: HashMap<Uuid, String>,
}

/// La struttura completa del file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFile<'a> {
    #[serde(borrow)]
    pub doc: DocSection<'a>,
    pub payload: Vec<ProjectPayload>,
}

#[derive(Debug, Clone)]
pub struct IoEngine<'a> {
    pub script: Option<Script<'a>>,
    pub acts: HashMap<Uuid, Act<'a>>,
    pub scenes: HashMap<Uuid, Scene<'a>>,
    pub blocks: HashMap<Uuid, Block<'a>>,
    pub text_arena: HashMap<Uuid, String>,
    pub payloads: Vec<ProjectPayload>,
}

impl<'a> IoEngine<'a> {
    pub fn new() -> Self {
        Self {
            script: None,
            acts: HashMap::new(),
            scenes: HashMap::new(),
            blocks: HashMap::new(),
            text_arena: HashMap::new(),
            payloads: Vec::new(),
        }
    }

    pub fn load_project(&mut self, project: ProjectFile<'a>) -> Result<(), IoError> {
        self.script = Some(project.doc.script);
        self.acts = project.doc.acts_pool;
        self.scenes = project.doc.scenes_pool;
        self.blocks = project.doc.blocks_pool;
        self.text_arena = project.doc.text_arena;
        self.payloads = project.payload;
        Ok(())
    }

    pub fn export_project(&self) -> Result<ProjectFile<'a>, IoError> {
        let script = self.script.ok_or(IoError::MissingScriptRoot)?;

        Ok(ProjectFile {
            doc: DocSection {
                script,
                acts_pool: self.acts.clone(),
                scenes_pool: self.scenes.clone(),
                blocks_pool: self.blocks.clone(),
                text_arena: self.text_arena.clone(),
            },
            payload: self.payloads.clone(),
        })
    }
}

//*************************************************************************************************
//* UNIT TEST
//*************************************************************************************************

#[cfg(test)]
mod engine_tests {
    use super::*;
    use crate::common::{ BlockType, LayerChannel, BoundingBox };
    use crate::core::{Meta, VisualLayer };
    use crate::elements::{Script, Act, Scene, Block};
    use crate::traits::TextElement;
    use crate::engines::io::error::IoError;
    use crate::engines::io::rules::{ThemeRules, PaginationRules};

    #[test]
    fn test_io_engine_roundtrip_with_lifetimes() {
        let mut engine = IoEngine::new();

        // 1. Iniezione Script Root
        let script = Script::new("Macbeth", "William Shakespeare");
        engine.script = Some(script);

        // 2. Creazione Scena e Blocco (Soddisfiamo i lifetime con stringhe locali al test)
        let scene_title = "Atto Primo, Scena Prima";
        let mut visual_scene = VisualLayer::new();
        visual_scene.bounds = BoundingBox::full_page();

        let scene = Scene {
            meta: Meta::new(),
            visual: visual_scene,
            number: 1,
            title: scene_title,
            duration_minutes: 2.5,
            blocks: [Uuid::nil(); 128],
            blocks_count: 0,
            characters: [Uuid::nil(); 32],
            characters_count: 0,
        };
        let scene_id = scene.meta.id;
        engine.scenes.insert(scene_id, scene);

        let block_text = "Quando ci rivedremo noi tre?";
        let block = Block::new(BlockType::Dialog, LayerChannel::ScriptText, block_text);
        let block_id = block.meta.id;
        engine.blocks.insert(block_id, block);

        // Alimentiamo l'arena con la stringa proprietaria
        engine.text_arena.insert(block_id, block_text.to_string());

        // 3. Configurazione dei Payload esterni (Theme e Pagination)
        let theme_rules = ThemeRules::new("Test Theme", "Courier", 12.0, 1.0);
        let pag_rules = PaginationRules::new("A4", 210.0, 297.0);

        engine.payloads.push(ProjectPayload::Settings {
            theme: theme_rules,
            pagination: pag_rules,
        });

        // 4. Esportazione strutturata nel ProjectFile
        let export_result = engine.export_project();
        assert!(export_result.is_ok());

        let project_file = export_result.unwrap();

        // 5. Validazione speculare delle tabelle piatte (Pool)
        assert_eq!(project_file.doc.script.title, "Macbeth");
        assert_eq!(project_file.doc.scenes_pool.get(&scene_id).unwrap().title, "Atto Primo, Scena Prima");
        assert_eq!(project_file.doc.blocks_pool.get(&block_id).unwrap().text_type(), "dialog");
        assert_eq!(project_file.doc.text_arena.get(&block_id).unwrap(), "Quando ci rivedremo noi tre?");
        assert_eq!(project_file.payload.len(), 1);
    }

    #[test]
    fn test_io_engine_missing_root_error() {
        let engine = IoEngine::new();

        // Un engine senza Script Root deve fallire l'esportazione sollevando l'errore del modulo error
        let result = engine.export_project();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), IoError::MissingScriptRoot);
    }
}
