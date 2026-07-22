# 傾城 / Adviz（WADVIZ）剧情文本工具

本工具用于《傾城》`KEISEI.EXE` 的 `.ADV` 剧情脚本。EXE 版本资源已确认引擎名称为 `Adviz for Windows95`，内部名为 `WADVIZ`；`KEISEI` 是本游戏的程序标识，并非引擎名称。

- `qc_extract.exe`：提取 UTF-8 JSON。
- `qc_inject.exe`：校验 JSON 后回注，并重建脚本地址。

工具永远拒绝覆盖已有输出。默认输出会创建在源文件或源目录旁边，游戏源文件不会被修改。

## 快速使用

提取单个文件：

```powershell
qc_extract.exe E:\GAL\qc\work\TEXT\A02.ADV
```

默认生成 `A02.ADV.json`。提取整个目录：

```powershell
qc_extract.exe E:\GAL\qc\work\TEXT
```

默认生成 `E:\GAL\qc\work\TEXT_json`，其中每个 `.ADV` 对应一个 `.ADV.json`。

回注单个文件：

```powershell
qc_inject.exe E:\GAL\qc\work\TEXT\A02.ADV E:\GAL\qc\work\TEXT\A02.ADV.json
```

默认生成 `A02_injected.ADV`。回注整个目录：

```powershell
qc_inject.exe E:\GAL\qc\work\TEXT E:\GAL\qc\work\TEXT_json
```

默认生成 `E:\GAL\qc\work\TEXT_injected`。目录模式会复制完整源目录，只修改存在对应 JSON 的 `.ADV`。

可以用 `--output PATH` 或 `-o PATH` 指定新输出。也可以把单个 `.ADV` 拖到 `qc_extract.exe`。

回注支持以下拖放方式：

- 单独把 `TEXT_json` 拖到 `qc_inject.exe`，自动寻找同级 `TEXT`。
- 单独把 `TEXT` 拖到 `qc_inject.exe`，自动寻找同级 `TEXT_json`。
- 单独拖入 `A01.ADV.json` 或 `A01.ADV`，自动寻找同级对应文件。
- 同时拖入源路径和 JSON 路径时，顺序不限。

自动推导的同级路径必须真实存在，否则工具会显示缺失的完整路径并停止。

## JSON 约定

JSON 文件编码固定为 UTF-8，顶层是数组。不要改动以下定位或验证字段：

```json
{
  "name": "少女",
  "_scr_name": "少女",
  "scr_msg": "原始正文",
  "message": "翻译后的正文",
  "_file": "A02.ADV",
  "_index": 26,
  "_offset": 1914,
  "_inst_offset": 1905,
  "_size": 72,
  "_type": "dialogue",
  "_opcode": "X",
  "_encoding": "CP932",
  "_policy": "relocate",
  "_name_writable": true
}
```

- `scr_msg` 是不可修改的源正文，只编辑 `message`。
- 静态姓名可编辑 `name`，但 `_scr_name` 必须保持原值。
- 含控制码的动态姓名会标记为 `"_name_writable": false`，不可修改。
- 旁白没有 `name` 和 `_scr_name`。
- 相同正文按 `_inst_offset` 精确定位，不会按文本内容批量替换。
- `_offset` 和 `_size` 描述源脚本位置及源正文跨度。即使提取时隐藏了 `\n`，也不要改动这些字段。

## 控制码与自动换行

- 文本框会按可用宽度自动换行。提取器不会根据画面显示结果添加换行。
- 源脚本中的 `\n` 不会进入 `scr_msg` 或 `message`。条目被修改后回注时也不会恢复这些 `\n`，由引擎自动换行。
- 完全未修改的条目直接保留原始字节，因此无修改回注仍与源文件完全一致。
- 末尾 `\k\*` 是结构后缀，不进入 JSON 正文；修改后的条目会自动恢复。
- `\k` 表示等待玩家操作。
- `\*` 表示恢复/清除消息区域并重置文字位置。
- `\=...;` 是选项注册前缀，不进入 JSON 正文；回注时自动恢复。
- `\%...;` 等其他正文控制码必须在 `message` 中原样、按原顺序保留，否则拒绝回注。
- 只含 `\c...;`、`\s...;`、`\?N;`、`\!...;` 等渲染/界面控制码而没有可翻译字符的 `X` 字符串不会写入 JSON；它们仍在 ADV 中原样保留。

选项文本可正常加长或缩短。每个 `0x34` 菜单记录包含 8 个表达式，其中第 5、第 6、第 7 个字段会在不同选择路径中作为绝对跳转目标；工具会同时重建这 3 个地址，不需要在 JSON 中处理目标地址。

## 回注与校验

ADV 文本位于 `0x58` (`X`) 指令的累计 XOR 加密字符串中，字符编码为 CP932。回注流程会：

1. 重新解析源 ADV，不依赖二进制文本扫描。
2. 校验 JSON 数量、顺序、源正文、姓名和定位元数据。
3. 校验 CP932 编码及正文控制码。
4. 重建变长字符串和所有已确认的绝对脚本地址，包括菜单分支。
5. 重新解析输出，确认指令与文本数量有效，并确认每个脚本目标都落在合法指令边界或文件末尾。

原引擎每个 ADV 的读取上限是 65,536 字节，所有目标地址必须能放入对应的 8 位或 16 位字段；超限会拒绝生成输出。

原版渲染器使用 `TextOutA`、`SHIFTJIS_CHARSET` 和 MS Gothic，并按 Shift-JIS 双字节范围处理文本。简体中文并非都能用 CP932 表示；遇到不可编码字符时，工具会列出 Unicode 码点并停止。完整中文化需要额外修改渲染/编码逻辑，或采用 CP932 字形映射方案。

## 已验证范围

- 172 个 `.ADV` 文件全部完成结构化解析。
- 19,331 条 `X` 字符串指令，其中 16,804 条可翻译文本写入 JSON；2 条空正文和 2,525 条纯渲染/界面控制串跳过。
- 新规则下所有提取 JSON 均不含 `\n`。
- 无修改回注后 172 个文件的 SHA-256 与源文件完全一致。
- 已用真实 `A02.ADV` 加长两条选项并修改静态姓名/正文，回注后可完整重新提取。
- 已核对 `sub_403B50` 的 3 条菜单选择路径，并用真实翻译版 `A02.ADV` 验证旧输出中 11 个未重定位的第 5 字段现已全部修正；修复版目标均落在合法指令边界。
- 未知或未使用的数据保持原字节；不支持无法完整解析的新变体。
