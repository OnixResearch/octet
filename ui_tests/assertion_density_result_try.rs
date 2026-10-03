#![feature(const_try, const_trait_impl, const_try_residual, try_blocks)]
#![deny(assertion_density, unfulfilled_lint_expectations)]

use std::ops::ControlFlow;

fn fallible(value: usize) -> Result<usize, &'static str> {
    if value == 0 { Err("zero") } else { Ok(value + 1) }
}

const fn const_fallible(value: usize) -> Result<usize, &'static str> {
    if value == 0 { Err("zero") } else { Ok(value + 1) }
}

fn two_authored_result_checks(value: usize) -> Result<usize, &'static str> {
    let first = fallible(value)?;
    let second = fallible(first)?;
    let a = second + 1;
    let b = a + 1;
    let c = b + 1;
    let d = c + 1;
    let e = d + 1;
    let f = e + 1;
    let g = f + 1;
    let h = g + 1;
    let i = h + 1;
    let j = i + 1;
    let k = j + 1;
    let l = k + 1;
    let m = l + 1;
    let n = m + 1;
    let o = n + 1;
    let p = o + 1;
    let q = p + 1;
    let r = q + 1;
    Ok(r)
}

fn nested_authored_result_checks(value: usize) -> Result<usize, &'static str> {
    let first = fallible(fallible(value)?)?;
    let a = first + 1;
    let b = a + 1;
    let c = b + 1;
    let d = c + 1;
    let e = d + 1;
    let f = e + 1;
    let g = f + 1;
    let h = g + 1;
    let i = h + 1;
    let j = i + 1;
    let k = j + 1;
    let l = k + 1;
    let m = l + 1;
    let n = m + 1;
    let o = n + 1;
    let p = o + 1;
    let q = p + 1;
    let r = q + 1;
    let s = r + 1;
    Ok(s)
}

#[expect(assertion_density)]
fn one_result_and_infallible_ok(value: usize) -> Result<usize, &'static str> {
    let first = fallible(value)?;
    let second = Ok::<_, &'static str>(first)?;
    let third = core::result::Result::<_, &'static str>::Ok(second)?;
    let a = third + 1;
    let b = a + 1;
    let c = b + 1;
    let d = c + 1;
    let e = d + 1;
    let f = e + 1;
    let g = f + 1;
    let h = g + 1;
    let i = h + 1;
    let j = i + 1;
    let k = j + 1;
    let l = k + 1;
    let m = l + 1;
    let n = m + 1;
    let o = n + 1;
    let p = o + 1;
    let q = p + 1;
    Ok(q)
}

#[expect(assertion_density)]
fn one_result_and_nested_closure(value: usize) -> Result<usize, &'static str> {
    let first = fallible(value)?;
    let nested = || -> Result<usize, &'static str> {
        let second = fallible(first)?;
        let third = fallible(second)?;
        Ok(third)
    };
    let a = first + 1;
    let b = a + 1;
    let c = b + 1;
    let d = c + 1;
    let e = d + 1;
    let f = e + 1;
    let g = f + 1;
    let h = g + 1;
    let i = h + 1;
    let j = i + 1;
    let k = j + 1;
    let l = k + 1;
    let m = l + 1;
    let n = m + 1;
    let o = n + 1;
    let p = o + 1;
    let _ = nested;
    Ok(p)
}

#[expect(assertion_density)]
fn one_result_and_nested_const(value: usize) -> Result<usize, &'static str> {
    let first = fallible(value)?;
    let nested: Result<usize, &'static str> = const {
        let local: Result<usize, &'static str> = try {
            let second = const_fallible(1)?;
            const_fallible(second)?
        };
        local
    };
    let a = first + 1;
    let b = a + 1;
    let c = b + 1;
    let d = c + 1;
    let e = d + 1;
    let f = e + 1;
    let g = f + 1;
    let h = g + 1;
    let i = h + 1;
    let j = i + 1;
    let k = j + 1;
    let l = k + 1;
    let m = l + 1;
    let n = m + 1;
    let o = n + 1;
    let p = o + 1;
    let _nested_is_ok = nested.is_ok();
    Ok(p)
}

