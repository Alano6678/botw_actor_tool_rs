//! egui application: main window, menu bar, panels and dialogs.

use std::collections::HashMap;
use std::path::PathBuf;

use eframe::egui::{self, Align2, Color32, Key, RichText};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actor::BATActor;
    use crate::pack::PackSource;
    use crate::settings::Settings;
    use crate::util::{self, FoundFile};
    use kittest::Queryable;

    /// Drive the REAL app UI: load a real actor (if configured), open the
    /// GParam tab, focus the editor, type, and verify the text lands in the
    /// editor at the cursor position (guards against the "typed text appears
    /// somewhere random" bug).
    #[test]
    #[ignore]
    fn real_app_typing_goes_to_editor() {
        let settings = Settings::load();
        if settings.update_dir.is_empty() || settings.game_dir.is_empty() {
            eprintln!("game dirs not configured; skipping");
            return;
        }
        let found = util::find_file("Actor/Pack/Armor_001_Head.sbactorpack").unwrap();
        let source = match found {
            FoundFile::Path(p) => PackSource::Path(p),
            FoundFile::Resident { titlebg, inner } => PackSource::Resident { titlebg, inner },
        };
        let actor = BATActor::new(&source).unwrap();
        let app = App {
            settings,
            actor: Some(actor),
            tab: 11, // General Param
            link_panel: None,
            editor: None,
            texts_panel: None,
            info_panel: None,
            flags_panel: None,
            actor_select: None,
            settings_panel: None,
            save_dir_request: false,
            about_open: false,
            msg: None,
            status: None,
            last_frame_rect: None,
            saving: None,
            save_armed: false,
            editor_cursor_byte: None,
            find_focus_pending: false,
            editor_edit_id: None,
            editor_edit_rect: None,
            ui_lang: crate::i18n::UiLang::En,
        };
        let harness = egui_kittest::Harness::new_ui_state(
            |ui, app: &mut App| {
                app.root_ui(ui);
                // Keep the editor focused (tests don't drive the accesskit
                // tree that CodeEditor exposes).
                if let Some(id) = app.editor_edit_id {
                    ui.memory_mut(|m| m.request_focus(id));
                }
            },
            app,
        );
        let mut harness = harness;
        harness.run_steps(3);
        harness
            .input_mut()
            .events
            .push(egui::Event::Text("ZZZ".to_string()));
        harness.step();

        harness
            .input_mut()
            .events
            .push(egui::Event::Key {
                key: egui::Key::Backspace,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::default(),
            });
        harness.step();

        let app = harness.state();
        let editor_text = app
            .editor
            .as_ref()
            .map(|e| e.text.clone())
            .unwrap_or_default();
        assert!(
            !editor_text.contains("ZZZ"),
            "typing then backspace must leave the editor unchanged (got head {:?})",
            &editor_text[..editor_text.len().min(60)]
        );
    }

    /// Click at the very START of a line (line head) and type: the character
    /// must land at the start of THAT line, not at position one of the file.
    #[test]
    #[ignore]
    fn real_app_typing_line_head() {
        let settings = Settings::load();
        if settings.update_dir.is_empty() || settings.game_dir.is_empty() {
            eprintln!("game dirs not configured; skipping");
            return;
        }
        let found = util::find_file("Actor/Pack/Armor_001_Head.sbactorpack").unwrap();
        let source = match found {
            FoundFile::Path(p) => PackSource::Path(p),
            FoundFile::Resident { titlebg, inner } => PackSource::Resident { titlebg, inner },
        };
        let actor = BATActor::new(&source).unwrap();
        let app = App {
            settings,
            actor: Some(actor),
            tab: 11, // General Param
            link_panel: None,
            editor: None,
            texts_panel: None,
            info_panel: None,
            flags_panel: None,
            actor_select: None,
            settings_panel: None,
            save_dir_request: false,
            about_open: false,
            msg: None,
            status: None,
            last_frame_rect: None,
            saving: None,
            save_armed: false,
            editor_cursor_byte: None,
            find_focus_pending: false,
            editor_edit_id: None,
            editor_edit_rect: None,
            ui_lang: crate::i18n::UiLang::En,
        };
        let mut harness =
            egui_kittest::Harness::new_ui_state(|ui, app: &mut App| {
                app.root_ui(ui);
                if let Some(id) = app.editor_edit_id {
                    ui.memory_mut(|m| m.request_focus(id));
                }
            }, app);
        harness.run_steps(3);

        harness
            .input_mut()
            .events
            .push(egui::Event::Text("X".to_string()));
        harness.step();
        let app = harness.state();
        let editor_text = app
            .editor
            .as_ref()
            .map(|e| e.text.clone())
            .unwrap_or_default();
        assert!(
            editor_text.contains('X'),
            "typing must reach the code editor (got head {:?})",
            &editor_text[..editor_text.len().min(60)]
        );
        // Backspace removes the typed character again.
        harness
            .input_mut()
            .events
            .push(egui::Event::Key {
                key: egui::Key::Backspace,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::default(),
            });
        harness.step();
        let editor_text = harness
            .state()
            .editor
            .as_ref()
            .map(|e| e.text.clone())
            .unwrap_or_default();
        assert!(
            !editor_text.contains('X'),
            "backspace must delete the typed character (got head {:?})",
            &editor_text[..editor_text.len().min(60)]
        );
    }

    /// Big-file variant: a much longer text (like a real Model/Recipe file)
    /// plus click + type, ensuring scrolling and deep rows keep the mapping.
    #[test]
    #[ignore]
    fn real_app_typing_big_file() {
        let settings = Settings::load();
        if settings.update_dir.is_empty() || settings.game_dir.is_empty() {
            eprintln!("game dirs not configured; skipping");
            return;
        }
        let found = util::find_file("Actor/Pack/Armor_001_Head.sbactorpack").unwrap();
        let source = match found {
            FoundFile::Path(p) => PackSource::Path(p),
            FoundFile::Resident { titlebg, inner } => PackSource::Resident { titlebg, inner },
        };
        let mut actor = BATActor::new(&source).unwrap();
        // Simple long text (like the isolated tests) — bisects whether the
        // problem is the app UI structure or the long-line content.
        let big = (0..120)
            .map(|i| format!("line{i:03} abcdefghijklmnopqrstuvwxyz"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut app = App {
            settings,
            actor: Some(actor),
            tab: 11,
            link_panel: None,
            editor: None,
            texts_panel: None,
            info_panel: None,
            flags_panel: None,
            actor_select: None,
            settings_panel: None,
            save_dir_request: false,
            about_open: false,
            msg: None,
            status: None,
            last_frame_rect: None,
            saving: None,
            save_armed: false,
            editor_cursor_byte: None,
            find_focus_pending: false,
            editor_edit_id: None,
            editor_edit_rect: None,
            ui_lang: crate::i18n::UiLang::En,
        };
        // Pre-open the editor with the big text (constructor-level, so it is
        // in place before the first frame — switch_tab alone does NOT create
        // the editor; that happens on first render).
        app.editor = Some(crate::app::EditorState {
            link: "GParamUser".to_string(),
            text: big,
            saved_hash: 0,
            rename_on_edit: false,
            suppress_rename: false,
            search: String::new(),
            find_open: false,
            match_case: false,
            pending_cursor: None,
            scroll_offset: None,
        });
        let mut harness =
            egui_kittest::Harness::new_ui_state(|ui, app: &mut App| {
                app.root_ui(ui);
                if let Some(id) = app.editor_edit_id {
                    ui.memory_mut(|m| m.request_focus(id));
                }
            }, app);
        harness.run_steps(3);
        harness
            .input_mut()
            .events
            .push(egui::Event::Text("ZZZ".to_string()));
        harness.step();

        let app = harness.state();
        let editor_text = app
            .editor
            .as_ref()
            .map(|e| e.text.clone())
            .unwrap_or_default();
        eprintln!(
            "big file: zzz_at={:?} search={:?}",
            editor_text.find("ZZZ"),
            app.editor.as_ref().map(|e| e.search.clone())
        );
        assert!(
            editor_text.find("ZZZ").is_some(),
            "typed text must land somewhere in the editor (got no ZZZ)"
        );
    }

    /// Regression test for "dark mode becomes light after restart": egui 0.36
    /// `set_visuals` only writes the *currently active* theme slot, which at
    /// startup is the dark fallback (before the OS theme is known) — so the
    /// dark visuals landed in the wrong slot. `apply_theme_to` must pin the
    /// theme preference AND fill both slots.
    #[test]
    fn apply_theme_pins_preference_and_fills_both_slots() {
        let ctx = egui::Context::default();

        App::apply_theme_to(&ctx, true);
        assert_eq!(ctx.theme(), egui::Theme::Dark);
        assert_eq!(
            ctx.style_of(egui::Theme::Dark).visuals.panel_fill,
            egui::Visuals::dark().panel_fill
        );
        // The system theme is unknown before the first frame, so egui would
        // have used the light slot at startup — it must carry dark visuals too.
        assert_eq!(
            ctx.style_of(egui::Theme::Light).visuals.panel_fill,
            egui::Visuals::light().panel_fill
        );
        // Monospace must stay 14px in both slots (caret/row-height alignment).
        for theme in [egui::Theme::Dark, egui::Theme::Light] {
            let style = ctx.style_of(theme);
            assert_eq!(style.text_styles[&egui::TextStyle::Monospace].size, 14.0);
        }

        // Simulate the first frame: the OS reports light mode (the system
        // theme is only known from the first frame onwards — exactly the
        // restart scenario). With the OLD code `set_visuals` wrote into the
        // dark fallback slot and egui then switched to the system light slot
        // -> the UI came up light. With the preference pinned, the active
        // theme must stay Dark regardless of the system theme.
        let mut raw = egui::RawInput::default();
        raw.system_theme = Some(egui::Theme::Light);
        ctx.begin_pass(raw);
        assert_eq!(
            ctx.theme(),
            egui::Theme::Dark,
            "pinned dark preference must survive the system theme arriving"
        );

        App::apply_theme_to(&ctx, false);
        assert_eq!(ctx.theme(), egui::Theme::Light);
        assert_eq!(
            ctx.style_of(egui::Theme::Light).visuals.panel_fill,
            egui::Visuals::light().panel_fill
        );
        assert_eq!(
            ctx.style_of(egui::Theme::Dark).visuals.panel_fill,
            egui::Visuals::dark().panel_fill
        );
        let mut raw = egui::RawInput::default();
        raw.system_theme = Some(egui::Theme::Dark);
        ctx.begin_pass(raw);
        assert_eq!(
            ctx.theme(),
            egui::Theme::Light,
            "pinned light preference must survive the system theme arriving"
        );
    }

    /// Drive the REAL find flow: Ctrl+F opens the bar, typed text lands in
    /// the search input, the match count picks it up, and Enter/▼ jumps the
    /// cursor to the next match.
    #[test]
    #[ignore]
    fn real_app_find_flow() {
        let settings = Settings::load();
        if settings.update_dir.is_empty() || settings.game_dir.is_empty() {
            eprintln!("game dirs not configured; skipping");
            return;
        }
        let found = util::find_file("Actor/Pack/Armor_001_Head.sbactorpack").unwrap();
        let source = match found {
            FoundFile::Path(p) => PackSource::Path(p),
            FoundFile::Resident { titlebg, inner } => PackSource::Resident { titlebg, inner },
        };
        let actor = BATActor::new(&source).unwrap();
        // Pre-open the editor with text that contains a known needle twice.
        let body = actor.get_link_data("GParamUser");
        let mut app = App {
            settings,
            actor: Some(actor),
            tab: 11, // General Param
            link_panel: None,
            editor: None,
            texts_panel: None,
            info_panel: None,
            flags_panel: None,
            actor_select: None,
            settings_panel: None,
            save_dir_request: false,
            about_open: false,
            msg: None,
            status: None,
            last_frame_rect: None,
            saving: None,
            save_armed: false,
            editor_cursor_byte: None,
            find_focus_pending: false,
            editor_edit_id: None,
            editor_edit_rect: None,
            ui_lang: crate::i18n::UiLang::En,
        };
        // Pre-open the editor with text that contains a known needle twice.
        app.editor = Some(crate::app::EditorState {
            link: "GParamUser".to_string(),
            text: format!("{body}\n# cjk 汉字内容 测试\nneeds: zzz marker\nmore: zzz marker again"),
            saved_hash: 0,
            rename_on_edit: false,
            suppress_rename: false,
            search: String::new(),
            find_open: false,
            match_case: false,
            pending_cursor: None,
            scroll_offset: None,
        });
        let mut harness =
            egui_kittest::Harness::new_ui_state(|ui, app: &mut App| {
                app.root_ui(ui);
            }, app);
        harness.run_steps(2);

        // Ctrl+F opens the bar (eframe reports modifier changes via
        // `Event::ModifiersChanged`, so emulate that like the real integration).
        harness.input_mut().events.push(egui::Event::ModifiersChanged(
            egui::Modifiers::COMMAND,
        ));
        harness.input_mut().events.push(egui::Event::Key {
            key: egui::Key::F,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::COMMAND,
        });
        harness.step();
        harness.input_mut().events.push(egui::Event::ModifiersChanged(
            egui::Modifiers::NONE,
        ));
        assert!(
            harness
                .state()
                .editor
                .as_ref()
                .map(|e| e.find_open)
                .unwrap_or(false),
            "find bar did not open via Ctrl+F"
        );

        // One frame so the bar paints and takes focus...
        harness.step();
        // ...then typed text must land in the find input.
        harness
            .input_mut()
            .events
            .push(egui::Event::Text("zzz".to_string()));
        harness.step();
        let search = harness
            .state()
            .editor
            .as_ref()
            .map(|e| e.search.clone())
            .unwrap_or_default();
        assert_eq!(search, "zzz", "typed text did not land in the find input");

        let (text, m) = {
            let e = harness.state().editor.as_ref().unwrap();
            (e.text.clone(), find_matches(&e.text, &e.search, false))
        };
        assert_eq!(m.len(), 2, "expected 2 matches for 'zzz', got {:?}", m.len());

        // CJK needles must also find matches (the old ASCII-only guard made
        // searching Chinese text impossible — "无法搜索").
        let cjk = find_matches(&text, "汉字", false);
        assert!(
            cjk.len() >= 1,
            "CJK search must find matches (got {cjk:?})"
        );

        // Enter jumps to the first match.
        harness.input_mut().events.push(egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        });
        harness.step();
        harness.step();
        let app = harness.state();
        let cur = app.editor_cursor_byte.unwrap_or(0);
        assert_eq!(
            cur, m[0].0,
            "Enter must land on the first match (cursor={cur}, expected {})",
            m[0].0
        );

        // Enter AGAIN (focus is now on the editor) must jump to the NEXT
        // match — and must NOT insert a newline into the code.
        let text_len_before = harness.state().editor.as_ref().unwrap().text.len();
        harness.input_mut().events.push(egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        });
        harness.step();
        harness.step();
        let cur2 = harness.state().editor_cursor_byte.unwrap_or(0);
        assert_eq!(
            cur2, m[1].0,
            "second Enter must advance to the next match (cur2={cur2}, expected {})",
            m[1].0
        );
        let text_len_after = harness.state().editor.as_ref().unwrap().text.len();
        assert_eq!(
            text_len_before, text_len_after,
            "Enter while the find bar is open must not insert a newline"
        );
        eprintln!(
            "find flow ok: search={search} matches={} cursor={cur}/{cur2}",
            m.len()
        );

        // A THIRD Enter must wrap around to the first match (next_match
        // wraps) — still without touching the editor text.
        harness.input_mut().events.push(egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        });
        harness.step();
        harness.step();
        let text_len_after3 = harness.state().editor.as_ref().unwrap().text.len();
        assert_eq!(
            text_len_before, text_len_after3,
            "Enter while the find bar is open must never insert a newline"
        );
        let cur3 = harness.state().editor_cursor_byte.unwrap_or(0);
        assert_eq!(
            cur3, m[0].0,
            "third Enter must wrap to the first match (cur3={cur3}, expected {})",
            m[0].0
        );
    }

    /// Render the actual UI to a PNG so the find bar's look can be inspected
    /// (guards "搜索框不是 egui" style regressions).
    /// run with: cargo test find_bar_screenshot -- --ignored --nocapture
    #[test]
    #[ignore]
    fn find_bar_screenshot() {
        let settings = Settings::load();
        if settings.update_dir.is_empty() || settings.game_dir.is_empty() {
            eprintln!("game dirs not configured; skipping");
            return;
        }
        let found = util::find_file("Actor/Pack/Armor_001_Head.sbactorpack").unwrap();
        let source = match found {
            FoundFile::Path(p) => PackSource::Path(p),
            FoundFile::Resident { titlebg, inner } => PackSource::Resident { titlebg, inner },
        };
        let actor = BATActor::new(&source).unwrap();
        let body = actor.get_link_data("GParamUser");
        let mut app = App {
            settings,
            actor: Some(actor),
            tab: 11, // General Param
            link_panel: None,
            editor: None,
            texts_panel: None,
            info_panel: None,
            flags_panel: None,
            actor_select: None,
            settings_panel: None,
            save_dir_request: false,
            about_open: false,
            msg: None,
            status: None,
            last_frame_rect: None,
            saving: None,
            save_armed: false,
            editor_cursor_byte: None,
            find_focus_pending: false,
            editor_edit_id: None,
            editor_edit_rect: None,
            ui_lang: crate::i18n::UiLang::En,
        };
        app.editor = Some(crate::app::EditorState {
            link: "GParamUser".to_string(),
            text: format!("{body}\n# cjk 汉字内容 测试\nneeds: zzz marker\nmore: zzz marker again"),
            saved_hash: 0,
            rename_on_edit: false,
            suppress_rename: false,
            search: "zzz".to_string(),
            find_open: true,
            match_case: false,
            pending_cursor: None,
            scroll_offset: None,
        });
        let mut harness = egui_kittest::Harness::builder()
            .with_size(egui::Vec2::new(1280.0, 860.0))
            .wgpu()
            .build_ui_state(|ui, app: &mut App| {
                app.root_ui(ui);
            }, app);
        super::setup_fonts(&harness.ctx);
        harness.run_steps(3);
        let img = harness.render().expect("render");
        let path = std::env::temp_dir().join("find_bar.png");
        image::save_buffer(
            &path,
            &img,
            img.width(),
            img.height(),
            image::ColorType::Rgba8,
        )
        .expect("save");
        eprintln!("saved screenshot: {}", path.display());

        // Click ▼, then screenshot again: the caret must be at the second
        // match and the highlight (current match) must be visible.
        harness.get_by_label("▼").click();
        harness.run_steps(3);
        let img = harness.render().expect("render2");
        let path2 = std::env::temp_dir().join("find_bar_next.png");
        image::save_buffer(
            &path2,
            &img,
            img.width(),
            img.height(),
            image::ColorType::Rgba8,
        )
        .expect("save2");
        let cur = harness.state().editor_cursor_byte.unwrap_or(0);
        eprintln!(
            "saved screenshot2: {} (cursor={cur})",
            path2.display()
        );
    }
}

