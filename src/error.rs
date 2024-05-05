#[derive(Debug)]
pub enum AppError {
    ParseFailed(Vec<String>),
    Misc(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::ParseFailed(messages) => {
                writeln!(f, "parse error:")?;
                for message in messages.iter() {
                    writeln!(f, "{message}")?;
                }
            }
            Self::Misc(message) => write!(f, "{message}")?,
        };
        Ok(())
    }
}
impl std::error::Error for AppError {}
