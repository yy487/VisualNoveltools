# YU-NO PC-98 disk, archive, and MES localization tools

用于 `この世の果てで恋を唄う少女 YU-NO` PC-98 版 Anex86 HDI、`YUNO_A` 到 `YUNO_Q` 第二层 AI5 资源容器，以及内部 `.MES` 脚本的结构化解包、文本提取、注入和封包工具。

工具只读取原始 HDI/容器，所有输出使用新目录或新文件；不会覆盖源碟、源容器或已有输出，也不会在 `yuno_unpacked` 内写入。

## 支持范围

- Anex86 0x1000 字节头及 PC-98 CHS 分区表。
- 样本使用的单 FAT16 分区、8.3 目录项和嵌套目录。
- 以 CP932/Shift-JIS 解码短文件名；不能安全映射到 Windows 的名称会使用 `__raw_HEX`。
- 提取所有常规文件，并记录目录结构和稳定定位元数据。
- 修改现有文件，可变长或变短；按需保留、释放和分配 FAT16 簇。
- 更新主 FAT 及备份 FAT 中受修改影响的表项。
- 无修改封碟保持整个 HDI 字节完全一致。

当前不支持新增、删除或改名，不支持 FAT12/FAT32、长文件名写回或重建不同几何的新空白 HDI。

## 解碟

```powershell
unpack_hdi.exe yuno.hdi
unpack_hdi.exe yuno.hdi --output yuno_unpacked
```

默认输出为输入旁的 `yuno_unpacked`。输出目录必须不存在。

Windows 可把一个 HDI 直接拖到 `unpack_hdi.exe` 上，效果与第一条命令相同。路径可含空格、`&`、中文和日文。

输出目录包含所有碟内文件以及 `.hdi_manifest.json`。manifest 使用 UTF-8 JSON，记录：

- 源 HDI 的 SHA-256 和长度。
- HDI 几何、分区、BPB、FAT 和数据区布局。
- 每个文件/目录的宿主路径、原始 8.3 名称、属性、目录项偏移、首簇、完整簇链和原始大小。
- 每个常规文件的 SHA-256。

不要删除或编辑 manifest；它不是翻译文件。

## 封碟

```powershell
pack_hdi.exe yuno.hdi yuno_unpacked
pack_hdi.exe yuno.hdi yuno_unpacked --output yuno_packed.hdi
```

默认输出为源碟旁的 `yuno_packed.hdi`。输出文件必须不存在，且不能是源碟。

封碟前会完整校验：

1. 原始 HDI 的大小和 SHA-256 必须与 manifest 一致。
2. manifest 中的布局和全部目录项必须与原始 HDI 一致。
3. 解碟目录不得缺文件、增加文件、改名或改变目录类型。
4. 全部修改必须能装入现有簇和空闲簇。
5. 内存中完成回封后重新解析，并逐个比对 740 个文件，再写输出。

可把源 HDI 和解碟目录同时拖到 `pack_hdi.exe`；Windows 传参顺序必须是“HDI、目录”。为避免资源管理器多选顺序不确定，推荐使用命令行。

## FAT 副本策略

此 YU-NO 样本的两份 FAT 原本就有 14,063 个不一致表项：主 FAT 可完整遍历全部文件，备份 FAT 大片区域为零。该状态属于已确认的源碟事实，不是工具生成的错误。

- 读取和空闲簇判断以主 FAT 为准。
- 无修改封碟不改任何 FAT 字节，因此整碟可 byte-exact。
- 有修改时只把受影响簇的表项写入每份 FAT，不把主 FAT 全量覆盖到备份 FAT。
- 每次操作都会报告 `fat_mismatch_entries` 或 `source_fat_mismatch_entries`。

## 第二层 AI5 资源容器

`unpack_yuno.exe` 和 `pack_yuno.exe` 只接受 `YUNO_A` 到 `YUNO_Q`。`EVE` 属于另一个游戏目录，`YUNO_ED.COM` / `YUNO_ED.S` 也不是这些资源容器，工具会明确拒绝。

已由 17 个真实容器和 `AI5X.EXE` 读取代码确认的格式：

- 4 字节头：小端 `u16` 条目数和 `u16` 密钥；全部样本密钥为 `0x5501`。
- 每条目录记录 20 字节：14 字节 NUL 填充文件名、`u32` 载荷相对偏移、`u16` 长度。
- 目录逐字节使用 `ROR + XOR` 解密，XOR 字节每处理一字节递增；封包应用精确逆变换。
- 数据区偏移为 `4 + 条目数 * 20`，资源按表顺序连续存放，没有对齐或尾部数据。
- AI5X 读取目录时会额外读入首个资源的 4 字节；工具验证该运行时边界，但不会把它误当成目录记录。

