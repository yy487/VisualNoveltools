; Full conservative disassembly for COLOR.BIN
; Columns: offset | size | raw bytes | mnemonic | operands/preview
; Unknown or ambiguous VM semantics are preserved as typed atoms / opaque bytes.

0x00000000      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000002      2  001F                                                 WORD_00XX                 u16_be=31, low_byte=31
0x00000004      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000006      1  FF                                                   TERMINATOR_FF             
0x00000007      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000009      2  0028                                                 WORD_00XX                 u16_be=40, low_byte=40
0x0000000B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000000D      1  FF                                                   TERMINATOR_FF             
0x0000000E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000010      2  0029                                                 WORD_00XX                 u16_be=41, low_byte=41
0x00000012      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000014      1  FF                                                   TERMINATOR_FF             
0x00000015      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000017      2  002A                                                 WORD_00XX                 u16_be=42, low_byte=42
0x00000019      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000001B      1  FF                                                   TERMINATOR_FF             
0x0000001C      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x0000001E      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000020      1  FF                                                   TERMINATOR_FF             
0x00000021      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000023      1  E1                                                   OPAQUE_RAW_BYTES          bytes=E1
0x00000024      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000026      2  001F                                                 WORD_00XX                 u16_be=31, low_byte=31
0x00000028      3  F1001F                                               IMM16_F1                  u16_be=31, u16_le=7936
0x0000002B      2  F21D                                                 IMM8_F2                   u8=29, s8=29
0x0000002D      1  11                                                   OPAQUE_RAW_BYTES          bytes=11
0x0000002E      2  F21D                                                 IMM8_F2                   u8=29, s8=29
0x00000030      1  03                                                   OPAQUE_RAW_BYTES          bytes=03
0x00000031      1  FF                                                   TERMINATOR_FF             
0x00000032      2  004C                                                 WORD_00XX                 u16_be=76, low_byte=76
0x00000034      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x00000036      3  F1001F                                               IMM16_F1                  u16_be=31, u16_le=7936
0x00000039      1  FF                                                   TERMINATOR_FF             
0x0000003A      2  001D                                                 WORD_00XX                 u16_be=29, low_byte=29
0x0000003C     12  800A4150303030322E767476                             LEN8_STRING_CP932         length=10, text="AP0002.vtv"
0x00000048     12  800A4150303030332E767476                             LEN8_STRING_CP932         length=10, text="AP0003.vtv"
0x00000054     12  800A4150303030342E767476                             LEN8_STRING_CP932         length=10, text="AP0004.vtv"
0x00000060     12  800A4150303030352E767476                             LEN8_STRING_CP932         length=10, text="AP0005.vtv"
0x0000006C     12  800A4150303030362E767476                             LEN8_STRING_CP932         length=10, text="AP0006.vtv"
0x00000078     12  800A4150303030382E767476                             LEN8_STRING_CP932         length=10, text="AP0008.vtv"
0x00000084     12  800A4150303030392E767476                             LEN8_STRING_CP932         length=10, text="AP0009.vtv"
0x00000090     12  800A4150303031302E767476                             LEN8_STRING_CP932         length=10, text="AP0010.vtv"
0x0000009C     12  800A4150303031312E767476                             LEN8_STRING_CP932         length=10, text="AP0011.vtv"
0x000000A8     12  800A4150303031322E767476                             LEN8_STRING_CP932         length=10, text="AP0012.vtv"
0x000000B4     12  800A4150303031342E767476                             LEN8_STRING_CP932         length=10, text="AP0014.vtv"
0x000000C0     12  800A4150303031362E767476                             LEN8_STRING_CP932         length=10, text="AP0016.vtv"
0x000000CC     12  800A4549303030332E767476                             LEN8_STRING_CP932         length=10, text="EI0003.vtv"
0x000000D8     12  800A4549303030342E767476                             LEN8_STRING_CP932         length=10, text="EI0004.vtv"
0x000000E4     12  800A4549303030352E767476                             LEN8_STRING_CP932         length=10, text="EI0005.vtv"
0x000000F0     12  800A4549303030382E767476                             LEN8_STRING_CP932         length=10, text="EI0008.vtv"
0x000000FC     12  800A4554303030312E767476                             LEN8_STRING_CP932         length=10, text="ET0001.vtv"
0x00000108     12  800A4554303030322E767476                             LEN8_STRING_CP932         length=10, text="ET0002.vtv"
0x00000114     12  800A4554303030332E767476                             LEN8_STRING_CP932         length=10, text="ET0003.vtv"
0x00000120     12  800A4554303030342E767476                             LEN8_STRING_CP932         length=10, text="ET0004.vtv"
0x0000012C     12  800A4554303030352E767476                             LEN8_STRING_CP932         length=10, text="ET0005.vtv"
0x00000138     12  800A4554303030362E767476                             LEN8_STRING_CP932         length=10, text="ET0006.vtv"
0x00000144     12  800A4554303030372E767476                             LEN8_STRING_CP932         length=10, text="ET0007.vtv"
0x00000150     12  800A4554303030382E767476                             LEN8_STRING_CP932         length=10, text="ET0008.vtv"
0x0000015C     13  800B414247303131412E767476                           LEN8_STRING_CP932         length=11, text="ABG011A.vtv"
0x00000169     13  800B414247303132412E767476                           LEN8_STRING_CP932         length=11, text="ABG012A.vtv"
0x00000176     13  800B544247303031412E767476                           LEN8_STRING_CP932         length=11, text="TBG001A.vtv"
0x00000183     13  800B544247303033412E767476                           LEN8_STRING_CP932         length=11, text="TBG003A.vtv"
0x00000190     13  800B544247303433412E767476                           LEN8_STRING_CP932         length=11, text="TBG043A.vtv"
0x0000019D      2  0040                                                 WORD_00XX                 u16_be=64, low_byte=64
0x0000019F      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x000001A1      2  004A                                                 WORD_00XX                 u16_be=74, low_byte=74
0x000001A3      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x000001A5      2  0044                                                 WORD_00XX                 u16_be=68, low_byte=68
0x000001A7      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000001A9      1  FF                                                   TERMINATOR_FF             
0x000001AA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000001AC      1  FF                                                   TERMINATOR_FF             
0x000001AD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000001AF      1  FF                                                   TERMINATOR_FF             
0x000001B0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000001B2      1  FF                                                   TERMINATOR_FF             
0x000001B3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001B5      2  002C                                                 WORD_00XX                 u16_be=44, low_byte=44
0x000001B7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000001B9      1  FF                                                   TERMINATOR_FF             
0x000001BA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001BC      2  0020                                                 WORD_00XX                 u16_be=32, low_byte=32
0x000001BE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000001C0      1  FF                                                   TERMINATOR_FF             
0x000001C1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001C3      2  002F                                                 WORD_00XX                 u16_be=47, low_byte=47
0x000001C5      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000001C7      1  FF                                                   TERMINATOR_FF             
0x000001C8      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x000001CA      3  F10020                                               IMM16_F1                  u16_be=32, u16_le=8192
0x000001CD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000001CF      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x000001D0      1  FF                                                   TERMINATOR_FF             
0x000001D1      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000001D3      1  DC                                                   OPAQUE_RAW_BYTES          bytes=DC
0x000001D4      2  0021                                                 WORD_00XX                 u16_be=33, low_byte=33
0x000001D6      2  0022                                                 WORD_00XX                 u16_be=34, low_byte=34
0x000001D8      2  0030                                                 WORD_00XX                 u16_be=48, low_byte=48
0x000001DA      2  0023                                                 WORD_00XX                 u16_be=35, low_byte=35
0x000001DC      2  002E                                                 WORD_00XX                 u16_be=46, low_byte=46
0x000001DE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001E0      2  002D                                                 WORD_00XX                 u16_be=45, low_byte=45
0x000001E2      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000001E4      1  FF                                                   TERMINATOR_FF             
0x000001E5      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x000001E7      3  F1002E                                               IMM16_F1                  u16_be=46, u16_le=11776
0x000001EA      3  F30100                                               IMM16_F3                  u16_be=256, u16_le=1
0x000001ED      1  41                                                   OPAQUE_RAW_BYTES          bytes=41
0x000001EE      1  FF                                                   TERMINATOR_FF             
0x000001EF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001F1      1  F9                                                   OPAQUE_RAW_BYTES          bytes=F9
0x000001F2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001F4      2  002D                                                 WORD_00XX                 u16_be=45, low_byte=45
0x000001F6      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000001F8      1  FF                                                   TERMINATOR_FF             
0x000001F9      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x000001FB      3  F10030                                               IMM16_F1                  u16_be=48, u16_le=12288
0x000001FE      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000200      1  41                                                   OPAQUE_RAW_BYTES          bytes=41
0x00000201      1  FF                                                   TERMINATOR_FF             
0x00000202      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000204      1  1F                                                   OPAQUE_RAW_BYTES          bytes=1F
0x00000205      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000207      2  0028                                                 WORD_00XX                 u16_be=40, low_byte=40
0x00000209      3  F10028                                               IMM16_F1                  u16_be=40, u16_le=10240
0x0000020C      3  F1002D                                               IMM16_F1                  u16_be=45, u16_le=11520
0x0000020F      1  12                                                   OPAQUE_RAW_BYTES          bytes=12
0x00000210      1  FF                                                   TERMINATOR_FF             
0x00000211      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000213      2  002F                                                 WORD_00XX                 u16_be=47, low_byte=47
0x00000215      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000217      1  FF                                                   TERMINATOR_FF             
0x00000218      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000021A      2  002C                                                 WORD_00XX                 u16_be=44, low_byte=44
0x0000021C      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000021E      1  FF                                                   TERMINATOR_FF             
0x0000021F      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00000221      3  F10030                                               IMM16_F1                  u16_be=48, u16_le=12288
0x00000224      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000226      1  41                                                   OPAQUE_RAW_BYTES          bytes=41
0x00000227      1  FF                                                   TERMINATOR_FF             
0x00000228      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000022A      1  45                                                   OPAQUE_RAW_BYTES          bytes=45
0x0000022B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000022D      2  0028                                                 WORD_00XX                 u16_be=40, low_byte=40
0x0000022F      3  F10028                                               IMM16_F1                  u16_be=40, u16_le=10240
0x00000232      3  F1002D                                               IMM16_F1                  u16_be=45, u16_le=11520
0x00000235      1  11                                                   OPAQUE_RAW_BYTES          bytes=11
0x00000236      1  FF                                                   TERMINATOR_FF             
0x00000237      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000239      2  002F                                                 WORD_00XX                 u16_be=47, low_byte=47
0x0000023B      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000023D      1  FF                                                   TERMINATOR_FF             
0x0000023E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000240      2  002C                                                 WORD_00XX                 u16_be=44, low_byte=44
0x00000242      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000244      1  FF                                                   TERMINATOR_FF             
0x00000245      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00000247      3  F10030                                               IMM16_F1                  u16_be=48, u16_le=12288
0x0000024A      2  F240                                                 IMM8_F2                   u8=64, s8=64
0x0000024C      1  41                                                   OPAQUE_RAW_BYTES          bytes=41
0x0000024D      1  FF                                                   TERMINATOR_FF             
0x0000024E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000250      1  6B                                                   OPAQUE_RAW_BYTES          bytes=6B
0x00000251      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000253      2  0029                                                 WORD_00XX                 u16_be=41, low_byte=41
0x00000255      3  F10029                                               IMM16_F1                  u16_be=41, u16_le=10496
0x00000258      3  F1002D                                               IMM16_F1                  u16_be=45, u16_le=11520
0x0000025B      1  12                                                   OPAQUE_RAW_BYTES          bytes=12
0x0000025C      1  FF                                                   TERMINATOR_FF             
0x0000025D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000025F      2  002F                                                 WORD_00XX                 u16_be=47, low_byte=47
0x00000261      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000263      1  FF                                                   TERMINATOR_FF             
0x00000264      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000266      2  002C                                                 WORD_00XX                 u16_be=44, low_byte=44
0x00000268      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000026A      1  FF                                                   TERMINATOR_FF             
0x0000026B      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x0000026D      3  F10030                                               IMM16_F1                  u16_be=48, u16_le=12288
0x00000270      2  F210                                                 IMM8_F2                   u8=16, s8=16
0x00000272      1  41                                                   OPAQUE_RAW_BYTES          bytes=41
0x00000273      1  FF                                                   TERMINATOR_FF             
0x00000274      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000276      1  91                                                   OPAQUE_RAW_BYTES          bytes=91
0x00000277      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000279      2  0029                                                 WORD_00XX                 u16_be=41, low_byte=41
0x0000027B      3  F10029                                               IMM16_F1                  u16_be=41, u16_le=10496
0x0000027E      3  F1002D                                               IMM16_F1                  u16_be=45, u16_le=11520
0x00000281      1  11                                                   OPAQUE_RAW_BYTES          bytes=11
0x00000282      1  FF                                                   TERMINATOR_FF             
0x00000283      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000285      2  002F                                                 WORD_00XX                 u16_be=47, low_byte=47
0x00000287      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000289      1  FF                                                   TERMINATOR_FF             
0x0000028A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000028C      2  002C                                                 WORD_00XX                 u16_be=44, low_byte=44
0x0000028E      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000290      1  FF                                                   TERMINATOR_FF             
0x00000291      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00000293      3  F10030                                               IMM16_F1                  u16_be=48, u16_le=12288
0x00000296      3  F30080                                               IMM16_F3                  u16_be=128, u16_le=32768
0x00000299      1  41                                                   OPAQUE_RAW_BYTES          bytes=41
0x0000029A      1  FF                                                   TERMINATOR_FF             
0x0000029B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000029D      1  B8                                                   OPAQUE_RAW_BYTES          bytes=B8
0x0000029E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002A0      2  002A                                                 WORD_00XX                 u16_be=42, low_byte=42
0x000002A2      3  F1002A                                               IMM16_F1                  u16_be=42, u16_le=10752
0x000002A5      3  F1002D                                               IMM16_F1                  u16_be=45, u16_le=11520
0x000002A8      1  12                                                   OPAQUE_RAW_BYTES          bytes=12
0x000002A9      1  FF                                                   TERMINATOR_FF             
0x000002AA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002AC      2  002F                                                 WORD_00XX                 u16_be=47, low_byte=47
0x000002AE      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000002B0      1  FF                                                   TERMINATOR_FF             
0x000002B1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002B3      2  002C                                                 WORD_00XX                 u16_be=44, low_byte=44
0x000002B5      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000002B7      1  FF                                                   TERMINATOR_FF             
0x000002B8      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x000002BA      3  F10030                                               IMM16_F1                  u16_be=48, u16_le=12288
0x000002BD      2  F220                                                 IMM8_F2                   u8=32, s8=32
0x000002BF      1  41                                                   OPAQUE_RAW_BYTES          bytes=41
0x000002C0      1  FF                                                   TERMINATOR_FF             
0x000002C1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002C3      1  DE                                                   OPAQUE_RAW_BYTES          bytes=DE
0x000002C4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002C6      2  002A                                                 WORD_00XX                 u16_be=42, low_byte=42
0x000002C8      3  F1002A                                               IMM16_F1                  u16_be=42, u16_le=10752
0x000002CB      3  F1002D                                               IMM16_F1                  u16_be=45, u16_le=11520
0x000002CE      1  11                                                   OPAQUE_RAW_BYTES          bytes=11
0x000002CF      1  FF                                                   TERMINATOR_FF             
0x000002D0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002D2      2  002F                                                 WORD_00XX                 u16_be=47, low_byte=47
0x000002D4      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000002D6      1  FF                                                   TERMINATOR_FF             
0x000002D7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002D9      2  002C                                                 WORD_00XX                 u16_be=44, low_byte=44
0x000002DB      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000002DD      1  FF                                                   TERMINATOR_FF             
0x000002DE      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x000002E0      3  F10030                                               IMM16_F1                  u16_be=48, u16_le=12288
0x000002E3      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000002E5      1  41                                                   OPAQUE_RAW_BYTES          bytes=41
0x000002E6      1  FF                                                   TERMINATOR_FF             
0x000002E7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002E9      1  FC                                                   OPAQUE_RAW_BYTES          bytes=FC
0x000002EA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002EC      2  001F                                                 WORD_00XX                 u16_be=31, low_byte=31
0x000002EE      3  F1001F                                               IMM16_F1                  u16_be=31, u16_le=7936
0x000002F1      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000002F3      1  12                                                   OPAQUE_RAW_BYTES          bytes=12
0x000002F4      1  FF                                                   TERMINATOR_FF             
0x000002F5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002F7      2  0020                                                 WORD_00XX                 u16_be=32, low_byte=32
0x000002F9      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000002FB      1  FF                                                   TERMINATOR_FF             
0x000002FC      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x000002FE      3  F10030                                               IMM16_F1                  u16_be=48, u16_le=12288
0x00000301      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000303      1  41                                                   OPAQUE_RAW_BYTES          bytes=41
0x00000304      1  FF                                                   TERMINATOR_FF             
0x00000305      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000307      1  1A                                                   OPAQUE_RAW_BYTES          bytes=1A
0x00000308      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000030A      2  001F                                                 WORD_00XX                 u16_be=31, low_byte=31
0x0000030C      3  F1001F                                               IMM16_F1                  u16_be=31, u16_le=7936
0x0000030F      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000311      1  11                                                   OPAQUE_RAW_BYTES          bytes=11
0x00000312      1  FF                                                   TERMINATOR_FF             
0x00000313      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000315      2  0020                                                 WORD_00XX                 u16_be=32, low_byte=32
0x00000317      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000319      1  FF                                                   TERMINATOR_FF             
0x0000031A      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x0000031C      3  F10030                                               IMM16_F1                  u16_be=48, u16_le=12288
0x0000031F      3  F31000                                               IMM16_F3                  u16_be=4096, u16_le=16
0x00000322      1  41                                                   OPAQUE_RAW_BYTES          bytes=41
0x00000323      1  FF                                                   TERMINATOR_FF             
0x00000324      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000326      1  2C                                                   OPAQUE_RAW_BYTES          bytes=2C
0x00000327      2  0052                                                 WORD_00XX                 u16_be=82, low_byte=82
0x00000329      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000032B      1  FF                                                   TERMINATOR_FF             
0x0000032C      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x0000032E      3  F10030                                               IMM16_F1                  u16_be=48, u16_le=12288
0x00000331      3  F32000                                               IMM16_F3                  u16_be=8192, u16_le=32
0x00000334      1  41                                                   OPAQUE_RAW_BYTES          bytes=41
0x00000335      1  FF                                                   TERMINATOR_FF             
0x00000336      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000338      1  3E                                                   OPAQUE_RAW_BYTES          bytes=3E
0x00000339      2  0052                                                 WORD_00XX                 u16_be=82, low_byte=82
0x0000033B      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000033D      1  FF                                                   TERMINATOR_FF             
0x0000033E      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00000340      3  F1002F                                               IMM16_F1                  u16_be=47, u16_le=12032
0x00000343      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000345      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00000346      1  FF                                                   TERMINATOR_FF             
0x00000347      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000349      1  C0                                                   OPAQUE_RAW_BYTES          bytes=C0
0x0000034A      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x0000034C      3  F1002E                                               IMM16_F1                  u16_be=46, u16_le=11776
0x0000034F      3  F30100                                               IMM16_F3                  u16_be=256, u16_le=1
0x00000352      1  41                                                   OPAQUE_RAW_BYTES          bytes=41
0x00000353      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000355      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00000356      1  FF                                                   TERMINATOR_FF             
0x00000357      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000359      1  6C                                                   OPAQUE_RAW_BYTES          bytes=6C
0x0000035A      2  004D                                                 WORD_00XX                 u16_be=77, low_byte=77
0x0000035C      3  F300FF                                               IMM16_F3                  u16_be=255, u16_le=65280
0x0000035F      1  FF                                                   TERMINATOR_FF             
0x00000360      3  F10028                                               IMM16_F1                  u16_be=40, u16_le=10240
0x00000363      1  FF                                                   TERMINATOR_FF             
0x00000364      3  F10029                                               IMM16_F1                  u16_be=41, u16_le=10496
0x00000367      1  FF                                                   TERMINATOR_FF             
0x00000368      3  F1002A                                               IMM16_F1                  u16_be=42, u16_le=10752
0x0000036B      1  FF                                                   TERMINATOR_FF             
0x0000036C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000036E      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x00000370      4  8002483A                                             LEN8_STRING_CP932         length=2, text="H:"
0x00000374      1  FF                                                   TERMINATOR_FF             
0x00000375      1  FF                                                   TERMINATOR_FF             
0x00000376      2  0051                                                 WORD_00XX                 u16_be=81, low_byte=81
0x00000378      2  0028                                                 WORD_00XX                 u16_be=40, low_byte=40
0x0000037A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000037C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000037E      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x00000380      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x00000382      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000384      5  800320533A                                           LEN8_STRING_CP932         length=3, text=" S:"
0x00000389      1  FF                                                   TERMINATOR_FF             
0x0000038A      1  FF                                                   TERMINATOR_FF             
0x0000038B      2  0051                                                 WORD_00XX                 u16_be=81, low_byte=81
0x0000038D      2  0029                                                 WORD_00XX                 u16_be=41, low_byte=41
0x0000038F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000391      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000393      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x00000395      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x00000397      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000399      5  800320563A                                           LEN8_STRING_CP932         length=3, text=" V:"
0x0000039E      1  FF                                                   TERMINATOR_FF             
0x0000039F      1  FF                                                   TERMINATOR_FF             
0x000003A0      2  0051                                                 WORD_00XX                 u16_be=81, low_byte=81
0x000003A2      2  002A                                                 WORD_00XX                 u16_be=42, low_byte=42
0x000003A4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003A6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000003A8      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x000003AA      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x000003AC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003AE      1  FF                                                   TERMINATOR_FF             
0x000003AF      1  FF                                                   TERMINATOR_FF             
0x000003B0      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x000003B2      3  F1002C                                               IMM16_F1                  u16_be=44, u16_le=11264
0x000003B5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000003B7      1  36                                                   OPAQUE_RAW_BYTES          bytes=36
0x000003B8      1  FF                                                   TERMINATOR_FF             
0x000003B9      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000003BB      1  C0                                                   OPAQUE_RAW_BYTES          bytes=C0
0x000003BC      2  004A                                                 WORD_00XX                 u16_be=74, low_byte=74
0x000003BE      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x000003C0      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x000003C2      3  F1002E                                               IMM16_F1                  u16_be=46, u16_le=11776
0x000003C5      3  F30100                                               IMM16_F3                  u16_be=256, u16_le=1
0x000003C8      1  41                                                   OPAQUE_RAW_BYTES          bytes=41
0x000003C9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000003CB      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x000003CC      1  FF                                                   TERMINATOR_FF             
0x000003CD      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000003CF      1  D7                                                   OPAQUE_RAW_BYTES          bytes=D7
0x000003D0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003D2      2  002F                                                 WORD_00XX                 u16_be=47, low_byte=47
0x000003D4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000003D6      1  FF                                                   TERMINATOR_FF             
0x000003D7      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x000003D9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003DB      1  C8                                                   OPAQUE_RAW_BYTES          bytes=C8
0x000003DC      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x000003DE      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x000003E0      1  1C                                                   OPAQUE_RAW_BYTES          bytes=1C
0x000003E1      1  FF                                                   TERMINATOR_FF             
0x000003E2      1  FF                                                   TERMINATOR_FF             
