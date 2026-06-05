//! FFI bindings for C integration

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use uuid::Uuid;

use crate::error::WalletError;
use crate::keystore::KeyType;
use crate::did::DIDMethod;

/// FFI result type
#[repr(C)]
pub struct FFIResult {
    pub success: c_int,
    pub error_message: *mut c_char,
    pub data: *mut c_void,
    pub data_len: usize,
}

/// FFI wallet handle
#[repr(C)]
pub struct FFIWallet {
    pub wallet_id: *mut c_char,
    pub did: *mut c_char,
    pub pdv_location: *mut c_char,
}

/// FFI key handle
#[repr(C)]
pub struct FFIKey {
    pub key_id: *mut c_char,
    pub key_type: c_int,
    pub public_key: *mut c_void,
    pub public_key_len: usize,
    pub did_binding: *mut c_char,
}

/// FFI signature result
#[repr(C)]
pub struct FFISignature {
    pub signature: *mut c_void,
    pub signature_len: usize,
    pub recovery_id: c_int,
    pub pqc_signature: *mut c_void,
    pub pqc_signature_len: usize,
}

unsafe fn write_string_result(result: *mut FFIResult, data: String) {
    let data_len = data.len();
    (*result).success = 1;
    (*result).error_message = ptr::null_mut();
    (*result).data = CString::new(data).unwrap().into_raw() as *mut c_void;
    (*result).data_len = data_len;
}

/// Initialize the wallet service
#[no_mangle]
pub extern "C" fn aeth_wallet_init(
    pdv_path: *const c_char,
    result: *mut FFIResult,
) -> c_int {
    if pdv_path.is_null() || result.is_null() {
        return -1;
    }

    let pdv_path_str = unsafe {
        match CStr::from_ptr(pdv_path).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        }
    };

    // Initialize wallet service (mock implementation)
    let wallet_id = Uuid::new_v4().to_string();
    let did = format!("did:key:{}", hex::encode(wallet_id.as_bytes()));
    let pdv_location = format!("{}/{}", pdv_path_str, wallet_id);

    let wallet_data = serde_json::json!({
        "wallet_id": wallet_id,
        "did": did,
        "pdv_location": pdv_location
    });

    let wallet_json = match serde_json::to_string(&wallet_data) {
        Ok(s) => s,
        Err(_) => return -1,
    };

    unsafe { write_string_result(result, wallet_json); }

    0
}

/// Create a new wallet
#[no_mangle]
pub extern "C" fn aeth_wallet_create(
    subject: *const c_char,
    intent_id: *const c_char,
    result: *mut FFIResult,
) -> c_int {
    if subject.is_null() || result.is_null() {
        return -1;
    }

    let subject_str = unsafe {
        match CStr::from_ptr(subject).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        }
    };

    let intent_id_str = if intent_id.is_null() {
        None
    } else {
        unsafe {
            match CStr::from_ptr(intent_id).to_str() {
                Ok(s) => Some(s),
                Err(_) => return -1,
            }
        }
    };

    // Mock wallet creation
    let wallet_id = Uuid::new_v4();
    let did = format!("did:key:{}", hex::encode(wallet_id.as_bytes()));
    let pdv_location = format!("/pdv/wallet/{}", wallet_id);

    let wallet_result = serde_json::json!({
        "wallet_id": wallet_id.to_string(),
        "did": did,
        "pdv_location": pdv_location,
        "created_at": chrono::Utc::now().to_rfc3339()
    });

    let wallet_json = match serde_json::to_string(&wallet_result) {
        Ok(s) => s,
        Err(_) => return -1,
    };

    unsafe { write_string_result(result, wallet_json); }

    0
}

