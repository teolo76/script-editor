// src/types.rs

//! Value types puri, senza comportamento strutturale.
//!
//! Qui vivono solo identificatori ed enum di valore: `ElementId`, `Revision`,
//! `Timestamp`, `SortKey`, `NodeKind`, `BlockKind`, `Dimension` e affini.
//! Le struct "strutturali" (BaseElement, Entity, Node, Block, Annotation,
//! Layer) vivono in `core/`, insieme all'arena che le contiene: mescolarle
//! qui renderebbe questo file un contenitore indistinto di tutto.

use std::fmt;
use std::sync::atomic::{ AtomicU64, Ordering };

// Module Packages
use crate::constants::{ ALIAS_MIN_LEN, ALIAS_MAX_LEN };
use crate::errors::{ CharacterAliasError, ColorParseError};

// ---------------------------------------------------------------------
// ElementId
// ---------------------------------------------------------------------

/// Contatore globale per generare `ElementId` univoci all'interno di un
/// processo. Non è pensato per la persistenza cross-sessione: quando
/// caricheremo un documento da disco, il generatore va riallineato con
/// `ElementId::advance_past` per evitare collisioni tra id letti da file
/// e nuovi id creati in sessione.
static NEXT_ELEMENT_ID: AtomicU64 = AtomicU64::new(1);

/// Identificatore stabile di un elemento (Node, Block, Annotation, Layer, ...).
///
/// È un new-type invece di un semplice alias a `u64` apposta: così un
/// `BlockId` non può essere scambiato per errore con un `LayerId` solo
/// perché sono entrambi interi. Node, Block, Annotation e Layer condividono
/// lo stesso spazio di id (tutti generati da `ElementId::new`), il che
/// semplifica i riferimenti incrociati (es. un'annotazione che ancora a
/// "un qualsiasi elemento", senza sapere se è un nodo o un blocco).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ElementId(u64);

impl ElementId {
    /// Nuovo id univoco generato dal contatore di processo.
    pub fn new() -> Self {
        Self(NEXT_ELEMENT_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// Ricostruisce un id da un valore noto (es. caricamento da file).
    /// Il chiamante deve poi chiamare `advance_past` per evitare collisioni
    /// con i nuovi id generati in sessione.
    pub fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    pub fn raw(self) -> u64 {
        self.0
    }

    /// Da chiamare dopo il caricamento di un documento: garantisce che i
    /// prossimi `ElementId::new()` non collidano con quelli appena letti.
    pub fn advance_past(raw: u64) -> Self {
        Self(NEXT_ELEMENT_ID.fetch_max(raw +1, Ordering::Relaxed))
    }
}

impl Default for ElementId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ElementId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Id riservato per il nodo Root, noto a compile time. Evita di usare `0`
/// come sentinella "nascosta" per `parent: Option<ElementId>` — la radice
/// è un elemento vero e proprio con questo id fisso, non un caso speciale
/// di parent mancante.
pub const ROOT_ID: ElementId = ElementId(0);

// ---------------------------------------------------------------------
// Revision
// ---------------------------------------------------------------------

/// Contatore di revisione monotono del documento. Usato per invalidare la
/// cache di layout e per il command bus (un batch di comandi applicato ->
/// nuova revisione -> eventi di delta taggati con quella revisione).
///
/// Non è un timestamp: due batch nello stesso istante di orologio devono
/// comunque produrre revisioni diverse e ordinabili, e l'orologio di
/// sistema può tornare indietro senza che la revisione se ne accorga.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Revision(u64);

impl Revision {
    pub const INITIAL: Revision = Revision(0);

    #[must_use]
    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }

    pub fn value(self) -> u64 {
        self.0
    }
}

impl fmt::Display for Revision {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "r{}", self.0)
    }
}

// ---------------------------------------------------------------------
// Timestamp
// ---------------------------------------------------------------------

/// Timestamp "umano" (creazione, ultima modifica): secondi da epoch Unix.
/// Deliberatamente separato da `Revision` perché serve solo a scopi
/// informativi per l'utente (cronologia, "modificato alle...") e non deve
/// mai essere usato per invalidare cache o ordinare eventi del core.
///
/// La conversione da/verso `SystemTime` resta fuori da questo modulo, che
/// non vuole dipendenze dall'orologio di sistema o da crate esterni.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(u64);

