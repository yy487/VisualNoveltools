from collections.abc import Callable, Mapping
from dataclasses import dataclass
import json
from pathlib import Path
import re
from typing import Any, TypedDict

import os
import struct
from dataclasses import field
from typing import Protocol, Type, cast, runtime_checkable


type StringDecoder = Callable[[bytes, int], tuple[str, int]]
type StringEncoder = Callable[[str], bytes]

type TypedValue = U8 | U16 | U32 | U64 | I8 | I16 | I32 | I64 | String | Bytes


@runtime_checkable
class BinaryType(Protocol):
    TAG: str
    STRUCT: struct.Struct | None

    def __new__(cls, value) -> TypedValue: ...


class BinaryError(Exception):
    """二进制读写基础异常。"""


class BufferUnderflowError(BinaryError):
    """读取超出缓冲区时抛出。"""


class CStringNotTerminatedError(BinaryError):
    """读取 C 字符串时找不到 NULL 终止符时抛出。"""


class InvalidTypedValueError(BinaryError):
    """类型值与声明类型不匹配时抛出。"""


class U8(int):
    TAG: str = "u8"
    STRUCT: struct.Struct | None = struct.Struct("<B")

    def __new__(cls, value: int) -> "U8":
        if not isinstance(value, int):
            raise TypeError("u8 应该是 int 类型")
        if not (0 <= value <= 0xFF):
            raise InvalidTypedValueError(f"u8 超出范围: {value}")
        return cast(U8, int.__new__(cls, value))


class U16(int):
    TAG: str = "u16"
    STRUCT: struct.Struct | None = struct.Struct("<H")

    def __new__(cls, value: int) -> "U16":
        if not isinstance(value, int):
            raise TypeError("u16 应该是 int 类型")
        if not (0 <= value <= 0xFFFF):
            raise InvalidTypedValueError(f"u16 超出范围: {value}")
        return cast(U16, int.__new__(cls, value))


class U32(int):
    TAG: str = "u32"
    STRUCT: struct.Struct | None = struct.Struct("<I")

    def __new__(cls, value: int) -> "U32":
        if not isinstance(value, int):
            raise TypeError("u32 应该是 int 类型")
        if not (0 <= value <= 0xFFFFFFFF):
            raise InvalidTypedValueError(f"u32 超出范围: {value}")
        return cast(U32, int.__new__(cls, value))


class U64(int):
    TAG: str = "u64"
    STRUCT: struct.Struct | None = struct.Struct("<Q")

    def __new__(cls, value: int) -> "U64":
        if not isinstance(value, int):
            raise TypeError("u64 应该是 int 类型")
        if not (0 <= value <= 0xFFFFFFFFFFFFFFFF):
            raise InvalidTypedValueError(f"u64 超出范围: {value}")
        return cast(U64, int.__new__(cls, value))


class I8(int):
    TAG: str = "i8"
    STRUCT: struct.Struct | None = struct.Struct("<b")

    def __new__(cls, value: int) -> "I8":
        if not isinstance(value, int):
            raise TypeError("i8 应该是 int 类型")
        if not (-0x80 <= value <= 0x7F):
            raise InvalidTypedValueError(f"i8 超出范围: {value}")
        return cast(I8, int.__new__(cls, value))


class I16(int):
    TAG: str = "i16"
    STRUCT: struct.Struct | None = struct.Struct("<h")

    def __new__(cls, value: int) -> "I16":
        if not isinstance(value, int):
            raise TypeError("i16 应该是 int 类型")
        if not (-0x8000 <= value <= 0x7FFF):
            raise InvalidTypedValueError(f"i16 超出范围: {value}")
        return cast(I16, int.__new__(cls, value))


class I32(int):
    TAG: str = "i32"
    STRUCT: struct.Struct | None = struct.Struct("<i")

    def __new__(cls, value: int) -> "I32":
        if not isinstance(value, int):
            raise TypeError("i32 应该是 int 类型")
        if not (-0x80000000 <= value <= 0x7FFFFFFF):
            raise InvalidTypedValueError(f"i32 超出范围: {value}")
        return cast(I32, int.__new__(cls, value))


class I64(int):
    TAG: str = "i64"
    STRUCT: struct.Struct | None = struct.Struct("<q")

    def __new__(cls, value: int) -> "I64":
        if not isinstance(value, int):
            raise TypeError("i64 应该是 int 类型")
        if not (-0x8000000000000000 <= value <= 0x7FFFFFFFFFFFFFFF):
            raise InvalidTypedValueError(f"i64 超出范围: {value}")
        return cast(I64, int.__new__(cls, value))


