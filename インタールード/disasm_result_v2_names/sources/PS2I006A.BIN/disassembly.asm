; Full conservative disassembly for PS2I006A.BIN
; Columns: offset | size | raw bytes | mnemonic | operands/preview
; Unknown or ambiguous VM semantics are preserved as typed atoms / opaque bytes.

0x00000000      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000002      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000004      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000006      1  FF                                                   TERMINATOR_FF             
0x00000007      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000009      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000000B      2  F259                                                 IMM8_F2                   u8=89, s8=89
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
0x0000001E      2  F215                                                 IMM8_F2                   u8=21, s8=21
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
0x0000004B      3  F31BD5                                               IMM16_F3                  u16_be=7125, u16_le=54555
0x0000004E      1  FF                                                   TERMINATOR_FF             
0x0000004F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000051      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000053     42  802882B582DC82C182BD81418C7D82A682C9978882A282C1...  LEN8_STRING_CP932         length=40, text="しまった、迎えに来いっていわれてたのに。"
0x0000007D      1  FF                                                   TERMINATOR_FF             
0x0000007E      1  FF                                                   TERMINATOR_FF             
0x0000007F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000081      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000083      1  FF                                                   TERMINATOR_FF             
0x00000084      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000086      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000088      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000008A      1  FF                                                   TERMINATOR_FF             
0x0000008B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000008D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000008F      3  F31BD6                                               IMM16_F3                  u16_be=7126, u16_le=54811
0x00000092      1  FF                                                   TERMINATOR_FF             
0x00000093      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000095      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000097    108  806A82C882F182BE82A9975D8C7682C882B182C682F08D6C...  LEN8_STRING_CP932         length=106, text="なんだか余計なことを考えてブラブラしてたら\n……ずいぶん遅刻してしまった。\nさすがに、玉はもう帰っていた。"
0x00000103      1  FF                                                   TERMINATOR_FF             
0x00000104      1  FF                                                   TERMINATOR_FF             
0x00000105      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000107      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000109      1  FF                                                   TERMINATOR_FF             
0x0000010A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000010C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000010E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000110      1  FF                                                   TERMINATOR_FF             
0x00000111      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000113      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000115      3  F31BD7                                               IMM16_F3                  u16_be=7127, u16_le=55067
0x00000118      1  FF                                                   TERMINATOR_FF             
0x00000119      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000011B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000011D     90  805882A482ED8160814182C782A482B582E682A481425C6E...  LEN8_STRING_CP932         length=88, text="うわ～、どうしよう。\nすっごい怒ってるんだろーな。\nあいつが怒るとメンドーくさいんだよ。"
0x00000177      1  FF                                                   TERMINATOR_FF             
0x00000178      1  FF                                                   TERMINATOR_FF             
0x00000179      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000017B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000017D      1  FF                                                   TERMINATOR_FF             
0x0000017E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000180      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000182      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000184      1  FF                                                   TERMINATOR_FF             
0x00000185      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000187      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000189      3  F31BD8                                               IMM16_F3                  u16_be=7128, u16_le=55323
0x0000018C      1  FF                                                   TERMINATOR_FF             
0x0000018D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000018F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000191     92  805A8EA995AA82CD92788D8F82CC8FED8F4B968282CC82AD...  LEN8_STRING_CP932         length=90, text="自分は遅刻の常習魔のくせに、人の遅刻には\n厳しいから。\nあ～……泣くほどグズるぞ、きっと。"
0x000001ED      1  FF                                                   TERMINATOR_FF             
0x000001EE      1  FF                                                   TERMINATOR_FF             
0x000001EF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000001F1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000001F3      1  FF                                                   TERMINATOR_FF             
0x000001F4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001F6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001F8      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000001FA      1  FF                                                   TERMINATOR_FF             
0x000001FB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001FD      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000001FF      3  F31BD9                                               IMM16_F3                  u16_be=7129, u16_le=55579
0x00000202      1  FF                                                   TERMINATOR_FF             
0x00000203      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000205      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000207     66  804082D382A48163816381425C6E96BE93FA82CC92A982CC...  LEN8_STRING_CP932         length=64, text="ふう……。\n明日の朝のことを考えると、気持ちが暗くなって\nいく。"
0x00000249      1  FF                                                   TERMINATOR_FF             
0x0000024A      1  FF                                                   TERMINATOR_FF             
0x0000024B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000024D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000024F      1  FF                                                   TERMINATOR_FF             
0x00000250      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000252      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000254      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000256      1  FF                                                   TERMINATOR_FF             
0x00000257      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000259      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000025B      3  F31BDA                                               IMM16_F3                  u16_be=7130, u16_le=55835
0x0000025E      1  FF                                                   TERMINATOR_FF             
0x0000025F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000261      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000263     18  801082DC82A282C182BD82C8816381638142                 LEN8_STRING_CP932         length=16, text="まいったな……。"
0x00000275      1  FF                                                   TERMINATOR_FF             
0x00000276      1  FF                                                   TERMINATOR_FF             
0x00000277      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000279      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000027B      1  FF                                                   TERMINATOR_FF             
0x0000027C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000027E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000280      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x00000282      1  FF                                                   TERMINATOR_FF             
0x00000283      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000285      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000287      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000289      1  FF                                                   TERMINATOR_FF             
0x0000028A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000028C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000028E      1  FF                                                   TERMINATOR_FF             
0x0000028F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000291      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000293      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000295      1  FF                                                   TERMINATOR_FF             
0x00000296      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000298      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000029A      2  F2FF                                                 IMM8_F2                   u8=255, s8=-1
0x0000029C      1  FF                                                   TERMINATOR_FF             
0x0000029D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000029F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000002A1      1  FF                                                   TERMINATOR_FF             
0x000002A2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002A4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002A6      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000002A8      1  FF                                                   TERMINATOR_FF             
0x000002A9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002AB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002AD      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000002AF      1  FF                                                   TERMINATOR_FF             
0x000002B0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002B2      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000002B4      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000002B6      1  FF                                                   TERMINATOR_FF             
0x000002B7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000002B9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000002BB      1  FF                                                   TERMINATOR_FF             
0x000002BC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002BE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002C0      2  F20B                                                 IMM8_F2                   u8=11, s8=11
0x000002C2      1  FF                                                   TERMINATOR_FF             
0x000002C3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000002C5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000002C7      1  FF                                                   TERMINATOR_FF             
0x000002C8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002CA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002CC      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x000002CE      1  FF                                                   TERMINATOR_FF             
0x000002CF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002D1      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x000002D3     14  800C50533246303032612E62696E                         LEN8_STRING_CP932         length=12, text="PS2F002a.bin"
0x000002E1      1  FF                                                   TERMINATOR_FF             
0x000002E2      1  FF                                                   TERMINATOR_FF             
0x000002E3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000002E5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000002E7      1  FF                                                   TERMINATOR_FF             
0x000002E8      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x000002EA      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000002EC      1  FF                                                   TERMINATOR_FF             
0x000002ED      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002EF      1  F7                                                   OPAQUE_RAW_BYTES          bytes=F7
0x000002F0      2  0021                                                 WORD_00XX                 u16_be=33, low_byte=33
0x000002F2      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x000002F4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002F6      1  E8                                                   OPAQUE_RAW_BYTES          bytes=E8
0x000002F7      1  FF                                                   TERMINATOR_FF             
0x000002F8      1  FF                                                   TERMINATOR_FF             
