# TVLost CPB2PNG Adler Hotfix

这个补丁修正 `inflated adler mismatch`：TYP1 block 前 4 字节校验的是压缩 zlib payload 本身，不是解压后的像素通道。

修正后，`bg015a.cpb`、`ev0007c.cpb`、`trueend.cpb` 这类 24bpp 图会正常输出。

仍然暂不支持：

- `bpp=32` 立绘/半透明图
- `bpp=8` map/调色板图

命令：

```bat
python cpb2png.py graphic.arc png_out --all
```