class String(str):
    TAG: str = "str"
    STRUCT: struct.Struct | None = None

    def __new__(cls, value: str) -> "String":
        if not isinstance(value, str):
            raise TypeError("String 应该是 str 类型")
        return cast(String, str.__new__(cls, value))


class Bytes(bytes):
    TAG: str = "bytes"
    STRUCT: struct.Struct | None = None

    def __new__(cls, value: bytes) -> "Bytes":
        if not isinstance(value, bytes):
            raise TypeError("Bytes 应该是 bytes 类型")
        return cast(Bytes, bytes.__new__(cls, value))


TYPE_REGISTRY: dict[str, Type[TypedValue]] = {
    U8.TAG: U8,
    U16.TAG: U16,
    U32.TAG: U32,
    U64.TAG: U64,
    I8.TAG: I8,
    I16.TAG: I16,
    I32.TAG: I32,
    I64.TAG: I64,
    Bytes.TAG: Bytes,
    String.TAG: String,
}


def decode_cstr(data: bytes, offset: int, encoding: str) -> tuple[str, int]:
    """
    读取以 NULL 结尾的 C 字符串。

    Args:
        data: 原始二进制数据。
        offset: 起始读取偏移。
        encoding: 文本解码所用编码。

    Returns:
        tuple[str, int]: 解码后的字符串与下一个读取偏移（跳过终止符）。

    Raises:
        CStringNotTerminatedError: 在 `offset` 之后未找到终止符 `0x00`。
    """
    end = data.find(0x00, offset)
    if end < 0:
        raise CStringNotTerminatedError(
            f"未找到 C 字符串结尾: offset={offset}, length={len(data)}"
        )
    return data[offset:end].decode(encoding), end + 1


def encode_cstr(text: str, encoding: str) -> bytes:
    """
    将字符串编码为以 NULL 结尾的 C 字符串字节序列。

    Args:
        text: 待编码文本。
        encoding: 文本编码。

    Returns:
        bytes: 编码结果，末尾附带 `0x00`。
    """
    return text.encode(encoding) + b"\x00"


def to_hex(value: bytes) -> str:
    """
    将字节序列转为大写十六进制字符串（空格分隔）。

    Args:
        value: 待转换字节序列。

    Returns:
        str: 形如 ``"AA BB CC"`` 的十六进制字符串。
    """
    return value.hex(" ").upper()


def retype_like(template: BinaryType, value) -> TypedValue:
    """
    根据模板值的类型，对给定值进行重新构造。

    该函数会使用 `template` 的实际类型（如 U32 / I16 / String / Bytes 等），
    尝试用 `value` 构造一个新的同类型实例。

    本质等价于：`type(template)(value)`，并对异常进行统一包装。

    Args:
        template: 用于提供目标类型的实例（仅使用其类型信息）。
        value: 待转换的原始值。

    Returns:
        TypedValue: 与 `template` 同类型的新实例。
    """
    target_type = type(template)

    try:
        return cast(TypedValue, target_type(value))
    except Exception as exc:
        raise InvalidTypedValueError(
            f"无法将值 {value!r} 转换为类型 {target_type.__name__}"
        ) from exc


def se(value: BinaryType) -> str:
    """
    将带类型信息的值序列化为文本。

    Args:
        value: 待序列化值。

    Returns:
        str: 序列化文本。
            - `Bytes` 输出为 `bytes:AA BB`；
            - `String` 直接输出原文；
            - 其他标量输出为 `tag:value`。
    """
    tag = value.TAG
    if tag == Bytes.TAG:
        return f"{tag}:{to_hex(cast(bytes, value))}"
    if tag == String.TAG:
        return str(value)
    return f"{tag}:{value}"


def de(value: str) -> TypedValue:
    """
    将 `se` 生成的文本反序列化为强类型值。

    Args:
        value: 待反序列化文本。

    Returns:
        TypedValue: 反序列化结果。
            - 不含 `:` 时返回 `String`；
            - 类型标签未知时，回退为原始 `String`；
            - 已知标签按对应类型解析。
    """
    if ":" not in value:
        return String(value)

    raw_type, raw_value = value.split(":", 1)

    cls = TYPE_REGISTRY.get(raw_type)
    if not cls:
        return String(value)
    elif cls is Bytes:
        return Bytes(bytes.fromhex(raw_value))
    elif cls is String:
        return String(raw_value)
    else:
        return cls(int(raw_value, 10))  # type: ignore