/// Generate a new key
#[no_mangle]
pub extern "C" fn aeth_key_generate(
    subject: *const c_char,
    wallet_id: *const c_char,
    key_type: c_int,
    intent_id: *const c_char,
    result: *mut FFIResult,
) -> c_int {
    if subject.is_null() || wallet_id.is_null() || result.is_null() {
        return -1;
    }

    let subject_str = unsafe {
        match CStr::from_ptr(subject).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        }
    };

    let wallet_id_str = unsafe {
        match CStr::from_ptr(wallet_id).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        }
    };

    let key_type_enum = match key_type {
        0 => KeyType::Secp256k1,
        1 => KeyType::Ed25519,
        2 => KeyType::Sr25519,
        3 => KeyType::X25519,
        4 => KeyType::Kyber768,
        5 => KeyType::Dilithium3,
        _ => return -1,
    };

    // Mock key generation
    let key_id = Uuid::new_v4();
    let public_key = vec![0u8; 32]; // Mock public key
    let did_binding = format!("did:key:{}", hex::encode(&public_key));
    let pdv_item_id = format!("{}/keys/{}", wallet_id_str, key_id);

    let key_result = serde_json::json!({
        "key_id": key_id.to_string(),
        "key_type": format!("{:?}", key_type_enum),
        "public_key": hex::encode(&public_key),
        "did_binding": did_binding,
        "pdv_item_id": pdv_item_id
    });

    let key_json = match serde_json::to_string(&key_result) {
        Ok(s) => s,
        Err(_) => return -1,
    };

    unsafe { write_string_result(result, key_json); }

    0
}

/// Derive a key
#[no_mangle]
pub extern "C" fn aeth_key_derive(
    subject: *const c_char,
    wallet_id: *const c_char,
    parent_key_id: *const c_char,
    derivation_path: *const c_char,
    intent_id: *const c_char,
    result: *mut FFIResult,
) -> c_int {
    if subject.is_null() || wallet_id.is_null() || parent_key_id.is_null() || derivation_path.is_null() || result.is_null() {
        return -1;
    }

    let derivation_path_str = unsafe {
        match CStr::from_ptr(derivation_path).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        }
    };

    // Mock key derivation
    let key_id = Uuid::new_v4();
    let derived_key = vec![0u8; 32]; // Mock derived key
    let address = format!("0x{}", hex::encode(&derived_key[..20])); // Mock address

    let derive_result = serde_json::json!({
        "key_id": key_id.to_string(),
        "derived_key": hex::encode(&derived_key),
        "address": address,
        "derivation_path": derivation_path_str,
        "persisted": true
    });

    let derive_json = match serde_json::to_string(&derive_result) {
        Ok(s) => s,
        Err(_) => return -1,
    };

    unsafe { write_string_result(result, derive_json); }

    0
}

/// Sign data
#[no_mangle]
pub extern "C" fn aeth_sign(
    subject: *const c_char,
    wallet_id: *const c_char,
    key_id: *const c_char,
    data: *const c_void,
    data_len: usize,
    intent_id: *const c_char,
    hybrid_pqc: c_int,
    result: *mut FFIResult,
) -> c_int {
    if subject.is_null() || wallet_id.is_null() || key_id.is_null() || data.is_null() || result.is_null() {
        return -1;
    }

    let data_slice = unsafe {
        std::slice::from_raw_parts(data as *const u8, data_len)
    };

    // Mock signing
    let signature = vec![0u8; 64]; // Mock signature
    let pqc_signature = if hybrid_pqc != 0 {
        Some(vec![0u8; 64]) // Mock PQC signature
    } else {
        None
    };

    let sign_result = serde_json::json!({
        "signature": hex::encode(&signature),
        "recovery_id": 0,
        "pqc_signature": pqc_signature.map(|s| hex::encode(&s)),
        "algorithm": if hybrid_pqc != 0 { "Secp256k1+PQC" } else { "Secp256k1" }
    });

    let sign_json = match serde_json::to_string(&sign_result) {
        Ok(s) => s,
        Err(_) => return -1,
    };

    unsafe { write_string_result(result, sign_json); }

    0
}

