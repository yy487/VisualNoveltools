; Full conservative disassembly for PS2F009A.BIN
; Columns: offset | size | raw bytes | mnemonic | operands/preview
; Unknown or ambiguous VM semantics are preserved as typed atoms / opaque bytes.

0x00000000      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000002      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000004      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000006      1  FF                                                   TERMINATOR_FF             
0x00000007      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000009      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000000B      2  F25B                                                 IMM8_F2                   u8=91, s8=91
0x0000000D      1  FF                                                   TERMINATOR_FF             
0x0000000E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000010      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000012      1  FF                                                   TERMINATOR_FF             
0x00000013      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000015      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000017      2  F20A                                                 IMM8_F2                   u8=10, s8=10
0x00000019      1  FF                                                   TERMINATOR_FF             
0x0000001A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000001C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000001E      2  F222                                                 IMM8_F2                   u8=34, s8=34
0x00000020      1  FF                                                   TERMINATOR_FF             
0x00000021      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000023      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000025      1  FF                                                   TERMINATOR_FF             
0x00000026      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000028      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000002A      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000002C      1  FF                                                   TERMINATOR_FF             
0x0000002D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000002F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000031      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000033      1  FF                                                   TERMINATOR_FF             
0x00000034      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000036      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000038      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000003A      1  FF                                                   TERMINATOR_FF             
0x0000003B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000003D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000003F      1  FF                                                   TERMINATOR_FF             
0x00000040      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000042      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000044      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000046      1  FF                                                   TERMINATOR_FF             
0x00000047      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000049      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000004B      3  F31713                                               IMM16_F3                  u16_be=5907, u16_le=4887
0x0000004E      1  FF                                                   TERMINATOR_FF             
0x0000004F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000051      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000053     24  80168DA193FA82CD93AA82AA92C982A282E782B582A28142     LEN8_STRING_CP932         length=22, text="今日は頭が痛いらしい。"
0x0000006B      1  FF                                                   TERMINATOR_FF             
0x0000006C      1  FF                                                   TERMINATOR_FF             
0x0000006D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000006F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000071      1  FF                                                   TERMINATOR_FF             
0x00000072      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000074      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000076      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000078      1  FF                                                   TERMINATOR_FF             
0x00000079      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000007B      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000007D      3  F31714                                               IMM16_F3                  u16_be=5908, u16_le=5143
0x00000080      1  FF                                                   TERMINATOR_FF             
0x00000081      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000083      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000085     90  805882E682ED82C182BD82C882A081425C6E8E7682C182C4...  LEN8_STRING_CP932         length=88, text="よわったなあ。\n思ってたより、こたえてるみたいだ。\nこの調子だと少し長びくかもしれない。"
0x000000DF      1  FF                                                   TERMINATOR_FF             
0x000000E0      1  FF                                                   TERMINATOR_FF             
0x000000E1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000000E3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000000E5      1  FF                                                   TERMINATOR_FF             
0x000000E6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000E8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000EA      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x000000EC      1  FF                                                   TERMINATOR_FF             
0x000000ED      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000EF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000000F1      2  F250                                                 IMM8_F2                   u8=80, s8=80
0x000000F3      1  FF                                                   TERMINATOR_FF             
0x000000F4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000000F6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000000F8      1  FF                                                   TERMINATOR_FF             
0x000000F9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000FB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000FD      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000000FF      1  FF                                                   TERMINATOR_FF             
0x00000100      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000102      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000104      2  F22D                                                 IMM8_F2                   u8=45, s8=45
0x00000106      1  FF                                                   TERMINATOR_FF             
0x00000107      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000109      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000010B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000010D      1  FF                                                   TERMINATOR_FF             
0x0000010E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000110      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000112      1  FF                                                   TERMINATOR_FF             
0x00000113      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000115      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000117      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000119      1  FF                                                   TERMINATOR_FF             
0x0000011A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000011C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000011E      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000120      1  FF                                                   TERMINATOR_FF             
0x00000121      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000123      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000125      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000127      1  FF                                                   TERMINATOR_FF             
0x00000128      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000012A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000012C      1  FF                                                   TERMINATOR_FF             
0x0000012D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000012F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000131      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000133      1  FF                                                   TERMINATOR_FF             
0x00000134      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000136      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000138      3  F31715                                               IMM16_F3                  u16_be=5909, u16_le=5399
0x0000013B      1  FF                                                   TERMINATOR_FF             
0x0000013C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000013E      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000140     10  80084B54303236353054                                 LEN8_STRING_CP932         length=8, text="KT02650T"
0x0000014A      1  FF                                                   TERMINATOR_FF             
0x0000014B      1  FF                                                   TERMINATOR_FF             
0x0000014C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000014E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000150     74  8048817582E282C182CF82E8815B814190E789C0814182A0...  LEN8_STRING_CP932         length=72, text="「やっぱりー、千佳、あやまりに行った方が\nいいかなーって思うんだけどー」"
0x0000019A      1  FF                                                   TERMINATOR_FF             
0x0000019B      1  FF                                                   TERMINATOR_FF             
0x0000019C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000019E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000001A0      1  FF                                                   TERMINATOR_FF             
0x000001A1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001A3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001A5      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000001A7      1  FF                                                   TERMINATOR_FF             
0x000001A8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001AA      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000001AC      3  F31716                                               IMM16_F3                  u16_be=5910, u16_le=5655
0x000001AF      1  FF                                                   TERMINATOR_FF             
0x000001B0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001B2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001B4     38  802482C695FA89DB8CE3814196D891BA82CD94DF82B582BB...  LEN8_STRING_CP932         length=36, text="と放課後、木村は悲しそうな顔をする。"
0x000001DA      1  FF                                                   TERMINATOR_FF             
0x000001DB      1  FF                                                   TERMINATOR_FF             
0x000001DC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000001DE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000001E0      1  FF                                                   TERMINATOR_FF             
0x000001E1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001E3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001E5      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000001E7      1  FF                                                   TERMINATOR_FF             
0x000001E8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001EA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001EC      2  F27C                                                 IMM8_F2                   u8=124, s8=124
0x000001EE      1  FF                                                   TERMINATOR_FF             
0x000001EF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001F1      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000001F3      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000001F5      1  FF                                                   TERMINATOR_FF             
0x000001F6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000001F8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000001FA      1  FF                                                   TERMINATOR_FF             
0x000001FB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001FD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001FF      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000201      1  FF                                                   TERMINATOR_FF             
0x00000202      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000204      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000206      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000208      1  FF                                                   TERMINATOR_FF             
0x00000209      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000020B      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000020D      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000020F      1  FF                                                   TERMINATOR_FF             
0x00000210      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000212      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000214      1  FF                                                   TERMINATOR_FF             
0x00000215      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000217      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000219      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000021B      1  FF                                                   TERMINATOR_FF             
0x0000021C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000021E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000220      3  F31717                                               IMM16_F3                  u16_be=5911, u16_le=5911
0x00000223      1  FF                                                   TERMINATOR_FF             
0x00000224      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000226      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000228     10  80084846303234323054                                 LEN8_STRING_CP932         length=8, text="HF02420T"
0x00000232      1  FF                                                   TERMINATOR_FF             
0x00000233      1  FF                                                   TERMINATOR_FF             
0x00000234      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000236      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000238     14  800C817582BE82A982E782B38176                         LEN8_STRING_CP932         length=12, text="「だからさ」"
0x00000246      1  FF                                                   TERMINATOR_FF             
0x00000247      1  FF                                                   TERMINATOR_FF             
0x00000248      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000024A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000024C      1  FF                                                   TERMINATOR_FF             
0x0000024D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000024F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000251      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000253      1  FF                                                   TERMINATOR_FF             
0x00000254      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000256      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000258      3  F31718                                               IMM16_F3                  u16_be=5912, u16_le=6167
0x0000025B      1  FF                                                   TERMINATOR_FF             
0x0000025C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000025E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000260     26  801882C693A1945695D382AA94FB82F082B582A982DF82E9...  LEN8_STRING_CP932         length=24, text="と藤之辺が眉をしかめる。"
0x0000027A      1  FF                                                   TERMINATOR_FF             
0x0000027B      1  FF                                                   TERMINATOR_FF             
0x0000027C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000027E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000280      1  FF                                                   TERMINATOR_FF             
0x00000281      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000283      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000285      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000287      1  FF                                                   TERMINATOR_FF             
0x00000288      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000028A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000028C      2  F27A                                                 IMM8_F2                   u8=122, s8=122
0x0000028E      1  FF                                                   TERMINATOR_FF             
0x0000028F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000291      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000293      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000295      1  FF                                                   TERMINATOR_FF             
0x00000296      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000298      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000029A      1  FF                                                   TERMINATOR_FF             
0x0000029B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000029D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000029F      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000002A1      1  FF                                                   TERMINATOR_FF             
0x000002A2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002A4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002A6      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000002A8      1  FF                                                   TERMINATOR_FF             
0x000002A9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002AB      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000002AD      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000002AF      1  FF                                                   TERMINATOR_FF             
0x000002B0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000002B2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000002B4      1  FF                                                   TERMINATOR_FF             
0x000002B5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002B7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002B9      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000002BB      1  FF                                                   TERMINATOR_FF             
0x000002BC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002BE      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000002C0      3  F31719                                               IMM16_F3                  u16_be=5913, u16_le=6423
0x000002C3      1  FF                                                   TERMINATOR_FF             
0x000002C4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002C6      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000002C8     10  80084846303234333054                                 LEN8_STRING_CP932         length=8, text="HF02430T"
0x000002D2      1  FF                                                   TERMINATOR_FF             
0x000002D3      1  FF                                                   TERMINATOR_FF             
0x000002D4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002D6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002D8     42  80288175836F834A8BCA81418AC382E282A982B782CC82B3...  LEN8_STRING_CP932         length=40, text="「バカ玉、甘やかすのさ、もうやめよーよ」"
0x00000302      1  FF                                                   TERMINATOR_FF             
0x00000303      1  FF                                                   TERMINATOR_FF             
0x00000304      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000306      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000308      1  FF                                                   TERMINATOR_FF             
0x00000309      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000030B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000030D      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000030F      1  FF                                                   TERMINATOR_FF             
0x00000310      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000312      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000314      2  F22E                                                 IMM8_F2                   u8=46, s8=46
0x00000316      1  FF                                                   TERMINATOR_FF             
0x00000317      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000319      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000031B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000031D      1  FF                                                   TERMINATOR_FF             
0x0000031E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000320      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000322      1  FF                                                   TERMINATOR_FF             
0x00000323      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000325      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000327      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000329      1  FF                                                   TERMINATOR_FF             
0x0000032A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000032C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000032E      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000330      1  FF                                                   TERMINATOR_FF             
0x00000331      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000333      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000335      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000337      1  FF                                                   TERMINATOR_FF             
0x00000338      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000033A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000033C      1  FF                                                   TERMINATOR_FF             
0x0000033D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000033F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000341      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000343      1  FF                                                   TERMINATOR_FF             
0x00000344      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000346      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000348      3  F3171A                                               IMM16_F3                  u16_be=5914, u16_le=6679
0x0000034B      1  FF                                                   TERMINATOR_FF             
0x0000034C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000034E      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000350     10  80084B54303236363054                                 LEN8_STRING_CP932         length=8, text="KT02660T"
0x0000035A      1  FF                                                   TERMINATOR_FF             
0x0000035B      1  FF                                                   TERMINATOR_FF             
0x0000035C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000035E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000360     48  802E817582BE82C182C48141977982BF82E182F181428BCA...  LEN8_STRING_CP932         length=46, text="「だって、遥ちゃん。玉ちゃん、かわいそーだよ」"
0x00000390      1  FF                                                   TERMINATOR_FF             
0x00000391      1  FF                                                   TERMINATOR_FF             
0x00000392      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000394      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000396      1  FF                                                   TERMINATOR_FF             
0x00000397      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000399      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000039B      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000039D      1  FF                                                   TERMINATOR_FF             
0x0000039E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003A0      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000003A2      3  F3171B                                               IMM16_F3                  u16_be=5915, u16_le=6935
0x000003A5      1  FF                                                   TERMINATOR_FF             
0x000003A6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000003A8      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000003AA     10  80084846303234343054                                 LEN8_STRING_CP932         length=8, text="HF02440T"
0x000003B4      1  FF                                                   TERMINATOR_FF             
0x000003B5      1  FF                                                   TERMINATOR_FF             
0x000003B6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000003B8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000003BA     78  804C81758EA98BC68EA993BE82C882F182BE82C182C48142...  LEN8_STRING_CP932         length=76, text="「自業自得なんだって。噛みつくんなら、自分が痛い目にあうってわかっとけって」"
0x00000408      1  FF                                                   TERMINATOR_FF             
0x00000409      1  FF                                                   TERMINATOR_FF             
0x0000040A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000040C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000040E      1  FF                                                   TERMINATOR_FF             
0x0000040F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000411      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000413      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000415      1  FF                                                   TERMINATOR_FF             
0x00000416      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000418      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000041A      2  F22D                                                 IMM8_F2                   u8=45, s8=45
0x0000041C      1  FF                                                   TERMINATOR_FF             
0x0000041D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000041F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000421      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000423      1  FF                                                   TERMINATOR_FF             
0x00000424      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000426      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000428      1  FF                                                   TERMINATOR_FF             
0x00000429      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000042B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000042D      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000042F      1  FF                                                   TERMINATOR_FF             
0x00000430      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000432      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000434      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000436      1  FF                                                   TERMINATOR_FF             
0x00000437      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000439      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000043B      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000043D      1  FF                                                   TERMINATOR_FF             
0x0000043E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000440      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000442      1  FF                                                   TERMINATOR_FF             
0x00000443      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000445      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000447      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000449      1  FF                                                   TERMINATOR_FF             
0x0000044A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000044C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000044E      3  F3171C                                               IMM16_F3                  u16_be=5916, u16_le=7191
0x00000451      1  FF                                                   TERMINATOR_FF             
0x00000452      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000454      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000456     10  80084B54303236373154                                 LEN8_STRING_CP932         length=8, text="KT02671T"
0x00000460      1  FF                                                   TERMINATOR_FF             
0x00000461      1  FF                                                   TERMINATOR_FF             
0x00000462      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000464      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000466     88  8056817581638163914F82A982E7815B82A282A8815B82C6...  LEN8_STRING_CP932         length=86, text="「……前からーいおーと思ってたんだけどー、\n遥ちゃん、玉ちゃんにキビしすぎると思うー」"
0x000004BE      1  FF                                                   TERMINATOR_FF             
0x000004BF      1  FF                                                   TERMINATOR_FF             
0x000004C0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000004C2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000004C4      1  FF                                                   TERMINATOR_FF             
0x000004C5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004C7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004C9      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000004CB      1  FF                                                   TERMINATOR_FF             
0x000004CC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004CE      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000004D0      3  F3171D                                               IMM16_F3                  u16_be=5917, u16_le=7447
0x000004D3      1  FF                                                   TERMINATOR_FF             
0x000004D4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000004D6      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000004D8     10  80084B54303236373254                                 LEN8_STRING_CP932         length=8, text="KT02672T"
0x000004E2      1  FF                                                   TERMINATOR_FF             
0x000004E3      1  FF                                                   TERMINATOR_FF             
0x000004E4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000004E6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000004E8     74  804881758BCA82BF82E182F182CD815B88AB8B4382C882A2...  LEN8_STRING_CP932         length=72, text="「玉ちゃんはー悪気ないからーあんまりキビしくしちゃうとかわいそーだよー」"
0x00000532      1  FF                                                   TERMINATOR_FF             
0x00000533      1  FF                                                   TERMINATOR_FF             
0x00000534      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000536      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000538      1  FF                                                   TERMINATOR_FF             
0x00000539      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000053B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000053D      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000053F      1  FF                                                   TERMINATOR_FF             
0x00000540      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000542      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000544      2  F276                                                 IMM8_F2                   u8=118, s8=118
0x00000546      1  FF                                                   TERMINATOR_FF             
0x00000547      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000549      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000054B      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000054D      1  FF                                                   TERMINATOR_FF             
0x0000054E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000550      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000552      1  FF                                                   TERMINATOR_FF             
0x00000553      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000555      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000557      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000559      1  FF                                                   TERMINATOR_FF             
0x0000055A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000055C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000055E      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000560      1  FF                                                   TERMINATOR_FF             
0x00000561      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000563      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000565      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000567      1  FF                                                   TERMINATOR_FF             
0x00000568      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000056A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000056C      1  FF                                                   TERMINATOR_FF             
0x0000056D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000056F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000571      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000573      1  FF                                                   TERMINATOR_FF             
0x00000574      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000576      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000578      3  F3171E                                               IMM16_F3                  u16_be=5918, u16_le=7703
0x0000057B      1  FF                                                   TERMINATOR_FF             
0x0000057C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000057E      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000580     10  80084846303234353154                                 LEN8_STRING_CP932         length=8, text="HF02451T"
0x0000058A      1  FF                                                   TERMINATOR_FF             
0x0000058B      1  FF                                                   TERMINATOR_FF             
0x0000058C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000058E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000590    106  8068817590E789C082CD82E282B382B582A282A982E782BB...  LEN8_STRING_CP932         length=104, text="「千佳はやさしいからそーゆーけど、前だって、かわいそーって小犬ひろって、家で怒られたことあったじゃない」"
0x000005FA      1  FF                                                   TERMINATOR_FF             
0x000005FB      1  FF                                                   TERMINATOR_FF             
0x000005FC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000005FE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000600      1  FF                                                   TERMINATOR_FF             
0x00000601      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000603      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000605      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000607      1  FF                                                   TERMINATOR_FF             
0x00000608      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000060A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000060C      3  F3171F                                               IMM16_F3                  u16_be=5919, u16_le=7959
0x0000060F      1  FF                                                   TERMINATOR_FF             
0x00000610      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000612      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000614     10  80084846303234353254                                 LEN8_STRING_CP932         length=8, text="HF02452T"
0x0000061E      1  FF                                                   TERMINATOR_FF             
0x0000061F      1  FF                                                   TERMINATOR_FF             
0x00000620      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000622      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000624     70  804481758F9582AF82C482A082B082BD82A282CC82CD82ED...  LEN8_STRING_CP932         length=68, text="「助けてあげたいのはわかるけど、できることとできないことってあるよ」"
0x0000066A      1  FF                                                   TERMINATOR_FF             
0x0000066B      1  FF                                                   TERMINATOR_FF             
0x0000066C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000066E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000670      1  FF                                                   TERMINATOR_FF             
0x00000671      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000673      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000675      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000677      1  FF                                                   TERMINATOR_FF             
0x00000678      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000067A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000067C      2  F22C                                                 IMM8_F2                   u8=44, s8=44
0x0000067E      1  FF                                                   TERMINATOR_FF             
0x0000067F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000681      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000683      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000685      1  FF                                                   TERMINATOR_FF             
0x00000686      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000688      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000068A      1  FF                                                   TERMINATOR_FF             
0x0000068B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000068D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000068F      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000691      1  FF                                                   TERMINATOR_FF             
0x00000692      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000694      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000696      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000698      1  FF                                                   TERMINATOR_FF             
0x00000699      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000069B      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000069D      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000069F      1  FF                                                   TERMINATOR_FF             
0x000006A0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000006A2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000006A4      1  FF                                                   TERMINATOR_FF             
0x000006A5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006A7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006A9      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000006AB      1  FF                                                   TERMINATOR_FF             
0x000006AC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006AE      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000006B0      3  F31720                                               IMM16_F3                  u16_be=5920, u16_le=8215
0x000006B3      1  FF                                                   TERMINATOR_FF             
0x000006B4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006B6      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000006B8     10  80084B54303236383054                                 LEN8_STRING_CP932         length=8, text="KT02680T"
0x000006C2      1  FF                                                   TERMINATOR_FF             
0x000006C3      1  FF                                                   TERMINATOR_FF             
0x000006C4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006C6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006C8     46  802C817582B682E182A08141977982BF82E182F182CD82BB...  LEN8_STRING_CP932         length=44, text="「じゃあ、遥ちゃんはそーゆーのほっとける？」"
0x000006F6      1  FF                                                   TERMINATOR_FF             
0x000006F7      1  FF                                                   TERMINATOR_FF             
0x000006F8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000006FA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000006FC      1  FF                                                   TERMINATOR_FF             
0x000006FD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006FF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000701      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000703      1  FF                                                   TERMINATOR_FF             
0x00000704      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000706      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000708      3  F31721                                               IMM16_F3                  u16_be=5921, u16_le=8471
0x0000070B      1  FF                                                   TERMINATOR_FF             
0x0000070C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000070E      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000710     10  80084846303234363054                                 LEN8_STRING_CP932         length=8, text="HF02460T"
0x0000071A      1  FF                                                   TERMINATOR_FF             
0x0000071B      1  FF                                                   TERMINATOR_FF             
0x0000071C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000071E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000720     36  8022817582A282E2814182BB815B82E4815B82B182C682B6...  LEN8_STRING_CP932         length=34, text="「いや、そーゆーことじゃなくてさ」"
0x00000744      1  FF                                                   TERMINATOR_FF             
0x00000745      1  FF                                                   TERMINATOR_FF             
0x00000746      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000748      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000074A      1  FF                                                   TERMINATOR_FF             
0x0000074B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000074D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000074F      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000751      1  FF                                                   TERMINATOR_FF             
0x00000752      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000754      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000756      2  F227                                                 IMM8_F2                   u8=39, s8=39
0x00000758      1  FF                                                   TERMINATOR_FF             
0x00000759      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000075B      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000075D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000075F      1  FF                                                   TERMINATOR_FF             
0x00000760      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000762      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000764      1  FF                                                   TERMINATOR_FF             
0x00000765      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000767      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000769      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000076B      1  FF                                                   TERMINATOR_FF             
0x0000076C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000076E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000770      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000772      1  FF                                                   TERMINATOR_FF             
0x00000773      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000775      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000777      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000779      1  FF                                                   TERMINATOR_FF             
0x0000077A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000077C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000077E      1  FF                                                   TERMINATOR_FF             
0x0000077F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000781      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000783      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000785      1  FF                                                   TERMINATOR_FF             
0x00000786      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000788      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000078A      3  F31722                                               IMM16_F3                  u16_be=5922, u16_le=8727
0x0000078D      1  FF                                                   TERMINATOR_FF             
0x0000078E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000790      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000792     10  80084B54303236393054                                 LEN8_STRING_CP932         length=8, text="KT02690T"
0x0000079C      1  FF                                                   TERMINATOR_FF             
0x0000079D      1  FF                                                   TERMINATOR_FF             
0x0000079E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000007A0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000007A2     54  8034817582B682E182A08141977982BF82E182F182CD82C7...  LEN8_STRING_CP932         length=52, text="「じゃあ、遥ちゃんはどーすればいいと\n思ってるの？」"
0x000007D8      1  FF                                                   TERMINATOR_FF             
0x000007D9      1  FF                                                   TERMINATOR_FF             
0x000007DA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000007DC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000007DE      1  FF                                                   TERMINATOR_FF             
0x000007DF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007E1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007E3      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000007E5      1  FF                                                   TERMINATOR_FF             
0x000007E6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007E8      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000007EA      3  F31723                                               IMM16_F3                  u16_be=5923, u16_le=8983
0x000007ED      1  FF                                                   TERMINATOR_FF             
0x000007EE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000007F0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000007F2     44  802A82DC82B882A282C882A081428BCA82CC82B182C682C5...  LEN8_STRING_CP932         length=42, text="まずいなあ。玉のことでケンカが始まりそう。"
0x0000081E      1  FF                                                   TERMINATOR_FF             
0x0000081F      1  FF                                                   TERMINATOR_FF             
0x00000820      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000822      2  0004                                                 WORD_00XX                 u16_be=4, low_byte=4
0x00000824     10  800896D982C182C482E9                                 LEN8_STRING_CP932         length=8, text="黙ってる"
0x0000082E      1  FF                                                   TERMINATOR_FF             
0x0000082F      1  FF                                                   TERMINATOR_FF             
0x00000830      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000832      2  000A                                                 WORD_00XX                 u16_be=10, low_byte=10
0x00000834      3  F31724                                               IMM16_F3                  u16_be=5924, u16_le=9239
0x00000837      1  FF                                                   TERMINATOR_FF             
0x00000838      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000083A      2  0005                                                 WORD_00XX                 u16_be=5, low_byte=5
0x0000083C     14  800C82ED82C182C482CD82A282E9                         LEN8_STRING_CP932         length=12, text="わってはいる"
0x0000084A      1  FF                                                   TERMINATOR_FF             
0x0000084B      1  FF                                                   TERMINATOR_FF             
0x0000084C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000084E      2  000B                                                 WORD_00XX                 u16_be=11, low_byte=11
0x00000850      3  F31725                                               IMM16_F3                  u16_be=5925, u16_le=9495
0x00000853      1  FF                                                   TERMINATOR_FF             
0x00000854      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000856      2  0008                                                 WORD_00XX                 u16_be=8, low_byte=8
0x00000858      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x0000085A      1  FF                                                   TERMINATOR_FF             
0x0000085B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000085D      2  0009                                                 WORD_00XX                 u16_be=9, low_byte=9
0x0000085F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000861      1  FF                                                   TERMINATOR_FF             
0x00000862      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000864      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000866      2  F212                                                 IMM8_F2                   u8=18, s8=18
0x00000868      1  FF                                                   TERMINATOR_FF             
0x00000869      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000086B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000086D      1  FF                                                   TERMINATOR_FF             
0x0000086E      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00000870      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x00000873      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000875      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00000876      1  FF                                                   TERMINATOR_FF             
0x00000877      2  000D                                                 WORD_00XX                 u16_be=13, low_byte=13
0x00000879      1  EF                                                   OPAQUE_RAW_BYTES          bytes=EF
0x0000087A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000087C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000087E      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000880      1  FF                                                   TERMINATOR_FF             
0x00000881      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000883      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000885      3  F31726                                               IMM16_F3                  u16_be=5926, u16_le=9751
0x00000888      1  FF                                                   TERMINATOR_FF             
0x00000889      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000088B      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000088D     10  80084B54303237303054                                 LEN8_STRING_CP932         length=8, text="KT02700T"
0x00000897      1  FF                                                   TERMINATOR_FF             
0x00000898      1  FF                                                   TERMINATOR_FF             
0x00000899      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000089B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000089D     64  803E817582BE82C182C48141977982BF82E182F182BE82C1...  LEN8_STRING_CP932         length=62, text="「だって、遥ちゃんだって、玉ちゃんのこと、\n気になるでしょ？」"
0x000008DD      1  FF                                                   TERMINATOR_FF             
0x000008DE      1  FF                                                   TERMINATOR_FF             
0x000008DF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000008E1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000008E3      1  FF                                                   TERMINATOR_FF             
0x000008E4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008E6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008E8      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000008EA      1  FF                                                   TERMINATOR_FF             
0x000008EB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008ED      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000008EF      2  F277                                                 IMM8_F2                   u8=119, s8=119
0x000008F1      1  FF                                                   TERMINATOR_FF             
0x000008F2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008F4      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000008F6      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000008F8      1  FF                                                   TERMINATOR_FF             
0x000008F9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000008FB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000008FD      1  FF                                                   TERMINATOR_FF             
0x000008FE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000900      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000902      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000904      1  FF                                                   TERMINATOR_FF             
0x00000905      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000907      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000909      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000090B      1  FF                                                   TERMINATOR_FF             
0x0000090C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000090E      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000910      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000912      1  FF                                                   TERMINATOR_FF             
0x00000913      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000915      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000917      1  FF                                                   TERMINATOR_FF             
0x00000918      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000091A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000091C      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000091E      1  FF                                                   TERMINATOR_FF             
0x0000091F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000921      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000923      3  F31727                                               IMM16_F3                  u16_be=5927, u16_le=10007
0x00000926      1  FF                                                   TERMINATOR_FF             
0x00000927      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000929      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000092B     10  80084846303234373054                                 LEN8_STRING_CP932         length=8, text="HF02470T"
0x00000935      1  FF                                                   TERMINATOR_FF             
0x00000936      1  FF                                                   TERMINATOR_FF             
0x00000937      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000939      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000093B     30  801C817595CA82C9814282A082F182C882CC8AD68C5782C8...  LEN8_STRING_CP932         length=28, text="「別に。あんなの関係ないし」"
0x00000959      1  FF                                                   TERMINATOR_FF             
0x0000095A      1  FF                                                   TERMINATOR_FF             
0x0000095B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000095D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000095F      1  FF                                                   TERMINATOR_FF             
0x00000960      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000962      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000964      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000966      1  FF                                                   TERMINATOR_FF             
0x00000967      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000969      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000096B      2  F22D                                                 IMM8_F2                   u8=45, s8=45
0x0000096D      1  FF                                                   TERMINATOR_FF             
0x0000096E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000970      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000972      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000974      1  FF                                                   TERMINATOR_FF             
0x00000975      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000977      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000979      1  FF                                                   TERMINATOR_FF             
0x0000097A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000097C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000097E      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000980      1  FF                                                   TERMINATOR_FF             
0x00000981      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000983      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000985      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000987      1  FF                                                   TERMINATOR_FF             
0x00000988      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000098A      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000098C      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000098E      1  FF                                                   TERMINATOR_FF             
0x0000098F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000991      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000993      1  FF                                                   TERMINATOR_FF             
0x00000994      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000996      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000998      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000099A      1  FF                                                   TERMINATOR_FF             
0x0000099B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000099D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000099F      3  F31728                                               IMM16_F3                  u16_be=5928, u16_le=10263
0x000009A2      1  FF                                                   TERMINATOR_FF             
0x000009A3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009A5      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000009A7     10  80084B54303237313154                                 LEN8_STRING_CP932         length=8, text="KT02711T"
0x000009B1      1  FF                                                   TERMINATOR_FF             
0x000009B2      1  FF                                                   TERMINATOR_FF             
0x000009B3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009B5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009B7     36  8022817582BE82C182C481418AD68C5782C882A282B182C6...  LEN8_STRING_CP932         length=34, text="「だって、関係ないことないと思う」"
0x000009DB      1  FF                                                   TERMINATOR_FF             
0x000009DC      1  FF                                                   TERMINATOR_FF             
0x000009DD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000009DF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000009E1      1  FF                                                   TERMINATOR_FF             
0x000009E2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009E4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009E6      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000009E8      1  FF                                                   TERMINATOR_FF             
0x000009E9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009EB      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000009ED      3  F31729                                               IMM16_F3                  u16_be=5929, u16_le=10519
0x000009F0      1  FF                                                   TERMINATOR_FF             
0x000009F1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009F3      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000009F5     10  80084B54303237313254                                 LEN8_STRING_CP932         length=8, text="KT02712T"
0x000009FF      1  FF                                                   TERMINATOR_FF             
0x00000A00      1  FF                                                   TERMINATOR_FF             
0x00000A01      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A03      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A05    116  80728175977982BF82E182F182CD8BCA82BF82E182F182CC...  LEN8_STRING_CP932         length=114, text="「遥ちゃんは玉ちゃんのこと、ときどきすごく\n嫌いなんだと思うし、千佳も最初玉ちゃんのこと、すごくびっくりしたけど」"
0x00000A79      1  FF                                                   TERMINATOR_FF             
0x00000A7A      1  FF                                                   TERMINATOR_FF             
0x00000A7B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000A7D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A7F      1  FF                                                   TERMINATOR_FF             
0x00000A80      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A82      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A84      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000A86      1  FF                                                   TERMINATOR_FF             
0x00000A87      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A89      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000A8B      3  F3172A                                               IMM16_F3                  u16_be=5930, u16_le=10775
0x00000A8E      1  FF                                                   TERMINATOR_FF             
0x00000A8F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A91      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000A93     10  80084B54303237313354                                 LEN8_STRING_CP932         length=8, text="KT02713T"
0x00000A9D      1  FF                                                   TERMINATOR_FF             
0x00000A9E      1  FF                                                   TERMINATOR_FF             
0x00000A9F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000AA1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000AA3     92  805A81758BCA82BF82E182F1814182BF82E582C182C682B1...  LEN8_STRING_CP932         length=90, text="「玉ちゃん、ちょっとこどもなだけで、あんな\n自由でいられるのって、すごくうらやましいよお」"
0x00000AFF      1  FF                                                   TERMINATOR_FF             
0x00000B00      1  FF                                                   TERMINATOR_FF             
0x00000B01      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000B03      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000B05      1  FF                                                   TERMINATOR_FF             
0x00000B06      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B08      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B0A      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000B0C      1  FF                                                   TERMINATOR_FF             
0x00000B0D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B0F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000B11      3  F3172B                                               IMM16_F3                  u16_be=5931, u16_le=11031
0x00000B14      1  FF                                                   TERMINATOR_FF             
0x00000B15      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B17      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000B19     10  80084846303234383054                                 LEN8_STRING_CP932         length=8, text="HF02480T"
0x00000B23      1  FF                                                   TERMINATOR_FF             
0x00000B24      1  FF                                                   TERMINATOR_FF             
0x00000B25      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B27      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B29     14  800C817581638163816381638176                         LEN8_STRING_CP932         length=12, text="「…………」"
0x00000B37      1  FF                                                   TERMINATOR_FF             
0x00000B38      1  FF                                                   TERMINATOR_FF             
0x00000B39      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000B3B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000B3D      1  FF                                                   TERMINATOR_FF             
0x00000B3E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B40      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B42      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000B44      1  FF                                                   TERMINATOR_FF             
0x00000B45      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B47      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B49      2  F227                                                 IMM8_F2                   u8=39, s8=39
0x00000B4B      1  FF                                                   TERMINATOR_FF             
0x00000B4C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B4E      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000B50      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000B52      1  FF                                                   TERMINATOR_FF             
0x00000B53      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000B55      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000B57      1  FF                                                   TERMINATOR_FF             
0x00000B58      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B5A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B5C      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000B5E      1  FF                                                   TERMINATOR_FF             
0x00000B5F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B61      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B63      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000B65      1  FF                                                   TERMINATOR_FF             
0x00000B66      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B68      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000B6A      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000B6C      1  FF                                                   TERMINATOR_FF             
0x00000B6D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000B6F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000B71      1  FF                                                   TERMINATOR_FF             
0x00000B72      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B74      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B76      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000B78      1  FF                                                   TERMINATOR_FF             
0x00000B79      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B7B      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000B7D      3  F3172C                                               IMM16_F3                  u16_be=5932, u16_le=11287
0x00000B80      1  FF                                                   TERMINATOR_FF             
0x00000B81      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B83      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000B85     10  80084B54303237323154                                 LEN8_STRING_CP932         length=8, text="KT02721T"
0x00000B8F      1  FF                                                   TERMINATOR_FF             
0x00000B90      1  FF                                                   TERMINATOR_FF             
0x00000B91      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B93      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B95     88  8056817590E789C082CD814182B182ED82AD82C48BCA82BF...  LEN8_STRING_CP932         length=86, text="「千佳は、こわくて玉ちゃんみたいにできない\nけど、玉ちゃん見てるといーなーって思うの」"
0x00000BED      1  FF                                                   TERMINATOR_FF             
0x00000BEE      1  FF                                                   TERMINATOR_FF             
0x00000BEF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000BF1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000BF3      1  FF                                                   TERMINATOR_FF             
0x00000BF4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BF6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BF8      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000BFA      1  FF                                                   TERMINATOR_FF             
0x00000BFB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BFD      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000BFF      3  F3172D                                               IMM16_F3                  u16_be=5933, u16_le=11543
0x00000C02      1  FF                                                   TERMINATOR_FF             
0x00000C03      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C05      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000C07     10  80084B54303237323254                                 LEN8_STRING_CP932         length=8, text="KT02722T"
0x00000C11      1  FF                                                   TERMINATOR_FF             
0x00000C12      1  FF                                                   TERMINATOR_FF             
0x00000C13      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C15      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C17     70  8044817582BE82A982E7814190E789C082CD81418BCA82BF...  LEN8_STRING_CP932         length=68, text="「だから、千佳は、玉ちゃん、ずっとあのまんまでもいーなーって思うし」"
0x00000C5D      1  FF                                                   TERMINATOR_FF             
0x00000C5E      1  FF                                                   TERMINATOR_FF             
0x00000C5F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000C61      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000C63      1  FF                                                   TERMINATOR_FF             
0x00000C64      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C66      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C68      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000C6A      1  FF                                                   TERMINATOR_FF             
0x00000C6B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C6D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C6F      2  F27A                                                 IMM8_F2                   u8=122, s8=122
0x00000C71      1  FF                                                   TERMINATOR_FF             
0x00000C72      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C74      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000C76      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000C78      1  FF                                                   TERMINATOR_FF             
0x00000C79      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000C7B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000C7D      1  FF                                                   TERMINATOR_FF             
0x00000C7E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C80      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C82      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000C84      1  FF                                                   TERMINATOR_FF             
0x00000C85      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C87      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C89      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000C8B      1  FF                                                   TERMINATOR_FF             
0x00000C8C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C8E      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000C90      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000C92      1  FF                                                   TERMINATOR_FF             
0x00000C93      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000C95      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000C97      1  FF                                                   TERMINATOR_FF             
0x00000C98      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C9A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C9C      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000C9E      1  FF                                                   TERMINATOR_FF             
0x00000C9F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CA1      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000CA3      3  F3172E                                               IMM16_F3                  u16_be=5934, u16_le=11799
0x00000CA6      1  FF                                                   TERMINATOR_FF             
0x00000CA7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000CA9      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000CAB     10  80084846303234393054                                 LEN8_STRING_CP932         length=8, text="HF02490T"
0x00000CB5      1  FF                                                   TERMINATOR_FF             
0x00000CB6      1  FF                                                   TERMINATOR_FF             
0x00000CB7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000CB9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000CBB     62  803C817590E789C082CC8B438E9D82BF82CD82ED82A982E9...  LEN8_STRING_CP932         length=60, text="「千佳の気持ちはわかるけど……そーゆーわけ\nにはいかないよ」"
0x00000CF9      1  FF                                                   TERMINATOR_FF             
0x00000CFA      1  FF                                                   TERMINATOR_FF             
0x00000CFB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000CFD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000CFF      1  FF                                                   TERMINATOR_FF             
0x00000D00      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D02      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D04      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000D06      1  FF                                                   TERMINATOR_FF             
0x00000D07      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D09      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D0B      2  F22E                                                 IMM8_F2                   u8=46, s8=46
0x00000D0D      1  FF                                                   TERMINATOR_FF             
0x00000D0E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D10      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000D12      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000D14      1  FF                                                   TERMINATOR_FF             
0x00000D15      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000D17      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000D19      1  FF                                                   TERMINATOR_FF             
0x00000D1A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D1C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D1E      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000D20      1  FF                                                   TERMINATOR_FF             
0x00000D21      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D23      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D25      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000D27      1  FF                                                   TERMINATOR_FF             
0x00000D28      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D2A      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000D2C      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000D2E      1  FF                                                   TERMINATOR_FF             
0x00000D2F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000D31      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000D33      1  FF                                                   TERMINATOR_FF             
0x00000D34      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D36      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D38      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000D3A      1  FF                                                   TERMINATOR_FF             
0x00000D3B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D3D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000D3F      3  F3172F                                               IMM16_F3                  u16_be=5935, u16_le=12055
0x00000D42      1  FF                                                   TERMINATOR_FF             
0x00000D43      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D45      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000D47     10  80084B54303237333054                                 LEN8_STRING_CP932         length=8, text="KT02730T"
0x00000D51      1  FF                                                   TERMINATOR_FF             
0x00000D52      1  FF                                                   TERMINATOR_FF             
0x00000D53      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D55      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D57     34  8020817582A4814182A4816082F18163816382BB815B82C8...  LEN8_STRING_CP932         length=32, text="「う、う～ん……そーなのかなあ」"
0x00000D79      1  FF                                                   TERMINATOR_FF             
0x00000D7A      1  FF                                                   TERMINATOR_FF             
0x00000D7B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000D7D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000D7F      1  FF                                                   TERMINATOR_FF             
0x00000D80      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D82      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D84      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000D86      1  FF                                                   TERMINATOR_FF             
0x00000D87      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D89      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000D8B      3  F31730                                               IMM16_F3                  u16_be=5936, u16_le=12311
0x00000D8E      1  FF                                                   TERMINATOR_FF             
0x00000D8F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D91      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D93     80  804E82D382BD82E882CD837D8357838182C98BCA82CC82B1...  LEN8_STRING_CP932         length=78, text="ふたりはマジメに玉のことを考えていた。\nじゃあ……あいつとずっと一緒のぼくは？"
0x00000DE3      1  FF                                                   TERMINATOR_FF             
0x00000DE4      1  FF                                                   TERMINATOR_FF             
0x00000DE5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000DE7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000DE9      1  FF                                                   TERMINATOR_FF             
0x00000DEA      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00000DEC      2  0011                                                 WORD_00XX                 u16_be=17, low_byte=17
0x00000DEE      1  67                                                   OPAQUE_RAW_BYTES          bytes=67
0x00000DEF      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00000DF1      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x00000DF4      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000DF6      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00000DF7      1  FF                                                   TERMINATOR_FF             
0x00000DF8      2  0011                                                 WORD_00XX                 u16_be=17, low_byte=17
0x00000DFA      1  67                                                   OPAQUE_RAW_BYTES          bytes=67
0x00000DFB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000DFD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000DFF      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000E01      1  FF                                                   TERMINATOR_FF             
0x00000E02      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E04      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000E06      3  F31731                                               IMM16_F3                  u16_be=5937, u16_le=12567
0x00000E09      1  FF                                                   TERMINATOR_FF             
0x00000E0A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E0C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E0E     12  800A817582A082CC82B38176                             LEN8_STRING_CP932         length=10, text="「あのさ」"
0x00000E1A      1  FF                                                   TERMINATOR_FF             
0x00000E1B      1  FF                                                   TERMINATOR_FF             
0x00000E1C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000E1E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000E20      1  FF                                                   TERMINATOR_FF             
0x00000E21      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E23      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E25      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000E27      1  FF                                                   TERMINATOR_FF             
0x00000E28      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E2A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E2C      2  F278                                                 IMM8_F2                   u8=120, s8=120
0x00000E2E      1  FF                                                   TERMINATOR_FF             
0x00000E2F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E31      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000E33      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000E35      1  FF                                                   TERMINATOR_FF             
0x00000E36      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000E38      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000E3A      1  FF                                                   TERMINATOR_FF             
0x00000E3B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E3D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E3F      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000E41      1  FF                                                   TERMINATOR_FF             
0x00000E42      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E44      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E46      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000E48      1  FF                                                   TERMINATOR_FF             
0x00000E49      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E4B      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000E4D      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000E4F      1  FF                                                   TERMINATOR_FF             
0x00000E50      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000E52      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000E54      1  FF                                                   TERMINATOR_FF             
0x00000E55      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E57      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E59      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000E5B      1  FF                                                   TERMINATOR_FF             
0x00000E5C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E5E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000E60      3  F31732                                               IMM16_F3                  u16_be=5938, u16_le=12823
0x00000E63      1  FF                                                   TERMINATOR_FF             
0x00000E64      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E66      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000E68     10  80084846303235303054                                 LEN8_STRING_CP932         length=8, text="HF02500T"
0x00000E72      1  FF                                                   TERMINATOR_FF             
0x00000E73      1  FF                                                   TERMINATOR_FF             
0x00000E74      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E76      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E78     12  800A817582A482F181488176                             LEN8_STRING_CP932         length=10, text="「うん？」"
0x00000E84      1  FF                                                   TERMINATOR_FF             
0x00000E85      1  FF                                                   TERMINATOR_FF             
0x00000E86      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000E88      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000E8A      1  FF                                                   TERMINATOR_FF             
0x00000E8B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E8D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E8F      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000E91      1  FF                                                   TERMINATOR_FF             
0x00000E92      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E94      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000E96      3  F31733                                               IMM16_F3                  u16_be=5939, u16_le=13079
0x00000E99      1  FF                                                   TERMINATOR_FF             
0x00000E9A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E9C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E9E     84  8052817582A082A282C2814182A082F182C882BE82AF82C7...  LEN8_STRING_CP932         length=82, text="「あいつ、あんなだけど……悪意はないんだ。\nそれだけはわかってやってくれないか？」"
0x00000EF2      1  FF                                                   TERMINATOR_FF             
0x00000EF3      1  FF                                                   TERMINATOR_FF             
0x00000EF4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000EF6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000EF8      1  FF                                                   TERMINATOR_FF             
0x00000EF9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000EFB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000EFD      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000EFF      1  FF                                                   TERMINATOR_FF             
0x00000F00      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F02      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F04      2  F279                                                 IMM8_F2                   u8=121, s8=121
0x00000F06      1  FF                                                   TERMINATOR_FF             
0x00000F07      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F09      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000F0B      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000F0D      1  FF                                                   TERMINATOR_FF             
0x00000F0E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000F10      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000F12      1  FF                                                   TERMINATOR_FF             
0x00000F13      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F15      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F17      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000F19      1  FF                                                   TERMINATOR_FF             
0x00000F1A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F1C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F1E      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000F20      1  FF                                                   TERMINATOR_FF             
0x00000F21      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F23      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000F25      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000F27      1  FF                                                   TERMINATOR_FF             
0x00000F28      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000F2A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000F2C      1  FF                                                   TERMINATOR_FF             
0x00000F2D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F2F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F31      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000F33      1  FF                                                   TERMINATOR_FF             
0x00000F34      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F36      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000F38      3  F31734                                               IMM16_F3                  u16_be=5940, u16_le=13335
0x00000F3B      1  FF                                                   TERMINATOR_FF             
0x00000F3C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F3E      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000F40     10  80084846303235313054                                 LEN8_STRING_CP932         length=8, text="HF02510T"
0x00000F4A      1  FF                                                   TERMINATOR_FF             
0x00000F4B      1  FF                                                   TERMINATOR_FF             
0x00000F4C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F4E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F50     38  8024817582A082A0814182BB82EA82CD82CB814182ED82A9...  LEN8_STRING_CP932         length=36, text="「ああ、それはね、わかってるけどね」"
0x00000F76      1  FF                                                   TERMINATOR_FF             
0x00000F77      1  FF                                                   TERMINATOR_FF             
0x00000F78      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000F7A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000F7C      1  FF                                                   TERMINATOR_FF             
0x00000F7D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F7F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F81      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000F83      1  FF                                                   TERMINATOR_FF             
0x00000F84      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F86      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F88      2  F22D                                                 IMM8_F2                   u8=45, s8=45
0x00000F8A      1  FF                                                   TERMINATOR_FF             
0x00000F8B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F8D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000F8F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000F91      1  FF                                                   TERMINATOR_FF             
0x00000F92      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000F94      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000F96      1  FF                                                   TERMINATOR_FF             
0x00000F97      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F99      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F9B      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000F9D      1  FF                                                   TERMINATOR_FF             
0x00000F9E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FA0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000FA2      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000FA4      1  FF                                                   TERMINATOR_FF             
0x00000FA5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FA7      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000FA9      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000FAB      1  FF                                                   TERMINATOR_FF             
0x00000FAC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000FAE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000FB0      1  FF                                                   TERMINATOR_FF             
0x00000FB1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FB3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FB5      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000FB7      1  FF                                                   TERMINATOR_FF             
0x00000FB8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FBA      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000FBC      3  F31735                                               IMM16_F3                  u16_be=5941, u16_le=13591
0x00000FBF      1  FF                                                   TERMINATOR_FF             
0x00000FC0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000FC2      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000FC4     10  80084B54303237343054                                 LEN8_STRING_CP932         length=8, text="KT02740T"
0x00000FCE      1  FF                                                   TERMINATOR_FF             
0x00000FCF      1  FF                                                   TERMINATOR_FF             
0x00000FD0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000FD2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000FD4     52  8032817582A482F1816381638BCA82BF82E182F181418DA1...  LEN8_STRING_CP932         length=50, text="「うん……玉ちゃん、今ごろ、どーしてるの\nかなあ」"
0x00001008      1  FF                                                   TERMINATOR_FF             
0x00001009      1  FF                                                   TERMINATOR_FF             
0x0000100A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000100C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000100E      1  FF                                                   TERMINATOR_FF             
0x0000100F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001011      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001013      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00001015      1  FF                                                   TERMINATOR_FF             
0x00001016      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001018      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000101A      2  F277                                                 IMM8_F2                   u8=119, s8=119
0x0000101C      1  FF                                                   TERMINATOR_FF             
0x0000101D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000101F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001021      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001023      1  FF                                                   TERMINATOR_FF             
0x00001024      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001026      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001028      1  FF                                                   TERMINATOR_FF             
0x00001029      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000102B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000102D      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000102F      1  FF                                                   TERMINATOR_FF             
0x00001030      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001032      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001034      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001036      1  FF                                                   TERMINATOR_FF             
0x00001037      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001039      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000103B      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000103D      1  FF                                                   TERMINATOR_FF             
0x0000103E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001040      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001042      1  FF                                                   TERMINATOR_FF             
0x00001043      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001045      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001047      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001049      1  FF                                                   TERMINATOR_FF             
0x0000104A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000104C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000104E      3  F31736                                               IMM16_F3                  u16_be=5942, u16_le=13847
0x00001051      1  FF                                                   TERMINATOR_FF             
0x00001052      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001054      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001056     10  80084846303235323054                                 LEN8_STRING_CP932         length=8, text="HF02520T"
0x00001060      1  FF                                                   TERMINATOR_FF             
0x00001061      1  FF                                                   TERMINATOR_FF             
0x00001062      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001064      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001066     38  80248175837D8393834B82C582E093C782F182C582F182B6...  LEN8_STRING_CP932         length=36, text="「マンガでも読んでんじゃないのか？」"
0x0000108C      1  FF                                                   TERMINATOR_FF             
0x0000108D      1  FF                                                   TERMINATOR_FF             
0x0000108E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001090      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001092      1  FF                                                   TERMINATOR_FF             
0x00001093      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001095      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001097      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001099      1  FF                                                   TERMINATOR_FF             
0x0000109A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000109C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000109E      3  F31737                                               IMM16_F3                  u16_be=5943, u16_le=14103
0x000010A1      1  FF                                                   TERMINATOR_FF             
0x000010A2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000010A4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000010A6     96  805E817582C7815B82A982C88163816382A082A282C28141...  LEN8_STRING_CP932         length=94, text="「どーかな……あいつ、みんなが自分のこと好きじゃないと、ガマンできなくなっちゃうヤツ\nだから」"
0x00001106      1  FF                                                   TERMINATOR_FF             
0x00001107      1  FF                                                   TERMINATOR_FF             
0x00001108      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000110A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000110C      1  FF                                                   TERMINATOR_FF             
0x0000110D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000110F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001111      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001113      1  FF                                                   TERMINATOR_FF             
0x00001114      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001116      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001118      3  F31738                                               IMM16_F3                  u16_be=5944, u16_le=14359
0x0000111B      1  FF                                                   TERMINATOR_FF             
0x0000111C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000111E      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001120     10  80084B54303237353054                                 LEN8_STRING_CP932         length=8, text="KT02750T"
0x0000112A      1  FF                                                   TERMINATOR_FF             
0x0000112B      1  FF                                                   TERMINATOR_FF             
0x0000112C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000112E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001130     48  802E817582A482F1814182BE82A982E7814190E789C08141...  LEN8_STRING_CP932         length=46, text="「うん、だから、千佳、悪いことしちゃった……」"
0x00001160      1  FF                                                   TERMINATOR_FF             
0x00001161      1  FF                                                   TERMINATOR_FF             
0x00001162      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001164      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001166      1  FF                                                   TERMINATOR_FF             
0x00001167      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001169      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000116B      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000116D      1  FF                                                   TERMINATOR_FF             
0x0000116E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001170      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001172      3  F31739                                               IMM16_F3                  u16_be=5945, u16_le=14615
0x00001175      1  FF                                                   TERMINATOR_FF             
0x00001176      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001178      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000117A     74  804882B182EA82CD82E082A4814182B582A982BD82C882A2...  LEN8_STRING_CP932         length=72, text="これはもう、しかたないね。\n木村たちにこれ以上のめんどうはかけられない。"
0x000011C4      1  FF                                                   TERMINATOR_FF             
0x000011C5      1  FF                                                   TERMINATOR_FF             
0x000011C6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000011C8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000011CA      1  FF                                                   TERMINATOR_FF             
0x000011CB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000011CD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000011CF      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000011D1      1  FF                                                   TERMINATOR_FF             
0x000011D2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000011D4      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000011D6      3  F3173A                                               IMM16_F3                  u16_be=5946, u16_le=14871
0x000011D9      1  FF                                                   TERMINATOR_FF             
0x000011DA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000011DC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000011DE     24  801681758B4182E88141976C8E718CA982C482AD82E98176     LEN8_STRING_CP932         length=22, text="「帰り、様子見てくる」"
0x000011F6      1  FF                                                   TERMINATOR_FF             
0x000011F7      1  FF                                                   TERMINATOR_FF             
0x000011F8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000011FA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000011FC      1  FF                                                   TERMINATOR_FF             
0x000011FD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000011FF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001201      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00001203      1  FF                                                   TERMINATOR_FF             
0x00001204      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001206      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001208      2  F229                                                 IMM8_F2                   u8=41, s8=41
0x0000120A      1  FF                                                   TERMINATOR_FF             
0x0000120B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000120D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000120F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001211      1  FF                                                   TERMINATOR_FF             
0x00001212      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001214      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001216      1  FF                                                   TERMINATOR_FF             
0x00001217      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001219      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000121B      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000121D      1  FF                                                   TERMINATOR_FF             
0x0000121E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001220      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001222      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001224      1  FF                                                   TERMINATOR_FF             
0x00001225      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001227      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001229      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000122B      1  FF                                                   TERMINATOR_FF             
0x0000122C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000122E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001230      1  FF                                                   TERMINATOR_FF             
0x00001231      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001233      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001235      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001237      1  FF                                                   TERMINATOR_FF             
0x00001238      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000123A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000123C      3  F3173B                                               IMM16_F3                  u16_be=5947, u16_le=15127
0x0000123F      1  FF                                                   TERMINATOR_FF             
0x00001240      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001242      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001244     10  80084B54303237363054                                 LEN8_STRING_CP932         length=8, text="KT02760T"
0x0000124E      1  FF                                                   TERMINATOR_FF             
0x0000124F      1  FF                                                   TERMINATOR_FF             
0x00001250      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001252      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001254     46  802C817582A482F1816381638BCA82BF82E182F182C98141...  LEN8_STRING_CP932         length=44, text="「うん……玉ちゃんに、よろしくいっておいて」"
0x00001282      1  FF                                                   TERMINATOR_FF             
0x00001283      1  FF                                                   TERMINATOR_FF             
0x00001284      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001286      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001288      1  FF                                                   TERMINATOR_FF             
0x00001289      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000128B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000128D      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x0000128F      1  FF                                                   TERMINATOR_FF             
0x00001290      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001292      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001294      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001296      1  FF                                                   TERMINATOR_FF             
0x00001297      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001299      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000129B      1  FF                                                   TERMINATOR_FF             
0x0000129C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000129E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012A0      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x000012A2      1  FF                                                   TERMINATOR_FF             
0x000012A3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012A5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000012A7      2  F2FF                                                 IMM8_F2                   u8=255, s8=-1
0x000012A9      1  FF                                                   TERMINATOR_FF             
0x000012AA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000012AC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000012AE      1  FF                                                   TERMINATOR_FF             
0x000012AF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012B1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012B3      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000012B5      1  FF                                                   TERMINATOR_FF             
0x000012B6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012B8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000012BA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000012BC      1  FF                                                   TERMINATOR_FF             
0x000012BD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012BF      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000012C1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000012C3      1  FF                                                   TERMINATOR_FF             
0x000012C4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000012C6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000012C8      1  FF                                                   TERMINATOR_FF             
0x000012C9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012CB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012CD      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000012CF      1  FF                                                   TERMINATOR_FF             
0x000012D0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012D2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000012D4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000012D6      1  FF                                                   TERMINATOR_FF             
0x000012D7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012D9      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000012DB      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000012DD      1  FF                                                   TERMINATOR_FF             
0x000012DE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000012E0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000012E2      1  FF                                                   TERMINATOR_FF             
0x000012E3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012E5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012E7      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000012E9      1  FF                                                   TERMINATOR_FF             
0x000012EA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012EC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000012EE      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000012F0      1  FF                                                   TERMINATOR_FF             
0x000012F1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012F3      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000012F5      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000012F7      1  FF                                                   TERMINATOR_FF             
0x000012F8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000012FA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000012FC      1  FF                                                   TERMINATOR_FF             
0x000012FD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012FF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001301      2  F20B                                                 IMM8_F2                   u8=11, s8=11
0x00001303      1  FF                                                   TERMINATOR_FF             
0x00001304      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001306      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001308      1  FF                                                   TERMINATOR_FF             
0x00001309      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000130B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000130D      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x0000130F      1  FF                                                   TERMINATOR_FF             
0x00001310      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001312      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x00001314     14  800C50533246303130612E62696E                         LEN8_STRING_CP932         length=12, text="PS2F010a.bin"
0x00001322      1  FF                                                   TERMINATOR_FF             
0x00001323      1  FF                                                   TERMINATOR_FF             
0x00001324      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001326      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001328      1  FF                                                   TERMINATOR_FF             
0x00001329      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x0000132B      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000132D      1  FF                                                   TERMINATOR_FF             
0x0000132E      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00001330      1  38                                                   OPAQUE_RAW_BYTES          bytes=38
0x00001331      2  0021                                                 WORD_00XX                 u16_be=33, low_byte=33
0x00001333      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00001335      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00001337      1  29                                                   OPAQUE_RAW_BYTES          bytes=29
0x00001338      1  FF                                                   TERMINATOR_FF             
0x00001339      1  FF                                                   TERMINATOR_FF             
