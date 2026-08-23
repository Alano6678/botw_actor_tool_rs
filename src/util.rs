//! Shared constants and file-level helpers, port of the Python `util.py`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use roead::byml::Byml;
use roead::sarc::{Sarc, SarcWriter};
use roead::Endian;

use crate::settings::Settings;
use crate::store::FlagStore;

pub const RESIDENT_ACTORS: &[&str] = &[
    "GameROMPlayer",
    "Dm_Npc_Gerudo_HeroSoul_Kago",
    "Dm_Npc_Goron_HeroSoul_Kago",
    "Dm_Npc_Rito_HeroSoul_Kago",
    "Dm_Npc_Zora_HeroSoul_Kago",
    "Dm_Npc_RevivalFairy",
    "PlayerStole2",
    "WakeBoardRope",
    "Armor_Default_Extra_00",
    "Armor_Default_Extra_01",
    "Item_Conductor",
    "Animal_Insect_X",
    "Animal_Insect_A",
    "Animal_Insect_B",
    "Animal_Insect_M",
    "Animal_Insect_S",
    "Explode",
    "NormalArrow",
    "FireArrow",
    "IceArrow",
    "ElectricArrow",
    "BombArrow_A",
    "AncientArrow",
    "BrightArrow",
    "BrightArrowTP",
    "RemoteBomb",
    "RemoteBomb2",
    "RemoteBombCube",
    "RemoteBombCube2",
    "Item_Magnetglove",
    "Obj_IceMakerBlock",
    "CarryBox",
    "PlayerShockWave",
    "FireRodLv1Fire",
    "FireRodLv2Fire",
    "FireRodLv2FireChild",
    "ThunderRodLv1Thunder",
    "ThunderRodLv2Thunder",
    "ThunderRodLv2ThunderChild",
    "IceRodLv1Ice",
    "IceRodLv2Ice",
    "Animal_Insect_H",
    "Animal_Insect_F",
    "Item_Material_07",
    "Item_Material_03",
    "Item_Material_01",
    "Item_Ore_F",
];

pub const LINKS: &[&str] = &[
    "ActorNameJpn",
    "AIProgramUser",
    "AIScheduleUser",
    "ASUser",
    "AttentionUser",
    "AwarenessUser",
    "BoneControlUser",
    "ActorCaptureUser",
    "ChemicalUser",
    "DamageParamUser",
    "DropTableUser",
    "ElinkUser",
    "GParamUser",
    "LifeConditionUser",
    "LODUser",
    "ModelUser",
    "PhysicsUser",
    "ProfileUser",
    "RgBlendWeightUser",
    "RgConfigListUser",
    "RecipeUser",
    "ShopDataUser",
    "SlinkUser",
    "UMiiUser",
    "XlinkUser",
    "AnimationInfo",
];

pub const AAMP_LINK_REFS: &[(&str, (&str, &str))] = &[
    ("AIProgramUser", ("AIProgram", ".baiprog")),
    ("ASUser", ("ASList", ".baslist")),
    ("AttentionUser", ("AttClientList", ".batcllist")),
    ("AwarenessUser", ("Awareness", ".bawareness")),
    ("BoneControlUser", ("BoneControl", ".bbonectrl")),
    ("ChemicalUser", ("Chemical", ".bchemical")),
    ("DamageParamUser", ("DamageParam", ".bdmgparam")),
    ("DropTableUser", ("DropTable", ".bdrop")),
    ("GParamUser", ("GeneralParamList", ".bgparamlist")),
    ("LifeConditionUser", ("LifeCondition", ".blifecondition")),
    ("LODUser", ("LOD", ".blod")),
    ("ModelUser", ("ModelList", ".bmodellist")),
    ("PhysicsUser", ("Physics", ".bphysics")),
    ("RgBlendWeightUser", ("RagdollBlendWeight", ".brgbw")),
    ("RgConfigListUser", ("RagdollConfigList", ".brgconfiglist")),
    ("RecipeUser", ("Recipe", ".brecipe")),
    ("ShopDataUser", ("ShopData", ".bshop")),
    ("UMiiUser", ("UMii", ".bumii")),
];

pub const BYML_LINK_REFS: &[(&str, (&str, &str))] = &[
    ("AIScheduleUser", ("AISchedule", ".baischedule")),
    ("AnimationInfo", ("AnimationInfo", ".baniminfo")),
];

pub const LANGUAGES: &[&str] = &[
    "USen",
    "EUen",
    "USfr",
    "USes",
    "EUde",
    "EUes",
    "EUfr",
    "EUit",
    "EUnl",
    "EUru",
    "CNzh",
    "JPja",
    "KRko",
    "TWzh",
];

