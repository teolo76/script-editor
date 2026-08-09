// src/elements/script.rs

use serde::{ Deserialize, Serialize };
use crate::core::{ Container, Node };
use crate::elements::{ Act, Cover };
use crate::elements::characters::CastSheet;

/// Marcatore semantico per il nodo Radice dell'Opera (Zero-Sized Type).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Script;

/// Enumerativo fortemente tipizzato dei soli elementi validi come figli diretti della radice `Script`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScriptChild {
    Cover(Node<Cover>),
    CastSheet(Node<CastSheet>),
    Act(Node<Act>),
}

impl ScriptChild {
    /// Restituisce l'ID del nodo contenuto indipendentemente dalla variante.
    pub fn node_id(&self) -> uuid::Uuid {
        match self {
            Self::Cover(n) => n.id,
            Self::CastSheet(n) => n.id,
            Self::Act(n) => n.id,
        }
    }

    /// Restituisce il record_id nell'arena contenuto indipendentemente dalla variante.
    pub fn record_id(&self) -> uuid::Uuid {
        match self {
            Self::Cover(n) => n.record_id,
            Self::CastSheet(n) => n.record_id,
            Self::Act(n) => n.record_id,
        }
    }
}

/// Il Master Container dell'opera, vincolato ad accogliere solo varianti di `ScriptChild`.
pub type ScriptContainer = Container<Script, ScriptChild>;

impl ScriptContainer {
    /// Helper per creare una nuova radice Script vuota.
    pub fn new_script(record_id: uuid::Uuid) -> Self {
        let node = Node::<Script>::new(record_id, 0);
        Container::new(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_script_container_hierarchy() {
        let script_rec_id = Uuid::new_v4();
        let mut script = ScriptContainer::new_script(script_rec_id);

        assert!(script.is_empty());

        let cover_node = Node::<Cover>::new(Uuid::new_v4(), 0);
        let cast_node = Node::<CastSheet>::new(Uuid::new_v4(), 1);
        let act1_node = Node::<Act>::new(Uuid::new_v4(), 2);

        script.push_child(ScriptChild::Cover(cover_node));
        script.push_child(ScriptChild::CastSheet(cast_node));
        script.push_child(ScriptChild::Act(act1_node));

        assert_eq!(script.len(), 3);
        assert_eq!(script.children()[0], ScriptChild::Cover(cover_node));
        assert_eq!(script.children()[2], ScriptChild::Act(act1_node));
    }

    #[test]
    fn test_serde_serialization() {
        let script_rec_id = Uuid::new_v4();
        let mut script = ScriptContainer::new_script(script_rec_id);
        let act_node = Node::<Act>::new(Uuid::new_v4(), 0);
        script.push_child(ScriptChild::Act(act_node));

        let json = serde_json::to_string(&script).expect("Serializzazione fallita");
        let deserialized: ScriptContainer = serde_json::from_str(&json).expect("Deserializzazione fallita");

        assert_eq!(script, deserialized);
    }
}