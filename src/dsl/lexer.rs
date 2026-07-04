//! src/dsl/lever.rs

use std::collections::HashMap;

// Package Modules & Files
use crate::dsl::tokens::{
    DslCommand,
    DslField,
    DslOperator,
    DslTarget,
    DslToken
};
use crate::dsl::error::DslError;

/// Registro che memorizza la HashTable delle parole chiave caricate dal file JSON di grammatica.
pub struct GrammarRegistry {
    pub version: u32,
    keywords: HashMap<String, DslToken<'static>>,
}

impl GrammarRegistry {
    /// Inizializza il registro popolando la HashTable con le specifiche della grammatica V1.
    /// In produzione, questa mappa viene idratata leggendo dinamicamente il file in `../data/grammar/v1.grammar.json`.
    pub fn new_v1() -> Self {
        let mut keywords = HashMap::new();

        // Comandi
        keywords.insert("COUNT".to_string(), DslToken::Command(DslCommand::Count));
        keywords.insert("SUM".to_string(), DslToken::Command(DslCommand::Sum));
        keywords.insert("GET".to_string(), DslToken::Command(DslCommand::Get));

        // Target
        keywords.insert("ACTS".to_string(), DslToken::Target(DslTarget::Acts));
        keywords.insert("BLOCKS".to_string(), DslToken::Target(DslTarget::Blocks));
        keywords.insert("SCENES".to_string(), DslToken::Target(DslTarget::Scenes));
        keywords.insert("CHARACTERS".to_string(), DslToken::Target(DslTarget::Characters));

        // Connettivi e Clausole
        keywords.insert("WHERE".to_string(), DslToken::Where);
        keywords.insert("AND".to_string(), DslToken::And);
        keywords.insert("OR".to_string(), DslToken::Or);
        keywords.insert("NOT".to_string(), DslToken::Not);

        // Campi
        keywords.insert("TYPE".to_string(), DslToken::Field(DslField::BlockType));
        keywords.insert("WORDS".to_string(), DslToken::Field(DslField::WordCount));
        keywords.insert("CHARACTER".to_string(), DslToken::Field(DslField::CharacterName));
        keywords.insert("LINES".to_string(), DslToken::Field(DslField::LineCount));

        // Operatori
        keywords.insert("=".to_string(), DslToken::Operator(DslOperator::Eq));
        keywords.insert(">".to_string(), DslToken::Operator(DslOperator::Gt));
        keywords.insert("<".to_string(), DslToken::Operator(DslOperator::Lt));

        Self {
            version: 1,
            keywords,
        }
    }
    /// Esegue il lookup in O(1) per verificare se la slice appartiene a una parola chiave registrata.
    pub fn lookup(&self, word: &str) -> Option<DslToken<'static>> {
        self.keywords.get(&word.to_uppercase()).cloned()
    }
}

pub struct DslLexer<'a> {
    input: &'a str,
    registry: &'a GrammarRegistry,
}

impl<'a> DslLexer<'a> {
    pub fn new(input: &'a str, registry: &'a GrammarRegistry) -> Self {
        Self { input, registry }
    }
    /// Scansiona l'input ed estrae i token a Zero-Allocation.
    pub fn tokenize(&self) -> Result<Vec<DslToken<'a>>, DslError> {
        let mut tokens = Vec::new();
        let chars: Vec<(usize, char)> = self.input.char_indices().collect();
        let mut i = 0;

