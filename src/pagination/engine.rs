//! src/pagination/engine.rs

use uuid::Uuid;
use std::collections::HashMap;
use serde::{ Serialize, Deserialize };

// Package Modules & Files
use crate::common::BlockType;
use crate::common::rules::{ ThemeRules, PaginationRules };
use crate::pagination::{ BlockSplitter, FontMetricProvider, SplitResult, TextLayoutMetrics };

/// Rappresenta un blocco di testo già posizionato geometricamente all'interno di una pagina
/// specifica.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LayoutedBlock {
    pub block_id: Uuid,
    pub block_type: BlockType,
    /// Il testo effettivo (può essere parziale se il blocco originale è stato diviso)
    pub text: String,
    /// L'altezza millimetrica occupata da questo blocco nella pagina (incluso overhead verticale)
    pub height_mm: f32,
}

/// Rappresenta una singola pagina discreta generata dal motore di impaginazione.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Page {
    pub page_number: usize,
    /// Lista ordinata dei blocchi geometrici che appartengono a questa pagina
    pub blocks: Vec<LayoutedBlock>,
    /// Spazio verticale totale occupato in questa pagina (in mm)
    pub accumulated_height_mm: f32,
}

/// Il documento finale impaginato pronto per essere serializzato verso la UI di Tauri.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct LayoutedDocument {
    pub pages: Vec<Page>,
    pub total_pages: usize,
}

pub struct InterleaveEngine;

impl InterleaveEngine {
    /// Prende una lista piatta di blocchi (ID e Testo) ed esegue la paginazione
    /// lineare sequenziale.
    pub fn paginate(
        provider: &dyn FontMetricProvider,
        raw_blocks: &[(Uuid, BlockType, &str)],
        theme: &ThemeRules,
        pagination: &PaginationRules,
    ) -> LayoutedDocument {
        let mut document = LayoutedDocument::default();
        let max_page_height = pagination.printable_height();

        let mut current_page = Page {
            page_number: 1,
            blocks: Vec::new(),
            accumulated_height_mm: 0.0,
        };
        // Utilizziamo un iteratore manuale per poter reinserire i blocchi divisi in cima alla coda
        let mut block_queue: Vec<(Uuid, BlockType, String)> = raw_blocks
            .iter()
            .map(|(id, b_type, text)| (*id, *b_type, text.to_string()))
            .collect();
        // Diventa una coda (LIFO tramite pop)
        block_queue.reverse();

        while let Some((id, b_type, text)) = block_queue.pop() {
            let remaining_height = max_page_height - current_page.accumulated_height_mm;

            // Chiediamo allo splitter di valutare la geometria del blocco rispetto allo s
            // pazio residuo
            let split_decision = BlockSplitter::evaluate_split(
                provider,
                &text,
                b_type,
                remaining_height,
                theme,
                pagination,
            );

            match split_decision {
                // Il blocco entra tutto. Calcoliamo l'altezza reale inclusi i margini
                SplitResult::FitsEntirely => {
                    let metrics = provider.compute_metrics(&text, b_type, theme, pagination).unwrap_or_else(|_| {
                        TextLayoutMetrics { total_height: 0.0, line_count: 1}
                    });
                    let geometry =theme.geometries.get(&b_type);
                    let overhead=geometry.map(|g| g.space_before + g.space_after).unwrap_or(0.0);
                    let block_height = metrics.total_height + overhead;

                    current_page.blocks.push(LayoutedBlock {
                        block_id: id,
                        block_type: b_type,
                        text,
                        height_mm: block_height,
                    });
                    current_page.accumulated_height_mm += block_height;
                }
                // Il blocco va spinto alla pagina successiva.
                SplitResult::PushEntirely => {
                    // Chiudiamo la pagina attuale (se non è vuota) e creiamo la nuova.
                    if !current_page.blocks.is_empty() {
                        document.pages.push(current_page.clone());
                        current_page = Page {
                            page_number: current_page.page_number +1,
                            blocks: Vec::new(),
                            accumulated_height_mm: 0.0,
                        };
                    }
                    // Reinseriamo lo stesso identico blocco in cima alla coda per la nuova pagina
                    block_queue.push((id, b_type, text));
                }
                // Il blocco viene spezzato a metà.
                // Modifica interna a src/pagination/engine.rs (Dentro il metodo InterleaveEngine::paginate)

                SplitResult::Split { mut current_page_text, next_page_text, current_height } => {
                    // 1. Se stiamo splittando un dialogo, iniettiamo il marcatore di continuità (MORE) a fine testo
                    if b_type == BlockType::Dialog {
                        current_page_text.push_str("\n");
                        current_page_text.push_str(&pagination.more_marker);
                    }

                    current_page.blocks.push(LayoutedBlock {
                        block_id: id,
                        block_type: b_type,
                        text: current_page_text,
                        height_mm: current_height,
                    });
                    current_page.accumulated_height_mm += current_height;

                    // Chiudiamo la pagina corrente
                    document.pages.push(current_page.clone());

                    // 2. Apriamo la pagina successiva
                    current_page = Page {
                        page_number: current_page.page_number + 1,
                        blocks: Vec::new(),
                        accumulated_height_mm: 0.0,
                    };

                    // 3. REGOLA CINEMATOGRAFICA: Se abbiamo spezzato un dialogo, la nuova pagina DEVE
                    // aprirsi ripetendo il nome del personaggio orfano seguito da (CONT.) prima di riprendere il testo.
                    if b_type == BlockType::Dialog {
                        // Generiamo un ID fittizio per il blocco di continuità visiva della UI
                        let cont_block_id = Uuid::new_v4();
                        let cont_text = format!("PERSONAGGIO {}", pagination.cont_marker);

                        let metrics_cont = provider.compute_metrics(&cont_text, BlockType::CharacterName, theme, pagination).unwrap_or_else(|_| {
                            crate::pagination::font_provider::TextLayoutMetrics { total_height: 4.23, line_count: 1 }
                        });

                        current_page.blocks.push(LayoutedBlock {
                            block_id: cont_block_id,
                            block_type: BlockType::CharacterName,
                            text: cont_text,
                            height_mm: metrics_cont.total_height,
                        });
                        current_page.accumulated_height_mm += metrics_cont.total_height;
                    }

                    // 4. Reinseriamo il testo rimasto in cima alla coda della nuova pagina
                    block_queue.push((id, b_type, next_page_text));
                }
            }
        }
        // Se l'ultima pagina contiene blocchi, la salviamo nel documento
        if !current_page.blocks.is_empty() {
            document.pages.push(current_page);
        }

        document.total_pages = document.pages.len();
        document
    }
}

