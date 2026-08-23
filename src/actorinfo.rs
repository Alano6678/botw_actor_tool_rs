//! ActorInfo entry generation-?port of the Python `actorinfo.py`.
//!
//! This automates a system that was originally done manually: it pulls
//! relevant params out of the AAMP files referenced by an actor and writes a
//! matching `ActorInfo.product.sbyml` entry.

use std::collections::HashMap;

use roead::aamp::{Parameter, ParameterIO, ParameterObject};
use roead::byml::{Byml, Map};

use crate::data;
use crate::pack::ActorPack;

pub const ACTORLINK_KEYS: &[(&str, &str)] = &[
    ("actorScale", "ActorScale"),
    ("elink", "ElinkUser"),
    ("profile", "ProfileUser"),
    ("slink", "SlinkUser"),
    ("xlink", "XlinkUser"),
];

const DROPTABLE_ARRAY_KEYS: &[&str] = &[
    "ItemName01",
    "ItemName02",
    "ItemName03",
    "ItemName04",
    "ItemName05",
    "ItemName06",
    "ItemName07",
    "ItemName08",
    "ItemName09",
    "ItemName10",
];
const DROPTABLE_TABLES: &[&str] = &["Normal", "Normal2", "Normal3", "Normal4", "Normal5"];

#[derive(Clone, Copy, Debug)]
pub enum FieldType {
    Str,
    I32,
    F32,
    Bool,
}

#[derive(Clone, Debug)]
pub struct Field {
    pub info: &'static str,
    pub param: &'static str,
    pub ftype: FieldType,
    pub check_null: bool,
}

macro_rules! fields {
    ($info:literal, $param:literal, $ftype:tt, $check:literal) => {
        Field {
            info: $info,
            param: $param,
            ftype: FieldType::$ftype,
            check_null: $check,
        }
    };
}

