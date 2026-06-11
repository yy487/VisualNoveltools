# MLR ARC 解包/回包工具

## 已确认格式

样本 `sound.arc` 的结构：

```text
0x00  char[4]  magic = "ARC\x1A"
0x04  u32le    version = 1
0x08  u32le    file_count
0x0C  char[32] default_ext，例如 ".ogg"
0x2C  entry[file_count]

entry:
  0x00 u32le    data_offset
  0x04 u32le    data_size
  0x08 char[24] filename，CP932/ASCII C 字符串

data:
  raw bytes，无压缩、无加密；sound.arc 内是 OggS/Vorbis 数据
```

反编译侧对应关系：游戏读取 44 字节头，检查 magic `0x1A435241`，保存 version、file_count、default_ext，然后一次性读取 `32 * file_count` 字节文件表。

## 命令

列出文件表：

```bash
python mlr_arc_tool.py list sound.arc
```

解包：

```bash
python mlr_arc_tool.py unpack sound.arc sound_unpacked
```

解包会生成 `_arc_manifest.json`，其中保存原始顺序、offset、size、sha256 和头部信息。

回包：

```bash
python mlr_arc_tool.py pack sound_unpacked sound_new.arc --overwrite
```

默认会读取 `sound_unpacked/_arc_manifest.json`，因此能保持原始文件顺序和头部 `default_ext`。

验证目录中文件是否与封包内条目一致：

```bash
python mlr_arc_tool.py verify sound.arc sound_unpacked
```

## 限制

- 文件名字段只有 24 字节。为保证游戏侧 C 字符串安全，工具限制为最多 23 字节并保留末尾 `NUL`。
- 当前样本没有压缩/加密。工具按 raw data 直接切片写出。
- 回包会重建为连续数据区，offset/size 会重新计算。
- ARC 内部路径虽然做了安全处理，但该引擎样本看起来使用平铺文件名，不建议写入子目录路径。
