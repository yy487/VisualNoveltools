# Refrain Blue MES 非等长注入修正版 v3

本版修正了零结尾文本命令的 PC 恢复问题，并把“变短译文”的处理从 EOF-only 跳板优化为优先使用原文本块内部的尾跳转。

## 文本命令

本作 MES 文本命令为：

```text
01 <cp932 zero-terminated string> 00
```

文本绘制函数读到 `00` 后，VM 会从 `00` 的下一个字节继续执行。因此，变短译文不能直接写入并用 `00` 填满旧空间；那会让 VM 提前恢复执行，进入填充区。

## v3 注入策略

对每条被修改文本：

1. 长度完全相等：原地写入。
2. 译文变短，且剩余空间至少 5 字节：写成

```text
01 <new_text> 00 0A <old_end>
```

这里 `0A <old_end>` 是尾跳转。文本读完后执行该跳转，回到原文本块结束位置。

3. 译文变长，或没有足够空间放尾跳转：写成 EOF trampoline：

```text
原位置：0A <append_off> 00 00 ...
EOF：   01 <new_text> 00 0A <old_end>
```

这样原始区域长度不变，已有绝对跳转目标不需要全局重算。

## 命令

```bat
python rp_mes_inject.py MES.ARC json mes_new.arc
```

强制所有修改文本都走 EOF trampoline：

```bat
python rp_mes_inject.py MES.ARC json mes_new.arc --force-jump
```

一般不建议 `--force-jump`，v3 默认混合策略更稳。

## 报告字段

- `inplace_equal`：完全等长原地写入数量。
- `inline_tail_jump`：变短且使用原块内部尾跳转的数量。
- `eof_trampoline`：变长或无法内联时追加到 EOF 的数量。
- `jump`：两类跳转注入总数。
