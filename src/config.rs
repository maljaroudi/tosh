use toml::*;
use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use serde::{Serialize,Deserialize};
 use std::fs::OpenOptions;
use crate::error::Error;
type Result<T> = std::result::Result<T, Error>;

#[derive(Serialize,Deserialize)]
struct Conf {
    alias: HashMap<String,String>,
    env: HashMap<String,String>
}
impl Default for Conf {
    fn default() -> Self {
    Self {
    alias: HashMap::new(),
    env: HashMap::new(),
    }
  }
}
impl Conf {
fn add_alias(&mut self, alias: (String,String) ) -> Result<()> {
   Ok(())
    }
fn add_env_var(&mut self, envvar: (String,String)) {
todo!()

}
fn save_conf(&self) -> Result<()> {
 let fd = OpenOptions::new().write(true).create(true)
        .open(dirs::home_dir().unwrap().join("tosh_config.toml"))
        .map_err(Error::File)?;
    let mut f = std::io::BufWriter::new(fd);
    let tt = toml::to_string(&self).map_err(Error::Parse)?;
    writeln!(f, "{}", tt).map_err(Error::File)?;
    Ok(())
}
fn load_conf() -> Result<Self> {
let fd = OpenOptions::new().read(true).open(dirs::home_dir().unwrap()
        .join("tosh_config.toml")).map_err(Error::File)?;
let mut t = std::io::BufReader::new(fd);
let mut conf_str = String::new();
    t.read_to_string(&mut conf_str).unwrap();
    let conf: Self = toml::from_str(&conf_str).unwrap();
    Ok(conf)
  }
}


