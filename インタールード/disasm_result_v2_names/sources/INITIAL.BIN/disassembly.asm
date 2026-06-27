; Full conservative disassembly for INITIAL.BIN
; Columns: offset | size | raw bytes | mnemonic | operands/preview
; Unknown or ambiguous VM semantics are preserved as typed atoms / opaque bytes.

0x00000000      2  0049                                                 WORD_00XX                 u16_be=73, low_byte=73
0x00000002      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000004      1  FF                                                   TERMINATOR_FF             
0x00000005     10  8008737461792E62696E                                 LEN8_STRING_CP932         length=8, text="stay.bin"
0x0000000F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000011      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000013      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000015      1  FF                                                   TERMINATOR_FF             
0x00000016      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000018      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000001A      1  FF                                                   TERMINATOR_FF             
0x0000001B      2  0068                                                 WORD_00XX                 u16_be=104, low_byte=104
0x0000001D      2  0031                                                 WORD_00XX                 u16_be=49, low_byte=49
0x0000001F      2  0089                                                 WORD_00XX                 u16_be=137, low_byte=137
0x00000021      1  FF                                                   TERMINATOR_FF             
0x00000022      1  FF                                                   TERMINATOR_FF             
