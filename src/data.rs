//! Static game data loaded from JSON files in `data/`.
//!
//! All of these files are direct copies of the ones shipped with the original
//! Python tool, loaded lazily on first use.

use std::collections::HashMap;
use std::sync::LazyLock;

use serde_json::Value;

/// `generic_link_files.json`:
/// link name -> file reference -> name of a vanilla actor pack that contains it.
pub static GENERIC_LINK_FILES: LazyLock<HashMap<String, HashMap<String, String>>> =
    LazyLock::new(|| {
        serde_json::from_str(include_str!("../data/generic_link_files.json"))
            .expect("generic_link_files.json is valid")
    });

/// `keys_by_profile.json`:
/// profile -> list of ActorInfo entry keys that are legal for that profile.
pub static KEYS_PER_PROFILE: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| {
    let v: Value =
        serde_json::from_str(include_str!("../data/keys_by_profile.json")).unwrap();
    v["keys_per_profile"]
        .as_object()
        .expect("keys_per_profile is an object")
        .iter()
        .map(|(k, v)| {
            (
                k.clone(),
                v.as_array()
                    .expect("keys_per_profile values are arrays")
                    .iter()
                    .map(|x| x.as_str().expect("keys are strings").to_string())
                    .collect(),
            )
        })
        .collect()
});

/// `overrides.json`, parsed with `preserve_order` so that rule application
/// matches the Python `json` behaviour (later entries win).
pub static OVERRIDES: LazyLock<Value> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../data/overrides.json"))
        .expect("overrides.json is valid")
});

/// Returns a set of keys for a profile, or an empty set if unknown.
pub fn keys_for_profile(profile: &str) -> &'static [String] {
    KEYS_PER_PROFILE
        .get(profile)
        .map(|v| v.as_slice())
        .unwrap_or(&[])
}

/// Whether a file reference is a vanilla file for the given link.
pub fn get_generic_file(link: &str, file_ref: &str) -> Option<&'static String> {
    GENERIC_LINK_FILES
        .get(link)
        .and_then(|m| m.get(file_ref))
}
