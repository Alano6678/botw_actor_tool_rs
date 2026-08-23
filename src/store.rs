//! Flag store — port of the Python `store.py`.

use std::collections::{HashMap, HashSet};

use roead::byml::{Byml, Map};

use crate::flag::{Flag, FlagKind};
use crate::util::BGDATA_MAPPING;

pub const FLAG_MAPPING: &[&str] = &[
    "bool_data",
    "bool_array_data",
    "s32_data",
    "s32_array_data",
    "f32_data",
    "f32_array_data",
    "string_data",
    "string64_data",
    "string256_data",
    "string64_array_data",
    "string256_array_data",
    "vector2f_data",
    "vector2f_array_data",
    "vector3f_data",
    "vector3f_array_data",
    "vector4f_data",
];

/// Creates a default Flag for the given store type.
pub fn new_flag(ftype: &str, revival: bool) -> Flag {
    let kind = match ftype {
        "bool_data" => FlagKind::Bool {
            category: -1,
            init_value: 0,
            max_value: true,
            min_value: false,
        },
        "bool_array_data" => FlagKind::BoolArray {
            init_value: vec![0],
            max_value: true,
            min_value: false,
        },
        "s32_data" => FlagKind::S32 {
            init_value: 0,
            max_value: 2147483647,
            min_value: 0,
        },
        "s32_array_data" => FlagKind::S32Array {
            init_value: vec![0],
            max_value: 6553500,
            min_value: -1,
        },
        "f32_data" => FlagKind::F32 {
            init_value: 0.0,
            max_value: 1000000.0,
            min_value: 0.0,
        },
        "f32_array_data" => FlagKind::F32Array {
            init_value: vec![0.0],
            max_value: 360.0,
            min_value: -1.0,
        },
        "string_data" | "string64_data" | "string256_data" => FlagKind::String {
            init_value: String::new(),
            max_value: String::new(),
            min_value: String::new(),
        },
        "string64_array_data" | "string256_array_data" => FlagKind::StringArray {
            init_value: vec![String::new()],
            max_value: String::new(),
            min_value: String::new(),
        },
        "vector2f_data" => FlagKind::Vec2 {
            init_value: (0.0, 0.0),
            max_value: (255.0, 255.0),
            min_value: (0.0, 0.0),
        },
        "vector2f_array_data" => FlagKind::Vec2Array {
            init_value: vec![(0.0, 0.0)],
            max_value: (255.0, 255.0),
            min_value: (0.0, 0.0),
        },
        "vector3f_data" => FlagKind::Vec3 {
            init_value: (0.0, 0.0, 0.0),
            max_value: (100000.0, 100000.0, 100000.0),
            min_value: (-100000.0, -100000.0, -100000.0),
        },
        "vector3f_array_data" => FlagKind::Vec3Array {
            init_value: vec![(0.0, 0.0, 0.0)],
            max_value: (255.0, 255.0, 255.0),
            min_value: (0.0, 0.0, 0.0),
        },
        "vector4f_data" => FlagKind::Vec4 {
            init_value: (0.0, 0.0, 0.0, 0.0),
            max_value: (255.0, 255.0, 255.0, 255.0),
            min_value: (0.0, 0.0, 0.0, 0.0),
        },
        _ => FlagKind::Bool {
            category: -1,
            init_value: 0,
            max_value: true,
            min_value: false,
        },
    };
    Flag::new(kind, revival)
}

