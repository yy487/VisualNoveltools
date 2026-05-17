# MMO2PNG 工具说明

用于处理 `doukyousei.exe` / AI5WIN 系资源中的 `.MMO` 图像文件。

v4 增加了 C 加速核心：`mmo_fast.c`。Python 会优先尝试加载同目录下的 `mmo_fast.dll` / `mmo_fast.so`，找不到时自动回退纯 Python。

## 文件说明

- `mmo_common.py`：MMO header 解析、LZSS 解码、RGB 差分还原、C 加速加载。
- `mmo2png.py`：命令行转换工具，支持单文件、目录、递归目录输入。
- `mmo_fast.c`：C 加速核心，负责 RGB 主数据流的 LZSS 解码、差分还原、BGR->RGB 转换。
- `build_fast_msvc.bat`：Windows + MSVC 编译 `mmo_fast.dll`。
- `build_fast_mingw.bat`：Windows + MinGW-w64 编译 `mmo_fast.dll`。
- `build_fast_gcc.sh`：Linux 编译 `mmo_fast.so`，主要用于验证。

## 先编译 C 加速

### Windows / MSVC

打开 “x64 Native Tools Command Prompt for VS”，进入工具目录：

```bat
build_fast_msvc.bat
```

成功后同目录会生成：

```text
mmo_fast.dll
```

### Windows / MinGW-w64

确保 `gcc` 已加入 PATH：

```bat
build_fast_mingw.bat
```

### 不编译也能用

不编译 DLL 时工具会自动走纯 Python，只是速度慢。

查看当前是否启用 C 加速：

```bat
python mmo2png.py OMI_CHIHA.MMO --list --fast-info
```

如果看到类似下面说明已启用：

```text
mmo_fast: enabled: ...\mmo_fast.dll
```

## 用法

单文件：

```bat
python mmo2png.py OMI_KAO_A.MMO -o OMI_KAO_A.png
```

目录批量，默认递归扫描：

```bat
python mmo2png.py path\to\mmo_dir -o png_out
```

只扫描当前层，不递归：

```bat
python mmo2png.py path\to\mmo_dir -o png_out --no-recursive
```

禁用 C 加速，用纯 Python 对照：

```bat
python mmo2png.py path\to\mmo_dir -o png_out --no-fast
```

只查看 header：

```bat
python mmo2png.py OMI_KAO_A.MMO --list
```

保留游戏内 bottom-up 内存方向，不翻转：

```bat
python mmo2png.py OMI_KAO_A.MMO --no-flip
```

## 反汇编对应关系

在 `doukyousei.exe.c` 中能看到 MMO 被作为界面/回想图资源读取，例如：

- `FUN_00422170("xxx.mmo")`：打开 MMO 资源并建立 24bit surface。
- `FUN_00421f10`：根据 MMO header 中的矩形区域解码图像。
- `FUN_0042c480`：LZSS 解码函数，使用 `0x1000` 字节环形字典，初始写入位置 `0xFEE`。
- `FUN_0042c600`：RGB 差分还原函数。

## MMO header

样本 header 长度为 `0x28`：

```text
0x00  char[4]  magic = "MMO "
0x04  u32le    image_left
0x08  u32le    image_top
0x0C  u32le    image_right
0x10  u32le    image_bottom
0x14  u32le    alpha_left / second_rect_left
0x18  u32le    alpha_top
0x1C  u32le    alpha_right
0x20  u32le    alpha_bottom
0x24  u32le    alpha_stream_offset，0 表示无第二数据流
0x28  bytes    RGB/BGR LZSS 压缩流
```

你给的三个样本均为：

```text
width  = 0x1C8 = 456
height = 0x330 = 816
alpha_stream_offset = 0
```

## RGB 解码流程

1. 从 `0x28` 开始 LZSS 解码到 `width * height * 3` 字节。
2. 对像素做差分还原：
   - 第一行：每个像素加左侧已还原像素。
   - 第二行开始：每个像素加上一行同列已还原像素。
3. 解出的 24bit 像素字节顺序实际为 `B,G,R`，导出时转换为标准 `RGB`。
4. 游戏内 surface 是 bottom-up，导出 PNG 默认做一次垂直翻转，方便正常观看。

## 当前加速范围

C 加速当前覆盖 RGB 主数据流，也就是最耗时部分：

- LZSS 解码
- 差分还原
- BGR -> RGB 通道转换

`alpha_stream_offset != 0` 的第二数据流仍由 Python 分支处理。你给的三个样本都没有 alpha 数据流。

## 验证结果

已用以下样本测试通过：

- `OMI_KAO_A.MMO` -> `456x816`
- `OMI_CHIHA.MMO` -> `456x816`
- `OMI_YOSHI.MMO` -> `456x816`

本地 Linux `.so` 测试中，三张样本纯 Python 约 `0.57s/张`，C 加速约 `0.007s/张`，主要耗时会转移到 PNG 保存。
