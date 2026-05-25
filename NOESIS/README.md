# Noesis / Love es M 本地化工具链

适用对象：Noesis 引擎 `Love es M` 的 `script.iga` 与解包后的 `*.s` 脚本。

本工具按项目 JSON 工作流实现：`scr_msg` 保存原始脚本文本，只用于定位和校验；`message` 为实际写回文本。注入器支持变长重定位，不提供截断/原地覆盖模式。

## 目录结构

```text
noesis_script_tool/
├─ noesis_unpack.py        # script.iga 解包
├─ noesis_pack.py          # script.iga 重封包
├─ noesis_extract.py       # .s -> JSON
├─ noesis_inject.py        # JSON -> .s，支持变长重定位、GBK 写回、name 字段/字典替换
├─ noesis_pipeline.py      # unpack / pack / extract / inject / roundtrip
└─ noesis/
   ├─ iga_common.py
   ├─ iga_unpack.py
   ├─ iga_pack.py
   ├─ script_common.py
   ├─ script_extract.py
   ├─ script_inject.py
   └─ pipeline.py
```

## 1. IGA0 封包格式

当前样本 `script.iga` 已确认：

- Magic：`IGA0`
- `0x10` 后是 packed uint 编码的索引区长度。
- 索引区由 `name_offset, data_offset, size` 三元组构成，三者都是 Noesis packed uint。
- 索引区后是 packed uint 编码的文件名区长度。
- 文件名区保存文件名字节的 packed uint 序列。
- 数据区紧随文件名区。
- 索引中的 `data_offset` 是相对数据区起点的偏移。
- 条目内容有 XOR：
  - `.s`：`byte[i] ^= ((i + 2) ^ 0xFF) & 0xFF`
  - 其他：`byte[i] ^= (i + 2) & 0xFF`

零修改回包测试已经通过，`script.iga -> unpack -> pack` 可 byte-exact。

## 2. `.s` 脚本文本结构

当前只触碰已经确认的文本相关 op 和已确认的脚本内部跳转 offset，其余 VM 字节全部原样保留。

### 2.1 普通文本 / 说话人标识

已确认格式：

```text
?? 04 00 LL <payload>
```

说明：

- 第二、三字节固定为 `04 00`。
- `LL` 是 payload 字节长度，当前样本按 1 字节长度处理。
- `payload` 是原始 CP932 文本，通常第一个 `00` 前是可见文本。
- 第一个 `00` 后面经常夹杂若干旧填充字节；变长注入时会重建为干净的 `message + 00`，不再保留旧填充。
- 样本中存在 `00 04 00 LL`、`3F 04 00 LL`、`0F 04 ...`、`1B 04 ...` 等同类长度字符串结构；工具只导出剧情文本、选项和 dialogue 的说话人上下文，不导出资源名。

说话人标识使用井号前缀：

```text
＃あやか
＃由人
＃彩華
```

正常翻译流程不再把这些 `#name` 单独导出成一堆 `_type=name` 行。提取结果只在 dialogue 条目中保留：

```json
{
  "name": "あやか",
  "scr_msg": "「何だかこういうの普通になっちゃったなあ、って。しみじみしたの」",
  "message": "「何だかこういうの普通になっちゃったなあ、って。しみじみしたの」",
  "_file": "0010.s",
  "_index": 9,
  "_type": "dialogue"
}
```

如果把 `name` 改成译名：

```json
{
  "name": "绫香",
  "scr_msg": "「何だかこういうの普通になっちゃったなあ、って。しみじみしたの」",
  "message": "「只是觉得，像现在这样已经变得习以为常了呢。 我不禁有些感慨。」",
  "_file": "0010.s",
  "_index": 9,
  "_type": "dialogue"
}
```

注入器会根据原脚本中该条 dialogue 对应的原始说话人 `あやか` 推断出映射：

```text
あやか -> 绫香
```

然后在当前脚本内统一把匹配的 `＃あやか` 变长替换为“原始井号控制前缀字节 + 译名”。不需要额外翻译一条 `_type=name` 行，也不会新增一条可显示的 `#绫香` 对话文本。

### 2.2 name dict

提取目录时默认会生成：

```text
_noesis_name_dict.json
```

格式是简单的原名到译名映射：

```json
{
  "あやか": "绫香",
  "由人": "由人"
}
```

注入时如果 JSON 目录中存在 `_noesis_name_dict.json`，会自动读取并对所有脚本统一应用 name 替换。也可以显式指定：

```bash
python noesis_inject.py mes json mes_chs --encoding gbk --name-dict json/_noesis_name_dict.json
```

优先级/规则：

1. `_noesis_name_dict.json` 中 `原名 != 译名` 的项会全局替换所有对应 `#name` op。
2. 如果没有改 name dict，也可以直接改 dialogue 条目的 `name` 字段；注入器会从该条原脚本上下文推断 `原名 -> 译名`，并在当前脚本内统一替换。
3. 如果同一个原名被映射到多个不同译名，注入器会报 conflict warning，避免混乱替换。
4. `--export-names` 仍保留为调试/兼容选项，但正常翻译不推荐使用。

