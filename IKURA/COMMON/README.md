# IKURA / ISF JSON 工作流工具

## 1. 处理范围

本目录用于 IKURA/ISF 系脚本的文本本地化流程，支持：

- DRS / MPX 包解包；
- ISF / SNR 脚本文本提取；
- 项目统一 JSON 格式导出；
- 从 JSON 变长注入回 ISF / SNR；
- 按 `file_order.json` 重封 DRS / MPX。

默认脚本分支为新版 `MPX`。如果是旧版 DRS 脚本，提取和注入时加 `--engine DRS`。

## 2. 当前修正点

原工具的主要问题是把独立的名字显示行当成正文导出，例如：

```json
{
  "name": "",
  "ori": "【晶】",
  "message": "【晶】"
}
```

这类 `【角色】` 行本质上是角色名显示槽，后面一条才是正文。新版提取逻辑会把它挂到下一条正文的 `name` 字段中，并且不再把该名字槽作为待翻译正文导出：

```json
{
  "name": "晶",
  "scr_msg": "「そこのちょっと死んでるひとー」",
  "message": "「そこのちょっと死んでるひとー」",
  "_file": "xxxx.isf",
  "_index": 0,
  "_raw_index": 1,
  "_name_raw_index": 0,
  "_type": "dialogue"
}
```

注入时仍会按原始 ISF 重新扫描全部文本槽。没有导出的名字槽会原样保留，因此不会出现因为少导出名字行导致的注入序列错位。

同时把原来的 `ori/message` 改成项目统一格式：

```json
{
  "name": "角色名，可选",
  "scr_msg": "原始脚本文本，只用于校验，不修改",
  "message": "翻译文本，注入时写回"
}
```

内部定位字段全部以下划线开头，例如 `_file`、`_index`、`_raw_index`、`_op_index`、`_inst_offset`、`_offset`、`_size`、`_opcode`。

有 `name` 的条目会按字段顺序输出为 `name`、`scr_msg`、`message`，方便翻译时先看说话人。

## 3. 命令

### 3.1 只处理散 ISF 文件

提取：

```bash
python extract.py "isf_dir" "json_out" --engine MPX
```

注入：

```bash
python inject.py "isf_dir" "json_translated" "isf_chs" --engine MPX
```

### 3.2 解包 MPX / DRS

如果脚本包中的 ISF 带 `SECRETFILTER100a` 外层 XOR 壳，需要提供 EXE 或 secret：

```bash
python unpack.py "Isf" "unpacked" --exe "game.exe"
```

或：

```bash
python unpack.py "Isf" "unpacked" --secret "secret.bin"
```

解包会在输出目录生成 `file_order.json`，重封时用它保持文件顺序。

### 3.3 重封 MPX / DRS

```bash
python pack.py "isf_chs" "out/Isf" --order "unpacked/file_order.json"
```

如果没有 `file_order.json`，MPX 会按目录文件顺序回退，但不建议这样做。

## 4. JSON 字段说明

- `name`：可选。只有识别到说话人时输出。
- `scr_msg`：原始脚本文本，注入时用于校验。
- `message`：译文，实际写回字段。
- `_file`：来源 ISF/SNR 文件。
- `_index`：当前 JSON 文件内导出序号。
- `_raw_index`：底层文本槽序号，注入主定位字段。
- `_name_raw_index`：如果角色名来自上一条独立 `【角色】` 槽，则记录该槽序号。
- `_type`：`dialogue` / `choice` / `ui` / `system`。
- `_policy`：当前工具为 `relocate`，会重建 opcode 长度和 ISF 跳转表。

## 5. 注入策略

本工具不是原地覆盖，而是对 ISF 内部 opcode 做变长重建：

1. 读取原始 ISF；
2. 重新扫描底层文本槽；
3. 通过 `_raw_index` 定位 JSON 对应槽；
4. 用 `scr_msg` 校验；
5. 写入 `message`；
6. 重建 opcode 长度；
7. 重建 ISF 头部跳转表；
8. 按原文件加密方式重新加密保存。

因此译文可以比原文长，但仍需保证文本能被 `cp932` 编码。简体中文请先通过 cn_jp/subs 映射或字库替换方案处理。

## 6. 已知限制

1. 默认按 `MPX` 分支处理散 ISF；旧 DRS 必须手动指定 `--engine DRS`。
2. 角色名识别采用结构扫描后的 `【角色】` 行配对，不是全局正则扫二进制。极少数剧情正文如果单独一行正好是 `【某某】`，会被当作名字槽。
3. 当前没有自动修改字体和字符映射，只检查/执行 cp932 编码。
4. `0x2B/0x2C` 的 MPX 文本块沿用该引擎常见结构：一个 PM 文本 opcode 内通常只有一个正文 payload。
5. `file_order.json` 需要随解包结果保存，重封 MPX 时用于保持原始文件顺序。
