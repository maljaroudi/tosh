mod error;
use std::{
    env, fs,
    io::{stderr, stdin, stdout, BufRead, Write},
    path::Path,
    process::Command,
};

use error::*;
use snafu::ResultExt;
type Result<T, E = Errors> = std::result::Result<T, E>;
fn main() -> Result<()> {
    // init for sheller, use stdin
    let mut stdin = stdin();
    // lock stdin
    let mut t = stdin.lock();
    // call sheller with stdin
    sheller(&mut t)?;
    Ok(())
}

fn sheller<R: BufRead>(input: &mut R) -> Result<()> {
    //input.read_line(&mut line).unwrap();
    //let line: i32 = line.trim().parse().context(PraseStdin)?;
    // -----------------------------------------------------
    loop {
        print!("{}> ", env::current_dir().unwrap().display());
        stdout().flush();
        let mut line = String::new();
        input.read_line(&mut line).unwrap();

        let mut parts = line.trim().split_whitespace();
        let command = parts.next().unwrap_or("");
        let args = parts;
        // stdout defination
        let stdout = stdout();
        //let mut stdout = stdout.lock();
        // check tab completion
        if command == "tab" {
            let mut cmd = Command::new("bash");
            cmd.arg("-c").arg("compgen -ac");
            let output = cmd.output().unwrap();
            let mut stdout = stdout.lock();
            stdout.write_all(&output.stdout).unwrap();
            continue;
        }
        match command {
            "cd" => {
                let new_dir = args.peekable().peek().map_or("~/", |x| *x);
                let root = Path::new(new_dir);
                if let Err(e) = env::set_current_dir(&root) {
                    eprintln!("{}", e);
                }
            }
            "exit" => return Ok(()),
            "" => continue,
            "ls" => {
                // use system's ls command
                // run windows internal command "Dir" to list files // check if windows
                if cfg!(windows) {
                    let mut cmd = Command::new("powershell")
                        .arg("/c")
                        .arg("ls")
                        .output()
                        .unwrap();
                    let mut stdout = stdout.lock();
                    stdout.write_all(&cmd.stdout).unwrap();
                } else {
                    let mut cmd = Command::new("ls").arg("-l").output().unwrap();
                    let mut stdout = stdout.lock();
                    stdout.write_all(&cmd.stdout).unwrap();
                }
                // let mut stdout = stdout.lock();
                // stdout.write_all(&cmd.stdout).unwrap();
            }
            // handle mkdir
            "mkdir" => {
                let mut args = args.peekable();
                let dir = args.next().unwrap_or("");
                let dir = Path::new(dir);
                // if -p is set, create parent dir
                if args.peek() == Some(&"-p") {
                    if let Err(e) = fs::create_dir_all(dir) {
                        eprintln!("{}", e);
                    }
                } else if let Err(e) = fs::create_dir(dir) {
                    eprintln!("{}", e);
                }
            }
            command => {
                let child = Command::new(command).args(args).spawn();

                // gracefully handle malformed user input
                match child {
                    Ok(mut child) => {
                        child.wait();
                    }
                    Err(e) => eprintln!("{}", e),
                };
            } // empty line
        }
    }
    // -----------------------------------------------------
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::OpenOptions;
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
    fn test_serialize() -> Result<()> {
        let stdin = b"Test123";
        let result = sheller(&mut stdin.as_ref());
        assert!(result.is_err());
        let err = result.unwrap_err();
        let serialized = toml::to_string(&err).context(SerializeError)?;
        //print to file
        let mut file = std::fs::File::create("test.toml").context(File)?;
        file.write_all(serialized.as_bytes()).context(File)?;
        Ok(())
    }
    // test serialize to a test.toml file, but fail and serialize the error to test.toml
    #[test]
    fn test_serialize_fail() -> Result<()> {
        let stdin = b"Test123";
        let result = sheller(&mut stdin.as_ref());
        assert!(result.is_err());
        let err = result.unwrap_err();
        let serialized = toml::to_string(&err).context(SerializeError)?;
        //print to file
        let mut file_err = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open("test.toml")
            .context(File)
            .unwrap_err();
        let mut file = std::fs::File::create("test.toml").context(File)?;
        file.write_all(
            toml::to_string(&file_err)
                .context(SerializeError)?
                .as_bytes(),
        )
        .context(File)?;
        Ok(())
    }
}
