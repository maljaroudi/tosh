use std::fmt::Debug;

use serde::ser::SerializeMap;
use toml::{value::Map, Value};
#[derive(Debug)]
pub enum Error {
    Cd(std::io::Error),
    Inout(std::io::Error),
    Term(std::io::Error),
    Signal(std::io::Error),
    Parse(toml::ser::Error),
    Cmd(std::io::Error),
}

impl serde::ser::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            _e @ Error::Cd(inner) => {
                let mut map = serializer.serialize_map(Some(1))?;
                //create a map
                let mut mapper = Map::new();
                // insert type into mapper
                mapper.insert(
                    "type".to_string(),
                    Value::String("cd Error (chdir)".to_string()),
                );
                // add the source
                mapper.insert("source".to_string(), toml::Value::String(inner.to_string()));
                map.serialize_entry("error", &mapper)?;
                map.end()
            }
            _ => unimplemented!(),
        }
    }
}