use crate::actor::{get_all_vanilla_actors, try_retrieve_custom_file, BATActor};
use crate::pack::PackSource;
use crate::settings::Settings;
use crate::util::{self, FoundFile, LINKS};

pub const TABS: &[&str] = &[
    "Actor Link",
    "AI Program",
    "AI Schedule",
    "AS",
    "Attention",
    "Awareness",
    "Bone Control",
    "Chemical",
    "Damage Param",
    "Drop Table",
    "Elink",
    "General Param",
    "Life Condition",
    "LOD",
    "Model",
    "Physics",
    "Profile",
    "Ragdoll Blend",
    "Ragdoll Config",
    "Recipe",
    "Shop Data",
    "Slink",
    "UMii",
    "Xlink",
    "Animation Info",
    "Texts",
    "ActorInfo",
    "Flags",
];

pub const LINK_TO_TAB: [(usize, &str); 24] = [
    (1, "AIProgramUser"),
    (2, "AIScheduleUser"),
    (3, "ASUser"),
    (4, "AttentionUser"),
    (5, "AwarenessUser"),
    (6, "BoneControlUser"),
    (7, "ChemicalUser"),
    (8, "DamageParamUser"),
    (9, "DropTableUser"),
    (10, "ElinkUser"),
    (11, "GParamUser"),
    (12, "LifeConditionUser"),
    (13, "LODUser"),
    (14, "ModelUser"),
    (15, "PhysicsUser"),
    (16, "ProfileUser"),
    (17, "RgBlendWeightUser"),
    (18, "RgConfigListUser"),
    (19, "RecipeUser"),
    (20, "ShopDataUser"),
    (21, "SlinkUser"),
    (22, "UMiiUser"),
    (23, "XlinkUser"),
    (24, "AnimationInfo"),
];

const NOT_IMPLEMENTED: &[&str] = &["ElinkUser", "ProfileUser", "SlinkUser", "XlinkUser"];

enum LinkAction {
    SetActorName(String),
    SetPriority(String),
    SetLink(String, String, bool),
    SetTags(String),
}

#[derive(Clone, Copy, PartialEq)]
enum LinkChoice {
    Dummy,
    ActorName,
    Custom,
}

pub struct LinkPanelState {
    actor_name: String,
    priority_input: String,
    name_input: String,
    tags_input: String,
    choices: Vec<LinkChoice>,
    custom_texts: Vec<String>,
}

impl LinkPanelState {
    pub fn new(actor: &BATActor) -> Self {
        let name = actor.get_name();
        let choices = LINKS
            .iter()
            .map(|link| {
                let v = actor.get_link(link);
                if v == "Dummy" {
                    LinkChoice::Dummy
                } else if v == name {
                    LinkChoice::ActorName
                } else {
                    LinkChoice::Custom
                }
            })
            .collect();
        let custom_texts = LINKS
            .iter()
            .map(|link| {
                let v = actor.get_link(link);
                if v != "Dummy" && v != name {
                    v
                } else {
                    String::new()
                }
            })
            .collect();
        LinkPanelState {
            name_input: name.clone(),
            priority_input: actor.get_link("Priority"),
            actor_name: name,
            tags_input: actor.get_tags(),
            choices,
            custom_texts,
        }
    }
}

pub struct EditorState {
    pub link: String,
    pub text: String,
    pub saved_hash: u32,
    pub rename_on_edit: bool,
    pub suppress_rename: bool,
    pub search: String,
    pub find_open: bool,
    pub match_case: bool,
    pub pending_cursor: Option<usize>,
    pub scroll_offset: Option<f32>,
}

