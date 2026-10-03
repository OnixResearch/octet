#![warn(assertion_density)]

fn sparse_function(data: &[u8]) -> usize {
    // A single debug assertion is one check, not two overlapping macro names.
    let a = data.len();
    debug_assert_eq!(a, data.len());
    let b = a + 1;
    let c = b + 2;
    let d = c + 3;
    let e = d + 4;
    let f = e + 5;
    let g = f + 6;
    let h = g + 7;
    let i = h + 8;
    let j = i + 9;
    let k = j + 10;
    let l = k + 11;
    let m = l + 12;
    let n = m + 13;
    let o = n + 14;
    let p = o + 15;
    let q = p + 16;
    let r = q + 17;
    let s = r + 18;
    let t = s + 19;
    t
}

fn dense_function(data: &[u8]) -> usize {
    // Should NOT warn: has assertions
    let a = data.len();
    assert!(a > 0);
    let b = a + 1;
    let c = b + 2;
    let d = c + 3;
    let e = d + 4;
    let f = e + 5;
    let g = f + 6;
    let h = g + 7;
    let i = h + 8;
    let j = i + 9;
    let k = j + 10;
    let l = k + 11;
    let m = l + 12;
    let n = m + 13;
    let o = n + 14;
    let p = o + 15;
    let q = p + 16;
    let r = q + 17;
    let s = r + 18;
    assert!(s > 0);
    let t = s + 19;
    t
}

fn guarded_result(data: &[usize]) -> Result<usize, &'static str> {
    // Should NOT warn: each branch returns an error when an invariant fails.
    if data.is_empty() {
        return Err("missing first");
    }
    if data.len() < 2 {
        return Err("missing second");
    }
    let a = data[0];
    let b = data[1];
    let c = a + b;
    let d = c + 1;
    let e = d + 2;
    let f = e + 3;
    let g = f + 4;
    let h = g + 5;
    let i = h + 6;
    let j = i + 7;
    let k = j + 8;
    let l = k + 9;
    let m = l + 10;
    let n = m + 11;
    let o = n + 12;
    let p = o + 13;
    let q = p + 14;
    let r = q + 15;
    let s = r + 16;
    let t = s + 17;
    let u = t + 18;
    Ok(u)
}

fn guarded_option(data: &[usize]) -> Option<usize> {
    // Should NOT warn: match and if branches can both fail closed with None.
    let first = match data.first() {
        Some(first) => first,
        None => return None,
    };
    if data.len() < 2 {
        return None;
    }
    let a = *first;
    let b = data[1];
    let c = a + b;
    let d = c + 1;
    let e = d + 2;
    let f = e + 3;
    let g = f + 4;
    let h = g + 5;
    let i = h + 6;
    let j = i + 7;
    let k = j + 8;
    let l = k + 9;
    let m = l + 10;
    let n = m + 11;
    let o = n + 12;
    let p = o + 13;
    let q = p + 14;
    let r = q + 15;
    let s = r + 16;
    let t = s + 17;
    Some(t)
}

fn fallible_step(value: usize) -> Result<usize, &'static str> {
    if value == 0 { Err("zero") } else { Ok(value + 1) }
}

fn unchecked_result(data: &[usize]) -> Result<usize, &'static str> {
    // Should warn: one real propagation plus a direct Ok expression, comments,
    // strings, and a nested closure do not meet the two-check threshold.
    // assert!(true); return Err("not a guard"); let checked = data.first()?;
    let _text = "assert!(true); assert_eq!(1, 1); return Err(\"no\");";
    let _nested = || {
        assert!(!data.is_empty());
        assert_eq!(data[0], 1);
    };
    let a = fallible_step(data.len())?;
    let b = Ok::<_, &'static str>(a)?;
    let c = b + 2;
    let d = c + 3;
    let e = d + 4;
    let f = e + 5;
    let g = f + 6;
    let h = g + 7;
    let i = h + 8;
    let j = i + 9;
    let k = j + 10;
    let l = k + 11;
    let m = l + 12;
    let n = m + 13;
    let o = n + 14;
    let p = o + 15;
    let q = p + 16;
    let r = q + 17;
    let s = r + 18;
    let t = s + 19;
    Ok(t)
}