/// Resolve a DID
#[no_mangle]
pub extern "C" fn aeth_did_resolve(
    subject: *const c_char,
    did: *const c_char,
    intent_id: *const c_char,
    result: *mut FFIResult,
) -> c_int {
    if subject.is_null() || did.is_null() || result.is_null() {
        return -1;
    }

    let did_str = unsafe {
        match CStr::from_ptr(did).to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        }
    };

    // Mock DID resolution
    let did_doc = serde_json::json!({
        "id": did_str,
        "@context": ["https://www.w3.org/ns/did/v1"],
        "verificationMethod": [{
            "id": format!("{}#key-1", did_str),
            "type": "Ed25519VerificationKey2020",
            "controller": did_str,
            "publicKeyMultibase": "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
        }],
        "authentication": [format!("{}#key-1", did_str)],
        "assertionMethod": [format!("{}#key-1", did_str)]
    });

    let did_json = match serde_json::to_string(&did_doc) {
        Ok(s) => s,
        Err(_) => return -1,
    };

    unsafe { write_string_result(result, did_json); }

    0
}

/// Export public key
#[no_mangle]
pub extern "C" fn aeth_export_pubkey(
    subject: *const c_char,
    wallet_id: *const c_char,
    key_id: *const c_char,
    intent_id: *const c_char,
    result: *mut FFIResult,
) -> c_int {
    if subject.is_null() || wallet_id.is_null() || key_id.is_null() || result.is_null() {
        return -1;
    }

    // Mock public key export
    let public_key = vec![0u8; 33]; // Mock public key

    unsafe {
        (*result).success = 1;
        (*result).error_message = ptr::null_mut();
        (*result).data = CString::new(hex::encode(&public_key)).unwrap().into_raw() as *mut c_void;
        (*result).data_len = public_key.len();
    }

    0
}

/// Free FFI result data
#[no_mangle]
pub extern "C" fn aeth_free_result(result: *mut FFIResult) {
    if result.is_null() {
        return;
    }

    unsafe {
        if !(*result).error_message.is_null() {
            let _ = CString::from_raw((*result).error_message);
        }
        if !(*result).data.is_null() {
            let _ = CString::from_raw((*result).data as *mut c_char);
        }
    }
}

/// Free FFI wallet handle
#[no_mangle]
pub extern "C" fn aeth_free_wallet(wallet: *mut FFIWallet) {
    if wallet.is_null() {
        return;
    }

    unsafe {
        if !(*wallet).wallet_id.is_null() {
            let _ = CString::from_raw((*wallet).wallet_id);
        }
        if !(*wallet).did.is_null() {
            let _ = CString::from_raw((*wallet).did);
        }
        if !(*wallet).pdv_location.is_null() {
            let _ = CString::from_raw((*wallet).pdv_location);
        }
    }
}

/// Free FFI key handle
#[no_mangle]
pub extern "C" fn aeth_free_key(key: *mut FFIKey) {
    if key.is_null() {
        return;
    }

    unsafe {
        if !(*key).key_id.is_null() {
            let _ = CString::from_raw((*key).key_id);
        }
        if !(*key).public_key.is_null() {
            let _ = CString::from_raw((*key).public_key as *mut c_char);
        }
        if !(*key).did_binding.is_null() {
            let _ = CString::from_raw((*key).did_binding);
        }
    }
}

/// Free FFI signature
#[no_mangle]
pub extern "C" fn aeth_free_signature(signature: *mut FFISignature) {
    if signature.is_null() {
        return;
    }

    unsafe {
        if !(*signature).signature.is_null() {
            let _ = CString::from_raw((*signature).signature as *mut c_char);
        }
        if !(*signature).pqc_signature.is_null() {
            let _ = CString::from_raw((*signature).pqc_signature as *mut c_char);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_result_creation() {
        let result = FFIResult {
            success: 1,
            error_message: ptr::null_mut(),
            data: ptr::null_mut(),
            data_len: 0,
        };
        assert_eq!(result.success, 1);
    }
}
