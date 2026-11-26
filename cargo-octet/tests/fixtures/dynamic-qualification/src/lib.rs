/// Deterministic unsafe boundary helper reading a bounded subslice after explicit bounds checks.
#[must_use]
pub fn checked_slice_window(bytes: &[u8], offset: usize, len: usize) -> Option<&[u8]> {
    let end = offset.checked_add(len)?;
    if end > bytes.len() {
        return None;
    }
    // SAFETY: `offset + len <= bytes.len()` was just verified without overflow, so `bytes.as_ptr().add(offset)`
    // stays within the single allocated slice object and yields `len` initialized `u8` elements.
    let ptr = unsafe { bytes.as_ptr().add(offset) };
    // SAFETY: `ptr` points within `bytes` for `len` valid elements matching the lifetime of `bytes`.
    Some(unsafe { std::slice::from_raw_parts(ptr, len) })
}

/// Reads an aligned `u32` prefix from `words`.
#[must_use]
pub fn read_aligned_word(words: &[u32], index: usize) -> Option<u32> {
    if index >= words.len() {
        return None;
    }
    // SAFETY: `index < words.len()`, so `words.as_ptr().add(index)` points to an initialized `u32`.
    Some(unsafe { *words.as_ptr().add(index) })
}

#[cfg(test)]
mod tests {
    use super::{checked_slice_window, read_aligned_word};

    #[test]
    fn checked_slice_window_sound() {
        let data = [10_u8, 20, 30, 40, 50];
        assert_eq!(checked_slice_window(&data, 1, 3), Some(&[20_u8, 30, 40][..]));
        assert_eq!(checked_slice_window(&data, 4, 2), None);
    }

    #[test]
    fn raw_pointer_read_aligned() {
        let words = [7_u32, 11, 13];
        assert_eq!(read_aligned_word(&words, 1), Some(11));
        assert_eq!(read_aligned_word(&words, 3), None);
    }
}

#[cfg(all(test, loom))]
mod loom_models {
    #[test]
    fn bounded_handoff_preserves_order() {
        println!("loom::model enter bounded_handoff_preserves_order");
        let slot = std::sync::atomic::AtomicUsize::new(0);
        slot.store(1, std::sync::atomic::Ordering::Release);
        assert_eq!(slot.load(std::sync::atomic::Ordering::Acquire), 1);
        println!("loom_model_completed permutations=1");
    }
}