### 解包资源

```powershell
$root = 'yuno_unpacked\YU-NO'
$archives = [char[]]'ABCDEFGHIJKLMNOPQ' | ForEach-Object { Join-Path $root ("YUNO_" + $_) }
unpack_yuno.exe $archives --output res
```

输出为 `res\YUNO_A` 到 `res\YUNO_Q`，共 2,719 个资源；每个目录另有一个 UTF-8 `.yuno_manifest.json`。manifest 记录原始头、目录顺序、14 字节原始名称、偏移、长度、类型和 SHA-256，不是翻译文件，不要编辑或删除。

可以把一个或多个 `YUNO_X` 同时拖到 `unpack_yuno.exe`。不指定 `--output` 时，每个输入默认解到旁边的 `YUNO_X_unpacked`；指定 `--output` 时，该路径作为共同根目录。

### 封包资源

```powershell
pack_yuno.exe res\YUNO_A --output YUNO_A_packed
pack_yuno.exe res\YUNO_A res\YUNO_B res\YUNO_C
```

单个输入可用 `--output` 指定精确输出文件。多输入不允许 `--output`，每个目录默认在旁边生成 `YUNO_X_packed`。

封包前会完整校验：

1. 目录名、manifest 格式、表头、条目顺序、原始名称和原始哈希。
2. 目录中不得缺少、增加、改名资源，不得包含子目录或符号链接。
3. 资源可以变长或变短；工具重算所有后续偏移，但单个资源不得超过 65,535 字节。
4. 重建后先在内存中重新解析并逐个验证资源哈希，再创建输出文件。
5. 资源未修改时必须生成 byte-exact 容器，否则拒绝写出。

`.GP4`、音频和其他非 `.MES` 资源仍作为不透明内容无损保留。

## MES 压缩与脚本结构

`.MES` 使用 AI5X 的 4 KiB LZSS 变体：

- 4 KiB 零初始化字典，初始写入位置为 1，位流按 MSB-first 读取。
- `1 + 8 bit` 表示 literal；`0 + 12 bit` 表示字典索引。
- 索引 0 终止；非零索引后接 4 bit，匹配长度为 2 到 17。
- 解压数据没有长度头；开头的小端 `u16` 是脚本入口，也是字符字典末尾。

脚本文本使用 AI5 PC-98 令牌编码，而不是裸 CP932：

- `0xD0..0xFF` 固定映射到最多 48 个脚本字典字符。
- `0x60..0x7F`、`0xC0..0xCF` 加 `0x20` 后作为双字节 CP932 的 lead byte。
- 运行时命令 `0x11` 开始一条显示文本；遍历器按脚本令牌边界跳过 `06...06` 字符串和 `07/08/09` 立即数。
- `0x13`、`0x16` 可出现在可见文本片段之间；工具将它们保存为不可编辑控制字节。

运行时使用实时 `SI` 指针和 `01/00` 嵌套块处理控制流，没有需要随正文长度修正的序列化字节跳转。工具可整体重建解压脚本并重新压缩，因此正文和静态角色名可以变长或变短；解压脚本仍不得超过 `0xFFFF` 字节，外层单个资源仍不得超过 65,535 字节。

### 单文件压缩工具

```powershell
unpack_mes.exe 01.MES
pack_mes.exe 01.MES.decoded
verify_mes.exe 01.MES
```

默认分别生成 `01.MES.decoded` 和 `01.MES.packed.MES`，已有输出一律拒绝覆盖。二者用于分析和验证；正式翻译使用下面的 JSON 工作流。

## MES 文本提取

```powershell
extract_mes.exe res --output res_json
extract_mes.exe res\YUNO_A\01.MES
```

目录输入会递归扫描 `.MES`，并按原相对路径生成独立的 `*.MES.json`；不会把不同脚本混在一个文件。默认目录输出为输入旁的 `res_json`，单文件默认输出为 `01.MES.json`。输出必须不存在。

翻译 JSON 使用 UTF-8 无 BOM，格式标识为 `yuno-pc98-mes-v1`。主要字段：

- `scr_msg` / `scr_msg_parts`：不可修改的原文与定位校验。
- `message` / `message_parts`：实际写回的正文；提取时与原文相同。
- `_scr_name`：静态角色原名，只用于校验；`name` 是允许写回的静态角色名。
- `_file`、`_index`、`_inst_offset`、`_offset`、`_size`、`_source_sha256`：不可修改的来源和稳定定位元数据。
- `_message_controls`、`_name_controls`：不可删除、修改或重排的控制字节元数据。

普通静态角色名示例：

```json
{
  "_scr_name": "亜由美",
  "name": "亜由美",
  "scr_msg": "こんにちは。",
  "message": "こんにちは。"
}
```

