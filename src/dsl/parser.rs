//! src/dsl/parser.rs

use uuid::Uuid;

// Package Modules & Files
use crate::common::BlockType;
use crate::dsl::tokens::{
    DslToken,
    DslTarget,
    DslOperator,
    DslField,
    DslCommand
};
use crate::dsl::error::DslError;

/// I nodi condizionali e logici dell'Abstract Syntax Tree (AST).
/// Completamente zero-copy grazie al lifetime `'a`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterValue<'a> {
    String(&'a str),
    Number(u32),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalOpType {
    And,
    Or,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstNode<'a> {
    /// Condizione atomica di confronto (es: type = 'dialog')
    Condition {
        field: DslField,
        operator: DslOperator,
        value: FilterValue<'a>,
    },
    /// Operatore logico binario (AND / OR)
    LogicalOp {
        op: LogicalOpType,
        left: Box<AstNode<'a>>,
        right: Box<AstNode<'a>>,
    },
    /// Operatore logico unario di negazione (NOT)
    NotOp(Box<AstNode<'a>>),
}

// La struttura principale che rappresenta l'intera query compilata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledQuery<'a> {
    pub command: DslCommand,
    pub target: DslTarget,
    pub filter_ast: Option<AstNode<'a>>,
}

impl<'a>AstNode<'a> {
    /// Valuta se un blocco reale del copione soddisfa l'albero delle condizioni logiche.
    pub fn evaluate(&self, block_type: BlockType, text: &str, line_count: usize) -> bool {
        match self {
            Self::Condition { field, operator, value } => match field {
                DslField::BlockType => {
                    if let FilterValue::String(val) = value {
                        // Mappiamo la stringa passata nel DSL al nostro enum BlockType interno
                        let target_type = match val.to_uppercase().as_str() {
                            "DIALOG" => BlockType::Dialog,
                            "CHARACTER_NAME" => BlockType::CharacterName,
                            "STAGE_DIRECTION" => BlockType::StageDirection,
                            "HEADING1" => BlockType::Heading1,
                            _ => return false,
                        };
                        match operator {
                            DslOperator::Eq => block_type == target_type,
                            _ => false, // Operatori > o < non hanno senso logico su un enum/tipo
                        }
                    } else {
                        false
                    }
                }
                DslField::WordCount => {
                    if let FilterValue::Number(limit) = value {
                        let actual_words = text.split_whitespace().count() as u32;
                        match operator {
                            DslOperator::Eq => actual_words == *limit,
                            DslOperator::Gt => actual_words > *limit,
                            DslOperator::Lt => actual_words < *limit,
                        }
                    } else {
                        false
                    }
                }
                DslField::LineCount => {
                    if let FilterValue::Number(limit) = value {
                        let actual_lines = line_count as u32;
                        match operator {
                            DslOperator::Eq => actual_lines == *limit,
                            DslOperator::Gt => actual_lines > *limit,
                            DslOperator::Lt => actual_lines < *limit,
                        }
                    } else {
                        false
                    }
                }
                DslField::CharacterName => {
                    if let FilterValue::String(expected_name) = value {
                        // Il controllo ha senso solo se il blocco è un CharacterName o un Dialog
                        if block_type == BlockType::CharacterName {
                            match operator {
                                DslOperator::Eq => text.trim().to_uppercase() == expected_name.to_uppercase(),
                                _ => false,
                            }
                        } else {
                            false // Se implementeremo metadati contestuali estenderemo qui
                        }
                    } else {
                        false
                    }
                }
            },
            Self::LogicalOp { op, left, right } => {
                let left_res = left.evaluate(block_type, text, line_count);
                let right_res = right.evaluate(block_type, text, line_count);
                match op {
                    LogicalOpType::And => left_res && right_res,
                    LogicalOpType::Or => left_res || right_res,
                }
            }
            Self::NotOp(inner) => !inner.evaluate(block_type, text, line_count),
        }
    }
}

pub struct DslParser<'a> {
    tokens: Vec<DslToken<'a>>,
    cursor: usize,
}

