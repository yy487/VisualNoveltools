# 《J 的悲剧》D88 汉化工具

面向 PC-98 版 `J no Higeki - Las Vegas Renzoku Satsujin Jiken` 的独立文本提取、注入、D88 回封与 NP2 字库工具。

## 操作

- `export`：从现有解包工作区导出 `JACK_C.ATX.json`、`JACK_C.ADW.json` 和 `JACK.EXE.ui.json`。
- `build`：读取翻译工作区目录或所选单个 JSON，生成完整 D88、`font.bmp`、字槽映射和构建报告。
- `unpack`：复用通用 ENIX-DOS D88 模块重新解包原始镜像。

直接运行 exe 会显示中文操作面板；也可用 `--help` 查看等价命令行。

## 翻译规则

只编辑 JSON 中的 `message`。`scr_msg`、所有以下划线开头的字段、ADW 别名和清单字段均为来源校验值，不得修改。

- ATX 的 `logical_entries` 是按游戏实际输出顺序合并的整句；其中顶层 `message` 是唯一的翻译字段，优先编辑它。
- `logical_entries[].parts[].text` 仅表示原始分段，供定位和核对使用，不是第二份翻译字段。构建时若整句被修改，会按顶层整句回写并合并原来的 `?`/`+` 链。

- ATX：1,779 项剧情文本、MSG 和物品名。注入保留全部指令、注释、缺分号、原有拼写和多余引号，只替换引号内正文。
- ADW：306 项右侧命令菜单显示词，输入别名保持原样。
- EXE：`Select Menu:` 固定字段，编码后最多 14 字节。
- 半角 ASCII 会自动转为全角；禁止在正文中使用双引号、换行、NUL 或控制字符。

ATX 变长后，工具会重建 `JACK_C.MSG` 与 `JACK_C.FRM` 索引，并复核读回结果。没有修改的构建必须保持五个目标资源、两张 D88 和内置 `font.bmp` 逐字节一致。

## 输出与边界

构建成品写入独立目录：

- `d88/`：完整回封镜像；
- `font.bmp`：NP2/兼容模拟器 PC-98 字库；
- `font_mapping.json`：中文字槽映射；
- `build-report.json`：来源、数量和校验摘要。

书页、标题和场景画面属于 `TITLE*.GAS` / `JACK_GRA.*` 图像资源，不是独立文本，不在本工具的文本提取注入范围。
