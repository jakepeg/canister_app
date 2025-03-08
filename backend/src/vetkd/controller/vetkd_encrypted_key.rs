use crate::declarations::vetkd_system_api::{
    vetkd_system_api, VetkdCurve, VetkdDeriveEncryptedKeyArgs, VetkdDeriveEncryptedKeyArgsKeyId,
};
// use ic_cdk::println;
use crate::with_state;
use ic_cdk::update;
use serde_bytes::ByteBuf;

#[update]
async fn vetkd_encrypted_key(
    file_id: u64,
    encryption_public_key: Vec<u8>,
) -> Result<Vec<u8>, String> {
    let caller = ic_cdk::api::caller(); // Replaced ethadress with ICP principal of the user

    // Get file metadata from existing state
    let (owner_principal, _) = with_state(|s| {
        let file = s.file_data.get(&file_id).ok_or("File not found")?;

        // Verify caller has access to this file
        if !s
            .file_owners
            .get(&caller)
            .map(|files| files.contains(&file_id))
            .unwrap_or(false)
        {
            return Err("Permission denied".into());
        }

        Ok((
            file.metadata.requester_principal,
            file.metadata.user_public_key.clone(),
        ))
    })?;

    let address_bytes = caller.as_slice().to_vec(); // Converts Principal to Vec<u8>

    let args = VetkdDeriveEncryptedKeyArgs {
        key_id: VetkdDeriveEncryptedKeyArgsKeyId {
            name: "insecure_test_key_1".to_string(),
            curve: VetkdCurve::Bls12381G2,
        },
        derivation_path: vec![],
        // Use requester_principal as derivation ID
        derivation_id: ByteBuf::from(owner_principal.as_slice().to_vec()),
        encryption_public_key: ByteBuf::from(encryption_public_key),
    };

    let (result,) = vetkd_system_api
        .vetkd_derive_encrypted_key(args)
        .await
        .unwrap();

    Ok(result.encrypted_key.to_vec())
}
