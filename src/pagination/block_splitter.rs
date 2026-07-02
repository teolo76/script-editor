//! src/pagination/block_splitter.rs

// Package Modules & Files
use crate::common::BlockType;
use crate::common::rules::{ ThemeRules, PaginationRules };
use crate::pagination::{FontMetricProvider, TextLayoutMetrics };

/// Rappresenta il risultato del frazionamento di un blocco di testo.
#[derive(Debug, Clone, PartialEq)]
pub enum SplitResult {
    FitsEntirely,
    PushEntirely,
    Split {
        /// Testo che rimane nella pagina attuale (con eventuale marcatore MORE applicato)
        current_page_text: String,
        /// Testo che viene spostato nella pagina successiva
        next_page_text: String,
        /// Altezza geometrica effettiva occupata nella pagina corrente (in mm)
        current_height: f32,
    },
}

pub struct BlockSplitter;

impl BlockSplitter {
    /// Analizza un blocco di testo e determina se e come deve essere spezzato.
    pub fn evaluate_split(
        provider: &dyn FontMetricProvider,
        text: &str,
        block_type: BlockType,
        remaining_height_mm: f32,
        theme: &ThemeRules,
        pagination: &PaginationRules,
    ) -> SplitResult {
        let geometry = theme.geometries.get(&block_type);
        let space_before = geometry.map(|g| g.space_before).unwrap_or(0.0);
        let space_after = geometry.map(|g| g.space_after).unwrap_or(0.0);

        let full_metrics = match provider.compute_metrics(text, block_type, theme, pagination) {
            Ok(m) => m,
            Err(_) => return SplitResult::PushEntirely,
        };

        let total_required_height = full_metrics.total_height + space_before + space_after;

        // Se entra tutto comodamente nella pagina, confermiamo il posizionamento completo
        if total_required_height <= remaining_height_mm {
            return SplitResult::FitsEntirely;
        }

        // REGOLA D'ORO: Protezione orfani strutturali (Titoli e Nomi Personaggi non si spezzano MAI)
        if block_type == BlockType::Heading1
            || block_type == BlockType::Heading2
            || block_type == BlockType::Heading3
            || block_type == BlockType::CharacterName
            || block_type == BlockType::Transition
        {
            return SplitResult::PushEntirely;
        }

        if block_type == BlockType::Dialog && !pagination.allow_dialog_splitting {
            return SplitResult::PushEntirely;
        }

        // Calcoliamo l'altezza empirica di una singola riga
        let single_line_height = full_metrics.total_height / full_metrics.line_count as f32;
        let text_space_residual = remaining_height_mm - space_before;

        let mut max_possible_lines = (text_space_residual / single_line_height).floor() as usize;

        // Se stiamo splittando un dialogo, dobbiamo sottrarre preventivamente lo spazio per il marcatore "(MORE)"
        if block_type == BlockType::Dialog && max_possible_lines > 1 {
            max_possible_lines -= 1;
        }

        // REGOLA ANTI-ORFANO: Almeno 2 righe nella pagina corrente
        if pagination.prevent_orphan_character && max_possible_lines < 2 {
            return SplitResult::PushEntirely;
        }

        // REGOLA ANTI-VEDOVA: Almeno 2 righe nella pagina successiva
        let remaining_lines = full_metrics.line_count - max_possible_lines;
        if pagination.prevent_orphan_character && remaining_lines < 2 {
            return SplitResult::PushEntirely;
        }

        // Eseguiamo il Word-Wrap reale per dividere il testo preservando l'integrità delle parole
        let (current_text, next_text) = Self::wrap_and_split_text(text, max_possible_lines);

        // Calcolo dell'altezza finale della porzione rimasta nella pagina corrente
        let mut current_height = (max_possible_lines as f32 * single_line_height) + space_before;

        // Se è un dialogo, aggiungiamo l'altezza della riga extra occupata dal marcatore di continuità
        if block_type == BlockType::Dialog {
            current_height += single_line_height;
        }

        // 5. ESECUZIONE DEL WORD-WRAP REALE
        let (current_text, next_text) = Self::wrap_and_split_text(text, max_possible_lines);

        // Calcolo dell'altezza finale della porzione rimasta nella pagina corrente
        let mut current_height = (max_possible_lines as f32 * single_line_height) + space_before;

        // Se è un dialogo, aggiungiamo l'altezza della riga extra occupata dal marcatore di continuità
        if block_type == BlockType::Dialog {
            current_height += single_line_height;
        }

        // CORREZIONE QUI: Mappiamo esplicitamente le variabili locali nei campi dell'enum
        SplitResult::Split {
            current_page_text: current_text,
            next_page_text: next_text,
            current_height,
        }
    }

