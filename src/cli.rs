use std::{error::Error, ffi::OsStr, io::Write, path::PathBuf};

#[derive(Debug)]
enum Action {
    Help(String),
    Format(FormatRequest),
}

#[derive(Debug)]
struct FormatRequest {
    write_to_file: bool,
    target_paths: Vec<String>,
}

#[derive(Debug)]
struct SomeError(String);

impl std::fmt::Display for SomeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)?;
        Ok(())
    }
}
impl std::error::Error for SomeError {}

pub fn run(
    w: &mut impl Write,
    args: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> Result<(), Box<dyn Error>> {
    let action = parse_args(args)?;
    match action {
        Action::Help(message) => {
            write!(w, "{message}")?;
            Ok(())
        }
        Action::Format(request) => run_format(w, request),
    }
}

fn run_format(w: &mut impl Write, request: FormatRequest) -> Result<(), Box<dyn Error>> {
    let target_paths = flatten_target_paths(request.target_paths)?;
    let need_file_separator = target_paths.len() > 1;
    for path in target_paths {
        let source = std::fs::read(&path)?;
        let result = crate::format_source(source)?;
        if request.write_to_file {
            std::fs::write(&path, result)?;
        } else {
            if need_file_separator {
                writeln!(w, "\n------ {:?} -----", &path)?;
            }
            write!(w, "{}", result)?;
        }
    }
    Ok(())
}

fn flatten_target_paths(target_paths: Vec<String>) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut paths = vec![];
    for path in target_paths {
        let path = PathBuf::from(path);
        append_paths_recursively(path, &mut paths)?;
    }
    Ok(paths)
}

fn append_paths_recursively(path: PathBuf, paths: &mut Vec<PathBuf>) -> Result<(), Box<dyn Error>> {
    if !path.exists() {
        let message = format!("file not exist: {}", path.as_os_str().to_string_lossy());
        return Err(Box::new(SomeError(message)));
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
            append_paths_recursively(path, paths)?;
        }
    }
    Ok(())
}

fn parse_args(args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Result<Action, Box<dyn Error>> {
    let args = args.into_iter();
    let options = build_options();

    let matches = options.parse(args.skip(1))?;
    if matches.opt_present("h") || matches.free.is_empty() {
        let usage = options.usage("Usage: rbfmt [options] [path]...");
        return Ok(Action::Help(usage));
    }

    let fmt_request = FormatRequest {
        write_to_file: matches.opt_present("w"),
        target_paths: matches.free,
    };
    Ok(Action::Format(fmt_request))
}

fn build_options() -> getopts::Options {
    let mut o = getopts::Options::new();
    o.optflag("h", "help", "print this help message");
    o.optflag("w", "write", "Write output to files instead of STDOUT");
    o
}

#[cfg(test)]
mod test {
    use std::error::Error;

    #[test]
    fn print_help_when_no_args_provided() -> Result<(), Box<dyn Error>> {
        let mut output = Vec::new();
        super::run(&mut output, [""])?;

        let output = String::from_utf8(output)?.to_string();
        assert!(output.starts_with("Usage:"));
        Ok(())
    }
}
