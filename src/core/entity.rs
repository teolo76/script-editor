// src/core/entity.rs

//! Dati anagrafici comuni a tutte le entità del documento, e l'estensione
//! `BaseElement` per quelle che vivono nell'albero (Node, Block).
//!
//! `Annotation` e `Layer` (prossimo passo) useranno solo `Meta`: non hanno
//! una posizione fra fratelli né un parent nel senso dell'albero — un layer
//! è una voce di un registro piatto, un'annotazione ancora a un elemento
//! ma non è essa stessa un nodo della gerarchia.
use crate::types::{ ElementId, Revision, SortKey, Timestamp };

/// Identità e cronologia di un'entità: id stabile, revisione locale,
/// timestamp di creazione/modifica.
///
/// La `revision` qui è quella dell'entità, non quella del documento (che
/// vivrà nell'arena/command bus): serve per confronti locali ("questa
/// versione è più recente di quella"), non per l'invalidazione della cache
/// di layout, che dipende dalla revisione globale del documento.
#[derive(Debug, Clone, PartialEq)]
pub struct Meta {
    pub id: ElementId,
    pub revision: Revision,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Meta {
    /// Nuova entità con id generato automaticamente.
    pub fn new(created_at: Timestamp) -> Self {
        Self {
            id: ElementId::new(),
            revision: Revision::INITIAL,
            created_at,
            updated_at: created_at,
        }
    }

    /// Costruisce Meta con un id esplicito. Usato solo per il nodo Root
    /// (id fisso noto a compile time) o quando si ricostruisce un'entità
    /// letta da un documento salvato su disco.
    pub fn with_id(id: ElementId, created_at: Timestamp) -> Self {
        Self {
            id,
            revision: Revision::INITIAL,
            created_at,
            updated_at: created_at,
        }
    }

    /// Da richiamare quando l'entità viene modificata: aggiorna la
    /// revisione locale e il timestamp di ultima modifica.
    pub fn touch(&mut self, rev: Revision, at: Timestamp) {
        self.revision = rev;
        self.updated_at = at;
    }
}

/// Accesso uniforme ai metadati comuni. Implementato da ogni entità del
/// documento delegando a un campo `Meta` incorporato: in Rust non c'è
/// ereditarietà, quindi "derivare da Meta" significa contenerlo ed
/// esporlo tramite questo trait, non ereditarne i campi.
pub trait Entity {
    fn meta(&self) -> &Meta;

    fn id(&self) -> ElementId {
        self.meta().id
    }

    fn revision(&self) -> Revision {
        self.meta().revision
    }

    fn created_at(&self) -> Timestamp {
        self.meta().created_at
    }

    fn updated_at(&self) -> Timestamp {
        self.meta().updated_at
    }
}

/// Dati comuni alle entità che vivono nell'albero del copione: identità
/// (`Meta`) più posizione fra fratelli (`parent`, `sort_order`).
///
/// `parent: None` significa "questo è il nodo Root", non "dato mancante":
/// non usiamo uno zero-sentinella per questo (vedi `types::ROOT_ID`, che è
/// comunque un id fisso e vero, non un caso speciale del parent).
#[derive(Debug, Clone, PartialEq)]
pub struct BaseElement {
    pub meta: Meta,
    pub parent: Option<ElementId>,
    pub sort_order: SortKey,
}

impl BaseElement {
    pub fn new(parent: Option<ElementId>, sort_order: SortKey, created_at: Timestamp) -> Self {
        Self {
            meta: Meta::new(created_at),
            parent,
            sort_order,
        }
    }

    /// Come `new`, ma con id esplicito (nodo Root, o ricostruzione da file).
    pub fn with_id(id: ElementId, parent: Option<ElementId>, sort_order: SortKey, created_at: Timestamp) -> Self {
        Self {
            meta: Meta::with_id(id, created_at),
            parent,
            sort_order,
        }
    }
}

impl Entity for BaseElement {
    fn meta(&self) -> &Meta {
        &self.meta
    }
}

/// Entità con una posizione nell'albero. Solo Node e Block la implementano:
/// Annotation e Layer restano fuori da questo trait perché non hanno un
/// posto fra fratelli nello stesso senso (vedi commento in cima al file).
pub trait TreeElement: Entity {
    fn parent(&self) -> Option<ElementId>;
    fn sort_order(&self) -> &SortKey;
}

impl TreeElement for BaseElement {
    fn parent(&self) -> Option<ElementId> {
        self.parent
    }

    fn sort_order(&self) -> &SortKey {
        &self.sort_order
    }
}
