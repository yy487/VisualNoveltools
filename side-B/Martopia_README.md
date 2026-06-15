# side-B メルトピア / Martopia 本地化工具说明

本目录用于处理 side-B《メルトピア / Martopia》的 DAT 资源包、`SjisFont` 贴图字库，以及基于 `subs_cn_jp.json` 的 CP932 宿主字符映射汉化流程。

本项目不采用 GBK 直接注入。游戏文本和字库索引仍按原本的 Shift-JIS / CP932 逻辑运行；中文显示通过“中文字符 → CP932 可编码宿主字符 → 重绘宿主字符字格”的方式实现。

---

## 1. 文件组成

推荐目录结构：

```text
tools/
├─ dat_tool.py                    # DAT 解包 / 封包工具
├─ martopia_cnjp_font_tool.py     # SjisFont 字库重绘与 JSON 映射转换工具
├─ subs_cn_jp.json                # 中文字符 -> CP932 宿主字符映射表
├─ Martopia.exe                   # 用于读取 SjisFont 字形索引表
├─ dat/                           # 原始 DAT 文件目录
│  ├─ 0001.dat
│  ├─ ...
│  └─ 0248.dat
├─ unpacked/                      # 解包输出目录
├─ meta/                          # 解包时生成的回包 metadata
├─ font_in/                       # 原始四张 SjisFont PNG
├─ font_out/                      # 重绘后的四张 SjisFont PNG
├─ json_trans/                    # 翻译后的 UTF-8 JSON
├─ json_cp932/                    # 转换为 CP932 宿主字符后的 JSON
└─ dat_new/                       # 回包后的 DAT 文件
```

---

## 2. DAT 容器格式

DAT 使用 AttacheCase v2 变体，外层 magic 被处理过。当前确认格式为：

```text
算法: Rijndael-256/256 CBC
block size: 32 bytes
key size: 32 bytes
rounds: 14
payload: zlib
key: struct.pack("<I", 0x0047544F) + 28 bytes zero
```

文件布局：

```text
0x00..0x03          subver = 0x00030006
0x04..0x13          16 字节 magic 占位，通常为 FF*16 或 00*16
0x14..0x17          task_code = 105
0x18..0x1B          data_version = 1
0x1C..0x1F          TOC 密文字节数，32 字节对齐
0x20..0x3F          TOC IV，32 字节
0x40..              TOC 密文
TOC 后 32 字节      Payload IV
剩余                Payload 密文，解密后为 zlib 数据
```

注意：

- 不需要从游戏进程提取 key。
- 不需要 `conf.dat`。
- 不要使用旧版 `extract_key.py` / `dat_unpacker.py` 的 AES-256-CBC / Lua bytecode 假设。
- 回包后 DAT 不要求与原文件 byte-exact，因为 IV 会重新生成；应以 `selftest` 的内部文件一致性作为校验标准。

---

## 3. 环境要求

Python 版本建议：

```bat
python --version
```

建议 Python 3.10 或 3.11。

安装依赖：

```bat
pip install pillow numpy
```

说明：

- `pillow` 用于重绘 `SjisFont` PNG。
- `numpy` 用于加速 DAT 解密；没有 numpy 时工具会退回纯 Python，但速度较慢。

---

## 4. DAT 解包

### 4.1 解包单个 DAT

```bat
python dat_tool.py unpack dat\0248.dat unpacked\0248 --meta meta
```

输出示例：

```text
unpacked\0248\
└─ SjisFont\
   ├─ SjisFont01.png
   ├─ SjisFont01_b.png
   ├─ SjisFont02.png
   └─ SjisFont02_b.png

meta\
└─ 0248.json
```

`meta\0248.json` 保存 TOC、文件顺序、原始 header 等信息，回包时必须保留。

### 4.2 批量解包

```bat
python dat_tool.py unpack-all dat unpacked --meta meta
```

如果有非 AttacheCase DAT，工具会复制到：

```text
unpacked\_raw\
```

---

## 5. DAT 回包

