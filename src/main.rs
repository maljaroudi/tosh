use serde::{de::value, ser::SerializeMap, Deserialize, Serialize};
use snafu::{ensure, Backtrace, ErrorCompat, ResultExt, Snafu};

#[derive(Debug, Snafu)]
enum Errors {
    PraseError,
    #[snafu(display("{}", msg))]
    OtherError {
        msg: String
    },
    #[snafu(display("Error with Parser: {}", source))]
    PraseStdinError {
        source: std::num::ParseIntError,
    },
}

//impl Serialize for the Errors
impl serde::Serialize for Errors {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Errors::PraseError => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("type", "PraseError")?;
                map.end()
            }
            Errors::OtherError { msg } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "OtherError")?;
                map.serialize_entry("msg", msg)?;
                map.end()
            }
            Errors::PraseStdinError { source } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("type", "PraseStdinError")?;
                map.serialize_entry("source", &source.to_string())?;
                map.end()
            }
        }
    }
}

type Result<T, E = Errors> = std::result::Result<T, E>;
fn main() -> Result<()> {
    println!("Hello, world!");
    // take a line from stdin
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    // parse the line
    let line: i32 = line.trim().parse().context(PraseStdinError)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;
    #[test]
    fn test_parse_error() {
        let result = main();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.to_string(),
            "Error with Parser: invalid digit found in string"
        );
    }
    #[test]
    // test the serialize
    fn test_serialize() {
        let result = main();
        assert!(result.is_err());
        let err = result.unwrap_err();
        let serialized = toml::to_string(&err).unwrap();
        //print to file 
        let mut file = std::fs::File::create("test.toml").unwrap();
        file.write_all(serialized.as_bytes()).unwrap();
        
    }
}
