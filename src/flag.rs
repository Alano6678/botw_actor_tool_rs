//! GameData flag model, a faithful port of the Python `flag.py`.
//!
//! Each flag is a `Flag` with a base `BFUFlag` payload plus a type-specific
//! `FlagKind`. Hash values are CRC32 of the UTF-8 data name, wrapped to i32
//! (matching `zlib.crc32` + `ctypes.c_int32`).

use std::sync::LazyLock;

use regex::Regex;
use roead::byml::{Byml, Map};
use serde_json::Value;

use crate::data::OVERRIDES;

/// CRC32 (IEEE, zlib-compatible) of a string, wrapped to signed i32.
pub fn crc32_str(s: &str) -> i32 {
    crc32fast::hash(s.as_bytes()) as i32
}

#[derive(Clone, Debug, PartialEq)]
pub struct Flag {
    pub data_name: String,
    pub hash_value: i32,
    pub delete_rev: i32,
    pub is_event_associated: bool,
    pub is_one_trigger: bool,
    pub is_program_readable: bool,
    pub is_program_writable: bool,
    pub is_save: bool,
    pub reset_type: i32,
    pub is_revival: bool,
    pub kind: FlagKind,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FlagKind {
    Bool {
        category: i32,
        init_value: i32,
        max_value: bool,
        min_value: bool,
    },
    BoolArray {
        init_value: Vec<i32>,
        max_value: bool,
        min_value: bool,
    },
    S32 {
        init_value: i32,
        max_value: i32,
        min_value: i32,
    },
    S32Array {
        init_value: Vec<i32>,
        max_value: i32,
        min_value: i32,
    },
    F32 {
        init_value: f32,
        max_value: f32,
        min_value: f32,
    },
    F32Array {
        init_value: Vec<f32>,
        max_value: f32,
        min_value: f32,
    },
    String {
        init_value: String,
        max_value: String,
        min_value: String,
    },
    StringArray {
        init_value: Vec<String>,
        max_value: String,
        min_value: String,
    },
    Vec2 {
        init_value: (f32, f32),
        max_value: (f32, f32),
        min_value: (f32, f32),
    },
    Vec2Array {
        init_value: Vec<(f32, f32)>,
        max_value: (f32, f32),
        min_value: (f32, f32),
    },
    Vec3 {
        init_value: (f32, f32, f32),
        max_value: (f32, f32, f32),
        min_value: (f32, f32, f32),
    },
    Vec3Array {
        init_value: Vec<(f32, f32, f32)>,
        max_value: (f32, f32, f32),
        min_value: (f32, f32, f32),
    },
    Vec4 {
        init_value: (f32, f32, f32, f32),
        max_value: (f32, f32, f32, f32),
        min_value: (f32, f32, f32, f32),
    },
}

// ---------------------------------------------------------------------------
// BYML <-> Flag helpers
// ---------------------------------------------------------------------------

fn get_i32(m: &Map, key: &str, default: i32) -> i32 {
    m.get(key).and_then(|v| v.as_i32().ok()).unwrap_or(default)
}

fn get_bool(m: &Map, key: &str, default: bool) -> bool {
    m.get(key).and_then(|v| v.as_bool().ok()).unwrap_or(default)
}

fn get_str(m: &Map, key: &str, default: &str) -> String {
    m.get(key)
        .and_then(|v| v.as_string().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| default.to_string())
}

/// `[ { Values: [...] } ]` -> inner `[...]` values as Byml.
fn values_from_flag(m: &Map, key: &str) -> Vec<Byml> {
    m.get(key)
        .and_then(|v| v.as_array().ok())
        .and_then(|arr| arr.first())
        .and_then(|h| h.as_map().ok())
        .and_then(|inner| inner.get("Values"))
        .and_then(|v| v.as_array().ok())
        .map(|v| v.to_vec())
        .unwrap_or_default()
}

/// Load a scalar vector `[ [x, y, ...] ]` -> (f32...) tuple.
fn load_vec(m: &Map, key: &str, n: usize) -> Vec<f32> {
    m.get(key)
        .and_then(|v| v.as_array().ok())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_array().ok())
        .map(|v| {
            v.iter()
                .take(n)
                .filter_map(|f| f.as_float().ok())
                .collect()
        })
        .unwrap_or_default()
}

/// Build `[ [x, y, ...] ]` for scalar vec values.
fn vec_hash(values: &[f32]) -> Byml {
    Byml::Array(vec![Byml::Array(
        values.iter().map(|f| Byml::Float(*f)).collect(),
    )])
}

/// Build `[ { Values: [ [x, y], ... ] } ]` for vec arrays.
fn vec_array_hash(vectors: &[Vec<f32>]) -> Byml {
    let values = vectors
        .iter()
        .map(|v| Byml::Array(vec![vec_hash(v)]))
        .collect();
    let mut h: Map = Map::default();
    h.insert("Values".into(), Byml::Array(values));
    Byml::Array(vec![Byml::Map(h)])
}

/// `[ { Values: [ v1, v2, ... ] } ]` for scalar arrays.
fn array_hash(values: Vec<Byml>) -> Byml {
    let mut h: Map = Map::default();
    h.insert("Values".into(), Byml::Array(values));
    Byml::Array(vec![Byml::Map(h)])
}

