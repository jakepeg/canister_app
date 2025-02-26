use crate::declarations::vetkd_system_api::{
    chainkey_testing_canister, VetkdCurve, VetkdDeriveEncryptedKeyArgs,
    VetkdDeriveEncryptedKeyArgsKeyId,
};
use ic_cdk::update;
use serde_bytes::ByteBuf;

#[update]
async fn vetkd_encrypted_key(encryption_public_key: Vec<u8>) -> Result<Vec<u8>, String> {
    let address = ic_cdk::api::caller(); // Replaced ethadress with ICP principal of the user
    let address_bytes = address.as_slice().to_vec(); // Convert Principal to Vec<u8>

    let args = VetkdDeriveEncryptedKeyArgs {
        key_id: VetkdDeriveEncryptedKeyArgsKeyId {
            name: "insecure_test_key_1".to_string(),
            curve: VetkdCurve::Bls12381G2,
        },
        derivation_path: vec![],
        derivation_id: ByteBuf::from(address_bytes),
        encryption_public_key: ByteBuf::from(encryption_public_key),
    };

    let (result,) = chainkey_testing_canister
        .vetkd_derive_encrypted_key(args)
        .await
        .unwrap();

    Ok(result.encrypted_key.to_vec())
}
