use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
type Result<T> = std::result::Result<T, Error>;

#[derive(Serialize, Deserialize, Default)]
pub struct Conf {
    alias: Option<HashMap<String, String>>,
    env: HashMap<String, Vec<String>>,
}
impl Conf {
    fn _add_alias(&mut self, alias: (String, String)) -> Result<()> {
        if let Some(hash) = &mut self.alias {
            hash.insert(alias.0, alias.1).unwrap();
        }
        Ok(())
    }
    pub fn add_env_var(&mut self, mut envvar: (String, String)) -> Result<()> {
        // switch this to if let some for all paths if they can be split by path
        if envvar.0 == "PATH" {
            let vals = std::env::var("PATH").map_err(Error::Config)?;
            let mut splitted = std::env::split_paths(&vals).collect::<Vec<_>>();
            splitted.push(PathBuf::from_str(&envvar.1).unwrap());
            let joined = std::env::join_paths(splitted.iter()).unwrap();
            envvar.1 = joined.into_string().unwrap();
        }
        std::env::set_var(&envvar.0, &envvar.1);
        self.env.entry(envvar.0).or_default().push(envvar.1);
        Ok(())
    }
    pub fn save_conf(&self) -> Result<()> {
        let fd = OpenOptions::new()
            .write(true)
            .create(true)
            .open(dirs::home_dir().unwrap().join("tosh_config.toml"))
            .map_err(Error::File)?;
        let mut f = std::io::BufWriter::new(fd);
        let tt = toml::to_string(&self).map_err(Error::Parse)?;
        writeln!(f, "{}", tt).map_err(Error::File)?;
        Ok(())
    }
    pub fn load_conf() -> Result<Self> {
        let paths = std::env::vars()
            .map(|x| {
                (
                    x.0,
                    x.1.split(':')
                        .map(|x| x.to_owned())
                        .collect::<Vec<String>>(),
                )
            })
            .collect::<HashMap<String, Vec<String>>>();
        let fd = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(dirs::home_dir().unwrap().join("tosh_config.toml"))
            .map_err(Error::File)?;
        let mut t = std::io::BufReader::new(fd);
        let mut conf_str = String::new();
        t.read_to_string(&mut conf_str).unwrap();
        let mut conf: Self = toml::from_str(&conf_str).unwrap_or_else(|_| Conf::default());
        conf.env = paths;

        Ok(conf)
    }
}