pub const GPARAMLIST_KEYS: &[(&str, &[Field])] = &[
    (
        "AnimalUnit",
        &[fields!("animalUnitBasePlayRate", "BasePlayRate", F32, false)],
    ),
    (
        "Armor",
        &[
            fields!("armorDefenceAddLevel", "DefenceAddLevel", I32, false),
            fields!("armorNextRankName", "NextRankName", Str, false),
            fields!("armorStarNum", "StarNum", I32, false),
        ],
    ),
    (
        "ArmorEffect",
        &[
            fields!("armorEffectAncientPowUp", "AncientPowUp", Bool, false),
            fields!("armorEffectEffectLevel", "EffectLevel", I32, false),
            fields!("armorEffectEffectType", "EffectType", Str, false),
            fields!("armorEffectEnableClimbWaterfall", "EnableClimbWaterfall", Bool, false),
            fields!("armorEffectEnableSpinAttack", "EnableSpinAttack", Bool, false),
        ],
    ),
    (
        "ArmorHead",
        &[fields!("armorHeadMantleType", "HeadMantleType", I32, false)],
    ),
    (
        "ArmorLower",
        &[fields!("armorLowerDisableSelfMantle", "DisableSelfMantle", Bool, false)],
    ),
    (
        "ArmorUpper",
        &[
            fields!("armorUpperDisableSelfMantle", "DisableSelfMantle", Bool, false),
            fields!("armorUpperUseMantleType", "UseMantleType", I32, false),
        ],
    ),
    (
        "Arrow",
        &[
            fields!("arrowArrowDeletePer", "ArrowDeletePer", I32, false),
            fields!("arrowArrowNum", "ArrowNum", I32, false),
            fields!("arrowDeleteTime", "DeleteTime", I32, false),
            fields!("arrowDeleteTimeWithChemical", "DeleteTimeWithChemical", I32, false),
            fields!("arrowEnemyShootNumForDelete", "EnemyShootNumForDelete", I32, false),
        ],
    ),
    ("Attack", &[fields!("attackPower", "Power", I32, false)]),
    (
        "Bow",
        &[
            fields!("bowArrowName", "ArrowName", Str, false),
            fields!("bowIsLeadShot", "IsLeadShot", Bool, false),
            fields!("bowIsRapidFire", "IsRapidFire", Bool, false),
            fields!("bowLeadShotNum", "LeadShotNum", I32, false),
            fields!("bowRapidFireNum", "RapidFireNum", I32, false),
        ],
    ),
    (
        "CookSpice",
        &[
            fields!("cookSpiceBoostEffectiveTime", "BoostEffectiveTime", I32, false),
            fields!("cookSpiceBoostHitPointRecover", "BoostHitPointRecover", I32, false),
            fields!("cookSpiceBoostMaxHeartLevel", "BoostMaxHeartLevel", I32, false),
            fields!("cookSpiceBoostStaminaLevel", "BoostStaminaLevel", I32, false),
            fields!("cookSpiceBoostSuccessRate", "BoostSuccessRate", I32, false),
        ],
    ),
    (
        "CureItem",
        &[
            fields!("cureItemEffectLevel", "EffectLevel", I32, false),
            fields!("cureItemEffectType", "EffectType", Str, false),
            fields!("cureItemEffectiveTime", "EffectiveTime", I32, true),
            fields!("cureItemHitPointRecover", "HitPointRecover", I32, false),
        ],
    ),
    ("Enemy", &[fields!("enemyRank", "Rank", I32, false)]),
    ("General", &[fields!("generalLife", "Life", I32, false)]),
    (
        "Horse",
        &[
            fields!("horseASVariation", "ASVariation", Str, false),
            fields!("horseGearTopChargeNum", "GearTopChargeNum", I32, false),
            fields!("horseNature", "Nature", I32, false),
        ],
    ),
    (
        "HorseUnit",
        &[fields!("horseUnitRiddenAnimalType", "RiddenAnimalType", I32, false)],
    ),
    (
        "Item",
        &[
            fields!("itemBuyingPrice", "BuyingPrice", I32, false),
            fields!("itemCreatingPrice", "CreatingPrice", I32, false),
            fields!("itemSaleRevivalCount", "SaleRevivalCount", I32, false),
            fields!("itemSellingPrice", "SellingPrice", I32, false),
            fields!("itemStainColor", "StainColor", I32, false),
            fields!("itemUseIconActorName", "UseIconActorName", Str, false),
        ],
    ),
    (
        "MasterSword",
        &[
            fields!("masterSwordSearchEvilDist", "SearchEvilDist", F32, false),
            fields!("masterSwordSleepActorName", "SleepActorName", Str, false),
            fields!("masterSwordTrueFormActorName", "TrueFormActorName", Str, false),
            fields!("masterSwordTrueFormAttackPower", "TrueFormAttackPower", I32, false),
        ],
    ),
    (
        "MonsterShop",
        &[
            fields!("monsterShopBuyMamo", "BuyMamo", I32, false),
            fields!("monsterShopSellMamo", "SellMamo", I32, false),
        ],
    ),
    (
        "PictureBook",
        &[
            fields!("pictureBookLiveSpot1", "LiveSpot1", I32, false),
            fields!("pictureBookLiveSpot2", "LiveSpot2", I32, false),
            fields!("pictureBookSpecialDrop", "SpecialDrop", I32, false),
        ],
    ),
    ("Rupee", &[fields!("rupeeRupeeValue", "RupeeValue", I32, false)]),
    (
        "SeriesArmor",
        &[
            fields!("seriesArmorEnableCompBonus", "EnableCompBonus", Bool, false),
            fields!("seriesArmorSeriesType", "SeriesType", Str, false),
        ],
    ),
    (
        "System",
        &[
            fields!("systemIsGetItemSelf", "IsGetItemSelf", Bool, false),
            fields!("systemSameGroupActorName", "SameGroupActorName", Str, false),
        ],
    ),
    (
        "Traveler",
        &[
            fields!("travelerAppearGameDataName", "AppearGameDataName", Str, true),
            fields!("travelerDeleteGameDataName", "DeleteGameDataName", Str, true),
            fields!("travelerRideHorseName", "RideHorseName", Str, true),
            fields!("travelerRoutePoint0Name", "RoutePoint0Name", Str, true),
            fields!("travelerRoutePoint1Name", "RoutePoint1Name", Str, true),
            fields!("travelerRoutePoint2Name", "RoutePoint2Name", Str, true),
            fields!("travelerRoutePoint3Name", "RoutePoint3Name", Str, true),
            fields!("travelerRoutePoint4Name", "RoutePoint4Name", Str, true),
            fields!("travelerRoutePoint5Name", "RoutePoint5Name", Str, true),
            fields!("travelerRoutePoint6Name", "RoutePoint6Name", Str, true),
            fields!("travelerRoutePoint7Name", "RoutePoint7Name", Str, true),
            fields!("travelerRoutePoint8Name", "RoutePoint8Name", Str, true),
            fields!("travelerRoutePoint9Name", "RoutePoint9Name", Str, true),
            fields!("travelerRoutePoint10Name", "RoutePoint10Name", Str, true),
            fields!("travelerRoutePoint11Name", "RoutePoint11Name", Str, true),
            fields!("travelerRoutePoint12Name", "RoutePoint12Name", Str, true),
            fields!("travelerRoutePoint13Name", "RoutePoint13Name", Str, true),
            fields!("travelerRoutePoint14Name", "RoutePoint14Name", Str, true),
            fields!("travelerRoutePoint15Name", "RoutePoint15Name", Str, true),
            fields!("travelerRoutePoint16Name", "RoutePoint16Name", Str, true),
            fields!("travelerRoutePoint17Name", "RoutePoint17Name", Str, true),
            fields!("travelerRoutePoint18Name", "RoutePoint18Name", Str, true),
            fields!("travelerRoutePoint19Name", "RoutePoint19Name", Str, true),
            fields!("travelerRoutePoint20Name", "RoutePoint20Name", Str, true),
            fields!("travelerRoutePoint21Name", "RoutePoint21Name", Str, true),
            fields!("travelerRoutePoint22Name", "RoutePoint22Name", Str, true),
            fields!("travelerRoutePoint23Name", "RoutePoint23Name", Str, true),
            fields!("travelerRoutePoint24Name", "RoutePoint24Name", Str, true),
            fields!("travelerRoutePoint25Name", "RoutePoint25Name", Str, true),
            fields!("travelerRoutePoint26Name", "RoutePoint26Name", Str, true),
            fields!("travelerRoutePoint27Name", "RoutePoint27Name", Str, true),
            fields!("travelerRouteType", "RouteType", Str, true),
        ],
    ),
    (
        "WeaponCommon",
        &[
            fields!("weaponCommonGuardPower", "GuardPower", I32, false),
            fields!("weaponCommonPoweredSharpAddAtkMax", "PoweredSharpAddAtkMax", I32, false),
            fields!("weaponCommonPoweredSharpAddAtkMin", "PoweredSharpAddAtkMin", I32, false),
            fields!("weaponCommonPoweredSharpAddLifeMax", "PoweredSharpAddLifeMax", I32, false),
            fields!("weaponCommonPoweredSharpAddLifeMin", "PoweredSharpAddLifeMin", I32, false),
            fields!("weaponCommonPoweredSharpAddRapidFireMax", "PoweredSharpAddRapidFireMax", F32, false),
            fields!("weaponCommonPoweredSharpAddRapidFireMin", "PoweredSharpAddRapidFireMin", F32, false),
            fields!("weaponCommonPoweredSharpAddSpreadFire", "PoweredSharpAddSpreadFire", Bool, false),
            fields!("weaponCommonPoweredSharpAddSurfMaster", "PoweredSharpAddSurfMaster", Bool, false),
            fields!("weaponCommonPoweredSharpAddThrowMax", "PoweredSharpAddThrowMax", F32, false),
            fields!("weaponCommonPoweredSharpAddThrowMin", "PoweredSharpAddThrowMin", F32, false),
            fields!("weaponCommonPoweredSharpAddZoomRapid", "PoweredSharpAddZoomRapid", Bool, false),
            fields!("weaponCommonPoweredSharpWeaponAddGuardMax", "PoweredSharpWeaponAddGuardMax", I32, false),
            fields!("weaponCommonPoweredSharpWeaponAddGuardMin", "PoweredSharpWeaponAddGuardMin", I32, false),
            fields!("weaponCommonRank", "Rank", I32, false),
            fields!("weaponCommonSharpWeaponAddAtkMax", "SharpWeaponAddAtkMax", I32, false),
            fields!("weaponCommonSharpWeaponAddAtkMin", "SharpWeaponAddAtkMin", I32, false),
            fields!("weaponCommonSharpWeaponAddCrit", "SharpWeaponAddCrit", Bool, false),
            fields!("weaponCommonSharpWeaponAddGuardMax", "SharpWeaponAddGuardMax", I32, false),
            fields!("weaponCommonSharpWeaponAddGuardMin", "SharpWeaponAddGuardMin", I32, false),
            fields!("weaponCommonSharpWeaponAddLifeMax", "SharpWeaponAddLifeMax", I32, false),
            fields!("weaponCommonSharpWeaponAddLifeMin", "SharpWeaponAddLifeMin", I32, false),
            fields!("weaponCommonSharpWeaponPer", "SharpWeaponPer", F32, false),
            fields!("weaponCommonStickDamage", "StickDamage", I32, false),
        ],
    ),
];

