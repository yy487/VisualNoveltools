# Seven Bridges 资源与 GSC 本地化工具

本目录是独立工作区。工具只读取用户提供的游戏目录，所有生成物都写到显式指定的输出目录；不会修改本仓库中的其他参考工程，也不会修改 `scr_unpacked` 或 `grps_unpacked`。

## 构建

```powershell
cargo build --release --offline
```

生成两个程序：

- `target/release/wcg-png.exe`：Liar-soft WCG → 透明 RGBA PNG。
- `target/release/sbridge-gsc.exe`：GSC → UTF-8 JSON，以及只回写 `message` 的 GSC 重建。

两个程序都支持无参数会话菜单和单路径拖放预填。只有参数完整并带 `--yes` 时才进行一次性非交互写入；目标已存在时默认拒绝覆盖。

## WCG → PNG

单文件：

```powershell
wcg-png.exe convert "输入.wcg" --output "输出.png" --yes
```

整个目录（按内容识别 WCG，不依赖扩展名）：

```powershell
wcg-png.exe convert "grps_unpacked" --output "grps_png" --yes
```

可选参数：`--recursive` 递归子目录，`--overwrite` 明确允许替换现有 PNG。PNG 使用有效但不追求体积的 Deflate stored blocks，以保持工具离线、零运行时依赖。

本游戏 120 个 WCG 已全部转换并由 Pillow 独立校验。66 个 `gfNNN` 姓名牌的识图结果保存在 [speaker_map.json](data/speaker_map.json)，黑底联系表位于 `artifacts/nameplates_1.png` 和 `artifacts/nameplates_2.png`。

## 提取 GSC

```powershell
sbridge-gsc.exe extract "scr_unpacked" `
  --output "translation_json" `
  --speaker-map "data\speaker_map.json" `
  --yes
```

输出是一棵 UTF-8 JSON 目录，附带 `.sbridge-gsc-manifest.json`。manifest 明确记录源路径角色、每个 GSC 的声明/物理长度和尾部大小；移动 JSON 工作区后仍可在注入时用 `--source` 指定新的只读源根目录。

典型条目：

```json
{
  "name": "乗務員",
  "scr_msg": "お客さま、どうなさいました？お部屋がどちらか、お忘れでしょうか？",
  "message": "お客さま、どうなさいました？お部屋がどちらか、お忘れでしょうか？",
  "_file": "2002.gsc",
  "_index": 18,
  "_type": "dialogue",
  "_speaker_id": 118
}
```

字段政策：

- `scr_msg` 是不可修改的源文本校验值。
- `message` 是唯一可写字段。
- `name` 由 `gfNNN` 图像映射提供，只是译者上下文。注入器完全忽略它；修改、删除或添加 `name` 都不会进入 GSC。
- `_speaker_id`、定位、长度、类型与指令信息是不可修改的源校验元数据。
- 结构性 `^gNNN` 始终从原 GSC 保留，绝不由 `name` 反推。

游戏运行时具有按字宽自动换行和日文禁则处理。因此提取时会从 `scr_msg` 与 `message` 删除全部原始 `^n`，包括歌词/诗行。未编辑的正文在注入时直接复用原始字节，以保证零修改回环；正文一旦编辑，旧 `^n` 不会恢复。译者新加入的实际换行或合法 `^n` 会编码为强制换行。

## 注入 GSC

```powershell
sbridge-gsc.exe inject "translation_json" `
  --output "scr_rebuilt" `
  --yes
```

如果 manifest 中的源路径失效，可显式改用：

```powershell
sbridge-gsc.exe inject "translation_json" `
  --source "新的只读源根目录" `
  --output "scr_rebuilt" `
  --yes
```

注入器会先完整验证所有 JSON 和源文件，再创建输出。它重建文本偏移表及文本池，更新头内 `text_pool_size` 和“声明文件长度”，复制其余声明区，并把声明长度之外的物理尾部逐字节保留。原报错样本 `0000.gsc` 的声明长度 947、物理长度 960 属于受支持的合法包络。

## 已验证与限制

- 111 个真实 GSC 全部可提取，共 11,541 条；识别到 37 个 `0x000E` 选项。
- 零编辑提取/注入后，112 个源文件逐文件 SHA-256 完全一致。
- 真实变长编辑只改变目标 GSC，重新提取成功，且其 15,436 字节物理尾部哈希不变。
- 当前编码仍是 CP932；不可编码字符会在写出前报错。尚未实现中文编码或字体补丁。
- 原始 XFL 路径未提供，因此 XFL 重封和游戏运行时测试尚未完成。请先把重建 GSC 放入测试副本并完成显示、选项跳转与存读档验证，再用于正式补丁。
- WCG 目前只做 PNG 导出，不做 PNG → WCG 回写；姓名映射只服务翻译上下文。

WCG 算法参考与许可证见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。