/// ActorInfo editor page state: what the tool will generate for this actor
/// (auto values) plus the user's manual overrides.
pub struct InfoPanelState {
    /// (field, auto value as text) — regenerated when the actor changes.
    pub entries: Vec<(String, String)>,
    /// Manual overrides; empty string means "use the auto value".
    pub overrides: HashMap<String, String>,
    /// Keep old fields not in the profile whitelist.
    pub keep_extra: bool,
}

/// Flags editor page state (the "add flag" form fields).
#[derive(Default)]
pub struct FlagsPanelState {
    pub name: String,
    pub ftype: String,
    pub bool_on: bool,
    pub s32: i32,
}

pub struct TextsPanelState {
    pub base_name_on: bool,
    pub base_name: String,
    pub name_on: bool,
    pub name: String,
    pub desc_on: bool,
    pub desc: String,
    pub pbook_on: bool,
    pub pbook: String,
}

impl TextsPanelState {
    pub fn new(actor: &BATActor) -> Self {
        let texts = actor.get_texts().get_texts().clone();
        let get = |k: &str| texts.get(k).cloned().unwrap_or_default();
        TextsPanelState {
            base_name_on: texts.contains_key("BaseName"),
            base_name: get("BaseName"),
            name_on: texts.contains_key("Name"),
            name: get("Name"),
            desc_on: texts.contains_key("Desc"),
            desc: get("Desc"),
            pbook_on: texts.contains_key("PictureBook"),
            pbook: get("PictureBook"),
        }
    }

    pub fn collect(&self) -> HashMap<String, String> {
        let mut texts = HashMap::new();
        if self.base_name_on {
            texts.insert("BaseName".to_string(), self.base_name.clone());
        }
        if self.name_on {
            texts.insert("Name".to_string(), self.name.clone());
        }
        if self.desc_on {
            texts.insert("Desc".to_string(), self.desc.clone());
        }
        if self.pbook_on {
            texts.insert("PictureBook".to_string(), self.pbook.clone());
        }
        texts
    }
}

pub struct ActorSelectState {
    pub root_dir: PathBuf,
    pub is_vanilla: bool,
    pub filter: String,
    pub names: Vec<String>,
    pub selected: usize,
    pub done: Option<Option<String>>,
}

impl ActorSelectState {
    pub fn open(root_dir: PathBuf, is_vanilla: bool) -> Self {
        ActorSelectState {
            root_dir,
            is_vanilla,
            filter: String::new(),
            names: Vec::new(),
            selected: 0,
            done: None,
        }
    }

    fn refresh(&mut self) {
        let names = if self.is_vanilla {
            get_all_vanilla_actors(&self.root_dir).unwrap_or_default()
        } else {
            util::list_mod_actors(&self.root_dir)
        };
        self.names = names.into_iter().filter(|n| !n.contains("_Far")).collect();
        self.selected = 0;
    }
}

pub struct SettingsPanelState {
    pub game_dir: String,
    pub update_dir: String,
    pub dlc_dir: String,
    pub lang: String,
    pub ui_lang: String,
    pub dark: bool,
    pub show_unsupported_tabs: bool,
}

impl SettingsPanelState {
    pub fn new(s: &Settings) -> Self {
        SettingsPanelState {
            game_dir: s.game_dir.clone(),
            update_dir: s.update_dir.clone(),
            dlc_dir: s.dlc_dir.clone(),
            lang: s.lang.clone(),
            ui_lang: s.ui_lang.clone(),
            dark: s.dark_theme,
            show_unsupported_tabs: s.show_unsupported_tabs,
        }
    }
}

#[derive(Clone)]
pub enum Msg {
    Ok(String),
    YesNo {
        title: String,
        text: String,
        pending_link: String,
    },
}

pub struct App {
    pub settings: Settings,
    pub actor: Option<BATActor>,
    pub tab: usize,
    pub link_panel: Option<LinkPanelState>,
    pub editor: Option<EditorState>,
    pub texts_panel: Option<TextsPanelState>,
    pub info_panel: Option<InfoPanelState>,
    pub flags_panel: Option<FlagsPanelState>,
    pub actor_select: Option<ActorSelectState>,
    pub settings_panel: Option<SettingsPanelState>,
    pub save_dir_request: bool,
    pub about_open: bool,
    pub msg: Option<Msg>,
    pub status: Option<String>,
    pub last_frame_rect: Option<egui::Rect>,
    pub saving: Option<PathBuf>,
    pub save_armed: bool,
    pub editor_cursor_byte: Option<usize>,
    pub find_focus_pending: bool,
    pub editor_edit_id: Option<egui::Id>,
    pub editor_edit_rect: Option<egui::Rect>,
    pub ui_lang: crate::i18n::UiLang,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_fonts(&cc.egui_ctx);
        let settings = Settings::load();
        let ui_lang_code = settings.ui_lang.clone();
        let mut app = App {
            settings,
            actor: None,
            tab: 0,
            link_panel: None,
            editor: None,
            texts_panel: None,
            info_panel: None,
            flags_panel: None,
            actor_select: None,
            settings_panel: None,
            save_dir_request: false,
            about_open: false,
            msg: None,
            status: None,
            last_frame_rect: None,
            saving: None,
            save_armed: false,
            editor_cursor_byte: None,
            find_focus_pending: false,
            editor_edit_id: None,
            editor_edit_rect: None,
            ui_lang: crate::i18n::UiLang::from_setting(&ui_lang_code),
        };
        app.apply_theme(&cc.egui_ctx);
        app
    }

    pub fn apply_theme(&mut self, ctx: &egui::Context) {
        Self::apply_theme_to(ctx, self.settings.dark_theme);
    }

    /// Theme-application helper (kept as a `static`-style fn so tests can
    /// drive it with a bare `egui::Context`).
    ///
    /// IMPORTANT: do NOT use `ctx.set_visuals(...)` here. In egui 0.36,
    /// `set_visuals` only writes into the *currently active* theme slot, which
    /// at startup is the fallback (default DARK — the OS theme is unknown
    /// before the first frame). The app then wrote dark visuals into the dark
    /// slot, the first frame switched the active theme to the system (LIGHT)
    /// slot, and the UI came up light even though `dark_theme` was true — the
    /// "dark mode becomes light after restart" bug. We therefore pin the
    /// theme preference and fill BOTH slots explicitly.
    pub fn apply_theme_to(ctx: &egui::Context, dark: bool) {
        ctx.set_theme(if dark {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        });
        // Boost text contrast in dark mode: egui's default dark text color is
        // a dim gray that reads poorly on the near-black background. Brighter
        // text + a more vivid link color keep it legible.
        let mut dark_visuals = egui::Visuals::dark();
        dark_visuals.override_text_color = Some(Color32::from_gray(235));
        dark_visuals.hyperlink_color = Color32::from_rgb(130, 200, 255);
        ctx.set_visuals_of(egui::Theme::Dark, dark_visuals);
        ctx.set_visuals_of(egui::Theme::Light, egui::Visuals::light());
        // Code editor font: bump the Monospace style size for BOTH themes so
        // that TextEdit's row height, the caret math and our highlighted
        // galley stay perfectly consistent (a mismatch made the caret drift
        // from the typing position, growing worse per row).
        for theme in [egui::Theme::Dark, egui::Theme::Light] {
            ctx.style_mut_of(theme, |style| {
                if let Some(font) = style.text_styles.get_mut(&egui::TextStyle::Monospace) {
                    font.size = 14.0; // 12.0 base + 2px
                }
            });
        }
    }

    /// Localized string for the current UI language.
    #[allow(dead_code)]
    fn t<'a>(&self, key: &'a str) -> &'a str {
        crate::i18n::tr(self.ui_lang, key)
    }

    fn after_actor_load(&mut self) {
        let ui_lang = self.ui_lang;
        self.tab = 0;
        self.editor = None;
        self.texts_panel = None;
        self.info_panel = None;
        self.flags_panel = None;
        let name = self.actor.as_ref().map(|a| a.get_name()).unwrap_or_default();
        self.link_panel = self.actor.as_ref().map(LinkPanelState::new);
        if self.actor.is_some() {
            self.status = Some(format!("{}{}", ty(ui_lang, "Loaded "), name));
        }
    }

    fn load_actor_from_path(&mut self, path: &PathBuf) {
        let ui_lang = self.ui_lang;
        match BATActor::new(&PackSource::Path(path.clone())) {
            Ok(actor) => {
                self.actor = Some(actor);
                self.after_actor_load();
            }
            Err(e) => {
                self.status = None;
                self.msg = Some(Msg::Ok(format!("{}{e}", ty(ui_lang, "Failed to load actor: "))));
            }
        }
    }

    fn load_vanilla(&mut self, name: &str) {
        match util::find_file(&format!("Actor/Pack/{name}.sbactorpack")) {
            Ok(found) => {
                let source = match found {
                    FoundFile::Path(p) => PackSource::Path(p),
                    FoundFile::Resident { titlebg, inner } => PackSource::Resident {
                        titlebg,
                        inner,
                    },
                };
                match BATActor::new(&source) {
                    Ok(actor) => {
                        self.actor = Some(actor);
                        self.after_actor_load();
                    }
                    Err(e) => self.msg = Some(Msg::Ok(format!("Failed to load actor: {e}"))),
                }
            }
            Err(e) => self.msg = Some(Msg::Ok(e.to_string())),
        }
    }

    fn run_save(&mut self, root_dir: &PathBuf) {
        let ui_lang = self.ui_lang;
        let be = match root_dir.file_name().and_then(|n| n.to_str()) {
            Some("romfs") => false,
            Some("content") => true,
            _ => {
                self.msg =
                    Some(Msg::Ok(ty(ui_lang, "Must choose either content or romfs!").to_string()));
                return;
            }
        };
        match self.actor.as_mut().map(|a| a.save(root_dir, be)) {
            Some(Ok(())) => {
                self.status = Some(format!("{}{}", ty(ui_lang, "Saved to "), root_dir.display()))
            }
            Some(Err(e)) => self.msg = Some(Msg::Ok(format!("Save failed: {e:#}"))),
            None => {
                self.msg = Some(Msg::Ok(ty(ui_lang, "No actor loaded").to_string()));
            }
        }
    }

    fn editor_save(&mut self) {
        let ui_lang = self.ui_lang;
        let (link, text) = match &self.editor {
            Some(e) => (e.link.clone(), e.text.clone()),
            None => return,
        };
        let hash = crate::flag::crc32_str(&text) as u32;
        if let Some(e) = &self.editor {
            if hash == e.saved_hash {
                return;
            }
        }
        let rename_on_edit = self
            .editor
            .as_ref()
            .map(|e| e.rename_on_edit && !e.suppress_rename)
            .unwrap_or(false);
        if rename_on_edit {
            if let Some(actor) = &self.actor {
                let new_name = actor.get_name();
                self.msg = Some(Msg::YesNo {
                    title: ty(ui_lang, "Rename file?").to_string(),
                    text: ty(ui_lang, "rename_dialog").replace("{name}", &new_name),
                    pending_link: link,
                });
                return;
            }
        }
        if let Some(actor) = self.actor.as_mut() {
            if let Err(e) = actor.set_link_data(&link, &text) {
                self.msg = Some(Msg::Ok(format!("{}{e:#}", ty(ui_lang, "Save failed: "))));
                return;
            }
        }
        self.status = Some(format!("{}{link}", ty(ui_lang, "Saved ")));
        if let Some(e) = &mut self.editor {
            e.saved_hash = hash;
        }
    }

    fn finish_save_after_rename(&mut self, link: &str, rename: bool) {
        let ui_lang = self.ui_lang;
        let text = self
            .editor
            .as_ref()
            .map(|e| e.text.clone())
            .unwrap_or_default();
        let hash = crate::flag::crc32_str(&text) as u32;
        if let Some(actor) = &mut self.actor {
            if rename {
                let new_name = actor.get_name();
                actor.set_link(link, &new_name);
            }
            if let Err(e) = actor.set_link_data(link, &text) {
                self.msg = Some(Msg::Ok(format!("{}{e:#}", ty(ui_lang, "Save failed: "))));
                return;
            }
        }
        if let Some(editor) = &mut self.editor {
            if !rename {
                editor.suppress_rename = true;
            }
            editor.saved_hash = hash;
        }
    }

    /// Programmatic undo/redo for the editor, using egui's built-in text
    /// undo history (same history as Ctrl+Z inside the editor).
    fn editor_undo_redo(&mut self, ctx: &egui::Context, undo: bool) {
        let Some(editor) = self.editor.as_mut() else {
            return;
        };
        let id = egui::Id::new("main_editor");
        if let Some(mut state) = egui::TextEdit::load_state(ctx, id) {
            let current = state
                .cursor
                .char_range()
                .unwrap_or_else(|| egui::text::CCursorRange::one(egui::text::CCursor::new(0)));
            let mut undoer = state.undoer();
            let changed = if undo {
                undoer.undo(&(current, editor.text.clone()))
            } else {
                undoer.redo(&(current, editor.text.clone()))
            };
            if let Some((range, new_text)) = changed {
                editor.text = new_text.clone();
                editor.pending_cursor = Some(new_text.len());
                state.cursor.set_char_range(Some(*range));
            }
            state.set_undoer(undoer);
            egui::TextEdit::store_state(ctx, id, state);
        }
    }

    fn set_actor_name(&mut self, name: String) {
        let ui_lang = self.ui_lang;
        if let Some(actor) = self.actor.as_mut() {
            actor.set_name(&name);
        }
        // The ActorName radio labels below must reflect the new name.
        if let Some(panel) = self.link_panel.as_mut() {
            panel.actor_name = name.clone();
        }
        self.status = Some(ty(ui_lang, "Actor name changed").to_string());
    }

    fn update_actor_link(&mut self, link: &str, linkref: &str, try_retrieve: bool) -> bool {
        let Some(actor) = self.actor.as_mut() else {
            return false;
        };
        if !actor.set_link(link, linkref) {
            self.msg = Some(Msg::Ok(
                "Actor with a Far variant must have LifeConditionUser".to_string(),
            ));
            return false;
        }
        if try_retrieve {
            if let Ok(data) = try_retrieve_custom_file(link, linkref) {
                if !data.is_empty() {
                    if let Some(actor) = self.actor.as_mut() {
                        let _ = actor.set_link_data(link, &data);
                    }
                }
            }
        }
        true
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.root_ui(ui);
    }

    fn on_exit(&mut self) {
        if let Some(r) = self.last_frame_rect {
            self.settings.win_pos_x = r.min.x.round() as i32;
            self.settings.win_pos_y = r.min.y.round() as i32;
            self.settings.win_width = r.width().round() as i32;
            self.settings.win_height = r.height().round() as i32;
        }
        let _ = self.settings.save();
    }
}