#[expect(assertion_density)]
fn one_result_and_local_try_block(value: usize) -> Result<usize, &'static str> {
    let first = fallible(value)?;
    let local: Result<usize, &'static str> = try {
        let second = fallible(first)?;
        fallible(second)?
    };
    let a = first + 1;
    let b = a + 1;
    let c = b + 1;
    let d = c + 1;
    let e = d + 1;
    let f = e + 1;
    let g = f + 1;
    let h = g + 1;
    let i = h + 1;
    let j = i + 1;
    let k = j + 1;
    let l = k + 1;
    let m = l + 1;
    let n = m + 1;
    let o = n + 1;
    let p = o + 1;
    let _local_is_ok = local.is_ok();
    Ok(p)
}

#[expect(assertion_density)]
fn only_option_checks(value: Option<usize>) -> Option<usize> {
    let first = value?;
    let second = Some(first)?;
    let a = second + 1;
    let b = a + 1;
    let c = b + 1;
    let d = c + 1;
    let e = d + 1;
    let f = e + 1;
    let g = f + 1;
    let h = g + 1;
    let i = h + 1;
    let j = i + 1;
    let k = j + 1;
    let l = k + 1;
    let m = l + 1;
    let n = m + 1;
    let o = n + 1;
    let p = o + 1;
    let q = p + 1;
    let r = q + 1;
    Some(r)
}

#[expect(assertion_density)]
fn only_controlflow_checks(value: usize) -> ControlFlow<&'static str, usize> {
    let first = ControlFlow::<&'static str, usize>::Continue(value)?;
    let second = ControlFlow::<&'static str, usize>::Continue(first)?;
    let a = second + 1;
    let b = a + 1;
    let c = b + 1;
    let d = c + 1;
    let e = d + 1;
    let f = e + 1;
    let g = f + 1;
    let h = g + 1;
    let i = h + 1;
    let j = i + 1;
    let k = j + 1;
    let l = k + 1;
    let m = l + 1;
    let n = m + 1;
    let o = n + 1;
    let p = o + 1;
    let q = p + 1;
    let r = q + 1;
    ControlFlow::Continue(r)
}

macro_rules! generated_try {
    ($value:expr) => {
        fallible($value)?
    };
}

#[expect(assertion_density)]
fn one_result_and_macro_generated_checks(value: usize) -> Result<usize, &'static str> {
    let first = fallible(value)?;
    let second = generated_try!(first);
    let third = generated_try!(second);
    let text = "fallible(value)? fallible(value)?";
    // fallible(value)? is not an expression here.
    let a = third + text.len();
    let b = a + 1;
    let c = b + 1;
    let d = c + 1;
    let e = d + 1;
    let f = e + 1;
    let g = f + 1;
    let h = g + 1;
    let i = h + 1;
    let j = i + 1;
    let k = j + 1;
    let l = k + 1;
    let m = l + 1;
    let n = m + 1;
    let o = n + 1;
    let p = o + 1;
    Ok(p)
}

mod shadowed {
    #[allow(non_snake_case)]
    fn Err(_: &'static str) -> Result<usize, &'static str> {
        Ok(1)
    }

    #[expect(assertion_density)]
    pub(super) fn one_real_try(value: usize) -> Result<usize, &'static str> {
        let first = super::fallible(value)?;
        if first == 1 {
            return Err("not an error");
        }
        let a = first + 1;
        let b = a + 1;
        let c = b + 1;
        let d = c + 1;
        let e = d + 1;
        let f = e + 1;
        let g = f + 1;
        let h = g + 1;
        let i = h + 1;
        let j = i + 1;
        let k = j + 1;
        let l = k + 1;
        let m = l + 1;
        let n = m + 1;
        let o = n + 1;
        let p = o + 1;
        let q = p + 1;
        Ok(q)
    }
}

enum Refusal {
    Missing,
    Denied,
}

