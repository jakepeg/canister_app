use crate::declarations::vetkd_system_api::{
    vetkd_system_api, VetkdCurve, VetkdDeriveEncryptedKeyArgs, VetkdDeriveEncryptedKeyArgsKeyId,
};
// use ic_cdk::println;
use ic_cdk::update;
use serde_bytes::ByteBuf;

#[update]
async fn vetkd_encrypted_key(encryption_public_key: Vec<u8>) -> Result<Vec<u8>, String> {
    let address = ic_cdk::api::caller(); // Replaced ethadress with ICP principal of the user

    // println!("Caller address: {:?}", address.to_string());

    let address_bytes = address.as_slice().to_vec(); // Convert Principal to Vec<u8>

    // println!("Principal raw bytes: {:?}", address_bytes);

    // println!("Encryption public key  {:?}", encryption_public_key);

    let args = VetkdDeriveEncryptedKeyArgs {
        key_id: VetkdDeriveEncryptedKeyArgsKeyId {
            name: "insecure_test_key_1".to_string(),
            curve: VetkdCurve::Bls12381G2,
        },
        derivation_path: vec![],
        derivation_id: ByteBuf::from(address_bytes),
        encryption_public_key: ByteBuf::from(encryption_public_key),
    };

    let (result,) = vetkd_system_api
        .vetkd_derive_encrypted_key(args)
        .await
        .unwrap();

    Ok(result.encrypted_key.to_vec())
}
