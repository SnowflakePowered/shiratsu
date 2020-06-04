use shiratsu_lib::stone::PlatformId;
use serde_yaml;

use std::collections::HashMap;
use std::result::Result;


pub fn load_map<S: AsRef<str>>(yaml_str: S) -> Result<HashMap<PlatformId, Vec<String>>, serde_yaml::Error> {
    let mut map: HashMap<PlatformId, Vec<String>> = serde_yaml::from_str(yaml_str.as_ref())?;
    for glob in map.values_mut().flat_map(|f| f) {
        glob.insert_str(0, "unsorted/**/");
    }
    Ok(map)
}