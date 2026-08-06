# MyMerryMay MPK / Script / Bitmap Font Tool

`mpk_tool.exe` 用于 `MyMerryMayWithbe` 的 `DataMA`、`DataMB` 和 `DataMS`。
它处理 MPK 解包/封包、`.msb`/`.scx` 的 UTF-8 JSON 提取与注入，以及
`system_win` 位图字库的 TTF 造字。源游戏文件不会被覆盖。

## 双击交互模式

无参数双击 `mpk_tool.exe` 会进入主菜单：

```text
1  自动解包 / 提取翻译 JSON
2  构建翻译资源（MSB/SCX + BIN/PNG）
3  仅解包/封包/验证
4  仅重建 BIN/PNG 字库
0  退出
```

所有路径输入框都支持拖入、粘贴完整路径，并会自动去掉 Windows 拖放附带的成对
引号；输入 `0` 可返回上一级。路径不存在、类型不符或字体无法读取时不会退出程序，
而是在当前步骤显示中文原因并重新输入。

主菜单也可直接粘贴路径，由工具自动进入对应流程：三包游戏目录自动提取、`.mpk`
解包、翻译工作区/JSON 构建、单个 `.msb/.scx` 或旧脚本目录提取、含
`.mpk-manifest.json` 的目录封包。归档子菜单同样会区分 MPK、解包目录和验证输入。

每项写操作开始前都会显示输入、文件/条目/字库数量、输出路径及覆盖状态，并要求
确认。提取显示逐脚本进度，构建显示脚本注入和每套字库进度。操作结束后会给出
输出统计和下一步提示；按 Enter 返回已清屏的菜单，菜单顶部保留上一项简短结果。

### 1 自动解包 / 提取翻译 JSON

推荐输入同一目录内包含以下三个包的 `DataMA`、`DataMB` 或 `DataMS` 目录：

```text
mes00.mpk
script.mpk
system_win.mpk
```

也可输入同时包含多个 `DataM?` 的游戏上级目录。工具按三包组合识别游戏，自动解包，
递归提取 `.msb/.scx`，并在输入旁生成 `<目录名>_translation` 工作区。不会修改源 MPK：

```text
DataMB_translation/
├─ translation_json/       # 只编辑这里
│  ├─ mes00/*.msb.json
│  └─ script/*.scx.json
└─ .mpk_tool/              # 工具内部源文件和定位元数据，不要编辑
   ├─ source/
   ├─ source_json/
   └─ workspace.json
```

翻译 JSON 每项只保留 `name`（有说话人时）和 `message`：

```json
{
  "name": "校长先生",
  "message": "……你也真是倒霉呢"
}
```

这等价于对完整提取结果按用户规则读取 `name`，并用
`[　]*(?P<message>.+)` 取得正文：开头原有的全角空格不暴露给翻译者，构建时由内部
元数据自动恢复。`_file/_index/scr_msg/_scr_name`、控制符和原始字节仍保存在
`.mpk_tool/source_json`，用于按文件和索引校验回注。翻译 JSON 文件和数组条目不能
删除、增加或调换顺序。输出工作区已存在时停止，不覆盖。

旧的单个 `.msb/.scx` 和已解包目录仍可提取为完整 JSON，兼容原命令行工作流。

### 2 构建翻译资源

输入模式 1 生成的工作区根目录或 `translation_json`。工具会把精简的
`name/message` 与内部不可变元数据合并，自动定位三包解包树和 `system_win` 两套
字库，不要求再输入 MSB/SCX/BIN/PNG 路径。旧版完整 JSON 继续兼容。

工具静默扫描 `message` 和可写 `name`，收集最终文本实际使用的全部已识别可见字符；
随后要求输入主重绘 TTF。主字体缺字时会列出 `U+XXXX '字'`，继续要求输入补字
TTF，可按顺序反复补充，直到实际用字全部覆盖。映射简体字按目标简体字重绘到载体
槽位，原字库可直接编码的汉字、标点和其他可见字符按自身重绘，从而避免新旧字形
大小、粗细混排。空白、控制字节、`<g:XXXX>` 未确认槽位及未被译文引用的 UI 字形
保持原样；未使用的 3025 条内置映射不会导致无意义的缺字提示。