pub const IGNORED_SAVE_FLAGS: &[&str] = &[
    "AlbumPictureIndex",
    "IsGet_Obj_AmiiboItem",
    "CaptionPictSize",
    "SeakSensorPictureIndex",
    "AoC_HardMode_Enabled",
    "FamouseValue",
    "SaveDistrictName",
    "LastSaveTime_Lower",
    "GameClear",
    "IsChangedByDebug",
    "SaveLocationName",
    "IsSaveByAuto",
    "LastSaveTime_Upper",
    "IsLogicalDelete",
    "GyroOnOff",
    "PlayReport_CtrlMode_Ext",
    "PlayReport_CtrlMode_Free",
    "NexUniqueID_Upper",
    "MiniMapDirection",
    "CameraRLReverse",
    "JumpButtonChange",
    "TextRubyOnOff",
    "VoiceLanguage",
    "PlayReport_CtrlMode_Console_Free",
    "PlayReport_PlayTime_Handheld",
    "BalloonTextOnOff",
    "PlayReport_AudioChannel_Other",
    "PlayReport_AudioChannel_5_1ch",
    "NexIsPosTrackUploadAvailableCache",
    "NexsSaveDataUploadIntervalHoursCache",
    "NexUniqueID_Lower",
    "TrackBlockFileNumber",
    "Option_LatestAoCVerPlayed",
    "NexPosTrackUploadIntervalHoursCache",
    "NexLastUploadTrackBlockHardIndex",
    "MainScreenOnOff",
    "PlayReport_AudioChannel_Stereo",
    "NexIsSaveDataUploadAvailableCache",
    "NexLastUploadSaveDataTime",
    "PlayReport_AllPlayTime",
    "NexLastUploadTrackBlockIndex",
    "PlayReport_CtrlMode_Console_Ext",
    "AmiiboItemOnOff",
    "TrackBlockFileNumber_Hard",
    "StickSensitivity",
    "TextWindowChange",
    "IsLastPlayHardMode",
    "PlayReport_CtrlMode_Console_FullKey",
    "NexLastUploadTrackBlockTime",
    "PlayReport_CtrlMode_FullKey",
    "PlayReport_PlayTime_Console",
    "PlayReport_AudioChannel_Mono",
    "CameraUpDownReverse",
    "PlayReport_CtrlMode_Handheld",
];

#[derive(Default)]
pub struct FlagStore {
    store: HashMap<&'static str, HashMap<i32, Flag>>,
    orig_store: HashMap<&'static str, HashMap<i32, Flag>>,
}

impl FlagStore {
    pub fn new() -> Self {
        let mut store = HashMap::new();
        let mut orig_store = HashMap::new();
        for ftype in FLAG_MAPPING {
            store.insert(*ftype, HashMap::new());
            orig_store.insert(*ftype, HashMap::new());
        }
        FlagStore {
            store,
            orig_store,
        }
    }

