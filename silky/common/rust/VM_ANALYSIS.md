# Silky MES VM 静态分析

## 证据范围

本文件把原 `silky_op.py` 的指令表和双向编码行为整理为 Rust 实现的静态真值源。没有提供宿主 EXE 或真实 MES 样本，因此业务语义没有升级为运行时确认。

```text
VM 类型：未确认；PUSH/PUSH_STR 暗示 stack-like，但仅为 hypothesis
指令编码：1 byte opcode + 表驱动定长/变长参数
code 起点：8 + message_count*4 + special_count*4
未知字节：单字节 opaque raw node，无损保留
```

## 参数 schema

```text
I    u32 little-endian
i    i32 little-endian
B    u8
>I   u32 big-endian
S    NUL-terminated byte string
```

所有字符串在未修改时保留原始槽字节，不经过解码再编码；这使非法编码字符串也能参与零修改回环。只有修改后的字符串才按 `--encoding` 严格编码。

## 全量 opcode 表

空 mnemonic 表示原 Python 表未确认语义，只确认了边界和参数 schema。

| opcode | mnemonic | schema | 证据/用途 |
|---:|---|---|---|
| 00 | RETURN | - | 块/ruby 结构 |
| 01 | - | I | 原注释：仅 LIBLARY.LIB |
| 02 | - | - | opaque semantic |
| 03 | - | - | 原注释：仅 LIBLARY.LIB |
| 04 | - | - | opaque semantic |
| 05 | - | - | opaque semantic |
| 06 | - | - | 原注释：仅 LIBLARY.LIB |
| 0A | STR_CRYPT | S | 变换字符串 |
| 0B | STR_UNCRYPT | S | 直接编码字符串 |
| 0C | - | - | opaque semantic |
| 0D | - | - | opaque semantic |
| 0E | - | - | opaque semantic |
| 0F | - | - | opaque semantic |
| 10 | - | B | opaque semantic |
| 11 | - | - | opaque semantic |
| 14 | JUMP | >I | code-relative target，重定位 |
| 15 | MSG_OFSETTER | >I | code-relative target，重定位 |
| 16 | SPEC_OFSETTER | >I | code-relative target，重定位；原注释仅 LIBLARY.LIB |
| 17 | - | - | opaque semantic |
| 18 | - | - | 名字块终止结构 |
| 19 | MESSAGE | >I | 文本块入口/消息编号 |
| 1A | - | >I | 块终止；参数是否 offset 未确认 |
| 1B | - | >I | code-relative target，重定位 |
| 1C | TO_NEW_STRING | B | 0=字面换段，1=ruby 入口 |
| 32 | PUSH | i | 名字块等结构 |
| 33 | PUSH_STR | S | 直接编码字符串/可写名字槽 |
| 34 | - | - | 名字 Pattern B |
| 35 | - | - | opaque semantic |
| 36 | JUMP_2 | B | 不是物理 code offset |
| 37 | - | - | opaque semantic |
| 38 | - | - | opaque semantic |
| 3A | - | - | opaque semantic |
| 3B | - | - | opaque semantic |
| 3C | - | - | opaque semantic |
| 3D | - | - | opaque semantic |
| 3E | - | - | opaque semantic |
| 3F | - | - | opaque semantic |
| 40 | - | - | opaque semantic |
| 41 | - | - | opaque semantic |
| 42 | - | - | opaque semantic |
| 43 | - | - | opaque semantic |
| FA | - | - | opaque semantic |
| FB | - | - | opaque semantic |
| FC | - | - | opaque semantic |
| FD | - | - | opaque semantic |
| FE | - | - | opaque semantic |
| FF | - | - | opaque semantic |

没有主 opcode 的子 opcode 表；`0x1C` 的单字节参数目前按已观察结构区分 0/1 变体。发现其他参数值时只保留，不猜测语义。

## offset 与重定位

header 内两组 offset 和 `0x14/0x15/0x16/0x1B` 参数都是 code 区相对 offset。Rust IR 为每个已解析 command/raw node 保存旧 code offset；重建先计算所有新边界，再执行：

```text
old code boundary -> new code boundary
header first_offsets  -> relocate
header second_offsets -> relocate
0x14/15/16/1B target  -> relocate and write u32be
```

任何 target 不位于已解析 node 边界都会拒绝解析/重建，不进行猜测。`0x1A` 虽然是 `>I`，原 Python offset 表没有把它标为物理 offset，因此当前只无损保留数值。

## STR_CRYPT

`0x0A` 的字符串不是通用加密层。原实现行为：

- CP932/Shift-JIS 双字节值 `0x829F..0x831E` 可压成 `value - 0x829E` 的单字节值（结果 `< 0x81`）。
- 解码时 `< 0x81` 的单字节按 `(byte - 0x7D62) & 0xFFFF` 恢复为双字节。
- 其他多字节序列按原字节保留。
- `0x0B` 和 `0x33` 不做此变换。

Rust LZSS 和 STR_CRYPT 都有 Python 参考行为测试；这证明编码兼容，不证明渲染语义。

## 未知与异常策略

- 不在 opcode 表中的单字节作为 `Raw(u8)` 保留。
- 在 opcode 表中的字节按该 schema 解析；如果实际数据区恰好含相同字节，仍可能被识别为指令，这是继承原表驱动格式的已知限制。
- 截断参数、未终止字符串、越界 header、非边界 jump 立即报错。
- 无法按指定编码解码的字符串保留原始字节并 warning；相关翻译块不导出。
- 工具不暴露或接受语义 asm，翻译交换格式只使用 UTF-8 JSON。
