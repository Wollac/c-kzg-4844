use crate::KzgSettings;
use alloc::sync::Arc;
use once_cell::sync::Lazy;

static RAW_KZG_SETTINGS: &[u8] = include_bytes!("./kzg_settings_raw.bin");

static ETHEREUM_KZG_SETTINGS: Lazy<Arc<KzgSettings>> =
    Lazy::new(|| Arc::new(KzgSettings::from_u8_slice(RAW_KZG_SETTINGS)));

/// Returns the default Ethereum mainnet KZG settings.
///
/// If you need a cloneable settings use `ethereum_kzg_settings_arc` instead.
pub fn ethereum_kzg_settings() -> &'static KzgSettings {
    ETHEREUM_KZG_SETTINGS.as_ref()
}

/// Returns default Ethereum mainnet KZG settings as an `Arc`.
///
/// It is useful for sharing the settings in multiple places.
pub fn ethereum_kzg_settings_arc() -> Arc<KzgSettings> {
    ETHEREUM_KZG_SETTINGS.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bindings::BYTES_PER_BLOB, Blob, KzgCommitment, KzgProof, KzgSettings};
    use std::path::Path;

    #[test]
    pub fn compare_default_with_file() {
        let ts_settings =
            KzgSettings::load_trusted_setup_file(Path::new("src/trusted_setup.txt")).unwrap();
        let eth_settings = ethereum_kzg_settings();
        let blob = Blob::new([1u8; BYTES_PER_BLOB]);

        // generate commitment
        let ts_commitment = KzgCommitment::blob_to_kzg_commitment(&blob, &ts_settings)
            .unwrap()
            .to_bytes();
        let eth_commitment = KzgCommitment::blob_to_kzg_commitment(&blob, &eth_settings)
            .unwrap()
            .to_bytes();
        assert_eq!(ts_commitment, eth_commitment);

        // generate proof
        let ts_proof = KzgProof::compute_blob_kzg_proof(&blob, &ts_commitment, &ts_settings)
            .unwrap()
            .to_bytes();
        let eth_proof = KzgProof::compute_blob_kzg_proof(&blob, &eth_commitment, &eth_settings)
            .unwrap()
            .to_bytes();
        assert_eq!(ts_proof, eth_proof);
    }
}
