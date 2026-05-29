# TVLost `graphic.arc` / `TYP1` CPB 转 PNG 工具

## 当前结论

`graphic.arc` 使用两层不同 key：

```text
ARC header/table key      = 0xADD1F4AA
图像 entry fread stream key = 0x4AF3D7A3
```

前一个 key 只用于解析 ARC 头和文件表；图像数据进入 `TYP1` 解码器时，实际使用后一个 key 按每次 `fread` 的真实 `ftell()` 位置重新初始化 XOR 状态。

`TYP1` 24bpp 结构：

```text
00  54 59 50 31        "TYP1"
04  bpp                24 / 32 / 8
05  flag
06  width              u16le
08  height             u16le
0A  size0              u32le
0E  size1              u32le
12  size2              u32le
16  size3              u32le
1A  reserved           u32le
1E  compressed blocks
```

24bpp 路径实际按 `size3 -> size2 -> size1` 顺序读取三个 zlib block，解压后是 `B/G/R` 三个通道，每个通道大小为 `width * height`。输出 PNG 时转换为 RGB。

## 关于 BLACK / RED / WHITE

`black.cpb`、`white.cpb`、`red.cpb` 不是解码 `ev*.cpb` 所必须的色通道外置资源。它们本身就是普通 `TYP1 24bpp` 图像，可以独立解成 `800x600` PNG。

验证结果：

```text
black.cpb  -> 800x600 纯黑
white.cpb  -> 800x600 纯白
red.cpb    -> 800x600 固定色，约 RGB(172, 0, 53)
```

因此 `ev*.cpb` / `bg*.cpb` 不需要和这些纯色 CPB 叠加才能还原；它们的像素数据是自包含的。

## 使用

安装 Pillow：

```bat
pip install pillow
```

可选：编译 C 加速。Windows / MinGW-w64：

```bat
build_c_accel.bat
```

Linux / MSYS2：

```bash
./build_c_accel.sh
```

转换指定文件：

```bat
python cpb2png.py graphic.arc png_out --names bg002b.cpb bg005b.cpb black.cpb red.cpb white.cpb
```

尝试转换全部 CPB：

```bat
python cpb2png.py graphic.arc png_out --all
```

当前版本完整支持 `TYP1 24bpp`。`32bpp` 立绘和 `8bpp` map 类图片会在 `--all` 下跳过，后续可以继续按 `FUN_0041eac0` / `FUN_0041e640` 补。

## 已验证样本

```text
bg002b.cpb  800x600  24bpp  OK
bg005b.cpb  800x600  24bpp  OK
black.cpb   800x600  24bpp  OK
white.cpb   800x600  24bpp  OK
red.cpb     800x600  24bpp  OK
```

## 注意

不要用之前按 `0xADD1F4AA` 解包出来的单独 `.cpb` 作为转换输入。那只是 ARC 表层 key 下的错误数据视图。正确转换需要从 `graphic.arc` 的物理 entry offset 读取原始字节，再用 `0x4AF3D7A3` 和真实文件位置进行 fread 层解密。