男主姓名由游戏内编辑并在运行时展开。`【 13 30 03 】` 对应条目固定输出为：

```json
{
  "_name_dynamic": true,
  "_name_controls": [
    {
      "after_part": 0,
      "offset": 527,
      "hex": "133003"
    }
  ],
  "scr_msg": "俺はいったい何をやっているんだ。",
  "message": "俺はいったい何をやっているんだ。"
}
```

动态名条目不会生成 `name` 或 `_scr_name`。注入器拒绝给它添加名字，并原样保留 `133003`；因此不会把玩家设置的男主名固化进脚本。

正文被控制命令分隔时使用 `scr_msg_parts` / `message_parts`。只能修改各个 `message_parts` 的文字，不能改变 parts 数量或控制元数据。

## MES 文本注入

```powershell
inject_mes.exe res res_json --output res_injected
inject_mes.exe res\YUNO_A\01.MES res_json\YUNO_A\01.MES.json
```

目录注入先完整复制源 `res` 树，再只替换有 JSON 的 `.MES`，因此 `.GP4`、音频、manifest 和其他未知资源都会保留。源目录、源 `.MES` 和 `res_json` 均不会修改；输出已存在时拒绝覆盖。

注入前会校验 JSON 格式、来源路径、SHA-256、条目数、索引、原文、静态原名、位置、类型、控制字节和 parts 布局。`scr_msg`、`scr_msg_parts` 或其他元数据不一致时整次操作失败，不产生半成品输出。零修改 JSON 会直接复用原压缩字节。

写回字符必须是脚本字典字符，或可表示为该令牌格式支持的双字节 CP932。不可编码字符、ASCII/半角单字节字符、NUL、换行和结构括号会被明确拒绝；中文本地化仍需要另行完成引擎编码和字体扩展。

翻译完成后的封包顺序：

```powershell
pack_yuno.exe res_injected\YUNO_A --output YUNO_A_packed
pack_hdi.exe yuno.hdi yuno_unpacked_modified --output yuno_packed.hdi
```

不要在原 `yuno_unpacked` 或原 `res` 内直接修改。把回封的 `YUNO_A` 到 `YUNO_Q` 放入单独的解碟副本，再生成新 HDI。

## 已验证样本

源碟：

```text
bytes   42827776
sha256  9DBFAA9827A6EB7FE50AF12F8B25E126A292740062549F2995BD3294158A4B04
```

真实 HDI 解碟结果：740 个文件、2 个目录、30,447,766 字节文件内容、0 个交叉簇、0 个孤儿簇。

已执行：

- 20 个单元测试：HDI/FAT16、AI5 容器、MES 解压/压缩、静态名、动态男主名、multipart、不可变原文和词法边界。
- `source -> unpack -> unchanged pack -> unpack`：回封 HDI 与源碟 SHA-256 完全相同；两次目录树 741 个文件（含 manifest）无差异。
- 真实修改回环：把 0 字节 `A.TXT` 改为 41 字节，分配 1 个簇；回封再解碟的 740 个碟内文件与修改输入完全一致，且相对原碟仅 `A.TXT` 改变。
- 17 个 `YUNO_A` 到 `YUNO_Q`：共 2,719 个资源，全部完成 `archive -> unpack -> unchanged pack -> unpack`；17 个回封容器均 byte-exact，第二次目录树的 2,736 个文件（含 17 个 manifest）逐个同哈希。
- 真实变长回环：`YUNO_A/D_B01.MES` 从 1,609 增至 1,617 字节，回封容器增加 8 字节；重新解包的 250 个资源全部与修改输入一致。
- 904 个真实 `.MES`：压缩数据 3,132,939 字节，解压数据 6,596,121 字节；全部解压、重压和重新解压成功。
- 文本提取：66,140 条，动态名 42,927 条，multipart 979 条，warning 0。
- 全资源树零修改注入：904 个 JSON、66,140 条，`patched=0`；源与输出均为 2,736 个文件，缺失、额外和 SHA-256 不一致均为 0。
- `01.MES` 真实修改回环：同时修改 multipart、静态名和变长正文，4,808 字节变为 4,824 字节；重新提取的 108 条全部符合 JSON，92 条动态名的控制载荷全部不变。
- 9 个 release EXE 均运行 `--help` 成功；`cargo fmt -- --check`、测试、Clippy `-D warnings` 和离线 release 构建通过。

## 构建

```powershell
cargo fmt -- --check
cargo test --offline
cargo clippy --offline --all-targets -- -D warnings
cargo build --release --offline --bins
```

构建产生 9 个命令：`unpack_hdi`、`pack_hdi`、`unpack_yuno`、`pack_yuno`、`unpack_mes`、`pack_mes`、`extract_mes`、`inject_mes`、`verify_mes`。