### 5.1 回包单个 DAT

实际 CLI 参数顺序为：

```bat
python dat_tool.py pack <unpacked_dir> <meta_json> <out_dat>
```

示例：

```bat
python dat_tool.py pack unpacked\0248 meta\0248.json dat_new\0248.dat
```

### 5.2 批量回包

实际 CLI 参数顺序为：

```bat
python dat_tool.py pack-all <unpacked_root> <meta_dir> <out_dir>
```

示例：

```bat
python dat_tool.py pack-all unpacked meta dat_new
```

---

## 6. DAT 往返校验

修改工具或确认某个 DAT 是否支持正常回包时，先跑：

```bat
python dat_tool.py selftest dat\0248.dat
```

预期结果：

```text
[OK] roundtrip dat\0248.dat: N files match
```

`selftest` 的含义是：

```text
原始 DAT
  -> 解包到临时目录 A
  -> 重新回包成临时 DAT
  -> 再解包到临时目录 B
  -> 比较 A/B 内部文件名和字节内容是否一致
```

这个测试通过，说明解包/封包结构基本正确。

---

## 7. SjisFont 字库汉化原理

游戏的 `SjisFont` 不是系统字体，而是 32×32 的贴图字库。运行时根据 CP932 双字节字符查 EXE 内置索引表，得到 glyph index，再从以下贴图页中切字格绘制：

```text
SjisFont01.png
SjisFont01_b.png
SjisFont02.png
SjisFont02_b.png
```

汉化时不直接写 GBK，而是：

```text
中文字符
  -> subs_cn_jp.json 映射到 CP932 宿主字符
  -> 注入后的脚本文本仍可 cp932 编码
  -> 游戏按原逻辑查宿主字符字格
  -> 字库中该宿主字符字格已被重绘成中文字符
  -> 屏幕显示为中文
```

示例：

```json
{
  "这": "這",
  "说": "説",
  "们": "們"
}
```

文本注入前：

```text
这是我们说的内容
```

转换后写入脚本：

```text
這是我們説的内容
```

如果 `SjisFont` 中 “這 / 們 / 説” 的字格已重绘为 “这 / 们 / 说”，游戏显示出来就是简体中文。

---

## 8. 检查 cn_jp 映射

先检查 `subs_cn_jp.json` 中的宿主字符是否都能在游戏字库索引表中找到：

```bat
python martopia_cnjp_font_tool.py check-map Martopia.exe subs_cn_jp.json
```

预期：

```text
[check-map] total=3018 page01=1326 page02=1692 missing=0 bad_cp932=0 duplicate_host=0
```

字段含义：

```text
total          映射总数
page01         落在 SjisFont01.png 的字数
page02         落在 SjisFont02.png 的字数
missing        宿主字符无法在 EXE 字库索引表中定位
bad_cp932      宿主字符不能 cp932 编码
duplicate_host 多个中文映射到同一个宿主字符
```

要求：

```text
missing = 0
bad_cp932 = 0
duplicate_host = 0
```

否则不要继续重绘。

---

## 9. 重绘 SjisFont

### 9.1 准备 font_in

从解包目录复制四张图到 `font_in`：

```text
font_in\
├─ SjisFont01.png
├─ SjisFont01_b.png
├─ SjisFont02.png
└─ SjisFont02_b.png
```

### 9.2 使用微软雅黑重绘

```bat
python martopia_cnjp_font_tool.py patch-font Martopia.exe font_in subs_cn_jp.json font_out --ttf C:\Windows\Fonts\msyh.ttc --size 28 --yoff -1
```

### 9.3 使用宋体重绘

```bat
python martopia_cnjp_font_tool.py patch-font Martopia.exe font_in subs_cn_jp.json font_out --ttf C:\Windows\Fonts\simsun.ttc --size 28 --yoff -1
```

输出：

```text
font_out\
├─ SjisFont01.png
├─ SjisFont01_b.png
├─ SjisFont02.png
├─ SjisFont02_b.png
└─ cnjp_font_patch_report.json
```