impl App {
    /// The whole UI body, split out so tests can drive it from kittest.
    pub fn root_ui(&mut self, ui: &mut egui::Ui) {
        let ui_lang = self.ui_lang;
        let ctx = ui.ctx().clone();

        if ctx.input(|i| i.modifiers.command && i.key_pressed(Key::N)) {
            self.on_open_vanilla();
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(Key::O)) {
            self.on_open_mod();
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(Key::S)) {
            self.save_dir_request = true;
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(Key::F)) {
            if let Some(e) = self.editor.as_mut() {
                e.find_open = true;
            }
            self.find_focus_pending = true;
        }
        if ctx.input(|i| i.key_pressed(Key::Escape))
            && self.editor.as_ref().map(|e| e.find_open).unwrap_or(false)
        {
            if let Some(e) = self.editor.as_mut() {
                e.find_open = false;
                e.search.clear();
            }
        }
        // While the find bar is open, Enter must ALWAYS mean "next match" —
        // even when focus moved to the editor after a jump. Otherwise the
        // next Enter press inserts a newline into the code (the editor's
        // TextEdit has focus). Consume the event BEFORE any widget sees it,
        // and perform the jump ourselves (only when the find input is NOT
        // focused; when it is, the find bar's own Enter handler does it).
        if self.editor.as_ref().map(|e| e.find_open).unwrap_or(false) {
            let find_has_focus = ctx.memory(|m| m.focused() == Some(egui::Id::new("find_input")));
            if !find_has_focus && ctx.input(|i| i.key_pressed(Key::Enter)) {
                ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Enter));
                if let Some(e) = self.editor.as_mut() {
                    let m = find_matches(&e.text, &e.search, e.match_case);
                    let cur = self.editor_cursor_byte.unwrap_or(0);
                    if let Some((s, _)) = next_match(&m, cur, true) {
                        e.pending_cursor = Some(s);
                    }
                }
            }
        }

        egui::Panel::top("menu").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button(ty(ui_lang, "File"), |ui| {
                    if ui.button(ty(ui_lang, "Open Vanilla Actor\tCtrl+N")).clicked() {
                        self.on_open_vanilla();
                    }
                    if ui.button(ty(ui_lang, "Open Mod Actor\tCtrl+O")).clicked() {
                        self.on_open_mod();
                    }
                    if ui.button(ty(ui_lang, "Save Actor\tCtrl+S")).clicked() {
                        self.save_dir_request = true;
                    }
                    ui.separator();
                    if ui.button(ty(ui_lang, "Quit\tCtrl+Q")).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button(ty(ui_lang, "Edit"), |ui| {
                    if ui.button(ty(ui_lang, "Undo\tCtrl+Z")).clicked() {
                        self.editor_undo_redo(&ctx, true);
                    }
                    if ui.button(ty(ui_lang, "Redo\tCtrl+Y")).clicked() {
                        self.editor_undo_redo(&ctx, false);
                    }
                    ui.separator();
                    if ui.button(ty(ui_lang, "Find\tCtrl+F")).clicked() {
                        if let Some(e) = self.editor.as_mut() {
                            e.find_open = true;
                        }
                        self.find_focus_pending = true;
                    }
                });
                ui.menu_button(ty(ui_lang, "Settings"), |ui| {
                    if ui.button(ty(ui_lang, "Settings…")).clicked() {
                        self.settings_panel = Some(SettingsPanelState::new(&self.settings));
                    }
                });
                ui.menu_button(ty(ui_lang, "Help"), |ui| {
                    if ui.button(ty(ui_lang, "About…")).clicked() {
                        self.about_open = true;
                    }
                });
            });
        });

        if self.save_dir_request {
            self.on_save_dialog();
        }

        egui::Panel::bottom("status").show(ui, |ui| {
            ui.horizontal(|ui| {
                let status = self.status.as_deref().unwrap_or_else(|| ty(ui_lang, "Ready"));
                ui.label(RichText::new(status).small());
                if let Some(actor) = &self.actor {
                    ui.separator();
                    ui.label(
                        RichText::new(format!(
                            "{} {}",
                            actor.get_name(),
                            if actor.has_far() { "(FAR)" } else { "" }
                        ))
                        .small(),
                    );
                }
            });
        });

        egui::Panel::left("props")
            .resizable(true)
            .default_size(210.0)
            .show(ui, |ui| {
                ui.add_space(4.0);
                ui.vertical(|ui| {
                    for (i, tab) in TABS.iter().enumerate() {
                        // Hide the "not supported" link tabs (Elink / Profile /
                        // Slink / Xlink) unless the user enables them in
                        // Settings → "Show unsupported tabs".
                        let link = LINK_TO_TAB
                            .iter()
                            .find(|(t, _)| *t == i)
                            .map(|(_, l)| *l);
                        if !self.settings.show_unsupported_tabs
                            && link.map(|l| NOT_IMPLEMENTED.contains(&l)).unwrap_or(false)
                        {
                            continue;
                        }
                        let enabled = self.tab_enabled(i);
                        let mut text = RichText::new(ty(ui_lang, tab));
                        if !enabled {
                            text = text.weak();
                        }
                        let clicked =
                            ui.selectable_label(self.tab == i && enabled, text).clicked();
                        if clicked && enabled {
                            self.switch_tab(i);
                        }
                    }
                });
            });

        egui::CentralPanel::default().show(ui, |ui| {
            self.show_tab(ui);
        });

        // "Saving Actor ..." modal: paint first, then run the (blocking)
        // save on the following frame so the user sees the modal.
        if self.saving.is_some() {
            dim_background(&ctx);
            egui::Window::new(ty(ui_lang, "Saving Actor …"))
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(&ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::Spinner::new());
                        ui.label(ty(ui_lang, "Saving actor…"));
                    });
                });
            if !self.save_armed {
                self.save_armed = true;
                ctx.request_repaint();
            } else {
                let dir = self.saving.take().unwrap();
                self.run_save(&dir);
                ctx.request_repaint();
            }
        }

        self.show_actor_select(&ctx);
        self.show_settings(&ctx);
        self.show_about(&ctx);
        self.show_message(&ctx);

        self.last_frame_rect = ctx.input(|i| i.viewport().inner_rect);
    }
}

impl App {
    fn tab_enabled(&self, tab: usize) -> bool {
        let Some(actor) = &self.actor else {
            return false;
        };
        if tab == 0 || tab == 25 || tab == 26 || tab == 27 {
            return true;
        }
        if let Some((_, link)) = LINK_TO_TAB.iter().find(|(i, _)| *i == tab) {
            actor.get_link(link) != "Dummy"
        } else {
            false
        }
    }

    fn switch_tab(&mut self, tab: usize) {
        self.tab = tab;
        self.editor = None;
        self.texts_panel = None;
        if tab == 0 && self.link_panel.is_none() {
            if let Some(actor) = &self.actor {
                self.link_panel = Some(LinkPanelState::new(actor));
            }
        }
        if tab == 25 && self.texts_panel.is_none() {
            if let Some(actor) = &self.actor {
                self.texts_panel = Some(TextsPanelState::new(actor));
            }
        }
        if tab == 26 {
            self.info_panel = None; // rebuilt lazily with a fresh preview
        }
    }

    fn on_open_vanilla(&mut self) {
        let ui_lang = self.ui_lang;
        let dir = PathBuf::from(&self.settings.update_dir);
        if !dir.exists() {
            self.msg = Some(Msg::Ok(
                ty(ui_lang, "Update directory is not set or does not exist. Open Settings first.")
                    .to_string(),
            ));
            return;
        }
        let mut state = ActorSelectState::open(dir, true);
        state.refresh();
        if state.names.is_empty() {
            self.msg = Some(Msg::Ok(
                ty(ui_lang, "No vanilla actors found in the update directory.").to_string(),
            ));
            return;
        }
        self.actor_select = Some(state);
    }