请求 TTF 之前会按最终 `name/message` 检查字库槽位。如果多个简体目标字占用同一
glyph index、映射载体字本身仍被最终文本直接使用，或 `<g:XXXX>` 仍引用待重绘槽位，
工具会拒绝构建，并列出冲突目标字、载体字、`0xXXXX` 槽位及前几处文件/条目位置。
这类冲突不能忽略，否则原本需要显示载体字的位置会被重绘成另一个简体字。

工作区模式确认后生成工作区根部的 `chs`，完整保留 `mes00/script/system_win` 解包
结构，并把翻译后的 `.msb/.scx` 及重绘后的 `.bin/.png` 放回同一相对路径和文件名。
分别把 `chs` 内含 manifest 的三个目录交给模式 3 封包即可。若旧 JSON 来自单独的
`Data??_mes00` 或 `Data??_script` 目录，脚本写入该目录的 `chs`，字库写入匹配的
`Data??_system_win\chs`。已有 `chs` 时只有明确回答 `y` 才允许整体覆盖；构建失败会
清理本次未完成输出。

脚本中保存的仍是日文载体 glyph index，简体显示来自重绘后的 PNG。因此对已构建
脚本重新提取时看到的是载体字符，不是简体字面值，这是映射方案本身的预期结果。

### 3 仅解包/封包/验证

进入子菜单后选择：

```text
1  解包 MPK
2  封包目录
3  验证文件或目录
0  返回主菜单
```

解包和封包会先显示预测的默认输出，可直接 Enter 接受，也可输入新的输出路径；
输出已存在或上级目录不存在时会在原地要求重输。验证前会显示识别到的文件类型和
脚本/归档成员数量。

### 4 仅重建 BIN/PNG 字库

输入模式 1 工作区或翻译 JSON，执行与模式 2 相同的全文实际用字、缺字和槽位冲突检查，
但不注入 MSB/SCX。工作区模式把所有匹配的 `font_df_jpn`、`font2_df_jpn` 结果写到
根部 `font_chs` 并保持 `system_win` 相对路径；BIN 原样复制，PNG 按 TTF 重绘。

重绘时使用 TTF outline 的真实原点计算 baseline，并沿用原 atlas crop 的底部锚点；
不能把 baseline 当作 crop 左上角。这样可避免窄横线字（例如“一”）被裁成空白，
也避免 `，。！？「」…` 等标点被放到文字行上方。

## 命令行模式

```text
mpk_tool.exe prepare [--output DIR] PACKAGE_DIR
mpk_tool.exe unpack [--output DIR] ARCHIVE.mpk [ARCHIVE.mpk ...]
mpk_tool.exe pack [--output ARCHIVE.mpk] UNPACKED_DIR
mpk_tool.exe extract [--output DIR] SCRIPT_FILE_OR_DIR
mpk_tool.exe inject [--output PATH] SCRIPT_FILE_OR_DIR TRANSLATION_JSON_OR_DIR
mpk_tool.exe font-build [options] FONT_DIR_OR_BIN
mpk_tool.exe verify FILE_OR_DIR
mpk_tool.exe map
```

`prepare` 执行模式 1 的三包自动工作区流程。原有命令行和单参数拖放仍可用：拖入
含三包组合的目录默认 prepare，拖入 `.mpk` 默认解包，拖入含
`.mpk-manifest.json` 的目录默认封包。命令行输出路径已经存在时工具停止，不会
删除或覆盖旧输出；允许覆盖 `chs/font_chs` 只属于对应交互模式的明确确认。

## 脚本与字形

`.msb` 使用 `MES\0`、版本 1、`16 + count * 8` 的记录表；`.scx` 使用 `SC3\0`
和文件内偏移/指针表。两者的正文字形令牌都是：

```text
80 00 <u16 big-endian glyph index>
```

索引直接对应 `font_df_jpn.bin` 的 `8194` 个 `16` 字节记录。未知字节和脚本尾部
保留原样；尾部 `03 FF`/`08 FF` 只在字形令牌边界上识别。glyph index 的低字节
即使恰好为 `03`/`08` 也不会被误拆为后缀，此时只保留其后的 `FF`。变长 `.msb`
会重建记录偏移，变长 `.scx` 会重建字符串尾和 `f4..f8` 指针表。