    fn store<'a>(&'a self, ftype: &str) -> &'a HashMap<i32, Flag> {
        self.store.get(ftype).unwrap_or(&EMPTY_MAP)
    }

    fn orig<'a>(&'a self, ftype: &str) -> &'a HashMap<i32, Flag> {
        self.orig_store.get(ftype).unwrap_or(&EMPTY_MAP)
    }

    /// Load flags from a `Map` of `{ ftype: [flag hashes] }`.
    pub fn add_flags_from_hash(&mut self, name: &str, data: &Map) {
        let is_revival = name.contains("revival");
        for (ftype, value) in data.iter() {
            if let Some(arr) = value.as_array().ok() {
                let ftype = ftype.as_str();
                for flag in arr {
                    if let Some(hash) = flag.as_map().ok() {
                        let h = hash
                            .get("HashValue")
                            .and_then(|v| v.as_i32().ok())
                            .unwrap_or(0);
                        self.store
                            .get_mut(ftype)
                            .map(|m| m.insert(h, Flag::from_hash(new_flag(ftype, is_revival).kind, hash, is_revival)));
                        self.orig_store
                            .get_mut(ftype)
                            .map(|m| m.insert(h, Flag::from_hash(new_flag(ftype, is_revival).kind, hash, is_revival)));
                    }
                }
            }
        }
    }

    pub fn add_flags_from_hash_no_overwrite(&mut self, name: &str, data: &Map) {
        let is_revival = name.contains("revival");
        for (ftype, value) in data.iter() {
            if let Some(arr) = value.as_array().ok() {
                let ftype = ftype.as_str();
                for flag in arr {
                    if let Some(hash) = flag.as_map().ok() {
                        let h = hash
                            .get("HashValue")
                            .and_then(|v| v.as_i32().ok())
                            .unwrap_or(0);
                        if self.find(ftype, h).is_none() {
                            self.store
                                .get_mut(ftype)
                                .map(|m| m.insert(h, Flag::from_hash(new_flag(ftype, is_revival).kind, hash, is_revival)));
                            self.orig_store
                                .get_mut(ftype)
                                .map(|m| m.insert(h, Flag::from_hash(new_flag(ftype, is_revival).kind, hash, is_revival)));
                        }
                    }
                }
            }
        }
    }

    pub fn find(&self, ftype: &str, hash: i32) -> Option<&Flag> {
        self.store(ftype).get(&hash)
    }

    pub fn find_all(&self, ftype: &str, search: &str) -> Vec<&Flag> {
        self.store(ftype)
            .values()
            .filter(|f| f.name_contains(search))
            .collect()
    }

    pub fn find_all_hashes(&self, ftype: &str, search: &str) -> HashSet<i32> {
        self.store(ftype)
            .iter()
            .filter(|(_, f)| f.name_contains(search))
            .map(|(h, _)| *h)
            .collect()
    }

    pub fn add(&mut self, ftype: &str, flag: Flag) {
        self.store
            .get_mut(ftype)
            .map(|m| m.insert(flag.hash_value, flag));
    }

    pub fn remove(&mut self, ftype: &str, hash: i32) {
        self.store
            .get_mut(ftype)
            .map(|m| m.remove(&hash));
    }

    pub fn get_num_new(&self) -> usize {
        FLAG_MAPPING.iter().map(|ftype| self.get_new_ftype(ftype).len()).sum()
    }

    pub fn get_num_modified(&self) -> usize {
        FLAG_MAPPING.iter().map(|ftype| self.get_modified_ftype(ftype).len()).sum()
    }

    pub fn get_num_deleted(&self) -> usize {
        FLAG_MAPPING.iter().map(|ftype| self.get_deleted_ftype(ftype).len()).sum()
    }

    pub fn get_new_ftype(&self, ftype: &str) -> Vec<String> {
        self.store(ftype)
            .iter()
            .filter(|(h, _)| !self.orig(ftype).contains_key(h))
            .map(|(_, f)| f.data_name.clone())
            .collect()
    }

    pub fn get_modified_ftype(&self, ftype: &str) -> Vec<String> {
        self.store(ftype)
            .iter()
            .filter(|(h, f)| {
                self.orig(ftype)
                    .get(h)
                    .map(|o| o != *f)
                    .unwrap_or(false)
            })
            .map(|(_, f)| f.data_name.clone())
            .collect()
    }

    pub fn get_deleted_ftype(&self, ftype: &str) -> Vec<String> {
        self.orig(ftype)
            .iter()
            .filter(|(h, _)| !self.store(ftype).contains_key(h))
            .map(|(_, f)| f.data_name.clone())
            .collect()
    }

    pub fn get_total_changes(&self) -> usize {
        self.get_num_new() + self.get_num_modified() + self.get_num_deleted()
    }

    pub fn get_num_new_svdata(&self) -> usize {
        FLAG_MAPPING
            .iter()
            .map(|ftype| self.get_new_ftype_svdata(ftype).len())
            .sum()
    }

    pub fn get_num_deleted_svdata(&self) -> usize {
        FLAG_MAPPING
            .iter()
            .map(|ftype| self.get_deleted_ftype_svdata(ftype).len())
            .sum()
    }

    pub fn get_new_ftype_svdata(&self, ftype: &str) -> Vec<String> {
        self.store(ftype)
            .iter()
            .filter(|(h, f)| f.is_save && !self.orig(ftype).contains_key(h))
            .map(|(_, f)| f.data_name.clone())
            .collect()
    }

    pub fn get_deleted_ftype_svdata(&self, ftype: &str) -> Vec<String> {
        self.orig(ftype)
            .iter()
            .filter(|(h, f)| f.is_save && !self.store(ftype).contains_key(h))
            .map(|(_, f)| f.data_name.clone())
            .collect()
    }

    /// Flags for a bgdata prefix, sorted by hash value ascending.
    pub fn flags_to_bgdata_array(&self, prefix: &str) -> Vec<Byml> {
        let ftype = BGDATA_MAPPING
            .iter()
            .find(|(p, _)| *p == prefix)
            .map(|(_, t)| *t)
            .unwrap_or(prefix);
        let mut flags: Vec<&Flag> = match prefix {
            "revival_bool_data" | "revival_s32_data" => self
                .store(ftype)
                .values()
                .filter(|f| f.is_revival)
                .collect(),
            "bool_data" | "s32_data" => self
                .store(ftype)
                .values()
                .filter(|f| !f.is_revival)
                .collect(),
            _ => self.store(ftype).values().collect(),
        };
        flags.sort_by_key(|f| f.hash_value);
        flags.iter().map(|f| Byml::Map(f.to_hash())).collect()
    }

    /// Flags for saveformat files, sorted by hash value ascending.
    pub fn flags_to_svdata_array(&self) -> Vec<Byml> {
        let mut flags: Vec<&Flag> = self
            .store
            .values()
            .flat_map(|m| m.values())
            .filter(|f| f.is_save && !IGNORED_SAVE_FLAGS.contains(&f.data_name.as_str()))
            .collect();
        flags.sort_by_key(|f| f.hash_value);
        flags.iter().map(|f| Byml::Map(f.to_sv_hash())).collect()
    }
}

static EMPTY_MAP: std::sync::LazyLock<HashMap<i32, Flag>> =
    std::sync::LazyLock::new(HashMap::new);
