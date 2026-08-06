# MyMerryMayWithbe 项目档案

## 项目与边界

- 游戏：`DataMA`、`DataMB`、`DataMS`；引擎：MGS。
- 原始目录：`E:\BaiduNetdiskDownload\MyMerryMayWithbe`。
- 开发目录：`H:\IDA-PRO-MCP\merry_mpk_tool_dev`。
- 用户指定工具目录：`E:\BaiduNetdiskDownload\MyMerryMayWithbe\work`；未收到最终更新通知前不自动同步或覆盖。
- 实现：Rust `merry-mpk-tool`，release 入口 `mpk_tool.exe`。
- 原始游戏文件只读；验证输出放在开发工作区临时目录。
- 无参数双击进入 `1 自动解包/提取 JSON / 2 构建脚本及字库 / 3 解封包与验证 /
  4 仅重建 BIN/PNG / 0 退出` 的中文交互菜单；原命令行与拖放入口继续保留。
- 所有交互路径提示支持 `0` 返回、成对引号清理、中文错误和原地重试；主菜单及
  归档子菜单可直接粘贴路径并按 MPK、manifest、JSON、MSB/SCX 自动分流。
- 写操作执行前显示资源统计、默认/实际输出、覆盖状态并确认；提取和字体构建显示
  进度。完成页显示下一步，按 Enter 返回清屏菜单，菜单保留上一项简短结果。

## MPK（confirmed-structure）

- magic `MPK\0`，小端版本 `0x00020000`，`0x40` 字节头。
- `+0x08` 为文件数；记录从 `0x40` 开始，每项 `0x100` 字节。
- 记录字段：状态、顺序号、绝对 offset、stored/original size、224 字节 CP932 名称。
- 载荷和成员按 `0x800` 对齐；当前样本均为未压缩、零填充。
- manifest 保留原始头/记录/名称字节；封包只对当前未压缩成员重建 offset 和 size。
- 压缩状态和未知压缩算法为 `opaque-preserved`，工具拒绝猜测解压。

## `.msb` / `.scx`（confirmed-structure）

- `mes00.mpk` 成员是 `.msb`：`MES\0`、版本 1、`16` 字节头、记录表
  `(u32 id, u32 relative_body_offset)`，body base 为 `16 + count * 8`。
- `script.mpk` 成员是 `.scx`：`SC3\0`，`fc <= f4 <= f8 <= file_length`；
  `f4..f8` 是排序的字符串块绝对指针表，尾部含 code-offset vector 和字符串块。
- 两种脚本的字形令牌均为 `80 00 + big-endian u16 glyph index`；值直接索引
  `font_df_jpn.bin` 的 `8194` 个 `16` 字节记录。
- 文本尾部必须按令牌边界识别：只有边界上的 `03`/`08` 才与 `FF` 组成两字节后缀；
  glyph index 的低字节为 `03`/`08` 时仍属于完整四字节字形令牌，并只把其后的
  `FF` 作为后缀。其他历史上作为 `<b:FF>` 提取的单独 `FF` 保持旧工作区兼容。
- 不属于令牌的控制字节、未知 opcode、字符串后缀和 vector 全部原样保留。
- `.msb` 变长注入重建所有记录相对 offset；`.scx` 变长注入重建字符串尾和
  `f4..f8` 指针，不修改 code 区和 vector。

## 文本合同（用户确认）

- 交换格式只使用 UTF-8 JSON 数组。
- 自动工作区对翻译者只输出 `name`（有名记录）和 `message`；等价于用户给出的
  `name` / `[　]*(?P<message>.+)` 提取规则，原正文开头的 U+3000 全角空格在回注时
  由内部元数据恢复。翻译文件和数组条目不得删除、增加或调换顺序。
- 完整 `_file/_index/scr_msg/_scr_name/_raw_body` 保存在工作区 `.mpk_tool/source_json`，
  不暴露给翻译编辑，但合并回注时继续承担定位和不可变源校验。
