; Full conservative disassembly for PS2I003A.BIN
; Columns: offset | size | raw bytes | mnemonic | operands/preview
; Unknown or ambiguous VM semantics are preserved as typed atoms / opaque bytes.

0x00000000      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000002      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000004      2  F20B                                                 IMM8_F2                   u8=11, s8=11
0x00000006      1  FF                                                   TERMINATOR_FF             
0x00000007      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000009      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000000B      1  FF                                                   TERMINATOR_FF             
0x0000000C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000000E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000010      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000012      1  FF                                                   TERMINATOR_FF             
0x00000013      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000015      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x00000017     14  800C50533249303034612E62696E                         LEN8_STRING_CP932         length=12, text="PS2I004a.bin"
0x00000025      1  FF                                                   TERMINATOR_FF             
0x00000026      1  FF                                                   TERMINATOR_FF             
0x00000027      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000029      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000002B      1  FF                                                   TERMINATOR_FF             
0x0000002C      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x0000002E      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000030      1  FF                                                   TERMINATOR_FF             
0x00000031      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x00000033      1  3B                                                   OPAQUE_RAW_BYTES          bytes=3B
0x00000034      2  0021                                                 WORD_00XX                 u16_be=33, low_byte=33
0x00000036      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00000038      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x0000003A      1  2C                                                   OPAQUE_RAW_BYTES          bytes=2C
0x0000003B      1  FF                                                   TERMINATOR_FF             
0x0000003C      1  FF                                                   TERMINATOR_FF             
