# BGI V1 工作流版文本提取/注入工具

这个目录是在现有 BGI V1 反汇编/汇编工具基础上整理出的工作流版。重点改动是：输出统一 JSON 字段 `name` / `scr_msg` / `message`，并补充 V1 选项文本识别。

## 文件说明

```text
asdis.py              汇编/反汇编通用文本转义函数
bgiop.py             BGI V1 opcode 表
bgidis.py            BGI V1 编译脚本 -> .bsd 反汇编
bgias.py             .bsd -> BGI V1 编译脚本汇编
bgi_dialog_json.py   对话/选项/UI 识别逻辑，已补充 slct::f_160~f_17f、f_01c("SelectEx")、AutoSaveRange、sys_::f_11e
common.py            工作流公共逻辑
audit_bsd_strings.py BSD 字符串覆盖率审计：扫描反汇编字符串并与 JSON diff
extract.py           提取入口
inject.py            注入入口
pipeline.py          简单调度入口
bss_mapping.json     当前函数名映射
```

## JSON 格式

提取结果示例：

```json
{
  "name": "キャラ名",
  "scr_msg": "原始文本",
  "message": "原始文本",
  "_file": "script001",
  "_index": 12,
  "_type": "dialogue",
  "_line": 345
}
```

说明：

- `scr_msg` 是原始文本，只用于定位和校验，不要改。
- `message` 是实际写回文本，翻译只改这个字段。
- 没有角色名时不输出 `name`。
- 选项文本 `_type` 为 `choice`。
- 章节标题、选择提示、窗口标题等非对白 UI 字符串 `_type` 为 `ui`。
- `_file` 和 `_index` 用于稳定定位。

## 提取 .bsd

如果你已经有反汇编后的 `.bsd`：

```bat
python extract.py input_bsd_dir json_out --mode bsd
```

单文件：

```bat
python extract.py "script001.bsd" "script001.json" --mode bsd
```

## 提取编译脚本

如果输入是 BGI V1 原始编译脚本，通常是无扩展名文件：

```bat
python extract.py script_dir json_out --mode script --encoding shift_jis --fallback-encoding gbk
```

单文件：

```bat
python extract.py "script001" "script001.json" --mode script --encoding shift_jis --fallback-encoding gbk
```

`--mode auto` 会把 `.bsd` 当反汇编文本处理，把无扩展名文件当编译脚本处理。


## 覆盖率审计：BSD 字符串 diff JSON

本工具包新增 `audit_bsd_strings.py`，用于你说的这条排查流程：

```text
编译脚本 -> bgidis.py 反汇编为 .bsd
扫描 .bsd 里所有候选 CP932/日文字符串
与 extract.py 输出的 JSON 中 scr_msg/message/name 做归一化 diff
输出漏提候选及上下文
```

直接对编译脚本目录审计：

```bat
python audit_bsd_strings.py script_dir json_out audit_report.json --mode script --encoding shift_jis --fallback-encoding gbk --context-lines 8
```

如果已经有 `.bsd` 目录：

```bat
python audit_bsd_strings.py bsd_dir json_out audit_report.json --mode bsd --context-lines 8
```

审计时会做这些归一化，避免大量误报：

- 对话控制后缀 `<>&.` 会从比较文本中去掉；
- `PrintMessage XXX` 会按角色名 `XXX` 比较；
- 文件路径、资源名、纯 ASCII 函数名/标签名会过滤；
- 已知内部错误文本 `指定されたラベルは見つかりませんでした。` 默认过滤。

如需把内部错误文本也列出来，加：

```bat
--include-internal
```

## 注入 .bsd

```bat
python inject.py input_bsd_dir json_translated out_bsd_dir --mode bsd
```

单文件：

```bat
python inject.py "script001.bsd" "script001.json" "out\script001.bsd" --mode bsd
```

## 注入编译脚本

```bat
python inject.py script_dir json_translated out_script_dir --mode script --encoding shift_jis --fallback-encoding gbk
```

单文件：

```bat
python inject.py "script001" "script001.json" "out\script001" --mode script --encoding shift_jis --fallback-encoding gbk
```

## 选项提取逻辑

原有逻辑只覆盖：

```text
push_string("选项");
move(2);
```

本版额外补充两种 V1 形态：

```text
push_string("左の方のカードを引く");
...
slct::f_16x();
```

以及本样本实际出现的 `SelectEx` 封装形态：

```text
f_0fe();
push_string("左の方のカードを引く");
push_string("右の方のカードを引く");
push_string("真ん中のカードを引く");
push_dword(...);
...
push_string("SelectEx");
f_01c();
```

处理方式是：遇到 `f_01c()` 且直前函数名为 `SelectEx` / `_SelectEx` 时，回溯到上一条 `f_0fe()` 之后，只提取该窗口里的可见字符串作为 `_type: "choice"`。如果窗口内已经是 `push_string(...); move(2);` 旧形态，则交给旧逻辑处理，避免把提示语误提为选项。

同时兼容 V0 映射里的：

```text
f_SetNextSelectingJumping / f_0a9
f_Select / f_0b0
```

本样本里 `root_common` 还有几类不是对白、也不是按钮选项，但属于可见 UI/章节标题的字符串，现在也纳入 `_type: "ui"`：

```text
push_string("見たいシーンを選択してください");
push_string("_SelectEx");
f_01c();

push_string("プロローグ");
push_string("AutoSaveRange");
f_01c();

push_string("おまけ");
nargs(1);
sys_::f_11e();
```

## 注入定位策略

注入时优先：

```text
_file + _index + scr_msg 校验
```

如果 `_index` 对不上，会退回到同一文件内唯一 `scr_msg` 匹配。若同一句重复出现，则不会盲目注入，会输出 warning。

## 注意事项

1. 目前这是针对 BGI V1 / `.bsd` 工作流的版本，不是封包工具。
2. 编译脚本注入会先反汇编成临时 `.bsd`，修改后再汇编回编译脚本。
3. 如果翻译文本无法用主编码或 fallback 编码写回，`bgias.py` 会按现有编码回退逻辑处理；正式补丁仍建议配合字符映射/字体重绘方案。
4. 已用 `mew.zip` 当前 10 文件样本验证：`prologue` 中 `左の方のカードを引く` / `右の方のカードを引く` / `真ん中のカードを引く` 可正常提取为 `_type: "choice"`。
5. 当前样本审计结果：扫描 10 个脚本、候选可见字符串 8075 条，与 JSON diff 后漏提候选为 0 条。
6. 已做注入冒烟测试：`root_common` 中 `見たいシーンを選択してください`、`プロローグ`、`おまけ` 可通过 JSON 注入回对应 `push_string`。
7. 如果后续完整包还有漏选项/漏 UI，先运行 `audit_bsd_strings.py`，再看 `audit_report.json` 里的 context 补具体调用模式。