- `scr_msg` 是不可修改的源投影；实际写回字段是 `message`。
- `.msb` 记录开头的 `01` 分隔名字、`02` 分隔正文；有名记录输出 `name`，名字允许翻译。
- `_scr_name` 保存并校验原始名字；无名记录不自动添加名字。
- `_file`、`_index`、`_id`、`_offset`、`_size`、`_raw_body` 和控制/令牌元数据用于注入校验。
- `<g:XXXX>` 是直接 glyph index 的提取层转义；`<b:XX>` 是单个不透明控制字节。
  重复视觉槽位保留 `<g:XXXX>`，不能按 Unicode 字符规范化。
- 目标字符先查内置 `assets/subs_cn_jp.json` 的 3025 条“简中 -> 日文载体”映射，
  再解析载体 glyph index；无法编码的字符报错并列出码点。

## 位图字库造字（confirmed-structure）

- `system_win.mpk` 含 `font_df_jpn.bin/.png` 和 `font2_df_jpn.bin/.png` 两套。
- BIN 为 `8194 * 16 = 131104` 字节；PNG 为 8-bit 灰度 `4096x7546`。
- 记录 `+0x04..+0x0B` 提供 atlas crop 的 x/y/width/height；字段名称仍按
  `layout candidate` 记录，不宣称运行时语义。
- `font-build` 读取原 BIN 槽位，使用 TTF 在 carrier crop 内重绘目标字符；BIN
  字节和 glyph index 不变。`--donor` 可按顺序提供补字 TTF，缺字时不写输出。
- 交互模式 2 扫描翻译 JSON 的 `message/name` 最终实际用字，统一重绘所有已识别
  可见字符：映射简体字按目标字画入载体槽位，原字库直用汉字、标点等按自身重绘；
  空白、控制字节、`<g:XXXX>` 和未被译文引用的 UI 字形保持原样。主 TTF 缺字时
  循环要求补字 TTF，直到完整。
- 模式 2 明确同时构建重定位 MSB/SCX 与 BIN/PNG；模式 4 可只重建字库。自动工作区
  分别输出到根部 `chs` 和 `font_chs`，不要求用户进入 `.mpk_tool` 内部目录。
- 字库写入前按最终文本检测：不同目标字共用槽位、载体字仍被直接使用、
  `<g:XXXX>` 仍直接引用待重绘槽位。冲突时列出字、glyph index 和位置并拒绝构建。
- DataMS 只有 glyph `0x0101` 的 BIN/PNG 槽位与 MA/MB 不同；工具不会统一覆盖该差异。

## 验证记录

- Rust 单元测试：28 passed；包括精简 JSON 投影/合并、全角空格恢复、UTF-8 控制串
  边界、多换行控制恢复、载体字冲突和
  `<g:XXXX>` 槽位冲突；`fmt`、`clippy -D warnings`、offline release build 通过。
- DataMA/DataMB/DataMS 的 `mes00` 与 `script` 目录：898 个脚本文件、297,457 条记录，
  extract -> unchanged inject 全部 `patched=0`，输出树 SHA-256 与源树完全一致。
- DataMB 两套真实 atlas：3018 槽位/套，输出尺寸保持 `4096x7546`，BIN SHA-256 完全一致，
  PNG 解码尺寸一致且文件内容发生预期变化。
- 单条 `.msb`/`.scx` 短文本、变长文本和指针 relocation 已通过真实样本验证。
- 交互模式 2 已用 DataMB `mb_a01.msb` 和两套真实字库验证：简体名字/正文变长注入、
  实际用字筛选、主 TTF 缺字、补字 TTF、`chs` 覆盖、两套 PNG 重绘、BIN SHA-256
  不变及重建脚本 byte-exact 自检均通过。
