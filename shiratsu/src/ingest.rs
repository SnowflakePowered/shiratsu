use consolespec::MachineSpec;
use std::convert::TryInto;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

fn machine_spec(entry: &DirEntry) -> Option<MachineSpec> {
    entry.file_name().to_str().and_then(|id| id.try_into().ok())
}

fn is_platform_id(entry: &DirEntry) -> bool {
    machine_spec(entry).is_some()
}

fn is_dat_file(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.ends_with(".dat"))
        .unwrap_or(false)
}

pub fn get_paths<T: AsRef<Path>>(root_path: T) -> Vec<(MachineSpec, DirEntry)> {
    let root_path = root_path.as_ref();
    let mut result = Vec::new();
    for entry in WalkDir::new(root_path)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_entry(|entry| is_platform_id(entry))
        .flat_map(|ent| ent)
    {
        let platform = machine_spec(&entry).unwrap();
        for dat in WalkDir::new(entry.path())
            .min_depth(1)
            .into_iter()
            .filter_entry(|entry| is_dat_file(entry))
            .flat_map(|ent| ent)
        {
            result.push((platform, dat));
        }
    }
    result
}

pub fn redump_platform(system_code: &str) -> Option<MachineSpec> {
    Some(match system_code {
        "3DO" => MachineSpec::PANASONIC_3DO,
        "AJCD" => MachineSpec::ATARI_JAGUAR_CD,
        "CDI" => MachineSpec::PHILIPS_CDI,
        "DC" => MachineSpec::SEGA_DC,
        "FMT" => MachineSpec::FUJITSU_FMT,
        "GC" => MachineSpec::NINTENDO_GCN,
        "MCD" => MachineSpec::SEGA_CD,
        "NGCD" => MachineSpec::SNK_NGCD,
        "PC-FX" => MachineSpec::NEC_PCFX,
        "PCE" => MachineSpec::NEC_TGCD,
        "PS2" => MachineSpec::SONY_PS2,
        "PS3" => MachineSpec::SONY_PS3,
        "PS4" => MachineSpec::SONY_PS4,
        "PSP" => MachineSpec::SONY_PSP,
        "PSX" => MachineSpec::SONY_PSX,
        "SS" => MachineSpec::SEGA_SAT,
        "WII" => MachineSpec::NINTENDO_WII,
        "WIIU" => MachineSpec::NINTENDO_WIIU,
        "XBOX" => MachineSpec::MICROSOFT_XBOX,
        "XBOX360" => MachineSpec::MICROSOFT_X360,
        "XBOXONE" => MachineSpec::MICROSOFT_XBO,
        _ => return None,
    })
}
