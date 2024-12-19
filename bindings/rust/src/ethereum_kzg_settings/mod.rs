use crate::{
    KzgSettings,
};
use alloc::{boxed::Box, sync::Arc};
use once_cell::race::OnceBox;

/// Returns default Ethereum mainnet KZG settings.
///
/// If you need a cloneable settings use `ethereum_kzg_settings_arc` instead.
pub fn ethereum_kzg_settings() -> &'static KzgSettings {
    ethereum_kzg_settings_inner().as_ref()
}

/// Returns default Ethereum mainnet KZG settings as an `Arc`.
///
/// It is useful for sharing the settings in multiple places.
pub fn ethereum_kzg_settings_arc() -> Arc<KzgSettings> {
    ethereum_kzg_settings_inner().clone()
}

fn ethereum_kzg_settings_inner() -> &'static Arc<KzgSettings> {
    static DEFAULT: OnceBox<([u8; 739624], Arc<KzgSettings>)> = OnceBox::new();
    &DEFAULT.get_or_init(|| {
        let mut data = *include_bytes!("./kzg_settings_raw.bin");
        let settings = KzgSettings::from_u8_slice(data.as_mut_slice());
        Box::new((data, Arc::new(settings)))
    }).1
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
