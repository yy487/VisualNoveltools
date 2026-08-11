# diannao_ks

`diannao_ks` 是面向游戏 `scenario` 目录的 KAG `.ks` 状态机式正文
提取/注入工具。源脚本保持无 BOM CP932，翻译交换格式仅使用 UTF-8 JSON。

工具逐文件输出 JSON，不覆盖原脚本或已有输出目录。注入前会重新解析全部源脚本，
校验条目顺序、原文、偏移和显示前后缀，然后复制完整源目录并应用变长文本补丁。

## 构建

```powershell
cargo build --release --offline --bins
```

可执行文件为 `target\release\diannao_ks.exe`。

## 拖放

把整个 `scenario` 目录拖到 EXE：

```text
scenario\ -> scenario_json\
```

翻译后，把含 `audit.json` 的 `scenario_json` 拖到 EXE：

```text
scenario_json\ -> scenario_injected\
```

输出目录已存在时工具会停止，不会覆盖。

## 命令行

```powershell
.\target\release\diannao_ks.exe extract `
  "<SCENARIO_DIR>" `
  -o .\analysis_output\scenario_json

.\target\release\diannao_ks.exe inject `
  .\analysis_output\scenario_json `
  --source "<SCENARIO_DIR>" `
  -o .\analysis_output\scenario_injected
```

运行 `diannao_ks.exe --help` 可查看完整入口。拖放和命令行使用同一实现。

## JSON 合同

正文条目示意：

```json
{
  "_file": "booth_a_01.ks",
  "_index": 12,
  "_line": 27,
  "_end_line": 27,
  "_offset": 1018,
  "_size": 26,
  "_type": "body",
  "_encoding": "cp932",
  "_boundary": "p",
  "_source_kind": "body_part",
  "_display_prefix": "──",
  "scr_msg": "──何だ、何をしているんだ！？",
  "message": "──何だ、何をしているんだ！？"
}
```

字段规则：

- 每个连续可见源文本片段输出为一条独立正文；不再输出 `scr_msg_parts`、
  `message_parts` 或 `_controls`。
- `scr_msg` 是不可修改的原文，只用于定位和校验；译文直接写入同一条的 `message`。
- `_offset/_size` 只覆盖该条源文本字节，不包括标签、命令、注释或物理 CRLF。
- `_file`、`_index`、`_offset` 等所有下划线字段不可修改。
- `_display_prefix` / `_display_suffix` 是宏的只读显示形式；编辑 `message` 时必须
  原样保留，注入只写除去前后缀后的译文。
- 本项目没有姓名宏，不输出 `name` 或 `_scr_name`。
- `[seladd text="..."]` 输出为 `_type: "choice"`，同样直接编辑 `message`。

每个源 `.ks` 都有对应的 `.ks.json`；无正文文件也会输出空数组。`audit.json`
保存源目录、统计和诊断。

## 状态机边界

正文从结构确认的可见文本开始，以下命令结束当前正文：

- `@p`：常规正文边界。
- `[select]`：选项块结束；为避免与选项属性补丁重叠，选项前的提示正文在首个
  `[seladd]` 处以 `select` 边界关闭。
- `@cm`：条件正文未经过 `@p` 时的边界。
- `[jump]` / `@jump`：跳转前正文边界。
- EOF：兜底边界，会产生可定位 warning。

宏定义和 `[iscript]` / `@iscript` 脚本块不作为正文扫描。注释、标签、音效、
背景、等待和条件命令都在正文补丁范围之外，不会被误当译文或被回注覆盖。

## 控制结构

控制结构不进入正文条目的源字节跨度：

- `[dash]正文` 显示为只读前缀 `──正文`；页尾 `正文[dash]` 显示为只读后缀
  `正文──`。回注只改正文，原始 `[dash]` 宏不动。
- `[emb exp="..."]正文` 显示为 `{{emb:表达式}}正文`；动态表达式不可编辑。
- `[r]`、`@r`、`[l]`、`[wait]`、`[ruby]`、`[font]`、`[resetfont]` 不向 JSON
  正文加入换行或标签字符，并按原始字节留在相邻独立条目之间。
- 正文内部的 `@if/@else/@endif`、音效、背景切换、等待、注释以及 CRLF
  均保持原始位置和字节。
- 物理 CRLF 不进入正文；源脚本没有行尾 `\`。

修改正文 `message` 时禁止加入物理 CR/LF、NUL 或 KAG 方括号标签。需要的显示
换行仍由源脚本已有的 `[r]` / `@r` 控制提供。

## 注入校验

写输出前会完成全部校验：

- `audit.json` 中的源目录必须与 `--source` 完全一致。
- JSON 文件、条目数量、顺序和所有不可变字段必须匹配重新解析的源脚本。
- `scr_msg`、`_display_prefix` 和 `_display_suffix` 不允许变化。
- 选项译文不能破坏原属性引号。
- 每个译文严格编码为 CP932；失败时列出不可编码 Unicode 字符。
- 重复原文依靠 `_file + _index + scr_msg` 定位，不做全局替换。
- 补丁按偏移倒序应用，重叠或越界会在写目录前失败。
- 完整源树先复制到临时目录，成功后再改名为最终输出。

明文 `.ks` 可做变长注入；工具不修改标签目标、脚本标签或外层归档表。

## 已验证样本

当前样本为 89 个无 BOM CP932 `.ks`，共 745,346 字节，全部使用 CRLF 且通过
CP932 字节回环。首轮提取结果：

```text
body                10153
choice                 14
total               10167
tracked controls    16520
dash                  220
wait                   92
ruby                    8
font/resetfont          6 / 6
emb                     1
l/r/@r             8171 / 7272 / 744
warnings                0
```

零修改注入报告 `patched=0`、`unchanged=10167`；源与输出的 89 个相对路径和
SHA-256 全部一致。真实修改验收分别修改了含 `dash`、`wait`、`ruby`、`font`、
动态 `emb` 的正文及一个选项；重新提取的条目数、专用控制统计和原始命令均保持
一致。

## 限制

- 当前只处理明文 `.ks`，不解包或封装外层归档。
- 精确 KAG/吉里吉里版本尚未由运行时文件确认；工具能力以这批脚本结构为准。
- 条件分支和跨物理行正文按源顺序输出为独立条目；JSON 不模拟运行时分支选择。
- 动态 `[emb]` 只显示表达式占位，运行时结果无法在静态 JSON 中展开。
- 源脚本保持 CP932，不自动转换为 UTF-8/UTF-16，也不处理中文字库映射。
