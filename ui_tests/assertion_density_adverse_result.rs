#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![deny(assertion_density, unfulfilled_lint_expectations)]

fn fallible(value: usize) -> Result<usize, &'static str> {
    if value == 0 { Err("zero") } else { Ok(value + 1) }
}

fn observe(value: Result<usize, &'static str>) -> Result<usize, &'static str> {
    value
}

fn guarded_nested_refusal(value: usize) -> Result<Result<usize, &'static str>, &'static str> {
    let checked = fallible(value)?;
    if checked > 8 {
        return Ok(Err("refused"));
    }
    let a = checked + 1;
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
    Ok(Ok(o))
}

#[expect(assertion_density)]
fn unconditional_nested_refusal(value: usize) -> Result<Result<usize, &'static str>, &'static str> {
    let checked = fallible(value)?;
    let a = checked + 1;
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
    return Ok(Err("unconditional"));
}

#[expect(assertion_density)]
fn guarded_nested_success(value: usize) -> Result<Result<usize, &'static str>, &'static str> {
    let checked = fallible(value)?;
    if checked > 8 {
        return Ok(Ok(checked));
    }
    let a = checked + 1;
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
    Ok(Ok(o))
}

macro_rules! generated_nested_refusal {
    () => {
        return Ok(Err("generated"));
    };
}

#[expect(assertion_density)]
fn macro_nested_refusal(value: usize) -> Result<Result<usize, &'static str>, &'static str> {
    let checked = fallible(value)?;
    if checked > 8 {
        generated_nested_refusal!();
    }
    let a = checked + 1;
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
    Ok(Ok(o))
}

#[expect(assertion_density)]
fn nested_closure_refusal(value: usize) -> Result<Result<usize, &'static str>, &'static str> {
    let checked = fallible(value)?;
    let deferred = || -> Result<Result<usize, &'static str>, &'static str> {
        if checked > 8 {
            return Ok(Err("nested"));
        }
        Ok(Ok(checked))
    };
    let a = checked + 1;
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
    let _ = deferred;
    Ok(Ok(o))
}

mod shadowed_outer {
    #[allow(non_snake_case)]
    fn Ok(value: Result<usize, &'static str>) -> Result<Result<usize, &'static str>, &'static str> {
        core::result::Result::Ok(value)
    }

    #[expect(assertion_density)]
    pub(super) fn guarded_but_shadowed(value: usize) -> Result<Result<usize, &'static str>, &'static str> {
        let checked = super::fallible(value)?;
        if checked > 8 {
            return Ok(Err("shadowed"));
        }
        let a = checked + 1;
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
        Ok(core::result::Result::Ok(o))
    }
}

fn consumed_adverse_match(
    value: usize,
    sampled: Result<(), &'static str>,
    prior: Result<usize, &'static str>,
) -> Result<usize, &'static str> {
    let checked = fallible(value)?;
    let a = checked + 1;
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
    let observed = match sampled {
        Ok(()) => prior,
        Err(reason) => Err(reason),
    };
    observe(observed)
}

#[expect(assertion_density)]
fn discarded_adverse_match(
    value: usize,
    sampled: Result<(), &'static str>,
    prior: Result<usize, &'static str>,
) -> Result<usize, &'static str> {
    let checked = fallible(value)?;
    let a = checked + 1;
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
    let _ignored = match sampled {
        Ok(()) => prior,
        Err(reason) => Err(reason),
    };
    Ok(o)
}

#[expect(assertion_density)]
fn successful_fallback_for_error(
    value: usize,
    sampled: Result<(), &'static str>,
    prior: Result<usize, &'static str>,
) -> Result<usize, &'static str> {
    let checked = fallible(value)?;
    let a = checked + 1;
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
    let observed = match sampled {
        Ok(()) => prior,
        Err(_) => Ok(o),
    };
    observe(observed)
}

#[expect(tigerstyle::catch_all_on_enum, assertion_density)]
fn wildcard_error_branch(
    value: usize,
    sampled: Result<(), &'static str>,
    prior: Result<usize, &'static str>,
) -> Result<usize, &'static str> {
    let checked = fallible(value)?;
    let a = checked + 1;
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
    let observed = match sampled {
        Ok(()) => prior,
        _ => Err("hidden failure"),
    };
    observe(observed)
}

#[expect(assertion_density)]
fn non_result_scrutinee(
    value: usize,
    sampled: Option<()>,
    prior: Result<usize, &'static str>,
) -> Result<usize, &'static str> {
    let checked = fallible(value)?;
    let a = checked + 1;
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
    let observed = match sampled {
        Some(()) => prior,
        None => Err("no sample"),
    };
    observe(observed)
}

#[expect(assertion_density)]
fn shadowed_match_binding(
    value: usize,
    sampled: Result<(), &'static str>,
    prior: Result<usize, &'static str>,
) -> Result<usize, &'static str> {
    let checked = fallible(value)?;
    let a = checked + 1;
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
    let observed = match sampled {
        Ok(()) => prior,
        Err(reason) => Err(reason),
    };
    let observed = Ok(observed.unwrap_or_default());
    observe(observed)
}

mod shadowed_error {
    #[allow(non_snake_case)]
    fn Err(reason: &'static str) -> Result<usize, &'static str> {
        core::result::Result::Err(reason)
    }

    #[expect(assertion_density)]
    pub(super) fn shadowed_error_constructor(
        value: usize,
        sampled: Result<(), &'static str>,
        prior: Result<usize, &'static str>,
    ) -> Result<usize, &'static str> {
        let checked = super::fallible(value)?;
        let a = checked + 1;
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
        let observed = match sampled {
            Ok(()) => prior,
            core::result::Result::Err(reason) => Err(reason),
        };
        super::observe(observed)
    }
}

fn main() {}