@dataclass(slots=True)
class BinaryReader:
    """高性能二进制读取器。"""

    data: bytes
    offset: int = 0
    _view: memoryview = field(init=False, repr=False)

    def __post_init__(self) -> None:
        self._view = memoryview(self.data)

    def is_eof(self) -> bool:
        """
        检查是否已到达数据末尾。

        当偏移量 >= 数据总长度时返回 True，表示下一次读取将触发 BufferUnderflowError。
        """
        return self.offset >= len(self._view)

    def seek(self, offset: int, whence: int = os.SEEK_SET) -> int:
        """
        改变当前读取位置。

        Args:
            offset: 偏移量。
            whence: 相对位置基准：
                - os.SEEK_SET (0): 从起始位置开始（默认）。
                - os.SEEK_CUR (1): 从当前位置开始。
                - os.SEEK_END (2): 从末尾位置开始。

        Returns:
            int: 移动后的绝对偏移量位置。

        Raises:
            ValueError: 如果 whence 参数无效，或计算出的偏移量为负数值。
        """
        if whence == os.SEEK_SET:
            new_offset = offset
        elif whence == os.SEEK_CUR:
            new_offset = self.offset + offset
        elif whence == os.SEEK_END:
            new_offset = len(self._view) + offset
        else:
            raise ValueError(f"无效的 whence 参数: {whence}。应为 0, 1 或 2。")

        # 核心约束：偏移量不能为负
        if new_offset < 0:
            raise ValueError(f"非法的 seek 位置: {new_offset} (不能为负数)")

        # 注意：在只读模式下，通常允许 seek 超过 len(data)，
        # 但这会导致下一次 read 时触发你定义的 _require 报错。
        # 这种设计符合 Python IO 规范，也保持了 seek 操作的 O(1) 性能。
        self.offset = new_offset
        return self.offset

    def tell(self) -> int:
        """
        获取当前读取偏移。

        Returns:
            int: 当前偏移。
        """
        return self.offset

    def fork(self, offset: int | None = None) -> "BinaryReader":
        """
        基于同一底层数据创建一个新的读取器。

        Args:
            offset: 新读取器的起始偏移。若为 ``None``，则使用当前偏移。

        Returns:
            BinaryReader: 指向同一 ``data`` 的新读取器实例。
        """
        target_offset = self.offset if offset is None else offset
        return BinaryReader(self.data, target_offset)

    def startswith(self, prefix: bytes, offset: int | None = None) -> bool:
        """
        判断指定偏移处是否以给定前缀开头。

        Args:
            prefix: 待匹配的字节前缀。
            offset: 匹配起点。若为 ``None``，使用当前偏移。

        Returns:
            bool: 是否匹配成功。
        """
        target_offset = self.offset if offset is None else offset
        return self.data.startswith(prefix, target_offset)

    def _require(self, size: int) -> None:
        if self.offset + size > len(self._view):
            raise BufferUnderflowError(
                f"读取越界: offset={self.offset}, need={size}, total={len(self._view)}"
            )

    def read_scalar[T: BinaryType](self, cls: Type[T]) -> T:
        """
        按给定标量类型读取定长值。

        Args:
            cls: 标量类型（必须定义 `STRUCT`）。

        Returns:
            T: 对应类型实例。

        Raises:
            TypeError: 传入类型不支持定长读取。
            BufferUnderflowError: 缓冲区剩余长度不足。
        """
        if not cls.STRUCT:
            raise TypeError("read方法仅支持定长标量，变长请使用 read_bytes/read_str")

        size = cls.STRUCT.size
        self._require(size)

        val = cls.STRUCT.unpack_from(self._view, self.offset)[0]
        self.offset += size
        return cast(T, cls(val))

    def read_u8(self) -> U8:
        """读取无符号 8 位整数。

        Returns:
            U8: 读取结果。

        Raises:
            BufferUnderflowError: 缓冲区剩余长度不足。
        """
        return self.read_scalar(U8)

    def read_u16(self) -> U16:
        """读取无符号 16 位整数。

        Returns:
            U16: 读取结果。

        Raises:
            BufferUnderflowError: 缓冲区剩余长度不足。
        """
        return self.read_scalar(U16)

    def read_u32(self) -> U32:
        """读取无符号 32 位整数。

        Returns:
            U32: 读取结果。

        Raises:
            BufferUnderflowError: 缓冲区剩余长度不足。
        """
        return self.read_scalar(U32)

    def read_u64(self) -> U64:
        """读取无符号 64 位整数。

        Returns:
            U64: 读取结果。

        Raises:
            BufferUnderflowError: 缓冲区剩余长度不足。
        """
        return self.read_scalar(U64)

    def read_i8(self) -> I8:
        """读取有符号 8 位整数。

        Returns:
            I8: 读取结果。

        Raises:
            BufferUnderflowError: 缓冲区剩余长度不足。
        """
        return self.read_scalar(I8)

    def read_i16(self) -> I16:
        """读取有符号 16 位整数。

        Returns:
            I16: 读取结果。

        Raises:
            BufferUnderflowError: 缓冲区剩余长度不足。
        """
        return self.read_scalar(I16)

    def read_i32(self) -> I32:
        """读取有符号 32 位整数。

        Returns:
            I32: 读取结果。

        Raises:
            BufferUnderflowError: 缓冲区剩余长度不足。
        """
        return self.read_scalar(I32)

    def read_i64(self) -> I64:
        """读取有符号 64 位整数。

        Returns:
            I64: 读取结果。

        Raises:
            BufferUnderflowError: 缓冲区剩余长度不足。
        """
        return self.read_scalar(I64)

    def read_bytes(self, length: int) -> Bytes:
        """
        读取指定长度字节。

        Args:
            length: 字节长度。

        Returns:
            Bytes: 读取结果。

        Raises:
            BufferUnderflowError: 缓冲区剩余长度不足。
        """
        self._require(length)
        start = self.offset
        self.offset += length
        return Bytes(self._view[start : start + length].tobytes())

    def read_rest_bytes(self, *, leave_size: int = 0) -> Bytes:
        """
        读取从当前偏移量到数据末尾、并保留指定长度字节之前的所有数据。

        Args:
            leave_size: 数据末尾需要保留不读的字节数。

        Returns:
            Bytes: 读取的字节片段。

        Raises:
            ValueError: 如果 leave_size 为负数。
            BufferUnderflowError: 如果当前偏移量已经超出了可读取边界（即剩余空间不足以保留 leave_size）。
        """
        if leave_size < 0:
            raise ValueError(f"保留的字节数不能为负数: {leave_size}")

        # 计算在保留 leave_size 之后，理论上应该读取的长度
        length = len(self._view) - leave_size - self.offset

        # 如果 length < 0，说明当前 offset 已经越界进入了保留区，直接触发越界报错
        if length < 0:
            raise BufferUnderflowError(
                f"读取越界 (保留区重叠): offset={self.offset}, leave_size={leave_size}, total={len(self._view)}"
            )

        return self.read_bytes(length)

    def read_str(
        self,
        *,
        codec: StringDecoder = lambda data, offset: decode_cstr(data, offset, "cp932"),
    ) -> String:
        """
        读取字符串。

        Args:
            codec: 自定义解码器，输入为 `(data, offset)`，输出为
                `(text, new_offset)`。

        Returns:
            String: 读取并解码后的字符串。
        """
        text, new_offset = codec(self.data, self.offset)
        self.offset = new_offset
        return String(text)


