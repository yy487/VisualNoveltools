# Refrain Blue / RP MES VM 分析定义（基于 rp.EXE.c）

## 1. 解释器入口

当前脚本 VM 的主解释循环位于 `thunk_FUN_00406110`。该函数通过脚本流对象的 vtable 读取字节：

- `(**(code **)(*param_1 + 8))(0)`：窥视当前位置字节，不推进 PC。
- `(**(code **)(*param_1 + 8))(1)`：读取并推进 1 字节。
- 同一接口在部分代码中也以 `4` 为参数跳过/读取 4 字节。
- `thunk_FUN_004059a0(ctx, addr)`：设置当前脚本 PC / 跳转目标。
- `thunk_FUN_004059c0(ctx)`：读取当前脚本 PC。

因此 MES 脚本不是独立文本池结构，而是内联字节码指令流：文本以 `01 <cstring> 00` 直接嵌入指令流。

## 2. 顶层 opcode

顶层 opcode 由 `thunk_FUN_00406110` 原始读取一个字节后分派。已确认分派如下：

| opcode | 处理函数 | 格式/含义 |
|---:|---|---|
| `00` | loop return | 块结束 / 返回。消费 `00` 后返回。 |
| `01` | `thunk_FUN_0040e5f0` | 可见文本。格式 `01 <cp932 cstring> 00`。内部按 Shift-JIS lead byte 识别双字节。 |
| `02` | `thunk_FUN_00406ca0` | 系统/单字节绘制文本。格式类似 `02 <cstring> 00`。 |
| `03` | `thunk_FUN_00406e30` | 写 byte 变量。格式 `03 <u16le base> <expr> <sep> ... 00`。 |
| `04` | `thunk_FUN_00406ea0` | 写 short 表。格式 `04 <u8 start_index> <expr> <sep> ... 00`。 |
| `05` | `thunk_FUN_00406ef0` | 写 byte 表。格式 `05 <expr offset> <expr> <sep> ... 00`。 |
| `06` | `thunk_FUN_00406f90` | 基于表基址写 byte。格式 `06 <expr offset> <u8 table> <expr> <sep> ... 00`。 |
| `07` | `thunk_FUN_00407010` | 基于表基址写 word。格式 `07 <expr index> <u8 table> <expr> <sep> ... 00`。 |
| `08` | `thunk_FUN_004070a0` | 基于表基址写 dword。格式 `08 <expr index> <u8 table> <expr> <sep> ... 00`。 |
| `09` | `thunk_FUN_00407120` | 条件跳转。格式 `09 <expr> <target:u32le>`；若 expr == 1，则跳过 target 继续执行；否则跳转到 target。 |
| `0A` | `thunk_FUN_004071b0` | 无条件绝对跳转。格式 `0A <target:u32le>`。 |
| `0B` | `thunk_FUN_00407230` | 二级 SYS/显示命令分派。格式 `0B <expr subop> <subop operands...>`。 |
| `0C` | `thunk_FUN_00407460` | 保存/使用当前消息缓冲相关。变长，依赖当前字符串缓冲。 |
| `0D` | `thunk_FUN_00407520` | 菜单/选择项累计相关。变长，依赖 `thunk_FUN_004059e0`。 |
| `0E` | `thunk_FUN_004077c0` | 选择分支/调用式跳转。先解析 `thunk_FUN_004059e0` 参数块，再读 `<target:u32le>` 并跳转。样本中的 `0E 02 01 FF 00 50 59 00 00` 应理解为 `0E <argblock: 02 01 FF 00> <target=0x5950>`。 |
| `0F` | text wait/control | 解析参数块后调用显示控制。 |
| `10` | display/control | 空/显示相关。 |
| `11` | `thunk_FUN_0040b710` | 行/页推进相关。额外读 1 字节。 |
| `12` | `thunk_FUN_0040b750` | 保存当前位置并跳转。格式 `12 <expr index> <target:u32le>`。 |
| `13` | `thunk_FUN_0040b7f0` | 换行/等待/显示控制。额外读 1 字节；样本常见 `13 00`、`13 01`。 |
| `14` | `thunk_FUN_00406f40` | 写 dword 表，格式类似 `14 <u8 start> <expr> <sep> ... 00`。 |
| `15` | inline generated branch | 在顶层 loop 中触发 `0xEF` 记录输出后进入 `thunk_FUN_0040e7f0`。 |

内部生成/消息缓冲中还会出现：`CD`、`CE`、`CF`、`DD`、`DF`、`EE`、`EF`。这些在 `thunk_FUN_00406110` 中也有分支，但主要由引擎运行时写入输出缓冲，不应直接作为普通 MES 文本提取锚点。

## 3. 表达式编码

表达式解析器位于 `thunk_FUN_00413350`，使用值栈；`thunk_FUN_00413990` 从栈弹出。表达式以 `FF` 结束并返回栈顶值。