- 2026-08-05 使用隔离 DataMA 工作区全量验证统一重绘：378 个脚本、44,046 条记录、
  3,249 个实际可见字形，主宋体加 `seguisym.ttf` 补齐 7 个符号；脚本注入完成
  `patched=43410`、`unchanged=636`，两套字库共重绘 6,498 个槽位。三包重封并再解包
  后共 403 个成员 SHA-256 全部一致，BIN 保持 byte-exact，PNG 尺寸仍为 `4096x7546`。
- 2026-08-05 修复 outline baseline 定位：按 TTF 原点和原 atlas crop 底部锚点绘制；
  DataMA v2 实测 `一` 槽位 `0x05CB` 从空白恢复为有效像素，`，。！？「」…` 均回到
  槽位底部，最终 release EXE 的三包重封验证全部通过。
- `chs` 在源目录外临时完成整树注入和字体构建，再一次性移入 `源目录\chs`，避免
  输出目录被递归复制；已有 `chs` 仅在交互确认后覆盖。
- 2026-08-04 已同步到用户指定 `work\merry_mpk_tool`；源码、映射、文档和 release
  EXE 共 15 个交付文件与开发副本 SHA-256 完全一致，正式 EXE 无参数主菜单可启动。
- 2026-08-05 统一重绘实现已同步到用户指定 `work\merry_mpk_tool`；源码、映射、文档
  和根目录 release EXE 的 SHA-256 与开发副本一致，正式 EXE `--help` 通过。
- 2026-08-04 交互优化验收使用隔离 DataMB 小样本：直接粘贴带引号脚本路径、无效
  路径原地重试、各级 `0` 返回、预检取消、提取 `1/1` 进度和完成摘要均通过；将
  一条正文改为“测试”后，Arial 正确报告 2 个缺字，黑体补齐后完成 1 个脚本注入、
  两套字库 `1/2`、`2/2` 构建及 `chs` 输出。
- 2026-08-04 自动工作区验收使用 DataMS 真实三包：自动解包 3 个 MPK/31 个成员，
  递归提取 20 个脚本/20 个 JSON/1013 条记录；翻译视图无任何内部元数据字段。
  将首条正文改为“测试”后完成 1 条变长注入和两套字库重绘，回提为预期载体
  “測試”；综合 `chs` 与独立 `font_chs` 的 BIN 哈希保持不变且 PNG 结果一致。
  `mes00/script/system_win` 重新封包并再解包后分别 10/10、10/10、11/11 个成员哈希
  全部一致。故意保留载体“測”时正确拒绝并报告槽位 `0x0C27` 及条目位置。
- 2026-08-05 使用 DataMA 的 184 个真实翻译 JSON 完成模式 2 全量预检：修复相邻
  `<b:XX>` 控制串后接多字节字符时的 UTF-8 边界 panic，并验证同一控制串内多个
  `0x00` 可按译文换行位置恢复；新增 7 个实际缺失字符映射。正式 EXE 不再崩溃，
  会继续按设计报告未翻译系统文件占用载体槽位的冲突。
- 2026-08-05 DataMA `ma_sc0301.msb#166` 与 `ma_sc0308.msb#23` 证明末尾 glyph
  `0x0203`（`80 00 02 03`）后可直接跟单字节 `FF`。尾部检测已改为按 token 边界
  判断，避免把 glyph 低字节 `03` 误拆为 `03 FF` 后缀；MSB/SCX 共用同一规则。
  两条真实译文分别完成 `patched=1` 并通过脚本 byte-exact 结构回环。开发区完整
  DataMA 源树随后以旧版内部 JSON 完成 139858 条 unchanged 注入；源/输出均为
  406 个文件，相对路径与 SHA-256 全部一致，证明普通 `<b:FF>` 合同未被破坏。

## 未覆盖范围

- MPK 压缩成员尚未实现压缩/解压。
- 字体造字只负责位图 BIN/PNG，不改变游戏 glyph index；输入 TTF 必须能被 `ab_glyph` 解析。
- `<g:XXXX>` 的未知视觉语义、部分控制 opcode 的运行时含义仍标为 `opaque-preserved`，
  但不影响当前结构化零修改回环和已确认的注入路径。