@dataclass(slots=True)
class BinaryWriter:
    """高性能二进制写入器。"""

    _buf: bytearray = field(init=False, repr=False)

    def __post_init__(self) -> None:
        self._buf = bytearray()

    def write_u8(self, value: int) -> None:
        """
        写入无符号 8 位整数。

        Args:
            value: 待写入整数。

        Returns:
            None
        """
        self._buf += U8.STRUCT.pack(value)  # type: ignore

    def write_u16(self, value: int) -> None:
        """
        写入无符号 16 位整数。

        Args:
            value: 待写入整数。

        Returns:
            None
        """
        self._buf += U16.STRUCT.pack(value)  # type: ignore

    def write_u32(self, value: int) -> None:
        """
        写入无符号 32 位整数。

        Args:
            value: 待写入整数。

        Returns:
            None
        """
        self._buf += U32.STRUCT.pack(value)  # type: ignore

    def write_u64(self, value: int) -> None:
        """
        写入无符号 64 位整数。

        Args:
            value: 待写入整数。

        Returns:
            None
        """
        self._buf += U64.STRUCT.pack(value)  # type: ignore

    def write_i8(self, value: int) -> None:
        """
        写入有符号 8 位整数。

        Args:
            value: 待写入整数。

        Returns:
            None
        """
        self._buf += I8.STRUCT.pack(value)  # type: ignore

    def write_i16(self, value: int) -> None:
        """
        写入有符号 16 位整数。

        Args:
            value: 待写入整数。

        Returns:
            None
        """
        self._buf += I16.STRUCT.pack(value)  # type: ignore

    def write_i32(self, value: int) -> None:
        """
        写入有符号 32 位整数。

        Args:
            value: 待写入整数。

        Returns:
            None
        """
        self._buf += I32.STRUCT.pack(value)  # type: ignore

    def write_i64(self, value: int) -> None:
        """
        写入有符号 64 位整数。

        Args:
            value: 待写入整数。

        Returns:
            None
        """
        self._buf += I64.STRUCT.pack(value)  # type: ignore

    def write_bytes(self, value: bytes) -> None:
        """
        写入原始字节序列。

        Args:
            value: 待写入字节序列。

        Returns:
            None

        Raises:
            InvalidTypedValueError: 传入值不是 `bytes`。
        """
        if not isinstance(value, bytes):
            raise InvalidTypedValueError("bytes 需要 bytes")
        self._buf += value

    def write_str(
        self,
        value: str,
        *,
        codec: StringEncoder = lambda value: encode_cstr(value, "cp932"),
    ) -> None:
        """写入字符串。

        Args:
            value: 待写入字符串。
            codec: 字符串编解码器。

        Returns:
            None
        """
        if not isinstance(value, str):
            raise InvalidTypedValueError("str 需要 str")
        self._buf += codec(value)

    def write(
        self,
        value: BinaryType,
        *,
        codec: StringEncoder = lambda value: encode_cstr(value, "cp932"),
    ) -> None:
        """
        按值类型自动写入。

        Args:
            value: 待写入的强类型值。
            codec: 字符串编码器，仅在 `String` 类型时使用。

        Returns:
            None

        Raises:
            TypeError: 遇到不支持的类型。
        """
        cls = type(value)

        if cls.STRUCT:
            self._buf += cls.STRUCT.pack(value)
        elif isinstance(value, Bytes):
            self._buf += value
        elif isinstance(value, String):
            self._buf += codec(value)
        else:
            raise TypeError(f"未知类型: {value}")

    def to_bytes(self) -> bytes:
        """
        导出当前缓冲区内容。

        Returns:
            bytes: 写入器中的全部字节数据。
        """
        return bytes(self._buf)

    def tell(self) -> int:
        """
        返回当前已写入的数据长度（即下一个写入位置的偏移量）。

        Returns:
            int: 当前已写入的数据长度偏移。
        """
        return len(self._buf)


