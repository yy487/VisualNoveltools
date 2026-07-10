# 紅殻町博物誌本地化工具

用于处理 raiL-soft《紅殻町博物誌》的 XFL 封包和 GSC 脚本文本。工具使用 Rust 编写，翻译中间格式固定为 UTF-8 JSON，GSC 原始编码为 CP932。

## 构建

已在 Windows Rust 1.92.0 验证：

```powershell
cargo build --release --offline
```

生成以下程序：

```text
target\release\xfl_unpack.exe
target\release\xfl_pack.exe
target\release\gsc_extract.exe
target\release\gsc_inject.exe
target\release\railsoft-xfl.exe
```

最终可直接使用的四个拖放程序也会复制到本项目的 `release` 目录；该目录是本地构建产物，不提交 Git。

## 拖放流程

四个程序均不会覆盖已有输出。运行结束后按 Enter 关闭窗口。

1. 把 `scr.xfl` 拖到 `xfl_unpack.exe`，生成 `scr_unpacked`。
2. 把 `scr_unpacked` 文件夹拖到 `gsc_extract.exe`，生成 `scr_unpacked_json`。
3. 只修改 JSON 中的 `message`，然后把整个 `scr_unpacked_json` 文件夹拖到 `gsc_inject.exe`，生成 `scr_rebuilt`。
4. 把 `scr_rebuilt` 文件夹拖到 `xfl_pack.exe`，生成 `scr_rebuilt.xfl`。

解包目录中的 `.xfl-manifest.json` 保存原条目顺序和原始文件名字节；JSON 目录中的 `.gsc-manifest.json` 保存原 GSC 目录及文件对应关系。两份内部清单都不要删除或修改。

命令行调用时可增加 `--no-pause`，例如：

```powershell
xfl_unpack.exe "E:\GAL\紅殻町博物誌\work\scr.xfl" --no-pause
gsc_extract.exe "E:\GAL\紅殻町博物誌\work\scr_unpacked" --no-pause
gsc_inject.exe "E:\GAL\紅殻町博物誌\work\scr_unpacked_json" --no-pause
xfl_pack.exe "E:\GAL\紅殻町博物誌\work\scr_rebuilt" --no-pause
```

## JSON 格式

每个 GSC 对应一个 `<文件名>.gsc.json`。数组中的典型条目为：

```json
{
  "name": "十湖",
  "scr_msg": "「ありがとね」",
  "message": "「ありがとね」",
  "_file": "1203.gsc",
  "_index": 97,
  "_offset": 25785,
  "_size": 24,
  "_type": "dialogue",
  "_encoding": "cp932",
  "_policy": "relocate"
}
```

- `message`：唯一允许翻译或修改的字段，注入时实际写回。
- `scr_msg`：原文及严格校验依据，不得修改。
- `name`：确认过的说话人上下文，不作为写回字段，不得修改。
- `_file + _index + scr_msg`：稳定定位键。
- `_offset/_size/_type/_encoding/_policy`：结构校验字段，不得修改。
- 选项额外包含 `_inst_offset`、`_opcode`、`_target` 和 `_choice_style`，不得修改。

说话人前缀 `【name】^n` 和选项样式前缀 `<01>` 会在导出时拆出，在注入时自动按原始字节恢复。空字符串以及已确认的 `grpo*`、`REP001..REP008` 资源标识不会导出。

## 变长注入

注入采用完整重建文本偏移表和文本池的方式：

- 所有未修改文本保留原始 CP932 字节。
- 修改文本重新编码为 CP932；出现不可编码字符或 NUL 时直接报错，不做静默替换。
- 指令区整体原样复制，因此文本索引、选择目标和 `0x0003/0x0004/0x0005` 跳转地址不变。
- 文本池之后的 `u16` 序列区、符号表和符号名池原样复制。
- 更新 GSC 文件总大小和文本池大小。
- XFL 封包时重新计算所有条目的 `offset/size`。

普通简体中文的字体和 CP932 字符映射不属于本工具范围；注入器只负责严格执行最终映射后的 CP932 编码。

## 命令行 XFL 工具

`railsoft-xfl.exe` 保留显式输出路径和索引查看功能：

```powershell
railsoft-xfl.exe list <archive.xfl>
railsoft-xfl.exe unpack <archive.xfl> <output-dir> [--force]
railsoft-xfl.exe pack <input-dir> <archive.xfl> [--force]
```

## 格式摘要

XFL 文件头为 12 字节，小端序；每条索引固定 40 字节，其中包括 32 字节文件名、相对数据区的 `u32 offset` 和 `u32 size`。当前严格支持 magic `LB`、版本 `1`、保留字节 `0`。

GSC 文件头为 36 字节，物理布局如下：

```text
code
text offset table
CP932 NUL text pool
u16 sequence offset table
u16 sequence pool
symbol-name offset table
symbol value/code-offset table
symbol-name pool
```

完整结构、文本控制码、选项和跳转分析见 [vm_analysis.md](vm_analysis.md)。

## 验证结果

真实样本：

```text
scr.xfl SHA-256  53D094A3B41AC7217BD8D313BB4F6C3EBF69AD04EEF16990C1547BADE3E2EFEA
XFL 条目数         74
GSC 文本索引数      8,461
导出翻译条目数      8,357
对话               4,452
旁白               3,856
选项                  49
跳过记录              104（74 个空字符串、30 个资源标识）
```

已完成：

- 15 个 Rust 单元测试通过。
- `cargo fmt --check` 通过。
- `cargo clippy --all-targets --offline -- -D warnings` 通过。
- 74 个 GSC 零修改提取再注入，74/74 SHA-256 一致。
- 重建目录重新封成 XFL 后与原 `scr.xfl` 逐字节一致。
- 在真实 `1203.gsc` 同时加长一个选项和一条对话：文件增长 98 字节，代码区和文本池之后全部区域逐字节不变，选项目标仍为 `0x00001002`。

## 已知限制

- 仅针对当前游戏已验证的 XFL v1 和 GSC 布局。
- JSON 条目不能新增、删除、重排定位字段或修改 `scr_msg`。
- 工具不提供字体、中文字符映射或可执行文件补丁。
- 当前不提供完整 VM 反汇编/重汇编；文本池重建不移动指令区，因此不需要重算代码跳转。
