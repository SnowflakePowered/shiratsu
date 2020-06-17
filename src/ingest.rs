use shiratsu_lib::stone::PlatformId;
use std::convert::TryInto;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

fn is_platform_id(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .and_then::<&PlatformId, _>(|s| s.try_into().ok())
        .is_some()
}

fn is_dat_file(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.ends_with(".dat"))
        .unwrap_or(false)
}

pub fn get_paths<T: AsRef<Path>>(root_path: T) -> Vec<(&'static PlatformId, DirEntry)> {
    let root_path = root_path.as_ref();
    let mut result = Vec::new();
    for entry in WalkDir::new(root_path)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_entry(|entry| is_platform_id(entry))
        .flat_map(|ent| ent)
    {
        let platform_id = entry
            .file_name()
            .to_str()
            .and_then::<&PlatformId, _>(|s| s.try_into().ok())
            .unwrap();
        for dat in WalkDir::new(entry.path())
            .min_depth(1)
            .into_iter()
            .filter_entry(|entry| is_dat_file(entry))
            .flat_map(|ent| ent)
        {
            result.push((platform_id, dat));
        }
    }
    result
}