impl<'a> DslParser<'a> {
    pub fn new(tokens: Vec<DslToken<'a>>) -> Self {
        Self { tokens, cursor: 0 }
    }
    fn peek(&self) -> Option<&DslToken<'a>> {
        self.tokens.get(self.cursor)
    }
    fn advance(&mut self) {
        if self.cursor < self.tokens.len() {
            self.cursor += 1;
        }
    }
    /// Parsing ricorsivo delle espressioni logiche condizionali (Risolve AND / OR / NOT)
    fn parse_expression(&mut self) -> Result<AstNode<'a>, DslError> {
        let mut left_node = self.parse_primary_condition()?;

        // Ciclo per concatenare operatori logici successivi in senso lineare (left-associative)
        while let Some(token) = self.peek() {
            match token {
                DslToken::And => {
                    self.advance();
                    let right_node = self.parse_primary_condition()?;
                    left_node = AstNode::LogicalOp {
                        op: LogicalOpType::And,
                        left: Box::new(left_node),
                        right: Box::new(right_node),
                    }
                },
                DslToken::Or => {
                    self.advance();
                    let right_node = self.parse_primary_condition()?;
                    left_node = AstNode::LogicalOp {
                        op: LogicalOpType::Or,
                        left: Box::new(left_node),
                        right: Box::new(right_node),
                    }
                },
                _ => break,
            }
        }
        Ok(left_node)
    }
    /// Analizza una condizione atomica o una negazione NOT
    fn parse_primary_condition(&mut self) -> Result<AstNode<'a>, DslError> {
        // 1. Isolamento del NOT
        if std::matches!(self.peek(), Some(DslToken::Not)) {
            self.advance();
            let inner_expr = self.parse_primary_condition()?;
            return Ok(AstNode::NotOp(Box::new(inner_expr)));
        }

        // 2. Isolamento dell'estrazione del Campo
        let field = match self.peek() {
            Some(DslToken::Field(fld)) => Some(*fld),
            _ => None,
        };
        let field = match field {
            Some(fld) => { self.advance(); fld }
            None => {
                let found = self.peek().map(|t| t.to_string()).unwrap_or_else(|| "EOF".to_string());
                return Err(DslError::ParserUnexpectedToken { expected: "Un campo".to_string(), found });
            }
        };

        // 3. Isolamento dell'estrazione dell'Operatore
        let operator = match self.peek() {
            Some(DslToken::Operator(op)) => Some(*op),
            _ => None,
        };
        let operator = match operator {
            Some(op) => { self.advance(); op }
            None => {
                let found = self.peek().map(|t| t.to_string()).unwrap_or_else(|| "EOF".to_string());
                return Err(DslError::ParserUnexpectedToken {
                    expected: "Un operatore".to_string(),
                    found
                });
            }
        };

        // 4. Isolamento dell'estrazione del Valore (String o Number)
        let value = match self.peek() {
            Some(DslToken::LiteralString(val)) => Some(FilterValue::String(*val)),
            Some(DslToken::LiteralNumber(num)) => Some(FilterValue::Number(*num)),
            _ => None,
        };
        let value = match value {
            Some(val) => { self.advance(); val }
            None => {
                let found = self.peek().map(|t| t.to_string()).unwrap_or_else(|| "EOF".to_string());
                return Err(DslError::ParserUnexpectedToken {
                    expected: "Un valore letterale".to_string(),
                    found
                });
            }
        };

        // 5. Validazione semantica elementare dei tipi accoppiati
        match field {
            DslField::BlockType | DslField::CharacterName => {
                if std::matches!(value, FilterValue::Number(_)) {
                    return Err(DslError::TypeMismatch {
                        field: format!("{:?}", field),
                        expected_type: "String".to_string()
                    });
                }
            }
            DslField::WordCount | DslField::LineCount => {
                if std::matches!(value, FilterValue::String(_)) {
                    return Err(DslError::TypeMismatch {
                        field: format!("{:?}", field),
                        expected_type: "Number".to_string()
                    });
                }
            }
        }

        Ok(AstNode::Condition { field, operator, value })
    }
    /// Compila la sequenza di token nell'oggetto radice CompiledQuery.
    pub fn parse(&mut self) -> Result<CompiledQuery<'a>, DslError> {
        // 0.Salta eventuali SystemDirective residue prima di cercare il comando
        while let Some(DslToken::SystemDirective(_)) = self.peek() {
            self.advance();
        }
        
        // 1. Estrazione del Comando iniziale (Isoliamo il peek)
        let command = match self.peek() {
            Some(DslToken::Command(cmd)) => Some(*cmd),
            _ => None,
        };

        let command = match command {
            Some(cmd) => {
                self.advance(); // Sicuro: il borrow immutabile è già morto
                cmd
            }
            None => {
                let found = self.peek().map(|t| t.to_string()).unwrap_or_else(|| "EOF".to_string());
                return Err(DslError::ParserUnexpectedToken {
                    expected: "Un comando (COUNT/SUM/GET)".to_string(),
                    found
                });
            }
        };

        // 2. Estrazione del Target (Isoliamo il peek)
        let target = match self.peek() {
            Some(DslToken::Target(tgt)) => Some(*tgt),
            _ => None,
        };

        let target = match target {
            Some(tgt) => {
                self.advance(); // Sicuro
                tgt
            }
            None => {
                let found = self.peek().map(|t| t.to_string()).unwrap_or_else(|| "EOF".to_string());
                return Err(DslError::ParserUnexpectedToken {
                    expected: "Un target (blocks/scenes/acts/characters)".to_string(),
                    found
                });
            }
        };

        // 3. Controllo se è presente una clausola opzionale WHERE
        let is_where = std::matches!(self.peek(), Some(DslToken::Where));
        let mut filter_ast = None;
        if is_where {
            self.advance(); // Consuma il WHERE
            filter_ast = Some(self.parse_expression()?);
        }

        Ok(CompiledQuery { command, target, filter_ast })
    }
}

