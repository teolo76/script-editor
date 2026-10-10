// src/constants.rs

//! Costanti condivise dal core. Per ora vuoto: ci mettiamo qui i default
//! di layout (margini, formato pagina) quando arriveremo a `layout`/TOML,
//! così non restano magic number sparsi nel codice.

//------------------------------------------------------------------------
// Character
//------------------------------------------------------------------------

/// Lunghezza minima di un `CharacterAlias` (in caratteri). Sotto questa
/// soglia il rischio di collisione fra personaggi dal nome simile diventa
/// concreto (es. due personaggi che finiscono entrambi per abbreviarsi
/// nello stesso carattere).
pub const ALIAS_MIN_LEN: usize = 2;

/// Lunghezza massima di un `CharacterAlias`. Oltre questa soglia l'alias
/// perde il suo scopo: tanto vale scrivere il nome del personaggio per
/// esteso durante la stesura.
pub const ALIAS_MAX_LEN: usize = 4;