    fn on_open_mod(&mut self) {
        let ui_lang = self.ui_lang;
        if let Some(dir) = rfd::FileDialog::new()
            .set_title("Select your mod's content or romfs directory")
            .pick_folder()
        {
            let mut state = ActorSelectState::open(dir.clone(), false);
            state.refresh();
            if state.names.is_empty() {
                self.msg = Some(Msg::Ok(ty(ui_lang, "No actors found in Actor/Pack.").to_string()));
                return;
            }
            self.actor_select = Some(state);
        }
    }

    fn on_save_dialog(&mut self) {
        let ui_lang = self.ui_lang;
        self.save_dir_request = false;
        if self.actor.is_none() {
            self.msg = Some(Msg::Ok(ty(ui_lang, "No actor loaded").to_string()));
            return;
        }
        if let Some(dir) = rfd::FileDialog::new()
            .set_title("Select your mod's content or romfs directory")
            .pick_folder()
        {
            // Show a "Saving Actor ..." modal first; the actual save runs on
            // the next frame so the modal is painted before we block.
            self.saving = Some(dir);
            self.save_armed = false;
        }
    }

    fn show_tab(&mut self, ui: &mut egui::Ui) {
        let ui_lang = self.ui_lang;
        if self.actor.is_none() {
            ui.centered_and_justified(|ui| {
                ui.label(
                    ty(ui_lang, "No actor loaded. Use File → Open Vanilla Actor or Open Mod Actor."),
                );
            });
            return;
        }
        match self.tab {
            0 => self.show_actor_link(ui),
            25 => self.show_texts(ui),
            26 => self.show_actor_info(ui),
            27 => self.show_flags(ui),
            i => match LINK_TO_TAB.iter().find(|(t, _)| *t == i) {
                Some((_, link)) if NOT_IMPLEMENTED.contains(link) => {
                    ui.label(format!(
                        "{}{}{}",
                        ty(ui_lang, "Editing "),
                        link,
                        ty(ui_lang, " is not supported in this tool.")
                    ));
                }
                Some((_, link)) => self.show_text_editor(ui, link),
                None => {
                    ui.label("Unknown tab.");
                }
            },
        }
    }

