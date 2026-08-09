// src/elements/scene.rs

use serde::{Deserialize, Serialize};

// Package Modules
use crate::core::{ Container, Node };
use crate::elements::blocks::{ SceneBlock };

/// Marcatore semantico per il nodo Scena (Zero-Sized Type).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Scene;

/// Un contenitore Scena tipizzato, vincolato ad accogliere solo varianti di `SceneBlock`.
pub type SceneContainer = Container<Scene, SceneBlock>;

impl SceneContainer {
    /// Helper per creare una nuova Scena vuota assegnandole un `record_id`
    /// per l'arena e un `sort_order`.
    pub fn new_scene(record_id: uuid::Uuid, sort_order: u32) -> Self {
        let node = Node::<Scene>::new(record_id, sort_order);
        Container::new(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use crate::elements::blocks::{ DialogBlock, StageDirectionBlock };

    #[test]
    fn test_scene_container_creation_and_block_push() {
        let scene_rec_id = Uuid::new_v4();
        let mut scene = SceneContainer::new_scene(scene_rec_id, 1);

        assert!(scene.is_empty());
        assert_eq!(scene.node.record_id, scene_rec_id);

        let dialog_node = Node::<DialogBlock>::new(Uuid::new_v4(), 1);
        let stage_node = Node::<StageDirectionBlock>::new(Uuid::new_v4(), 2);

        scene.push_child(SceneBlock::Dialog(dialog_node));
        scene.push_child(SceneBlock::StageDirection(stage_node));

        assert_eq!(scene.len(), 2);
        assert_eq!(scene.children()[0], SceneBlock::Dialog(dialog_node));
    }

    #[test]
    fn test_scene_serde_serialization() {
        let scene_rec_id = Uuid::new_v4();
        let mut scene = SceneContainer::new_scene(scene_rec_id, 5);

        let dialog_node = Node::<DialogBlock>::new(Uuid::new_v4(), 1);
        scene.push_child(SceneBlock::Dialog(dialog_node));

        let json = serde_json::to_string(&scene).expect("Serializzazione fallita");
        let deserialized: SceneContainer = serde_json::from_str(&json).expect("Deserializzazione fallita");

        assert_eq!(scene, deserialized);
    }
}