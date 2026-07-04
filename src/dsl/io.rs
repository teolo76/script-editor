//! src/dsl/io.rs

use std::fs::{ File, read_to_string };
use std::io::Write;
use std::path::{ Path, PathBuf };

// Package Modules & Files
use crate::common::constants::QUERY_EXT;
use crate::dsl::{
    DslParser,
    DslLexer,
    GrammarRegistry,
    tokens::DslToken,
};

pub struct QScriptStorage;

impl QScriptStorage {
    /// Valida una query DSL e, se corretta, la salva in un file con estensione `.qscript`.
    /// Se la query contiene commenti o spazi extra, vengono preservati nel file originale.
    pub fn save_query<P: AsRef<Path>>(path: P, query_text: &str, registry: &GrammarRegistry,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Validazione preventiva
        let tokens = DslLexer::new(query_text, registry).tokenize()?;

        // Rimuoviamo i token di direttiva prima di darli al parser sintattico
        let clean_tokens: Vec<DslToken> = tokens.iter()
            .filter(|t| !matches!(t, DslToken::SystemDirective(_)))
            .cloned()
            .collect();

        let mut parser = DslParser::new(clean_tokens);
        let _compiled = parser.parse()?;

        // 2. Costruzione brutale e sicura del percorso stringa
        let path_str = path.as_ref().to_string_lossy();
        let final_path = if path_str.ends_with(QUERY_EXT) {
            path_str.into_owned()
        } else {
            format!("{}.{}", path_str,QUERY_EXT)
        };

        // 3. Scrittura fisica
        let mut file = File::create(final_path)?;
        file.write_all(query_text.as_bytes())?;
        Ok(())
    }
    /// Carica il testo di una query da un file `.qscript` e lo valida prima di restituirlo.
    pub fn load_query<P: AsRef<Path>>(path: P, registry: &GrammarRegistry,
    ) -> Result<(String, Option<u32>), Box<dyn std::error::Error>> {
        let content = read_to_string(path)?;

        // Eseguiamo il lexing completo per estrarre le SystemDirective
        let tokens = DslLexer::new(&content, registry).tokenize()?;

        let mut detected_version = None;
        for token in &tokens {
            if let DslToken::SystemDirective(directive_text) = token {
                if directive_text.starts_with("grammar_version:") {
                    if let Some(ver_str) = directive_text.split(':').nth(1) {
                        if let Ok(ver) = ver_str.trim().parse::<u32>() {
                            detected_version = Some(ver);
                        }
                    }
                }
            }
        }

        // Puliamo i token per il parser dell'AST
        let clean_tokens: Vec<DslToken> = tokens.into_iter()
            .filter(|t| !matches!(t, DslToken::SystemDirective(_)))
            .collect();

        let mut parser = DslParser::new(clean_tokens);
        let _compiled = parser.parse()?;

        Ok((content, detected_version))
    }
}

#[cfg(test)]
mod storage_tests {
    use super::*;
    use std::fs::{remove_file, read_to_string};

    #[test]
    fn test_storage_lifecycle_with_system_directives() {
        let registry = GrammarRegistry::new_v1();
        let test_path = "test_report_pipeline.qscript";

        let query_text = "#! grammar_version: 1\n#! author: Marco\n# Cerca dialoghi complessi\nCOUNT blocks WHERE type = 'dialog' AND words > 10";

        // 1. Test di salvataggio (Se fallisce qui, stampiamo l'errore atomico)
        if let Err(e) = QScriptStorage::save_query(test_path, query_text, &registry) {
            panic!("Il metodo save_query ha fallito la validazione interna: {}", e);
        }

        // 2. Verifica lettura grezza per assicurarsi che esista su disco
        let raw_file_content = read_to_string(test_path);
        assert!(raw_file_content.is_ok(), "Il file .qscript NON esiste su disco. Il salvataggio è fallito silenziosamente.");
        assert_eq!(raw_file_content.unwrap(), query_text);

        // 3. Test di caricamento tramite il nuovo metodo load_query
        let load_res = QScriptStorage::load_query(test_path, &registry);
        assert!(load_res.is_ok(), "Il metodo load_query ha fallito: {:?}", load_res);

        let (content, version) = load_res.unwrap();
        assert_eq!(content, query_text);
        assert_eq!(version, Some(1), "La versione della grammatica non è stata estratta correttamente.");

        // Pulizia finale
        let _ = remove_file(test_path);
    }

    #[test]
    fn test_storage_prevents_saving_corrupted_query() {
        let registry = GrammarRegistry::new_v1();
        let invalid_path = "test_invalid_query.qscript";
        let corrupted_query = "#! grammar_version: 1\nCOUNT WHERE type = 'dialog'";

        let save_res = QScriptStorage::save_query(invalid_path, corrupted_query, &registry);
        assert!(save_res.is_err(), "Lo storage ha salvato una query corrotta!");

        assert!(read_to_string(invalid_path).is_err(), "Il file corrotto è stato scritto su disco.");
    }
}