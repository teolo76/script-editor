// src/command/doc_meta_cmd.rs

//! Comandi su `DocMeta`: raggruppati per sezione (vedi il commento in
//! cima a `commands::command`) invece che per singolo campo. Non hanno
//! un proprio `ElementId` — `DocMeta` è singleton per documento — quindi
//! `Command::classify` restituisce `None` per questi e `Delta` li marca
//! con `meta_changed` invece di un id.
//!
//! Le varianti non portano il prefisso "Set" (a differenza dei
//! costruttori pubblici `Command::set_doc_*`, che lo mantengono per
//! leggibilità lato chiamante): annidate sotto `DocMetaCommand`, il
//! prefisso sarebbe ridondante — è quello che segnalava clippy
//! (`enum_variant_names`).

use crate::core::{Authorship, CoverImage, Production};
use crate::errors::CoreResult;
use crate::lake::Document;
use crate::types::{Copyright, ProjectStatus, Timestamp, Version};

#[derive(Debug, PartialEq)]
pub enum DocMetaCommand {
    Title(Option<String>),
    Synopsis(Option<String>),
    Setting(Option<String>),
    Cover(Option<CoverImage>),
    Authorship(Authorship),
    Production(Production),
    Rights(Option<Copyright>),
    Credits(Vec<(String, String)>),
    Status {
        status: ProjectStatus,
        distribution_note: Option<String>,
    },
    /// Decisione deliberata dell'autore: passa dal bus con undo, a
    /// differenza di `Document::bump_version_sub` (bookkeeping
    /// automatico, fuori dal bus). Per convenzione azzera il minor —
    /// vedi `types::Version`.
    VersionMajor(u32),
    /// Correzione manuale del minor, distinta dall'incremento automatico.
    VersionSub(u32),
}

#[derive(Debug, PartialEq)]
pub(crate) enum DocMetaUndo {
    Title(Option<String>),
    Synopsis(Option<String>),
    Setting(Option<String>),
    Cover(Option<CoverImage>),
    Authorship(Authorship),
    Production(Production),
    Rights(Option<Copyright>),
    Credits(Vec<(String, String)>),
    Status {
        status: ProjectStatus,
        distribution_note: Option<String>,
    },
    /// Ripristina l'intera coppia (major, minor) precedente. Riusato sia
    /// da `VersionMajor` sia da `VersionSub`: entrambi mutano lo stesso
    /// `Version`, quindi l'inverso è sempre "rimetti il Version di
    /// prima", non un campo alla volta.
    Version(Version),
}

pub(crate) fn apply(cmd: &DocMetaCommand, doc: &mut Document, _at: Timestamp) -> CoreResult<DocMetaUndo> {
    match cmd {
        DocMetaCommand::Title(title) => {
            let previous = doc.metadata.title.clone();
            doc.metadata.title = title.clone();
            Ok(DocMetaUndo::Title(previous))
        }
        DocMetaCommand::Synopsis(synopsis) => {
            let previous = doc.metadata.synopsis.clone();
            doc.metadata.synopsis = synopsis.clone();
            Ok(DocMetaUndo::Synopsis(previous))
        }
        DocMetaCommand::Setting(setting) => {
            let previous = doc.metadata.setting.clone();
            doc.metadata.setting = setting.clone();
            Ok(DocMetaUndo::Setting(previous))
        }
        DocMetaCommand::Cover(cover) => {
            let previous = doc.metadata.cover.clone();
            doc.metadata.cover = cover.clone();
            Ok(DocMetaUndo::Cover(previous))
        }
        DocMetaCommand::Authorship(authorship) => {
            let previous = doc.metadata.authorship.clone();
            doc.metadata.authorship = authorship.clone();
            Ok(DocMetaUndo::Authorship(previous))
        }
        DocMetaCommand::Production(production) => {
            let previous = doc.metadata.production.clone();
            doc.metadata.production = production.clone();
            Ok(DocMetaUndo::Production(previous))
        }
        DocMetaCommand::Rights(copyright) => {
            let previous = doc.metadata.copyright.clone();
            doc.metadata.copyright = copyright.clone();
            Ok(DocMetaUndo::Rights(previous))
        }
        DocMetaCommand::Credits(credits) => {
            let previous = doc.metadata.credits.clone();
            doc.metadata.credits = credits.clone();
            Ok(DocMetaUndo::Credits(previous))
        }
        DocMetaCommand::Status { status, distribution_note } => {
            let previous_status = doc.metadata.status;
            let previous_note = doc.metadata.distribution_note.clone();
            doc.metadata.status = *status;
            doc.metadata.distribution_note = distribution_note.clone();
            Ok(DocMetaUndo::Status { status: previous_status, distribution_note: previous_note })
        }
        DocMetaCommand::VersionMajor(major) => {
            let previous = doc.metadata.version;
            doc.metadata.version = Version::new(*major, 0);
            Ok(DocMetaUndo::Version(previous))
        }
        DocMetaCommand::VersionSub(sub) => {
            let previous = doc.metadata.version;
            doc.metadata.version.minor = *sub;
            Ok(DocMetaUndo::Version(previous))
        }
    }
}

pub(crate) fn revert(undo: &DocMetaUndo, doc: &mut Document) -> CoreResult<()> {
    match undo {
        DocMetaUndo::Title(title) => {
            doc.metadata.title = title.clone();
            Ok(())
        }
        DocMetaUndo::Synopsis(synopsis) => {
            doc.metadata.synopsis = synopsis.clone();
            Ok(())
        }
        DocMetaUndo::Setting(setting) => {
            doc.metadata.setting = setting.clone();
            Ok(())
        }
        DocMetaUndo::Cover(cover) => {
            doc.metadata.cover = cover.clone();
            Ok(())
        }
        DocMetaUndo::Authorship(authorship) => {
            doc.metadata.authorship = authorship.clone();
            Ok(())
        }
        DocMetaUndo::Production(production) => {
            doc.metadata.production = production.clone();
            Ok(())
        }
        DocMetaUndo::Rights(copyright) => {
            doc.metadata.copyright = copyright.clone();
            Ok(())
        }
        DocMetaUndo::Credits(credits) => {
            doc.metadata.credits = credits.clone();
            Ok(())
        }
        DocMetaUndo::Status { status, distribution_note } => {
            doc.metadata.status = *status;
            doc.metadata.distribution_note = distribution_note.clone();
            Ok(())
        }
        DocMetaUndo::Version(version) => {
            doc.metadata.version = *version;
            Ok(())
        }
    }
}