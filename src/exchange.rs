use std::error::Error;
use std::str::FromStr;

pub enum Exchange {
    SH(String),
    SZ(String),
}

impl AsRef<str> for Exchange {
    fn as_ref(&self) -> &str {
        match self {
            Exchange::SH(s) | Exchange::SZ(s) => s.as_ref(),
        }
    }
}

impl FromStr for Exchange {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "SH" => Ok(Exchange::SH("SH".to_string())),
            "SZ" => Ok(Exchange::SZ("SZ".to_string())),
            _ => Err("SH or SZ".into()),
        }
    }
}