impl Flag {
    pub fn new(kind: FlagKind, revival: bool) -> Self {
        Flag {
            data_name: String::new(),
            hash_value: 0,
            delete_rev: -1,
            is_event_associated: false,
            is_one_trigger: false,
            is_program_readable: true,
            is_program_writable: true,
            is_save: false,
            reset_type: 0,
            is_revival: revival,
            kind,
        }
    }

    pub fn set_data_name(&mut self, name: String) {
        self.hash_value = crc32_str(&name);
        self.data_name = name;
    }

    #[allow(dead_code)]
    pub fn exists(&self) -> bool {
        self.hash_value != 0
    }

    #[allow(dead_code)]
    pub fn name_contains(&self, search: &str) -> bool {
        self.data_name.contains(search)
    }

    /// Parse a flag hash (a `Byml::Map`) into a Flag.
    pub fn from_hash(kind: FlagKind, hash: &Map, revival: bool) -> Self {
        let data_name = get_str(hash, "DataName", "");
        let hash_value = crc32_str(&data_name);
        let delete_rev = get_i32(hash, "DeleteRev", -1);
        let is_event_associated = get_bool(hash, "IsEventAssociated", false);
        let is_one_trigger = get_bool(hash, "IsOneTrigger", false);
        let is_program_readable = get_bool(hash, "IsProgramReadable", true);
        let is_program_writable = get_bool(hash, "IsProgramWritable", true);
        let is_save = get_bool(hash, "IsSave", false);
        let reset_type = get_i32(hash, "ResetType", 0);

        let kind = match kind {
            FlagKind::Bool { .. } => FlagKind::Bool {
                category: get_i32(hash, "Category", -1),
                init_value: get_i32(hash, "InitValue", 0),
                max_value: get_bool(hash, "MaxValue", true),
                min_value: get_bool(hash, "MinValue", false),
            },
            FlagKind::BoolArray { .. } => FlagKind::BoolArray {
                init_value: values_from_flag(hash, "InitValue")
                    .iter()
                    .filter_map(|v| v.as_i32().ok())
                    .collect(),
                max_value: get_bool(hash, "MaxValue", true),
                min_value: get_bool(hash, "MinValue", false),
            },
            FlagKind::S32 { .. } => FlagKind::S32 {
                init_value: get_i32(hash, "InitValue", 0),
                max_value: get_i32(hash, "MaxValue", 2147483647),
                min_value: get_i32(hash, "MinValue", 0),
            },
            FlagKind::S32Array { .. } => FlagKind::S32Array {
                init_value: values_from_flag(hash, "InitValue")
                    .iter()
                    .filter_map(|v| v.as_i32().ok())
                    .collect(),
                max_value: get_i32(hash, "MaxValue", 6553500),
                min_value: get_i32(hash, "MinValue", -1),
            },
            FlagKind::F32 { .. } => FlagKind::F32 {
                init_value: get_i32(hash, "InitValue", 0) as f32,
                max_value: get_i32(hash, "MaxValue", 1000000) as f32,
                min_value: get_i32(hash, "MinValue", 0) as f32,
            },
            FlagKind::F32Array { .. } => FlagKind::F32Array {
                init_value: values_from_flag(hash, "InitValue")
                    .iter()
                    .filter_map(|v| v.as_float().ok())
                    .collect(),
                max_value: get_i32(hash, "MaxValue", 360) as f32,
                min_value: get_i32(hash, "MinValue", -1) as f32,
            },
            FlagKind::String { .. } => FlagKind::String {
                init_value: get_str(hash, "InitValue", ""),
                max_value: get_str(hash, "MaxValue", ""),
                min_value: get_str(hash, "MinValue", ""),
            },
            FlagKind::StringArray { .. } => FlagKind::StringArray {
                init_value: values_from_flag(hash, "InitValue")
                    .iter()
                    .filter_map(|v| v.as_string().ok().map(|s| s.to_string()))
                    .collect(),
                max_value: get_str(hash, "MaxValue", ""),
                min_value: get_str(hash, "MinValue", ""),
            },
            FlagKind::Vec2 { .. } => FlagKind::Vec2 {
                init_value: v2(&load_vec(hash, "InitValue", 2)),
                max_value: v2(&load_vec(hash, "MaxValue", 2)),
                min_value: v2(&load_vec(hash, "MinValue", 2)),
            },
            FlagKind::Vec3 { .. } => FlagKind::Vec3 {
                init_value: v3(&load_vec(hash, "InitValue", 3)),
                max_value: v3(&load_vec(hash, "MaxValue", 3)),
                min_value: v3(&load_vec(hash, "MinValue", 3)),
            },
            FlagKind::Vec4 { .. } => FlagKind::Vec4 {
                init_value: v4(&load_vec(hash, "InitValue", 4)),
                max_value: v4(&load_vec(hash, "MaxValue", 4)),
                min_value: v4(&load_vec(hash, "MinValue", 4)),
            },
            FlagKind::Vec2Array { .. } => FlagKind::Vec2Array {
                init_value: values_from_flag(hash, "InitValue")
                    .iter()
                    .map(|v| v2(&load_vec_from_byml(v, 2)))
                    .collect(),
                max_value: v2(&load_vec(hash, "MaxValue", 2)),
                min_value: v2(&load_vec(hash, "MinValue", 2)),
            },
            FlagKind::Vec3Array { .. } => FlagKind::Vec3Array {
                init_value: values_from_flag(hash, "InitValue")
                    .iter()
                    .map(|v| v3(&load_vec_from_byml(v, 3)))
                    .collect(),
                max_value: v3(&load_vec(hash, "MaxValue", 3)),
                min_value: v3(&load_vec(hash, "MinValue", 3)),
            },
        };

        Flag {
            data_name,
            hash_value,
            delete_rev,
            is_event_associated,
            is_one_trigger,
            is_program_readable,
            is_program_writable,
            is_save,
            reset_type,
            is_revival: revival,
            kind,
        }
    }

