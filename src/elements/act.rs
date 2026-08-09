// src/elements/act.rs

use serde::{ Deserialize, Serialize };

// Package Modules
use crate::core::{ Container, Node };
use crate::elements::Scene;

/// Marcatore semantico per il nodo Atto (Zero-Sized Type).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Act;

/// Un contenitore Atto tipizzato, vincolato ad accogliere solo nodi di tipo `Scene`.
pub type ActContainer = Container<Act, Node<Scene>>;

impl ActContainer {
    /// Helper per creare un nuovo Atto vuoto assegnandogli un `record_id` per l'arena e un `sort_order`.
    pub fn new_act(record_id: uuid::Uuid, sort_order: u32) -> Self {
        let node = Node::<Act>::new(record_id, sort_order);
        Container::new(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_act_container_creation_and_scene_push() {
        let act_rec_id = Uuid::new_v4();
        let mut act = ActContainer::new_act(act_rec_id, 1);

        assert!(act.is_empty());
        assert_eq!(act.node.record_id, act_rec_id);

        let scene1_node = Node::<Scene>::new(Uuid::new_v4(), 1);
        let scene2_node = Node::<Scene>::new(Uuid::new_v4(), 2);

        act.push_child(scene1_node);
        act.push_child(scene2_node);

        assert_eq!(act.len(), 2);
        assert_eq!(act.children()[0], scene1_node);
        assert_eq!(act.children()[1], scene2_node);
    }

    #[test]
    fn test_act_serde_serialization() {
        let act_rec_id = Uuid::new_v4();
        let mut act = ActContainer::new_act(act_rec_id, 2);

        let scene_node = Node::<Scene>::new(Uuid::new_v4(), 1);
        act.push_child(scene_node);

        let json = serde_json::to_string(&act).expect("Serializzazione fallita");
        let deserialized: ActContainer = serde_json::from_str(&json).expect("Deserializzazione fallita");

        assert_eq!(act, deserialized);
    }
}