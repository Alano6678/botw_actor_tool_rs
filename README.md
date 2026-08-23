# BotW Actor Tool (Rust + egui)

A Rust desktop rewrite of the original Python **BotW Actor Tool**
([GingerAvalanche/botw_actor_tool](https://github.com/GingerAvalanche/botw_actor_tool)) for
_*The Legend of Zelda: Breath of the Wild*_. It edits actor packs (`.sbactorpack`):
load vanilla/mod actors, edit the Actor Link (Dummy / ActorName / Custom +
custom-file import), edit every link file as YAML (AAMP/BYML), edit Texts
(MSBT), dark theme, settings — and on save it rewrites `ActorInfo.product.sbyml`,
the text files and the GameData flags, all regenerated automatically.

The goal is a faithful port: same workflow and behavior as the original,
with a modern GUI and a few improvements (see *Differences* below).

## Features

- Open vanilla actors from your game dump (`Ctrl+N`) or actors from a mod
  directory (`Ctrl+O`), including `TitleBG.pack`-resident actors and
  `*_Far` variants.
- **Actor Link** panel: actor name + Priority, every link as
  Dummy / ActorName / Custom, custom-link data import from vanilla, Tags.
- YAML editor for every link file (VS Code style: line numbers, YAML syntax
  highlighting, accurate caret/click positioning) with a floating find bar
  (`Ctrl+F`, case-insensitive, match highlighting, n/m counter, next/prev).
- **Texts** tab: BaseName / Name / Desc / PictureBook per language.
- Full save pipeline (`Ctrl+S`): Yaz0-compressed pack write-back
  (`content` = Wii U big-endian, `romfs` = Switch little-endian),
  `ActorInfo.product.sbyml` regeneration (CRC32-sorted hashes,
  `keys_by_profile.json` / `overrides.json`), `Pack/TitleBG.pack` injection for
  resident actors, `Pack/Bootup.pack` flag injection
  (`gamedata.ssarc` + `savedataformat.ssarc`) and MSBT text writing.
- **Localized UI**: English / 中文 (independent from the game text language).
- Dark mode with a complete theme (not just a window background — the
  original only recolored a few widgets).

## Tech (all formats handled by libraries)

| Format | Library |
| ---- | ---- |
| AAMP / BYML / SARC / Yaz0 | [roead](https://github.com/NiceneNerd/roead) 1.0 (Rust port of oead) |
| MSBT (texts) | [msyt](https://github.com/NiceneNerd/msyt) 1.4 + [msbt-rs](https://github.com/NiceneNerd/msbt-rs) |
| GUI | [egui / eframe](https://github.com/emilk/egui) 0.36 |
| File dialogs | [rfd](https://crates.io/crates/rfd) |
| YAML code editor | [egui_code_editor](https://crates.io/crates/egui_code_editor) 0.4 |
| Data | `data/*.json` reused from the original tool (embedded at compile time) |

> **About vendor/**: `msyt`/`msbt-rs` are vendored under `vendor/` as local
> path dependencies because the original machine had `github.com` blocked.
> On a normal network you can change `Cargo.toml` back to
> `msyt = { git = "https://github.com/NiceneNerd/msyt" }` and delete `vendor/`.
> `.cargo/config.toml` is a machine-specific workaround for a dead proxy and
> can be removed as well.

## Build & run

```bash
cargo run          # debug mode
cargo build --release && ./target/release/botw_actor_tool_rs.exe
cargo test         # unit tests (CRC32, flag round-trips, editor input)
```

Requires CMake + MSVC (roead's Yaz0 is a C++ FFI and builds zlib-ng via CMake).

## Usage

1. Open **Settings** first and set the Game / Update / DLC directories
   (unpacked ROM directories, same as the original tool).
2. `Ctrl+N`: pick a vanilla actor from the Update directory.
3. `Ctrl+O`: pick the mod's `content` or `romfs` directory, then an actor from
   `Actor/Pack`.
4. Left tabs: Actor Link (links / tags / name), YAML editors for each link
   file, Texts.
5. `Ctrl+S`: pick the mod directory to save (the folder must be named
   `content` (big-endian / Wii U) or `romfs` (little-endian / Switch)).

## Source layout (src/)

```
main.rs       entry point (eframe)
app.rs        egui UI (main window / panels / dialogs)
actor.rs      BATActor orchestration (load / rename / save; port of actor.py)
actorinfo.rs  ActorInfo entry generation (port of actorinfo.py)
pack.rs       ActorPack: SARC/AAMP/BYML container (port of pack.py)
flag.rs       GameData flag model + overrides rules (port of flag.py)
store.rs      FlagStore (port of store.py)
texts.rs      MSBT text read/write via msyt (port of texts.py)
util.rs       constants, find_file, gamedata/savedata writing (port of util.py)
settings.rs   settings (JSON, %LOCALAPPDATA%\botw_actor_tool_rs\settings.json)
data.rs       static data loading (data/*.json, compile-time include_str!)
```

## Performance

- `[profile.dev]` uses `opt-level = 1` with dependencies at `opt-level = 2`
  (an unoptimized egui debug build is noticeably laggy; the first build is
  slower).
- The YAML editor uses `CodeEditor::show` which caches highlighting/layout by
  text hash — unchanged text is never re-laid out.
- Release build: `cargo build --release`.

## Tests

```bash
cargo test                    # unit tests (CRC32, flag round-trips, editor input)
cargo test real_dump -- --ignored --nocapture
# ignored tests read the real game dirs from settings.json, load several
# representative actors and verify Texts read Name/Desc/PictureBook correctly.
```

## Differences vs the original

- **Localized UI**: Settings → UI Language (English / 中文), independent of
  the game text language.
- **Code editor** uses egui_code_editor (VS Code style) with accurate
  caret/click mapping (a custom-layouter TextEdit had caret drift and broken
  deletion and was replaced).
- **Find bar**: `Ctrl+F` or Edit → Find opens a VS Code-style floating search
  bar (case-insensitive, match highlighting, n/m counter, next/prev with
  auto-scroll).
- **Edit menu**: undo (Ctrl+Z) / redo (Ctrl+Y) / find (Ctrl+F).
- Editing font is a consistent 14 px monospace so the caret aligns with
  characters pixel-perfectly.
- A **"Saving Actor …"** modal is shown while saving.
- Save/load errors surface as clear dialogs (the original silently swallowed
  them).
- Settings moved from INI to JSON; vanilla custom-file imports no longer ask
  before importing.
- `vanilla_params.json` / `instSize_data.json` are not shipped (unused by the
  original code as well).
- The PyPI update check was removed.
- Not implemented (same as the original): the Flags tab, and Elink/Profile/
  Slink/Xlink editing (the original showed blank pages for those tabs).

## License

GNU **AGPL-3.0-or-later** (same as the original; roead is GPL-3.0-or-later,
AGPLv3 is compatible).

---

## 中文说明

`botw_actor_tool_rs` 是原 Python 版 BotW Actor Tool（`GingerAvalanche/botw_actor_tool`）
的 Rust 重写版：桌面 GUI，用于编辑《塞尔达传说：旷野之息》的 Actor
（`.sbactorpack`）。功能与行为尽量与原版保持一致：加载原始/Mod Actor、编辑
Actor Link（Dummy / ActorName / Custom + 自定义链接导入）、按 YAML 编辑各链接
文件（AAMP/BYML）、编辑 Texts（MSBT）、暗色主题、设置，以及保存时回写
`ActorInfo.product.sbyml`、文本与 GameData Flags（全部由程序自动重新生成）。

### 功能

- `Ctrl+N` 从游戏解包目录打开原版 Actor；`Ctrl+O` 从 Mod 目录的
  `Actor/Pack` 选择 Actor 打开（含 TitleBG.pack 内 resident actor 与
  `*_Far` 变体）。
- **Actor Link** 页：Actor Name + Priority、每个链接（Dummy / ActorName /
  Custom 单选 + "Update Custom Link"）、自定义链接从原版文件导入、Tags。
- 各链接文件的 YAML 编辑器（VS Code 风格：行号、语法高亮、精确的光标/点击定位）
  + `Ctrl+F` 浮动搜索条（大小写不敏感、匹配高亮、n/m 计数、上一个/下一个）。
- **Texts** 页：BaseName / Name / Desc / PictureBook（按语言）。
- 保存（`Ctrl+S`）完整回写：Yaz0 压缩 pack（`content` 大端/Wii U、
  `romfs` 小端/Switch）、`ActorInfo.product.sbyml` 条目重生成（CRC32 哈希、`keys_by_profile.json` /
  `overrides.json`）、resident actor 注入 `Pack/TitleBG.pack`、`Pack/Bootup.pack`
  注入标志（`gamedata.ssarc` + `savedataformat.ssarc`）、MSBT 文本写入。
- **中英文本地化**：设置 → UI Language（English / 中文），与游戏文本语言独立。

### 技术选型（格式全部使用库）

| 格式 | 库 |
| ---- | ---- |
| AAMP / BYML / SARC / Yaz0 | [roead](https://github.com/NiceneNerd/roead) 1.0（oead 的 Rust 移植） |
| MSBT（文本） | [msyt](https://github.com/NiceneNerd/msyt) 1.4 + [msbt-rs](https://github.com/NiceneNerd/msbt-rs) |
| GUI | [egui / eframe](https://github.com/emilk/egui) 0.36 |
| 文件对话框 | [rfd](https://crates.io/crates/rfd) |
| YAML 代码编辑器 | [egui_code_editor](https://crates.io/crates/egui_code_editor) 0.4 |
| 数据 | `data/*.json` 直接沿用原版工具的数据（编译期嵌入） |

> **关于 vendor/**：因为本机 hosts 将 `github.com` 指向 127.0.0.1（git 拉取失败），
> `msyt`/`msbt-rs` 以 tarball 形式放入了 `vendor/` 并作为本地 path 依赖。
> 若网络正常，可将 `Cargo.toml` 中的 `msyt = { path = "vendor/msyt" }` 改回
> `msyt = { git = "https://github.com/NiceneNerd/msyt" }`，并删除 `vendor/`。
> 另外 `.cargo/config.toml` 针对本机关闭了失效的 127.0.0.1:2080 代理，
> 换成正常环境后可以删除。

### 构建与运行

```bash
cargo run          # 调试模式
cargo build --release && ./target/release/botw_actor_tool_rs.exe
cargo test         # 单元测试（CRC32、Flag 往返）
```

需要 CMake + MSVC（roead 的 Yaz0 是 C++ FFI，会自动用 CMake 编译 zlib-ng）。

### 使用

1. 启动后先打开 **Settings**，设置 Game / Update / DLC 目录（纯文本 ROM 解包目录，
   与原版相同）。
2. `Ctrl+N`：从 Update 目录选择原始 Actor 打开。
3. `Ctrl+O`：选择 Mod 的 `content` 或 `romfs` 目录，从其 `Actor/Pack` 选择 Actor 打开。
4. 左侧标签页：Actor Link（链接/标签/名称）、各链接文件的 YAML 编辑器、Texts。
5. `Ctrl+S`：选择 Mod 目录保存（目录名必须是 `content`（大端/Wii U）或
   `romfs`（小端/Switch））。

### 目录结构（src/）

```
main.rs      入口（eframe 启动）
app.rs       egui 界面（主窗口/面板/对话框）
actor.rs     BATActor 编排（加载/重命名/保存，Port of actor.py）
actorinfo.rs ActorInfo 条目生成（Port of actorinfo.py）
pack.rs      ActorPack：SARC/AAMP/BYML 容器（Port of pack.py）
flag.rs      GameData 标志模型 + overrides 规则（Port of flag.py）
store.rs     FlagStore（Port of store.py）
texts.rs     MSBT 文本读写（使用 msyt，Port of texts.py）
util.rs      常量、find_file、gamedata/savedata 写入（Port of util.py）
settings.rs  设置（JSON，%LOCALAPPDATA%\botw_actor_tool_rs\settings.json）
data.rs      静态数据加载（data/*.json，编译期 include_str!）
```

### 性能

- `[profile.dev]` 开启 `opt-level = 1`、依赖包 `opt-level = 2`
  （egui 未优化的 debug 构建会明显卡顿；首次构建时间会变长）。
- YAML 编辑器只用 egui_code_editor 的 `CodeEditor::show`（内部按文本哈希缓存高亮
  与布局），未更改的文本不会重复排版。
- 发布版：`cargo build --release`。

### 测试

```bash
cargo test                    # 单元测试（CRC32、Flag 往返、MSBT/SARC 往返、编辑器输入）
cargo test real_dump -- --ignored --nocapture
# 上面的 ignored 测试会读取 settings.json 里的真实游戏目录，
# 加载若干代表 actor 验证 Texts 能正确读到 Name/Desc/PictureBook。
```

### 与原版的差异

- **中英文本地化**：设置 → UI Language（English / 中文），与游戏文本语言独立。
- **代码编辑器**使用 egui_code_editor（VS Code 风格）：行号、YAML 语法高亮、
  点击/光标定位准确（自定义 layouter 的 TextEdit 存在光标错位与删除问题，已弃用）。
- 文本编辑器增加了**搜索/查找**：`Ctrl+F` 或 Edit → Find 打开 VS Code 风格浮动搜索条
  （大小写不敏感、匹配高亮、n/m 计数、上一个/下一个自动滚动）。
- **Edit 菜单**：撤回（Ctrl+Z）/ 前进（Ctrl+Y）/ 查找（Ctrl+F）。
- 代码/编辑字体统一为 14px 等宽字体，保证光标与字符像素级对齐。
- 保存 Actor 时显示 **"Saving Actor …"** 模态提示。
- 编辑保存失败（YAML 解析错误等）会弹出明确的错误信息（原版静默吞掉）。
- Settings 从 INI 改为 JSON；导入 vanilla 自定义文件时不再弹窗询问而是直接导入。
- `vanilla_params.json` / `instSize_data.json` 在原版代码中未被引用，本版未包含。
- 移除了原版的 PyPI 更新检查。
- 未实现原版也未实现的功能：Flags 标签页、Elink/Profile/Slink/Xlink 编辑
  （原版在这些标签页同样只显示空白）。

### License

GNU **AGPL-3.0-or-later**（与原版一致；依赖 roead 为 GPL-3.0-or-later，
AGPLv3 与其兼容）。