    /// Serialize to a BYML map for `gamedata` files.
    pub fn to_hash(&self) -> Map {
        let mut m: Map = Map::default();
        m.insert("DataName".into(), Byml::String(self.data_name.clone().into()));
        m.insert("DeleteRev".into(), Byml::I32(self.delete_rev));
        m.insert("HashValue".into(), Byml::I32(self.hash_value));
        m.insert("IsEventAssociated".into(), Byml::Bool(self.is_event_associated));
        m.insert("IsOneTrigger".into(), Byml::Bool(self.is_one_trigger));
        m.insert("IsProgramReadable".into(), Byml::Bool(self.is_program_readable));
        m.insert("IsProgramWritable".into(), Byml::Bool(self.is_program_writable));
        m.insert("IsSave".into(), Byml::Bool(self.is_save));
        m.insert("ResetType".into(), Byml::I32(self.reset_type));
        match &self.kind {
            FlagKind::Bool {
                category,
                init_value,
                max_value,
                min_value,
            } => {
                if *category != -1 {
                    m.insert("Category".into(), Byml::I32(*category));
                }
                m.insert("InitValue".into(), Byml::I32(*init_value));
                m.insert("MaxValue".into(), Byml::Bool(*max_value));
                m.insert("MinValue".into(), Byml::Bool(*min_value));
            }
            FlagKind::BoolArray {
                init_value,
                max_value,
                min_value,
            } => {
                m.insert(
                    "InitValue".into(),
                    array_hash(init_value.iter().map(|v| Byml::I32(*v)).collect()),
                );
                m.insert("MaxValue".into(), Byml::Bool(*max_value));
                m.insert("MinValue".into(), Byml::Bool(*min_value));
            }
            FlagKind::S32 {
                init_value,
                max_value,
                min_value,
            } => {
                m.insert("InitValue".into(), Byml::I32(*init_value));
                m.insert("MaxValue".into(), Byml::I32(*max_value));
                m.insert("MinValue".into(), Byml::I32(*min_value));
            }
            FlagKind::S32Array {
                init_value,
                max_value,
                min_value,
            } => {
                m.insert(
                    "InitValue".into(),
                    array_hash(init_value.iter().map(|v| Byml::I32(*v)).collect()),
                );
                m.insert("MaxValue".into(), Byml::I32(*max_value));
                m.insert("MinValue".into(), Byml::I32(*min_value));
            }
            FlagKind::F32 {
                init_value,
                max_value,
                min_value,
            } => {
                m.insert("InitValue".into(), Byml::Float(*init_value));
                m.insert("MaxValue".into(), Byml::Float(*max_value));
                m.insert("MinValue".into(), Byml::Float(*min_value));
            }
            FlagKind::F32Array {
                init_value,
                max_value,
                min_value,
            } => {
                m.insert(
                    "InitValue".into(),
                    array_hash(init_value.iter().map(|v| Byml::Float(*v)).collect()),
                );
                m.insert("MaxValue".into(), Byml::Float(*max_value));
                m.insert("MinValue".into(), Byml::Float(*min_value));
            }
            FlagKind::String {
                init_value,
                max_value,
                min_value,
            } => {
                m.insert("InitValue".into(), Byml::String(init_value.clone().into()));
                m.insert("MaxValue".into(), Byml::String(max_value.clone().into()));
                m.insert("MinValue".into(), Byml::String(min_value.clone().into()));
            }
            FlagKind::StringArray {
                init_value,
                max_value,
                min_value,
            } => {
                m.insert(
                    "InitValue".into(),
                    array_hash(
                        init_value
                            .iter()
                            .map(|v| Byml::String(v.clone().into()))
                            .collect(),
                    ),
                );
                m.insert("MaxValue".into(), Byml::String(max_value.clone().into()));
                m.insert("MinValue".into(), Byml::String(min_value.clone().into()));
            }
            FlagKind::Vec2 {
                init_value,
                max_value,
                min_value,
            } => {
                m.insert("InitValue".into(), vec_hash(&[init_value.0, init_value.1]));
                m.insert("MaxValue".into(), vec_hash(&[max_value.0, max_value.1]));
                m.insert("MinValue".into(), vec_hash(&[min_value.0, min_value.1]));
            }
            FlagKind::Vec3 {
                init_value,
                max_value,
                min_value,
            } => {
                m.insert(
                    "InitValue".into(),
                    vec_hash(&[init_value.0, init_value.1, init_value.2]),
                );
                m.insert(
                    "MaxValue".into(),
                    vec_hash(&[max_value.0, max_value.1, max_value.2]),
                );
                m.insert(
                    "MinValue".into(),
                    vec_hash(&[min_value.0, min_value.1, min_value.2]),
                );
            }
            FlagKind::Vec4 {
                init_value,
                max_value,
                min_value,
            } => {
                m.insert(
                    "InitValue".into(),
                    vec_hash(&[init_value.0, init_value.1, init_value.2, init_value.3]),
                );
                m.insert(
                    "MaxValue".into(),
                    vec_hash(&[max_value.0, max_value.1, max_value.2, max_value.3]),
                );
                m.insert(
                    "MinValue".into(),
                    vec_hash(&[min_value.0, min_value.1, min_value.2, min_value.3]),
                );
            }
            FlagKind::Vec2Array {
                init_value,
                max_value,
                min_value,
            } => {
                m.insert(
                    "InitValue".into(),
                    vec_array_hash(&init_value.iter().map(|v| vec![v.0, v.1]).collect::<Vec<_>>()),
                );
                m.insert("MaxValue".into(), vec_hash(&[max_value.0, max_value.1]));
                m.insert("MinValue".into(), vec_hash(&[min_value.0, min_value.1]));
            }
            FlagKind::Vec3Array {
                init_value,
                max_value,
                min_value,
            } => {
                m.insert(
                    "InitValue".into(),
                    vec_array_hash(
                        &init_value
                            .iter()
                            .map(|v| vec![v.0, v.1, v.2])
                            .collect::<Vec<_>>(),
                    ),
                );
                m.insert(
                    "MaxValue".into(),
                    vec_hash(&[max_value.0, max_value.1, max_value.2]),
                );
                m.insert(
                    "MinValue".into(),
                    vec_hash(&[min_value.0, min_value.1, min_value.2]),
                );
            }
        }
        m
    }

