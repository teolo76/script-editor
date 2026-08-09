// src/core/node.rs

use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use uuid::Uuid;

/// Atomo di navigazione gerarchica dell'albero del documento.
///
/// Rappresenta una posizione nell'albero ed è fortemente tipizzato tramite il tipo semantico `T`.
/// Non memorizza il testo o la geometria del blocco, ma mantiene solo l'identità,
/// l'indice di ordinamento e il puntatore al `DocumentRecord` reale presente nell'arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Node<T> {
    /// Identificativo univoco del nodo all'interno della struttura ad albero
    pub id: Uuid,
    /// Puntatore all'identificativo del `DocumentRecord` memorizzato nell'arena piatta
    pub record_id: Uuid,
    /// Indice progressivo di ordinamento relativo tra nodi fratelli
    pub sort_order: u32,
    /// Marcatore a costo zero per vincolare il nodo a un tipo semantico (es. Scene, Act, Dialog)
    #[serde(skip)]
    pub _marker: PhantomData<T>,
}

impl<T> Node<T> {
    /// Crea un nuovo `Node<T>` assegnandogli un ID univoco casuale (v4).
    pub fn new(record_id: Uuid, sort_order: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            record_id,
            sort_order,
            _marker: PhantomData,
        }
    }

    /// Costruttore esplicito con ID predefinito (utile per il ripristino da I/O o deserializzazione).
    pub fn with_id(id: Uuid, record_id: Uuid, sort_order: u32) -> Self {
        Self {
            id,
            record_id,
            sort_order,
            _marker: PhantomData,
        }
    }

    /// Aggiorna l'indice di ordinamento progressivo del nodo.
    pub fn set_sort_order(&mut self, new_order: u32) {
        self.sort_order = new_order;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Struct dummy usata come marcatore di test.
    // NOTA: Deriviamo Copy e Clone affinché PhantomData<MockScene> consenta a Node<MockScene> di derivare Copy!
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct MockScene;

    #[test]
    fn test_node_creation_and_fields() {
        let record_id = Uuid::new_v4();
        let node: Node<MockScene> = Node::new(record_id, 10);

        assert_ne!(node.id, Uuid::nil());
        assert_eq!(node.record_id, record_id);
        assert_eq!(node.sort_order, 10);
    }

    #[test]
    fn test_node_with_id() {
        let custom_id = Uuid::new_v4();
        let record_id = Uuid::new_v4();
        let node: Node<MockScene> = Node::with_id(custom_id, record_id, 5);

        assert_eq!(node.id, custom_id);
        assert_eq!(node.record_id, record_id);
        assert_eq!(node.sort_order, 5);
    }

    #[test]
    fn test_set_sort_order() {
        let mut node: Node<MockScene> = Node::new(Uuid::new_v4(), 0);
        node.set_sort_order(42);
        assert_eq!(node.sort_order, 42);
    }

    #[test]
    fn test_node_copy_and_clone() {
        let node1: Node<MockScene> = Node::new(Uuid::new_v4(), 1);
        let node2 = node1; // Ora è una vera copia bit-a-bit poiché MockScene è Copy!

        assert_eq!(node1, node2);
    }

    #[test]
    fn test_serde_serialization() {
        let record_id = Uuid::new_v4();
        let node: Node<MockScene> = Node::new(record_id, 3);

        let json = serde_json::to_string(&node).expect("Serializzazione fallita");
        let deserialized: Node<MockScene> = serde_json::from_str(&json).expect("Deserializzazione fallita");

        assert_eq!(node.id, deserialized.id);
        assert_eq!(node.record_id, deserialized.record_id);
        assert_eq!(node.sort_order, deserialized.sort_order);
    }
}