/// bgdata prefix -> byml data type key.
pub const BGDATA_MAPPING: &[(&str, &str)] = &[
    ("bool_array_data", "bool_array_data"),
    ("bool_data", "bool_data"),
    ("f32_array_data", "f32_array_data"),
    ("f32_data", "f32_data"),
    ("revival_bool_data", "bool_data"),
    ("revival_s32_data", "s32_data"),
    ("s32_array_data", "s32_array_data"),
    ("s32_data", "s32_data"),
    ("string256_array_data", "string256_array_data"),
    ("string256_data", "string256_data"),
    ("string32_data", "string_data"),
    ("string64_array_data", "string64_array_data"),
    ("string64_data", "string64_data"),
    ("vector2f_array_data", "vector2f_array_data"),
    ("vector2f_data", "vector2f_data"),
    ("vector3f_array_data", "vector3f_array_data"),
    ("vector3f_data", "vector3f_data"),
    ("vector4f_data", "vector4f_data"),
];

/// Location of a game file: either on disk or inside a resident pack.
#[derive(Clone, Debug)]
pub enum FoundFile {
    Path(PathBuf),
    Resident { titlebg: PathBuf, inner: String },
}

impl FoundFile {
    pub fn read_bytes(&self) -> anyhow::Result<Vec<u8>> {
        match self {
            FoundFile::Path(p) => Ok(std::fs::read(p)?),
            FoundFile::Resident { titlebg, inner } => {
                let sarc = Sarc::new(std::fs::read(titlebg)?)?;
                let data = sarc
                    .get_data(inner)
                    .ok_or_else(|| anyhow::anyhow!("{inner} not found in {titlebg:?}"))?;
                Ok(data.to_vec())
            }
        }
    }
}

pub fn find_file(rel_path: &str) -> anyhow::Result<FoundFile> {
    let settings = Settings::load();
    let rel = Path::new(rel_path);
    let stem = rel.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    if RESIDENT_ACTORS.contains(&stem.as_str()) {
        let titlebg = PathBuf::from(settings.update()).join("Pack").join("TitleBG.pack");
        return Ok(FoundFile::Resident {
            titlebg,
            inner: rel_path.to_string(),
        });
    }
    let base = PathBuf::from(settings.update());
    if !base.as_os_str().is_empty() && base.join(rel).exists() {
        return Ok(FoundFile::Path(base.join(rel)));
    }
    let base = PathBuf::from(settings.dlc());
    if !base.as_os_str().is_empty() && base.join(rel).exists() {
        return Ok(FoundFile::Path(base.join(rel)));
    }
    let base = PathBuf::from(settings.game());
    if !base.as_os_str().is_empty() && base.join(rel).exists() {
        return Ok(FoundFile::Path(base.join(rel)));
    }
    Err(anyhow::anyhow!("{rel_path} doesn't seem to exist."))
}

/// Decompress Yaz0 data if it is compressed, otherwise return as-is.
pub fn unyaz_if_needed(data: &[u8]) -> Vec<u8> {
    if data.len() >= 4 && &data[0..4] == b"Yaz0" {
        roead::yaz0::decompress(data).unwrap_or_default()
    } else {
        data.to_vec()
    }
}

pub fn get_gamedata_sarc(bootup_path: &Path) -> anyhow::Result<Sarc<'static>> {
    let bootup = Sarc::new(std::fs::read(bootup_path)?)?;
    let data = bootup
        .try_get_data("GameData/gamedata.ssarc")?
        .ok_or_else(|| anyhow::anyhow!("GameData/gamedata.ssarc not found"))?;
    // Tolerate both compressed (canonical) and plain SARC payloads.
    let decompressed = unyaz_if_needed(data);
    Ok(Sarc::new(decompressed)?)
}

/// Reconstruct the last two `saveformat_*.bgsvdata` files.
pub fn get_last_two_savedata_files(bootup_path: &Path) -> anyhow::Result<Vec<Vec<u8>>> {
    let bootup = Sarc::new(std::fs::read(bootup_path)?)?;
    let data = bootup
        .try_get_data("GameData/savedataformat.ssarc")?
        .ok_or_else(|| anyhow::anyhow!("GameData/savedataformat.ssarc not found"))?;
    let decompressed = unyaz_if_needed(data);
    let sarc = Sarc::new(decompressed)?;
    let mut indexes: Vec<i32> = sarc
        .files()
        .filter_map(|f| {
            let name = f.name()?;
            let rest = name.strip_prefix("/saveformat_")?;
            let rest = rest.strip_suffix(".bgsvdata")?;
            rest.parse::<i32>().ok()
        })
        .collect();
    indexes.sort_unstable();
    let last = *indexes.last().ok_or_else(|| anyhow::anyhow!("no saveformat files"))?;
    let mut out = Vec::new();
    for idx in [last - 1, last] {
        let name = format!("/saveformat_{idx}.bgsvdata");
        let data = sarc
            .try_get_data(&name)?
            .ok_or_else(|| anyhow::anyhow!("{name} not found"))?;
        out.push(data.to_vec());
    }
    Ok(out)
}