    /// Serialize to the minimal hash used in `saveformat` files.
    pub fn to_sv_hash(&self) -> Map {
        let mut m: Map = Map::default();
        m.insert("DataName".into(), Byml::String(self.data_name.clone().into()));
        m.insert("HashValue".into(), Byml::I32(self.hash_value));
        m
    }

    /// Applies the override rules from overrides.json, mirroring the
    /// Python `use_name_to_override_params` (last matching rule wins).
    pub fn use_name_to_override_params(&mut self) {
        let t = &*OVERRIDE_TABLES;
        for (re, v) in &t.is_event_associated {
            if re.is_match(&self.data_name) {
                self.is_event_associated = *v;
            }
        }
        for (re, v) in &t.is_one_trigger {
            if re.is_match(&self.data_name) {
                self.is_one_trigger = *v;
            }
        }
        for (re, v) in &t.is_program_readable {
            if re.is_match(&self.data_name) {
                self.is_program_readable = *v;
            }
        }
        for (re, v) in &t.is_program_writable {
            if re.is_match(&self.data_name) {
                self.is_program_writable = *v;
            }
        }
        for (re, v) in &t.is_save {
            if re.is_match(&self.data_name) {
                self.is_save = *v;
            }
        }
        for (re, v) in &t.reset_type {
            if re.is_match(&self.data_name) {
                self.reset_type = *v;
            }
        }
        match &mut self.kind {
            FlagKind::Bool {
                category,
                init_value,
                ..
            } => {
                for (re, v) in &t.bool_category {
                    if re.is_match(&self.data_name) {
                        *category = *v;
                    }
                }
                for (re, v) in &t.bool_init_value {
                    if re.is_match(&self.data_name) {
                        *init_value = *v;
                    }
                }
            }
            FlagKind::BoolArray {
                init_value,
                max_value,
                min_value,
            } => {
                for (re, v) in &t.bool_array_init_value {
                    if re.is_match(&self.data_name) {
                        *init_value = v.clone();
                    }
                }
                for (re, v) in &t.bool_array_max_value {
                    if re.is_match(&self.data_name) {
                        *max_value = *v;
                    }
                }
                for (re, v) in &t.bool_array_min_value {
                    if re.is_match(&self.data_name) {
                        *min_value = *v;
                    }
                }
            }
            FlagKind::S32 {
                init_value,
                max_value,
                min_value,
            } => {
                for (re, v) in &t.s32_init_value {
                    if re.is_match(&self.data_name) {
                        *init_value = *v;
                    }
                }
                for (re, v) in &t.s32_max_value {
                    if re.is_match(&self.data_name) {
                        *max_value = *v;
                    }
                }
                for (re, v) in &t.s32_min_value {
                    if re.is_match(&self.data_name) {
                        *min_value = *v;
                    }
                }
            }
            FlagKind::S32Array {
                init_value,
                max_value,
                min_value,
            } => {
                for (re, v) in &t.s32_array_init_value {
                    if re.is_match(&self.data_name) {
                        *init_value = v.clone();
                    }
                }
                for (re, v) in &t.s32_array_max_value {
                    if re.is_match(&self.data_name) {
                        *max_value = *v;
                    }
                }
                for (re, v) in &t.s32_array_min_value {
                    if re.is_match(&self.data_name) {
                        *min_value = *v;
                    }
                }
            }
            FlagKind::F32 {
                init_value,
                max_value,
                min_value,
            } => {
                for (re, v) in &t.f32_init_value {
                    if re.is_match(&self.data_name) {
                        *init_value = *v;
                    }
                }
                for (re, v) in &t.f32_max_value {
                    if re.is_match(&self.data_name) {
                        *max_value = *v;
                    }
                }
                for (re, v) in &t.f32_min_value {
                    if re.is_match(&self.data_name) {
                        *min_value = *v;
                    }
                }
            }
            FlagKind::F32Array {
                init_value,
                max_value,
                min_value,
            } => {
                for (re, v) in &t.f32_array_init_value {
                    if re.is_match(&self.data_name) {
                        *init_value = v.clone();
                    }
                }
                for (re, v) in &t.f32_array_max_value {
                    if re.is_match(&self.data_name) {
                        *max_value = *v;
                    }
                }
                for (re, v) in &t.f32_array_min_value {
                    if re.is_match(&self.data_name) {
                        *min_value = *v;
                    }
                }
            }
            FlagKind::String {
                init_value,
                max_value,
                min_value,
            } => {
                for (re, v) in &t.string_init_value {
                    if re.is_match(&self.data_name) {
                        *init_value = v.clone();
                    }
                }
                for (re, v) in &t.string_max_value {
                    if re.is_match(&self.data_name) {
                        *max_value = v.clone();
                    }
                }
                for (re, v) in &t.string_min_value {
                    if re.is_match(&self.data_name) {
                        *min_value = v.clone();
                    }
                }
            }
            FlagKind::StringArray {
                init_value,
                max_value,
                min_value,
            } => {
                for (re, v) in &t.string_array_init_value {
                    if re.is_match(&self.data_name) {
                        *init_value = v.clone();
                    }
                }
                for (re, v) in &t.string_array_max_value {
                    if re.is_match(&self.data_name) {
                        *max_value = v.clone();
                    }
                }
                for (re, v) in &t.string_array_min_value {
                    if re.is_match(&self.data_name) {
                        *min_value = v.clone();
                    }
                }
            }
            FlagKind::Vec2 {
                init_value,
                max_value,
                min_value,
            } => {
                for (re, v) in &t.vec2_init_value {
                    if re.is_match(&self.data_name) {
                        *init_value = v2(v);
                    }
                }
                for (re, v) in &t.vec2_max_value {
                    if re.is_match(&self.data_name) {
                        *max_value = v2(v);
                    }
                }
                for (re, v) in &t.vec2_min_value {
                    if re.is_match(&self.data_name) {
                        *min_value = v2(v);
                    }
                }
            }
            FlagKind::Vec2Array {
                init_value,
                max_value,
                min_value,
            } => {
                for (re, v) in &t.vec2_array_init_value {
                    if re.is_match(&self.data_name) {
                        *init_value = v.iter().map(|x| v2(x)).collect();
                    }
                }
                for (re, v) in &t.vec2_array_max_value {
                    if re.is_match(&self.data_name) {
                        *max_value = v2(v);
                    }
                }
                for (re, v) in &t.vec2_array_min_value {
                    if re.is_match(&self.data_name) {
                        *min_value = v2(v);
                    }
                }
            }
            FlagKind::Vec3 {
                init_value,
                max_value,
                min_value,
            } => {
                for (re, v) in &t.vec3_init_value {
                    if re.is_match(&self.data_name) {
                        *init_value = v3(v);
                    }
                }
                for (re, v) in &t.vec3_max_value {
                    if re.is_match(&self.data_name) {
                        *max_value = v3(v);
                    }
                }
                for (re, v) in &t.vec3_min_value {
                    if re.is_match(&self.data_name) {
                        *min_value = v3(v);
                    }
                }
            }
            FlagKind::Vec3Array {
                init_value,
                max_value,
                min_value,
            } => {
                for (re, v) in &t.vec3_array_init_value {
                    if re.is_match(&self.data_name) {
                        *init_value = v.iter().map(|x| v3(x)).collect();
                    }
                }
                for (re, v) in &t.vec3_array_max_value {
                    if re.is_match(&self.data_name) {
                        *max_value = v3(v);
                    }
                }
                for (re, v) in &t.vec3_array_min_value {
                    if re.is_match(&self.data_name) {
                        *min_value = v3(v);
                    }
                }
            }
            FlagKind::Vec4 {
                init_value,
                max_value,
                min_value,
            } => {
                for (re, v) in &t.vec4_init_value {
                    if re.is_match(&self.data_name) {
                        *init_value = v4(v);
                    }
                }
                for (re, v) in &t.vec4_max_value {
                    if re.is_match(&self.data_name) {
                        *max_value = v4(v);
                    }
                }
                for (re, v) in &t.vec4_min_value {
                    if re.is_match(&self.data_name) {
                        *min_value = v4(v);
                    }
                }
            }
        }
    }
}

