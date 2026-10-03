#![deny(assertion_density, unfulfilled_lint_expectations)]

fn under_custom_line_threshold() -> u32 {
    let s1 = 1;
    let s2 = 2;
    let s3 = 3;
    let s4 = 4;
    let s5 = 5;
    let s6 = 6;
    let s7 = 7;
    let s8 = 8;
    let s9 = 9;
    let s10 = 10;
    let s11 = 11;
    let s12 = 12;
    let s13 = 13;
    let s14 = 14;
    let s15 = 15;
    let s16 = 16;
    let s17 = 17;
    let s18 = 18;
    let s19 = 19;
    let s20 = 20;
    let s21 = 21;
    let s22 = 22;
    let s23 = 23;
    s1
}

#[expect(assertion_density)]
fn insufficient_for_custom_assertion_count() -> u32 {
    let w1 = 1;
    let w2 = 2;
    let w3 = 3;
    let w4 = 4;
    let w5 = 5;
    let w6 = 6;
    let w7 = 7;
    let w8 = 8;
    let w9 = 9;
    let w10 = 10;
    let w11 = 11;
    let w12 = 12;
    let w13 = 13;
    let w14 = 14;
    assert!(w1 > 0);
    assert!(w2 > 0);
    let x15 = 15;
    let x16 = 16;
    let x17 = 17;
    let x18 = 18;
    let x19 = 19;
    let x20 = 20;
    let x21 = 21;
    let x22 = 22;
    let x23 = 23;
    let x24 = 24;
    let x25 = 25;
    let x26 = 26;
    let x27 = 27;
    let x28 = 28;
    let x29 = 29;
    let x30 = 30;
    x30
}

fn fallible_guards_meet_custom_count(data: &[usize]) -> Result<usize, &'static str> {
    // Three conditional error returns meet the custom minimum.
    let Some(&first) = data.first() else {
        return Err("missing first");
    };
    if data.len() < 2 {
        return Err("missing second");
    }
    let a = first;
    let b = data[1];
    if a == 0 {
        return Err("zero first value");
    }
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
    let v = u + 19;
    let w = v + 20;
    let x = w + 21;
    let y = x + 22;
    let z = y + 23;
    let aa = z + 24;
    let ab = aa + 25;
    let ac = ab + 26;
    Ok(ac)
}

#[expect(assertion_density)]
fn insufficient_fallible_checks(data: &[usize]) -> Result<usize, &'static str> {
    // Should warn: two real error guards are below the custom minimum.
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
    let v = u + 19;
    let w = v + 20;
    let x = w + 21;
    let y = x + 22;
    let z = y + 23;
    let aa = z + 24;
    let ab = aa + 25;
    let ac = ab + 26;
    Ok(ac)
}

fn main() -> Result<(), &'static str> {
    under_custom_line_threshold();
    insufficient_for_custom_assertion_count();
    fallible_guards_meet_custom_count(&[1, 2])?;
    insufficient_fallible_checks(&[1, 2])?;
    Ok(())
}
