# ACV1 script.dat 通用归档档案

## 项目与边界

- 项目名称：ACV1 / legacy `script.dat` 通用归档工具。
- 已验证游戏：`姉小路直子と銀色の死神`。
- 参考实现：`H:\vn-tool\AVC1\らぶおぶ恋愛皇帝ofLOVE!\dat_tool.py` 与 `H:\vn-tool\AVC1\真剣で私に恋しなさい! A 猟犬ルートアフター\dat_tool.py`。
- 真实样本：`E:\GAL\jxl\姉小路直子と銀色の死神\work\script.dat`。
- 开发工作目录：`H:\IDA-PRO-MCP\acv1_dat_tool`。
- 正式工具目录：未确认；当前不得覆盖 `H:\vn-tool` 下的参考工具。
- 实现：Rust 2024；现有参考工具为 Python。

## 归档结构

- 扩展名：`.dat`；当前用途为 `script.dat`。
- `ACV1` 分支：magic `ACV1`；count XOR `0x8B6A4E5F`；offset 额外 XOR 同一常量。
- legacy 分支：无 magic；count XOR `0x26ACA46E`；offset 无额外常量。
- 索引项：21 字节，依次为 `key_lo:u32`、`key_hi:u32`、混淆 flag、混淆 offset、混淆 packed size、混淆 out capacity。
- payload：游戏名 CP932 字节的 CRC64-ECMA 低 32 位 XOR `key_lo`；只 XOR 完整 DWORD；随后 zlib inflate。
- 文件名：归档不保存原文件名。工具使用索引与脚本首个 `*label` 生成稳定工作文件名，此命名不参与格式解析。
- 对齐：没有统一对齐假设。索引后、payload 之间和文件尾的所有未引用字节按 opaque region 保存。
- 变长回封：整体重建归档表及所有 payload offset/size；默认扩展 out capacity；不修改内部脚本引用。

## 脚本与本地化状态

- 脚本编码：目标样本可按 CP932 观察标签；全格式编码规则未确认。
- 脚本结构、正文、选项、系统文本、跳转、标签语义、控制符和换行：未确认，本工具不解析。
- name/message 组织规则：未确认且不适用于当前归档层交付。
- 角色名写回与 `_scr_name`：未实现。
- 翻译 JSON：未实现；本工具的 JSON 仅为 UTF-8 归档 manifest。

## 证据与验证

- `姉小路直子と銀色の死神` 标题 CRC64：`0xF9EF88FE7D5DC8F6`；payload key 低 32 位：`0x7D5DC8F6`。
- 目标样本 SHA-256：`B7D275DF961D8E7292B90138DE3969DA4A7C6CA46D569146CE046FDECC11400F`。
- 目标样本为 legacy 分支，95 项，索引末尾 `0x7CF`，首 payload `0x7D7`，索引后 opaque gap 为 8 字节。
- 95/95 payload 已由参考实现成功解密和 zlib 解压，总明文 2,119,938 字节。
- `ACV1` 分支结构来自参考反汇编工具；真实样本回环状态在验收后更新。
- 未知 gap 与尾部数据：`opaque-preserved`。
- magic、表项边界、CRC/XOR/zlib：`confirmed-structure`；没有运行时语义声明。

