; Full conservative disassembly for PROLOG.BIN
; Columns: offset | size | raw bytes | mnemonic | operands/preview
; Unknown or ambiguous VM semantics are preserved as typed atoms / opaque bytes.

0x00000000      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000002      2  0087                                                 WORD_00XX                 u16_be=135, low_byte=135
0x00000004      3  F10000                                               IMM16_F1                  u16_be=0, u16_le=0
0x00000007      1  FF                                                   TERMINATOR_FF             
0x00000008      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x0000000A      3  F10000                                               IMM16_F1                  u16_be=0, u16_le=0
0x0000000D      3  F300FF                                               IMM16_F3                  u16_be=255, u16_le=65280
0x00000010      1  41                                                   OPAQUE_RAW_BYTES          bytes=41
0x00000011      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000013      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00000014      1  FF                                                   TERMINATOR_FF             
0x00000015      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x00000017      1  24                                                   OPAQUE_RAW_BYTES          bytes=24
0x00000018      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000001A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000001C      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x0000001E      1  FF                                                   TERMINATOR_FF             
0x0000001F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000021      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000023      1  FF                                                   TERMINATOR_FF             
0x00000024      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000026      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000028      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x0000002A      1  FF                                                   TERMINATOR_FF             
0x0000002B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000002D      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x0000002F     14  800C50533258303030612E62696E                         LEN8_STRING_CP932         length=12, text="PS2X000a.bin"
0x0000003D      1  FF                                                   TERMINATOR_FF             
0x0000003E      1  FF                                                   TERMINATOR_FF             
0x0000003F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000041      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000043      1  FF                                                   TERMINATOR_FF             
0x00000044      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00000046      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000048      1  FF                                                   TERMINATOR_FF             
0x00000049      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x0000004B      1  53                                                   OPAQUE_RAW_BYTES          bytes=53
0x0000004C      2  0021                                                 WORD_00XX                 u16_be=33, low_byte=33
0x0000004E      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00000050      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x00000052      1  44                                                   OPAQUE_RAW_BYTES          bytes=44
0x00000053      1  FF                                                   TERMINATOR_FF             
0x00000054      1  FF                                                   TERMINATOR_FF             
