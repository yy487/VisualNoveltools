# 字库实现对比基准

`pc88/src/font.rs` 与 `pc98/src/font.rs` 分别原样保存外部工具库
`PC88/foxy/src/font.rs`、`PC98/源氏/src/font.rs`，并保留其 `include_bytes!` 所需资源。
来源标识、文件大小与 SHA-256 记录在 [provenance.json](../../provenance.json)。

这些文件只由 `tests/compatibility.rs` 编译，不进入库和正式程序；请勿用于新增功能。
原实现的内置测试与新旧差分一起运行。参考文件保持原样，不为适应新接口改写它们。
源氏所需的 `crate::sha256_hex` 由测试入口提供。

允许新模块拒绝明确的旧缺陷输入，此类变化有独立回归测试；合法旧用例仍要求输出一致。
