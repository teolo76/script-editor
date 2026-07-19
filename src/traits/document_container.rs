// src/traits/document_container.rs

// Package Modules
use crate::traits::DocumentNode;

/// Gestisce le sole operazioni di aggregazione e indicizzazione dei figli in modo agnostico.
pub trait DocumentContainer {
    /// Il tipo di nodo figlio, vincolato a implementare l'interfaccia atomica DocumentNode.
    type Child: DocumentNode;

    /// Restituisce una slice immutabile di tutti gli elementi figli attualmente ospitati.
    fn children(&self) -> &[Self::Child];

    /// Restituisce un riferimento mutabile alla collezione interna per consentire inserimenti o rimozioni.
    fn children_mut(&mut self) -> &mut Vec<Self::Child>;

    /// Ordina i figli internamente basandosi sul loro coefficiente di ordinamento nativo.
    fn sort_children(&mut self) {
        self.children_mut().sort_by_key(|c| c.sort_order());
    }

    /// Ritorna true se il contenitore è privo di elementi figli.
    fn is_empty(&self) -> bool {
        self.children().is_empty()
    }
}


