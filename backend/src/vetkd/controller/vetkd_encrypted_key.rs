use crate::declarations::vetkd_system_api::{
    vetkd_system_api, VetkdCurve, VetkdDeriveEncryptedKeyArgs, VetkdDeriveEncryptedKeyArgsKeyId,
};
use crate::with_state;
// use ic_cdk::println;
use ic_cdk::update;
use serde_bytes::ByteBuf;

#[update]
async fn vetkd_encrypted_key(
    encryption_public_key: Vec<u8>,
    file_id: Option<u64>,
) -> Result<Vec<u8>, String> {
    // println!("vetkd_encrypted_key called with file_id: {:?}", file_id);
    // If a file_id is provided, use the file owner's principal as derivation_id
    let derivation_id = if let Some(id) = file_id {
        // println!("Looking up owner principal for file_id: {}", id);
        // Look up the file's owner principal from metadata
        let principal = with_state(|state| {
            state
                .items
                .get(&id)
                .map(|file| file.metadata.requester_principal.as_slice().to_vec())
                .ok_or_else(|| "File not found".to_string())
        })?;
        // println!("Found owner principal for file_id {}: {:?}", id, principal);
        principal
    } else {
        // Default to using the caller's principal
        let caller = ic_cdk::api::caller().as_slice().to_vec();
        // println!("Using caller principal as derivation_id: {:?}", caller);
        caller
    };

    let args = VetkdDeriveEncryptedKeyArgs {
        key_id: VetkdDeriveEncryptedKeyArgsKeyId {
            name: "insecure_test_key_1".to_string(),
            curve: VetkdCurve::Bls12381G2,
        },
        derivation_path: vec![],
        // Use requester_principal as derivation ID
        derivation_id: ByteBuf::from(derivation_id),
        encryption_public_key: ByteBuf::from(encryption_public_key),
    };

    let (result,) = vetkd_system_api
        .vetkd_derive_encrypted_key(args)
        .await
        .unwrap();

    Ok(result.encrypted_key.to_vec())
}
