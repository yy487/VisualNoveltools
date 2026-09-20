# vn-cli：共享 Rust 命令行面板

项目内可直接复用的本地 crate，核心位于 [src/cli_panel.rs](src/cli_panel.rs)，不依赖第三方库。
实现已认可的 [Rust 工具流规范](../../../rules/rust-cli-panel.md)。
后续工具通过依赖接入，菜单修复集中维护，无需复制面板源码。

## 运行示例

在本目录构建并运行：

```powershell
cargo build --release --offline --bins
.\target\release\panel_demo.exe
```

已验证的 Windows 示例也保存在 [release/panel_demo.exe](release/panel_demo.exe)，可双击或拖入文件。
示例的真实能力是检查文件元数据、将单个文件复制到独立位置；它不解析 JSON 内容，
也不代表任何现有游戏已接入解包、回注或字库流程。

```text
共享 CLI 面板示例 / 主菜单
[1] 检查输入文件
[2] 单项操作
    → 复制文件（接入示例）
[0] 退出
```

选择操作后，各参数按编号单独修改；`P` 预检、`R` 执行、`C` 显示等价命令、`0` 返回、`Q` 退出。
写入操作在本次预检通过后要求确认；空行取消执行。执行失败留在参数页，成功返回菜单。
参数在当前面板会话保留，退出程序后不写入配置文件。
编辑时空行保留，`:clear` 清除，`:cancel` 取消本次编辑，`:quit` 退出；路径列表每行一个路径，空行保存。
EOF 在任意输入位置退出；未完成的多路径编辑不会保存。

完整命令执行一次，不询问输入，失败返回非零退出码。例如：

```powershell
.\release\panel_demo.exe --help
.\release\panel_demo.exe inspect --source '.\原始文件.bin' --translation '..\译文\章节一.json' --translation '..\译文\章节二.json'
.\release\panel_demo.exe copy --source '.\原始文件.bin' --output '..\成品\文件.bin'
```

复制示例要求输出父目录已存在，默认拒绝已有输出，显式 `--overwrite` 才允许替换。
无参数或仅路径进入面板，拖入路径只预填；不存在且不含路径分隔符的名字用 `-- <路径>` 明确指定。
布尔参数支持 `--flag`、`--flag=false`；路径列表重复使用同名参数。
`--unset=字段名` 清除默认值，`--translation=` 表示空路径列表；必填参数不能清空后执行。
`C` 输出的复制命令针对 **PowerShell 7.3+，Standard 原生命令参数模式**；Windows PowerShell 5.1 的参数转义不受支持。
在 PowerShell 7.3+ 中如需切换模式，先执行 `$PSNativeCommandArgumentPassing = 'Standard'`。

## 接入现有工具

同级 Rust 工具的 `Cargo.toml` 增加：

```toml
[dependencies]
vn-cli = { path = "../vn_cli" }
```

只注册当前已经实现的操作。研究初期注册 `unpack`；实现提取后增加 `extract`；
回注、字库和封碟确认后，再用游戏的 `workflow.rs` 组合 `export`、`import` 两项主操作。
设 `OperationSpec.primary = false` 的操作进入“单项操作”，仍可作为完整子命令使用。

| 接口 | 工具提供的内容 |
| --- | --- |
| `Operation::spec` | 操作 ID、名称、说明、主菜单位置、是否写入、参数字段 |
| `Operation::prefill` | 根据拖入路径提出可编辑建议；不得写入文件 |
| `Operation::prepare` | 只读校验，构造本次任务的数据快照与预检报告 |
| `PreparedOperation::execute` | 消费已准备任务，调用已有业务逻辑，安全提交并返回报告 |
| `Progress::report` | 当前步骤或进度文字 |

实际接入示例见 [panel_demo.rs](src/bin/panel_demo.rs)。入口通常如下，其中 `Export` 和 `Import`
是各工具自己实现 `Operation` 的适配器：

```rust,ignore
use vn_cli::Panel;

fn run() -> vn_cli::Result<()> {
    Panel::new(
        std::env::current_exe()?.into_os_string(),
        "工具名称",
        env!("CARGO_PKG_VERSION"),
        vec![Box::new(Export), Box::new(Import)],
    )?.run_env()
}
```

主函数应打印 `run()` 的错误并返回非零退出码，参考示例。保留旧命令兼容时，由工具入口
将旧参数归一化为同一份操作和参数；共享层不猜测每个工具的旧别名。
字段支持单路径、路径列表、文本、布尔和固定选项。操作与字段 ID 使用小写字母开头的
小写字母、数字和连字符；`help`、`version`、`unset` 是保留字段名。
写入操作自动得到默认 `false` 的 `overwrite` 字段；业务层通过 `parameters.flag("overwrite")` 读取策略。

## 共享边界与写入约定

共享层集中处理交互、参数解析、会话状态、重复预检、确认、进度和结果。
`P` 的准备结果不会缓存给下一次执行，`R` 总是重新准备任务。
完整命令和面板使用同一个 `prepare` / `execute`，完整命令本身即表示执行授权。

操作在 `Preview.inputs` 中声明所有受保护输入：原盘或完整原工作区、独立译后 JSON、
原始字库和其他依赖。`Preview.outputs` 声明最终文件或目录根，不要同时列出根和子文件。
共享层检查输入与输出的路径重叠、输出彼此重叠、默认不覆盖，并在交互确认后再次检查。
路径检查解析已有路径及最近的已存在父路径，覆盖常见链接和 Windows 目录联接；
它不是任意文件系统别名或并发写入的安全隔离机制。

以下职责由游戏或格式后端实现，接入时必须验证：

- `prepare` 只读，执行使用已校验的快照；若执行重新读取输入，应校验其仍与预检一致。
- JSON 的来源标识、条目完整性、可编辑 `message`、原文和控制符校验。
- 将独立选择的部分 JSON 合入完整原始基准，未选择的源文件保持原样；输出完整目录树。
- 执行遵守覆盖选项，写前完成结构与编码验证，通过临时输出及安全提交避免留下部分成品。
- 运行期间的输出变化、硬链接等别名、多文件/多盘提交与回滚策略。
- 回注结果、字库映射、重封盘与模拟器验证。

复制示例先写同目录临时文件，Windows 下以单次文件替换提交；默认不覆盖的提交也拒绝
执行期间新出现的目标。显式覆盖会检查预检后旧目标的内容变化，但不能排除检查与替换之间
其他进程的写入；需要排他控制的正式工具应自行实现锁定或构建目录隔离。

PC88 的 KANJI1 ROM 和 PC98 的 NP2 `font.tmp` 保持独立后端，面板不包含字槽寻址、
绘字、编码或游戏 JSON 格式。无需重绘字库的工具只省略该步骤。
当前共享 crate 和示例已实现，既有游戏工具尚未批量迁移，跨进程保存参数尚未实现。

## 验证

```powershell
cargo fmt -- --check
cargo test --offline
cargo clippy --offline --all-targets -- -D warnings
cargo build --release --offline --bins
.\target\release\panel_demo.exe --help
```

测试覆盖完整命令不询问、菜单返回、参数保留、EOF、预检失效、确认取消、业务错误恢复、
独立多路径输入、清除默认值、特殊字符引用、源路径保护、覆盖策略，以及示例实际复制。
Windows 测试包含大小写与目录联接。测试临时目录自动清理。
具体构建检查记录见 [verification.json](verification.json)。