### 2.3 选项跳转

已确认格式：

```text
1D 08 LL LL TT TT TT TT <choice_text>
```

说明：

- `1D 08` 是选项文本 op。
- `LL LL` 是小端文本长度。
- `TT TT TT TT` 是小端跳转目标 offset。
- 后面紧跟 CP932 选项文本，不带 `00` 终止。
- 变长注入会重写 `LL LL`，并根据文本长度变化修正 `TT TT TT TT`。

样本中已识别 19 条选项，例如：

```text
0110H.s 0x6ABB len=8  target=0x00006ADF  中に出す
0110H.s 0x6ACB len=8  target=0x00006F9E  外に出す
0220.s  0x05DC len=12 target=0x0000063A  姉だけとする
0220.s  0x05F0 len=14 target=0x00000876  あやかともする
0220.s  0x0626 len=8  target=0x00000AAA  選べない
```

### 2.4 已修正的跳转 offset

变长注入时会修正以下已确认的脚本内部绝对 offset：

```text
1D 08 LL LL TT TT TT TT <choice_text>   # 选项目标
0D 08 00 00 TT TT TT TT                 # 跳转
3B 08 xx 00 TT TT TT TT                 # 条件分支 / 选择相关分支
```

工具不会把 `0C08`、`1408`、`0E08` 这类看起来带 4 字节数字的 op 当作物理跳转处理。样本中它们更多表现为页号、等待时间、转场时间或资源相关参数。

### 2.5 换页 / 行推进 op

截图中出现的类似：

```text
0C 08 00 00 0A 09 00 00 00 04 00 LL ...
```

当前按结构边界/行推进处理，不导出、不修改。中间字节可能与日文编码偶然重合，但不作为文本处理。正式提取依据是后面的文本 op / choice op，而不是直接扫“像日文”的字节。

## 3. 注音 / ruby 结构

样本中注音形式类似：

```text
<鶴<つる><谷<たに>あやか。
俺、<水<みず><城<しろ><由<よし><人<と>...
```

工具默认在 `message` 字段中删除注音，只保留基字：

```json
{
  "scr_msg": "<鶴<つる><谷<たに>あやか。",
  "message": "鶴谷あやか。",
  "_ruby_stripped": true
}
```

这样 `scr_msg` 仍然保留原始脚本文本用于校验，`message` 则是实际写回文本。写回后注音结构会被删除。

如果希望 `message` 初始也完全等于 `scr_msg`，使用：

```bash
python noesis_extract.py mes json --keep-ruby
```

## 4. 命令

### 4.1 解包

```bash
python noesis_unpack.py script.iga mes
```

### 4.2 提取文本

默认删除注音到 `message`，不单独导出 name 行，并生成 `_noesis_name_dict.json`：

```bash
python noesis_extract.py mes json
```

保留注音：

```bash
python noesis_extract.py mes json --keep-ruby
```

不生成 name dict：

```bash
python noesis_extract.py mes json --no-name-dict
```

仅调试时使用：单独导出 name op 为 `_type=name` 行：

```bash
python noesis_extract.py mes json --export-names
```

### 4.3 变长注入

默认按 CP932 写入 `message` 和被替换的 name：

```bash
python noesis_inject.py mes json mes_chs
```

GBK 写回：

```bash
python noesis_inject.py mes json mes_chs --output-encoding gbk
```

短别名：

```bash
python noesis_inject.py mes json mes_chs --encoding gbk
```

使用指定 name dict：

```bash
python noesis_inject.py mes json mes_chs --encoding gbk --name-dict json/_noesis_name_dict.json
```

新版注入器默认就是变长重定位模式，不再提供截断/原地覆盖模式。它会：

1. 用 `_index + scr_msg` 定位并校验原文。
2. 从 `_noesis_name_dict.json` 和被修改的 dialogue `name` 字段收集 name 映射。
3. 按 `message` 重新编码并重建当前文本/选项指令；编码优先级为命令行 `--output-encoding` > JSON `_output_encoding` > 默认 `cp932`。
4. 对匹配的 `#name` op 只替换 name 本体；井号控制前缀保留原脚本字节，不按 GBK 重新编码，避免游戏把 `#绫香` 当普通正文显示。
5. 重新计算后续字节位置。
6. 修正 `1D08 / 0D08 / 3B08` 中已经确认的脚本内部 offset。
7. 输出新的 `.s` 文件。

限制：普通文本/name op 的长度字段仍是 1 字节，因此单条普通文本或 name 的 `message + 00` 不能超过 255 字节；选项文本长度字段是 2 字节，不能超过 65535 字节。

### 4.4 重封包

