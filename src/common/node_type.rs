// src/common/node_type.rs

use serde::{ Deserialize, Serialize };

/// Rappresenta le categorie strutturali e drammaturgiche dei nodi che compongono
/// l'albero del copione.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum NodeType {
    /// Il livello radice supremo del copione, guardiano del documento globale
    Script = 0,
    /// La copertina del copione (gestisce titoli, contatti e elementi grafici)
    Cover = 1,
    /// Il foglio del cast (l'anagrafica che associa i personaggi agli attori)
    CastSheet = 2,
    /// La macro-area di un'opera (es: Atto I, Atto II)
    Act = 3,
    /// L'unità spazio-temporale che ospita direttamente i blocchi di testo
    Scene = 4,
    /// Nodo foglia atomico appartenente a un blocco di testo (Dialogo, Didascalia, ecc.)
    Block = 5,
}

impl NodeType {
    /// Restituisce l'indice numerico progressivo del tipo di nodo.
    pub fn to_index(&self) -> usize {
        *self as usize
    }

    /// Tenta di convertire un indice numerico (0-5) nella corrispondente variante `NodeType`.
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Script),
            1 => Some(Self::Cover),
            2 => Some(Self::CastSheet),
            3 => Some(Self::Act),
            4 => Some(Self::Scene),
            5 => Some(Self::Block),
            _ => None,
        }
    }

    /// Restituisce la rappresentazione testuale statica (snake_case) della variante.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Script => "script",
            Self::Cover => "cover",
            Self::CastSheet => "cast_sheet",
            Self::Act => "act",
            Self::Scene => "scene",
            Self::Block => "block",
        }
    }

    /// Tenta di convertire una stringa nella corrispondente variante `NodeType`.
    pub fn from_str(text: &str) -> Option<Self> {
        match text.trim() {
            "script" => Some(Self::Script),
            "cover" => Some(Self::Cover),
            "cast_sheet" => Some(Self::CastSheet),
            "act" => Some(Self::Act),
            "scene" => Some(Self::Scene),
            "block" => Some(Self::Block),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_type_indices() {
        assert_eq!(NodeType::Script.to_index(), 0);
        assert_eq!(NodeType::Cover.to_index(), 1);
        assert_eq!(NodeType::CastSheet.to_index(), 2);
        assert_eq!(NodeType::Act.to_index(), 3);
        assert_eq!(NodeType::Scene.to_index(), 4);
        assert_eq!(NodeType::Block.to_index(), 5);
    }

    #[test]
    fn test_from_index_coherence() {
        let nodes = [
            NodeType::Script,
            NodeType::Cover,
            NodeType::CastSheet,
            NodeType::Act,
            NodeType::Scene,
            NodeType::Block,
        ];

        for node in nodes {
            let idx = node.to_index();
            assert_eq!(NodeType::from_index(idx), Some(node));
        }

        assert_eq!(NodeType::from_index(6), None);
        assert_eq!(NodeType::from_index(100), None);
    }

    #[test]
    fn test_as_str_and_from_str_coherence() {
        let nodes = [
            NodeType::Script,
            NodeType::Cover,
            NodeType::CastSheet,
            NodeType::Act,
            NodeType::Scene,
            NodeType::Block,
        ];

        for node in nodes {
            let str_repr = node.as_str();
            assert_eq!(NodeType::from_str(str_repr), Some(node));
        }

        assert_eq!(NodeType::from_str("invalid_node"), None);
        assert_eq!(NodeType::from_str("   scene  \n"), Some(NodeType::Scene));
    }

    #[test]
    fn test_serde_serialization() {
        let node = NodeType::Scene;
        let json = serde_json::to_string(&node).expect("Serializzazione fallita");

        let deserialized: NodeType = serde_json::from_str(&json).expect("Deserializzazione fallita");
        assert_eq!(deserialized, NodeType::Scene);
    }
}