pub fn make_new_gamedata(store: &FlagStore, big_endian: bool) -> anyhow::Result<Vec<u8>> {
    let endian = if big_endian { Endian::Big } else { Endian::Little };
    let mut bg = SarcWriter::new(endian);
    for (prefix, data_type) in BGDATA_MAPPING {
        let flags = store.flags_to_bgdata_array(prefix);
        let num_files = flags.len().div_ceil(4096);
        for idx in 0..num_files.max(1) {
            let start = idx * 4096;
            let end = ((idx + 1) * 4096).min(flags.len());
            let chunk = flags[start..end].to_vec();
            let mut root: HashMap<String, Byml> = HashMap::new();
            root.insert(data_type.to_string(), Byml::Array(chunk));
            let byml = Byml::from_iter(root.into_iter());
            bg.add_file(format!("/{prefix}_{idx}.bgdata"), byml.to_binary(endian));        }
    }
    Ok(bg.to_binary())
}

pub fn make_new_savedata(
    store: &FlagStore,
    big_endian: bool,
    orig_files: Vec<Vec<u8>>,
) -> anyhow::Result<Vec<u8>> {
    let endian = if big_endian { Endian::Big } else { Endian::Little };
    let mut sv = SarcWriter::new(endian);
    let svdata = store.flags_to_svdata_array();
    let num_files = svdata.len().div_ceil(8192);
    for idx in 0..num_files.max(1) {
        let start = idx * 8192;
        let end = ((idx + 1) * 8192).min(svdata.len());
        let chunk = svdata[start..end].to_vec();
        let mut root: HashMap<String, Byml> = HashMap::new();
        root.insert(
            "file_list".to_string(),
            Byml::Array(vec![
                Byml::from_iter([
                    ("IsCommon", Byml::Bool(false)),
                    ("IsCommonAtSameAccount", Byml::Bool(false)),
                    ("IsSaveSecureCode", Byml::Bool(true)),
                    ("file_name", Byml::String("game_data.sav".into())),
                ]),
                Byml::Array(chunk),
            ]),
        );
        root.insert(
            "save_info".to_string(),
            Byml::Array(vec![Byml::from_iter([
                ("directory_num", Byml::I32(num_files as i32 + 2)),
                ("is_build_machine", Byml::Bool(true)),
                ("revision", Byml::I32(18203)),
            ])]),
        );
        sv.add_file(
            format!("/saveformat_{idx}.bgsvdata"),
            Byml::from_iter(root.into_iter()).to_binary(endian),
        );
    }
    if let (Some(a), Some(b)) = (orig_files.first(), orig_files.get(1)) {
        sv.add_file(format!("/saveformat_{num_files}.bgsvdata"), a.clone());
        sv.add_file(format!("/saveformat_{}.bgsvdata", num_files + 1), b.clone());
    }
    Ok(sv.to_binary())
}

pub fn inject_files_into_bootup(
    bootup_path: &Path,
    files: &[(&str, Vec<u8>)],
) -> anyhow::Result<()> {
    let raw = std::fs::read(bootup_path)?;
    let yaz = raw.len() >= 4 && &raw[0..4] == b"Yaz0";
    let data = if yaz { roead::yaz0::decompress(&raw)? } else { raw };
    let old_sarc = Sarc::new(data)?;
    let mut new_sarc = SarcWriter::from_sarc(&old_sarc);
    for (name, data) in files {
        new_sarc.add_file((*name).to_string(), data.clone());
    }
    let new_bytes = new_sarc.to_binary();
    let out = if yaz { roead::yaz0::compress(&new_bytes) } else { new_bytes };
    std::fs::write(bootup_path, out)?;
    Ok(())
}

pub fn inject_bytes_into_sarc(sarc_path: &Path, name: &str, data: &[u8]) -> anyhow::Result<()> {
    let raw = std::fs::read(sarc_path)?;
    let yaz = raw.len() >= 4 && &raw[0..4] == b"Yaz0";
    let bytes = if yaz { roead::yaz0::decompress(&raw)? } else { raw };
    let old_sarc = Sarc::new(bytes)?;
    let mut new_sarc = SarcWriter::from_sarc(&old_sarc);
    new_sarc.add_file(name.to_string(), data.to_vec());
    let new_bytes = new_sarc.to_binary();
    let out = if yaz { roead::yaz0::compress(&new_bytes) } else { new_bytes };
    std::fs::write(sarc_path, out)?;
    Ok(())
}

/// List all actor names in a mod `Actor/Pack` directory, sorted.
pub fn list_mod_actors(root_dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root_dir.join("Actor").join("Pack")) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "sbactorpack").unwrap_or(false)
                && let Some(stem) = path.file_stem()
            {
                let name = stem.to_string_lossy().to_string();
                if !name.ends_with("_Far") {
                    out.push(name);
                }
            }
        }
    }
    out.sort();
    out
}
