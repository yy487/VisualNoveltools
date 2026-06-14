# ACTGS

这引擎脚本大多数都是明文，garbro 能打开，需要的密钥可用 `scr_crypto` 提取。脚本提取/注入支持新版与旧版 ACTGS 引擎。

> **更新 (2026-06-15):** `scr_crypto.py` 密钥搜索与档案重建已修复。详见下文「密钥搜索」和「零突变验证」章节。

## 目录定位

ACTGS 目录下的引擎/游戏工具集合。

本 README 根据本目录内 Python 源码的实际入口、参数、注释和数据结构整理，用于说明当前目录工具的用途与推荐使用顺序。

## 文件分工

| 文件 | 定位 | 说明 |
|---|---|---|
| `scr_crypto.py` | 公共库/编解码 | ACTGS 引擎 - 加解密与档案处理核心模块。提供: `auto_find_key`(多层特征码+密钥环回退), `xor_cycle`, `decrypt_script`/`encrypt_script`, `parse_archive`(保留空隙/尾部数据), `build_archive`(精确/连续双模式) |
| `scr_extract.py` | 提取/解析 | ACTGS 引擎 .scr 脚本文本提取工具。从 arc.scr 档案中提取可翻译文本为 GalTransl 兼容 JSON。用法: `python scr_extract.py <ACTGS.exe> <arc.scr> [输出目录]` |
| `scr_inject.py` | 注入/回写 | ACTGS 引擎 .scr 脚本文本注入工具。将翻译 JSON 注入回 arc.scr。用法: `python scr_inject.py <ACTGS.exe> <arc.scr> <翻译JSON目录> [输出arc.scr] [编码]`。编码默认 cp932，汉化用 gbk |

## 密钥搜索

`auto_find_key` 采用多层回退策略：

1. **特征码搜索** — 在 EXE 的 `.text` 段搜索解密函数特征（4 种指令模式）：
   - `cmp eax, 0x58` → `mov [ebp-xx], key_addr`（Delphi/C++Builder）
   - `cmp al, 0x58` → `mov [ebp-xx], key_addr`（8 位优化）
   - `cmp ..., 0x58` → `push offset key_addr`（传参方式）
   - `cmp ..., 0x58` → `mov eax, offset key_addr`（寄存器方式）

2. **密钥环回退** — 若特征码未命中，用已知密钥尝试解密同目录下的 `.dat`/`.scr` 档案索引：
   - `"\x22\x33\x41\x11\x05\x54\x16"`（7 字节，新版 ACTGS）
   - `"ACTGS ACTRESS Game System"`（25 字节，旧版文本密钥）
   - `"\x10\xD3\x27\x53..."`（17 字节，旧版二进制密钥）

也可直接运行 `python scr_crypto.py <ACTGS.exe>` 仅提取密钥。

## 推荐流程

1. 运行 `python scr_crypto.py <EXE路径>` 确认密钥可提取。
2. 运行 `scr_extract.py` 导出文本为 JSON。
3. 只修改翻译字段（`message`）后运行 `scr_inject.py` 回写。原文字段用于定位与校验，不建议改动。
4. 注入产出连续布局档案（引擎按索引读取，兼容所有版本）。

## 零突变验证

若需验证工具完整性（解密→加密与原文件逐字节一致），使用精确模式：

```python
from scr_crypto import auto_find_key, parse_archive, build_archive, encrypt_script

key = auto_find_key('あるかね.exe')
scripts, header, he, gaps, trailing = parse_archive('scr.dat', key)
output = [(name, encrypt_script(scr, key)) for name, scr in scripts]
rebuilt = build_archive(header, output, key, he, gaps=gaps, trailing=trailing)

with open('scr.dat', 'rb') as f:
    assert f.read() == rebuilt  # 逐字节一致
```

> **注意:** 某些 ACTGS 档案的文件间存在旧脚本残片（空隙数据）和尾部数据。这些数据引擎运行时忽略，但零突变验证必须保留。`parse_archive` 现返回 `gaps`/`trailing`，传给 `build_archive` 即可原样还原。

## 文本/JSON 字段约定

源码中出现的主要字段：`name`, `message`。
- `msg/message` 通常是可修改译文字段，提取后默认等于原文或解析后的正文。

## 命令示例

```bash
# 仅提取密钥
python scr_crypto.py "あるかね.exe"

# 提取文本
python scr_extract.py "あるかね.exe" scr.dat scr_json

# 注入翻译（默认 cp932）
python scr_inject.py "あるかね.exe" scr.dat scr_json arc_new.scr cp932

# 注入翻译（汉化用 gbk）
python scr_inject.py "あるかね.exe" scr.dat scr_json arc_new.scr gbk
```

## 注意事项

- 操作前请备份原始封包、脚本和 EXE；注入/封包类脚本通常会直接生成可替换资源。
- 保持提取时的目录结构与文件名；多数注入器依赖相对路径、偏移或原文校验。
- 默认编码多为 CP932/Shift-JIS；若脚本提供 `--encoding`，除非目标游戏已确认，否则不要随意改成 GBK。
- 对等长/截断注入器，译文过长可能被截断、报错或破坏后续指令；非等长注入器也需要确认跳转/长度表是否已同步修正。
- **密钥搜索失败时**，确认 EXE 与 .dat/.scr 在同一目录下，工具会自动回退到密钥环尝试匹配。
