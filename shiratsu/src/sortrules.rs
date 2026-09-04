use anyhow::Result;
use consolespec::MachineSpec;

use std::collections::HashMap;
use std::convert::TryFrom;

pub fn load_map<S: AsRef<str>>(yaml_str: S) -> Result<HashMap<MachineSpec, Vec<String>>> {
    let source: HashMap<String, Vec<String>> = serde_yaml::from_str(yaml_str.as_ref())?;
    let mut map = HashMap::with_capacity(source.len());

    for (platform_id, mut globs) in source {
        let platform = MachineSpec::try_from(platform_id.as_str())?;
        for glob in &mut globs {
            glob.insert_str(0, "unsorted/**/");
        }
        map.insert(platform, globs);
    }

    Ok(map)
}