const LIFECONDITION_ARRAY_KEYS: &[Field] = &[
    fields!("invalidTimes", "InvalidTimes", Str, false),
    fields!("invalidWeathers", "InvalidWeathers", Str, false),
];

const LIFECONDITION_KEYS: &[(&str, &[Field])] = &[
    ("DisplayDistance", &[fields!("traverseDist", "Item", F32, true)]),
    ("YLimitAlgorithm", &[fields!("yLimitAlgorithm", "Item", Str, false)]),
];

const MODELLIST_KEYS: &[(&str, &[Field])] = &[
    ("Attention", &[fields!("cursorOffsetY", "CursorOffsetY", F32, true)]),
    (
        "ControllerInfo",
        &[
            fields!("variationMatAnim", "VariationMatAnim", Str, true),
            fields!("variationMatAnimFrame", "VariationMatAnimFrame", I32, true),
        ],
    ),
];

const RECIPE_KEYS: &[(&str, &[Field])] = &[(
    "Normal0",
    &[
        fields!("normal0ItemName01", "ItemName01", Str, false),
        fields!("normal0ItemName02", "ItemName02", Str, false),
        fields!("normal0ItemName03", "ItemName03", Str, false),
        fields!("normal0ItemNum01", "ItemNum01", I32, false),
        fields!("normal0ItemNum02", "ItemNum02", I32, false),
        fields!("normal0ItemNum03", "ItemNum03", I32, false),
        fields!("normal0StuffNum", "ColumnNum", I32, false),
    ],
)];

