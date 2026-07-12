# AI5WIN / 百鬼本地化工具包

本目录集中保存《百鬼》当前有效的 MES 文本工具、`DATA.MR` 字库重绘工具和对应分析文档。翻译交换格式统一为 UTF-8 JSON；`scr_msg` 仅用于原文校验，翻译只写入 `message`。

## 目录

```text
mes_vm_rs/     Rust MES 提取、校验、变长注入和 VM/偏移修复工具
font_tools/    DATA.MR 解压、检查、全量重绘、重压缩和 ARC 解封包工具
docs/          MES VM/opcode、偏移表、字库结构和 EXE 缓冲区分析
```

## MES 文本

发布版：

```powershell
.\mes_vm_rs\mes_vm_rs.exe export-dir <mes目录> <json目录>
.\mes_vm_rs\mes_vm_rs.exe import-dir <mes目录> <json目录> <输出mes目录>
```

JSON 条目保留位置和校验元数据。`name` 可翻译；内联人名条目使用原始内联人名校验源字节，再以当前 `name + message` 写回。

源码位于 `mes_vm_rs/src/main.rs`，可在该目录执行：

```powershell
cargo build --release
```

## DATA.MR 字库

`font_tools/data_mr_tool.py` 支持 `info`、`unpack`、`pack`、`sheet` 和 `redraw`。默认按 CP932 字表处理，且只重绘已确认的主字形区 section 3。section 4/5 虽然尺寸类似字形记录，但参与运行期资源查找，必须逐字节保留。

典型全量重绘：

```powershell
python .\font_tools\data_mr_tool.py redraw `
  .\font_tools\assets\DATA_MR_chs_full.MR `
  .\font_tools\assets\subs_cn_jp.json `
  .\font_tools\assets\alyce_humming.ttf `
  .\font_tools\output\DATA_MR_redrawn.MR
```

工具会优先调用同目录的 `baigui_lzss_pack.exe`；缺失时回退到 Python 压缩器。`arc_tool.py` 用于把生成的 `DATA.MR` 放回 `data.arc`。

依赖：Python 3、Pillow。`fontTools` 不是这套 24x24 `DATA.MR` 工具的必需依赖。

## 文档

- `docs/mes_vm_analysis.md`：文本流、`0x01` 字符串、`0x0B` 显示边界、VM 参数和选择项分析。
- `docs/mes_vm_offset_table_notes.md`：MES 文件头 entry table、变长注入和偏移重建依据。
- `docs/data_mr_cache_patch.md`：字库结构、扩容后 EXE 缓冲区边界和 patch 依据。
- `docs/fontfix_windowtest_notes.md`：字库构建产物、标点处理和相关实机排查记录。

## 约束

- 默认文本编码为 CP932。
- 不修改 `scr_msg`；只修改 `message`，多段文本按工具导出的分段元数据处理。
- 注入前必须通过源文本校验；变长注入会重建已确认的文件头 entry table。
- 不按字节特征猜测并改写 VM 操作数。`F3` 已确认是变量表索引前缀，不是跳转；尚未结构化建模的 VM 内部地址不会被声称已修正。
- 未知 VM 数据保持原样；无修改回环应以解压后的 MES 字节一致为准。