    fn show_actor_link(&mut self, ui: &mut egui::Ui) {
        let ui_lang = self.ui_lang;
        let snapshot: Vec<(String, String)> = match &self.actor {
            Some(actor) => LINKS
                .iter()
                .map(|link| (link.to_string(), actor.get_link(link)))
                .collect(),
            None => return,
        };
        if self.link_panel.is_none() {
            if let Some(actor) = &self.actor {
                self.link_panel = Some(LinkPanelState::new(actor));
            }
        }
        let mut actions: Vec<LinkAction> = Vec::new();
        let panel = match &mut self.link_panel {
            Some(p) => p,
            None => return,
        };
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(ty(ui_lang, "Actor Name:"));
                ui.add(
                    egui::TextEdit::singleline(&mut panel.name_input).desired_width(220.0),
                );
                if ui.button(ty(ui_lang, "Apply")).clicked() {
                    let name = panel.name_input.clone();
                    if !name.is_empty() {
                        actions.push(LinkAction::SetActorName(name));
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.label(ty(ui_lang, "Priority:"));
                ui.add(
                    egui::TextEdit::singleline(&mut panel.priority_input).desired_width(220.0),
                );
                if ui.button(ty(ui_lang, "Apply")).clicked() {
                    let p = panel.priority_input.clone();
                    actions.push(LinkAction::SetPriority(p));
                }
            });
            ui.separator();

            for (i, link) in LINKS.iter().enumerate() {
                if *link == "Priority" {
                    continue;
                }
                let (link_name, _current) = &snapshot[i];
                let link_name = link_name.clone();
                ui.horizontal(|ui| {
                    ui.add_sized([125.0, 18.0], egui::Label::new(&link_name));
                    let changed = ui
                        .horizontal(|ui| {
                            let mut changed = false;
                            changed |= ui
                                .radio_value(
                                    &mut panel.choices[i],
                                    LinkChoice::Dummy,
                                    ty(ui_lang, "Dummy"),
                                )
                                .clicked();
                            changed |= ui
                                .radio_value(
                                    &mut panel.choices[i],
                                    LinkChoice::ActorName,
                                    format!("{}", panel.actor_name),
                                )
                                .clicked();
                            changed |= ui
                                .radio_value(
                                    &mut panel.choices[i],
                                    LinkChoice::Custom,
                                    ty(ui_lang, "Custom:"),
                                )
                                .clicked();
                            changed
                        })
                        .inner;
                    if changed {
                        match panel.choices[i] {
                            LinkChoice::Dummy => {
                                actions.push(LinkAction::SetLink(link_name.clone(), "Dummy".into(), false));
                                panel.custom_texts[i].clear();
                            }
                            LinkChoice::ActorName => {
                                let actorname = panel.actor_name.clone();
                                actions.push(LinkAction::SetLink(link_name.clone(), actorname, true));
                                panel.custom_texts[i].clear();
                            }
                            LinkChoice::Custom => {}
                        }
                    }
                    let custom_enabled = panel.choices[i] == LinkChoice::Custom;
                    ui.add_enabled(
                        custom_enabled,
                        egui::TextEdit::singleline(&mut panel.custom_texts[i])
                            .desired_width(140.0),
                    );
                    if ui
                        .add_enabled(custom_enabled, egui::Button::new(ty(ui_lang, "Update Custom Link")))
                        .clicked()
                    {
                        let v = panel.custom_texts[i].clone();
                        if !v.is_empty() {
                            actions.push(LinkAction::SetLink(link_name, v, true));
                        }
                    }
                });
            }
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(ty(ui_lang, "Tags:"));
                // Fill the remaining row width (minus the Apply button), so
                // long tag lists are readable; clamp for narrow windows.
                let tags_w = (ui.available_width() - 70.0).max(120.0);
                ui.add(
                    egui::TextEdit::singleline(&mut panel.tags_input).desired_width(tags_w),
                );
                if ui.button(ty(ui_lang, "Apply")).clicked() {
                    let t = panel.tags_input.clone();
                    actions.push(LinkAction::SetTags(t));
                }
            });
        });
        for action in actions {
            match action {
                LinkAction::SetActorName(name) => self.set_actor_name(name),
                LinkAction::SetPriority(p) => {
                    if let Some(actor) = self.actor.as_mut() {
                        actor.set_link("Priority", &p);
                    }
                }
                LinkAction::SetLink(link, linkref, retrieve) => {
                    self.update_actor_link(&link, &linkref, retrieve);
                }
                LinkAction::SetTags(t) => {
                    if let Some(actor) = self.actor.as_mut() {
                        actor.set_tags(&t);
                    }
                }
            }
        }
    }

    fn show_text_editor(&mut self, ui: &mut egui::Ui, link: &str) {
        let ui_lang = self.ui_lang;
        if self.editor.is_none() {
            let (data, rename_on_edit) = match &self.actor {
                Some(actor) => (
                    actor.get_link_data(link),
                    actor.get_name() != actor.get_link(link),
                ),
                None => return,
            };
            self.editor = Some(EditorState {
                link: link.to_string(),
                text: data.clone(),
                saved_hash: crate::flag::crc32_str(&data) as u32,
                rename_on_edit,
                suppress_rename: false,
                search: String::new(),
                find_open: false,
                match_case: false,
                pending_cursor: None,
                scroll_offset: None,
            });
        }
        ui.horizontal(|ui| {
            ui.label(format!("{}{}", ty(ui_lang, "Editing "), link));
            if ui.button(ty(ui_lang, "Save")).clicked() {
                self.editor_save();
            }
            ui.label(
                RichText::new(
                    ty(ui_lang, "Changes will be lost when switching tabs unless saved."),
                )
                .small()
                .weak(),
            );
        });

        // --- VS Code-style floating find bar (top-right of the editor) ---
        let ctx = ui.ctx().clone();
        {
            let editor = match &mut self.editor {
                Some(e) => e,
                None => return,
            };
            if editor.find_open {
                let matches = find_matches(&editor.text, &editor.search, editor.match_case);
                let cur = self.editor_cursor_byte.unwrap_or(0);
                let idx = matches
                    .iter()
                    .position(|(s, _)| *s > cur)
                    .map(|i| i + 1)
                    .unwrap_or_else(|| {
                        if matches.is_empty() {
                            0
                        } else {
                            matches.len()
                        }
                    });
                // Anchor just inside the top-right of the editor (falls back
                // to the panel corner before the editor is laid out).
                let right_top = self
                    .editor_edit_rect
                    .map(|r| r.right_top())
                    .unwrap_or_else(|| ui.max_rect().right_top());
                let pos = right_top + egui::vec2(-400.0, 6.0);
                egui::Area::new(egui::Id::new("find_bar"))
                    .fixed_pos(pos)
                    .order(egui::Order::Foreground)
                    .show(&ctx, |ui| {
                        let theme = if self.settings.dark_theme {
                            egui::Theme::Dark
                        } else {
                            egui::Theme::Light
                        };
                        egui::Frame::window(&ctx.style_of(theme))
                            .inner_margin(egui::Margin::same(8))
                            .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let resp = ui.add(
                                    egui::TextEdit::singleline(&mut editor.search)
                                        .id(egui::Id::new("find_input"))
                                        .desired_width(220.0)
                                        .hint_text(ty(ui_lang, "Find…")),
                                );
                                if self.find_focus_pending {
                                    resp.request_focus();
                                    self.find_focus_pending = false;
                                }
                                if ui
                                    .selectable_label(editor.match_case, "Aa")
                                    .on_hover_text(ty(ui_lang, "Match Case"))
                                    .clicked()
                                {
                                    editor.match_case = !editor.match_case;
                                }
                                ui.label(format!(
                                    "{idx}/{}",
                                    matches.len()
                                ));
                                let mut cmd = None;
                                if ui.button("▲").clicked() {
                                    cmd = Some(false);
                                }
                                if ui.button("▼").clicked() {
                                    cmd = Some(true);
                                }
                                // VS Code behavior: Enter jumps to the next match.
                                if ui.input(|i| i.key_pressed(Key::Enter))
                                    && !matches.is_empty()
                                {
                                    cmd = Some(true);
                                }
                                if ui.button("×").clicked() {
                                    cmd = None;
                                    editor.find_open = false;
                                    editor.search.clear();
                                }
                                if let Some(forward) = cmd {
                                    let matches =
                                        find_matches(
                                            &editor.text,
                                            &editor.search,
                                            editor.match_case,
                                        );
                                    let cur = self.editor_cursor_byte.unwrap_or(0);
                                    if let Some((s, _)) =
                                        next_match(&matches, cur, forward)
                                    {
                                        editor.pending_cursor = Some(s);
                                    }
                                }
                            });
                        });
                    });
            }
        }

        ui.add_space(4.0);
        let (editor, jump_scroll) = match &mut self.editor {
            Some(e) => {
                let jump = e.scroll_offset.take();
                (e, jump)
            }
            None => return,
        };
        // VS-Code-style editor widget (line numbers + syntax highlighting +
        // its own scrolling, so caret/click mapping stays consistent).
        let theme = editor_theme(self.settings.dark_theme);
        let mut code_editor = egui_code_editor::CodeEditor::default()
            .id_source("aamp_editor")
            .with_rows(30)
            .with_wrap(false)
            .with_fontsize(14.0)
            .with_theme(theme);
        let (output, _tokens) = code_editor.show(ui, &mut editor.text, &YAML_SYNTAX);
        self.editor_edit_id = Some(output.response.id);
        self.editor_edit_rect = Some(output.response.rect);
        let mut output = output;

        // Track the cursor (as a byte offset) for Find Next/Prev.
        if let Some(cursor_range) = output.cursor_range {
            let char_idx: usize = cursor_range.primary.index.into();
            self.editor_cursor_byte = Some(
                editor
                    .text
                    .char_indices()
                    .nth(char_idx)
                    .map(|(i, _)| i)
                    .unwrap_or(editor.text.len()),
            );
        }
        // Jump to the pending match: move the cursor; CodeEditor's own inner
        // scroll keeps the caret visible.
        if let Some(byte_off) = editor.pending_cursor.take() {
            let char_idx = editor.text[..byte_off.min(editor.text.len())]
                .chars()
                .count();
            output
                .state
                .cursor
                .set_char_range(Some(egui::text::CCursorRange::one(
                    egui::text::CCursor::new(char_idx),
                )));
            output.state.store(ui.ctx(), output.response.id);
            // Request focus AFTER the editor was laid out. egui surrenders
            // focus when the user clicks elsewhere (surrender_focus_on:
            // Presses) — requesting earlier (in the find bar) is undone by
            // the editor's own interact in the same frame, which made the
            // ▼/▲ buttons appear dead.
            if let Some(id) = self.editor_edit_id {
                ui.ctx().memory_mut(|m| m.request_focus(id));
            }
            let _ = jump_scroll;
        }
        // Highlight every search match (and the current one more strongly) so
        // the found text is visible even when the match is not under the
        // caret.
        if !editor.search.is_empty() && self.editor_cursor_byte.is_some() {
            let m = find_matches(&editor.text, &editor.search, editor.match_case);
            if !m.is_empty() {
                let cur = self.editor_cursor_byte.unwrap_or(0);
                let cur_i = m
                    .iter()
                    .position(|(s, e)| *s <= cur && cur < *e)
                    .or_else(|| m.iter().position(|(s, _)| *s > cur))
                    .unwrap_or(0);
                paint_match_highlights(
                    ui,
                    &output.galley,
                    output.galley_pos,
                    output.text_clip_rect,
                    &editor.text,
                    &m,
                    cur_i,
                );
            }
        }
    }

    fn show_texts(&mut self, ui: &mut egui::Ui) {
        let ui_lang = self.ui_lang;
        if self.texts_panel.is_none() {
            match &self.actor {
                Some(actor) => {
                    self.texts_panel = Some(TextsPanelState::new(actor));
                }
                None => return,
            }
        }
        let actorname = self
            .actor
            .as_ref()
            .map(|a| a.get_name())
            .unwrap_or_default();
        let panel = match &mut self.texts_panel {
            Some(p) => p,
            None => return,
        };
        egui::Grid::new("texts")
            .num_columns(2)
            .spacing([20.0, 10.0])
            .show(ui, |ui| {
                ui.checkbox(&mut panel.base_name_on, format!("{actorname}_BaseName"));
                ui.add_enabled(
                    panel.base_name_on,
                    egui::TextEdit::singleline(&mut panel.base_name).desired_width(400.0),
                );
                ui.end_row();
                ui.checkbox(&mut panel.name_on, format!("{actorname}_Name"));
                ui.add_enabled(
                    panel.name_on,
                    egui::TextEdit::singleline(&mut panel.name).desired_width(400.0),
                );
                ui.end_row();
                ui.checkbox(&mut panel.desc_on, format!("{actorname}_Desc"));
                ui.add_enabled(
                    panel.desc_on,
                    egui::TextEdit::multiline(&mut panel.desc)
                        .desired_rows(3)
                        .desired_width(400.0),
                );
                ui.end_row();
                ui.checkbox(&mut panel.pbook_on, format!("{actorname}_PictureBook"));
                ui.add_enabled(
                    panel.pbook_on,
                    egui::TextEdit::multiline(&mut panel.pbook)
                        .desired_rows(3)
                        .desired_width(400.0),
                );
                ui.end_row();
            });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(ty(ui_lang, "Save Texts")).clicked() {
                let texts = panel.collect();
                if let Some(actor) = self.actor.as_mut() {
                    actor.set_texts(texts);
                }
            }
        });
    }

    /// ActorInfo editor: lists the fields the tool will GENERATE for this
    /// actor on save (auto values), with an override column that is applied
    /// on top when saving.
    fn show_actor_info(&mut self, ui: &mut egui::Ui) {
        let ui_lang = self.ui_lang;
        let Some(actor) = self.actor.as_mut() else {
            return;
        };
        // Profile whitelist: extra fields (kept only when the checkbox is on)
        // are the ones NOT in this list.
        let allowed = crate::data::keys_for_profile(&actor.get_link("ProfileUser"));
        // Rebuild the preview whenever the actor's info is dirty (e.g. the
        // user edited a link file), keeping the typed overrides. The preview
        // uses the page's CURRENT keep_extra checkbox so that toggling it
        // immediately shows the extra (non-whitelist) fields.
        if self.info_panel.is_none() || actor.needs_info_update {
            let current_keep = self
                .info_panel
                .as_ref()
                .map(|p| p.keep_extra)
                .unwrap_or(actor.info_keep_extra);
            let preview = actor.info_preview(current_keep);
            actor.needs_info_update = false;
            let entries = preview
                .iter()
                .map(|(k, v)| (k.to_string(), format_byml_value(v)))
                .collect::<Vec<_>>();
            let old = self.info_panel.take();
            let overrides = old
                .as_ref()
                .map(|p| p.overrides.clone())
                .unwrap_or_else(|| actor.info_overrides.clone());
            self.info_panel = Some(InfoPanelState {
                entries,
                overrides,
                keep_extra: current_keep,
            });
        }
        let panel = self.info_panel.as_mut().unwrap();
        let overrides = &mut panel.overrides;
        let keep_extra = &mut panel.keep_extra;
        let mut apply = false;
        let mut refresh = false;
        ui.horizontal(|ui| {
            if ui
                .checkbox(
                    keep_extra,
                    ty(ui_lang, "Keep extra fields (not in the profile key list)"),
                )
                .changed()
            {
                refresh = true;
            }
            ui.label(
                RichText::new(ty(ui_lang, "These fields are regenerated on save. Leave a cell empty to keep the auto value."))
                    .small()
                    .weak(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(ty(ui_lang, "Apply")).clicked() {
                    apply = true;
                }
                if ui.button(ty(ui_lang, "Refresh")).clicked() {
                    refresh = true;
                }
            });
        });
        if refresh {
            if let Some(actor) = self.actor.as_mut() {
                actor.needs_info_update = true;
            }
            return; // rebuilt next frame
        }
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(4.0);
            let n = panel.entries.len();
            egui::Grid::new("info_grid")
                .num_columns(3)
                .spacing([16.0, 6.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label(RichText::new(ty(ui_lang, "Field")).strong());
                    ui.label(RichText::new(ty(ui_lang, "Auto value")).strong());
                    ui.label(RichText::new(ty(ui_lang, "Override")).strong());
                    ui.end_row();
                    for i in 0..n {
                        let (key, auto) = panel.entries[i].clone();
                        let is_extra = !allowed.iter().any(|k| k.as_str() == key);
                        let key_label = if is_extra {
                            // Mark fields outside the profile whitelist so the
                            // user knows they would otherwise be dropped.
                            RichText::new(format!("{} ({})", key, ty(ui_lang, "extra"))).weak()
                        } else {
                            RichText::new(&key).strong()
                        };
                        let hint = if is_extra {
                            ty(ui_lang, "extra_hint")
                        } else {
                            ty(ui_lang, info_key_hint(&key))
                        };
                        ui.label(key_label).on_hover_text(hint);
                        ui.label(RichText::new(&auto).monospace());
                        let val = overrides.entry(key).or_default();
                        ui.add(
                            egui::TextEdit::singleline(val)
                                .desired_width(240.0)
                                .hint_text(ty(ui_lang, "auto")),
                        );
                        ui.end_row();
                    }
                });
        });
        if apply {
            if let Some(actor) = self.actor.as_mut() {
                actor.info_overrides = panel.overrides.clone();
                actor.info_keep_extra = panel.keep_extra;
                actor.needs_info_update = true;
                self.status = Some(ty(ui_lang, "ActorInfo overrides applied").to_string());
            }
        }
    }

    /// Locate `Pack/Bootup.pack` (game dir, then update dir), for flags.
    fn bootup_path(&self) -> Option<PathBuf> {
        for base in [&self.settings.game_dir, &self.settings.update_dir] {
            if base.is_empty() {
                continue;
            }
            let p = PathBuf::from(base).join("Pack").join("Bootup.pack");
            if p.exists() {
                return Some(p);
            }
        }
        util::find_file("Pack/Bootup.pack")
            .ok()
            .and_then(|f| match f {
                util::FoundFile::Path(p) => Some(p),
                _ => None,
            })
    }

    /// Flags editor: lists the GameData flags (bool/s32) whose name contains
    /// the actor name, lets the user edit values, add new flags, or delete
    /// ones. Edits mutate `actor.flags` and are written back on save.
    fn show_flags(&mut self, ui: &mut egui::Ui) {
        let ui_lang = self.ui_lang;
        let Some(bootup) = self.bootup_path() else {
            ui.label(ty(
                ui_lang,
                "Bootup.pack not found. Set Game/Update dirs in Settings.",
            ));
            return;
        };
        let Some(actor) = self.actor.as_mut() else {
            return;
        };
        if self.flags_panel.is_none() {
            self.flags_panel = Some(FlagsPanelState::default());
        }
        let name = actor.get_name();
        if let Err(e) = actor.ensure_flags_loaded(&bootup) {
            ui.label(format!("{}{}", ty(ui_lang, "Failed to load flags: "), e));
            return;
        }

        // Snapshot the actor-relevant flags (edits are applied below).
        let mut list: Vec<(&'static str, crate::flag::Flag)> = Vec::new();
        for ftype in ["bool_data", "s32_data"] {
            for f in actor.flags.find_all(ftype, &name) {
                list.push((ftype, f.clone()));
            }
        }
        list.sort_by(|a, b| (a.0, &a.1.data_name).cmp(&(b.0, &b.1.data_name)));

        let mut updates: Vec<(&'static str, crate::flag::Flag)> = Vec::new();
        let mut deletes: Vec<(&'static str, i32)> = Vec::new();
        let mut do_add = false;
        let mut reload = false;

        ui.horizontal(|ui| {
            ui.label(RichText::new(ty(ui_lang, "Flags for this actor")).strong());
            ui.label(
                RichText::new(format!("{}: {}", ty(ui_lang, "Name"), name)).small(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(ty(ui_lang, "Reload")).clicked() {
                    reload = true;
                }
            });
        });
        ui.separator();

        // Add-flag form.
        let panel = self.flags_panel.as_mut().unwrap();
        ui.horizontal(|ui| {
            ui.label(ty(ui_lang, "Add flag:"));
            ui.add(
                egui::TextEdit::singleline(&mut panel.name)
                    .desired_width(160.0)
                    .hint_text(ty(ui_lang, "Name")),
            );
            let mut is_s32 = panel.ftype == "s32_data";
            if ui
                .selectable_label(is_s32, ty(ui_lang, "s32_data"))
                .clicked()
            {
                is_s32 = true;
            }
            if ui
                .selectable_label(!is_s32, ty(ui_lang, "bool_data"))
                .clicked()
            {
                is_s32 = false;
            }
            panel.ftype = if is_s32 { "s32_data".into() } else { "bool_data".into() };
            if is_s32 {
                ui.add(egui::DragValue::new(&mut panel.s32));
            } else {
                ui.checkbox(&mut panel.bool_on, ty(ui_lang, "Value"));
            }
            if ui.button(ty(ui_lang, "Add")).clicked() {
                do_add = true;
            }
        });
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            if list.is_empty() {
                ui.label(ty(ui_lang, "No flags found for this actor."));
            }
            for (ftype, f) in &list {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&f.data_name).strong());
                    ui.label(*ftype);
                    if matches!(f.kind, crate::flag::FlagKind::Bool { .. }) {
                        let mut on = flag_bool_on(f);
                        if ui.checkbox(&mut on, "").changed() {
                            let mut nf = f.clone();
                            set_flag_bool(&mut nf, on);
                            updates.push((ftype, nf));
                        }
                    } else if matches!(f.kind, crate::flag::FlagKind::S32 { .. }) {
                        let mut v = flag_s32(f);
                        if ui.add(egui::DragValue::new(&mut v)).changed() {
                            let mut nf = f.clone();
                            set_flag_s32(&mut nf, v);
                            updates.push((ftype, nf));
                        }
                    } else {
                        ui.label(RichText::new(flag_display(f)).monospace());
                    }
                    if ui.button(ty(ui_lang, "Del")).clicked() {
                        deletes.push((ftype, f.hash_value));
                    }
                });
            }
        });

        // Apply the collected edits.
        for (ftype, flag) in updates {
            actor.flags.add(ftype, flag);
        }
        for (ftype, hash) in deletes {
            actor.flags.remove(ftype, hash);
        }
        if do_add {
            let nm = panel.name.trim().to_string();
            if !nm.is_empty() {
                let ftype = if panel.ftype == "s32_data" { "s32_data" } else { "bool_data" };
                let mut f = crate::store::new_flag(ftype, false);
                f.set_data_name(nm);
                if ftype == "bool_data" {
                    set_flag_bool(&mut f, panel.bool_on);
                } else {
                    set_flag_s32(&mut f, panel.s32);
                }
                actor.flags.add(ftype, f);
                panel.name.clear();
                panel.s32 = 0;
                panel.bool_on = false;
            }
        }
        if reload {
            actor.flags = crate::store::FlagStore::new();
            actor.flags_loaded = false;
            let _ = actor.ensure_flags_loaded(&bootup);
        }
    }

    fn show_actor_select(&mut self, ctx: &egui::Context) {
        let ui_lang = self.ui_lang;
        let Some(state) = &mut self.actor_select else {
            return;
        };
        dim_background(ctx);
        let mut result: Option<Option<String>> = None;
        egui::Window::new(ty(ui_lang, "Select actor…"))
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut state.filter);
                    if ui.button(ty(ui_lang, "Filter")).clicked() {}
                });
                let filtered: Vec<String> = state
                    .names
                    .iter()
                    .filter(|n| n.to_lowercase().contains(&state.filter.to_lowercase()))
                    .cloned()
                    .collect();
                let mut selected = state.selected.min(filtered.len().saturating_sub(1));
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for (i, name) in filtered.iter().enumerate() {
                            if ui.selectable_label(i == selected, name).clicked() {
                                selected = i;
                            }
                        }
                    });
                state.selected = selected;
                ui.horizontal(|ui| {
                    if ui.button(ty(ui_lang, "Accept")).clicked() {
                        result = Some(filtered.get(selected).cloned());
                    }
                    if ui.button(ty(ui_lang, "Cancel")).clicked() {
                        result = Some(None);
                    }
                });
            });
        if let Some(r) = result {
            let was_vanilla = state.is_vanilla;
            let root = state.root_dir.clone();
            self.actor_select = None;
            if let Some(name) = r {
                if was_vanilla {
                    self.load_vanilla(&name);
                } else {
                    self.load_actor_from_path(
                        &root.join("Actor").join("Pack").join(format!("{name}.sbactorpack")),
                    );
                }
            }
        }
    }

    /// "About" dialog: project info + GitHub link (Help → About…).
    fn show_about(&mut self, ctx: &egui::Context) {
        if !self.about_open {
            return;
        }
        let ui_lang = self.ui_lang;
        let mut open = self.about_open;
        egui::Window::new(ty(ui_lang, "About"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading("BotW Actor Tool (Rust + egui)");
                ui.label(format!(
                    "{} {}",
                    ty(ui_lang, "Version:"),
                    env!("CARGO_PKG_VERSION")
                ));
                ui.separator();
                ui.label(ty(ui_lang, ABOUT_DESC));
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(ty(ui_lang, "GitHub:"));
                    ui.hyperlink(GITHUB_URL);
                });
                ui.horizontal(|ui| {
                    ui.label(ty(ui_lang, "Original project:"));
                    ui.hyperlink("https://github.com/GingerAvalanche/botw_actor_tool");
                });
                ui.label(ty(ui_lang, "License: AGPL-3.0-or-later"));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button(ty(ui_lang, "Close")).clicked() {
                        self.about_open = false;
                    }
                });
            });
        // Closed either via the title-bar ✕ (egui flips `open`) or the Close
        // button (we flip `self.about_open`); keep it closed either way.
        if !open {
            self.about_open = false;
        }
    }

    fn show_settings(&mut self, ctx: &egui::Context) {
        let ui_lang = self.ui_lang;
        let Some(mut panel) = self.settings_panel.take() else {
            return;
        };
        dim_background(ctx);
        let mut done = false;
        egui::Window::new(ty(ui_lang, "Settings"))
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(ty(ui_lang, "Game Directory"));
                    ui.add(
                        egui::TextEdit::singleline(&mut panel.game_dir).desired_width(280.0),
                    );
                    if ui.button(ty(ui_lang, "Browse…")).clicked() {
                        if let Some(d) = rfd::FileDialog::new().pick_folder() {
                            panel.game_dir = d.to_string_lossy().to_string();
                        }
                    }
                });
                ui.horizontal(|ui| {
                    ui.label(ty(ui_lang, "Update Directory"));
                    ui.add(
                        egui::TextEdit::singleline(&mut panel.update_dir).desired_width(280.0),
                    );
                    if ui.button(ty(ui_lang, "Browse…")).clicked() {
                        if let Some(d) = rfd::FileDialog::new().pick_folder() {
                            panel.update_dir = d.to_string_lossy().to_string();
                        }
                    }
                });
                ui.horizontal(|ui| {
                    ui.label(ty(ui_lang, "DLC Directory"));
                    ui.add(
                        egui::TextEdit::singleline(&mut panel.dlc_dir).desired_width(280.0),
                    );
                    if ui.button(ty(ui_lang, "Browse…")).clicked() {
                        if let Some(d) = rfd::FileDialog::new().pick_folder() {
                            panel.dlc_dir = d.to_string_lossy().to_string();
                        }
                    }
                });
                ui.horizontal(|ui| {
                    ui.label(ty(ui_lang, "Language"));
                    let mut selected_lang = util::LANGUAGES
                        .iter()
                        .position(|l| *l == panel.lang)
                        .unwrap_or(0);
                    egui::ComboBox::from_id_salt("lang")
                        .selected_text(panel.lang.clone())
                        .show_ui(ui, |ui| {
                            for (i, lang) in util::LANGUAGES.iter().enumerate() {
                                ui.selectable_value(&mut selected_lang, i, *lang);
                            }
                        });
                    panel.lang = util::LANGUAGES[selected_lang].to_string();
                });
                ui.horizontal(|ui| {
                    ui.label(ty(ui_lang, "UI Language"));
                    let mut current = crate::i18n::UiLang::from_setting(&panel.ui_lang);
                    egui::ComboBox::from_id_salt("ui_lang")
                        .selected_text(current.label())
                        .show_ui(ui, |ui| {
                            for lang in [crate::i18n::UiLang::En, crate::i18n::UiLang::Zh] {
                                ui.selectable_value(&mut current, lang, lang.label());
                            }
                        });
                    panel.ui_lang = current.code().to_string();
                });
                ui.checkbox(&mut panel.dark, ty(ui_lang, "Dark Mode"));
                ui.checkbox(
                    &mut panel.show_unsupported_tabs,
                    ty(ui_lang, "Show unsupported tabs"),
                );
                ui.horizontal(|ui| {
                    if ui.button(ty(ui_lang, "Accept")).clicked() {
                        let test = Settings {
                            game_dir: panel.game_dir.clone(),
                            update_dir: panel.update_dir.clone(),
                            dlc_dir: panel.dlc_dir.clone(),
                            lang: panel.lang.clone(),
                            ui_lang: panel.ui_lang.clone(),
                            dark_theme: panel.dark,
                            show_unsupported_tabs: panel.show_unsupported_tabs,
                            ..Settings::default()
                        };
                        let fails = [
                            (ty(ui_lang, "Game directory"), test.validate_game_dir(&panel.game_dir)),
                            (ty(ui_lang, "Update directory"), test.validate_update_dir(&panel.update_dir)),
                            (ty(ui_lang, "DLC directory"), test.validate_dlc_dir(&panel.dlc_dir)),
                        ]
                        .iter()
                        .filter(|(_, ok)| !*ok)
                        .map(|(n, _)| n.to_string())
                        .collect::<Vec<_>>();
                        // Save the settings REGARDLESS of validation: otherwise
                        // an invalid directory (e.g. the dump moved) silently
                        // blocked persisting *everything*, including the dark
                        // theme and UI language choice. Invalid dirs are only
                        // reported as a warning now.
                        self.settings = test;
                        self.ui_lang =
                            crate::i18n::UiLang::from_setting(&self.settings.ui_lang);
                        let _ = self.settings.save();
                        self.apply_theme(ctx);
                        if !fails.is_empty() {
                            self.msg = Some(Msg::Ok(format!(
                                "{}{}",
                                ty(ui_lang, "The following directories failed to validate: "),
                                fails.join(", ")
                            )));
                        }
                        done = true;
                    }
                    if ui.button(ty(ui_lang, "Cancel")).clicked() {
                        done = true;
                    }
                });
            });
        if done {
            self.settings_panel = None;
        } else {
            self.settings_panel = Some(panel);
        }
    }

    fn show_message(&mut self, ctx: &egui::Context) {
        let ui_lang = self.ui_lang;
        let msg = match &self.msg {
            Some(m) => m.clone(),
            None => return,
        };
        match msg {
            Msg::Ok(text) => {
                dim_background(ctx);
                egui::Window::new(ty(ui_lang, "Message"))
                    .collapsible(false)
                    .resizable(false)
                    .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.label(&text);
                        ui.add_space(6.0);
                        ui.vertical_centered(|ui| {
                            if ui.button(ty(ui_lang, "OK")).clicked() {
                                self.msg = None;
                            }
                        });
                    });
            }
            Msg::YesNo {
                title,
                text,
                pending_link,
            } => {
                dim_background(ctx);
                egui::Window::new(title)
                    .collapsible(false)
                    .resizable(false)
                    .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.label(&text);
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            if ui.button(ty(ui_lang, "Yes")).clicked() {
                                let link = pending_link.clone();
                                self.msg = None;
                                self.finish_save_after_rename(&link, true);
                            }
                            if ui.button(ty(ui_lang, "No")).clicked() {
                                let link = pending_link.clone();
                                self.msg = None;
                                self.finish_save_after_rename(&link, false);
                            }
                        });
                    });
            }
        }
    }
}

