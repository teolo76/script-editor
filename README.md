# Script Editor

## Script Engine Core Architecture & Migration Specification (JS to Rust)

## Executive Summary
Questo documento definisce l'architettura ad alte prestazioni per il nuovo engine di composizione e layout di copioni teatrali e sceneggiature (`script-engine`).
Il progetto sostituisce l'implementazione JS/OOP originale con un'architettura **Data-Driven (ECS-like)** in **Rust**, ottimizzata per l'integrazione desktop via **Tauri**, ricalcolo della paginazione a latenza zero e supporto dinamico a layer e temi.

---

## 1. Principi Architetturali Chiave

1. **Flat Data Lake (Zero Cache Misses):**
   Abbandono dell'albero ricorsivo ad allocazioni sparse sulla Heap. I dati risiedono in store piatti allocati in blocchi di memoria contigua, indicizzati tramite `NodeIndex` o `Uuid` leggeri.

2. **Type-Safety Gerarchica a Compile-Time:**
   Utilizzo di generics vincolati (`Node<T>` e `Leaf<T>`) per garantire che la gerarchia strutturale del copione (`Script` > `Act` / `Section` > `Scene` > `Content`) sia rigida e validata dal compilatore Rust, eliminando i controlli d'errore a runtime.

3. **Disaccoppiamento Geometrico e Stilistico:**
   Netta separazione tra la **topologia dell'albero** (`NodeStore`), i **dati di contenuto grezzi** (`LeafStore`) e le **proprietà visive/layer** (`VisualStore`).

4. **Paginazione Dinamica e "Zero-Cost" Layer Filtering:**
   Calcolo del layout lineare $O(N)$ guidato da un predicato booleano ad alta velocità che scarta i nodi appartenenti a layer disattivati per lo `Scope` corrente (UI, Stampa, Export).

---

## 2. Architettura dei Data Lake (Store Fondamentali)

Il documento principale (`ScriptDocument`) orchestra tre Data Lake piatti e quattro registri di servizio:



### A. `NodeStore` (`Node<T>`)
Gestisce la gerarchia macro. Ogni `Node<T>` possiede un `id`, un `parent_id` e un `sort_order`.
* **`Node<Script>`**: Radice unica del documento. Può contenere solo indici di `Node<Section>` o `Node<Act>`.
* **`Node<Act>`**: Atto del copione. Può contenere solo indici di `Node<Scene>` o `Node<Section>`.
* **`Node<Section>`**: Sezione procedurale o speciale (Cover, Cast Sheet, Sinossi, Note).
* **`Node<Scene>`**: Scena. Può contenere solo indici di `Leaf<T>`.

### B. `LeafStore` (`Leaf<T>`)
Contiene gli elementi terminali di contenuto.
* **`Leaf<TextElement>`**: Testo effettivo del copione (Dialogo, Battuta, Nome Personaggio, Indicazione di regia).
* **`Leaf<VisualElement>`**: Asset visuali o grafiche applicate a pagina.

### C. `VisualStore` (`GeometryStore`)
Archivia i metadati di resa visiva e spaziale indicizzati per ID:
* **`BoundingBox`**: Posizione e dimensioni relative (`x`, `y`, `width`, `height`, `scale`, `rotation`, `z_index`).
* **`Layer Binding`**: Associazione diretta del nodo al relativo `LayerId`.
* **`Style Overrides`**: Override locali per regole di stile che scavalcano il tema globale.

---

## 3. Registri di Servizio e Engine Extensibility

* **`ThemeRegistry`**: Mantiene i temi visivi (es. "Classico Teatrale", "Broadway") ed eroga le regole CSS / Layout per ciascun tipo di nodo (`act`, `scene`, `character`, `dialogue`, `stagedirection`).
* **`LayerRegistry`**: Gestisce visibilità, colori e profili I/O per i Layer (es. *Note di Regia*, *Luci*, *Costumi*, *Parentesi*) in base allo `Scope` attivo (`UI`, `Print`, `Export`).
* **`MediaRegistry`**: Archivia disaccoppiati i payload binari (Base64 / Immagini) con supporto all'interning dei dati per evitare duplicazioni in RAM.
* **`LayoutEngine`**: Engine headless di paginazione. Legge in sequenza i nodi, verifica la visibilità del layer nel `VisualStore`, applica le metriche del `ThemeRegistry` e genera un vettore piatto di strutture `Page`.

---

## 4. Struttura dei Moduli del Progetto (`script-engine`)

```text
script-engine/
├── Cargo.toml
├── ARCHITECTURE.md
|──src/
   ├── lib.rs
   ├── constants.rs             # Magic bytes, estensioni, preset embedded
   ├── types.rs                 # Timestamps, identificatori generici
   ├── errors.rs                # Error handling centralizzato (thiserror)
   │
   ├── geometry/                # GEOMETRIA 2D & BOX MODEL (Dimension, MarginBox, BoundingBox)
   │   ├── mod.rs
   │   ├── dimension.rs
   │   ├── margin_box.rs
   │   └── bounding_box.rs
   │
   ├── core/                    # DOMINIO FONDAMENTALE E DATAMODEL (ex AST)
   │   ├── mod.rs
   │   ├── node.rs              # Node<T> e Marker Types (Script, Act, Scene, Section)
   │   ├── leaf.rs              # Leaf<T>, TextElement, AssetElement e Kind (Custom)
   │   └── document.rs          # ScriptDocument (Orchestratore di Data Lake + Registri)
   │
   ├── lake/                    # DATA LAKE ARCHITECTURE (Flat Memory Stores)
   │   ├── mod.rs
   │   ├── node_store.rs        # NodeStore
   │   ├── leaf_store.rs        # LeafStore
   │   └── asset_store.rs       # AssetStore (AssetRecord + geometry)
   │
   ├── registries/              # REGISTRI DI CONFIGURAZIONE & PRESET
   │   ├── mod.rs
   │   ├── pagination.rs        # PaginationRegistry (utilizza geometry::MarginBox)
   │   ├── theme.rs             # ThemeRegistry e regole stilistiche
   │   └── layer.rs             # LayerRegistry e Scope Matrix
   │
   ├── renderers/               # ENGINE DI LAYOUT E PAGINAZIONE HEADLESS
   │   ├── mod.rs
   │   ├── font_provider.rs     # Metriche dei caratteri, altezze e calcolo righe
   │   ├── layer_filter.rs      # Predicato rapido per i layer visibili
   │   └── layout_engine.rs     # Impaginazione reattiva (Orfani, Vedove, Keep-With-Next)
   ├── io/
   │   ├── mod.rs
   │   ├── binary.rs           # DocumentIO: Header SCRPT + Payload Serde
   │   └── json.rs             # Serializzazione Serde nativa
