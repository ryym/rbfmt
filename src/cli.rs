use std::{
    ffi::OsStr,
    io::{Read, Write},
    os::unix::prelude::OsStrExt,
    path::PathBuf,
};

use anyhow::Context;

use crate::{config, error::AppError};

const VERSION: &str = "0.0.2";

#[derive(Debug)]
enum Action {
    Print(String),
    Format(FormatRequest),
}

#[derive(Debug)]
struct FormatRequest {
    write_to_file: bool,
    target: FormatTarget,
}

#[derive(Debug)]
enum FormatTarget {
    Files { paths: Vec<String> },
    Stdin,
}

pub fn run(
    r: &mut impl Read,
    w: &mut impl Write,
    args: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> Result<(), anyhow::Error> {
    let action = parse_args(args)?;
    match action {
        Action::Print(message) => {
            writeln!(w, "{}", message.trim())?;
            Ok(())
        }
        Action::Format(request) => run_format(r, w, request),
    }
}

fn run_format(
    r: &mut impl Read,
    w: &mut impl Write,
    request: FormatRequest,
) -> Result<(), anyhow::Error> {
    match request.target {
        FormatTarget::Stdin => {
            let mut source = Vec::new();
            r.read_to_end(&mut source)
                .context("failed to read source")?;
            let cwd = std::env::current_dir()?;
            let config = config::config_of_dir(&cwd)?;
            let result = crate::format_source(source, config.format)?;
            write!(w, "{}", result.code)?;
            if let Some(diff) = result.meaning_diff {
                std::fs::write("stdin.rbfmt-before", diff.0)?;
                std::fs::write("stdin.rbfmt-after", diff.1)?;
                eprintln!("WARNING: code meaning changes detected");
            }
            Ok(())
        }
        FormatTarget::Files { ref paths } => {
            let target_paths =
                flatten_target_paths(paths).context("failed to collect target file paths")?;
            let need_file_separator = paths.len() > 1;
            for path in target_paths {
                let source = std::fs::read(&path).with_context(|| {
                    let path = String::from_utf8_lossy(path.as_os_str().as_bytes());
                    format!("failed to read target file: {path}")
                })?;
                let config = config::config_of_path(&path)?;
                let result = crate::format_source(source, config.format);
                match result {
                    Ok(result) => {
                        if let Some(diff) = result.meaning_diff {
                            let path = path.as_os_str().to_string_lossy();
                            std::fs::write(format!("{path}.rbfmt-before"), diff.0)?;
                            std::fs::write(format!("{path}.rbfmt-after"), diff.1)?;
                            eprintln!("WARNING: code meaning changes detected: {path}");
                        }
                        if request.write_to_file {
                            std::fs::write(&path, result.code)?;
                        } else {
                            if need_file_separator {
                                writeln!(w, "\n------ {:?} -----", &path)?;
                            }
                            write!(w, "{}", result.code)?;
                        }
                    }
                    Err(err) => {
                        writeln!(w, "failed to format {:?}", &path)?;
                        writeln!(w, "error: {err}")?;
                    }
                }
            }
            Ok(())
        }
    }
}

fn flatten_target_paths(target_paths: &Vec<String>) -> Result<Vec<PathBuf>, anyhow::Error> {
    let mut paths = vec![];
    for path in target_paths {
        let path = PathBuf::from(path);
        append_paths_recursively(path, &mut paths, false)?;
    }
    Ok(paths)
}

fn append_paths_recursively(
    path: PathBuf,
    paths: &mut Vec<PathBuf>,
    ignore_non_existiing_path: bool,
) -> Result<(), anyhow::Error> {
    if !path.exists() {
        if ignore_non_existiing_path {
            return Ok(());
        } else {
            let message = format!("file not exist: {}", path.as_os_str().to_string_lossy());
            return Err(AppError::Misc(message).into());
        }
    }
    if path.is_file() {
        if let Some(ext) = path.extension() {
            if ext == "rb" {
                paths.push(path);
            }
        }
    } else if path.is_dir() {
        let entries = std::fs::read_dir(path)?;
        for entry in entries {
            let path = entry?.path();
            append_paths_recursively(path, paths, true)?;
        }
    }
    Ok(())
}

const USAGE_FOOTER: &str = "
To configure formatting, put .rbfmt.yml file.
ref: https://github.com/ryym/rbfmt
";

fn parse_args(args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Result<Action, anyhow::Error> {
    let args = args.into_iter();
    let options = build_options();

    let matches = options.parse(args)?;
    if matches.opt_present("v") {
        return Ok(Action::Print(VERSION.to_string()));
    }
    if matches.opt_present("h") || matches.free.is_empty() {
        let mut usage = options.usage("Usage: rbfmt [options] [path/-]...");
        usage.push_str(USAGE_FOOTER);
        return Ok(Action::Print(usage));
    }

    #[cfg(feature = "safety")]
    if matches.opt_present("__print-meaning") {
        let target = matches.free.iter().next().unwrap();
        let result = crate::extract_meaning(target)?;
        return Ok(Action::Print(result));
    }

    let write_to_file = matches.opt_present("w");
    let target = if matches.free.iter().any(|s| s == "-") {
        FormatTarget::Stdin
    } else {
        FormatTarget::Files {
            paths: matches.free,
        }
    };

    let fmt_request = FormatRequest {
        write_to_file,
        target,
    };
    Ok(Action::Format(fmt_request))
}

fn build_options() -> getopts::Options {
    let mut o = getopts::Options::new();
    o.optflag("h", "help", "Print this help message");
    o.optflag("w", "write", "Write output to files instead of STDOUT");
    o.optflag("v", "version", "Print version");

    #[cfg(feature = "safety")]
    o.optflag(
        "",
        "__print-meaning",
        "[experimental] print meaning of code",
    );

    o
}

#[cfg(test)]
mod test {
    use std::error::Error;

    use similar_asserts::assert_eq;

    #[test]
    fn print_version() -> Result<(), Box<dyn Error>> {
        let mut output = Vec::new();
        super::run(&mut std::io::empty(), &mut output, ["-v"])?;

        let output = String::from_utf8(output)?.to_string();
        assert_eq!(output, format!("{}\n", super::VERSION));
        Ok(())
    }

    #[test]
    fn print_help_when_no_args_provided() -> Result<(), Box<dyn Error>> {
        let mut output = Vec::new();
        super::run(&mut std::io::empty(), &mut output, [] as [&str; 0])?;

        let output = String::from_utf8(output)?.to_string();
        assert!(output.starts_with("Usage:"));
        Ok(())
    }

    #[test]
    fn read_source_from_input() -> Result<(), Box<dyn Error>> {
        let input = b"foo  . bar(1  ,2+3,  4 )";
        let mut output = Vec::new();
        super::run(&mut &input[..], &mut output, ["-"])?;

        let output = String::from_utf8(output)?.to_string();
        assert_eq!(&output, "foo.bar(1, 2 + 3, 4)\n");
        Ok(())
    }
}