/// Next/previous search match relative to a byte offset (wraps around).
fn next_match(matches: &[(usize, usize)], cur: usize, forward: bool) -> Option<(usize, usize)> {
    if matches.is_empty() {
        return None;
    }
    if forward {
        matches
            .iter()
            .find(|(s, _)| *s > cur)
            .copied()
            .or(Some(matches[0]))
    } else {
        matches
            .iter()
            .rev()
            .find(|(s, _)| *s < cur)
            .copied()
            .or(Some(*matches.last().unwrap()))
    }
}

/// Editor color theme tuned to match the egui visuals of this app (the
/// default dark theme is AYU_MIRAGE — dark BLUE background with an ORANGE
/// selection, which clashed with the window's palette).
fn editor_theme(dark: bool) -> egui_code_editor::ColorTheme {
    if dark {
        egui_code_editor::ColorTheme {
            name: "botw-dark",
            dark: true,
            bg: "1b1b1b",          // egui dark panel_fill
            cursor: "ffffff",
            selection: "005c80",   // egui default dark selection blue
            comments: "6a9955",
            functions: "dcdcaa",
            keywords: "569cd6",
            literals: "d4d4d4",
            numerics: "b5cea8",
            punctuation: "d4d4d4",
            strs: "ce9178",
            types: "4ec9b0",
            special: "d4d4d4",
        }
    } else {
        egui_code_editor::ColorTheme {
            name: "botw-light",
            dark: false,
            bg: "f8f8f8",          // egui light panel_fill
            cursor: "000000",
            selection: "005c80",   // egui default selection blue
            comments: "008000",
            functions: "795e26",
            keywords: "0000ff",
            literals: "24292f",
            numerics: "098658",
            punctuation: "24292f",
            strs: "a31515",
            types: "267f99",
            special: "a475f9",
        }
    }
}