impl Timestamp {
    pub fn from_unix_seconds(seconds: u64) -> Self {
        Self(seconds)
    }

    pub fn unix_seconds(self) -> u64 {
        self.0
    }
}

// ---------------------------------------------------------------------
// SortKey (fractional indexing)
// ---------------------------------------------------------------------

/// Chiave d'ordinamento a indicizzazione frazionaria fra fratelli nello
/// stesso parent: permette di inserire un elemento fra altri due senza
/// rinumerare i vicini (a differenza di un intero progressivo). Le chiavi
/// sono confrontabili lessicograficamente come stringhe.
///
/// Implementazione base36: ogni carattere è una cifra 0-9a-z. `between`
/// calcola una chiave compresa fra due esistenti, allungando la stringa
/// solo quando serve.
///
/// Nota: questa è una prima versione pensata per essere corretta, non
/// ottimale. Un uso intensivo di inserimenti sempre nello stesso punto può
/// far crescere la lunghezza delle chiavi nel tempo; se diventa un problema
/// pratico, aggiungeremo un ribilanciamento periodico (rinumerazione
/// batch, non nel percorso critico di editing).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SortKey(String);

impl SortKey {
    const ALPHABET: &'static [u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";

    fn digit_index(c: u8) -> u8 {
        Self::ALPHABET
            .iter()
            .position(|&d| d == c)
            .expect("Carattere non in Alphabet") as u8
    }

    fn digit_char(c: u8) -> u8 {
        Self::ALPHABET[c as usize]
    }

    /// Calcola una stringa base36 strettamente compresa fra `low` e `high`
    /// (lessicograficamente), allungando la stringa quando i due prefissi
    /// coincidono.
    fn midpoint(low: &str, highh: &str) -> Self {
        let low_digits: Vec<u8> = low.bytes().map(Self::digit_index).collect();
        let hi_digits: Vec<u8> = highh.bytes().map(Self::digit_index).collect();
        let len = low_digits.len().max(hi_digits.len()) +1;

        let get = |v: &[u8], i: usize, pad: u8| -> u8 {
            *v.get(i).unwrap_or(&pad)
        };

        let mut result = Vec::with_capacity(len);
        let mut i = 0;

        loop {
            let lo = get(&low_digits, i, 0);

            // la stringa "high" mancante si intende come "oltre l'ultima cifra":
            // usiamo la lunghezza dell'alfabeto come limite superiore aperto.
            let hi = if i < hi_digits.len() { hi_digits[i] } else { Self::ALPHABET.len() as u8 };

            if hi - lo > 1 {
                let mid = lo + (hi - lo) / 2;
                result.push(Self::digit_char(mid));
                break;
            } else {
                // Prefissi troppo vicini a questa posizione: fissa la cifra
                // di "low" e continua sulla posizione successiva.
                result.push(Self::digit_char(lo));
                i += 1;
            }
        }
        Self(String::from_utf8(result).unwrap())
    }

    /// Prima chiave utile per il primo figlio inserito sotto un parent:
    /// il carattere centrale dell'alfabeto, per lasciare spazio a inserimenti
    /// sia prima sia dopo.
    pub fn initial() -> Self {
        let mid = Self::digit_char((Self::ALPHABET.len()/2) as u8);
        Self(String::from_utf8(vec![mid]).unwrap())
    }

    /// Chiave immediatamente successiva a `self`, utile per "aggiungi in coda".
    pub fn after(&self) -> Self {
        let mut bytes = self.0.as_bytes().to_vec();
        let last = *bytes.last().unwrap();
        let last_idx = Self::digit_index(last);

        if (last_idx as usize) < Self::ALPHABET.len() -1 {
            *bytes.last_mut().unwrap() = Self::digit_char(last_idx as u8 +1);
        } else {
            bytes.push(Self::digit_char((Self::ALPHABET.len() / 2) as u8));
        }
        Self(String::from_utf8(bytes).unwrap())
    }

