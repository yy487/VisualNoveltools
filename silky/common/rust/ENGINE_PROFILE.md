# Silky common 项目档案

## 项目与边界

```text
项目名称：リルカは幾重に夜を彩る / Silky common
引擎/运行时：Silky Engine（具体版本未确认）
Python 参考实现：上级目录中的 `*.py`
Rust 工具目录：当前目录
正式用户入口：unpack / repack / extract / inject
默认文本与文件名编码：CP932
真实样本：本目录未提供
```

本档案的静态结构事实来自现有 Python 实现和其中记录的项目样本观察；没有宿主程序调试或游戏运行结果时，不标记为 runtime confirmed。

## ARC

### silky-lzss

证据等级：`confirmed-structure`（由原解析/回封代码和 Rust 合成回环证明）。

```text
header_size：文件偏移 0 的 u32 little-endian，不含自身 4 字节
记录：u8 name_len + encrypted_name + u32be size + u32be unpacked_size + u32be offset
文件名变换：从尾到头按 1,2,3... 加/减
压缩判断：size != unpacked_size
数据：offset/size 指向文件载荷；压缩项使用 4096/18/2 参数的 LZSS
```

Rust 解析器拒绝头越界、记录截断、文件越界、重叠、重复路径、路径穿越、Windows drive prefix 和解压长度不一致。

### garbro-fixed

证据等级：`confirmed-structure`（由原解析/回封代码和 Rust 合成回环证明）。

```text
count：文件偏移 0 的 u32 little-endian
记录：0x20 字节 NUL padding 文件名 + u32le offset + u32le size
压缩：无
```

未知 ARC 间隙和尾随数据：`opaque/unconfirmed`。当前 manifest 不保存这类外层填充，因此不承诺外层 ARC byte-exact；解包文件、顺序和载荷是回封验证对象。

## MES

证据等级：

- header、opcode 编码、字符串槽、offset 基准：`confirmed-structure`（原 Python 双向实现与 Rust 合成 byte-exact 回环）。
- 指令业务语义和 VM 架构：`hypothesis`，未做宿主程序运行时确认。
- 未识别 opcode 字节：`opaque-preserved`。

```text
header：u32le message_count + u32le special_count
header 表：message_count 个 u32le offset，随后 special_count 个 u32le offset
offset 基准：MES code 区起始（header 之后）
opcode：单字节；参数由固定表决定
字符串：NUL terminated；0x0A 使用 STR_CRYPT 变换，0x0B/0x33 为直接编码
offset 参数：0x14/0x15/0x16/0x1B 的首参数为 u32 big-endian code offset
```

正文变长时整体重建 code 区，并重定位 header 表与上述 offset 参数。工具不改变指令数量或顺序。

## 用户确认的 name/message 规则

该规则继承现有项目 README 和 `silky_common.py`，不是跨 Silky 引擎通用规则。

有名对话的名字来源：

```text
Pattern A：PUSH_STR[name] -> PUSH[special] -> PUSH[...] -> 0x18
Pattern B：PUSH_STR[name] -> PUSH[special] -> PUSH[...] -> 0x34 -> PUSH[...] -> 0x18
special：83886080 / 117440512 / 167772160
```

- `name` 从首个 `PUSH_STR` 字符串槽取得，必须包含非 ASCII 字符。
- `message` 来自同一文本块内的 `STR_CRYPT/STR_UNCRYPT` 及已确认 ruby base。
- 无名字块时按无名正文导出，不猜测说话人。
- 角色名是可写字符串槽，允许翻译并写回。
- Rust 新提取增加 `_scr_name` 保存原名并严格校验；旧 JSON 无 `_scr_name` 时兼容写回并 warning。
- 选择项、系统文本和特殊名字变体：未单独确认；当前只按已证实文本块导出。

发现不符合上述模式的真实样本时，先统计并让用户确认，不能把候选规则直接加入注入器。

## 控制符与 ruby

### TO_NEW_STRING[0]

```text
原始形式：opcode 0x1C，参数 0
位置：正文块内
提取：JSON message 中的字面 \\n
注入：要求数量不变，按原节点保留
语义级别：confirmed-structure；最终显示语义未做 runtime 确认
```

### ruby

```text
结构：TO_NEW_STRING[1] -> reading/separator STR* -> RETURN -> STR_CRYPT[base]
提取：只导出 base
注入：每次都把 reading 写成等字符数全角空格（视觉删除全部注音），base 写 message
零修改：底层 parse/rebuild byte-exact；用户级 unchanged inject 对含 ruby 的条目按项目策略产生预期变化
语义级别：结构与项目策略 confirmed；最终渲染未做 runtime 确认
```

## 验证与限制

- Rust 单元测试包含合成 MES byte-exact 回环、正文/名字变长重定位、ruby/换行控制、两种 ARC 内容回环和 Python LZSS 参考压缩向量。
- 未提供真实 MES/ARC，因此当前没有“真实样本文件数、条目数、SHA-256”结论。
- 在取得真实样本后，必须补做完整目录 `extract -> unchanged inject` SHA-256 树比较，以及 `ARC -> unpack -> repack -> unpack` 内容/顺序比较。
