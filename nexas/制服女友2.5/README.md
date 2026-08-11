# Uniform Kanojo NeXAS JSON Tool

这是针对已解包 `mes` 目录中 NeXAS 脚本的 UTF-8 JSON 提取、注入和结构重建工具。工具不覆盖输入文件；目录注入会先完整复制源目录，再替换有对应 JSON 的 `.bin`。

## 适用格式

- 文件扩展名：`.bin`。
- 已确认脚本 magic：`VER-1.00\0`。
- 已确认脚本字符串编码：UTF-8。字符串表中的未知数据、未知指令区域和 `__global.bin` 均按原始字节保留。
- 当前工具处理已解包的 `mes` 目录。样本的外层 `script.pac`/压缩包不由本工具重新封包。

## 命令

```powershell
.\target\release\nexas_extract.exe "<MES_DIR>" --output "<WORK_DIR>\mes_json"
.\target\release\nexas_inject.exe "<MES_DIR>" "<WORK_DIR>\mes_json" --output "<WORK_DIR>\mes_injected"
.\target\release\nexas_rebuild.exe "<MES_DIR>" --output "<WORK_DIR>\mes_rebuilt"
```

三个 EXE 都支持 `-h/--help`。Windows 拖放等价于把文件或目录路径作为位置参数传入。省略 `--output` 时使用输入旁的新目录或新文件名；已有输出会直接报错，不会自动删除。

`nexas_extract` 按源文件输出独立 JSON，例如 `1001_本編.bin` 对应 `1001_本編.json`。`nexas_inject` 接收同样的 JSON 目录结构，并保留没有文本 JSON 的文件。`nexas_rebuild` 用于验证无修改解析/重建的字节回环。

## 提取规则

- 相邻字符串表项形如“角色名，正文”时，前一项提取为 `name`，后一项提取为 `message`；角色名字符串不会另外生成重复条目。
- 无名对白和旁白只生成 `message`。
- 连续两个或更多 `『...』` 字符串识别为选项，生成 `_type: "choice"`、`_choice_group` 和 `_choice_index`。
- 系统脚本/系统文本、资源名、纯控制字符串不生成翻译条目。
- 正文外的控制符拆到 `_control_prefix` / `_control_suffix`，字段为空时在 JSON 中省略；注入时自动恢复。正文内部控制符保留在 `scr_msg` 和 `message` 中，允许翻译时增删或调整 `@n` 等内部控制符。

当前样本的控制符包括 `@v...`、`@h...`、`@t...`、短控制符和 `@*@` 宏形式。控制符的视觉语义未作运行时命名，但字节边界和回环已验证。`message` 内部控制符可编辑；正文外控制字段仍用于校验源脚本边界，不应手工修改。

## JSON 合同

翻译者应修改 `message`，不要修改 `scr_msg`。角色名可修改时修改 `name`；`_scr_name` 是原始角色名校验值，注入前必须仍与源脚本一致。

```json
{
  "name": "女子生徒Ａ",
  "scr_msg": "「あっ、古賀先生さようならー！」",
  "message": "「啊，古贺老师再见！」",
  "_scr_name": "女子生徒Ａ",
  "_scr_raw": "@v09900000「あっ、古賀先生さようならー！」",
  "_control_prefix": "@v09900000",
  "_file": "1001_本編導入.bin",
  "_index": 0,
  "_offset": 49423,
  "_size": 58,
  "_body_size": 48,
  "_scope": "global",
  "_string_index": 3,
  "_type": "dialogue",
  "_opcode": "constant_string",
  "_rule": "direct-name",
  "_encoding": "utf-8",
  "_policy": "relocate"
}
```

示例中的 `_scr_raw` 仅用于说明字段形状；实际 JSON 中它始终保存未翻译源字符串。`_file + _index + scr_msg`、字符串表位置和 `_scr_raw` 会共同用于注入定位和校验。变长正文由完整脚本表重建，不做固定槽位截断。

## 验证结果

对原始 `mes` 样本目录已完成：

- 37 个 `.bin` 中 36 个 `VER-1.00\0` 脚本结构化解析并字节级重建一致；`__global.bin` 原样保留。
- 提取 24 个 JSON、5051 条文本：3294 条对白、1749 条旁白、8 条选项，62 条带姓名配对。
- 无修改注入：5051 条全部 `unchanged`；源目录 38 个文件与输出目录逐文件 SHA-256 一致。
- 修改回环：正文、正文内 `@n` 的保留与删除、前置 `@v...` 和角色名均成功写回；错误 `_scr_name` 以非零退出码拒绝。

## 已知限制

- 只处理已解包的 `mes` 脚本，不生成外层 PAC/压缩包。
- 系统文件过滤名单和资源名前缀按当前样本建立；遇到新变体应先复核项目档案。
- 无效 UTF-8 字符串不会猜测翻译，会保留并报告 warning。
- 未确认的控制符语义保持 opaque；工具只保证结构边界、控制符校验和字节回环。
