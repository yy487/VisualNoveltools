# TVLost ARC unpacker

## 当前结论

`script.arc` 是加密 ARC 包：

- header 加密后 magic 为 `ARC\0`
- header 大小：`0x30`
- entry 大小：`0x30`
- 文件数：`151`
- 默认脚本扩展名：`.asb`
- 文件表压缩：`zlib`，前 4 字节为 `adler32(compressed_payload)`
- 文件表压缩大小：`2526`
- 文件表解压大小：`7248 = 151 * 0x30`
- 数据区起始：`0xA0E`
- 当前样本 key：`0xADD1F4AA`

## 使用

```bat
python tvlost_arc_unpack.py script.arc script_unpacked
```

指定 key：

```bat
python tvlost_arc_unpack.py script.arc script_unpacked --key 0xADD1F4AA
```

输出目录中会生成：

- 解密后的 `.asb` 文件
- `_manifest.json`，记录文件名、offset、size、hash 等信息

## 结构

header 解密后：

```text
00  4 bytes  magic = ARC\0
04  4 bytes  version/flags，目前样本为 1
08  4 bytes  file count
0C  4 bytes  compressed file table size
10  32 bytes default extension string，例如 .asb
```

entry 解压后每项 `0x30`：

```text
00  4 bytes  data relative offset, 需加 data_base
04  4 bytes  file size
08  4 bytes  filename CRC/hash，用于快速查找
0C  4 bytes  unknown/reserved，目前样本为 0
10  32 bytes null-terminated filename
```

## 下一步

下一步可以分析 `.asb` 脚本结构，或转向图像包/图像格式 `.cpb`，实现 `cpb2png`，并把像素重排/解压部分改为 C 加速。