    /// Chiave compresa strettamente fra `low` e `high` (se forniti). Con
    /// entrambi `None` equivale a `initial()`; con solo `high` inserisce
    /// prima di tutto; con solo `low` inserisce dopo tutto (come `after`).
    pub fn between(low: Option<&SortKey>, high: Option<&SortKey>) -> Self {
        match (low, high) {
            (None, None) => Self::initial(),
            (None, Some(h)) => Self::midpoint("", &h.0),
            (Some(l), None) => l.after(),
            (Some(l), Some(h)) => {
                debug_assert!(l < h, "SortKey::between richiede low < high");
                Self::midpoint(&l.0, &h.0)
            }
        }
    }
}

impl fmt::Display for SortKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ---------------------------------------------------------------------
// NodeKind / BlockKind
// ---------------------------------------------------------------------

/// Ruolo strutturale di un `Node` nell'albero del copione.
///
/// I tipi aggiuntivi definiti a livello di progetto (es. "Prologo",
/// "Intermezzo") sono rappresentati come `Custom(String)`: non richiedono
/// di toccare questo enum né di ricompilare, ma restano comunque
/// distinguibili da `Act`/`Scene` per la logica del core che ne ha bisogno
/// (numerazione, indice, regole di impaginazione specifiche).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeKind {
    Root,
    Act,
    Scene,
    Custom(String),
}

/// Ruolo drammaturgico di un `Block` testuale — a cosa serve quel testo,
/// non di che natura tecnica è. Guida l'assegnazione di stile e layer di
/// default (es. `Parenthetical` va di default sul layer "didascalie").
///
/// Da non confondere con `BlockPayload` (definito in `core/`), che invece
/// distingue la natura tecnica del contenuto (testo vs asset): un
/// `AssetBlock` non ha un `BlockKind` drammaturgico nello stesso senso.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BlockKind {
    CharacterName,
    Dialogue,
    Parenthetical,
    StageDirection,
    Transition,
    Custom(String),
}


// ---------------------------------------------------------------------
// Dimension / Unit
// ---------------------------------------------------------------------

/// Unità di misura per una `Dimension`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    /// Percentuale rispetto a un riferimento (vedi `RelativeTo`).
    Percent,
    Px,
    Pt,
    Mm,
    Cm,
    In,
    /// Relativo alla dimensione del font corrente del blocco.
    Em,
}

/// Valore + unità di misura. Il significato concreto di una `Dimension`
/// espressa in `Percent` o `Em` dipende dal `RelativeTo` con cui viene
/// interpretata a valle, nel motore di layout: qui teniamo solo il dato,
/// non la risoluzione.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dimension {
    pub value: f64,
    pub unit: Unit,
}

impl Dimension {
    pub fn percent(value: f64) -> Self {
        Self { value, unit: Unit::Percent }
    }

    pub fn px(value: f64) -> Self {
        Self { value, unit: Unit::Px }
    }

    pub fn pt(value: f64) -> Self {
        Self { value, unit: Unit::Pt }
    }

    pub fn em(value: f64) -> Self {
        Self { value, unit: Unit::Em }
    }

    pub fn mm(value: f64) -> Self {
        Self { value, unit: Unit::Mm }
    }

    pub fn cm(value: f64) -> Self {
        Self { value, unit: Unit::Cm }
    }

    pub fn inches(value: f64) -> Self { Self { value, unit: Unit::In } }
}

/// A cosa è relativa una `Dimension` espressa in `Percent` (o, per `Em`,
/// rispetto a quale font-size). Una percentuale senza questo riferimento
/// è ambigua: "80%" della pagina o del blocco contenitore sono due cose
/// diverse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelativeTo {
    Page,
    Margin,
    Parent,
}

/// Punto dell'elemento usato come riferimento per il posizionamento
/// esplicito (es. "angolo in alto a destra dell'immagine coincide con
/// questa coordinata").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnchorPoint {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