def ensure_str(val: object, context: str = "") -> str:
    """确保val为str，否则抛出TypeError异常"""
    if not isinstance(val, str):
        msg = f"期待 str，但收到了 {type(val).__name__}"
        if context:
            msg += f" (上下文: {context})"
        raise TypeError(msg)
    return val


class InstError(Exception):
    """指令解析异常。"""


class MatchFailed(InstError):
    """当前 opcode 候选匹配失败（可回溯尝试下一个候选）。"""


class EndOfParsing(InstError):
    """主动终止解析。"""


class UnknownOpcodeError(InstError):
    """遇到未知 opcode。"""


type InstArg = str


class Instruction(TypedDict):
    """单条指令的结构化表示。"""

    op: str
    offset: int
    args: list[InstArg]
    meta: dict


type ParseContext = Instruction
type HandlerResult = InstArg | list[InstArg] | None
type HandlerCallable = Callable[[BinaryReader, ParseContext], HandlerResult]
type HandlerCallableWithArgs = Callable[..., HandlerResult]
type FixOffsetIndicesResolver = Callable[[Instruction], list[int]]


@dataclass(frozen=True, slots=True)
class ParseOptions:
    """解析配置。"""

    file_name: str = "<unknown>"
    offset: int = 0
    max_chunk_print_size: int = 18


def _normalize_parse_options(
    debug_info: ParseOptions | Mapping[str, object],
) -> ParseOptions:
    """
    将输入配置规整为 ParseOptions。

    Args:
        debug_info: `ParseOptions` 或兼容旧接口的 `Mapping`。

    Returns:
        规整后的 `ParseOptions` 对象。
    """
    if isinstance(debug_info, ParseOptions):
        return debug_info

    file_name = str(debug_info.get("file_name", "<unknown>"))
    base_offset = debug_info.get("offset", 0)
    max_chunk_print_size = debug_info.get("max_chunk_print_size", 18)
    if not isinstance(base_offset, int):
        raise TypeError(f"debug_info.offset 需要 int，实际为: {base_offset}")
    if not isinstance(max_chunk_print_size, int):
        raise TypeError(
            f"debug_info.max_chunk_print_size 需要 int，实际为: {max_chunk_print_size}"
        )

    return ParseOptions(
        file_name=file_name,
        offset=base_offset,
        max_chunk_print_size=max_chunk_print_size,
    )


def _ensure_scalar_inst_arg(value: HandlerResult, *, source: str) -> InstArg:
    """
    校验 handler 结果必须为标量字符串值。

    Args:
        value: handler 返回值。
        source: 调用来源标识，用于生成报错文案。

    Returns:
        合法的 `InstArg` 字符串。
    """
    if not isinstance(value, str):
        raise ValueError(f"{source} 只支持标量字符串结果，实际得到: {value}")
    return value