        while i < chars.len() {
            let (start_idx, c) = chars[i];

            // 1. Salta gli spazi bianchi
            if c.is_whitespace() {
                i += 1;
                continue;
            }

            // 2.GESTIONE DIRETTIVE DI SISTEMA (#!) VS COMMENTI NORMALI (#)
            if c == '#' {
                if i + 1 < chars.len() && chars[i + 1].1 == '!' {
                    // Trovato '#!' -> Estraiamo la direttiva fino a fine riga
                    let start_directive = chars[i + 2].0;
                    // CORRETTO: '\n' reale anziché '\\n'
                    while i < chars.len() && chars[i].1 != '\n' {
                        i += 1;
                    }
                    let end_directive = chars[i].0;
                    let slice = &self.input[start_directive..end_directive];
                    tokens.push(DslToken::SystemDirective(slice.trim()));
                } else {
                    // Trovato '#' normale -> Salta l'intera riga (commento utente)
                    // CORRETTO: '\n' reale anziché '\\n'
                    while i < chars.len() && chars[i].1 != '\n' {
                        i += 1;
                    }
                }
                continue;
            }

            // 3. Gestione dei simboli singoli speciali (Operatori di confronto)
            if c == '=' || c == '>' || c == '<' {
                if let Some(token) = self.registry.lookup(&c.to_string()) {
                    tokens.push(token);
                }
                i += 1;
                continue;
            }

            // 4. GESTIONE ZERO-COPY STRING LITERALS: Identifica il testo tra apici '...'
            if c == '\'' {
                let mut end_idx = None;
                i += 1; // Salta l'apice iniziale

                while i < chars.len() {
                    if chars[i].1 == '\'' {
                        end_idx = Some(chars[i].0);
                        break;
                    }
                    i += 1;
                }

                let end_pos = end_idx.ok_or(DslError::UnterminatedStringLiteral)?;
                let literal_slice = &self.input[start_idx + 1..end_pos];

                tokens.push(DslToken::LiteralString(literal_slice));
                i += 1; // Salta l'apice finale
                continue;
            }

            // 4. Estrazione di parole alfanumeriche o numeri continui
            let mut word_end = start_idx;
            while word_end < self.input.len() {
                let current_char = self.input.as_bytes()[word_end] as char;
                if current_char.is_whitespace() || current_char == '=' || current_char == '>' || current_char == '<' || current_char == '\'' || current_char == ',' || current_char == '#'{
                    break;
                }
                word_end += 1;
            }

            let word_slice = &self.input[start_idx..word_end];
            i += word_slice.chars().count();

            if word_slice.is_empty() {
                continue;
            }

            // 5. Lookup HashTable delle parole chiave o parsing numerico in-place
            if let Some(token) = self.registry.lookup(word_slice) {
                tokens.push(token);
            } else if let Ok(num) = word_slice.parse::<u32>() {
                tokens.push(DslToken::LiteralNumber(num));
            } else {
                return Err(DslError::LexerUnknownToken(word_slice.to_string()));
            }
        }

        Ok(tokens)
    }
}

// src/reporting/dsl/lexer.rs (In fondo al file)

#[cfg(test)]
mod lexer_tests {
    use super::*;

    #[test]
    fn test_lexer_zero_copy_and_hashtable_lookup() {
        let registry = GrammarRegistry::new_v1();
        let query = "COUNT blocks WHERE type = 'dialog' AND words > 100";

        let lexer = DslLexer::new(query, &registry);
        let tokens = lexer.tokenize().unwrap();

        // Verifica sequenza dei token
        assert_eq!(tokens[0], DslToken::Command(DslCommand::Count));
        assert_eq!(tokens[1], DslToken::Target(DslTarget::Blocks));
        assert_eq!(tokens[2], DslToken::Where);
        assert_eq!(tokens[3], DslToken::Field(DslField::BlockType));
        assert_eq!(tokens[4], DslToken::Operator(DslOperator::Eq));

        // Verifica estrazione Zero-Copy della stringa tra apici
        assert_eq!(tokens[5], DslToken::LiteralString("dialog"));

        assert_eq!(tokens[6], DslToken::And);
        assert_eq!(tokens[7], DslToken::Field(DslField::WordCount));
        assert_eq!(tokens[8], DslToken::Operator(DslOperator::Gt));

        // Verifica conversione numerica in-place nello stack
        assert_eq!(tokens[9], DslToken::LiteralNumber(100));
    }

    #[test]
    fn test_lexer_unterminated_string_error() {
        let registry = GrammarRegistry::new_v1();
        let query = "GET scenes WHERE character = 'MARCO"; // Manca l'apice di chiusura

        let lexer = DslLexer::new(query, &registry);
        let result = lexer.tokenize();

        assert_eq!(result, Err(DslError::UnterminatedStringLiteral));
    }

    // Inserire nel modulo mod lexer_tests in fondo a src/reporting/dsl/lexer.rs

    #[test]
    fn test_lexer_handles_comments_correctly() {
        let registry = GrammarRegistry::new_v1();

        // Stringa di query che include commenti in mezzo e alla fine
        let query_with_comments = "
            # Questo è un commento iniziale da saltare
            COUNT blocks WHERE type = 'dialog' # Filtra solo le battute
            # Commento finale orfano
        ";

        let lexer = DslLexer::new(query_with_comments, &registry);
        let tokens = lexer.tokenize().unwrap();

        // Ci aspettiamo che vengano estratti solo i token utili della query centrale
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0], DslToken::Command(DslCommand::Count));
        assert_eq!(tokens[1], DslToken::Target(DslTarget::Blocks));
        assert_eq!(tokens[2], DslToken::Where);
        assert_eq!(tokens[3], DslToken::Field(DslField::BlockType));
        assert_eq!(tokens[4], DslToken::Operator(DslOperator::Eq));
        assert_eq!(tokens[5], DslToken::LiteralString("dialog"));
    }
}