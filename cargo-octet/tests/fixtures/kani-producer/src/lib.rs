/// Deterministic bounded-window offset validator returning `(start, end)` when `offset + len <= total`.
#[must_use]
pub const fn checked_window_bounds(total: usize, offset: usize, len: usize) -> Option<(usize, usize)> {
    let Some(end) = offset.checked_add(len) else {
        return None;
    };
    if end > total {
        return None;
    }
    Some((offset, end))
}

/// Deterministic clamped byte slice checksum over at most `WINDOW_CAP` bytes.
pub const WINDOW_CAP: usize = 8;

#[must_use]
pub fn bounded_window_checksum(bytes: &[u8], offset: usize, len: usize) -> Option<u16> {
    if len > WINDOW_CAP {
        return None;
    }
    let (start, end) = checked_window_bounds(bytes.len(), offset, len)?;
    let mut acc = 0_u16;
    for byte in &bytes[start..end] {
        acc = acc.wrapping_add(u16::from(*byte));
    }
    Some(acc)
}

#[cfg(kani)]
mod proofs {
    use super::{WINDOW_CAP, bounded_window_checksum, checked_window_bounds};

    #[kani::proof]
    #[kani::unwind(10)]
    fn check_checked_window_bounds() {
        let len_arr: usize = kani::any();
        kani::assume(len_arr <= WINDOW_CAP);
        let buf = [3_u8; WINDOW_CAP];
        let slice = &buf[..len_arr];
        let offset: usize = kani::any();
        let len: usize = kani::any();
        let result = bounded_window_checksum(slice, offset, len);
        let reachable_valid = checked_window_bounds(slice.len(), offset, len).is_some() && len <= WINDOW_CAP;
        kani::cover!(reachable_valid, "cover_valid_window_reachable");
        if reachable_valid {
            assert!(result.is_some());
        }
    }

    #[kani::proof]
    fn check_failing_assertion_negative_control() {
        let x: u8 = kani::any();
        kani::assume(x < 10);
        assert!(x < 5);
    }

    #[kani::proof]
    fn check_contradictory_assumption_negative_control() {
        let x: u8 = kani::any();
        kani::assume(x > 10);
        kani::assume(x < 5);
        kani::cover!(x == 7, "cover_contradictory_unreachable");
    }
}

#[cfg(test)]
mod tests {
    use super::{bounded_window_checksum, checked_window_bounds};

    #[test]
    fn checked_window_bounds_behaves_deterministically() {
        assert_eq!(checked_window_bounds(5, 1, 3), Some((1, 4)));
        assert_eq!(checked_window_bounds(5, usize::MAX, 1), None);
        assert_eq!(bounded_window_checksum(&[1, 2, 3, 4], 1, 2), Some(5));
    }
}
