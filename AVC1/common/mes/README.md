# 少女たちは荒野を目指す MES 文本工具

本工具处理从本作 ACV legacy DAT 中解出的 CP932 明文 `.txt` 脚本。它只负责文本提取和注入，不负责 ACV 解包或封包。

## 文件

- `mes_extract.exe`：将脚本提取为逐文件 UTF-8 JSON。
- `mes_inject.exe`：校验 JSON 后生成新的 CP932 脚本或完整脚本目录。
- `PROJECT_PROFILE.md`：格式证据、姓名规则、选项结构和限制。

两个 EXE 都支持 `-h` / `--help`。Windows 拖放等价于传入位置参数；路径可包含空格、`&`、中文和日文。

## 提取

```powershell
mes_extract.exe "<MES_DIR>"
mes_extract.exe "<MES_DIR>" --output "<OUTPUT_DIR>\mes_json"
mes_extract.exe "<MES_DIR>\008_A_140414_0.txt"
```

默认输出：

- `script.txt` -> `script.txt.json`
- `mes\` -> `mes_json\`

目录模式递归扫描 `.txt`。只有包含文本条目的脚本生成 JSON；不同源文件不会混入同一个 JSON。输出已存在时会停止，不覆盖也不清理旧目录。

## 注入

```powershell
mes_inject.exe "<MES_DIR>" "<JSON_DIR>"
mes_inject.exe "<MES_DIR>" "<JSON_DIR>" --output "<OUTPUT_DIR>\mes_injected"
mes_inject.exe "008_A_140414_0.txt" "008_A_140414_0.txt.json"
```

默认输出：

- `script.txt` -> `script_injected.txt`
- `mes\` -> `mes_injected\`

目录注入先校验全部 JSON 和源脚本，再创建输出；随后复制完整源树并只替换有 JSON 的 `.txt`。`manifest.json`、无文本脚本和其他文件会原样保留。源目录和已有输出不会被覆盖。

拖放注入时要同时选中两个路径，并保证参数顺序为“源 TXT/目录、JSON/目录”。如果 Windows 调整了多选拖放顺序，请改用命令行。

## JSON 合同

JSON 顶层是数组。可编辑字段只有：

```json
{
  "name": "文太郎",
  "_scr_name": "文太郎",
  "scr_msg": "「おつかれーす」",
  "message": "「おつかれーす」",
  "_file": "008_A_140414_0.txt",
  "_index": 1,
  "_line": 24,
  "_offset": 123,
  "_size": 20,
  "_type": "dialogue",
  "_rule": "21",
  "_encoding": "cp932"
}
```

- `message`：正文或选项译文，注入时实际写回。
- `name`：姓名译文，允许修改并写回。
- `scr_msg`：不可修改的原始正文，仅用于定位和校验。
- `_scr_name`：不可修改的原始姓名；修改 `name` 前必须通过此字段校验。
- 其他下划线字段均为不可修改的来源、位置和结构元数据。

选项条目另有 `_opcode`、`_target` 和 `_choice_index`。只能修改 `message`，跳转标签不会写自 JSON，而是始终保留源脚本中的值。

## 姓名与正文

规则按项目确认的正则优先级实现：

- `【文太郎,S000_A_0001】「本文」` -> `name=文太郎`
- `【砂雪@？？？,S001_A_0001】「本文」` -> `name=？？？`
- `【砂雪@,S001_C_0001】「本文」` -> `name=砂雪`
- `【うぐいす】「本文」` -> `name=うぐいす`
- 无 `【...】` 头的非命令行作为 `monologue`。

姓名翻译不能包含结构分隔符 `【`、`】`、`@` 或半角逗号。注入只替换被对应规则捕获的姓名片段，不改变语音 ID、原始人物标识或其他头部字段。

## 选项

工具解析下列结构中的引号字符串，并按最后一个 `,*标签` 分离显示文本和目标：

```text
SELECT "显示文本,*label", "显示文本,*label2"
SELECT_INIT "显示文本,*label", ...
if(...) cal ... $$0="动态显示文本,*label" ...
```

已确认真实样本共有 66 个选项槽：54 个 `SELECT`、6 个 `SELECT_INIT`、6 个动态 `$g_select` 赋值。选项译文不能包含半角双引号或 `,*`。

## 编码、控制符与变长

- 源脚本和注入结果：CP932。
- JSON：无 BOM UTF-8、LF、稳定缩进。
- 脚本物理换行：LF；JSON 译文禁止真实 CR/LF。
- 显示换行：字面控制符 `[n]`。译文可以删除一个或多个控制符（包括全部删除）以测试自动换行；保留的控制符必须是源序列的有序子序列，不得新增或乱序。
- 禁止 NUL；所有无法编码为 CP932 的字符会按 Unicode 码位报告并拒绝输出。
- 明文脚本通过整文件重建支持正文、姓名和选项的变长修改。

外层 ACV 归档不属于本工具能力。把变长脚本重新封包时，封包器仍须正确重建压缩大小、offset 和文件表。

## 校验与失败策略

注入逐条校验 `_file + _index + scr_msg`，同时校验行号、字节偏移、原始长度、规则、类型、编码、姓名原文和选项目标。任何条目失败都会在创建输出前停止。错误包含文件、条目索引和行号。

程序结束时输出 `scanned_files`、`json_files`、`extracted_entries`、`json_entries`、`patched`、`unchanged`、`failed`、`warnings` 和输出路径。

## 已知限制

- 仅支持当前档案确认的 CP932、LF 明文 `.txt`。
- 不处理 ACV 解包/封包。
- 真实样本中没有命中原配置的 22、24、25、30、31、32 号规则；工具按用户提供的语法实现并用合成测试覆盖，但其运行时显示语义未确认。
- 不解释 `[n]` 以外的未知控制符；若以后出现其他方括号控制符，注入允许删减但会校验保留顺序，不允许新增或乱序。
