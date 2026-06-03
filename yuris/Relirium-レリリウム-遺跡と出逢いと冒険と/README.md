# YDG2PNG 批量转换工具

用于把 YU-RIS 的 `.ydg` 图片批量转换成 `.png`。

当前已验证样本：`YDG\0 + YU-RIS` 容器，内部为多个 `RIFF WEBP` 纵向分片。样本 `a_tow_01_a.ydg` 解析结果为 `1920x1080`，4 个 `1920x270` WebP 分片，纵向拼接后输出 PNG。

## 依赖

```bat
python -m pip install pillow
```

这里的 C 加速来自 Pillow 的 `libwebp/libpng/zlib` 后端；脚本本身不做 Python 逐像素循环，只解析 YDG 容器、切分 WEBP 块，然后调用 Pillow 的 C 后端解码和写 PNG。

## 用法

查看单个文件结构：

```bat
python ydg2png_batch.py info a_tow_01_a.ydg
```

转换单个文件：

```bat
python ydg2png_batch.py convert a_tow_01_a.ydg a_tow_01_a.png --overwrite
```

批量转换目录下所有 `.ydg`，递归处理子目录，并保持目录结构：

```bat
python ydg2png_batch.py convert input_ydg_dir output_png_dir -j 8 --overwrite
```

只处理输入目录第一层，不递归：

```bat
python ydg2png_batch.py convert input_ydg_dir output_png_dir --no-recursive
```

输出文件名加后缀：

```bat
python ydg2png_batch.py convert input_ydg_dir output_png_dir --suffix _converted
```

同时导出内嵌 WebP 分片，方便调试：

```bat
python ydg2png_batch.py convert input_ydg_dir output_png_dir --raw-webp
```

## 输出示例

```text
[ok] input/a_tow_01_a.ydg -> output/a_tow_01_a.png (1920x1080, chunks=4)
[ydg2png] files=1 ok=1 failed=0 output=output
```

## 当前格式判断

样本头部：

```text
00: 59 44 47 00              magic = YDG\0
04: 59 55 2D 52 49 53 00 00  signature = YU-RIS\0\0
0C: 64 00 00 00              version/type = 100
10: 30 00 00 00              chunk table offset = 0x30
14: 2A EB 19 00              file size = 0x19EB2A
20: 80 07 38 04              width=1920, height=1080
30: 04 00 00 00              chunk count = 4
```

分片表每项 16 字节：

```text
u32 chunk_offset
u32 chunk_size
u16 kind
u16 strip_height
u16 flags0
u16 flags1
```

当前样本中 `kind=2`，`strip_height=270`，每个分片都是标准 `RIFF WEBP`。

## 已知限制

- 当前只支持内嵌 `RIFF WEBP` 的 YDG。
- 暂未支持非 WEBP 的旧式 YDG 变体。
- 暂未做 PNG2YDG 回写。