// src/pagination/engine.rs (In fondo al file)

#[cfg(test)]
mod interleave_engine_tests {
    use super::*;
    use uuid::Uuid;
    use crate::common::BlockType;
    use crate::common::rules::{RuleProvider, PaginationRules};
    use crate::pagination::font_provider::FixedFontProvider;

    #[test]
    fn test_interleave_engine_simple_accumulation() {
        let provider = FixedFontProvider::new();
        let rule_manager = RuleProvider::new();
        let theme = rule_manager.themes.get("Tradizionale").unwrap();
        let pagination = rule_manager.paginations.get("A4").unwrap();

        let block_1_id = Uuid::new_v4();
        let block_2_id = Uuid::new_v4();

        let input_blocks = vec![
            (block_1_id, BlockType::Dialog, "Prima battuta breve."),
            (block_2_id, BlockType::Dialog, "Seconda battuta breve."),
        ];

        let doc = InterleaveEngine::paginate(&provider, &input_blocks, theme, pagination);

        // Verifica che i blocchi accumulati stiano tutti nella prima pagina
        assert_eq!(doc.total_pages, 1);
        assert_eq!(doc.pages[0].blocks.len(), 2);
        assert_eq!(doc.pages[0].blocks[0].block_id, block_1_id);
    }

    #[test]
    fn test_dialogue_splitting_injects_more_and_continued_markers() {
        let provider = FixedFontProvider::with_specs(2.0, 5.0); // Ogni riga = 5mm
        let rule_manager = RuleProvider::new();
        let theme = rule_manager.themes.get("Tradizionale").unwrap();

        // Pagina con area utile restrittiva a 40mm netti
        let pagination = PaginationRules::with_specs(
            "Hollywood-A4".to_string(),
            100.0, 60.0, 10.0, 10.0, 10.0, 10.0,
            true, true,
            "(MORE)".to_string(),
            "(CONT.)".to_string()
        );

        let dialog_id = Uuid::new_v4();
        let long_dialog_text = "Uno Due Tre Quattro Cinque Sei Sette Otto Nove Dieci Undici Dodici";

        let input_blocks = vec![
            (dialog_id, BlockType::Dialog, long_dialog_text),
        ];

        let doc = InterleaveEngine::paginate(&provider, &input_blocks, theme, &pagination);

        // Validazione della scomposizione cinematografica
        assert_eq!(doc.total_pages, 2);

        // Pagina 1 deve chiudersi iniettando il token (MORE) nel testo
        let p1_block = &doc.pages[0].blocks[0];
        assert!(p1_block.text.contains("(MORE)"), "Manca il marcatore (MORE) a fine pagina!");

        // Pagina 2 deve aprirsi automaticamente con il blocco di continuità visiva (CONT.)
        let p2_first_block = &doc.pages[1].blocks[0];
        assert_eq!(p2_first_block.block_type, BlockType::CharacterName);
        assert!(p2_first_block.text.contains("(CONT.)"), "Manca il marcatore (CONT.) ad inizio pagina!");

        // Pagina 2 deve contenere in seconda posizione la coda del dialogo originale
        let p2_second_block = &doc.pages[1].blocks[1];
        assert_eq!(p2_second_block.block_id, dialog_id);
        assert_eq!(p2_second_block.block_type, BlockType::Dialog);
    }
}
