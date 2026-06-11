# Lucrezia Scw5.x 结构化文本工具

这里只处理 `Scw5.x` 脚本层，不处理外层 `SCR.pak`。外层封包继续使用前一个 `lucrezia_pak.py`：

```bat
python lucrezia_pak.py unpack SCR.pak scw --add-ext .scw
python lucrezia_pak.py pack scw_chs SCR_chs.pak
```

## 文件说明

```text
common.py   Scw5.x 解码、LZSS、table/block 解析、opcode 文本识别、blockB/tableB 重建
extract.py  按指令流导出 JSON
inject.py   从 JSON 注入，变长重建 blockB/tableB
```

## 提取

```bat
python extract.py scw json
```

默认只导出已确认的剧情/选项相关 opcode：

```text
0x01A8  message_a8
0x01AA  message_aa
0x00CB  call_text_cb    只导出实际含日文的字符串操作数
0x01CE  choice_ce
```

需要同时导出系统/UI 标题：

```bat
python extract.py scw json --include-system
```

验证脚本是否可解析：

```bat
python extract.py scw --verify
```

## JSON 结构

面向翻译字段仍然是：

```json
{
  "scr_msg": "原文",
  "message": "译文"
}
```

定位字段包含：

```text
_file, _index, _opcode, _op_name, _chunk_index, _inst_offset,
_str_table, _str_index, _offset, _size, _encoding, _policy
```

注入时以 `_str_index + scr_msg` 校验，不按字符串池暴力替换。

## 注入

```bat
python inject.py scw json\new scw_chs --clean
```

单文件也可以：

```bat
python inject.py scw\MapRoom01.scw json\new\MapRoom01.scw.json scw_chs\MapRoom01.scw
```

## 零修改重建测试

```bat
python inject.py scw dummy scw_roundtrip --roundtrip --clean
python extract.py scw_roundtrip --verify
```

`--roundtrip` 不读取 `dummy`，只是为了保持参数位置简单。

## 重建策略

不是等长覆盖，也不是扫字符串池替换。流程是：

```text
statement table / blockA 指令流
  -> 确认文本 opcode
  -> 读取 type=16 的 blockB 字符串索引
  -> JSON 导出
  -> 注入时按 _str_index 校验 scr_msg
  -> 重建 blockB 和 tableB
  -> 更新 Scw5.x header 的 size_b / unpacked_size / packed_size
  -> 输出 mode_flag=0 的 XOR-only Scw5.x
```

当前输出采用引擎已有的 raw 分支：`mode_flag=0`，body 只做 XOR，不重新 LZSS 压缩。
