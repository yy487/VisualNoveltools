# zgf2png 工具

用于将 MoonLight Renewal 系列的 `.zgf` 图像批量转换为 PNG。

## 格式结构

样本 `bg001.zgf` 的结构：

```text
0x00  char[4]  "ZGF\x1A"
0x04  u32le    max_compressed_block_size
0x08  u8       flags / type，样本为 1
0x09  u8       bpp，支持 24 / 32
0x0A  u16le    width
0x0C  u16le    height
0x0E  block[]  每个颜色平面一个压缩块

block:
  u32le size              # 包含后面的 4 字节校验 + zlib 流
  u32le checksum_or_hash  # 引擎解码时跳过
  bytes zlib_stream       # inflate 后长度 = width * height
```

24bpp 文件有 3 个平面：B/G/R。32bpp 文件额外有 1 个 alpha 平面。

RGB 重建逻辑来自反汇编中的 `sub_41A852` + `sub_41A416`：

```c
pixel = B | (G << 8) | (R << 16);
pixel ^= prev;
prev = pixel;
```

初始 `prev = 0x007F7F7F`。

## C 加速

`zgf_fast.c` 负责颜色平面合成和累计 XOR 反滤波；zlib 解压与 PNG 压缩使用 Python 标准库中的原生 zlib。

首次运行时，`zgf2png.py` 会尝试自动编译：

```bash
# Windows / MinGW
python zgf2png.py bg001.zgf out

# 手动编译也可以
gcc -O3 -shared -o zgf_fast.dll zgf_fast.c
```

Linux/macOS 会生成 `libzgf_fast.so` 或 `libzgf_fast.dylib`。

如果没有 C 编译器，工具会自动退回纯 Python 合成，但速度会慢一些。

## 命令

单文件：

```bash
python zgf2png.py bg001.zgf out_png
```

批量目录递归转换：

```bash
python zgf2png.py input_zgf_dir output_png_dir -j 8 --force
```

只处理当前目录，不递归：

```bash
python zgf2png.py input_zgf_dir output_png_dir --no-recursive
```

不保留子目录结构，全部平铺输出：

```bash
python zgf2png.py input_zgf_dir output_png_dir --flat
```

强制输出 RGBA：

```bash
python zgf2png.py input_zgf_dir output_png_dir --rgba
```

## 说明

- 不依赖 Pillow。
- PNG 写入使用标准库实现，输出为无损 PNG。
- 默认保留输入目录结构。
- 默认不会覆盖已有 PNG；使用 `--force` 覆盖。