fn f32_isclose(a: f32, b: f32) -> bool {
    (a - b).abs() <= 1e-5 * a.max(b.abs()).max(1.0)
}

/// Fetch param value from an object by name; returns converted value.
fn param_value(obj: &ParameterObject, param: &str) -> Option<Byml> {
    let p = obj.get(param)?;
    Some(match p {
        Parameter::Bool(v) => Byml::Bool(*v),
        Parameter::I32(v) => Byml::I32(*v),
        Parameter::F32(v) => Byml::Float(*v),
        _ => {
            if let Ok(s) = p.as_str() {
                Byml::String(s.to_string().into())
            } else {
                return None;
            }
        }
    })
}

fn field_is_null(param: &Parameter, ftype: FieldType) -> bool {
    match ftype {
        FieldType::I32 => param.as_i32().ok().map(|v| v == 0).unwrap_or(false),
        FieldType::F32 => param.as_f32().ok().map(|v| f32_isclose(v, 0.0)).unwrap_or(false),
        FieldType::Str => param.as_str().ok().map(|v| v.is_empty()).unwrap_or(false),
        FieldType::Bool => false,
    }
}

/// Recursive retrieval mirroring the Python `_deepretrieve_info`: keys may be
/// a map of name -> sub-rules, where the name refers to a list or an object
/// inside `el`.
fn deep_retrieve(
    data: &mut HashMap<String, Byml>,
    el: &ParameterIO,
    keys: &[(&str, &[Field])],
) {
    for (name, fields) in keys {
        let obj = el
            .param_root
            .objects
            .get(*name)
            .or_else(|| {
                el.param_root
                    .lists
                    .get(*name)
                    .and_then(|l| l.objects.get("Item"))
            });
        if let Some(obj) = obj {
            deep_retrieve_fields(data, obj, fields);
        }
    }
}

fn deep_retrieve_fields(
    data: &mut HashMap<String, Byml>,
    obj: &ParameterObject,
    fields: &[Field],
) {
    for field in fields {
        let Some(param) = obj.get(field.param) else {
            continue;
        };
        if field.check_null && field_is_null(param, field.ftype) {
            continue;
        }
        if let Some(value) = param_value(obj, field.param) {
            data.insert(field.info.to_string(), value);
        }
    }
}