mod shadowed_result {
    // This callable has the same spelling as the standard Result variant,
    // but both of its conditional returns succeed.
    #[allow(non_snake_case)]
    fn Err(_: &'static str) -> Result<usize, &'static str> {
        Ok(1)
    }

    pub(super) fn unchecked(data: &[usize]) -> Result<usize, &'static str> {
        if data.is_empty() {
            return Err("not a failure");
        }
        if data.len() < 2 {
            return Err("still not a failure");
        }
        let a = data.len();
        let b = a + 1;
        let c = b + 2;
        let d = c + 3;
        let e = d + 4;
        let f = e + 5;
        let g = f + 6;
        let h = g + 7;
        let i = h + 8;
        let j = i + 9;
        let k = j + 10;
        let l = k + 11;
        let m = l + 12;
        let n = m + 13;
        let o = n + 14;
        let p = o + 15;
        let q = p + 16;
        Ok(q)
    }

    pub(super) fn guarded(data: &[usize]) -> Result<usize, &'static str> {
        if data.is_empty() {
            return core::result::Result::Err("real failure");
        }
        if data.len() < 2 {
            return core::result::Result::Err("another real failure");
        }
        let a = data.len();
        let b = a + 1;
        let c = b + 2;
        let d = c + 3;
        let e = d + 4;
        let f = e + 5;
        let g = f + 6;
        let h = g + 7;
        let i = h + 8;
        let j = i + 9;
        let k = j + 10;
        let l = k + 11;
        let m = l + 12;
        let n = m + 13;
        let o = n + 14;
        let p = o + 15;
        let q = p + 16;
        Ok(q)
    }
}

mod shadowed_option {
    #[allow(non_snake_case)]
    fn None() -> Option<usize> {
        Some(1)
    }

    pub(super) fn unchecked(data: &[usize]) -> Option<usize> {
        if data.is_empty() {
            return None();
        }
        if data.len() < 2 {
            return None();
        }
        let a = data.len();
        let b = a + 1;
        let c = b + 2;
        let d = c + 3;
        let e = d + 4;
        let f = e + 5;
        let g = f + 6;
        let h = g + 7;
        let i = h + 8;
        let j = i + 9;
        let k = j + 10;
        let l = k + 11;
        let m = l + 12;
        let n = m + 13;
        let o = n + 14;
        let p = o + 15;
        let q = p + 16;
        Some(q)
    }
}

mod shadowed_none_value {
    #[allow(non_upper_case_globals)]
    const None: Option<usize> = Some(1);

    pub(super) fn unchecked(data: &[usize]) -> Option<usize> {
        if data.is_empty() {
            return None;
        }
        if data.len() < 2 {
            return None;
        }
        let a = data.len();
        let b = a + 1;
        let c = b + 2;
        let d = c + 3;
        let e = d + 4;
        let f = e + 5;
        let g = f + 6;
        let h = g + 7;
        let i = h + 8;
        let j = i + 9;
        let k = j + 10;
        let l = k + 11;
        let m = l + 12;
        let n = m + 13;
        let o = n + 14;
        let p = o + 15;
        let q = p + 16;
        Some(q)
    }
}

fn short_function(x: u32) -> u32 {
    // Should NOT warn: under line threshold
    x + 1
}

fn main() -> Result<(), &'static str> {
    sparse_function(&[1]);
    dense_function(&[1]);
    guarded_result(&[1, 2])?;
    let _ = guarded_option(&[1, 2]);
    unchecked_result(&[1])?;
    shadowed_result::unchecked(&[1, 2])?;
    shadowed_result::guarded(&[1, 2])?;
    let _ = shadowed_option::unchecked(&[1, 2]);
    let _ = shadowed_none_value::unchecked(&[1, 2]);
    short_function(1);
    Ok(())
}