```bash
python noesis_pack.py mes_chs script_chs.iga
```

也可以统一使用 pipeline：

```bash
python noesis_pipeline.py unpack script.iga mes
python noesis_pipeline.py extract mes json
python noesis_pipeline.py inject mes json mes_chs --encoding gbk
python noesis_pipeline.py pack mes_chs script_chs.iga
```

## 5. JSON 字段

普通 dialogue 条目示例：

```json
{
  "name": "あやか",
  "scr_msg": "「ちょっと、もう……そういう言葉、使わないでよ、恥ずかしい」",
  "message": "「ちょっと、もう……そういう言葉、使わないでよ、恥ずかしい」",
  "_file": "0010.s",
  "_index": 11,
  "_offset": 1345,
  "_inst_offset": 1341,
  "_size": 64,
  "_visible_size": 60,
  "_type": "dialogue",
  "_opcode": "0004",
  "_encoding": "cp932",
  "_output_encoding": "cp932",
  "_policy": "relocate"
}
```

选项条目示例：

```json
{
  "scr_msg": "中に出す",
  "message": "中に出す",
  "_file": "0110H.s",
  "_index": 342,
  "_offset": 27331,
  "_inst_offset": 27323,
  "_size": 8,
  "_visible_size": 8,
  "_type": "choice",
  "_opcode": "1D08",
  "_target": "0x00006ADF",
  "_encoding": "cp932",
  "_output_encoding": "cp932",
  "_policy": "relocate"
}
```

## 6. 编码策略

- `_encoding` 表示原始 `scr_msg` 的解析/校验编码，当前固定为 `cp932`，不要为了 GBK 注入去改它。
- `_output_encoding` 表示当前条目默认写回 `message` 时使用的编码，默认 `cp932`。
- 命令行 `--output-encoding gbk` 会覆盖所有 JSON 条目的 `_output_encoding`。
- 支持的常用写回编码：`cp932`、`shift_jis`/`sjis`、`gbk`、`gb18030`。
- GBK 模式只负责把 `message` 和 name 编码成 GBK 字节并参与变长重定位；游戏端仍然需要对应 hook/字体层按 GBK 或兼容方案显示，否则运行时可能乱码。

## 7. 当前样本统计

对本次 `mes.zip` 解包目录的统计：

```text
默认提取：
script_files_with_text = 96
entries                = 13369
dialogue               = 7377
monologue              = 5973
choice                 = 19
name_dict_names        = 12

--export-names：
entries                = 20746
name                   = 7377
dialogue               = 7377
monologue              = 5973
choice                 = 19
```

## 8. 已验证

- `script.iga -> unpack -> pack`：byte-exact。
- `mes -> extract --keep-ruby -> inject`：默认不导出 name 行，未修改任何 `message/name` 时，输出目录与原目录 byte-exact。
- 默认提取会生成 `_noesis_name_dict.json`，注入时会自动忽略该文件的文本条目扫描，并作为 name 映射读取。
- 直接修改 dialogue 条目的 `name` 字段：例如在 `0010.s` 中把某条 `name: あやか` 改成 `name: 绫香`，注入器会在该脚本内统一替换所有 `＃あやか`。
- 修改 `_noesis_name_dict.json`：例如 `あやか -> 绫香`，注入器会在全部脚本中统一替换所有 `＃あやか`。
- 默认 `extract` 会把 22 条 ruby 文本的 `message` 改成无注音版本，再注入可以成功生成删除 ruby 后的脚本。
- 普通文本变长测试：`0010.s` index=7 从 64 字节可见正文扩展到 94 字节，注入成功，文件大小自动变化。
- 选项变长测试：`0220.s` 的 `姉だけとする` 扩展后，`1D08` 的长度、选项目标，以及后续 `0D08 / 3B08` 跳转目标均已按 delta 修正。
- GBK 写回测试：`0010.s` 单条 `message` 改为简中并使用 `--output-encoding gbk`，注入成功，输出文件中可定位到 GBK 编码字节。
- GBK name 替换测试：`あやか -> 绫香` 时，name op 中的井号控制前缀保持原始 CP932 `81 94`，只把后面的 name 本体写成 GBK，避免显示成字面量 `#绫香`。

## 9. 已知限制

1. 普通文本/name op 的长度字段是 1 字节，单条普通文本或 name 最大约 254 字节正文 + `00` 终止；超过会报错，不截断。
2. 当前只修正已确认的 `1D08 / 0D08 / 3B08` 物理 offset；如果后续发现新的物理跳转 op，需要继续加入 relocation 表。
3. GBK 注入只改变脚本写回字节；如果运行时 hook 仍按 CP932 解释文本，游戏内会乱码。
4. 同一个原始 name 如果被映射成多个不同译名，会报 conflict warning，应该统一修正 JSON 或 name dict。
5. 工具不会修改资源名、语音名、立绘名、背景名、BGM 名、SE 名。