class Handler:
    def __init__(self, func: HandlerCallableWithArgs) -> None:
        self.func: HandlerCallableWithArgs = func

    def __call__(self, reader: BinaryReader, ctx: ParseContext) -> HandlerResult:
        return self.func(reader, ctx)

    def repeat(self, count: int) -> "Handler":
        """
        构造固定次数重复 handler。

        Args:
            count: 重复次数。

        Returns:
            新的重复 handler。
        """

        def wrapped_handler(reader: BinaryReader, ctx: ParseContext) -> HandlerResult:
            results: list[InstArg] = []

            for _ in range(count):
                result = _ensure_scalar_inst_arg(
                    self.func(reader, ctx), source="repeat"
                )
                results.append(result)

            return results

        return Handler(wrapped_handler)

    def repeat_var(self, var_index: int = -1) -> "Handler":
        """
        构造按上下文变量次数重复的 handler。

        Args:
            var_index: 从 `ctx["args"]` 中读取重复次数的索引。

        Returns:
            新的重复 handler。
        """

        def wrapped_handler(reader: BinaryReader, ctx: ParseContext) -> HandlerResult:
            # 从上下文中获取重复次数
            args = ctx["args"]
            if not args:
                raise ValueError("repeat_var 上下文 args 为空")

            count_value = args[var_index]
            count = de(count_value)
            if not isinstance(count, int) or count <= 0:
                raise ValueError(f"非法的 count_value: {count_value}")

            results: list[InstArg] = []

            for _ in range(count):
                result = _ensure_scalar_inst_arg(
                    self.func(reader, ctx), source="repeat_var"
                )
                results.append(result)

            return results

        return Handler(wrapped_handler)

    def args(self, *handler_args: object) -> "Handler":
        """
        构造带固定额外参数的 handler。

        Args:
            *handler_args: 固定附加参数。

        Returns:
            新的包装 handler。
        """

        def wrapped_handler(reader: BinaryReader, ctx: ParseContext) -> HandlerResult:
            return self.func(reader, ctx, *handler_args)

        return Handler(wrapped_handler)

    def verify(self, predicate: Callable[[object], bool]) -> "Handler":
        """通用校验：传入一个 lambda/函数，如果返回 False 则匹配失败并回溯"""

        def wrapped_handler(reader: BinaryReader, ctx: ParseContext) -> HandlerResult:
            res = self.func(reader, ctx)
            raw_val: object = res
            if isinstance(res, str):
                raw_val = de(res)

            if not predicate(raw_val):
                raise MatchFailed()
            return res

        return Handler(wrapped_handler)

    def eq(self, target: object) -> "Handler":
        """值匹配校验快捷方式"""
        return self.verify(lambda x: x == target)


def u8_handler(reader: BinaryReader, ctx: ParseContext) -> HandlerResult:
    _ = ctx
    return se(reader.read_u8())


def u16_handler(reader: BinaryReader, ctx: ParseContext) -> HandlerResult:
    _ = ctx
    return se(reader.read_u16())


def u32_handler(reader: BinaryReader, ctx: ParseContext) -> HandlerResult:
    _ = ctx
    return se(reader.read_u32())


def i8_handler(reader: BinaryReader, ctx: ParseContext) -> HandlerResult:
    _ = ctx
    return se(reader.read_i8())


def i16_handler(reader: BinaryReader, ctx: ParseContext) -> HandlerResult:
    _ = ctx
    return se(reader.read_i16())


def i32_handler(reader: BinaryReader, ctx: ParseContext) -> HandlerResult:
    _ = ctx
    return se(reader.read_i32())


def string_handler(reader: BinaryReader, ctx: ParseContext) -> HandlerResult:
    _ = ctx
    return se(reader.read_str())


def end_handler(reader: BinaryReader, ctx: ParseContext) -> HandlerResult:
    _ = reader, ctx
    raise EndOfParsing()


def byte_slice_handler(
    reader: BinaryReader, ctx: ParseContext, length: int
) -> HandlerResult:
    _ = ctx
    return se(reader.read_bytes(length))


u8 = Handler(u8_handler)
u16 = Handler(u16_handler)
u32 = Handler(u32_handler)
i8 = Handler(i8_handler)
i16 = Handler(i16_handler)
i32 = Handler(i32_handler)
string = Handler(string_handler)
byte_slice = Handler(byte_slice_handler)
end = Handler(end_handler)


