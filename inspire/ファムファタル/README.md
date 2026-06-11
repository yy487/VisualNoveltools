# IDA/XAF Script.IDA 解包/封包工具

适用样本：`FF.exe` 使用的 `Script.IDA`。

## 已确认格式

- 文件头：`XAF\0`，版本 `0x00011400`。
- 目录项：MFC `CArchive` 风格记录，前 7 个 `uint32`，后接两个 `FILETIME` 和三个变长 `CString`。
- 字符串长度编码：`u8`；若为 `0xFF`，再读 `u16`；若 `u16 == 0xFFFF`，再读 `u32`。
- 目录终止：一个空 sentinel 记录，随后立即进入 payload 区。
- payload flag：
  - `0x01`：逐字节 NOT。
  - `0x02`：链式 XOR。
  - `0x08`：链式 ADD/SUB。
  - `0x10`：zlib。
  - `0x04`：旧 RLE wrapper，工具已实现，但当前样本未使用。
- 当前 `Script.IDA` 中实际 flag 只有 `0x0B` 和 `0x1B`。

## 使用

列目录：

```bash
python ida_xaf_tool.py list Script.IDA
```

解包：

```bash
python ida_xaf_tool.py unpack Script.IDA Script_unpacked
```

会生成：

```text
Script_unpacked/
├─ files/                  # 解密/解压后的实际文件
└─ _ida_xaf_manifest.json  # 封包需要的目录元数据
```

重封包：

```bash
python ida_xaf_tool.py pack Script_unpacked Script_rebuilt.IDA
```

验证原包与解包目录是否一致：

```bash
python ida_xaf_tool.py verify Script.IDA Script_unpacked
```

回环测试：

```bash
python ida_xaf_tool.py roundtrip Script.IDA rt_tmp
```

说明：重封包会重新 zlib 压缩，所以输出包不保证与原包 byte-exact，但再次解包后的 decoded 文件应与输入目录一致。