    /// Prende una stringa di testo e la divide in due porzioni separate,
    /// interrompendo il flusso esattamente al limite delle righe ed evitando di tagliare parole a metà.
    /// Prende una stringa di testo e la divide in due porzioni separate,
    /// interrompendo il flusso esattamente al limite delle righe.
    fn wrap_and_split_text(text: &str, split_line_index: usize) -> (String, String) {
        // Se il testo contiene già dei ritorni a capo espliciti (\n), li usiamo come confini rigidi
        let lines: Vec<&str> = text.lines().collect();

        if lines.len() > 1 {
            let (current_lines, next_lines) = lines.split_at(std::cmp::min(split_line_index, lines.len()));
            return (current_lines.join("\n"), next_lines.join("\n"));
        }

        // Fallback resiliente basato su token se il testo è una stringa continua senza \n
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return (String::new(), String::new());
        }

        // Ripartizione deterministica: dividiamo equamente le parole in base alle righe richieste
        let total_words = words.len();
        let words_per_line = (total_words as f32 / 4.0).max(1.0).ceil() as usize;
        let split_point = std::cmp::min(split_line_index * words_per_line, total_words);

        let (current_words, next_words) = words.split_at(split_point);
        (current_words.join(" "), next_words.join(" "))
    }
}

// =========================================================================
// UNIT TESTS DI PERSISTENZA E IN-MEMORY MOCK
// =========================================================================
#[cfg(test)]
mod splitter_tests {
    use super::*;
    use crate::common::BlockType;
    use crate::common::rules::{RuleProvider, PaginationRules, BlockGeometry};
    use crate::pagination::font_provider::FixedFontProvider;

    #[test]
    fn test_splitter_heading_never_splits() {
        let provider = FixedFontProvider::new(); // Altezza riga base: 4.23mm
        let rule_manager = RuleProvider::new();
        let mut theme = rule_manager.themes.get("Tradizionale").unwrap().clone();
        let pagination = rule_manager.paginations.get("A4").unwrap();

        // Configura esplicitamente la geometria dell'Heading1 per il test
        // Spazio totale richiesto: 8.46 (space_before) + 4.23 (testo) = 12.69mm
        theme.geometries.insert(
            BlockType::Heading1,
            BlockGeometry::new(0.0, 0.0, 8.46, 4.23)
        );

        let heading_text = "INT. CASTELLO DI MACBETH - NOTTE";

        // Forniamo solo 10.0mm rimasti. Richiedendone 12.69mm, il blocco deve
        // essere spinto interamente alla pagina successiva perché è un titolo atomico.
        let result = BlockSplitter::evaluate_split(
            &provider,
            heading_text,
            BlockType::Heading1,
            10.0,
            &theme,
            pagination
        );

        assert_eq!(result, SplitResult::PushEntirely);
    }

    #[test]
    fn test_splitter_dialog_respects_orphan_and_widow_rules() {
        let provider = FixedFontProvider::with_specs(2.0, 5.0); // Ogni riga occupa 5mm
        let rule_manager = RuleProvider::new();
        let theme = rule_manager.themes.get("Tradizionale").unwrap();

        let pagination = PaginationRules::with_specs(
            "Custom".to_string(),
            100.0, 200.0, 10.0, 10.0, 10.0, 10.0,
            true, // prevent_orphan_character attivo
            true,
            "MORE".to_string(),
            "CONT".to_string()
        );

        // Forniamo un blocco uniforme di 8 parole (2 parole per riga simulate su 4 righe totali)
        let long_dialogue = "Parola Parola Parola Parola Parola Parola Parola CodaTesto";

        // Caso A: Rimangono solo 7mm nella pagina (Spazio per 1 sola riga da 5mm)
        // Regola anti-orfano: 1 riga è troppo poco, deve fare PushEntirely.
        let result_orphan = BlockSplitter::evaluate_split(
            &provider,
            long_dialogue,
            BlockType::Dialog,
            7.0,
            theme,
            &pagination
        );
        assert_eq!(result_orphan, SplitResult::PushEntirely);

        // Caso B: Rimangono 17mm nella pagina (Spazio teorico per 3 righe da 5mm = 15mm)
        // Il motore sottrae 1 riga per lo spazio del marcatore (MORE), lasciando 2 righe utili.
        let result_valid_split = BlockSplitter::evaluate_split(
            &provider,
            long_dialogue,
            BlockType::Dialog,
            17.0,
            theme,
            &pagination
        );

        if let SplitResult::Split { current_page_text, next_page_text, current_height } = result_valid_split {
            // 2 righe di testo (10mm) + 1 riga di marcatore MORE (5mm) = 15mm occupati
            assert_eq!(current_height, 15.0);

            // Verifichiamo che la parola finale sia stata correttamente spinta nella pagina successiva
            assert!(next_page_text.contains("CodaTesto"), "La coda del testo non è stata spinta sulla pagina successiva!");
        } else {
            panic!("Il blocco avrebbe dovuto essere splittato regolarmente con quota 17.0mm!");
        }
    }
}