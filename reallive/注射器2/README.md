# RealLive Seen.txt structural tool

用于本项目 `Seen.txt` 的结构化解码、反汇编、JSON 提取和非等长注入。工具不依赖裸 SJIS 扫描；流程为：SEEN 索引表 -> SEEN 块头 -> XOR crypt -> RealLive LZ 解压 -> VM message stream 解析。

## 当前结构

- `Seen.txt` 开头 `0x13880` 字节为 10000 项索引表，每项 `<u32 offset, u32 size>`。
- 每个非空 SEEN 块头 magic 为 `10002`。
- packed code 先用 256 字节 key 做循环 XOR，再进入 RealLive LZ 流。
- 解压后的 VM stream 中：
  - `0x0A + u16` 为 line marker；
  - `@ / ! + u16` 为文本显示控制；
  - `# cls grp op argc flag` 为命令；
  - `$` 为表达式；
  - `#00:02:0003` 后 `{...}` 内为选择支文本。

## 静态 crypt 模板

`crypt_template.py` 内置了本作 RealLive.exe 中 `byte_596984` 的 256 字节 XOR key，因此正常使用不再需要 `RealLive.exe_export_for_ai`。

如果换了另一个 RealLive build，可以：

1. 修改 `crypt_template.py` 里的 `STATIC_XOR_KEY`；或
2. 命令中传 `--ida-export RealLive.exe_export_for_ai`，从 `memory/*.txt` 读取 key。

## 提取

推荐命令：

```bat
python extract.py Seen.txt json
```

兼容旧命令：

```bat
python extract.py Seen.txt RealLive.exe_export_for_ai json
```

指定 SEEN：

```bat
python extract.py Seen.txt json --seen 106 601
```

## 注入

推荐命令：

```bat
python inject.py Seen.txt json Seen_chs.txt
```

兼容旧命令：

```bat
python inject.py Seen.txt RealLive.exe_export_for_ai json Seen_chs.txt
```

指定 SEEN：

```bat
python inject.py Seen.txt json Seen_chs.txt --seen 601
```

带中日字符映射：

```bat
python inject.py Seen.txt json Seen_chs.txt --map-json subs_cn_jp.json
```

## 反汇编

```bat
python disasm.py Seen.txt disasm_out
```

或：

```bat
python disasm.py Seen.txt disasm_out --seen 106
```

## JSON 格式

角色名内嵌在正文开头的格式：

```text
【葵】「んはぁ！」
```

会导出为：

```json
{
  "name": "葵",
  "scr_msg": "「んはぁ！」",
  "message": "「んはぁ！」",
  "_name_source": "bracket_prefix"
}
```

注入时会自动组合回：

```text
【葵】 + message
```

因此翻译时只改 `message`；如果确实要改角色名，可以改 `name`，注入也会写回。

## 变长注入

注入器会重建 decoded VM code、重新 LZ 压缩、重新 XOR，并重建外层 10000 项索引表。不是等长覆盖。

当前不主动改 line table；本样本 line table 表现为 source/debug line 映射，不是 bytecode offset 表。选择支和普通场景流由 VM command/expression stream 保持。