/// Come un `AssetBlock` viene collocato nella pagina. Deliberatamente non
/// esiste una variante che porti una `BoundingBox`: quella è sempre un
/// *output* del motore di layout (vedi `LayoutResult`, in `core/`), mai un
/// dato intrinseco del blocco.
#[derive(Debug, Clone, PartialEq)]
pub enum Placement {
    /// Segue il flusso del testo; la posizione la calcola il layout, come
    /// per qualunque altro blocco.
    Flow,

    /// Posizione esplicita: `anchor` di questo asset coincide con il punto
    /// (`x`, `y`) calcolato rispetto a `relative_to`.
    Anchored {
        x: Dimension,
        y: Dimension,
        anchor: AnchorPoint,
        relative_to: RelativeTo,
    },
}

// ---------------------------------------------------------------------
// Color / ColorEncoding
// ---------------------------------------------------------------------

/// Rappresentazione canonica di un colore: RGBA già risolto, indipendente
/// da come è stato specificato in origine. È questa la forma che il resto
/// del core (Layer, in futuro il rendering) consuma — mai `ColorEncoding`
/// direttamente, così non deve sapere come si converte HSL in RGB.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

/// Colore così come può essere specificato da un file di progetto (TOML) o
/// da un'interfaccia utente: sintassi diverse per lo stesso dato. La
/// conversione nella rappresentazione canonica è responsabilità di questo
/// tipo (`resolve`), delegata qui apposta: chi legge un colore da un file
/// non deve conoscere la formula di conversione HSL → RGB, e se in futuro
/// aggiungiamo un formato (es. CMYK) tocchiamo solo questo enum.
#[derive(Debug, Clone, PartialEq)]
pub enum ColorEncoding {
    /// Es. "ff8800" o "#ff8800cc" (con alpha). Il cancelletto è opzionale.
    Hex(String),
    Rgb{ r: u8, g: u8, b: u8 },
    Rgba{ r: u8, g: u8, b: u8, a: u8 },
    /// `h` in gradi [0, 360), `s` e `l` come frazioni [0.0, 1.0].
    Hsl { h: f64, s: f64, l: f64 },
}

impl ColorEncoding {
    /// Conversione HSL → RGB standard. `h` viene normalizzato modulo 360
    /// prima del calcolo, così `h` negativo o oltre 360 non produce un
    /// risultato indefinito.
    fn hsl_to_rgb(h: f64, s: f64, l: f64) -> Color {
        let h = h.rem_euclid(360.0);
        let s = s.clamp(0.0, 1.0);
        let l = l.clamp(0.0, 1.0);

        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let h_prime = h / 60.0;
        let x = c * (1.0 - (h_prime.rem_euclid(2.0) - 1.0).abs());
        let (r1, g1, b1) = match h_prime as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        let m = l - c / 2.0;
        let to_u8 = |v: f64| -> u8 { ((v + m) * 255.0).round().clamp(0.0, 255.0) as u8 };

        Color::rgb(to_u8(r1), to_u8(g1), to_u8(b1))
    }


    fn parse_hex(input: &str) -> Result<Color, ColorParseError> {
        let hex = input.strip_prefix('#').unwrap_or(input);
        if !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(ColorParseError::InvalidHexDigits(hex.to_string()));
        }
        let byte_at = |start: usize| -> u8 {
            u8::from_str_radix(&hex[start..start + 2], 16).expect("già validato come esadecimale")
        };
        match hex.len() {
            6 => Ok(Color::rgb(byte_at(0), byte_at(2), byte_at(4))),
            8 => Ok(Color::rgba(byte_at(0), byte_at(2), byte_at(4), byte_at(6))),
            other => Err(ColorParseError::InvalidHexLength(other)),
        }
    }

    /// Converte nella rappresentazione canonica. Infallibile per le
    /// varianti già numeriche (Rgb/Rgba/Hsl); può fallire solo per `Hex`,
    /// se la stringa non è un colore esadecimale valido.
    pub fn resolve(&self) -> Result<Color, ColorParseError> {
        match self {
            ColorEncoding::Rgb { r, g, b } => Ok(Color::rgb(*r, *g, *b)),
            ColorEncoding::Rgba {r, g, b, a} => Ok(Color::rgba(*r, *g, *b, *a)),
            ColorEncoding::Hsl { h, s, l } => Ok(Self::hsl_to_rgb(*h, *s, *l)),
            ColorEncoding::Hex(s) => Self::parse_hex(s),
        }
    }
}

