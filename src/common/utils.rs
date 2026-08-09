// src/common/utils.rs

use std::time::{ SystemTime, UNIX_EPOCH };

/// Restituisce il timestamp attuale espresso in millisecondi trascorsi dall'Unix Epoch.
/// In caso di errore di clock di sistema (es. tempo impostato a prima del 1970),
/// restituisce 0 come valore di fallback difensivo.
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_current_timestamp_validity() {
        let ts = current_timestamp();
        // Un timestamp Unix valido in millisecondi dopo il 2020 è un valore > 1_500_000_000_000
        assert!(ts > 1_500_000_000_000, "Il timestamp generato non è coerente con l'epoca corrente");
    }

    #[test]
    fn test_current_timestamp_monotonicity() {
        let ts1 = current_timestamp();
        sleep(Duration::from_millis(5));
        let ts2 = current_timestamp();

        assert!(ts2 >= ts1, "Il timestamp successivo deve essere maggiore o uguale a quello precedente");
    }
}