use crate::KzgSettings;
use alloc::sync::Arc;
use once_cell::sync::Lazy;

/// A lazily initialized global instance of Ethereum mainnet KZG settings.
static ETHEREUM_KZG_SETTINGS: Lazy<Arc<KzgSettings>> = Lazy::new(load_kzg_settings_impl);

/// Returns the default Ethereum mainnet KZG settings as a reference.
///
/// If you need a cloneable settings instance, use [`ethereum_kzg_settings_arc`] instead.
pub fn ethereum_kzg_settings() -> &'static KzgSettings {
    ETHEREUM_KZG_SETTINGS.as_ref()
}

/// Returns the default Ethereum mainnet KZG settings as an `Arc`.
///
/// This is useful when sharing the settings in multiple places.
pub fn ethereum_kzg_settings_arc() -> Arc<KzgSettings> {
    ETHEREUM_KZG_SETTINGS.clone()
}

/// On little endian targets, load the KzgSettings dump directly.
#[cfg(target_endian = "little")]
fn load_kzg_settings_impl() -> Arc<KzgSettings> {
    // Ensure that the data is aligned to 8 bytes as it will be interpreted as arrays of u64.
    #[repr(align(8))]
    struct AlignedKzgSettings([u8; 739624]);
    static RAW_KZG_SETTINGS: AlignedKzgSettings =
        AlignedKzgSettings(*include_bytes!("./kzg_settings_raw_le.bin"));

    Arc::new(
        // SAFETY: The binary data is assumed to be in the correct format.
        unsafe {
            KzgSettings::deserialize(core::pin::Pin::static_ref(&RAW_KZG_SETTINGS.0))
                .expect("failed to deserialize KzgSettings")
        },
    )
}

/// On other targets compute the KzgSettings from the curve points.
#[cfg(not(target_endian = "little"))]
fn load_kzg_settings_impl() -> Arc<KzgSettings> {
    use crate::bindings::*;

    // Type aliases for the expected structure layout of the binary files.
    type G1Points = [[u8; BYTES_PER_G1_POINT]; NUM_G1_POINTS];
    type G2Points = [[u8; BYTES_PER_G2_POINT]; NUM_G2_POINTS];

    /// Default G1 points.
    const ETH_G1_POINTS: &G1Points = {
        const BYTES: &[u8] = include_bytes!("./g1_points.bin");
        assert!(BYTES.len() == core::mem::size_of::<G1Points>());
        unsafe { &*BYTES.as_ptr().cast::<G1Points>() }
    };

    /// Default G2 points.
    const ETH_G2_POINTS: &G2Points = {
        const BYTES: &[u8] = include_bytes!("./g2_points.bin");
        assert!(BYTES.len() == core::mem::size_of::<G2Points>());
        unsafe { &*BYTES.as_ptr().cast::<G2Points>() }
    };

    Arc::new(
        KzgSettings::load_trusted_setup(ETH_G1_POINTS, ETH_G2_POINTS)
            .expect("failed to load trusted setup"),
    )
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
        let eth_commitment = KzgCommitment::blob_to_kzg_commitment(&blob, eth_settings)
            .unwrap()
            .to_bytes();
        assert_eq!(ts_commitment, eth_commitment);

        // generate proof
        let ts_proof = KzgProof::compute_blob_kzg_proof(&blob, &ts_commitment, &ts_settings)
            .unwrap()
            .to_bytes();
        let eth_proof = KzgProof::compute_blob_kzg_proof(&blob, &eth_commitment, eth_settings)
            .unwrap()
            .to_bytes();
        assert_eq!(ts_proof, eth_proof);
    }
}
