mod error;
use std::io::BufRead;

use error::*;
use snafu::ResultExt;
type Result<T, E = Errors> = std::result::Result<T, E>;
fn main() -> Result<()> {
    Ok(())
}

fn sheller<R: BufRead>(input: &mut R) -> Result<()> {
    let mut line = String::new();
    input.read_line(&mut line).unwrap();
    let line: i32 = line.trim().parse().context(PraseStdin)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;
    #[test]
    fn test_parse_error() {
        let stdin = b"Test";
        let result = sheller(&mut stdin.as_ref());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.to_string(),
            "Error with Parser: invalid digit found in string"
        );
    }
    #[test]
    // test the serialize to a test.toml file
    fn test_serialize() {
        let stdin = b"Test123";
        let result = sheller(&mut stdin.as_ref());
        assert!(result.is_err());
        let err = result.unwrap_err();
        let serialized = toml::to_string(&err).unwrap();
        //print to file
        let mut file = std::fs::File::create("test.toml").unwrap();
        file.write_all(serialized.as_bytes()).unwrap();
    }
}