fn two_authored_option_refusals(value: usize) -> Option<Refusal> {
    if value == 0 {
        return Some(Refusal::Missing);
    }
    if value > 10 {
        return core::option::Option::Some(Refusal::Denied);
    }
    let a = value + 1;
    let b = a + 1;
    let c = b + 1;
    let d = c + 1;
    let e = d + 1;
    let f = e + 1;
    let g = f + 1;
    let h = g + 1;
    let i = h + 1;
    let j = i + 1;
    let k = j + 1;
    let l = k + 1;
    let m = l + 1;
    let n = m + 1;
    let o = n + 1;
    let _ = o;
    None
}

mod shadowed_option {
    #[allow(non_snake_case)]
    fn Some(_value: usize) -> Option<usize> {
        None
    }

    #[expect(assertion_density)]
    pub(super) fn user_defined_some(value: usize) -> Option<usize> {
        if value == 0 {
            return Some(value);
        }
        if value > 10 {
            return Some(value);
        }
        let a = value + 1;
        let b = a + 1;
        let c = b + 1;
        let d = c + 1;
        let e = d + 1;
        let f = e + 1;
        let g = f + 1;
        let h = g + 1;
        let i = h + 1;
        let j = i + 1;
        let k = j + 1;
        let l = k + 1;
        let m = l + 1;
        let n = m + 1;
        let o = n + 1;
        let _ = o;
        None
    }
}

#[expect(assertion_density)]
fn unconditional_option_some(value: usize) -> Option<usize> {
    if value == 0 {
        return Some(value);
    }
    let a = value + 1;
    let b = a + 1;
    let c = b + 1;
    let d = c + 1;
    let e = d + 1;
    let f = e + 1;
    let g = f + 1;
    let h = g + 1;
    let i = h + 1;
    let j = i + 1;
    let k = j + 1;
    let l = k + 1;
    let m = l + 1;
    let n = m + 1;
    let o = n + 1;
    let p = o + 1;
    return Some(p);
}

#[expect(assertion_density)]
fn nested_option_closure(value: usize) -> Option<usize> {
    if value == 0 {
        return Some(value);
    }
    let nested = |inner: usize| -> Option<usize> {
        if inner == 1 {
            return Some(inner);
        }
        if inner == 2 {
            return Some(inner);
        }
        None
    };
    let a = value + 1;
    let b = a + 1;
    let c = b + 1;
    let d = c + 1;
    let e = d + 1;
    let f = e + 1;
    let g = f + 1;
    let h = g + 1;
    let i = h + 1;
    let j = i + 1;
    let k = j + 1;
    let l = k + 1;
    let m = l + 1;
    let _ = (m, nested);
    None
}

macro_rules! generated_some {
    ($value:expr) => {
        return Some($value)
    };
}

#[expect(assertion_density)]
fn generated_option_some(value: usize) -> Option<usize> {
    if value == 0 {
        return Some(value);
    }
    if value == 1 {
        generated_some!(value);
    }
    let a = value + 1;
    let b = a + 1;
    let c = b + 1;
    let d = c + 1;
    let e = d + 1;
    let f = e + 1;
    let g = f + 1;
    let h = g + 1;
    let i = h + 1;
    let j = i + 1;
    let k = j + 1;
    let l = k + 1;
    let m = l + 1;
    let n = m + 1;
    let _ = n;
    None
}

#[expect(assertion_density)]
fn local_option_try(value: Option<usize>) -> Option<usize> {
    let local: Option<usize> = try {
        let inner = value?;
        Some(inner)?
    };
    let a = local.unwrap_or_default() + 1;
    let b = a + 1;
    let c = b + 1;
    let d = c + 1;
    let e = d + 1;
    let f = e + 1;
    let g = f + 1;
    let h = g + 1;
    let i = h + 1;
    let j = i + 1;
    let k = j + 1;
    let l = k + 1;
    let m = l + 1;
    let n = m + 1;
    let o = n + 1;
    let p = o + 1;
    Some(p)
}

fn main() {}
