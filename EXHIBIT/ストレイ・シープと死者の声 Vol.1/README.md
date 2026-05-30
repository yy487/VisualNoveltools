# ExHIBIT RLD JSON Tool v4

用于 ExHIBIT / Retouch 系 `.rld` 脚本的静态解密、反汇编、正文/选择支 JSON 提取和 JSON 注入回封。

## v4 主要修正

- 增加 `--export-names`：可额外导出 `defChara.rld` / opcode `0x0030` 中的角色名条目。
- `_type=name` 条目会参与注入，修改它的 `message` 后会写回 `defChara.rld`，所有通过角色 ID 引用该名字的正文都会跟着变。
- 正文条目里的 `name` 仍然只作为上下文，不参与写回，避免同一角色名在多条正文中被不同翻译反复覆盖。
- 不再把 `defChara.rld` 误判为固定 `DEF_SEED`。当前样本中：
  - `def.rld` 使用固定 seed `0xAE85A916`。
  - `defChara.rld` 使用普通场景 seed，用于建立角色名表。
- 提取目录时会先全局扫描 `defChara.rld` / `def.rld` / 其他 RLD 中的 `0x30` 角色定义，建立全局 `char_id -> name` 表。
- JSON 输出字段顺序为：`name`、`scr_msg`、`message` 在前。无角色名时不输出 `name`。
- 将 CP932 私用区字符 `U+E000~U+F8FF` 转义为 `<PUA_XXXX>`，例如原始字节 `F0 4A` 解码为 `U+E00A`，提取时输出 `<PUA_E00A>`，注入时自动还原。

## 基本流程

### 只提正文和选择支

```bat
python extract.py --exe "game.exe" --ini "ExHIBIT.ini" "rld" "json"
```

如果已经知道普通 seed，可以直接指定：

```bat
python extract.py --seed 0x851C549B "rld" "json"
```

### 同时导出可编辑角色名

```bat
python extract.py --seed 0x851C549B --export-names "rld" "json"
```

这会额外生成类似：

```text
json/defChara.rld.json
```

里面是 `_type=name` 条目。修改这些条目的 `message`，注入时会写回 `defChara.rld`。

### 注入

只修改 JSON 中的 `message` 字段，不修改 `scr_msg`。

```bat
python inject.py --exe "game.exe" --ini "ExHIBIT.ini" "rld" "json" "out_rld"
```

或：

```bat
python inject.py --seed 0x851C549B "rld" "json" "out_rld"
```

## 正文 JSON 格式

```json
{
  "name": "巴",
  "scr_msg": "「何故って……」",
  "message": "「何故って……」",
  "_file": "1_01_00_000.rld",
  "_index": 66,
  "_offset": 15673,
  "_inst_offset": 15619,
  "_opcode": "0x001C",
  "_op_index": 175,
  "_str_index": 1,
  "_type": "dialogue",
  "_encoding": "cp932",
  "_policy": "relocate"
}
```

正文条目里的 `name` 只作为上下文，不参与写回。注入实际写回 `message`。

## 角色名 JSON 格式

```json
{
  "scr_msg": "声",
  "message": "声",
  "_file": "defChara.rld",
  "_index": 0,
  "_offset": 280,
  "_inst_offset": 276,
  "_opcode": "0x0030",
  "_op_index": 1,
  "_str_index": 0,
  "_type": "name",
  "_name_id": 3,
  "_name_field": 3,
  "_encoding": "cp932",
  "_policy": "relocate"
}
```

`0x0030` 的角色定义实际是一个 CSV 字符串，角色名通常在第 4 个字段，也就是 `_name_field=3`。工具注入时会校验该字段是否等于 `scr_msg`，然后只替换这个字段并重建整条 CSV 字符串。

## 当前文本规则

- `0x001C`：正文显示。`init[0]` 为角色 ID，命中全局 name table 时输出 `name`。
- `0x0015`：普通正文/旁白。
- `0x00BF`：保守作为选择支候选；当前样本中未发现可翻译选择支文本。
- `0x0030`：角色定义。默认只用于建立 name table；开启 `--export-names` 后导出 `_type=name` 条目。
- `0x000C`：历史/缓存类重复文本。样本中它总是紧邻同文 `0x001C`，因此默认不导出，避免重复。

## 私用区控制符

样本中发现 `U+E00A`，原始 CP932 字节为 `F0 4A`。它不是乱码，而是引擎私用控制符，常出现在句中或句尾，疑似停顿/演出控制。提取时显示为 `<PUA_E00A>`，翻译时应保留，注入时工具会还原。

## 已验证样本

对 `rld.zip`：

- 角色名表：从 `defChara.rld` 使用普通 seed 解析得到 31 个角色名。
- `--export-names`：导出 31 条 `_type=name` 角色名条目。
- 正文/旁白：1701 条。
- 有 name 的 dialogue：1306 条。
- 无 name 的 monologue：395 条。
- `<PUA_E00A>` 出现：387 次，涉及 148 条 JSON。
- 原文 JSON 未修改直接注入回封：正文 RLD 与 `defChara.rld` 均 byte-exact 一致。

## 注意

如果要翻译角色名，请改 `defChara.rld.json` 里的 `_type=name` 条目的 `message`，不要改正文条目里的 `name`。正文条目里的 `name` 改了不会写回，这是为了避免同一个角色 ID 在不同正文条目里产生冲突。