def parse_data(
    debug_info: ParseOptions | Mapping[str, object],
    reader: BinaryReader,
    inst_map: Mapping[bytes, list[Handler] | list[list[Handler]]],
    fallback_handler: Handler | None = None,
) -> list[Instruction]:
    """
    按照声明式 `inst_map` 解析二进制流。

    Args:
        debug_info: 解析调试配置，兼容 `ParseOptions` 与旧版 dict 传参。
        reader: 二进制读取器。
        inst_map: opcode 到 handler 链（或 handler 链列表）的映射表。
        fallback_handler: 兜底处理函数，当所有 opcode 均不匹配时调用。

    Returns:
        解析后的指令列表。
    """
    options = _normalize_parse_options(debug_info)
    file_name = options.file_name
    base_offset = options.offset
    max_chunk_print_size = options.max_chunk_print_size

    insts: list[Instruction] = []

    # 按键长度降序排序
    sorted_keys = sorted(inst_map.keys(), key=len, reverse=True)

    while not reader.is_eof():
        matched = False
        start_offset = reader.tell()

        for signature in sorted_keys:
            if reader.startswith(signature, start_offset):
                raw_handlers = inst_map[signature]
                handler_chains: list[list[Handler]]
                if len(raw_handlers) > 0 and isinstance(raw_handlers[0], list):
                    handler_chains = raw_handlers  # type: ignore
                else:
                    handler_chains = [raw_handlers]  # type: ignore

                signature_len = len(signature)

                # 尝试每一个候选链 (Any 逻辑)
                for handlers in handler_chains:
                    cur_inst: Instruction = {
                        "op": to_hex(signature),
                        "offset": start_offset + base_offset,
                        "args": [],
                        "meta": {},
                    }
                    param_offset = start_offset + signature_len
                    trial_reader = reader.fork(param_offset)
                    try:
                        for handler in handlers:
                            res = handler(trial_reader, cur_inst)
                            if res is not None:
                                if isinstance(res, list):
                                    cur_inst["args"].extend(res)
                                else:
                                    cur_inst["args"].append(res)

                        # 整个链条处理成功
                        param_offset = trial_reader.tell()
                        insts.append(cur_inst)
                        reader.seek(param_offset)
                        matched = True
                        break  # 跳出候选链循环
                    except MatchFailed:
                        # 当前链不匹配，尝试下一个候选链
                        continue
                    except EndOfParsing:
                        param_offset = trial_reader.tell()
                        insts.append(cur_inst)
                        reader.seek(param_offset)
                        return insts
                    except Exception as exc:
                        prev_inst = insts[-1] if insts else None
                        raise InstError(
                            f"{file_name}: 处理 Opcode {to_hex(signature)} 在 "
                            + f"{hex(start_offset + base_offset)} 发生致命错误\n"
                            + f"当前指令草稿: {cur_inst}\n"
                            + f"前一条已解析指令: {prev_inst}\n"
                            + f"原始异常: {type(exc).__name__}: {exc}"
                        ) from exc

                if matched:
                    break

        # 尝试兜底处理
        if not matched and fallback_handler:
            cur_inst = {
                "op": "",
                "offset": start_offset + base_offset,
                "args": [],
                "meta": {},
            }
            trial_reader = reader.fork(start_offset)
            try:
                res = fallback_handler(trial_reader, cur_inst)
                if res is not None:
                    if isinstance(res, list):
                        cur_inst["args"].extend(res)
                    else:
                        cur_inst["args"].append(res)

                insts.append(cur_inst)
                reader.seek(trial_reader.tell())
                matched = True
            except MatchFailed:
                pass
            except EndOfParsing:
                param_offset = trial_reader.tell()
                insts.append(cur_inst)
                reader.seek(param_offset)
                return insts
            except Exception as exc:
                prev_inst = insts[-1] if insts else None
                raise InstError(
                    f"{file_name}: fallback 处理器在 "
                    + f"{hex(start_offset + base_offset)} 发生致命错误\n"
                    + f"当前指令草稿: {cur_inst}\n"
                    + f"前一条已解析指令: {prev_inst}\n"
                    + f"原始异常: {type(exc).__name__}: {exc}"
                ) from exc

        if not matched:
            unknown_byte = reader.data[start_offset]
            chunk = reader.data[start_offset : start_offset + max_chunk_print_size]
            has_more = (len(reader.data) - start_offset) > max_chunk_print_size
            suffix = "..." if has_more else ""
            prev_inst = insts[-1] if insts else None

            # print(
            #     f"\n{'=' * 40}\n"
            #     + f"解析失败 [文件: {file_name}]\n"
            #     + f"未知 Opcode: {hex(unknown_byte)} 偏移在: {hex(start_offset + base_offset)}\n"
            #     + f"数据片段 (HEX):    {to_hex(chunk)}{suffix}\n"
            #     + f"数据片段 (ASCII): {repr(chunk)}{suffix}\n"
            #     + f"{'-' * 40}\n"
            #     + f"前一条已解析指令: {prev_inst}\n"
            #     + f"{'=' * 40}"
            # )
            raise UnknownOpcodeError(
                f"\n{'=' * 40}\n"
                + f"解析失败 [文件: {file_name}]\n"
                + f"未知 Opcode: {hex(unknown_byte)} 偏移在: {hex(start_offset + base_offset)}\n"
                + f"数据片段 (HEX):    {to_hex(chunk)}{suffix}\n"
                + f"数据片段 (ASCII): {repr(chunk)}{suffix}\n"
                + f"{'-' * 40}\n"
                + f"前一条已解析指令: {prev_inst}\n"
                + f"{'=' * 40}"
            )
            return insts

    return insts


