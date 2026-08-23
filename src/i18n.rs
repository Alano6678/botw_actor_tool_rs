//! Minimal UI localization (English / Chinese).
//!
//! `tr(lang, key)` returns a translated string for `key`, falling back to the
//! key itself for English (so keys read like English sentences).

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum UiLang {
    #[default]
    En,
    Zh,
}

impl UiLang {
    pub fn from_setting(s: &str) -> Self {
        if s == "zh" {
            UiLang::Zh
        } else {
            UiLang::En
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            UiLang::En => "en",
            UiLang::Zh => "zh",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            UiLang::En => "English",
            UiLang::Zh => "中文",
        }
    }
}

pub fn tr(lang: UiLang, key: &str) -> &str {
    if lang == UiLang::En {
        return key;
    }
    match key {
        // Menus
        "File" => "文件",
        "Open Vanilla Actor\tCtrl+N" => "打开原始 Actor\tCtrl+N",
        "Open Mod Actor\tCtrl+O" => "打开 Mod Actor\tCtrl+O",
        "Save Actor\tCtrl+S" => "保存 Actor\tCtrl+S",
        "Quit\tCtrl+Q" => "退出\tCtrl+Q",
        "Edit" => "编辑",
        "Undo\tCtrl+Z" => "撤回\tCtrl+Z",
        "Redo\tCtrl+Y" => "前进\tCtrl+Y",
        "Find\tCtrl+F" => "查找\tCtrl+F",
        "Settings" => "设置",
        "Settings…" => "设置…",
        "Help" => "帮助",
        "About…" => "关于…",
        "About" => "关于",
        "Version:" => "版本：",
        "GitHub:" => "GitHub：",
        "Original project:" => "原版项目：",
        "License: AGPL-3.0-or-later" => "许可：AGPL-3.0-or-later",
        "Close" => "关闭",
        "A Rust + egui rewrite of the original Python botw_actor_tool for editing Breath of the Wild actor packs." => {
            "Rust + egui 重写版：原 Python 版 botw_actor_tool 的移植，用于编辑《塞尔达传说：旷野之息》的 Actor 包。"
        }
        // Status
        "Ready" => "就绪",
        "No actor loaded" => "未加载 Actor",
        "Actor name changed" => "Actor 名称已更改",
        "Save failed: " => "保存失败：",
        "Saved to " => "已保存至 ",
        "Saved " => "已保存 ",
        "Loaded " => "已加载 ",
        // Actor link panel
        "Actor Name:" => "Actor 名称：",
        "Priority:" => "优先级：",
        "Apply" => "应用",
        "Dummy" => "Dummy",
        "Custom:" => "自定义：",
        "Update Custom Link" => "更新自定义链接",
        "Tags:" => "标签：",
        // Editor
        "Save" => "保存",
        "Editing " => "编辑 ",
        "Changes will be lost when switching tabs unless saved." => "切换标签页时未保存的更改将丢失。",
        "Find…" => "查找…",
        "Run\n" => "运行\n",
        // Dialogs
        "Message" => "消息",
        "OK" => "确定",
        "Yes" => "是",
        "No" => "否",
        "Select actor…" => "选择 Actor…",
        "Filter" => "筛选",
        "Accept" => "确定",
        "Cancel" => "取消",
        "Settings" => "设置",
        "Rename file?" => "重命名文件？",
        "rename_dialog" =>
            "当前文件名是自定义的，可能被其他 Actor 共用。\n\
             如果该文件名被其他 Actor 共用，修改文件可能引发问题。\n\
             强烈建议将文件名改为自己的 Actor 名以避免冲突。\n\n\
             是否将文件名改为 {name}？",
        "Saving Actor …" => "正在保存 Actor…",
        "Saving actor…" => "正在保存 actor…",
        // Settings panel
        "Game Directory" => "游戏目录",
        "Update Directory" => "更新目录",
        "DLC Directory" => "DLC 目录",
        "Browse…" => "浏览…",
        "Language" => "文本语言",
        "UI Language" => "界面语言",
        "Dark Mode" => "深色模式",
        "Match Case" => "区分大小写",
        // Left tab bar (tab names shown when UI language is Chinese)
        "Actor Link" => "Actor 链接",
        "AI Program" => "AI 程序",
        "AI Schedule" => "AI 计划",
        "AS" => "AS",
        "Attention" => "注意力",
        "Awareness" => "感知",
        "Bone Control" => "骨骼控制",
        "Chemical" => "化学",
        "Damage Param" => "伤害参数",
        "Drop Table" => "掉落表",
        "Elink" => "Elink",
        "General Param" => "通用参数",
        "Life Condition" => "生存条件",
        "LOD" => "LOD",
        "Model" => "模型",
        "Physics" => "物理",
        "Profile" => "Profile",
        "Ragdoll Blend" => "布娃娃混合",
        "Ragdoll Config" => "布娃娃配置",
        "Recipe" => "配方",
        "Shop Data" => "商店数据",
        "Slink" => "Slink",
        "UMii" => "UMii",
        "Xlink" => "Xlink",
        "Animation Info" => "动画信息",
        "Texts" => "文本",
        "Flags" => "标志",
        // ActorInfo editor page
        "These fields are regenerated on save. Leave a cell empty to keep the auto value." => {
            "以下字段会在保存时自动重新生成。留空表示使用自动值。"
        }
        "Field" => "字段",
        "Auto value" => "自动值",
        "Override" => "覆盖",
        "Refresh" => "刷新",
        "Keep extra fields (not in the profile key list)" => {
            "保留额外字段（不在 profile 键列表中的旧字段）"
        }
        "auto" => "自动",
        "extra" => "额外",
        "extra_hint" => "额外字段：不在该 profile 的键列表中，仅在勾选「保留额外字段」时才会保留。",
        "ActorInfo overrides applied" => "ActorInfo 覆盖已应用",
        "from name" => "来源：Actor 名称",
        "from far variant" => "来源：Far 变体",
        "from ProfileUser link" => "来源：ProfileUser 链接",
        "from actor link tags" => "来源：Actor Link 标签",
        "from save rules" => "来源：保存规则",
        "extracted from link files" => "来源：从链接文件提取",
        // Flags editor page
        "Bootup.pack not found. Set Game/Update dirs in Settings." => {
            "未找到 Bootup.pack，请在设置中设置 Game/Update 目录。"
        }
        "Failed to load flags: " => "加载标志失败：",
        "Flags for this actor" => "该 Actor 的标志",
        "Name" => "名称",
        "Value" => "值",
        "Reload" => "重新加载",
        "Add flag:" => "添加标志：",
        "Add" => "添加",
        "Del" => "删除",
        "No flags found for this actor." => "未找到该 Actor 的标志。",
        // Misc messages
        "No actor loaded. Use File → Open Vanilla Actor or Open Mod Actor." => {
            "未加载 Actor。使用 文件 → 打开原始 Actor 或 打开 Mod Actor。"
        }
        "The Flags tab is not implemented yet (same as the original tool)." => {
            "Flags 标签页尚未实现（与原版相同）。"
        }
        " is not supported in this tool." => " 在本工具中不受支持。",
        "Update directory is not set or does not exist. Open Settings first." => {
            "更新目录未设置或不存在，请先打开设置。"
        }
        "No vanilla actors found in the update directory." => "更新目录中未找到原始 Actor。",
        "No actors found in Actor/Pack." => "在 Actor/Pack 中未找到 Actor。",
        "Must choose either content or romfs!" => "必须选择 content 或 romfs 目录！",
        "Failed to load actor: " => "加载 Actor 失败：",
        "Actor with a Far variant must have LifeConditionUser" => {
            "带 Far 变体的 Actor 必须保留 LifeConditionUser"
        }
        "The following directories failed to validate: " => "以下目录校验失败：",
        "Game directory" => "游戏目录",
        "Update directory" => "更新目录",
        "DLC directory" => "DLC 目录",
        _ => key,
    }
}
