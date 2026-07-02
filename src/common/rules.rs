//! src/common/rules.rs

use uuid::Uuid;
use serde::{ Serialize, Deserialize };
use std::collections::HashMap;

// Package Modules & Files
use crate::common::BlockType;
use crate::io::error::IoError;

/// Geometria di rientro e spaziatura specifica per un singolo elemento del copione.
/// Espressa interamente in millimetri (mm) per l'indipendenza dalla densità di pixel.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlockGeometry {
    /// Rientro dal margine sinistro della colonna stampabile
    pub margin_left: f32,
    /// Rientro dal margine destro della colonna stampabile
    pub margin_right: f32,
    /// Spazio vuoto verticale da lasciare SOPRA il blocco
    pub space_before: f32,
    /// Spazio vuoto verticale da lasciare SOTTO il blocco
    pub space_after: f32,
}

impl BlockGeometry {
    pub fn new(margin_left: f32, margin_right: f32, space_before: f32, space_after: f32) -> Self {
        Self {
            margin_left,
            margin_right,
            space_before,
            space_after,
        }
    }
}

/// Regole di Formattazione Estetica
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeRules {
    pub name: String,
    pub font_family: String,
    pub font_size: f32,
    pub line_height: f32,
    /// Associa ogni tipologia di blocco alla sua geometria dettagliata
    pub geometries: HashMap<BlockType, BlockGeometry>,
}

impl ThemeRules {
    /// Crea un nuovo tema vuoto/personalizzato
    pub fn new(name: &str, font_family: &str, font_size: f32, line_height: f32) -> Self {
        Self {
            name: name.trim().to_string(),
            font_family: font_family.trim().to_string(),
            font_size,
            line_height,
            geometries: HashMap::new(),
        }
    }

    /// Idratazione simmetrica completa da specs esterne
    pub fn with_specs(
        name: String,
        font_family: String,
        font_size: f32,
        line_height: f32,
        geometries: HashMap<BlockType, BlockGeometry>
    ) -> Self {
        Self { name, font_family, font_size, line_height, geometries }
    }
}

/// Regole di Formattazione della Pagina Fisiche e Tipografiche
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PaginationRules {
    /// Formato dell'area di paginazione: "A4", "Letter", ecc.
    pub format: String,
    pub page_width: f32,
    pub page_height: f32,
    // Margini fisici esterni del foglio
    pub margin_top: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,
    pub margin_right: f32,

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
            margin_top: 25.4,    // 1 pollice default industriale
            margin_bottom: 25.4,
            margin_left: 38.1,   // 1.5 pollici a sinistra per la rilegatura
            margin_right: 25.4,
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
        margin_top: f32,
        margin_bottom: f32,
        margin_left: f32,
        margin_right: f32,
        prevent_orphan_character: bool,
        allow_dialog_splitting: bool,
        more_marker: String,
        cont_marker: String,
    ) -> Self {
        Self {
            format,
            page_width,
            page_height,
            margin_top,
            margin_bottom,
            margin_left,
            margin_right,
            prevent_orphan_character,
            allow_dialog_splitting,
            more_marker,
            cont_marker,
        }
    }

    /// Calcola l'ampiezza netta della colonna in cui il testo effettua il wrap (escluso il rientro del blocco)
    pub fn printable_width(&self) -> f32 {
        self.page_width - (self.margin_left + self.margin_right)
    }

    /// Calcola l'altezza massima verticale sfruttabile prima del page break
    pub fn printable_height(&self) -> f32 {
        self.page_height - (self.margin_top + self.margin_bottom)
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
        let mut provider = Self {
            themes: HashMap::new(),
            paginations: HashMap::new(),
        };
        provider.load_default_presets();
        provider
    }

    /// Inietta i preset standard industriali di fabbrica aggiornati con BlockGeometry
    fn load_default_presets(&mut self) {
        let mut traditional_geometries = HashMap::new();

        // Setup geometrie standard Courier per i blocchi core
        traditional_geometries.insert(
            BlockType::CharacterName,
            BlockGeometry::new(55.88, 0.0, 4.23, 0.0)
        );
        traditional_geometries.insert(
            BlockType::Dialog,
            BlockGeometry::new(25.4, 38.1, 0.0, 4.23)
        );

        let courier_theme = ThemeRules::with_specs(
            "Tradizionale".to_string(),
            "Courier New".to_string(),
            12.0,
            1.0,
            traditional_geometries,
        );
        self.themes.insert(courier_theme.name.clone(), courier_theme);

        // Preset A4 Standard
        let a4_pag = PaginationRules::with_specs(
            "A4".to_string(),
            210.0,
            297.0,
            25.4,
            25.4,
            38.1,
            25.4,
            true,
            true,
            "(CONTIENE)".to_string(),
            "(CONT.)".to_string(),
        );
        self.paginations.insert(a4_pag.format.clone(), a4_pag);
    }

    pub fn import_theme_json(&mut self, json_str: &str) -> Result<(), IoError> {
        let theme: ThemeRules = serde_json::from_str(json_str)
            .map_err(|_| IoError::InvalidJsonStructure)?;

        self.themes.insert(theme.name.clone(), theme);
        Ok(())
    }

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
        // Verifica costruttore pulito di ThemeRules e inserimento BlockGeometry
        let mut theme = ThemeRules::new("Teatro Minimal", "Helvetica", 11.0, 1.2);
        let geom = BlockGeometry::new(10.0, 10.0, 5.0, 5.0);
        theme.geometries.insert(BlockType::Dialog, geom);

        assert_eq!(theme.name, "Teatro Minimal");
        assert_eq!(theme.geometries.get(&BlockType::Dialog).unwrap().margin_left, 10.0);

        // Verifica costruttore di PaginationRules e relative aree stampabili
        let pag = PaginationRules::new("Letter", 215.9, 279.4);
        assert_eq!(pag.format, "Letter");
        assert!(pag.prevent_orphan_character);
        assert_eq!(pag.printable_width(), 215.9 - (38.1 + 25.4));
    }

    #[test]// src/io/rules.rs (Dentro il test test_rule_provider_lifecycle)

    #[test]
    fn test_rule_provider_lifecycle() {
        let mut provider = RuleProvider::new();

        assert!(provider.themes.contains_key("Tradizionale"));
        assert!(provider.paginations.contains_key("A4"));

        // CORREZIONE: Cambiato "Dialog" in "dialog" per rispettare #[serde(rename = "dialog")]
        let external_theme_json = r#"{
            "name": "Cinematic High-Contrast",
            "font_family": "Courier Prime",
            "font_size": 12.0,
            "line_height": 1.0,
            "geometries": {
                "dialog": {
                    "margin_left": 25.4,
                    "margin_right": 38.1,
                    "space_before": 0.0,
                    "space_after": 4.23
                }
            }
        }"#;

        let import_res = provider.import_theme_json(external_theme_json);
        assert!(import_res.is_ok());
        assert!(provider.themes.contains_key("Cinematic High-Contrast"));

        let export_res = provider.export_theme_json("Cinematic High-Contrast");
        assert!(export_res.is_ok());
        assert!(export_res.unwrap().contains("Cinematic High-Contrast"));
    }
   
}