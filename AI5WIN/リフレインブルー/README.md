# Refrain Blue / AI5WIN-like 文本与字体工具包 v2

本包针对当前样本：

- `MES.ARC`
- `AI5WIN.exe / rp.EXE.c`
- `FONT*.FNT`
- `FONT*.PAL`
- `FONT.TBL`

v2 版把工作流明确改成：**先解包 `MES.ARC` 得到散的 `.MES` 文件，再按“一个 MES 对应一个 JSON”提取/注入，最后重新封回 `MES.ARC`**。`MES.ARC` 不是一个文本池；它是文件包，直接从 ARC 提取只是工具内部遍历每个 entry，不应把整个 ARC 当连续脚本文本扫描。

---

## 1. 文件列表

```text
rp_ai5win_workflow_v2/
├─ rp_arc.py              # MES.ARC 读写共用模块
├─ rp_arc_tool.py         # MES.ARC list/unpack/pack 命令
├─ rp_workflow.py         # 推荐工作流封装：unpack/extract/inject/pack
├─ rp_mes_common.py       # MES 文本扫描、CP932 编解码、非等长 patch 共用逻辑
├─ rp_mes_extract.py      # 从散 MES 目录/单 MES/ARC 提取 JSON
├─ rp_mes_inject.py       # 将 JSON 注入回散 MES 目录/单 MES/ARC
├─ rp_mes_find.py         # 在 MES.ARC/MES 中搜索 CP932 字符串，用于验证
├─ rp_font_common.py      # FNT/PAL/TBL 共用逻辑
├─ rp_font_tbl.py         # FONT.TBL 解析/生成辅助
├─ rp_font_dump.py        # FNT/PAL 导出 PNG 网格
├─ rp_font_build.py       # PNG 网格 + 字符表生成 FNT/PAL/TBL
├─ rp_font_make_charset.py# 从翻译 JSON 生成字符集
├─ rp_font_patch_exe.py   # 追加扩展字库到 EXE 新 section 并 patch 指针的实验工具
├─ requirements.txt       # Pillow 依赖
└─ README.md
```

---

## 2. 推荐文本工作流

### 2.1 解包 `MES.ARC`

```bat
python rp_workflow.py unpack MES.ARC mes_raw
```

也可以用底层工具：

```bat
python rp_arc_tool.py unpack MES.ARC mes_raw
```

输出示例：

```text
mes_raw/02.MES
mes_raw/03.MES
mes_raw/04.MES
...
```

### 2.2 一个 MES 对应一个 JSON 提取

```bat
python rp_workflow.py extract mes_raw json
```

输出示例：

```text
json/02.json
json/03.json
json/04.json
...
```

每条 JSON 只有纯文本，不输出 `name`：

```json
{
  "scr_msg": "目の前にあるのは、波打ち際の光景。",
  "message": "目の前にあるのは、波打ち際の光景。",
  "_file": "02.MES",
  "_index": 0,
  "_offset": 7012,
  "_text_offset": 7013,
  "_end": 7048,
  "_opcode": "01",
  "_type": "message",
  "_raw_hex": "01 ... 00"
}
```

### 2.3 修改 JSON

只改 `message` 字段，不要改：

```text
scr_msg
_file
_index
_offset
_text_offset
_end
_opcode
_raw_hex
```

`message` 目前仍需能编码为 CP932。中文显示需要先通过字体/字符映射，把中文映射到 CP932 可编码字符槽位。

### 2.4 注入回散 MES 目录

```bat
python rp_workflow.py inject mes_raw json mes_new
```

强制所有修改都使用 EOF 跳板：

```bat
python rp_workflow.py inject mes_raw json mes_new --force-jump
```

### 2.5 重新封成 ARC

```bat
python rp_workflow.py pack mes_new MES_chs.ARC --base-arc MES.ARC
```

`--base-arc` 用来保持原始文件顺序。

### 2.6 两条合并命令

第一次提取：

```bat
python rp_workflow.py unpack-extract MES.ARC work
```

这会生成：

```text
work/mes_raw/*.MES
work/json/*.json
```

翻译后注入并封包：

```bat
python rp_workflow.py inject-pack MES.ARC work MES_chs.ARC
```

这会生成：

```text
work/mes_new/*.MES
MES_chs.ARC
```

---

## 3. MES.ARC 格式分析

`MES.ARC` 是文件包，不是文本池。

头部：

```c
uint32 file_count;
```

当前样本：

```text
file_count = 100
```

每个文件表项 0x1C 字节：

```c
struct Entry {
    uint8  name_xored[20];
    uint32 size_xored;
    uint32 offset_xored;
};
```

解码方式：

```text
name   = each_byte ^ 0x55，ASCII，0 结尾
size   = size_xored   ^ 0xAA55AA55
offset = offset_xored ^ 0x55AA55AA
```

文件数据紧跟在文件表之后。底层命令：

```bat
python rp_arc_tool.py list MES.ARC
python rp_arc_tool.py unpack MES.ARC mes_raw
python rp_arc_tool.py pack mes_new MES_chs.ARC --base MES.ARC
```

---

## 4. MES 文本结构分析

当前原始 `MES.ARC` 内运行时脚本文本是普通 CP932 字节串：

