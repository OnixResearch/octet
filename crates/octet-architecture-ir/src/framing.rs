// A framed byte slice cannot exceed usize::MAX. Reject platforms where that
// length cannot be represented in the identity format, rather than truncating it.
const _: () = assert!(usize::BITS <= u64::BITS);

#[must_use]
pub fn framed_blake3(domain: &str, fields: &[&[u8]]) -> String {
    let mut hasher = blake3::Hasher::new();
    update_frame(&mut hasher, domain.as_bytes());
    for field in fields {
        update_frame(&mut hasher, field);
    }
    hasher.finalize().to_hex().to_string()
}

/// Serialize an architecture value for canonical identity input.
///
/// # Errors
///
/// Returns the serializer error when `value` cannot be encoded as JSON.
pub fn canonical_json<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, String> {
    serde_json::to_vec(value).map_err(|error| format!("serialize canonical architecture value: {error}"))
}

/// Bind the serialized value to an identity domain.
///
/// # Errors
///
/// Propagates `canonical_json`'s error if `value` cannot be serialized.
pub fn canonical_identity<T: serde::Serialize>(domain: &str, value: &T) -> Result<String, String> {
    let bytes = canonical_json(value)?;
    Ok(framed_blake3(domain, &[&bytes]))
}

fn update_frame(hasher: &mut blake3::Hasher, value: &[u8]) {
    let length_bytes = (value.len() as u64).to_le_bytes();
    hasher.update(&length_bytes);
    hasher.update(value);
}