// src/reporting/dsl/parser.rs (In fondo al file)

#[cfg(test)]
mod parser_tests {
    use super::*;
    use crate::dsl::lexer::{GrammarRegistry, DslLexer};

    #[test]
    fn test_parser_compiles_complex_logical_tree() {
        let registry = GrammarRegistry::new_v1();
        let query = "COUNT blocks WHERE type = 'dialog' AND words > 5";

        let tokens = DslLexer::new(query, &registry).tokenize().unwrap();
        let mut parser = DslParser::new(tokens);
        let compiled = parser.parse().unwrap();

        assert_eq!(compiled.command, DslCommand::Count);
        assert_eq!(compiled.target, DslTarget::Blocks);

        let ast = compiled.filter_ast.unwrap();

        // Verifichiamo che l'albero abbia iniettato correttamente l'operatore logico AND in cima
        if let AstNode::LogicalOp { op, left: _, right: _ } = &ast {
            assert_eq!(*op, LogicalOpType::And);
        } else {
            panic!("Atteso un nodo di tipo LogicalOp come radice dell'espressione!");
        }

        // Test di valutazione dell'AST sui blocchi finti
        // Blocco A: Battuta di dialogo lunga -> Deve superare entrambi i filtri (TRUE)
        assert!(ast.evaluate(BlockType::Dialog, "Questa è una battuta lunga di prova.", 1));

        // Blocco B: Battuta di dialogo corta -> Fallisce il filtro delle parole (FALSE)
        assert!(!ast.evaluate(BlockType::Dialog, "No.", 1));

        // Blocco C: Una descrizione d'azione lunga -> Fallisce il filtro del tipo (FALSE)
        assert!(!ast.evaluate(BlockType::StageDirection, "Il castello crolla improvvisamente sotto la tempesta.", 1));
    }

    #[test]
    fn test_parser_type_mismatch_error() {
        let registry = GrammarRegistry::new_v1();
        let query = "GET scenes WHERE words = 'molte'"; // Errore: atteso numero per 'words'

        let tokens = DslLexer::new(query, &registry).tokenize().unwrap();
        let mut parser = DslParser::new(tokens);
        let result = parser.parse();

        assert!(std::matches!(result, Err(DslError::TypeMismatch { .. })));
    }
}