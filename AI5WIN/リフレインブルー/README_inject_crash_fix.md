# Refrain Blue MES 注入崩溃修正说明

## 问题原因

旧版 `rp_mes_common.py` 的注入策略有严重问题：

```python
if len(new_command) <= old_span:
    原地写入，并把剩余区域填 00
```

这对 **零结尾字符串命令** 是错误的。

本作文本命令结构是：

```text
01 <cp932 string> 00
```

文本函数读到 `00` 后，VM 会从这个 `00` 的下一个字节继续执行。因此如果译文比原文短，新的 `00` 会提前出现，VM 就会从原文本槽内部的填充 `00` 开始执行，而不是跳到原来的下一条指令。`00` 在 VM 中又是 RETURN/结束类 opcode，所以会导致流程提前返回、黑屏或闪退。

你遇到的崩溃点正好对应 `02.MES` 第 3 条：

```text
原文：俺は閉じていた瞼を、ゆっくりと開いた。
译文：我緩緩諍開緊閉的双眼。
```

旧版把这条短译文原地写入，导致字符串在 `0x1A83` 提前结束，而原本下一条指令在 `0x1A93`。

## 修正策略

新版只允许 **完全等长** 的命令原地写入：

```text
len(01 + message + 00) == 原命令跨度
```

只要长度不同，无论变长还是变短，都使用 EOF 跳板：

```text
原位置：0A <append_offset>
EOF：01 <new_text> 00 0A <return_offset>
```

这样 VM 显示完新文本后，会跳回原始文本命令结束位置，不会执行填充区域。

## 使用方式

重新用原始 `MES.ARC` 和翻译 JSON 注入：

```bat
python rp_mes_inject.py MES.ARC json mes_new_fixed.arc
```

如果只有旧版错误注入后的 `mes_new.arc`，可以先恢复 JSON：

```bat
python rp_mes_recover_from_bad_arc.py MES.ARC mes_new.arc recovered_json
python rp_mes_inject.py MES.ARC recovered_json mes_new_fixed.arc
```

## 检查方式

可以用 `rp_mes_find.py` 搜 `02.MES` 开头几句，或直接解包后看：

- 原位置若长度不同，应以 `0A <append_offset>` 开头；
- EOF 追加块应为 `01 <译文> 00 0A <原end>`；
- 不应再出现“短译文 + 后面填 00”的情况。
