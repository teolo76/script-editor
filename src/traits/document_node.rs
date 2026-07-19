// src/traits/document_node.rs

/// Il contratto fondamentale per qualsiasi entità che abiti l'albero semantico del documento.
/// Garantisce l'identità univoca, la navigabilità gerarchica e la coerenza dell'ordinamento.

pub trait DocumentNode {
    /// Restituisce i 16 byte dell'UUID binario.
    fn uuid(&self) -> &[u8; 16];

    /// Restituisce l'UUID del nodo genitore.
    fn parent_uuid(&self) -> &[u8; 16];

    /// Assegna il genitore logico.
    fn set_parent_uuid(&mut self, parent: [u8; 16]);

    /// Restituisce l'indice numerico di ordinamento relativo tra fratelli.
    fn sort_order(&self) -> u32;

    /// Aggiorna l'indice di ordinamento.
    fn set_sort_order(&mut self, order: u32);
}