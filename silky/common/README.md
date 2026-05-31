# リルカは幾重に夜を彩る / Silky MES 文本工作流

本包已经改成项目常用结构：

```text
silky_op.py        MES <-> op.txt 反汇编/汇编
silky_common.py    extract/inject 共用的块识别、ruby、JSON、注入逻辑
silky_extract.py   op.txt -> JSON
silky_inject.py    JSON + op.txt -> 新 op.txt
silky_pipeline.py  批量一键 unpack / pack
```

## 1. 批量提取

```bat
python silky_pipeline.py unpack mes work -j 1
```

输出：

```text
work\op\*.op.txt      反汇编结果，通常不要给译者改
work\json\*.json      翻译 JSON，只改 message 字段
```

JSON 条目格式：

```json
{
  "_file": "xxx.MES",
  "_index": 0,
  "name": "角色名",
  "scr_msg": "原始脚本文本",
  "message": "翻译文本"
}
```

规则：

- `scr_msg` 是原始脚本文本，不要修改；
- `message` 初始等于 `scr_msg`，翻译时改 `message`；如果要改角色名，可以直接改同条目的 `name`；
- `name` 没有时不输出；有 `name` 时，注入会直接把 JSON 里的 `name` 写回对应 name 块；
- `_file` / `_index` 用于批处理定位和排错。

## 2. 批量注入并重组 MES

```bat
python silky_pipeline.py pack work out_mes -j 1
```

输出：

```text
work\op_injected\*.op.txt   注入后的中间 op
out_mes\*.MES               重组后的脚本
```

注入策略：

1. 优先用 `_index` 定位同文件内条目；
2. 注入前用 `scr_msg` 校验原文；
3. 默认校验失败也会按 `_index` 继续注入，并打印 warning；
4. 如果希望校验失败就跳过，使用：

```bat
python silky_pipeline.py pack work out_mes --strict -j 1
```

## 3. 单文件命令

反汇编：

```bat
python silky_op.py disasm input.MES input.op.txt --encoding cp932
```

提取 JSON：

```bat
python silky_extract.py input.op.txt input.json
```

注入 JSON：

```bat
python silky_inject.py input.op.txt input.json input.injected.op.txt
```

汇编：

```bat
python silky_op.py asm input.injected.op.txt output.MES --encoding cp932
```

## 4. 注意事项

- 本工具当前仍沿用原脚本的 Silky opcode 表与 STR_CRYPT/STR_UNCRYPT 处理。
- ruby/注音结构不会直接暴露给译者；注入时 reading 槽位会写为全角空格占位，base 文本写入 `message`。
- 如果某个文件注入后游戏报错，优先检查对应 JSON 是否误改了 `scr_msg`、是否删除了字面 `\n` 分段。
- 如果需要做封包层处理，建议外层另写 archive unpack/pack，本包只负责 MES 文本层。