impl TryFrom<ColorEncoding> for Color {
    type Error = ColorParseError;

    fn try_from(value: ColorEncoding) -> Result<Self, Self::Error> {
        value.resolve()
    }
}

// ---------------------------------------------------------------------
// CharacterAlias
// ---------------------------------------------------------------------

/// Alias breve e univoco di un personaggio, usato in fase di stesura al
/// posto del nome per esteso (es. "GIU" per "Giulietta"), risolto
/// dall'editor nel personaggio corrispondente.
///
/// Normalizzato in maiuscolo alla costruzione, così "giu" e "GIU" sono lo
/// stesso alias. La validazione qui riguarda solo *forma* (lunghezza,
/// caratteri ammessi): l'*unicità* fra personaggi non è compito di questo
/// tipo, ma di `lake::CharacterStore`, perché è un vincolo sulla
/// collezione, non sul singolo valore.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CharacterAlias(String);

impl CharacterAlias {
    /// Valida e normalizza un alias grezzo. Richiede lunghezza fra
    /// `constants::ALIAS_MIN_LEN` e `constants::ALIAS_MAX_LEN` e soli
    /// caratteri alfanumerici ASCII — abbastanza corto da restare comodo
    /// da digitare, abbastanza vincolato da non creare ambiguità visive.
    pub fn new(raw: impl AsRef<str>) -> Result<Self, CharacterAliasError> {
        let raw = raw.as_ref();
        let len = raw.chars().count();

        if !(ALIAS_MIN_LEN..=ALIAS_MAX_LEN).contains(&len) {
            return Err(CharacterAliasError::InvalidLength(len));
        }

        if !raw.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(CharacterAliasError::InvalidCharacters(raw.to_string()));
        }

        Ok(Self(raw.to_ascii_uppercase()))
    }

    pub fn as_str(&self) -> &str { &self.0}
}

impl fmt::Display for CharacterAlias {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ---------------------------------------------------------------------
// Version
// ---------------------------------------------------------------------

/// Versione del documento come la vede l'autore: `major` scelta a mano,
/// `sub` un progressivo. Non ha nulla a che fare con `Revision`, che resta
/// interna e invisibile alla UI (invalidazione cache, command bus).
///
/// I due campi hanno percorsi di scrittura diversi, e la differenza è
/// deliberata: `sub` avanza in automatico (`Document::bump_version_sub`,
/// chiamato dall'app al salvataggio o su qualunque altro trigger scelga —
/// `core` non decide quando, solo come) e resta *fuori* dal command bus,
/// perché non è contenuto su cui l'utente si aspetta di poter fare Ctrl+Z:
/// disfare una modifica al testo non deve "desalvare" il documento.
/// `major` invece è una decisione deliberata dell'autore, quindi passa dal
/// command bus (`Command::SetDocVersionMajor`) con undo vero. Per
/// convenzione, cambiare `major` azzera `sub` — comportamento del comando,
/// non di questo tipo, che resta un semplice value type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
}

impl Version {
    pub fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}.{}", self.major, self.minor)
    }
}

// ---------------------------------------------------------------------
// ProjectStatus
// ---------------------------------------------------------------------

/// Stato editoriale del documento. Dominio piccolo e stabile — a
/// differenza di `NodeKind`/`BlockKind` non ha una variante `Custom`,
/// perché non è pensato per essere esteso da un progetto o da un tema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProjectStatus {
    #[default]
    Draft,
    Final,
    Locked,
}

// ---------------------------------------------------------------------
// Copyright
// ---------------------------------------------------------------------

