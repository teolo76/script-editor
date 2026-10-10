// src/core/doc_meta.rs

//! Metadati di progetto che esistono in una sola copia per copione:
//! titolo, sinossi, copertina. A differenza di Character/Layer non sono
//! entità con un proprio `ElementId` — non ha senso cercarli per id o
//! tenerli in un registro, ce n'è uno solo per documento — quindi vivono
//! come campo diretto su `Document`, non in un lake.
//!
//! identità, produzione, diritti, stato editoriale, copertina. A
//! differenza di Character/Layer non sono entità con un proprio
//! `ElementId` — non ha senso cercarli per id o tenerli in un registro, ce
//! n'è uno solo per documento — quindi vivono come campo diretto su
//! `Document`, non in un lake.
//!
//! I campi sono raggruppati in piccole struct (`Authorship`, `Production`)
//! quando un insieme di campi tende a essere editato e salvato insieme
//! (tipicamente una sezione di un form nel `CoverEditor`): è lo stesso
//! raggruppamento che useranno i comandi che li mutano
//! (`Command::SetDocAuthorship`, `SetDocProduction`, ...), per cui
//! l'`Undo` di un comando del genere può limitarsi a salvare l'intero
//! blocco precedente invece che campo per campo. Un gruppo con un solo
//! campo (es. il copyright) resta un campo diretto, senza wrapper.

use crate::types::{Dimension, Copyright, ProjectStatus, Version };

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DocMeta {
    pub title: Option<String>,
    pub cover: Option<CoverImage>,
    pub synopsis: Option<String>,

    /// Tempo e luogo dell'azione (es. "Verona, XIV secolo"), tipico della
    /// prima pagina di un copione pubblicato.
    pub setting: Option<String>,
    pub authorship: Authorship,
    pub production: Production,
    pub copyright: Option<Copyright>,

    /// Ruoli non coperti da campi fissi (scenografo, costumista,
    /// compositore...): coppie libere `(ruolo, nome)`. Un campo dedicato
    /// per ognuno farebbe esplodere questa struct per ruoli che magari non
    /// servono mai a un dato progetto.
    pub credits: Vec<(String, String)>,

    pub status: ProjectStatus,
    /// Es. "Non riprodurre senza autorizzazione", frequente sulle bozze
    /// pre-produzione. Concettualmente vicino a `status` (entrambi
    /// regolano come il documento può circolare), per questo nello stesso
    /// comando (`SetDocStatus`).
    pub distribution_note: Option<String>,

    /// Vedi `types::Version`: `sub` avanza fuori dal command bus, `major`
    /// passa da `Command::SetDocVersionMajor`.
    pub version: Version,
}

/// Chi ha scritto il testo. Raggruppata a parte perché tende a essere
/// fissa una volta stabilita — a differenza di `Production`, che può
/// cambiare a ogni nuovo allestimento dello stesso testo.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Authorship {
    /// Plurale da subito: la co-autorialità è comune.
    pub authors: Vec<String>,
    pub translator: Option<String>,
    /// Fonte di un adattamento, se il testo non è originale — rilevante
    /// anche ai fini dei diritti.
    pub based_on: Option<String>,
}

/// Informazioni legate a una specifica messa in scena.
///
/// Limite noto e voluto per questa fase: un solo `Production` per
/// documento presume una sola regia attiva. Se in futuro serve tracciare
/// più allestimenti dello stesso testo nel tempo, questo andrà spostato in
/// una struttura `Production` ripetibile (con il proprio `ElementId`),
/// non duplicato qui — per ora teniamo il caso semplice.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Production {
    pub director: Option<String>,
    pub commissioned_by: Option<String>,
    /// Data/teatro/compagnia della prima rappresentazione, come nota
    /// storica in prosa libera — non un tipo data dedicato: qui serve
    /// flessibilità editoriale più che validazione.
    pub premiere: Option<String>,
}

/// Copertina del copione. Niente `Placement`: a differenza di un
/// `AssetBlock`, la copertina non vive nel flusso del testo — è una pagina
/// a sé, che il layout tratterà come caso speciale, non come un blocco fra
/// gli altri.
#[derive(Debug, Clone, PartialEq)]
pub struct CoverImage {
    pub source: String,
    pub width: Option<Dimension>,
    pub height: Option<Dimension>,
}