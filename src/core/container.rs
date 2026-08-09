// src/core/container.rs

use serde::{ Deserialize, Serialize };

// Package Modules
use crate::core::Node;

/// Gestore generico e tipizzato delle relazioni gerarchiche tra un nodo genitore e i suoi figli.
///
/// `T` rappresenta il tipo semantico del contenitore stesso (es. Script, Act, Scene).
/// `C` rappresenta il tipo ammissibile per i nodi figli (es. `Node<Scene>` per un `Act`,
/// oppure un'enum di blocchi per una `Scene`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Container<T, C> {
    /// Il nodo principale che definisce l'identità, il record_id e l'ordinamento del contenitore
    pub node: Node<T>,
    /// La collezione ordinata dei nodi figli direttamente ospitati
    pub children: Vec<C>,
}

impl<T, C> Container<T, C> {
    /// Crea un nuovo contenitore vuoto basato su un `Node<T>` esistente.
    pub fn new(node: Node<T>) -> Self {
        Self {
            node,
            children: Vec::new(),
        }
    }

    /// Crea un nuovo contenitore fornendo direttamente il nodo e la lista dei figli.
    pub fn with_children(node: Node<T>, children: Vec<C>) -> Self {
        Self { node, children }
    }

    /// Aggiunge un nodo figlio in coda alla collezione.
    pub fn push_child(&mut self, child: C) {
        self.children.push(child);
    }

    /// Inserisce un nodo figlio in una posizione specifica del vettore.
    pub fn insert_child(&mut self, index: usize, child: C) {
        let idx = index.min(self.children.len());
        self.children.insert(idx, child);
    }

    /// Rimuove un figlio in corrispondenza dell'indice specificato, restituendolo se presente.
    pub fn remove_child(&mut self, index: usize) -> Option<C> {
        if index < self.children.len() {
            Some(self.children.remove(index))
        } else {
            None
        }
    }

    /// Restituisce un riferimento immutabile alla lista dei figli.
    pub fn children(&self) -> &[C] {
        &self.children
    }

    /// Restituisce un riferimento mutabile alla lista dei figli per manipolazioni dirette.
    pub fn children_mut(&mut self) -> &mut Vec<C> {
        &mut self.children
    }

    /// Verifica se il contenitore è privo di figli.
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    /// Restituisce il numero totale di figli contenuti.
    pub fn len(&self) -> usize {
        self.children.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    // Struct dummy per i test di tipizzazione.
    // NOTA: Aggiunti Copy e Clone a tutti i tipi marcatore di test!
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    struct MockAct;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    struct MockScene;

    #[test]
    fn test_container_creation_and_push() {
        let parent_node: Node<MockAct> = Node::new(Uuid::new_v4(), 1);
        let mut container = Container::new(parent_node);

        assert!(container.is_empty());
        assert_eq!(container.len(), 0);

        let child1: Node<MockScene> = Node::new(Uuid::new_v4(), 1);
        let child2: Node<MockScene> = Node::new(Uuid::new_v4(), 2);

        container.push_child(child1);
        container.push_child(child2);

        assert_eq!(container.len(), 2);
        assert_eq!(container.children()[0], child1);
        assert_eq!(container.children()[1], child2);
    }

    #[test]
    fn test_container_insert_and_remove() {
        let parent_node: Node<MockAct> = Node::new(Uuid::new_v4(), 1);
        let mut container = Container::new(parent_node);

        let child1: Node<MockScene> = Node::new(Uuid::new_v4(), 1);
        let child2: Node<MockScene> = Node::new(Uuid::new_v4(), 2);
        let child_inserted: Node<MockScene> = Node::new(Uuid::new_v4(), 99);

        container.push_child(child1);
        container.push_child(child2);

        // Inserimento in posizione 1 (tra child1 e child2)
        container.insert_child(1, child_inserted);
        assert_eq!(container.len(), 3);
        assert_eq!(container.children()[1], child_inserted);

        // Rimozione del figlio inserito
        let removed = container.remove_child(1);
        assert_eq!(removed, Some(child_inserted));
        assert_eq!(container.len(), 2);
        assert_eq!(container.children()[1], child2);
    }

    #[test]
    fn test_serde_serialization() {
        let parent_node: Node<MockAct> = Node::new(Uuid::new_v4(), 1);
        let mut container = Container::new(parent_node);
        container.push_child(Node::<MockScene>::new(Uuid::new_v4(), 10));

        let json = serde_json::to_string(&container).expect("Serializzazione fallita");
        let deserialized: Container<MockAct, Node<MockScene>> =
            serde_json::from_str(&json).expect("Deserializzazione fallita");

        assert_eq!(container, deserialized);
    }
}