/// Vec array init values: each Values element is `[ [x, y] ]`.
fn load_vec_from_byml(v: &Byml, n: usize) -> Vec<f32> {
    v.as_array()
        .ok()
        .and_then(|arr| arr.first())
        .and_then(|vec| vec.as_array().ok())
        .map(|vec| {
            vec.iter()
                .take(n)
                .filter_map(|f| f.as_float().ok())
                .collect()
        })
        .unwrap_or_default()
}

fn v2(v: &[f32]) -> (f32, f32) {
    (v.first().copied().unwrap_or(0.0), v.get(1).copied().unwrap_or(0.0))
}

fn v3(v: &[f32]) -> (f32, f32, f32) {
    (
        v.first().copied().unwrap_or(0.0),
        v.get(1).copied().unwrap_or(0.0),
        v.get(2).copied().unwrap_or(0.0),
    )
}

fn v4(v: &[f32]) -> (f32, f32, f32, f32) {
    (
        v.first().copied().unwrap_or(0.0),
        v.get(1).copied().unwrap_or(0.0),
        v.get(2).copied().unwrap_or(0.0),
        v.get(3).copied().unwrap_or(0.0),
    )
}

// ---------------------------------------------------------------------------
// Override rules (from overrides.json)
// ---------------------------------------------------------------------------

