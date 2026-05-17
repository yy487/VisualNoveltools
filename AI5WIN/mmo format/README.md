# MMO2PNG 工具说明

用于处理 `doukyousei.exe` / AI5WIN 系资源中的 `.MMO` 图像文件。

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
0x28  bytes    RGB LZSS 压缩流
```

你给的三个样本均为：

```text
width  = 0x1C8 = 456
height = 0x330 = 816
alpha_stream_offset = 0
```

## RGB 解码流程

1. 从 `0x28` 开始 LZSS 解码到 `width * height * 3` 字节。
2. 对 RGB 做差分还原：
   - 第一行：每个像素加左侧已还原像素。
   - 第二行开始：每个像素加上一行同列已还原像素。
3. 解出的 24bit 像素字节顺序实际为 `B,G,R`，导出时需要转换为标准 `RGB`。
4. 游戏内 surface 是 bottom-up，导出 PNG 默认做一次垂直翻转，方便正常观看。

## 用法

单文件：

```bat
python mmo2png.py OMI_KAO_A.MMO -o OMI_KAO_A.png
```

批量目录，v3 起默认递归扫描子目录：

```bat
python mmo2png.py path\to\mmo_dir -o png_out
```

只扫描当前目录这一层：

```bat
python mmo2png.py path\to\mmo_dir -o png_out --no-recursive
```

只查看 header：

```bat
python mmo2png.py OMI_KAO_A.MMO --list
```

保留游戏内 bottom-up 内存方向，不翻转：

```bat
python mmo2png.py OMI_KAO_A.MMO --no-flip
```

## 验证结果

已用以下样本测试通过：

- `OMI_KAO_A.MMO` -> `456x816`
- `OMI_CHIHA.MMO` -> `456x816`
- `OMI_YOSHI.MMO` -> `456x816`

当前样本没有 `alpha_stream_offset != 0` 的 MMO。工具里已经按 EXE 逻辑预留了第二 alpha 流的解码分支，但后续如果遇到带 alpha 的 MMO，建议再用实际样本核对 alpha 的方向和贴图矩形。
