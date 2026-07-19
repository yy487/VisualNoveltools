# Silky common Rust 工具

这是 `H:\vn-tool\silky\common` Python 工具的 Rust 移植版。用户入口收敛为四个 EXE：

```text
unpack.exe   Silky ARC -> 文件目录
repack.exe   文件目录 -> Silky ARC
extract.exe  MES -> UTF-8 JSON
inject.exe   MES + UTF-8 JSON -> 新 MES
```

MES 的 opcode 解析、字符串变换、文本块识别和重定位全部集成在库内，不再要求用户管理 `op.txt`。

## 构建

```powershell
cd "H:\vn-tool\silky\common\rust"
cargo build --release --offline --bins
```

生成位置：`target\release\unpack.exe`、`repack.exe`、`extract.exe`、`inject.exe`。

## ARC 解包与回封

```powershell
unpack.exe "Script.arc" "Script_unpacked"
repack.exe "Script_unpacked" "Script_repacked.arc"
```

省略输出参数时使用相邻的新路径：

```text
Script.arc       -> Script_unpacked\
Script_unpacked\ -> Script_unpacked_repacked.arc
```

支持的 ARC 格式：

- `silky-lzss`：4 字节小端 header size、变换文件名、每项 3 个大端 `size/unpacked_size/offset`，支持 Silky LZSS。
- `garbro-fixed`：4 字节小端 count、`0x20` 字节文件名和两个小端 `offset/size`，不压缩。
- `--format auto`：解包时结构检测；回封时优先读取 `.silky_arc_manifest.json`，无 manifest 时默认 `silky-lzss`。

常用选项：

```powershell
unpack.exe "Script.arc" --format auto --encoding cp932
repack.exe "Script_unpacked" -j 8
repack.exe "Script_unpacked" --no-compress
repack.exe "Script_unpacked" --store-all
```

`--no-compress` 只让新增文件不压缩，manifest 中原本压缩的项仍压缩。`--store-all` 明确让所有项原样存储。

## MES 提取与注入

单文件：

```powershell
extract.exe "SCENE01.MES" "SCENE01.json"
inject.exe "SCENE01.MES" "SCENE01.json" "SCENE01_injected.MES"
```

目录批处理：

```powershell
extract.exe "mes" "json" -j 8
inject.exe "mes" "json" "mes_injected" -j 8
```

目录模式只处理当前目录中匹配 `--pattern "*.MES"` 的文件，不递归。批处理会先解析并验证全部输入，全部成功后才发布输出目录。

省略输出参数时：

```text
SCENE01.MES -> SCENE01.MES.json
mes\        -> mes_json\
MES + JSON  -> SCENE01_injected.MES / mes_injected\
```

四个工具均接受正常命令行位置参数；把单个 ARC/MES/目录拖放到只需要一个输入的 EXE 上时，会使用默认输出。所有输出都拒绝覆盖已有文件或目录。

## JSON 合同

提取结果是 UTF-8、无 BOM、LF、稳定缩进的 JSON 数组：

```json
[
  {
    "_file": "SCENE01.MES",
    "_index": 0,
    "name": "リルカ",
    "_scr_name": "リルカ",
    "scr_msg": "原始正文\\n第二段",
    "message": "原始正文\\n第二段",
    "_inst_offset": 1234,
    "_type": "dialogue",
    "_opcode": "STR_CRYPT",
    "_encoding": "cp932",
    "_policy": "relocate"
  }
]
```

规则：

- `scr_msg` 是不可修改的原文，只用于 `_file + _index + scr_msg` 定位和校验。
- `message` 是正文唯一写回字段；提取时 `message == scr_msg`。
- 有确认的可写名字块时输出 `name` 和 `_scr_name`；改名只改 `name`，`_scr_name` 保留原名用于注入前校验。
- 为兼容旧 Python JSON，没有 `_scr_name` 的 `name` 仍可写回，但会报告 warning。
- `_type` 当前为 `dialogue` 或 `monologue`；没有确认说话人时不输出 `name`。
- `_inst_offset` 是原 MES 中首个正文指令的文件偏移；注入仍以 `_file + _index + scr_msg` 为主，不做全局字符串替换。

注入会拒绝：`scr_msg`/`_scr_name` 不匹配、重复或越界 `_index`、错误 `_file`、NUL、真实 CR/LF、损坏的字面 `\n` 控制数量、不可按目标编码表示的字符，以及损坏的 MES/JSON。任何失败都不会生成目标文件或半成品目录。

## 控制符、ruby 与变长

- `TO_NEW_STRING [0]` 在 JSON 中表示为字面两个字符 `\n`，不是实际换行。注入必须保持其数量。
- 已确认的 ruby 结构不会把 reading 暴露给译者。每次 `inject` 都按已通过实机验证的旧 Python 策略，把 reading 槽写成等字符数全角空格（视觉上删除全部注音），保留 separator/控制结构，并把 `message` 写入 base 槽。
- 无 ruby 的未修改正文不重写字符串槽，保持 MES byte-exact。含 ruby 的 unchanged inject 会因“删除注音”项目策略产生预期字节变化；底层 `parse -> rebuild` 仍保持 byte-exact。
- MES 通过整体重建支持正文和可写名字变长/变短。重建会修正 header 两组 offset 及 `0x14/0x15/0x16/0x1B` 的代码相对目标。
- 工具不会新增、删除或重排指令；未知字节作为 opaque raw node 原样保留。

## 验证

```powershell
cargo fmt -- --check
cargo test --offline
cargo clippy --offline --all-targets -- -D warnings
cargo build --release --offline --bins
```

自动测试覆盖 LZSS Python 参考向量、LZSS 截断、两种 ARC 内容回环、路径穿越/重叠拒绝、MES 底层零修改 byte-exact、变长 offset/jump 重定位、正文/名字修改后重提取、每次注入清空 ruby reading、字面 `\n`、CP932 编码失败和 `scr_msg` 不一致拒绝。

## 已知限制

- opcode 集来自原 `silky_op.py`；没有在本目录提供真实 MES、ARC 或宿主程序，因此 VM 指令语义没有做新的运行时确认。
- 角色名只识别项目原有的两个 `PUSH_STR/PUSH/.../0x18` 结构和三个 special PUSH 值。发现新变体时应先展示真实样本并更新项目档案，不能直接扩大启发式规则。
- ruby 只支持原 Python 已实现的 `TO_NEW_STRING[1] ... RETURN ... STR_CRYPT[base]` 结构；提取时异常结构整块跳过并 warning，注入时为避免残留部分注音会直接失败且不产出文件。
- 无法按指定编码解码的 MES 字符串会保留原始字节，但所在翻译块不会导出。
- ARC 回封保证内部文件、顺序和载荷可验证；压缩结果会与当前 LZSS 实现一致，但不承诺含未知间隙/尾随填充的外层 ARC byte-exact。
- `garbro-fixed` 文件名最多 31 编码字节；`silky-lzss` 文件名最多 255 编码字节。

格式证据和未确认项见 [ENGINE_PROFILE.md](ENGINE_PROFILE.md) 与 [VM_ANALYSIS.md](VM_ANALYSIS.md)。
