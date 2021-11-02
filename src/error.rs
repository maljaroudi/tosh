use serde::{de::value, ser::SerializeMap, Deserialize, Serialize};
use snafu::{ensure, Backtrace, ErrorCompat, ResultExt, Snafu};
use toml::value::Map;
use toml::Value;
#[derive(Debug, Snafu)]
pub(crate) enum Errors {
    Prase,
    #[snafu(display("{}", msg))]
    Other {
        msg: String,
    },
    #[snafu(display("Error with Parser: {}", source))]
    #[snafu(visibility = "pub")]
    PraseStdin {
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
            Errors::Prase => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("type", "Prase")?;
                map.end()
            }
            Errors::Other { msg } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "Other")?;
                map.serialize_entry("msg", msg)?;
                map.end()
            }
            Errors::PraseStdin { source } => {
                let mut map = serializer.serialize_map(Some(1))?;
                //create a map
                let mut mapper = Map::new();
                //insert type into mapper
                mapper.insert("type".to_string(), Value::String("PraseStdin".to_string()));
                //add the source
                mapper.insert(
                    "source".to_string(),
                    toml::Value::String(source.to_string()),
                );

                // map.serialize_entry("type", "PraseStdin")?;
                // map.serialize_entry("source", &source.to_string())?;
                //serialize the mapper
                map.serialize_entry("error", &mapper)?;
                map.end()
            }
        }
    }
}
