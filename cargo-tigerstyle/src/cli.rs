use std::ffi::OsString;

#[derive(Debug)]
pub struct CheckArgs {
    pub scope_args: Vec<String>,
    pub cargo_check_args: Option<Vec<String>>,
}

pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<CheckArgs, String> {
    let mut args = args
        .into_iter()
        .map(|arg| arg.into_string().map_err(|_| "non-utf8 arguments are not supported".to_owned()))
        .collect::<Result<Vec<_>, _>>()?;

    if args.first().is_some_and(|arg| arg == "tigerstyle") {
        args.remove(0);
    }

    match args.first().map(String::as_str) {
        Some("check") => parse_check(&args[1..]),
        Some(other) => Err(format!("unknown subcommand `{other}`; expected `check`")),
        None => Err("missing subcommand; expected `check`".to_owned()),
    }
}

fn parse_check(args: &[String]) -> Result<CheckArgs, String> {
    let mut scope_args = Vec::new();
    let mut cargo_check_args = None;
    let mut index = 0;

    while index < args.len() {
        let arg = &args[index];
        if arg == "--" {
            cargo_check_args = Some(args[index + 1..].to_vec());
            break;
        }

        match arg.as_str() {
            "-p" | "--package" => {
                let value = args.get(index + 1).ok_or_else(|| format!("missing value after `{arg}`"))?;
                scope_args.push(arg.clone());
                scope_args.push(value.clone());
                index += 2;
            }
            "--workspace" => {
                scope_args.push(arg.clone());
                index += 1;
            }
            other => return Err(format!("unsupported argument before `--`: `{other}`")),
        }
    }

    Ok(CheckArgs {
        scope_args,
        cargo_check_args,
    })
}
