# AI5WIN

这个目录收集 AI5WIN / Silky's 系相关游戏的资源处理工具。各子目录大多按具体作品或资源格式拆开，包含 ARC 解包/封包、MES 文本提取注入、LZSS 编解码、字体和图片辅助工具。

这里的 `v1`、`v4`、`v6`、`v7` 等名称只是本仓库整理时留下的本地变体标签，不应当当作真实引擎版本号使用。AI5WIN 各作品格式差异很大，判断格式时应以实际 ARC 索引结构、MES 压缩方式、脚本头结构、opcode 表和跳转规则为准。

## 目录概览

| 子目录 | 主要内容 |
|---|---|
| `common/` | 通用 AI5WIN ARC/MES 试验性封装，包含可猜测 ARC scheme 的解包/封包逻辑，以及 v0/v1/v2 三套通用 MES 文本提取注入入口。 |
| `BE-YOND/` | BE-YOND 用 ARC 与 MES 工具。MES 为 LZSS 压缩脚本，另有较完整的反汇编/重汇编分支。 |
| `ドラゴンナイト4 Windows版/` | Dragon Knight 4 Windows 版工具。MES 为 LZSS 压缩，解压后有消息数量与 offset 表。 |
| `女系家族～淫謀～/` | 女系家族用 ARC、MES、bytecode 与 codec 工具。剧情 MES 压缩，部分 UI MES 为特殊未压缩格式。 |
| `愛しの言霊/` | 愛しの言霊用 ARC/MES 工具和结构分析文档。MES 从 ARC 中读取时需要 LZSS 解压，脚本结构有独立 opcode/V/C 表达式规则。 |
| `恋姬/` | 偏通用的 AI5WIN MES 解析、提取与注入工具。结构化程度较高，重点是 text slot、jump ref 和整体重建。 |
| `らいむいろ戦奇譚/` | らいむいろ戦奇譚用 ARC/MES 工具。ARC 数据区基本原样存储，MES 内部为 LZSS 压缩脚本。 |
| `らいむいろ流奇譚X cross～恋、教ヘテクダサイ。～/` | らいむいろ流奇譚X cross 用 ARC/MES 工具。ARC key 可从 EXE 或 key 文件取得，MES 为压缩脚本。 |
| `リフレインブルー/` | Refrain Blue 用 ARC、MES、字体修补与完整反汇编工具。既有保守的跳板式注入，也有 full rebuild 方向的分析代码。 |
| `百鬼/` | 百鬼用 ARC、MES、opcode、汇编/反汇编工具，并带有 Rust 原型。MES 外层为 LZSS，内部为混合文本/VM 流。 |
| `勝 あしたの雪之丞2/` | あしたの雪之丞2 用 ARC/MES、字体、图片、GUI 集成工具。文本注入、CP932 借码、字体生成和 EXE patch 流程较完整。 |
| `臭作/` | AIWIN/旧 ELF 系 ARC 与 GPX 图片工具。`.MES` 条目需要条目级 LZ 解压，`.GPX` 图像通常 raw 切出后再单独转换。 |
| `下级生/` | 下级生 ARC 解包工具。目录项使用 count 派生 key 与字节置换，数据区原样读取。 |
| `同级生2/` | 同级生2 ARC 解包与按 manifest 重封包工具。目录项固定 XOR 加密，数据区原样拼接。 |
| `mmo format/` | MMO 图像格式工具。用于 MMO 到 PNG 的解码，包含 Python 实现和 C 加速核心。 |

## 格式要点

AI5WIN 相关格式没有一个可以直接套用到所有游戏的统一版本。处理新作品或复用旧工具前，优先确认下面几件事：

- ARC 目录项长度、文件名长度、字段顺序和 XOR key 是否一致。
- ARC 数据区是原样 blob，还是按扩展名或资源类型做了条目级压缩。
- MES 文件本身是否 LZSS 压缩，以及解压后的脚本是否有 header。
- MES header 中的 offset 是相对 bytecode 起点，还是文件绝对偏移。
- 文本 opcode 是否只是 `0x01 <cstring> 00`，还是还有 SYSTEM_TEXT、MESSAGE、隐式 SJIS run、ruby、choice、arglist 字符串等变体。
- 注入时是否需要修正跳转、消息 offset 表、入口表或其它地址字段。

多数专用 ARC 工具只负责索引加解密和 data blob 切分，不在 ARC 层解压。很多作品的 MES 压缩发生在文件自身，封包时应把已重建的 MES 再按对应 MES codec 压回去，而不是把所有 ARC 条目统一重压缩。

## 推荐工作习惯

1. 先保留原始 ARC、MES、EXE 和字体资源备份。
2. 对 ARC 先做 list/unpack，确认条目数量、首个 data offset、最后条目 end offset 都合理。
3. 对 MES 先做 identity roundtrip：提取后不改 JSON 直接注入，再重新提取比对文本数量和原文。
4. 变长注入必须确认 jump fixup 或 trampoline 策略已经覆盖对应作品的控制流。
5. 字体相关工具通常和文本注入的编码策略绑定，尤其是 CP932 借码映射，`scan_chars`、`replace_map`、`font_gen`、`inject` 应使用同一套映射。
6. 注入后建议同时做二进制检查、重新提取检查和游戏内实机检查。

## 维护备注

- 不要只根据目录里的 `vX` 名称判断格式。
- 不要把某个作品的 opcode 表、LZSS 变体或 ARC key 直接套到另一个作品。
- 不要把翻译 JSON 的 `scr_msg` 当成可随意修改字段，它通常用于定位和校验原文。
- 二进制产物、预览图、字体构建输出和缓存文件只作为资源或验证结果保存，不代表通用格式定义。