fn sub<'a>(cat: &'a Value, name: &str) -> &'a Value {
    cat.get(name).unwrap_or(&Value::Null)
}

fn parse_regex(p: &str) -> Option<Regex> {
    match Regex::new(p) {
        Ok(r) => Some(r),
        Err(e) => {
            eprintln!("ignoring invalid override regex `{p}`: {e}");
            None
        }
    }
}

fn rules_bool(cat: &Value, name: &str) -> Vec<(Regex, bool)> {
    sub(cat, name)
        .as_object()
        .map(|m| {
            m.iter()
                .filter_map(|(p, v)| parse_regex(p).map(|r| (r, v.as_bool().unwrap_or(false))))
                .collect()
        })
        .unwrap_or_default()
}

fn rules_i32(cat: &Value, name: &str) -> Vec<(Regex, i32)> {
    sub(cat, name)
        .as_object()
        .map(|m| {
            m.iter()
                .filter_map(|(p, v)| parse_regex(p).map(|r| (r, v.as_i64().unwrap_or(0) as i32)))
                .collect()
        })
        .unwrap_or_default()
}

fn rules_i32_array(cat: &Value, name: &str) -> Vec<(Regex, Vec<i32>)> {
    sub(cat, name)
        .as_object()
        .map(|m| {
            m.iter()
                .filter_map(|(p, v)| {
                    let vals = v
                        .as_array()
                        .map(|a| a.iter().filter_map(|x| x.as_i64().map(|i| i as i32)).collect())
                        .unwrap_or_default();
                    parse_regex(p).map(|r| (r, vals))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn rules_f32(cat: &Value, name: &str) -> Vec<(Regex, f32)> {
    sub(cat, name)
        .as_object()
        .map(|m| {
            m.iter()
                .filter_map(|(p, v)| parse_regex(p).map(|r| (r, v.as_f64().unwrap_or(0.0) as f32)))
                .collect()
        })
        .unwrap_or_default()
}

fn rules_f32_array(cat: &Value, name: &str) -> Vec<(Regex, Vec<f32>)> {
    sub(cat, name)
        .as_object()
        .map(|m| {
            m.iter()
                .filter_map(|(p, v)| {
                    let vals = v
                        .as_array()
                        .map(|a| a.iter().filter_map(|x| x.as_f64().map(|f| f as f32)).collect())
                        .unwrap_or_default();
                    parse_regex(p).map(|r| (r, vals))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn rules_f32_array_array(cat: &Value, name: &str) -> Vec<(Regex, Vec<Vec<f32>>)> {
    sub(cat, name)
        .as_object()
        .map(|m| {
            m.iter()
                .filter_map(|(p, v)| {
                    let vals = v
                        .as_array()
                        .map(|a| {
                            a.iter()
                                .map(|x| {
                                    x.as_array()
                                        .map(|i| {
                                            i.iter()
                                                .filter_map(|f| f.as_f64().map(|f| f as f32))
                                                .collect()
                                        })
                                        .unwrap_or_default()
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    parse_regex(p).map(|r| (r, vals))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn rules_string(cat: &Value, name: &str) -> Vec<(Regex, String)> {
    sub(cat, name)
        .as_object()
        .map(|m| {
            m.iter()
                .filter_map(|(p, v)| {
                    parse_regex(p).map(|r| (r, v.as_str().unwrap_or("").to_string()))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn rules_string_array(cat: &Value, name: &str) -> Vec<(Regex, Vec<String>)> {
    sub(cat, name)
        .as_object()
        .map(|m| {
            m.iter()
                .filter_map(|(p, v)| {
                    let vals = v
                        .as_array()
                        .map(|a| {
                            a.iter()
                                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_default();
                    parse_regex(p).map(|r| (r, vals))
                })
                .collect()
        })
        .unwrap_or_default()
}

pub struct OverrideTables {
    pub is_event_associated: Vec<(Regex, bool)>,
    pub is_one_trigger: Vec<(Regex, bool)>,
    pub is_program_readable: Vec<(Regex, bool)>,
    pub is_program_writable: Vec<(Regex, bool)>,
    pub is_save: Vec<(Regex, bool)>,
    pub reset_type: Vec<(Regex, i32)>,
    pub bool_category: Vec<(Regex, i32)>,
    pub bool_init_value: Vec<(Regex, i32)>,
    pub bool_array_init_value: Vec<(Regex, Vec<i32>)>,
    pub bool_array_max_value: Vec<(Regex, bool)>,
    pub bool_array_min_value: Vec<(Regex, bool)>,
    pub s32_init_value: Vec<(Regex, i32)>,
    pub s32_max_value: Vec<(Regex, i32)>,
    pub s32_min_value: Vec<(Regex, i32)>,
    pub s32_array_init_value: Vec<(Regex, Vec<i32>)>,
    pub s32_array_max_value: Vec<(Regex, i32)>,
    pub s32_array_min_value: Vec<(Regex, i32)>,
    pub f32_init_value: Vec<(Regex, f32)>,
    pub f32_max_value: Vec<(Regex, f32)>,
    pub f32_min_value: Vec<(Regex, f32)>,
    pub f32_array_init_value: Vec<(Regex, Vec<f32>)>,
    pub f32_array_max_value: Vec<(Regex, f32)>,
    pub f32_array_min_value: Vec<(Regex, f32)>,
    pub string_init_value: Vec<(Regex, String)>,
    pub string_max_value: Vec<(Regex, String)>,
    pub string_min_value: Vec<(Regex, String)>,
    pub string_array_init_value: Vec<(Regex, Vec<String>)>,
    pub string_array_max_value: Vec<(Regex, String)>,
    pub string_array_min_value: Vec<(Regex, String)>,
    pub vec2_init_value: Vec<(Regex, Vec<f32>)>,
    pub vec2_max_value: Vec<(Regex, Vec<f32>)>,
    pub vec2_min_value: Vec<(Regex, Vec<f32>)>,
    pub vec2_array_init_value: Vec<(Regex, Vec<Vec<f32>>)>,
    pub vec2_array_max_value: Vec<(Regex, Vec<f32>)>,
    pub vec2_array_min_value: Vec<(Regex, Vec<f32>)>,
    pub vec3_init_value: Vec<(Regex, Vec<f32>)>,
    pub vec3_max_value: Vec<(Regex, Vec<f32>)>,
    pub vec3_min_value: Vec<(Regex, Vec<f32>)>,
    pub vec3_array_init_value: Vec<(Regex, Vec<Vec<f32>>)>,
    pub vec3_array_max_value: Vec<(Regex, Vec<f32>)>,
    pub vec3_array_min_value: Vec<(Regex, Vec<f32>)>,
    pub vec4_init_value: Vec<(Regex, Vec<f32>)>,
    pub vec4_max_value: Vec<(Regex, Vec<f32>)>,
    pub vec4_min_value: Vec<(Regex, Vec<f32>)>,
}

pub static OVERRIDE_TABLES: LazyLock<OverrideTables> = LazyLock::new(|| {
    let v = OVERRIDES.as_object().expect("overrides is an object");
    let std = v.get("STANDARD_OVERRIDES").unwrap_or(&Value::Null);
    let bools = v.get("BOOL_OVERRIDES").unwrap_or(&Value::Null);
    let bool_arrays = v.get("BOOL_ARRAY_OVERRIDES").unwrap_or(&Value::Null);
    let s32 = v.get("S32_OVERRIDES").unwrap_or(&Value::Null);
    let s32_arrays = v.get("S32_ARRAY_OVERRIDES").unwrap_or(&Value::Null);
    let f32 = v.get("F32_OVERRIDES").unwrap_or(&Value::Null);
    let f32_arrays = v.get("F32_ARRAY_OVERRIDES").unwrap_or(&Value::Null);
    let strings = v.get("STRING_OVERRIDES").unwrap_or(&Value::Null);
    let string_arrays = v.get("STRING_ARRAY_OVERRIDES").unwrap_or(&Value::Null);
    let vec2 = v.get("VEC2_OVERRIDES").unwrap_or(&Value::Null);
    let vec2_arrays = v.get("VEC2_ARRAY_OVERRIDES").unwrap_or(&Value::Null);
    let vec3 = v.get("VEC3_OVERRIDES").unwrap_or(&Value::Null);
    let vec3_arrays = v.get("VEC3_ARRAY_OVERRIDES").unwrap_or(&Value::Null);
    let vec4 = v.get("VEC4_OVERRIDES").unwrap_or(&Value::Null);
    OverrideTables {
        is_event_associated: rules_bool(std, "OVERRIDE_IS_EVENT_ASSOCIATED"),
        is_one_trigger: rules_bool(std, "OVERRIDE_IS_ONE_TRIGGER"),
        is_program_readable: rules_bool(std, "OVERRIDE_IS_PROGRAM_READABLE"),
        is_program_writable: rules_bool(std, "OVERRIDE_IS_PROGRAM_WRITABLE"),
        is_save: rules_bool(std, "OVERRIDE_IS_SAVE"),
        reset_type: rules_i32(std, "OVERRIDE_RESET_TYPE"),
        bool_category: rules_i32(bools, "OVERRIDE_BOOL_CATEGORY"),
        bool_init_value: rules_i32(bools, "OVERRIDE_BOOL_INIT_VALUE"),
        bool_array_init_value: rules_i32_array(bool_arrays, "OVERRIDE_BOOL_ARRAY_INIT_VALUE"),
        bool_array_max_value: rules_bool(bool_arrays, "OVERRIDE_BOOL_ARRAY_MAX_VALUE"),
        bool_array_min_value: rules_bool(bool_arrays, "OVERRIDE_BOOL_ARRAY_MIN_VALUE"),
        s32_init_value: rules_i32(s32, "OVERRIDE_S32_INIT_VALUE"),
        s32_max_value: rules_i32(s32, "OVERRIDE_S32_MAX_VALUE"),
        s32_min_value: rules_i32(s32, "OVERRIDE_S32_MIN_VALUE"),
        s32_array_init_value: rules_i32_array(s32_arrays, "OVERRIDE_S32_ARRAY_INIT_VALUE"),
        s32_array_max_value: rules_i32(s32_arrays, "OVERRIDE_S32_ARRAY_MAX_VALUE"),
        s32_array_min_value: rules_i32(s32_arrays, "OVERRIDE_S32_ARRAY_MIN_VALUE"),
        f32_init_value: rules_f32(f32, "OVERRIDE_F32_INIT_VALUE"),
        f32_max_value: rules_f32(f32, "OVERRIDE_F32_MAX_VALUE"),
        f32_min_value: rules_f32(f32, "OVERRIDE_F32_MIN_VALUE"),
        f32_array_init_value: rules_f32_array(f32_arrays, "OVERRIDE_F32_ARRAY_INIT_VALUE"),
        f32_array_max_value: rules_f32(f32_arrays, "OVERRIDE_F32_ARRAY_MAX_VALUE"),
        f32_array_min_value: rules_f32(f32_arrays, "OVERRIDE_F32_ARRAY_MIN_VALUE"),
        string_init_value: rules_string(strings, "OVERRIDE_STRING_INIT_VALUE"),
        string_max_value: rules_string(strings, "OVERRIDE_STRING_MAX_VALUE"),
        string_min_value: rules_string(strings, "OVERRIDE_STRING_MIN_VALUE"),
        string_array_init_value: rules_string_array(string_arrays, "OVERRIDE_STRING_ARRAY_INIT_VALUE"),
        string_array_max_value: rules_string(string_arrays, "OVERRIDE_STRING_ARRAY_MAX_VALUE"),
        string_array_min_value: rules_string(string_arrays, "OVERRIDE_STRING_ARRAY_MIN_VALUE"),
        vec2_init_value: rules_f32_array(vec2, "OVERRIDE_VEC2_INIT_VALUE"),
        vec2_max_value: rules_f32_array(vec2, "OVERRIDE_VEC2_MAX_VALUE"),
        vec2_min_value: rules_f32_array(vec2, "OVERRIDE_VEC2_MIN_VALUE"),
        vec2_array_init_value: rules_f32_array_array(vec2_arrays, "OVERRIDE_VEC2_ARRAY_INIT_VALUE"),
        vec2_array_max_value: rules_f32_array(vec2_arrays, "OVERRIDE_VEC2_ARRAY_MAX_VALUE"),
        vec2_array_min_value: rules_f32_array(vec2_arrays, "OVERRIDE_VEC2_ARRAY_MIN_VALUE"),
        vec3_init_value: rules_f32_array(vec3, "OVERRIDE_VEC3_INIT_VALUE"),
        vec3_max_value: rules_f32_array(vec3, "OVERRIDE_VEC3_MAX_VALUE"),
        vec3_min_value: rules_f32_array(vec3, "OVERRIDE_VEC3_MIN_VALUE"),
        vec3_array_init_value: rules_f32_array_array(vec3_arrays, "OVERRIDE_VEC3_ARRAY_INIT_VALUE"),
        vec3_array_max_value: rules_f32_array(vec3_arrays, "OVERRIDE_VEC3_ARRAY_MAX_VALUE"),
        vec3_array_min_value: rules_f32_array(vec3_arrays, "OVERRIDE_VEC3_ARRAY_MIN_VALUE"),
        vec4_init_value: rules_f32_array(vec4, "OVERRIDE_VEC4_INIT_VALUE"),
        vec4_max_value: rules_f32_array(vec4, "OVERRIDE_VEC4_MAX_VALUE"),
        vec4_min_value: rules_f32_array(vec4, "OVERRIDE_VEC4_MIN_VALUE"),
    }
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32_matches_zlib_wrap() {
        // Verified against Python: zlib.crc32(b"Hello, world!") == 0xEBE6C6E6
        assert_eq!(crc32_str("Hello, world!"), 0xEBE6C6E6u32 as i32);
        // Flag-style name: zlib.crc32(b"IsGet_Armor_001_Head") == 0x46711F96
        assert_eq!(crc32_str("IsGet_Armor_001_Head"), 0x46711F96u32 as i32);
    }

    #[test]
    fn flag_roundtrip() {
        let mut f = Flag::new(
            FlagKind::Bool {
                category: 1,
                init_value: 1,
                max_value: true,
                min_value: true,
            },
            false,
        );
        f.set_data_name("IsTest_".to_string());
        f.use_name_to_override_params();
        let m = f.to_hash();
        let f2 = Flag::from_hash(
            FlagKind::Bool {
                category: 0,
                init_value: 0,
                max_value: false,
                min_value: false,
            },
            &m,
            false,
        );
        assert_eq!(f.hash_value, f2.hash_value);
        assert_eq!(f.to_hash(), f2.to_hash());
    }

    #[test]
    fn flag_vec_roundtrip() {
        let mut f = Flag::new(
            FlagKind::Vec3 {
                init_value: (1.5, 2.5, 3.5),
                max_value: (0.0, 0.0, 0.0),
                min_value: (0.0, 0.0, 0.0),
            },
            false,
        );
        f.set_data_name("VecTest_".to_string());
        let m = f.to_hash();
        let f2 = Flag::from_hash(
            FlagKind::Vec3 {
                init_value: (0.0, 0.0, 0.0),
                max_value: (0.0, 0.0, 0.0),
                min_value: (0.0, 0.0, 0.0),
            },
            &m,
            false,
        );
        assert_eq!(f.to_hash(), f2.to_hash());
    }
}
