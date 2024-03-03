use std::{
    error::Error,
    ffi::OsStr,
    io::{Read, Write},
    path::PathBuf,
};

#[derive(Debug)]
enum Action {
    Help(String),
    Format(FormatRequest),
}

#[derive(Debug)]
struct FormatRequest {
    write_to_file: bool,
    target: FormatTarget,
}

#[derive(Debug)]
struct SomeError(String);

#[derive(Debug)]
enum FormatTarget {
    Files { paths: Vec<String> },
    Stdin,
}

impl std::fmt::Display for SomeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)?;
        Ok(())
    }
}
impl std::error::Error for SomeError {}

pub fn run(
    r: &mut impl Read,
    w: &mut impl Write,
    args: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> Result<(), Box<dyn Error>> {
    let action = parse_args(args)?;
    match action {
        Action::Help(message) => {
            write!(w, "{message}")?;
            Ok(())
        }
        Action::Format(request) => run_format(r, w, request),
    }
}

fn run_format(
    r: &mut impl Read,
    w: &mut impl Write,
    request: FormatRequest,
) -> Result<(), Box<dyn Error>> {
    match request.target {
        FormatTarget::Stdin => {
            let mut source = Vec::new();
            r.read_to_end(&mut source)?;
            let result = crate::format_source(source)?;
            write!(w, "{}", result)?;
            Ok(())
        }
        FormatTarget::Files { ref paths } => {
            let target_paths = flatten_target_paths(paths)?;
            let need_file_separator = paths.len() > 1;
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
    }
}

fn flatten_target_paths(target_paths: &Vec<String>) -> Result<Vec<PathBuf>, Box<dyn Error>> {
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

    let matches = options.parse(args)?;
    if matches.opt_present("h") || matches.free.is_empty() {
        let usage = options.usage("Usage: rbfmt [options] [path/-]...");
        return Ok(Action::Help(usage));
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
    o.optflag("h", "help", "print this help message");
    o.optflag("w", "write", "Write output to files instead of STDOUT");
    o
}

#[cfg(test)]
mod test {
    use std::error::Error;

    use similar_asserts::assert_eq;

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