/// Informazioni di copyright per la pagina di copertina/colophon.
/// `statement` è un override per quando la formula standard non basta;
/// quando è `None`, chi renderizza il colophon compone da sé qualcosa come
/// "© {year} {holder}. Tutti i diritti riservati." — quella formattazione
/// non è compito di questo tipo, che resta puro dato.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Copyright {
    pub holder: String,
    pub year: u32,
    pub statement: Option<String>
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_key_between_is_ordered() {
        let a = SortKey::initial();
        let b = a.after();
        let mid = SortKey::between(Some(&a), Some(&b));
        assert!(a < mid, "mid deve stare dopo a");
        assert!(mid < b, "mid deve stare prima di b");
    }

    #[test]
    fn sort_key_between_none_none_is_initial() {
        assert_eq!(SortKey::between(None, None), SortKey::initial());
    }

    #[test]
    fn sort_key_between_repeated_insert_stays_ordered() {
        // Inserimenti ripetuti sempre "appena prima" dell'ultimo: verifica
        // che l'ordine resti coerente anche se le chiavi si allungano.
        let mut keys = vec![SortKey::initial()];
        for _ in 0..20 {
            let last = keys.last().unwrap().clone();
            let new_key = SortKey::between(None, Some(&last));
            assert!(new_key < last);
            keys.push(new_key);
        }
        let mut sorted = keys.clone();
        sorted.sort();
        sorted.reverse();
        assert_eq!(keys, sorted);
    }

    #[test]
    fn element_id_advance_past_avoids_collision() {
        ElementId::advance_past(1000);
        let id = ElementId::new();
        assert!(id.raw() > 1000);
    }

    #[test]
    fn hex_with_and_without_hash_are_equivalent() {
        let a = ColorEncoding::Hex("ff8800".to_string()).resolve().unwrap();
        let b = ColorEncoding::Hex("#ff8800".to_string()).resolve().unwrap();
        assert_eq!(a, b);
        assert_eq!(a, Color::rgb(0xff, 0x88, 0x00));
    }

    #[test]
    fn hex_with_alpha_parses_rgba() {
        let color = ColorEncoding::Hex("#ff8800cc".to_string()).resolve().unwrap();
        assert_eq!(color, Color::rgba(0xff, 0x88, 0x00, 0xcc));
    }

    #[test]
    fn hex_invalid_length_is_rejected() {
        let err = ColorEncoding::Hex("fff".to_string()).resolve().unwrap_err();
        assert_eq!(err, ColorParseError::InvalidHexLength(3));
    }

    #[test]
    fn hex_invalid_digits_are_rejected() {
        let err = ColorEncoding::Hex("zzzzzz".to_string()).resolve().unwrap_err();
        assert!(matches!(err, ColorParseError::InvalidHexDigits(_)));
    }

    #[test]
    fn hsl_primary_colors_match_rgb() {
        let red = ColorEncoding::Hsl { h: 0.0, s: 1.0, l: 0.5 }.resolve().unwrap();
        assert_eq!(red, Color::rgb(255, 0, 0));

        let green = ColorEncoding::Hsl { h: 120.0, s: 1.0, l: 0.5 }.resolve().unwrap();
        assert_eq!(green, Color::rgb(0, 255, 0));

        let blue = ColorEncoding::Hsl { h: 240.0, s: 1.0, l: 0.5 }.resolve().unwrap();
        assert_eq!(blue, Color::rgb(0, 0, 255));
    }

    #[test]
    fn hsl_zero_saturation_is_gray() {
        let gray = ColorEncoding::Hsl { h: 0.0, s: 0.0, l: 0.5 }.resolve().unwrap();
        assert_eq!(gray.r, gray.g);
        assert_eq!(gray.g, gray.b);
    }

    #[test]
    fn character_alias_normalizes_to_uppercase() {
        let alias = CharacterAlias::new("giu").unwrap();
        assert_eq!(alias.as_str(), "GIU");
    }

    #[test]
    fn character_alias_rejects_out_of_range_length() {
        assert_eq!(
            CharacterAlias::new("g").unwrap_err(),
            CharacterAliasError::InvalidLength(1)
        );
        assert_eq!(
            CharacterAlias::new("giulia").unwrap_err(),
            CharacterAliasError::InvalidLength(6)
        );
    }

    #[test]
    fn character_alias_rejects_non_alphanumeric() {
        assert!(matches!(
            CharacterAlias::new("g-u").unwrap_err(),
            CharacterAliasError::InvalidCharacters(_)
        ));
    }
}