### 9.4 常用参数

```text
--size N        字号，默认 28
--xoff N        X 方向偏移
--yoff N        Y 方向偏移，默认 -1
--ttc-index N   TTC 字体集合索引，默认 0
--b-mode        _b 页处理方式：normal / blur / clear
--b-alpha F     _b 页透明度系数
--b-blur F      _b 页模糊半径
```

推荐先用：

```bat
--size 28 --yoff -1 --b-mode blur
```

如果字体偏上/偏下，调整：

```bat
--yoff 0
--yoff -2
```

如果字体太大/太小，调整：

```bat
--size 27
--size 29
--size 30
```

---

## 10. 替换字库并回包

假设 `0248.dat` 解出来的是 `SjisFont`：

```text
unpacked\0248\SjisFont\
```

将 `font_out` 中的四张 PNG 覆盖回去：

```text
font_out\SjisFont01.png    -> unpacked\0248\SjisFont\SjisFont01.png
font_out\SjisFont01_b.png  -> unpacked\0248\SjisFont\SjisFont01_b.png
font_out\SjisFont02.png    -> unpacked\0248\SjisFont\SjisFont02.png
font_out\SjisFont02_b.png  -> unpacked\0248\SjisFont\SjisFont02_b.png
```

然后回包：

```bat
python dat_tool.py pack unpacked\0248 meta\0248.json dat_new\0248.dat
```

最后把 `dat_new\0248.dat` 覆盖或通过补丁方式放回游戏目录。

---

## 11. 转换翻译 JSON 为 CP932 宿主字符

翻译时 JSON 里正常写中文：

```json
{
  "scr_msg": "そうだね。",
  "message": "这是我的想法。"
}
```

注入前转换：

```bat
python martopia_cnjp_font_tool.py convert-json json_trans json_cp932 subs_cn_jp.json --strict
```

转换后：

```json
{
  "scr_msg": "そうだね。",
  "message": "這是我的想法。"
}
```

只转换以下字段：

```text
message
message_parts
```

默认不转换：

```text
scr_msg
```

原因：`scr_msg` 是原文校验字段，不应被修改。

如果人名也需要转换：

```bat
python martopia_cnjp_font_tool.py convert-json json_trans json_cp932 subs_cn_jp.json --fields message,message_parts,name --strict
```

`--strict` 的作用：

```text
发现无法 cp932 编码、且没有写入 subs_cn_jp.json 的字符时直接报错。
```

遇到报错时，把缺字加入 `subs_cn_jp.json`，重新执行：

```bat
python martopia_cnjp_font_tool.py check-map Martopia.exe subs_cn_jp.json
python martopia_cnjp_font_tool.py patch-font Martopia.exe font_in subs_cn_jp.json font_out --ttf C:\Windows\Fonts\msyh.ttc --size 28 --yoff -1
python martopia_cnjp_font_tool.py convert-json json_trans json_cp932 subs_cn_jp.json --strict
```

---

## 12. 完整推荐流程

### 12.1 解包 SjisFont DAT

```bat
python dat_tool.py unpack dat\0248.dat unpacked\0248 --meta meta
```

### 12.2 准备字库输入目录

```bat
mkdir font_in
copy unpacked\0248\SjisFont\SjisFont01.png font_in\
copy unpacked\0248\SjisFont\SjisFont01_b.png font_in\
copy unpacked\0248\SjisFont\SjisFont02.png font_in\
copy unpacked\0248\SjisFont\SjisFont02_b.png font_in\
```

### 12.3 检查映射

```bat
python martopia_cnjp_font_tool.py check-map Martopia.exe subs_cn_jp.json
```

### 12.4 重绘字库

```bat
python martopia_cnjp_font_tool.py patch-font Martopia.exe font_in subs_cn_jp.json font_out --ttf C:\Windows\Fonts\msyh.ttc --size 28 --yoff -1
```

### 12.5 覆盖回解包目录

