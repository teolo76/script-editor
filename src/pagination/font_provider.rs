//! src/pagination/font_provider.rs

use std::fmt;

// Package Modules & Files
use crate::common::BlockType;
use crate::common::rules::{ ThemeRules, PaginationRules };
use crate::pagination::error::{ FontMetricError };

/// Rappresenta il calcolo geometrico restituito dal provider per un testo formattato.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextLayoutMetrics {
    /// Altezza totale occupata dal solo corpo del testo (in millimetri)
    pub total_height: f32,
    /// Numero di righe generate dopo aver applicato il Line-Wrap
    pub line_count: usize,
}

pub trait FontMetricProvider {
    /// Calcola l'ingombro millimetrico esatto di una stringa di testo,
    /// considerando il tipo di blocco, il tema e le regole di pagina correnti.
    fn compute_metrics(&self, text: &str, block_type: BlockType, theme: &ThemeRules, pagination: &PaginationRules,) -> Result<TextLayoutMetrics, FontMetricError>;
}

/// Provider deterministico a spaziatura fissa da utilizzare nei test unitari e in modalità headless.
/// Simula un font monospazio dove ogni carattere e ogni riga hanno dimensioni costanti.
pub struct FixedFontProvider {
    char_width_mm: f32,
    line_height_mm: f32,
}

impl FixedFontProvider {
    /// Inizializza un provider monospazio con parametri Courier standard (10 CPI, 1/6 di pollice per riga)
    pub fn new() -> Self {
        Self {
            char_width_mm: 2.54,
            line_height_mm: 4.23,
        }
    }

    pub fn with_specs(char_width: f32, line_height: f32) -> Self {
        Self { char_width_mm: char_width, line_height_mm: line_height }
    }
}

impl FontMetricProvider for FixedFontProvider {
    fn compute_metrics(
        &self,
        text: &str,
        block_type: BlockType,
        theme: &ThemeRules,
        pagination: &PaginationRules,
    ) -> Result<TextLayoutMetrics, FontMetricError> {
        // 1. Calcola la larghezza massima della colonna stampabile del foglio
        let printable_width = pagination.printable_width();

        // 2. Estrae la geometria specifica per questo tipo di blocco dal tema
        // Se manca, facciamo un fallback difensivo a zero margini
        let geometry = theme.geometries.get(&block_type);
        let (margin_left, margin_right) = match geometry {
            Some(g) => (g.margin_left, g.margin_right),
            None => (0.0, 0.0),
        };

        // 3. Calcola lo spazio di riga netto rimanente per il testo
        let max_text_width_mm = printable_width - (margin_left + margin_right);
        if max_text_width_mm <= 0.0 {
            return Err(FontMetricError::InvalidMetrics);
        }

        // Se il testo è vuoto, occupa lo spazio di una singola riga base
        if text.is_empty() {
            return Ok(TextLayoutMetrics {
                total_height: self.line_height_mm,
                line_count: 1,
            });
        }

        // 4. Determina quanti caratteri monospazio possono entrare in una riga utile
        let max_chars_per_line = (max_text_width_mm / self.char_width_mm).floor() as usize;
        let max_chars_per_line = if max_chars_per_line == 0 { 1 } else { max_chars_per_line };

        let mut line_count = 0;

        // 5. Algoritmo di Line Wrapping basato sulla lunghezza dei paragrafi
        for paragraph in text.lines() {
            if paragraph.is_empty() {
                line_count += 1;
                continue;
            }

            let char_count = paragraph.chars().count();
            let mut p_lines = char_count / max_chars_per_line;
            if char_count % max_chars_per_line != 0 {
                p_lines += 1;
            }
            line_count += p_lines;
        }

        if line_count == 0 {
            line_count = 1;
        }

        Ok(TextLayoutMetrics {
            total_height: line_count as f32 * self.line_height_mm,
            line_count,
        })
    }
}

#[cfg(test)]
mod font_provider_tests {
    use super::*;
    use crate::common::rules::RuleProvider;

    #[test]
    fn test_fixed_font_provider_with_block_geometries() {
        let provider = FixedFontProvider::new();

        // Estraiamo i preset industriali pronti dal RuleProvider
        let rule_manager = RuleProvider::new();
        let theme = rule_manager.themes.get("Tradizionale").unwrap();
        let pagination = rule_manager.paginations.get("A4").unwrap();

        // Test Dialogo vuoto: deve comunque occupare una riga
        let metrics_empty = provider.compute_metrics("", BlockType::Dialog, theme, pagination).unwrap();
        assert_eq!(metrics_empty.line_count, 1);
        assert_eq!(metrics_empty.total_height, provider.line_height_mm);
    }

    #[test]
    fn test_fixed_font_provider_wrapping_on_restricted_columns() {
        // Crea un provider fisso con caratteri da 2mm e linee da 5mm
        let provider = FixedFontProvider::with_specs(2.0, 5.0);

        let rule_manager = RuleProvider::new();
        let theme = rule_manager.themes.get("Tradizionale").unwrap();

        // Generiamo una regola di pagina ad hoc: larghezza totale 100mm, margini esterni 10mm l'uno
        // Area stampabile netta = 100 - 20 = 80mm
        let pagination = PaginationRules::with_specs(
            "Custom".to_string(),
            100.0, 200.0,
            10.0, 10.0, 10.0, 10.0,
            true, true, "...".to_string(), "...".to_string()
        );

        // Se passiamo un testo di 40 caratteri (40 * 2mm = 80mm), su area stampabile da 80mm
        // ed elemento senza margini interni, occuperà esattamente 1 riga.
        // Ma se usiamo BlockType::Dialog, che nel tema Tradizionale ha margin_left: 25.4 e margin_right: 38.1,
        // lo spazio utile per il testo si restringe a: 80 - (25.4 + 38.1) = 16.5mm!
        // Con 16.5mm utili e caratteri da 2mm, entrano al massimo (16.5 / 2).floor() = 8 caratteri per riga.

        let long_text = "1234567890123456"; // 16 caratteri
        let metrics = provider.compute_metrics(long_text, BlockType::Dialog, theme, &pagination).unwrap();

        // 16 caratteri / 8 per riga = esattamente 2 righe!
        assert_eq!(metrics.line_count, 2);
        assert_eq!(metrics.total_height, 10.0); // 2 righe * 5mm = 10mm
    }
}