| token | 含义 |
|---:|---|
| `00..7F` | 直接压入该字节常量。 |
| `80 <idx>` | 读取 VM 变量/word 表。 |
| `A0 <expr> <table>` | 读取 word 表。 |
| `C0 <expr> <table>` | 读取 byte 表。 |
| `E0` | 一元/函数表达式，调用 `thunk_FUN_004139d0`。 |
| `E1` | 一元/函数表达式，调用 `thunk_FUN_00413a00`。 |
| `E2` | 一元/函数表达式，调用 `thunk_FUN_00413a30`。 |
| `E3` | 二元除法。 |
| `E4` | 二元取模或相关算术。 |
| `E5` | 随机/取模相关。 |
| `E6..F0` | 比较/布尔/算术组合。 |
| `F1 <lo> <hi>` | 压入 imm16，小端。 |
| `F2 <b0> <b1> <b2> <b3>` | 压入 imm32，小端。 |
| `F3 <lo> <hi>` | 以 imm16 为偏移读 byte。 |
| `F4 <expr>` | 以表达式结果为偏移读 byte。 |
| `F5 <expr> <table>` | 读 dword 表。 |
| `F6 <idx>` | 读 dword 基址表。 |
| `FF` | 表达式结束，弹出并返回栈顶。 |

样例：

- `01 FF` 是表达式常量 1。
- `15 FF` 是表达式常量 0x15，因此 `0B 15 FF ...` 表示 `SYS subop=0x15`，并不是裸 `0B 15` 固定 opcode。
- `F1 C8 00 FF` 表示 imm16 0x00C8。

## 4. 跳转与标签化

确认需要重定位的地址字段：

1. `0A <target:u32le>` 无条件绝对跳转。
2. `09 <expr> <target:u32le>` 条件绝对跳转。
3. `0E <argblock> <target:u32le>` 选择分支/调用式绝对跳转。
4. `12 <expr> <target:u32le>` 保存返回点并跳转。
5. `0B <expr>` 的部分 SYS 子命令内部也可能间接改变控制流，必须逐个子码确认后再启用重定位。

整体重建注入时必须建立 `old_offset -> new_offset` 映射，并重写所有上述地址。不能继续依赖 EOF trampoline 修改原位置 opcode。

## 5. 注音/ruby 结构

样本中的 ruby 不是单独 opcode，而是 SYS 二级命令组合，典型模式：

```text
0B 15 FF 01 <ruby_text> 00 ... 01 <base_text> 00
```

其中 `15 FF` 是表达式常量 0x15。ruby reading 可在汉化注入时原地改为全角空格，但不应删除控制结构，也不应把 ruby reading 混入 `scr_msg`。

## 6. 当前工具策略

当前反汇编/汇编工具仍保持保守：

- 已确认的 TEXT、RUBY_TEXT、JUMP、CHOICE_DISPATCH、CHOICE_BRANCH 等输出语义化行。
- 未完全确认长度的复杂 SYS/参数块仍输出 `.byte`，保证 zero mutation。
- 下一步若要做 full-rebuild 注入，应把本文件中的顶层 opcode 和表达式语法实装为真正的线性 parser，减少扫描式识别。

## v6 strict parser update

Additional EXE functions supplied by the user fixed the VM grammar used by the full-rebuild injector.

### Expression VM: sub_413350

`sub_413350` is a stack expression evaluator.  Static rebuilding only uses byte-boundary rules:

| token | payload | note |
|---|---:|---|
| default / `00..7F` | none | push small literal |
| `80` | `u8` | table/variable lookup |
| `A0` | `u8` | indexed lookup |
| `C0` | `u8` | indexed byte lookup |
| `E0..F0` | none | runtime helper result pushed |
| `F1` | `u16le` | immediate |
| `F2` | `u32le` | immediate |
| `F3` | `u16le` | address/indexed byte source |
| `F4` | none | expression-indexed byte source |
| `F5` | `u8` | expression-indexed dword table source |
| `F6` | `u8` | dword table source |
| `FF` | none | expression terminator |

Do not terminate an expression at `FF` bytes contained inside `F1/F2/F3` immediates.

### Argument list: sub_4059E0

`sub_4059E0` reads an argument list into the VM argument cache:

```text
arglist := arg* 00
arg     := 01 <cstring> 00
         | 02 <expr_413350>
         | other_tag
```

Tags other than `01`/`02` consume only the tag byte in the EXE.

### Confirmed top-level control opcodes

```text
09 <expr_413350> <target:u32le>
    if expr == 1: skip target and continue
    else: PC = target

0A <target:u32le>
    unconditional absolute jump

0B <sys_id_expr_413350> <arglist_4059E0>
    SYS dispatch; SYS 21 covers ruby/display helper use-cases in current samples

0C <arglist_4059E0>
    context/message string setup; not a jump

0E <arglist_4059E0> <skip_target:u32le>
    register branch entry as current PC after the target field, then PC = skip_target

13
    wait/commit opcode; single byte, no operand
```

## v8 修正：opcode 0x10

通过 `06B.MES` 烟花动画段确认，顶层 `0x10` 不是单字节指令，而是读取 `sub_4059E0` 格式参数列表：

```text
10 <arglist_4059E0>
```

典型样本：

```text
10 02 F1 2C 01 FF 02 00 FF 02 00 FF 02 00 FF
01 "hanabi10.avi" 00
02 06 FF 02 01 FF 02 00 FF 02 00 FF 00
```

若误判为单字节，会导致后续动画循环中的 09/0A target 不被重定位，播放动画结束后跳到旧地址而闪退。