```bat
copy /Y font_out\SjisFont01.png unpacked\0248\SjisFont\SjisFont01.png
copy /Y font_out\SjisFont01_b.png unpacked\0248\SjisFont\SjisFont01_b.png
copy /Y font_out\SjisFont02.png unpacked\0248\SjisFont\SjisFont02.png
copy /Y font_out\SjisFont02_b.png unpacked\0248\SjisFont\SjisFont02_b.png
```

### 12.6 回包

```bat
python dat_tool.py pack unpacked\0248 meta\0248.json dat_new\0248.dat
```

### 12.7 转换翻译 JSON

```bat
python martopia_cnjp_font_tool.py convert-json json_trans json_cp932 subs_cn_jp.json --strict
```

### 12.8 注入脚本

使用脚本注入器时，输入目录应使用 `json_cp932`，不要直接使用 `json_trans`。

---

## 13. 注意事项

### 13.1 不要直接改 GBK

游戏字库查表是 CP932 / Shift-JIS 宿主逻辑。直接把文本改成 GBK 会导致大量中文字符无法被原字库索引表识别。

### 13.2 不要修改 scr_msg

`scr_msg` 只用于定位和校验。翻译、映射转换、注入都应以 `message` 或 `message_parts` 为准。

### 13.3 不要只改一张 PNG

当前映射同时使用 `SjisFont01.png` 与 `SjisFont02.png`。如果只重绘一张，另一页上的宿主字会仍然显示原日文字形。

### 13.4 `_b` 页也要处理

`SjisFont01_b.png` 与 `SjisFont02_b.png` 可能用于阴影、描边、备用绘制或特殊状态。建议始终同步处理。

### 13.5 回包前保留 meta

`meta/*.json` 是回包必须文件。没有 meta 时无法可靠重建 TOC 和文件顺序。

### 13.6 先 selftest 再实机

改动解包/回包代码后，先执行：

```bat
python dat_tool.py selftest dat\0248.dat
```

通过后再实机测试。

---

## 14. 常见问题

### Q1：为什么明明重绘了字库，游戏里还是显示繁体/日文？

检查三点：

```text
1. 注入用的是不是 json_cp932，而不是 json_trans。
2. 字库 DAT 是否真的被回包并替换进游戏目录。
3. 该中文字符是否存在于 subs_cn_jp.json，并且 check-map 没有 missing。
```

### Q2：convert-json 报 unmapped non-cp932 chars 怎么办？

说明译文中有不能 cp932 编码、且没有映射的字符。把该字符加入 `subs_cn_jp.json`：

```json
{
  "新缺字": "某个可用宿主字"
}
```

然后重新：

```bat
check-map
patch-font
convert-json
```

### Q3：怎么选择宿主字符？

原则：

```text
1. 宿主字符必须能 cp932 编码。
2. 宿主字符必须能被 EXE 字库索引表定位。
3. 一个宿主字符只能给一个中文字符使用。
4. 优先使用原文本中不常出现、不需要保留原义的字符。
```

选择后必须运行：

```bat
python martopia_cnjp_font_tool.py check-map Martopia.exe subs_cn_jp.json
```

### Q4：DAT 回包后大小不一样正常吗？

正常。回包会重新压缩并重新生成 IV，因此外层 DAT 大小和字节通常不会与原文件完全一致。判断标准是：

```text
selftest 内部文件一致
实机能正常读取
```

### Q5：字体太糊或太小怎么办？

调整：

```bat
--size
--xoff
--yoff
--b-mode
--b-alpha
--b-blur
```

推荐先微调 `--size` 和 `--yoff`。

---

## 15. 当前已知限制

```text
1. DAT 回包不是 byte-exact 重建。
2. 依赖 Martopia.exe 中固定的 SjisFont 索引表。
3. subs_cn_jp.json 只覆盖当前映射表内字符；新增译文缺字需要补映射。
4. 该 README 只覆盖 DAT / SjisFont / JSON 映射流程，不包含剧本文本指令流分析。
5. 若游戏版本不同，DAT 编号和 Resource 映射可能不同，应以实际解包 TOC 为准。
```
