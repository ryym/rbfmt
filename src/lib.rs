use std::error::Error;

use config::FormatConfig;

mod cli;
mod config;
mod fmt;
mod meaning;
mod parse;

#[cfg(test)]
mod test;

pub fn run() -> Result<(), Box<dyn Error>> {
    cli::run(
        &mut std::io::stdin(),
        &mut std::io::stdout(),
        std::env::args().skip(1),
    )
}

pub struct FormatResult {
    code: String,
    meaning_diff: Option<(String, String)>,
}

pub fn format_source(
    source: Vec<u8>,
    config: FormatConfig,
) -> Result<FormatResult, parse::ParseError> {
    let prism_result = prism::parse(&source);

    if cfg!(feature = "safety") {
        let meaning_before = meaning::extract(&prism_result.node());
        let code = parse_and_format(config, prism_result)?;
        let meaning_after = meaning::extract(&prism::parse(code.as_bytes()).node());
        let meaning_diff = if meaning_before == meaning_after {
            None
        } else {
            Some((meaning_before, meaning_after))
        };
        Ok(FormatResult { code, meaning_diff })
    } else {
        let code = parse_and_format(config, prism_result)?;
        Ok(FormatResult {
            code,
            meaning_diff: None,
        })
    }
}

fn parse_and_format(
    config: FormatConfig,
    prism_result: prism::ParseResult,
) -> Result<String, parse::ParseError> {
    let result = parse::parse_from_prism_result(prism_result)?;
    let formatted = fmt::format(config, result.node, result.heredoc_map);
    Ok(formatted)
}

pub fn extract_meaning(target_path: &String) -> Result<String, Box<dyn Error>> {
    let source = std::fs::read_to_string(target_path)?;
    let prism_result = prism::parse(source.as_bytes());
    let meaning = meaning::extract(&prism_result.node());
    Ok(meaning)
}