def h(hex_str: str) -> bytes:
    """
    将十六进制字符串转换为 bytes。

    Args:
        hex_str: 例如 ``"01 FF"``。

    Returns:
        对应字节串。
    """
    return bytes.fromhex(hex_str)


def assemble_one_inst(
    entry: Instruction,
    codec: StringEncoder = lambda value: encode_cstr(value, "cp932"),
) -> bytes:
    """
    将一条反汇编后的指令 JSON 转换为二进制。

    Args:
        entry: 指令对象。
        codec: 字符串编码器。

    Returns:
        单条指令的二进制数据。
    """
    writer = BinaryWriter()

    # 1. opcode
    # "00 03" -> bytes
    op_bytes = bytes.fromhex(ensure_str(entry["op"]))
    writer.write_bytes(op_bytes)

    # 2. 参数顺序拼接
    args = entry.get("args", [])

    for item in args:
        writer.write(de(item), codec=codec)

    return writer.to_bytes()


def fix_offset(
    file: str,
    insts: list[Instruction],
    old2new: Mapping[int, int],
    fix_inst_map: Mapping[str, list[int] | FixOffsetIndicesResolver],
) -> list[Instruction]:
    """
    修复指令中的偏移，将旧偏移映射为新偏移。

    Args:
        file: 当前处理文件名。
        insts: 指令列表。
        old2new: 旧偏移到新偏移的映射。
        fix_inst_map: 需要修复的指令的参数索引（或索引解析器）。

    Returns:
        修复后的指令列表（原地修改并返回）。
    """
    for inst in insts:
        key = ensure_str(inst["op"])
        if key not in fix_inst_map:
            continue

        indices_spec = fix_inst_map[key]

        # 支持列表或回调函数
        if callable(indices_spec):
            indices = indices_spec(inst)
        else:
            indices = indices_spec

        args = inst.get("args")

        for i in indices:
            raw_value = args[i]

            old_offset = de(raw_value)
            if not isinstance(old_offset, int):
                raise TypeError(f"偏移字段不是整型: {raw_value}")

            if old_offset not in old2new:
                raise ValueError(f"{file}, {inst} 指向不存在的 offset: {old_offset}")

            new_offset = old2new[old_offset]
            args[i] = se(type(old_offset)(new_offset))

    return insts


def _normalize_suffix(suffix: str) -> str:
    if not suffix:
        raise ValueError("后缀名不能为空字符串")
    return suffix if suffix.startswith(".") else f".{suffix}"


def collect_files(root: Path, suffix: str | None = None) -> list[Path]:
    if not root.is_dir():
        raise NotADirectoryError(f"不是有效目录: {root}")

    normalized_suffix = (
        _normalize_suffix(suffix).lower() if suffix is not None else None
    )
    files = [
        p
        for p in root.rglob("*")
        if p.is_file()
        and (normalized_suffix is None or p.suffix.lower() == normalized_suffix)
    ]

    # 使用标准库 re 实现自然排序（Natural Sort）
    def natural_sort_key(p: Path) -> list[Any]:
        relative_str = p.relative_to(root).as_posix()
        return [
            int(text) if text.isdigit() else text.lower()
            for text in re.split(r"(\d+)", relative_str)
        ]

    return sorted(files, key=natural_sort_key)


def write_json(
    path: Path,
    value: object,
    *,
    create_dir: bool = True,
    ensure_ascii: bool = False,
    indent: int | None = 2,
    encoding: str = "utf-8",
):
    if create_dir:
        path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding=encoding) as f:
        json.dump(value, f, ensure_ascii=ensure_ascii, indent=indent)


def read_json(path: Path, encoding: str = "utf-8") -> Any:
    with path.open("r", encoding=encoding) as f:
        return json.load(f)
