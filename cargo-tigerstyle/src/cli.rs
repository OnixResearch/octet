use std::ffi::OsString;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Human,
    Json,
    Sarif,
}

#[derive(Debug)]
pub struct CheckArgs {
    pub scope_args: Vec<String>,
    pub cargo_check_args: Option<Vec<String>>,
    pub output_format: OutputFormat,
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
    let mut output_format = OutputFormat::Human;
    let mut index = 0;

    while index < args.len() {
        let arg = &args[index];
        if arg == "--" {
            cargo_check_args = Some(args[index + 1..].to_vec());
            break;
        }

        if arg == "--output-format" {
            let value = args.get(index + 1).ok_or_else(|| "missing value after `--output-format`".to_owned())?;
            output_format = parse_output_format(value)?;
            index += 2;
        } else if let Some(value) = arg.strip_prefix("--output-format=") {
            output_format = parse_output_format(value)?;
            index += 1;
        } else {
            match arg.as_str() {
                "-p" | "--package" | "--exclude" => {
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
    }

    Ok(CheckArgs {
        scope_args,
        cargo_check_args,
        output_format,
    })
}

fn parse_output_format(value: &str) -> Result<OutputFormat, String> {
    match value {
        "human" => Ok(OutputFormat::Human),
        "json" => Ok(OutputFormat::Json),
        "sarif" => Ok(OutputFormat::Sarif),
        other => Err(format!("unsupported output format `{other}`; expected `human`, `json`, or `sarif`")),
    }
}