内部完整提取结果是 UTF-8 JSON 数组。`scr_msg` 永远是不可修改的源投影，注入只写
`message`，名字在有名记录中写入 `name`，并用 `_scr_name` 校验原始名字。自动工作区
把这些字段留在 `.mpk_tool/source_json`，对外只投影 `name/message`：

```json
{
  "_file": "mb_a01.msb",
  "_index": 0,
  "_id": 0,
  "_offset": 1320,
  "_size": 318,
  "_type": "dialogue",
  "_encoding": "glyph-index",
  "_policy": "relocate",
  "name": "主人公",
  "_scr_name": "主人公",
  "scr_msg": "……君も災難だったね",
  "message": "……君も災難だったね"
}
```

无法安全投影的索引写成 `<g:XXXX>`，原始控制字节写成 `<b:XX>`。这些标记是
提取层的转义，不是游戏文件中的文本；注入时会恢复为原始字节。重复视觉槽位
会继续使用 `<g:XXXX>`，确保未修改 JSON 的回注是 byte-exact。

内置 `assets/subs_cn_jp.json`（3025 条）把简体目标字符映射到游戏已有的日文
载体字符。注入时目标字符先查这张表，再解析载体字符的 glyph index；未映射字符
直接报错，不使用 DLL 字符表或 CP932 猜测。

## 位图造字

`font-build` 只负责造字和重绘 atlas，不改 glyph index、BIN 记录或 advance 值。
它读取原始 `font_df_jpn.bin` 的槽位坐标，在对应 `font_df_jpn.png` crop 内用
TTF 绘制内置映射中的简体字符，然后把 BIN 原样复制到 `chs` 输出目录。

```powershell
mpk_tool.exe font-build --font main.ttf --all \
  --output-dir DataMB_system_win\chs DataMB_system_win

mpk_tool.exe font-build --font main.ttf \
  --donor fallback.ttf --donor fallback2.ttf \
  --output-dir DataMB_system_win\chs DataMB_system_win
```

目录输入默认寻找 `font_df_jpn.bin/.png`，默认输出为同目录下的 `chs`；`--all`
在存在时同时生成 `font2_df_jpn.bin/.png`。主 TTF 优先，`--donor` 按出现顺序
补齐缺失字符；仍缺字时列出 Unicode 码点并且不生成半成品。也可用 `--bin`、
`--png` 指定单独的字库对。

## MPK 结构与封包

MPK 使用 `MPK\0`、小端版本 `0x00020000`、`0x100` 字节记录和 `0x800` 对齐的
未压缩成员。解包目录中的 `.mpk-manifest.json` 保留原始头、记录、CP932 名称和
未知字段。封包会重新计算未压缩成员的 offset/size，并拒绝压缩状态、路径穿越、
非零填充和不一致 manifest。

修改后的脚本或字库放回对应解包目录后，再用 `pack` 生成新的 MPK；不要直接替换
原始包。

## 验证

```powershell
cargo fmt -- --check
cargo test --offline
cargo clippy --offline --all-targets -- -D warnings
cargo build --release --offline --bins
target\release\mpk_tool.exe --help
```

真实样本已完成：DataMA/DataMB/DataMS 的 `mes00` 与 `script` 共 6 组目录、898 个
脚本文件、297,457 条 JSON 记录的 extract -> unchanged inject；所有输出文件数、
相对路径和 SHA-256 与源树一致。DataMB 两套真实 atlas 造字也已验证输出尺寸
`4096x7546`、BIN SHA-256 不变、PNG 像素文件发生预期变化。

DataMS 真实三包已完成 `prepare -> 精简 JSON 修改 -> MSB/SCX + 两套字库构建 ->
三个 MPK 封包 -> 再解包`：31 个归档成员、20 个脚本、1013 条翻译记录；修改正文
回提为预期载体“測試”，三个包分别 10/10、10/10、11/11 个成员 SHA-256 全部一致。
载体字直接占用及 `<g:XXXX>` 占用的冲突拒绝已有单元测试和真实交互验证。

已知限制：尚未实现 MPK 压缩成员的解压/压缩；字体造字输入为能被 `ab_glyph`
解析的单字体 TTF，复杂字体集合应先导出单个 TTF。
