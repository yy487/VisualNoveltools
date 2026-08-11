# NECRONOMICON MES JSON 工具 v0.2.1

本工具把 PC-98 版 `NECRONOMICON` 的 `.MES` 字节码提取为 UTF-8 JSON，并从
JSON 重建 MES。v0.2 不再把 `BA 23` 当记录终止符，而是按 OVL 中
`sub_D77C` 的真实长度规则建立覆盖全文件的 lexer/IR；A6–D8 的 51 个 opcode
均记录运行时 handler 目标和 operand grammar。

v0.2.1 修正自定义平假名压缩的字符边界：CP932 双字节汉字现在整对复制，不会
把相邻汉字的尾字节和首字节误认成 `82 9F..F1` 平假名序列。

## 命令

提取单文件：

```powershell
mes_extract.exe DISK_B\MES\OPEN1.MES --output OPEN1.MES.json
```

递归提取 A–K 盘目录：

```powershell
mes_extract.exe "<MES_ROOT>" --output json
```

若输入根目录下存在 `DISK_A` 至 `DISK_K`，提取器只扫描这些磁盘树，自动忽略
`work` 内的测试副本。普通目录仍递归搜索全部 `.MES`。

重要：2026-07-18 核对原 FDI 的 FAT12 目录后，确认旧拆包树中的
`DISK_D/G/H/I/J/K` 含错碟副本。最终介质的
真实清单为 208 个 MES、17,502 条；注入时必须使用从原 A–K FDI 重新提取且与
JSON `_file` 一一对应的真实 MES 树，不能直接使用旧 278 文件拆包树。

注入 JSON 树。`--source-root` 必须与 JSON `_file` 的相对路径对应：

```powershell
mes_inject.exe json `
  --source-root "<MES_ROOT>" `
  --output rebuilt
```

两个 EXE 的第一个位置参数均可由 Windows 拖放传入。省略 `--output` 时使用
输入旁的新名称；已有输出一律拒绝覆盖。

## JSON 合同

可翻译记录只使用以下字段：

```json
{
  "_scr_name": "男の声",
  "name": "男の声",
  "scr_msg": "　そろそろだな‥‥‥‥‥‥",
  "message": "　そろそろだな‥‥‥‥‥‥"
}
```

- `scr_msg` 是不可修改的源正文；只编辑 `message`。
- 文本以 `「名字」` 开头时，名字拆成不可修改的 `_scr_name` 和可写 `name`；
  注入器自动恢复括号。
- 无该前缀的文本不猜说话人，也不能凭空增加 `name`。
- `_file/_index/_offset/_size/_type/_opcode/_encoding/_policy` 是不可修改的定位元数据。
- `_tokens` 是全文件 IR。每项保留 `_offset/_size/_type/_role/_raw`；opcode 另有
  `_handler_target/_handler_grammar`，handler 参数另有 `_owner_opcode_index`。
- `_warnings` 是源分析结果，也会在注入前重新计算和校验。

JSON 使用 UTF-8。正文不允许 NUL、CR/LF 或会与 MES opcode 冲突的 CP932
单字节字符。

## `sub_D77C` lexer 与 handler

静态反汇编和运行时内存表共同确认 token 长度：

- `21`：到 NUL（含终止字节）；`22`：到下一个 `22`（含结束引号）；
- `23..27`：1 字节；`28`：2 字节；`29..2C`：3 字节；
- `00..0F`：2 字节；`10..1F`：3 字节；
- Shift-JIS lead `81..9F/E0..FC`：2 字节；其他项：1 字节。

`sub_C635` 还确认 `81 97`、`81 90`、`81 6F`、`81 70` 是特殊控制，而不是
普通正文。`2D..7F` 按游戏自定义平假名表显示；`A5` 是显示换行；A6–D8
进入 `DS:DB5A` 跳转表。IR 为每个 A6–D8 opcode 保存已恢复的 handler 目标，
并把连续 expression、引号字符串及 B3/B4/B5/B6/CA/D0 的直接字节参数归属到
对应 opcode。

`BA 23` 因此是 `BA @ D00F` 加一个 `23` expression，不是文件、记录或正文
终止符。

## 编码与变长

正文采用 CP932，并把平假名 `82 9F..F1` 恢复为游戏的单字节
`2D..7F`。无法表示的字符会列出并拒绝；生成字节必须重新解码为原字符串，
避免 ASCII/半角假名落入 opcode 区。

未修改正文直接复用原字节。修改后按 token 顺序重建全流，正文可以增长或
缩短；未知 token 和控制项从已验证源文件逐字节复制。中文测试仍应先按项目的
`subs_cn_jp.json` 把中文映射到可编码的日文字位，并配套重绘 PC-98 字库。

## 写回 FDI

```powershell
mes_inject.exe json `
  --source-root "<MES_ROOT>" `
  --output rebuilt

fdi_repack.exe "<FDI_DIR>\NECRONOMICON_B.FDI" `
  --replacements rebuilt\DISK_B `
  --output NECRONOMICON_B.translated.FDI
```

FDI repacker 负责更新目录项 size、两份 FAT12 和 cluster chain；不会覆盖原盘。

## 已验证结果

- 13 个 Rust 单元测试通过，覆盖全类 token 长度、51 个 handler、名字校验、
  immutable 元数据、未知字节、增长/缩短及非法编码。
- A–K 共 278 个 MES、24,161 条 entries；全文件均由 `_tokens` 连续覆盖。
- 零修改注入：24,161 条全部 unchanged，278 个逐文件 SHA-256 全部 byte-exact。
- 真实 `OPEN1.MES` 同时修改两条正文（一个增长、一个缩短），再次提取仍为
  77 条，两个译文均精确恢复；文件从 4103 字节变为 4095 字节。
- 12 个 warning 仅来自 3 个脚本中的未定义 CP932 pair，均作为 opaque token
  无损保留；正文中未发现 PUA 或误提的全角 `＠`。
- 已确认旧工具漏掉的 `MAIN.MES@0x563`、`NA_01C.MES@0xA6F` 和
  `NA_01C.MES@0xB21` 均进入新 JSON。
- 最终真实原碟集：208 MES、17,502 条；全量修改注入后重新提取，名字与正文
  17,502/17,502 一致。A–K 11 张 FDI 均通过空替换重封 SHA-256 byte-exact。

已知限制：`D5` handler 可在运行时变换后续流，IR 记录其 operand 但不模拟
运行时自修改；当前 A–K 样本不依赖该行为来恢复正文。`21 ... 00` 显示项在
本作 496 次出现均只有空格，因此作为显示 padding 保留而不输出翻译记录。
最终发行仍需在目标 PC-98 模拟器中走剧情检查显示宽度、字体和分支覆盖。