pub fn get_actorlink_entries(data: &ParameterIO) -> HashMap<String, Byml> {
    let mut d = HashMap::new();
    if let Some(lt) = data.param_root.objects.get("LinkTarget") {
        for (info, link) in ACTORLINK_KEYS {
            let Some(param) = lt.get(*link) else {
                continue;
            };
            if let Ok(s) = param.as_str() {
                if !s.is_empty() && s != "Dummy" {
                    d.insert(info.to_string(), Byml::String(s.to_string().into()));
                }
            } else if let Ok(f) = param.as_f32() {
                if (f - 1.0).abs() > f32::EPSILON {
                    d.insert(info.to_string(), Byml::Float(f));
                }
            }
        }
    }
    d
}

pub fn get_actorlink_tags(data: &ParameterIO) -> Option<Map> {
    let tags = data.param_root.objects.get("Tags")?;
    let mut m = Map::default();
    for (_, param) in tags.iter() {
        let tag = param.as_str().ok()?;
        let taghash = crate::flag::crc32_str(tag) as u32;
        let key = format!("tag{taghash:08x}");
        if taghash > i32::MAX as u32 {
            m.insert(key.into(), Byml::U32(taghash));
        } else {
            m.insert(key.into(), Byml::I32(taghash as i32));
        }
    }
    if m.is_empty() {
        None
    } else {
        Some(m)
    }
}

pub fn get_chemical_entries(data: &ParameterIO) -> HashMap<String, Byml> {
    let mut d = HashMap::new();
    let chemical_root = match data.param_root.lists.get("chemical_root") {
        Some(l) => l,
        None => return d,
    };
    let chemical_body = match chemical_root.lists.get("chemical_body") {
        Some(l) => l,
        None => return d,
    };
    if let Some(rc) = chemical_body.objects.get("rigid_c_00") {
        if let Some(attribute) = rc.get("attribute") {
            if attribute.as_i32().ok() == Some(650) {
                let mut chemical = Map::default();
                chemical.insert("Capaciter".into(), Byml::I32(1));
                d.entry("Chemical".to_string())
                    .or_insert_with(|| Byml::Map(chemical));
            }
        }
    }
    if let Some(s0) = chemical_body.objects.get("shape_00") {
        if let Some(name) = s0.get("name") {
            if name.as_str().ok() == Some("WeaponFire") {
                let mut chemical = Map::default();
                chemical.insert("Burnable".into(), Byml::I32(1));
                d.entry("Chemical".to_string())
                    .or_insert_with(|| Byml::Map(chemical));
            }
        }
    }
    d
}

pub fn get_droptable_entries(data: &ParameterIO) -> HashMap<String, Byml> {
    let mut d: HashMap<String, Byml> = HashMap::new();
    for table in DROPTABLE_TABLES {
        if let Some(obj) = data.param_root.objects.get(*table) {
            let mut arr: Vec<Byml> = Vec::new();
            for key in DROPTABLE_ARRAY_KEYS {
                if let Some(p) = obj.get(*key) {
                    if let Ok(s) = p.as_str() {
                        arr.push(Byml::String(s.to_string().into()));
                    }
                }
            }
            let mut drops = match d.remove("drops") {
                Some(Byml::Map(m)) => m,
                _ => Map::default(),
            };
            drops.insert(table.to_string().into(), Byml::Array(arr));
            d.insert("drops".into(), Byml::Map(drops));
        }
    }
    d
}

pub fn get_gparamlist_entries(data: &ParameterIO) -> HashMap<String, Byml> {
    let mut d = HashMap::new();
    deep_retrieve(&mut d, data, GPARAMLIST_KEYS);
    d
}

pub fn get_lifecondition_entries(data: &ParameterIO) -> HashMap<String, Byml> {
    let mut d = HashMap::new();
    deep_retrieve(&mut d, data, LIFECONDITION_KEYS);
    for field in LIFECONDITION_ARRAY_KEYS {
        if let Some(obj) = data.param_root.objects.get(field.param) {
            let mut arr: Vec<Byml> = Vec::new();
            let mut idx = 1;
            loop {
                let keyname = format!("Item{idx:03}");
                let Some(param) = obj.get(keyname.as_str()) else {
                    break;
                };
                if field.check_null && field_is_null(param, field.ftype) {
                    // skip null values (Python: continue)
                } else if let Ok(s) = param.as_str() {
                    arr.push(Byml::String(s.to_string().into()));
                }
                idx += 1;
            }
            d.insert(field.info.to_string(), Byml::Array(arr));
        }
    }
    d
}