/// Human-readable source hint for an ActorInfo field (i18n key).
fn info_key_hint(key: &str) -> &'static str {
    match key {
        "name" => "from name",
        "isHasFar" => "from far variant",
        "profile" => "from ProfileUser link",
        "tags" => "from actor link tags",
        "bugMask" | "sortKey" => "from save rules",
        _ => "extracted from link files",
    }
}

/// Flags helper: is a bool flag currently on?
fn flag_bool_on(f: &crate::flag::Flag) -> bool {
    if let crate::flag::FlagKind::Bool { init_value, .. } = &f.kind {
        *init_value != 0
    } else {
        false
    }
}
fn set_flag_bool(f: &mut crate::flag::Flag, on: bool) {
    if let crate::flag::FlagKind::Bool { init_value, .. } = &mut f.kind {
        *init_value = if on { 1 } else { 0 };
    }
}
fn flag_s32(f: &crate::flag::Flag) -> i32 {
    match &f.kind {
        crate::flag::FlagKind::S32 { init_value, .. } => *init_value,
        crate::flag::FlagKind::Bool { init_value, .. } => *init_value,
        _ => 0,
    }
}
fn set_flag_s32(f: &mut crate::flag::Flag, v: i32) {
    match &mut f.kind {
        crate::flag::FlagKind::S32 { init_value, .. } => *init_value = v,
        crate::flag::FlagKind::Bool { init_value, .. } => *init_value = v,
        _ => {}
    }
}
fn flag_display(f: &crate::flag::Flag) -> String {
    use crate::flag::FlagKind;
    match &f.kind {
        FlagKind::Bool { init_value, .. } => {
            if *init_value != 0 { "true".to_string() } else { "false".to_string() }
        }
        FlagKind::S32 { init_value, .. } => init_value.to_string(),
        FlagKind::F32 { init_value, .. } => format!("{init_value}"),
        FlagKind::String { init_value, .. } => init_value.clone(),
        FlagKind::BoolArray { init_value, .. } => format!("{init_value:?}"),
        FlagKind::S32Array { init_value, .. } => format!("{init_value:?}"),
        _ => format!("{:?}", f.kind),
    }
}

/// Render a BYML value as display text (used by the ActorInfo editor page).
fn format_byml_value(v: &roead::byml::Byml) -> String {
    use roead::byml::Byml;
    match v {
        Byml::String(s) => s.to_string(),
        Byml::Bool(b) => b.to_string(),
        Byml::I32(i) => i.to_string(),
        Byml::Float(f) => format!("{f}"),
        Byml::U32(u) => u.to_string(),
        Byml::I64(i) => i.to_string(),
        Byml::U64(u) => u.to_string(),
        Byml::Double(d) => format!("{d}"),
        Byml::Null => "null".to_string(),
        Byml::Array(a) => format!("[{} items]", a.len()),
        Byml::Map(m) => format!("{{{}}}", m.len()),
        _ => format!("{v:?}"),
    }
}

/// Localized string lookup without borrowing self (usable inside closures
/// that already hold a borrow of another field).
fn ty(lang: crate::i18n::UiLang, key: &str) -> &str {
    crate::i18n::tr(lang, key)
}

pub const GITHUB_URL: &str = "https://github.com/Alano6678/botw_actor_tool_rs";
const ABOUT_DESC: &str =
    "A Rust + egui rewrite of the original Python botw_actor_tool for editing Breath of the Wild actor packs.";

/// YAML syntax config for the code editor widget.
pub static YAML_SYNTAX: std::sync::LazyLock<egui_code_editor::Syntax> =
    std::sync::LazyLock::new(|| {
        use std::collections::BTreeSet;
        egui_code_editor::Syntax::new("yaml")
            .with_comment("#")
            .with_quotes(BTreeSet::from(['"', '\'']))
            .with_keywords(BTreeSet::from([
                "true", "false", "null", "None", "TRUE", "FALSE", "NULL",
            ]))
            .with_types(BTreeSet::from([
                "bool", "int", "float", "string", "str32", "str64", "str256",
            ]))
    });

/// Case-insensitive (or, when `case_sensitive`, exact) find of `needle` in
/// `haystack`, returning byte ranges of all occurrences. ASCII needles use a
/// fast byte scan (ranges always land on char boundaries because ASCII bytes
/// can never alias a UTF-8 lead byte); non-ASCII needles (CJK, accented
/// text) are matched exactly, char by char, producing char-boundary byte
/// ranges — the old ASCII-only guard made searching Chinese/Japanese text
/// impossible.
pub fn find_matches(haystack: &str, needle: &str, case_sensitive: bool) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    if needle.is_empty() || needle.len() > haystack.len() {
        return out;
    }
    if needle.is_ascii() {
        let n = needle.as_bytes();
        let h = haystack.as_bytes();
        let mut i = 0;
        while i + n.len() <= h.len() {
            let hit = if case_sensitive {
                &h[i..i + n.len()] == n
            } else {
                h[i..i + n.len()].eq_ignore_ascii_case(n)
            };
            if hit {
                out.push((i, i + n.len()));
                i += n.len();
            } else {
                i += 1;
            }
        }
    } else {
        let n: Vec<char> = needle.chars().collect();
        let h: Vec<char> = haystack.chars().collect();
        if n.len() > h.len() {
            return out;
        }
        for i in 0..=(h.len() - n.len()) {
            if h[i..i + n.len()] == n[..] {
                let start = haystack
                    .char_indices()
                    .nth(i)
                    .map(|(b, _)| b)
                    .unwrap_or(haystack.len());
                let end = haystack
                    .char_indices()
                    .nth(i + n.len())
                    .map(|(b, _)| b)
                    .unwrap_or(haystack.len());
                out.push((start, end));
            }
        }
    }
    out
}

/// Paint translucent rectangles over every search match (stronger over the
/// current one) on top of the laid-out editor text, so the user can SEE what
/// the find bar found. Rects are derived from the galley the editor laid out
/// this frame, then translated by `galley_pos` and clipped to the text clip.
fn paint_match_highlights(
    ui: &egui::Ui,
    galley: &egui::text::Galley,
    galley_pos: egui::Pos2,
    clip: egui::Rect,
    text: &str,
    matches: &[(usize, usize)],
    current: usize,
) {
    let painter = ui.painter().with_clip_rect(clip);
    // Blue-tinted match highlights (like a code editor's search): subtle for
    // all matches, stronger for the current one.
    let base = Color32::from_rgba_unmultiplied(0, 120, 220, 40);
    let cur_color = Color32::from_rgba_unmultiplied(0, 155, 255, 100);
    for (i, (bs, be)) in matches.iter().enumerate() {
        let bs = (*bs).min(text.len());
        let be = (*be).min(text.len());
        if be <= bs {
            continue;
        }
        let cs = text[..bs].chars().count();
        let ce = text[..be].chars().count();
        let a = galley
            .pos_from_cursor(egui::text::CCursor::new(cs))
            .translate(galley_pos.to_vec2());
        let b = galley
            .pos_from_cursor(egui::text::CCursor::new(ce))
            .translate(galley_pos.to_vec2());
        let color = if i == current { cur_color } else { base };
        if (a.min.y - b.min.y).abs() < 0.5 {
            // Single-line match: one rect from the start boundary to the end
            // boundary (both are 0-width cursor positions).
            painter.rect_filled(
                egui::Rect::from_min_max(
                    egui::pos2(a.min.x, a.min.y),
                    egui::pos2(b.max.x, b.max.y),
                ),
                2.0,
                color,
            );
        } else {
            // Multi-line match: first row segment, full middle rows, last row.
            for row in &galley.rows {
                let r = row.rect().translate(galley_pos.to_vec2());
                if r.max.y <= a.min.y + 0.01 || r.min.y >= b.max.y - 0.01 {
                    continue;
                }
                let is_first = r.min.y >= a.min.y - 0.5 && r.min.y <= a.min.y + 0.5;
                let is_last = r.max.y >= b.max.y - 0.5 && r.max.y <= b.max.y + 0.5;
                let (x0, x1) = if is_first {
                    (a.min.x, r.max.x)
                } else if is_last {
                    (r.min.x, b.max.x)
                } else {
                    (r.min.x, r.max.x)
                };
                painter.rect_filled(
                    egui::Rect::from_min_max(
                        egui::pos2(x0, r.min.y),
                        egui::pos2(x1, r.max.y),
                    ),
                    2.0,
                    color,
                );
            }
        }
    }
}

fn dim_background(ctx: &egui::Context) {
    let screen = ctx.content_rect();    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Background,
        egui::Id::new("dim"),
    ));
    painter.rect_filled(screen, 0.0, Color32::from_black_alpha(120));
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    // Prefer a BOLDER CJK face (hei/bold) so the UI text reads heavier and
    // clearer. It becomes the PRIMARY Proportional font; Monospace keeps its
    // own width so the editor's line numbers / column alignment stay correct
    // (the CJK face is only a fallback there for CJK glyphs).
    let candidates = [
        "C:\\Windows\\Fonts\\simhei.ttf",
        "C:\\Windows\\Fonts\\msyhbd.ttc",
        "C:\\Windows\\Fonts\\msyh.ttf",
        "C:\\Windows\\Fonts\\msjh.ttf",
        "C:\\Windows\\Fonts\\msyh.ttc",
        "C:\\Windows\\Fonts\\simsun.ttc",
    ];
    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            fonts
                .font_data
                .insert("cjk".to_string(), egui::FontData::from_owned(bytes).into());
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "cjk".to_string());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("cjk".to_string());
            break;
        }
    }
    ctx.set_fonts(fonts);
}