```text
01 <cp932 zero-terminated string> 00
```

例：

```text
01 96 DA 82 CC 91 4F ... 81 42 00
```

解码为：

```text
目の前にあるのは、波打ち際の光景。
```

验证命令：

```bat
python rp_mes_find.py MES.ARC "目の前にあるのは、波打ち際の光景。"
```

应能定位到：

```text
02.MES offset 0x1B65
```

其中 `0x1B64` 是 opcode `01`，`0x1B65` 是文本起始。

### 4.1 没有 name 行

本作没有独立 name 行。颜色、位置、括号都不作为 name 判断依据。JSON 不输出 `name`。

### 4.2 选择支

选择项文本和点击选择后正文复述文本在当前字节层都表现为 `01 + CP932 string + 00`。因此工具先统一提取为 `_type = "message"`，不去重。

例如同一句可能出现两次：

```text
菜单选项：01 「いや…時間まで待ってることにするよ」 00
分支正文：01 「いや…時間まで待ってることにするよ」 00
```

这两处必须作为两条 JSON 处理，因为 offset 不同。

### 4.3 ruby / furigana 辅助文本

发现结构：

```text
0B 15 FF 01 <cp932 string> 00
```

常见内容是短假名，默认跳过。检查时可加：

```bat
python rp_workflow.py extract mes_raw json --include-ruby
```

---

## 5. 非等长注入方案

采用 EOF 跳板法，不整体平移旧脚本。

原始：

```text
01 old_text 00
```

变长后原位置改成：

```text
0A append_offset
00 00 00 ...
```

EOF 追加：

```text
01 new_text 00
0A return_offset
```

执行流程：

```text
原文本位置 -> 跳到 EOF 新文本块 -> 显示新文本 -> 跳回原文本结束后的下一条命令
```

优势：

```text
1. 不移动原有脚本数据。
2. 不需要全局修正旧跳转。
3. 每个 MES 独立追加自己的新文本块。
4. 适合“散 MES 文件 -> 散 JSON -> 散 MES 文件 -> 重封包”的工作流。
```

限制：

```text
1. 原文本命令跨度必须至少 5 字节，才能放 0A + u32 跳板。
2. 当前只处理 opcode 01 的 CP932 文本。
3. 如果某些系统字符串不是 01 结构，需要后续补 opcode。
4. message 需要 CP932 可编码；中文需要配合字体映射。
```

---

## 6. 字体格式分析

当前字体是自带位图字体，不是 GDI/TTF 实时渲染。

### 6.1 FONT.TBL

`FONT.TBL` 是反序 CP932 双字节字符表：

```text
文件中存：low high
逻辑字符：high low
```

末尾 `00 00` 结束。

### 6.2 FONT.FNT

主字体字形：

```text
24 x 34 像素
4bpp packed
1 byte = 高 4bit 像素 + 低 4bit 像素
每字形 24 * 34 / 2 = 408 bytes
```

### 6.3 FONT.PAL

```text
16 色 RGB
16 * 3 = 48 bytes
0 号色透明/跳过
```

---

## 7. 字体工具命令

导出 PNG 网格：

```bat
python rp_font_dump.py FONT.FNT FONT.PAL FONT.TBL font_grid.png
```

从 PNG 网格重建：

```bat
python rp_font_build.py font_grid.png char_list.txt FONT_new.FNT FONT_new.PAL FONT_new.TBL
```

从翻译 JSON 统计字符：

```bat
python rp_font_make_charset.py json charset.txt
```

EXE 追加字库 patch 工具仍是实验性质，使用前必须备份 EXE：

```bat
python rp_font_patch_exe.py AI5WIN.exe AI5WIN_patched.exe FONT_new.TBL FONT_new.FNT
```

---

## 8. 当前可能存在的问题

```text
1. 当前文本 scanner 是实用型扫描器，不是完整 VM 反汇编器。
2. 选择支目前按纯 01 文本处理，不额外标注 choice。
3. ruby/furigana 默认跳过，可能存在极少数误判。
4. 只处理 01 + CP932 + 00 文本；若后续发现 02/DF/EF 等文本结构，需要继续补。
5. EOF 跳板法依赖 0A 为 raw absolute offset；该点已和当前样本表现相符，但仍建议实机多处分支测试。
6. 中文注入必须配合 CP932 兼容映射和字体重绘，否则编码阶段会失败。
7. EXE 字库 patch 涉及 PE section 和指针 patch，是实验工具，不建议一次性覆盖原 EXE。
```

---

## 9. 推荐目录结构

```text
project/
├─ MES.ARC
├─ AI5WIN.exe
├─ FONT.FNT
├─ FONT.PAL
├─ FONT.TBL
├─ tools/
│  └─ rp_ai5win_workflow_v2/*.py
└─ work/
   ├─ mes_raw/
   ├─ json/
   └─ mes_new/
```

完整命令：

```bat
cd tools\rp_ai5win_workflow_v2
python rp_workflow.py unpack-extract ..\..\MES.ARC ..\..\work
rem 修改 ..\..\work\json\*.json
python rp_workflow.py inject-pack ..\..\MES.ARC ..\..\work ..\..\MES_chs.ARC
```