pub fn get_modellist_entries(data: &ParameterIO) -> HashMap<String, Byml> {
    let mut d = HashMap::new();
    deep_retrieve(&mut d, data, MODELLIST_KEYS);
    if let Some(att) = data.param_root.objects.get("Attention") {
        if let Some(look_at) = att.get("LookAtOffset") {
            if let Ok(v) = look_at.as_vec3() {
                if !f32_isclose(v.y, 0.0) {
                    d.insert("lookAtOffsetY".into(), Byml::Float(v.y));
                }
            }
        }
    }
    if let Some(ci) = data.param_root.objects.get("ControllerInfo") {
        if let Some(add_color) = ci.get("AddColor") {
            if let Ok(clr) = add_color.as_color() {
                if !f32_isclose(clr.r + clr.g + clr.b + clr.a, 0.0) {
                    d.insert("addColorR".into(), Byml::Float(clr.r));
                    d.insert("addColorG".into(), Byml::Float(clr.g));
                    d.insert("addColorB".into(), Byml::Float(clr.b));
                    d.insert("addColorA".into(), Byml::Float(clr.a));
                }
            }
        }
        if let Some(base_scale) = ci.get("BaseScale") {
            if let Ok(bs) = base_scale.as_vec3() {
                if bs.x != 1.0 || bs.y != 1.0 || bs.z != 1.0 {
                    d.insert("baseScaleX".into(), Byml::Float(bs.x));
                    d.insert("baseScaleY".into(), Byml::Float(bs.y));
                    d.insert("baseScaleZ".into(), Byml::Float(bs.z));
                }
            }
        }
        let mut fmcc = (0.0f32, 0.0f32, 0.0f32);
        let mut fmch = 0.0f32;
        let mut fmcr = 0.0f32;
        if let Some(p) = ci.get("FarModelCullingCenter") {
            if let Ok(v) = p.as_vec3() {
                fmcc = (v.x, v.y, v.z);
            }
        }
        if let Some(p) = ci.get("FarModelCullingHeight") {
            fmch = p.as_f32().unwrap_or(0.0);
        }
        if let Some(p) = ci.get("FarModelCullingRadius") {
            fmcr = p.as_f32().unwrap_or(0.0);
        }
        if !f32_isclose(fmcc.0 + fmcc.1 + fmcc.2 + fmch + fmcr, 0.0) {
            let center = {
                let mut m = Map::default();
                m.insert("X".into(), Byml::Float(fmcc.0));
                m.insert("Y".into(), Byml::Float(fmcc.1));
                m.insert("Z".into(), Byml::Float(fmcc.2));
                Byml::Map(m)
            };
            let mut far = Map::default();
            far.insert("center".into(), center);
            far.insert("height".into(), Byml::Float(fmch));
            far.insert("radius".into(), Byml::Float(fmcr));
            d.insert("farModelCulling".into(), Byml::Map(far));
        }
    }
    if let Some(md) = data.param_root.lists.get("ModelData") {
        if let Some(md0) = md.lists.get("ModelData_0") {
            if let Some(base) = md0.objects.get("Base") {
                if let Some(folder) = base.get("Folder") {
                    if let Ok(s) = folder.as_str() {
                        d.insert("bfres".into(), Byml::String(s.to_string().into()));
                    }
                }
            }
            if let Some(u) = md0.lists.get("Unit") {
                if let Some(unit0) = u.objects.get("Unit_0") {
                    if let Some(unit_name) = unit0.get("UnitName") {
                        if let Ok(s) = unit_name.as_str() {
                            d.insert("mainModel".into(), Byml::String(s.to_string().into()));
                        }
                    }
                }
            }
        }
    }
    d
}

