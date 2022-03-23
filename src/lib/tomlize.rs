use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[derive(Serialize, Deserialize)]
struct Output {
    items: HashMap<String, String>,
}

struct Parent {
    children: Vec<String>,
}
