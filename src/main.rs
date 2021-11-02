mod error;
use error::*;
use snafu::ResultExt;
type Result<T, E = Errors> = std::result::Result<T, E>;
fn main() -> Result<()> {
    println!("Hello, world!");
    // take a line from stdin
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    // parse the line
    let line: i32 = line.trim().parse().context(PraseStdin)?;

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