pub fn get_physics_entries(data: &ParameterIO) -> HashMap<String, Byml> {
    let mut d = HashMap::new();
    let ps = match data.param_root.lists.get("ParamSet") {
        Some(l) => l,
        None => return d,
    };
    let rbs = match ps.lists.get("RigidBodySet") {
        Some(l) => l,
        None => return d,
    };
    let rbs0 = match rbs.lists.get("RigidBodySet_0") {
        Some(l) => l,
        None => return d,
    };
    let rb0 = match rbs0.lists.get("RigidBody_0") {
        Some(l) => l,
        None => return d,
    };
    if let Some(numbers) = rb0.objects.get(948250248u32) {
        if let Some(com) = numbers.get("center_of_mass") {
            if let Ok(v) = com.as_vec3() {
                d.insert("rigidBodyCenterY".into(), Byml::Float(v.y));
            }
        }
    }
    d
}

pub fn get_recipe_entries(data: &ParameterIO) -> HashMap<String, Byml> {
    let mut d = HashMap::new();
    deep_retrieve(&mut d, data, RECIPE_KEYS);
    d
}

/// Generate a fresh ActorInfo entry for the given pack, mutating a copy of the
/// old entry (port of Python `generate_actor_info`).
pub fn generate_actor_info(
    pack: &ActorPack,
    has_far: bool,
    old_info: &Map,
    old_actor: bool,
    keep_extra: bool,
) -> anyhow::Result<Map> {
    let mut entry: Map = old_info.clone();
    entry.insert("name".into(), Byml::String(pack.get_name().into()));
    entry.insert("isHasFar".into(), Byml::Bool(has_far));

    let profile = pack.get_link("ProfileUser");

    if !old_actor {
        if pack.get_link("SlinkUser") != "Dummy" {
            entry.insert("bugMask".into(), Byml::I32(2));
        }
        if let Some(Byml::I32(sort_key)) = entry.get("sortKey") {
            if *sort_key > 0 {
                entry.insert("sortKey".into(), Byml::I32(sort_key + 1));
            }
        }
    }

    let actorlink = pack.get_actorlink();
    for (key, value) in get_actorlink_entries(&actorlink) {
        if data::keys_for_profile(&profile).iter().any(|k| k == &key) {
            entry.insert(key.into(), value);
        }
    }
    if let Some(tags) = get_actorlink_tags(&actorlink) {
        entry.insert("tags".into(), Byml::Map(tags));
    }

    for link in [
        "ChemicalUser",
        "DropTableUser",
        "GParamUser",
        "LifeConditionUser",
        "ModelUser",
        "PhysicsUser",
        "RecipeUser",
    ] {
        let yaml = pack.get_link_data(link);
        if !yaml.is_empty() {
            if let Ok(pio) = ParameterIO::from_text(yaml) {
                let values = match link {
                    "ChemicalUser" => get_chemical_entries(&pio),
                    "DropTableUser" => get_droptable_entries(&pio),
                    "GParamUser" => get_gparamlist_entries(&pio),
                    "LifeConditionUser" => get_lifecondition_entries(&pio),
                    "ModelUser" => get_modellist_entries(&pio),
                    "PhysicsUser" => get_physics_entries(&pio),
                    "RecipeUser" => get_recipe_entries(&pio),
                    _ => HashMap::new(),
                };
                for (k, v) in values {
                    entry.insert(k.into(), v);
                }
            }
        }
    }

    let keys = data::keys_for_profile(&profile);
    if !keep_extra {
        let to_remove: Vec<String> = entry
            .keys()
            .map(|k| k.to_string())
            .filter(|k| !keys.iter().any(|kk| kk == k))
            .collect();
        for k in to_remove {
            entry.remove(k.as_str());
        }
    }

    Ok(entry)
}

/// Parse a user-typed override string into a BYML value (bool / int /
/// float / string). Used by the ActorInfo editor page's override column.
pub fn parse_api_value(s: &str) -> Byml {
    let t = s.trim();
    if t.eq_ignore_ascii_case("true") {
        return Byml::Bool(true);
    }
    if t.eq_ignore_ascii_case("false") {
        return Byml::Bool(false);
    }
    if let Ok(i) = t.parse::<i32>() {
        return Byml::I32(i);
    }
    if let Ok(f) = t.parse::<f64>() {
        if f.is_finite() {
            return Byml::Float(f as f32);
        }
    }
    Byml::String(t.to_string().into())
}
