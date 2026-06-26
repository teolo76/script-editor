//! src/engines/io/rules.rs

use uuid::Uuid;
use serde::{ Serialize, Deserialize };
use std::collections::HashMap;

// Package Modules & Files
use crate::common::BlockType;
use crate::engines::io::error::IoError;

/// Regole di Formattazione
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeRules {
    pub name: String,
    pub font_family: String,
    pub font_size: f32,
    pub line_height: f32,
    pub margins: HashMap<BlockType, f32>,
}

impl ThemeRules {
    /// Crea un nuovo tema vuoto/personalizzato
    pub fn new(name: &str, font_family :&str, font_size: f32, line_height: f32) -> Self {
        Self {
            name: name.trim().to_string(),
            font_family: font_family.trim().to_string(),
            font_size,
            line_height,
            margins: HashMap::new(),
        }
    }
    /// Idratazione simmetrica completa da specs esterne
    pub fn with_specs(name: String, font_family: String, font_size: f32, line_height: f32, margins: HashMap<BlockType, f32>) -> Self {
        Self { name, font_family, font_size, line_height, margins }
    }
}

/// Regole di Formattazione
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PaginationRules {
    /// Formato dell'area di paginazione: "A4", "Letter", ecc.
    pub format: String,
    pub page_width: f32,
    pub page_height: f32,
    pub prevent_orphan_character: bool,
    pub allow_dialog_splitting: bool,
    pub more_marker: String,
    pub cont_marker: String,
}

impl PaginationRules {
    pub fn new(format: &str, width: f32, height: f32) -> Self {
        Self {
            format: format.trim().to_string(),
            page_width: width,
            page_height: height,
            prevent_orphan_character: true,
            allow_dialog_splitting: true,
            more_marker: "...".to_string(),
            cont_marker: "...".to_string(),
        }
    }

    pub fn with_specs(
        format: String,
        page_width: f32,
        page_height: f32,
        prevent_orphan_character: bool,
        allow_dialog_splitting: bool,
        more_marker: String,
        cont_marker: String,
    ) -> Self {
        Self {
            format,
            page_width,
            page_height,
            prevent_orphan_character,
            allow_dialog_splitting,
            more_marker,
            cont_marker,
        }
    }
}

/// Registro centralizzato per l'importazione, esportazione e gestione dei preset.
#[derive(Debug, Clone, Default)]
pub struct RuleProvider {
    pub themes: HashMap<String, ThemeRules>,
    pub paginations: HashMap<String, PaginationRules>,
}

impl RuleProvider {
    pub fn new() -> Self {
        let mut provider = Self{
            themes: HashMap::new(),
            paginations: HashMap::new(),
        };
        provider.load_default_presets();
        provider
    }
    /// Inietta i preset standard industriali di fabbrica
    fn load_default_presets(&mut self) {
        // Preset Tradizionale Courier
        let mut traditional_margins = HashMap::new();
        traditional_margins.insert(BlockType::CharacterName, 3.5);
        traditional_margins.insert(BlockType::Dialog, 2.5);

        let courier_theme = ThemeRules::with_specs(
            "Tradizionale".to_string(),
            "Courier New".to_string(),
            12.0,
            1.0,
            traditional_margins,
        );
        self.themes.insert(courier_theme.name.clone(), courier_theme);

        // Preset A4 Standard
        let a4_pag = PaginationRules::with_specs(
            "A4".to_string(),
            210.0,
            297.0,
            true,
            true,
            "(CONTIENE)".to_string(),
            "(CONT.)".to_string(),
        );
        self.paginations.insert(a4_pag.format.clone(), a4_pag);
    }
    // Importa un tema da una stringa JSON esterna (es. da cartella config della UI)
    pub fn import_theme_json(&mut self, json_str: &str) -> Result<(), IoError> {
        let theme: ThemeRules = serde_json::from_str(json_str)
            .map_err(|_| IoError::InvalidJsonStructure)?;

        self.themes.insert(theme.name.clone(), theme);
        Ok(())
    }

    /// Esporta un tema registrato in formato JSON string per il salvataggio esterno
    pub fn export_theme_json(&self, name: &str) -> Result<String, IoError> {
        let theme = self.themes.get(name)
            .ok_or_else(|| IoError::PresetNotFound(name.to_string()))?;

        serde_json::to_string_pretty(theme).map_err(|_| IoError::InvalidJsonStructure)
    }
}

//*************************************************************************************************
//* UNIT TEST
//*************************************************************************************************

#[cfg(test)]
mod rules_tests {
    use super::*;

    #[test]
    fn test_theme_and_pagination_builders() {
        // Verifica costruttore pulito di ThemeRules
        let mut theme = ThemeRules::new("Teatro Minimal", "Helvetica", 11.0, 1.2);
        theme.margins.insert(BlockType::Dialog, 3.0);

        assert_eq!(theme.name, "Teatro Minimal");
        assert_eq!(*theme.margins.get(&BlockType::Dialog).unwrap(), 3.0);

        // Verifica costruttore di PaginationRules
        let pag = PaginationRules::new("Letter", 215.9, 279.4);
        assert_eq!(pag.format, "Letter");
        assert!(pag.prevent_orphan_character); // Default abilitato
    }

    #[test]
    fn test_rule_provider_lifecycle() {
        let mut provider = RuleProvider::new();

        // Verifica che i preset industriali siano pre-caricati di fabbrica
        assert!(provider.themes.contains_key("Tradizionale"));
        assert!(provider.paginations.contains_key("A4"));

        // Simulazione importazione JSON esterno (es. da file di configurazione UI)
        let external_theme_json = r#"{
            "name": "Cinematic High-Contrast",
            "font_family": "Courier Prime",
            "font_size": 12.0,
            "line_height": 1.0,
            "margins": {}
        }"#;

        let import_res = provider.import_theme_json(external_theme_json);
        assert!(import_res.is_ok());
        assert!(provider.themes.contains_key("Cinematic High-Contrast"));

        // Esportazione simmetrica
        let export_res = provider.export_theme_json("Cinematic High-Contrast");
        assert!(export_res.is_ok());
        assert!(export_res.unwrap().contains("Cinematic High-Contrast"));
    }
}