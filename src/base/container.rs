// src/base/container.rs

use serde::{ Serialize, Deserialize };
use uuid::Uuid;

// Package Modules
use crate::common::NodeType;
use crate::base::{
    Meta,
    VisualLayer
};
use crate::traits::{
    BaseElement,
    DocumentContainer,
    DocumentNode,
    LayeredElement,
};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ContainerBase<C: DocumentNode> {
    /// Il blocco dell'identità (Gestisce BaseElement)
    pub meta: Meta,
    /// Il blocco della grafica e dei canali (Gestisce LayeredElement)
    pub visual: VisualLayer,

    /// Dati relazionali gerarchici (Gestisce DocumentNode)
    pub node_type: NodeType,
    pub parent_id: Uuid,
    pub sort_order: u32,

    /// La collezione dei figli (Gestisce DocumentContainer)
    pub children: Vec<C>,
}

// Implementazione unificata e non invasiva dei contratti per ContainerBase

// 1. Risoluzione di BaseElement (Identità e Tempo delegati a Meta)
impl<C: DocumentNode> BaseElement for ContainerBase<C> {
    fn id(&self) -> Uuid {
        self.meta.id()
    }

    fn created_at(&self) -> u64 {
        self.meta.created_at()
    }

    fn updated_at(&self) -> u64 {
        self.meta.updated_at()
    }

    fn touch(&mut self) {
        self.meta.touch();
    }
}

// 2. Risoluzione di LayeredElement (Grafica delegata a VisualLayer)
impl<C: DocumentNode> LayeredElement for ContainerBase<C> {
    fn visual_layer(&self) -> &VisualLayer {
        &self.visual
    }

    fn visual_layer_mut(&mut self) -> &mut VisualLayer {
        self.meta.touch();
        &mut self.visual
    }
}

// 3. Risoluzione di DocumentNode (Albero gerarchico)
impl<C: DocumentNode> DocumentNode for ContainerBase<C> {
    fn uuid(&self) -> &[u8; 16] {
        self.meta.id.as_bytes()
    }

    fn parent_uuid(&self) -> &[u8; 16] {
        self.parent_id.as_bytes()
    }

    fn set_parent_uuid(&mut self, parent: [u8; 16]) {
        self.parent_id = Uuid::from_bytes(parent);
        self.meta.touch();
    }

    fn sort_order(&self) -> u32 {
        self.sort_order
    }

    fn set_sort_order(&mut self, order: u32) {
        self.sort_order = order;
        self.meta.touch();
    }
}

// 4. Risoluzione di DocumentContainer (Manipolazione Raw dei Figli)
impl<C: DocumentNode> DocumentContainer for ContainerBase<C> {
    type Child = C;

    fn children(&self) -> &[Self::Child] {
        &self.children
    }

    fn children_mut(&mut self) -> &mut Vec<Self::Child> {
        &mut self.children
    }
}