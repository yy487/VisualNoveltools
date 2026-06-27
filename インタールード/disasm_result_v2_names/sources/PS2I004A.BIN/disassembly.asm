; Full conservative disassembly for PS2I004A.BIN
; Columns: offset | size | raw bytes | mnemonic | operands/preview
; Unknown or ambiguous VM semantics are preserved as typed atoms / opaque bytes.

0x00000000      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000002      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000004      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000006      1  FF                                                   TERMINATOR_FF             
0x00000007      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000009      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000000B      2  F264                                                 IMM8_F2                   u8=100, s8=100
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
0x0000004B      3  F319C9                                               IMM16_F3                  u16_be=6601, u16_le=51481
0x0000004E      1  FF                                                   TERMINATOR_FF             
0x0000004F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000051      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000053     24  801682D082C682E882C982C882E882BD82A982C182BD8142     LEN8_STRING_CP932         length=22, text="ひとりになりたかった。"
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
0x0000007D      3  F33E9B                                               IMM16_F3                  u16_be=16027, u16_le=39742
0x00000080      1  FF                                                   TERMINATOR_FF             
0x00000081      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000083      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000085     44  802A82D082C682E882C182AB82E882C5814182E482C182AD...  LEN8_STRING_CP932         length=42, text="ひとりっきりで、ゆっくり考えてみたかった。"
0x000000B1      1  FF                                                   TERMINATOR_FF             
0x000000B2      1  FF                                                   TERMINATOR_FF             
0x000000B3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000000B5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000000B7      1  FF                                                   TERMINATOR_FF             
0x000000B8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000BA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000BC      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000000BE      1  FF                                                   TERMINATOR_FF             
0x000000BF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000C1      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000000C3      3  F319CA                                               IMM16_F3                  u16_be=6602, u16_le=51737
0x000000C6      1  FF                                                   TERMINATOR_FF             
0x000000C7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000000C9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000000CB     92  805A82A082CC92A9814196B3906C82CC897782C5817794DE...  LEN8_STRING_CP932         length=90, text="あの朝、無人の駅で『彼女』を見かけたときから、なにかが変わってしまったような気がしていた。"
0x00000127      1  FF                                                   TERMINATOR_FF             
0x00000128      1  FF                                                   TERMINATOR_FF             
0x00000129      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000012B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000012D      1  FF                                                   TERMINATOR_FF             
0x0000012E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000130      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000132      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000134      1  FF                                                   TERMINATOR_FF             
0x00000135      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000137      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000139      3  F319CB                                               IMM16_F3                  u16_be=6603, u16_le=51993
0x0000013C      1  FF                                                   TERMINATOR_FF             
0x0000013D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000013F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000141     26  801882BB82CC82C882C982A982AA82ED82A982E782C882A2...  LEN8_STRING_CP932         length=24, text="そのなにかがわからない。"
0x0000015B      1  FF                                                   TERMINATOR_FF             
0x0000015C      1  FF                                                   TERMINATOR_FF             
0x0000015D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000015F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000161      1  FF                                                   TERMINATOR_FF             
0x00000162      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000164      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000166      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000168      1  FF                                                   TERMINATOR_FF             
0x00000169      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000016B      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000016D      3  F33E9C                                               IMM16_F3                  u16_be=16028, u16_le=39998
0x00000170      1  FF                                                   TERMINATOR_FF             
0x00000171      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000173      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000175     90  80588BB982CC899C82C982D082C182A982A982C182C482A2...  LEN8_STRING_CP932         length=88, text="胸の奥にひっかかっているのに、はっきりしない。勘違いとか気のせい……ならいいのだけれど。"
0x000001CF      1  FF                                                   TERMINATOR_FF             
0x000001D0      1  FF                                                   TERMINATOR_FF             
0x000001D1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000001D3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000001D5      1  FF                                                   TERMINATOR_FF             
0x000001D6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001D8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001DA      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000001DC      1  FF                                                   TERMINATOR_FF             
0x000001DD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001DF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000001E1      3  F33E9D                                               IMM16_F3                  u16_be=16029, u16_le=40254
0x000001E4      1  FF                                                   TERMINATOR_FF             
0x000001E5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001E7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001E9    102  80648FAC82B382C88EED82CC82E682A482C882E082CC82AA...  LEN8_STRING_CP932         length=100, text="小さな種のようなものが、うえつけられて\nしまったような。\nその違和感が、ぼくの気持ちを重くしていた。"
0x0000024F      1  FF                                                   TERMINATOR_FF             
0x00000250      1  FF                                                   TERMINATOR_FF             
0x00000251      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000253      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000255      1  FF                                                   TERMINATOR_FF             
0x00000256      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00000258      3  F10034                                               IMM16_F1                  u16_be=52, u16_le=13312
0x0000025B      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000025D      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x0000025E      1  FF                                                   TERMINATOR_FF             
0x0000025F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000261      1  7C                                                   OPAQUE_RAW_BYTES          bytes=7C
0x00000262      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000264      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000266      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x00000268      1  FF                                                   TERMINATOR_FF             
0x00000269      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000026B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000026D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000026F      1  FF                                                   TERMINATOR_FF             
0x00000270      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000272      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000274      1  FF                                                   TERMINATOR_FF             
0x00000275      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000277      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000279      2  F209                                                 IMM8_F2                   u8=9, s8=9
0x0000027B      1  FF                                                   TERMINATOR_FF             
0x0000027C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000027E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000280      2  F267                                                 IMM8_F2                   u8=103, s8=103
0x00000282      1  FF                                                   TERMINATOR_FF             
0x00000283      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000285      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000287      1  FF                                                   TERMINATOR_FF             
0x00000288      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000028A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000028C      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000028E      1  FF                                                   TERMINATOR_FF             
0x0000028F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000291      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000293      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000295      1  FF                                                   TERMINATOR_FF             
0x00000296      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000298      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000029A      2  F20A                                                 IMM8_F2                   u8=10, s8=10
0x0000029C      1  FF                                                   TERMINATOR_FF             
0x0000029D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000029F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000002A1      1  FF                                                   TERMINATOR_FF             
0x000002A2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002A4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002A6      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x000002A8      1  FF                                                   TERMINATOR_FF             
0x000002A9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002AB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002AD      3  F3015F                                               IMM16_F3                  u16_be=351, u16_le=24321
0x000002B0      1  FF                                                   TERMINATOR_FF             
0x000002B1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000002B3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000002B5      1  FF                                                   TERMINATOR_FF             
0x000002B6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002B8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002BA      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000002BC      1  FF                                                   TERMINATOR_FF             
0x000002BD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002BF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002C1      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x000002C3      1  FF                                                   TERMINATOR_FF             
0x000002C4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002C6      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000002C8      2  F23C                                                 IMM8_F2                   u8=60, s8=60
0x000002CA      1  FF                                                   TERMINATOR_FF             
0x000002CB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000002CD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000002CF      1  FF                                                   TERMINATOR_FF             
0x000002D0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002D2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002D4      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000002D6      1  FF                                                   TERMINATOR_FF             
0x000002D7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002D9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000002DB      3  F319CC                                               IMM16_F3                  u16_be=6604, u16_le=52249
0x000002DE      1  FF                                                   TERMINATOR_FF             
0x000002DF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002E1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002E3     28  801A82BB82B582C4816381638DA192A982CC95CF8E8095F1...  LEN8_STRING_CP932         length=26, text="そして……今朝の変死報道。"
0x000002FF      1  FF                                                   TERMINATOR_FF             
0x00000300      1  FF                                                   TERMINATOR_FF             
0x00000301      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000303      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000305      1  FF                                                   TERMINATOR_FF             
0x00000306      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000308      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000030A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000030C      1  FF                                                   TERMINATOR_FF             
0x0000030D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000030F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000311      3  F319CD                                               IMM16_F3                  u16_be=6605, u16_le=52505
0x00000314      1  FF                                                   TERMINATOR_FF             
0x00000315      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000317      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000319     92  805A8CA982A682C882A28FEA8F8A82C58FAC82B382C88E95...  LEN8_STRING_CP932         length=90, text="見えない場所で小さな歯車がこぼれ落ち、\nかすかにきしみをあげている。\nそんな印象があった。"
0x00000375      1  FF                                                   TERMINATOR_FF             
0x00000376      1  FF                                                   TERMINATOR_FF             
0x00000377      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000379      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000037B      1  FF                                                   TERMINATOR_FF             
0x0000037C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000037E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000380      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000382      1  FF                                                   TERMINATOR_FF             
0x00000383      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000385      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000387      2  F229                                                 IMM8_F2                   u8=41, s8=41
0x00000389      1  FF                                                   TERMINATOR_FF             
0x0000038A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000038C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000038E      1  FF                                                   TERMINATOR_FF             
0x0000038F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000391      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000393      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000395      1  FF                                                   TERMINATOR_FF             
0x00000396      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000398      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000039A      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000039C      1  FF                                                   TERMINATOR_FF             
0x0000039D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000039F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000003A1      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000003A3      1  FF                                                   TERMINATOR_FF             
0x000003A4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000003A6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000003A8      1  FF                                                   TERMINATOR_FF             
0x000003A9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003AB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003AD      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000003AF      1  FF                                                   TERMINATOR_FF             
0x000003B0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003B2      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000003B4      3  F319CE                                               IMM16_F3                  u16_be=6606, u16_le=52761
0x000003B7      1  FF                                                   TERMINATOR_FF             
0x000003B8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000003BA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000003BC    108  806A975B95E982EA82AA8A5882C98BDF82C382A282C482A2...  LEN8_STRING_CP932         length=106, text="夕暮れが街に近づいていた。\n雑踏はふたつのグループにわかれていた。\n家路を急ぐ人々と、夜の街へと向かう人。"
0x00000428      1  FF                                                   TERMINATOR_FF             
0x00000429      1  FF                                                   TERMINATOR_FF             
0x0000042A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000042C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000042E      1  FF                                                   TERMINATOR_FF             
0x0000042F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000431      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000433      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000435      1  FF                                                   TERMINATOR_FF             
0x00000436      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000438      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000043A      3  F319CF                                               IMM16_F3                  u16_be=6607, u16_le=53017
0x0000043D      1  FF                                                   TERMINATOR_FF             
0x0000043E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000440      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000442     90  805882DA82AD82CD814182BB82CC82C782BF82E782C982E0...  LEN8_STRING_CP932         length=88, text="ぼくは、そのどちらにも属していない。\n自分がどこへ向かいたいのか、わからなくなっていた。"
0x0000049C      1  FF                                                   TERMINATOR_FF             
0x0000049D      1  FF                                                   TERMINATOR_FF             
0x0000049E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000004A0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000004A2      1  FF                                                   TERMINATOR_FF             
0x000004A3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004A5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004A7      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000004A9      1  FF                                                   TERMINATOR_FF             
0x000004AA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004AC      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000004AE      3  F319D0                                               IMM16_F3                  u16_be=6608, u16_le=53273
0x000004B1      1  FF                                                   TERMINATOR_FF             
0x000004B2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000004B4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000004B6     86  805482C882C982A982AA8E6E82DC82EB82A482C682B582C4...  LEN8_STRING_CP932         length=84, text="なにかが始まろうとしている予感があった。\nけれど、それが気のせいだともわかっていた。"
0x0000050C      1  FF                                                   TERMINATOR_FF             
0x0000050D      1  FF                                                   TERMINATOR_FF             
0x0000050E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000510      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000512      1  FF                                                   TERMINATOR_FF             
0x00000513      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000515      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000517      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000519      1  FF                                                   TERMINATOR_FF             
0x0000051A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000051C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000051E      3  F319D1                                               IMM16_F3                  u16_be=6609, u16_le=53529
0x00000521      1  FF                                                   TERMINATOR_FF             
0x00000522      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000524      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000526    100  806282A082CC92A9814196B3906C82CC837A815B838082C5...  LEN8_STRING_CP932         length=98, text="あの朝、無人のホームで見かけた『彼女』が\n幻であることをわかっているように。\n……そう思っていた。"
0x0000058A      1  FF                                                   TERMINATOR_FF             
0x0000058B      1  FF                                                   TERMINATOR_FF             
0x0000058C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000058E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000590      1  FF                                                   TERMINATOR_FF             
0x00000591      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000593      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000595      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000597      1  FF                                                   TERMINATOR_FF             
0x00000598      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000059A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000059C      3  F319D2                                               IMM16_F3                  u16_be=6610, u16_le=53785
0x0000059F      1  FF                                                   TERMINATOR_FF             
0x000005A0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000005A2      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000005A4      9  8007495A3030323330                                   LEN8_STRING_CP932         length=7, text="IZ00230"
0x000005AD      1  FF                                                   TERMINATOR_FF             
0x000005AE      1  FF                                                   TERMINATOR_FF             
0x000005AF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000005B1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000005B3     42  8028817582C582E082A78160816382B182DC82E982F182C5...  LEN8_STRING_CP932         length=40, text="「でもぉ～…こまるんですー、ホントにー」"
0x000005DD      1  FF                                                   TERMINATOR_FF             
0x000005DE      1  FF                                                   TERMINATOR_FF             
0x000005DF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000005E1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000005E3      1  FF                                                   TERMINATOR_FF             
0x000005E4      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x000005E6      3  F10034                                               IMM16_F1                  u16_be=52, u16_le=13312
0x000005E9      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000005EB      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x000005EC      1  FF                                                   TERMINATOR_FF             
0x000005ED      2  0035                                                 WORD_00XX                 u16_be=53, low_byte=53
0x000005EF      1  21                                                   OPAQUE_RAW_BYTES          bytes=21
0x000005F0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005F2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005F4      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x000005F6      1  FF                                                   TERMINATOR_FF             
0x000005F7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005F9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000005FB      2  F229                                                 IMM8_F2                   u8=41, s8=41
0x000005FD      1  FF                                                   TERMINATOR_FF             
0x000005FE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000600      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000602      1  FF                                                   TERMINATOR_FF             
0x00000603      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000605      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000607      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000609      1  FF                                                   TERMINATOR_FF             
0x0000060A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000060C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000060E      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000610      1  FF                                                   TERMINATOR_FF             
0x00000611      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000613      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000615      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000617      1  FF                                                   TERMINATOR_FF             
0x00000618      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000061A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000061C      1  FF                                                   TERMINATOR_FF             
0x0000061D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000061F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000621      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000623      1  FF                                                   TERMINATOR_FF             
0x00000624      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000626      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000628      3  F319D3                                               IMM16_F3                  u16_be=6611, u16_le=54041
0x0000062B      1  FF                                                   TERMINATOR_FF             
0x0000062C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000062E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000630     80  804E82DA82AD82CD96DA82F082A082B082E981425C6E96AD...  LEN8_STRING_CP932         length=78, text="ぼくは目をあげる。\n妙に聞きおぼえがあるのだけれど、誰だか\n思いあたらない声。"
0x00000680      1  FF                                                   TERMINATOR_FF             
0x00000681      1  FF                                                   TERMINATOR_FF             
0x00000682      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000684      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000686      1  FF                                                   TERMINATOR_FF             
0x00000687      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000689      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000068B      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000068D      1  FF                                                   TERMINATOR_FF             
0x0000068E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000690      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000692      3  F319D4                                               IMM16_F3                  u16_be=6612, u16_le=54297
0x00000695      1  FF                                                   TERMINATOR_FF             
0x00000696      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000698      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000069A     72  804682C282A28DC58BDF814182BB82CC90BA82F095B782A2...  LEN8_STRING_CP932         length=70, text="つい最近、その声を聞いた記憶があるけれど、\nどこでだったかわからない。"
0x000006E2      1  FF                                                   TERMINATOR_FF             
0x000006E3      1  FF                                                   TERMINATOR_FF             
0x000006E4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000006E6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000006E8      1  FF                                                   TERMINATOR_FF             
0x000006E9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006EB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006ED      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000006EF      1  FF                                                   TERMINATOR_FF             
0x000006F0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006F2      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000006F4      3  F319D5                                               IMM16_F3                  u16_be=6613, u16_le=54553
0x000006F7      1  FF                                                   TERMINATOR_FF             
0x000006F8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006FA      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000006FC      9  8007495A3030323430                                   LEN8_STRING_CP932         length=7, text="IZ00240"
0x00000705      1  FF                                                   TERMINATOR_FF             
0x00000706      1  FF                                                   TERMINATOR_FF             
0x00000707      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000709      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000070B     36  8022817582BB815B82C882F182C582B782AF82C7815B8163...  LEN8_STRING_CP932         length=34, text="「そーなんですけどー……でもお～」"
0x0000072F      1  FF                                                   TERMINATOR_FF             
0x00000730      1  FF                                                   TERMINATOR_FF             
0x00000731      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000733      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000735      1  FF                                                   TERMINATOR_FF             
0x00000736      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000738      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000073A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000073C      1  FF                                                   TERMINATOR_FF             
0x0000073D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000073F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000741      3  F319D6                                               IMM16_F3                  u16_be=6614, u16_le=54809
0x00000744      1  FF                                                   TERMINATOR_FF             
0x00000745      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000747      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000749     24  801682DA82AD82CD95D382E882F08CA989F182B582BD8142     LEN8_STRING_CP932         length=22, text="ぼくは辺りを見回した。"
0x00000761      1  FF                                                   TERMINATOR_FF             
0x00000762      1  FF                                                   TERMINATOR_FF             
0x00000763      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000765      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000767      1  FF                                                   TERMINATOR_FF             
0x00000768      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000076A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000076C      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x0000076E      1  FF                                                   TERMINATOR_FF             
0x0000076F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000771      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000773      3  F30160                                               IMM16_F3                  u16_be=352, u16_le=24577
0x00000776      1  FF                                                   TERMINATOR_FF             
0x00000777      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000779      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000077B      1  FF                                                   TERMINATOR_FF             
0x0000077C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000077E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000780      2  F20A                                                 IMM8_F2                   u8=10, s8=10
0x00000782      1  FF                                                   TERMINATOR_FF             
0x00000783      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000785      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000787      2  F218                                                 IMM8_F2                   u8=24, s8=24
0x00000789      1  FF                                                   TERMINATOR_FF             
0x0000078A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000078C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000078E      1  FF                                                   TERMINATOR_FF             
0x0000078F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000791      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000793      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000795      1  FF                                                   TERMINATOR_FF             
0x00000796      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000798      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000079A      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000079C      1  FF                                                   TERMINATOR_FF             
0x0000079D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000079F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000007A1      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000007A3      1  FF                                                   TERMINATOR_FF             
0x000007A4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000007A6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000007A8      1  FF                                                   TERMINATOR_FF             
0x000007A9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007AB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007AD      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000007AF      1  FF                                                   TERMINATOR_FF             
0x000007B0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007B2      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000007B4      3  F319D7                                               IMM16_F3                  u16_be=6615, u16_le=55065
0x000007B7      1  FF                                                   TERMINATOR_FF             
0x000007B8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000007BA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000007BC    112  806E837A83588367959782C682E4815B82A9834C83838362...  LEN8_STRING_CP932         length=110, text="ホスト風とゆーかキャッチセールス風とゆーか、そんな感じのお兄さんが、なれなれしく女性の\n肩に手をまわしている。"
0x0000082C      1  FF                                                   TERMINATOR_FF             
0x0000082D      1  FF                                                   TERMINATOR_FF             
0x0000082E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000830      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000832      1  FF                                                   TERMINATOR_FF             
0x00000833      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000835      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000837      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000839      1  FF                                                   TERMINATOR_FF             
0x0000083A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000083C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000083E      3  F319D8                                               IMM16_F3                  u16_be=6616, u16_le=55321
0x00000841      1  FF                                                   TERMINATOR_FF             
0x00000842      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000844      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000846    112  806E8F9782CC906C82CC95FB82CD814182A282E282AA82C1...  LEN8_STRING_CP932         length=110, text="女の人の方は、いやがっているのだろうけど、\n強くこばめないせいか、ふたりの間の距離が\nどんどん近くなっていく。"
0x000008B6      1  FF                                                   TERMINATOR_FF             
0x000008B7      1  FF                                                   TERMINATOR_FF             
0x000008B8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000008BA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000008BC      1  FF                                                   TERMINATOR_FF             
0x000008BD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008BF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008C1      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000008C3      1  FF                                                   TERMINATOR_FF             
0x000008C4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008C6      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000008C8      3  F319D9                                               IMM16_F3                  u16_be=6617, u16_le=55577
0x000008CB      1  FF                                                   TERMINATOR_FF             
0x000008CC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000008CE      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000008D0      9  8007495A3030323530                                   LEN8_STRING_CP932         length=7, text="IZ00250"
0x000008D9      1  FF                                                   TERMINATOR_FF             
0x000008DA      1  FF                                                   TERMINATOR_FF             
0x000008DB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000008DD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000008DF     54  8034817582A6814182BE82C182C4814182BB815B82E4815B...  LEN8_STRING_CP932         length=52, text="「え、だって、そーゆーわけじゃないんです\nけど……」"
0x00000915      1  FF                                                   TERMINATOR_FF             
0x00000916      1  FF                                                   TERMINATOR_FF             
0x00000917      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000919      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000091B      1  FF                                                   TERMINATOR_FF             
0x0000091C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000091E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000920      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000922      1  FF                                                   TERMINATOR_FF             
0x00000923      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000925      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000927      3  F319DA                                               IMM16_F3                  u16_be=6618, u16_le=55833
0x0000092A      1  FF                                                   TERMINATOR_FF             
0x0000092B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000092D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000092F     86  805482A0816082A0814182CD82C182AB82E881778C9982C5...  LEN8_STRING_CP932         length=84, text="あ～あ、はっきり『嫌です』っていえばいいのに。……って、こんなのつい最近もあったぞ。"
0x00000985      1  FF                                                   TERMINATOR_FF             
0x00000986      1  FF                                                   TERMINATOR_FF             
0x00000987      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000989      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000098B      1  FF                                                   TERMINATOR_FF             
0x0000098C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000098E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000990      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000992      1  FF                                                   TERMINATOR_FF             
0x00000993      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000995      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000997      3  F319DB                                               IMM16_F3                  u16_be=6619, u16_le=56089
0x0000099A      1  FF                                                   TERMINATOR_FF             
0x0000099B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000099D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000099F      9  8007495A3030323630                                   LEN8_STRING_CP932         length=7, text="IZ00260"
0x000009A8      1  FF                                                   TERMINATOR_FF             
0x000009A9      1  FF                                                   TERMINATOR_FF             
0x000009AA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009AC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009AE     44  802A817582A6814182C582E081418DA282E882DC82B78142...  LEN8_STRING_CP932         length=42, text="「え、でも、困ります。本当に興味ないんで」"
0x000009DA      1  FF                                                   TERMINATOR_FF             
0x000009DB      1  FF                                                   TERMINATOR_FF             
0x000009DC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000009DE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000009E0      1  FF                                                   TERMINATOR_FF             
0x000009E1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009E3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009E5      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000009E7      1  FF                                                   TERMINATOR_FF             
0x000009E8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009EA      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000009EC      3  F319DC                                               IMM16_F3                  u16_be=6620, u16_le=56345
0x000009EF      1  FF                                                   TERMINATOR_FF             
0x000009F0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009F2      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000009F4      9  800745583435303130                                   LEN8_STRING_CP932         length=7, text="EX45010"
0x000009FD      1  FF                                                   TERMINATOR_FF             
0x000009FE      1  FF                                                   TERMINATOR_FF             
0x000009FF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A01      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A03     82  8050817582B7815B82AE814282BF82E582C182C682BE82A9...  LEN8_STRING_CP932         length=80, text="「すーぐ。ちょっとだから。マジすぐ終わるって。オレ、ほら、あやしくないでしょ？」"
0x00000A55      1  FF                                                   TERMINATOR_FF             
0x00000A56      1  FF                                                   TERMINATOR_FF             
0x00000A57      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000A59      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A5B      1  FF                                                   TERMINATOR_FF             
0x00000A5C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A5E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A60      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000A62      1  FF                                                   TERMINATOR_FF             
0x00000A63      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A65      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A67      3  F30161                                               IMM16_F3                  u16_be=353, u16_le=24833
0x00000A6A      1  FF                                                   TERMINATOR_FF             
0x00000A6B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000A6D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A6F      1  FF                                                   TERMINATOR_FF             
0x00000A70      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A72      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A74      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000A76      1  FF                                                   TERMINATOR_FF             
0x00000A77      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A79      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A7B      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000A7D      1  FF                                                   TERMINATOR_FF             
0x00000A7E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A80      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000A82      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000A84      1  FF                                                   TERMINATOR_FF             
0x00000A85      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000A87      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A89      1  FF                                                   TERMINATOR_FF             
0x00000A8A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A8C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A8E      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000A90      1  FF                                                   TERMINATOR_FF             
0x00000A91      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A93      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000A95      3  F319DD                                               IMM16_F3                  u16_be=6621, u16_le=56601
0x00000A98      1  FF                                                   TERMINATOR_FF             
0x00000A99      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A9B      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000A9D      9  8007495A3030323730                                   LEN8_STRING_CP932         length=7, text="IZ00270"
0x00000AA6      1  FF                                                   TERMINATOR_FF             
0x00000AA7      1  FF                                                   TERMINATOR_FF             
0x00000AA8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000AAA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000AAC     38  8024817582A682A581608163816382BB82A482C882F182C5...  LEN8_STRING_CP932         length=36, text="「えぇ～……そうなんですけどお……」"
0x00000AD2      1  FF                                                   TERMINATOR_FF             
0x00000AD3      1  FF                                                   TERMINATOR_FF             
0x00000AD4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000AD6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000AD8      1  FF                                                   TERMINATOR_FF             
0x00000AD9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000ADB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000ADD      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000ADF      1  FF                                                   TERMINATOR_FF             
0x00000AE0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AE2      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000AE4      3  F319DE                                               IMM16_F3                  u16_be=6622, u16_le=56857
0x00000AE7      1  FF                                                   TERMINATOR_FF             
0x00000AE8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000AEA      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000AEC      9  800745583435303230                                   LEN8_STRING_CP932         length=7, text="EX45020"
0x00000AF5      1  FF                                                   TERMINATOR_FF             
0x00000AF6      1  FF                                                   TERMINATOR_FF             
0x00000AF7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000AF9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000AFB     45  802B8175975C92E882C882A282F182C582B582E581482098...  LEN8_STRING_CP932         length=43, text="「予定ないんでしょ？ 話だけでいーんだって」"
0x00000B28      1  FF                                                   TERMINATOR_FF             
0x00000B29      1  FF                                                   TERMINATOR_FF             
0x00000B2A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000B2C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000B2E      1  FF                                                   TERMINATOR_FF             
0x00000B2F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B31      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B33      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000B35      1  FF                                                   TERMINATOR_FF             
0x00000B36      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B38      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000B3A      3  F319DF                                               IMM16_F3                  u16_be=6623, u16_le=57113
0x00000B3D      1  FF                                                   TERMINATOR_FF             
0x00000B3E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B40      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B42     98  806082BB82CC8F9790AB82CD8ADB93A190F294FC82B382F1...  LEN8_STRING_CP932         length=96, text="その女性は丸藤泉美さん、だった。\n昨日は痴漢で、今日はキャッチ。\nなんだか、いそがしい人だなあ。"
0x00000BA4      1  FF                                                   TERMINATOR_FF             
0x00000BA5      1  FF                                                   TERMINATOR_FF             
0x00000BA6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000BA8      2  0004                                                 WORD_00XX                 u16_be=4, low_byte=4
0x00000BAA     12  800A90BA82F082A982AF82E9                             LEN8_STRING_CP932         length=10, text="声をかける"
0x00000BB6      1  FF                                                   TERMINATOR_FF             
0x00000BB7      1  FF                                                   TERMINATOR_FF             
0x00000BB8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BBA      2  000A                                                 WORD_00XX                 u16_be=10, low_byte=10
0x00000BBC      3  F319E0                                               IMM16_F3                  u16_be=6624, u16_le=57369
0x00000BBF      1  FF                                                   TERMINATOR_FF             
0x00000BC0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000BC2      2  0005                                                 WORD_00XX                 u16_be=5, low_byte=5
0x00000BC4     12  800A976C8E7182F08CA982E9                             LEN8_STRING_CP932         length=10, text="様子を見る"
0x00000BD0      1  FF                                                   TERMINATOR_FF             
0x00000BD1      1  FF                                                   TERMINATOR_FF             
0x00000BD2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BD4      2  000B                                                 WORD_00XX                 u16_be=11, low_byte=11
0x00000BD6      3  F319E1                                               IMM16_F3                  u16_be=6625, u16_le=57625
0x00000BD9      1  FF                                                   TERMINATOR_FF             
0x00000BDA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000BDC      2  0006                                                 WORD_00XX                 u16_be=6, low_byte=6
0x00000BDE     10  800882D982C182C682AD                                 LEN8_STRING_CP932         length=8, text="ほっとく"
0x00000BE8      1  FF                                                   TERMINATOR_FF             
0x00000BE9      1  FF                                                   TERMINATOR_FF             
0x00000BEA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BEC      2  000C                                                 WORD_00XX                 u16_be=12, low_byte=12
0x00000BEE      3  F319E2                                               IMM16_F3                  u16_be=6626, u16_le=57881
0x00000BF1      1  FF                                                   TERMINATOR_FF             
0x00000BF2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BF4      2  0008                                                 WORD_00XX                 u16_be=8, low_byte=8
0x00000BF6      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000BF8      1  FF                                                   TERMINATOR_FF             
0x00000BF9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BFB      2  0009                                                 WORD_00XX                 u16_be=9, low_byte=9
0x00000BFD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000BFF      1  FF                                                   TERMINATOR_FF             
0x00000C00      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C02      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C04      2  F212                                                 IMM8_F2                   u8=18, s8=18
0x00000C06      1  FF                                                   TERMINATOR_FF             
0x00000C07      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000C09      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000C0B      1  FF                                                   TERMINATOR_FF             
0x00000C0C      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00000C0E      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x00000C11      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000C13      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00000C14      1  FF                                                   TERMINATOR_FF             
0x00000C15      2  000C                                                 WORD_00XX                 u16_be=12, low_byte=12
0x00000C17      1  1D                                                   OPAQUE_RAW_BYTES          bytes=1D
0x00000C18      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00000C1A      2  0012                                                 WORD_00XX                 u16_be=18, low_byte=18
0x00000C1C      1  EF                                                   OPAQUE_RAW_BYTES          bytes=EF
0x00000C1D      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00000C1F      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x00000C22      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000C24      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00000C25      1  FF                                                   TERMINATOR_FF             
0x00000C26      2  000E                                                 WORD_00XX                 u16_be=14, low_byte=14
0x00000C28      1  DD                                                   OPAQUE_RAW_BYTES          bytes=DD
0x00000C29      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C2B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C2D      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000C2F      1  FF                                                   TERMINATOR_FF             
0x00000C30      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C32      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000C34      3  F319E3                                               IMM16_F3                  u16_be=6627, u16_le=58137
0x00000C37      1  FF                                                   TERMINATOR_FF             
0x00000C38      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C3A      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000C3C      9  800745583435303330                                   LEN8_STRING_CP932         length=7, text="EX45030"
0x00000C45      1  FF                                                   TERMINATOR_FF             
0x00000C46      1  FF                                                   TERMINATOR_FF             
0x00000C47      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C49      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C4B     59  8039817582A6814182C882C98148208349838C82CC82B182...  LEN8_STRING_CP932         length=57, text="「え、なに？ オレのこと、きらいー？\n傷ついちゃうよなぁ」"
0x00000C86      1  FF                                                   TERMINATOR_FF             
0x00000C87      1  FF                                                   TERMINATOR_FF             
0x00000C88      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000C8A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000C8C      1  FF                                                   TERMINATOR_FF             
0x00000C8D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C8F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C91      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000C93      1  FF                                                   TERMINATOR_FF             
0x00000C94      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C96      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000C98      3  F319E4                                               IMM16_F3                  u16_be=6628, u16_le=58393
0x00000C9B      1  FF                                                   TERMINATOR_FF             
0x00000C9C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C9E      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000CA0      9  8007495A3030323830                                   LEN8_STRING_CP932         length=7, text="IZ00280"
0x00000CA9      1  FF                                                   TERMINATOR_FF             
0x00000CAA      1  FF                                                   TERMINATOR_FF             
0x00000CAB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000CAD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000CAF     48  802E817582A6815B814182BB82F182C882B182C682A282C1...  LEN8_STRING_CP932         length=46, text="「えー、そんなこといってないじゃないですかあ」"
0x00000CDF      1  FF                                                   TERMINATOR_FF             
0x00000CE0      1  FF                                                   TERMINATOR_FF             
0x00000CE1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000CE3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000CE5      1  FF                                                   TERMINATOR_FF             
0x00000CE6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CE8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CEA      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000CEC      1  FF                                                   TERMINATOR_FF             
0x00000CED      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CEF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000CF1      3  F319E5                                               IMM16_F3                  u16_be=6629, u16_le=58649
0x00000CF4      1  FF                                                   TERMINATOR_FF             
0x00000CF5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000CF7      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000CF9      9  800745583435303430                                   LEN8_STRING_CP932         length=7, text="EX45040"
0x00000D02      1  FF                                                   TERMINATOR_FF             
0x00000D03      1  FF                                                   TERMINATOR_FF             
0x00000D04      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D06      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D08     55  8035817582B682E1814182A2815B82E682CB81482082BF82...  LEN8_STRING_CP932         length=53, text="「じゃ、いーよね？ ちょっと、そこ。マジすぐ\nだから」"
0x00000D3F      1  FF                                                   TERMINATOR_FF             
0x00000D40      1  FF                                                   TERMINATOR_FF             
0x00000D41      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000D43      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000D45      1  FF                                                   TERMINATOR_FF             
0x00000D46      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D48      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D4A      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000D4C      1  FF                                                   TERMINATOR_FF             
0x00000D4D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D4F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000D51      3  F319E6                                               IMM16_F3                  u16_be=6630, u16_le=58905
0x00000D54      1  FF                                                   TERMINATOR_FF             
0x00000D55      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D57      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000D59      9  8007495A3030323930                                   LEN8_STRING_CP932         length=7, text="IZ00290"
0x00000D62      1  FF                                                   TERMINATOR_FF             
0x00000D63      1  FF                                                   TERMINATOR_FF             
0x00000D64      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D66      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D68     47  802D817582A682A5816081482082C582E082A78163816382...  LEN8_STRING_CP932         length=45, text="「えぇ～？ でもぉ……そーゆーのじゃないしー」"
0x00000D97      1  FF                                                   TERMINATOR_FF             
0x00000D98      1  FF                                                   TERMINATOR_FF             
0x00000D99      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000D9B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000D9D      1  FF                                                   TERMINATOR_FF             
0x00000D9E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000DA0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000DA2      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000DA4      1  FF                                                   TERMINATOR_FF             
0x00000DA5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000DA7      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000DA9      3  F319E7                                               IMM16_F3                  u16_be=6631, u16_le=59161
0x00000DAC      1  FF                                                   TERMINATOR_FF             
0x00000DAD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000DAF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000DB1     70  804482A4816082F1814182C882F182C682C882AD975C917A...  LEN8_STRING_CP932         length=68, text="う～ん、なんとなく予想はしてたんだけど、\nやっぱり状況は変わらない。"
0x00000DF7      1  FF                                                   TERMINATOR_FF             
0x00000DF8      1  FF                                                   TERMINATOR_FF             
0x00000DF9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000DFB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000DFD      1  FF                                                   TERMINATOR_FF             
0x00000DFE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E00      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E02      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000E04      1  FF                                                   TERMINATOR_FF             
0x00000E05      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E07      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000E09      3  F319E8                                               IMM16_F3                  u16_be=6632, u16_le=59417
0x00000E0C      1  FF                                                   TERMINATOR_FF             
0x00000E0D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E0F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E11     52  803295CF82ED82E782C882A282C682E4815B82E682E882E0...  LEN8_STRING_CP932         length=50, text="変わらないとゆーよりも、\n説得されそうな気配だよ。"
0x00000E45      1  FF                                                   TERMINATOR_FF             
0x00000E46      1  FF                                                   TERMINATOR_FF             
0x00000E47      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000E49      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000E4B      1  FF                                                   TERMINATOR_FF             
0x00000E4C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E4E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E50      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000E52      1  FF                                                   TERMINATOR_FF             
0x00000E53      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000E55      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000E57      3  F319E9                                               IMM16_F3                  u16_be=6633, u16_le=59673
0x00000E5A      1  FF                                                   TERMINATOR_FF             
0x00000E5B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E5D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000E5F    114  80708DF093FA82CC92738ABF82E082BB82A482C882F182BE...  LEN8_STRING_CP932         length=112, text="昨日の痴漢もそうなんだけど、はっきりいえば\nいいだけだと思うんだ。\nそれができない性格の人なのかもしれないけど。"
0x00000ED1      1  FF                                                   TERMINATOR_FF             
0x00000ED2      1  FF                                                   TERMINATOR_FF             
0x00000ED3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000ED5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000ED7      1  FF                                                   TERMINATOR_FF             
0x00000ED8      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00000EDA      2  0012                                                 WORD_00XX                 u16_be=18, low_byte=18
0x00000EDC      1  EF                                                   OPAQUE_RAW_BYTES          bytes=EF
0x00000EDD      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00000EDF      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x00000EE2      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000EE4      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00000EE5      1  FF                                                   TERMINATOR_FF             
0x00000EE6      2  0012                                                 WORD_00XX                 u16_be=18, low_byte=18
0x00000EE8      1  EF                                                   OPAQUE_RAW_BYTES          bytes=EF
0x00000EE9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000EEB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000EED      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000EEF      1  FF                                                   TERMINATOR_FF             
0x00000EF0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000EF2      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000EF4      3  F319EA                                               IMM16_F3                  u16_be=6634, u16_le=59929
0x00000EF7      1  FF                                                   TERMINATOR_FF             
0x00000EF8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000EFA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000EFC     60  803A8DF093FA82CC82A88E6F82B382F1814182DC82BD82C2...  LEN8_STRING_CP932         length=58, text="昨日のお姉さん、またつかまってるのか。\nこりない人だよな。"
0x00000F38      1  FF                                                   TERMINATOR_FF             
0x00000F39      1  FF                                                   TERMINATOR_FF             
0x00000F3A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000F3C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000F3E      1  FF                                                   TERMINATOR_FF             
0x00000F3F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F41      2  0037                                                 WORD_00XX                 u16_be=55, low_byte=55
0x00000F43      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000F45      1  FF                                                   TERMINATOR_FF             
0x00000F46      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F48      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F4A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000F4C      1  FF                                                   TERMINATOR_FF             
0x00000F4D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000F4F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000F51      3  F319EB                                               IMM16_F3                  u16_be=6635, u16_le=60185
0x00000F54      1  FF                                                   TERMINATOR_FF             
0x00000F55      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F57      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000F59     72  804682DC82A08141834C83838362836082CC82A88C5A82B3...  LEN8_STRING_CP932         length=70, text="まあ、キャッチのお兄さんなら、それほどひどいことにもならないだろうし。"
0x00000FA1      1  FF                                                   TERMINATOR_FF             
0x00000FA2      1  FF                                                   TERMINATOR_FF             
0x00000FA3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000FA5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000FA7      1  FF                                                   TERMINATOR_FF             
0x00000FA8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FAA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FAC      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000FAE      1  FF                                                   TERMINATOR_FF             
0x00000FAF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FB1      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000FB3      3  F319EC                                               IMM16_F3                  u16_be=6636, u16_le=60441
0x00000FB6      1  FF                                                   TERMINATOR_FF             
0x00000FB7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000FB9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000FBB     32  801E82D982C182C682A282C482E0814195BD8B4382BE82C6...  LEN8_STRING_CP932         length=30, text="ほっといても、平気だと思うな。"
0x00000FDB      1  FF                                                   TERMINATOR_FF             
0x00000FDC      1  FF                                                   TERMINATOR_FF             
0x00000FDD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000FDF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000FE1      1  FF                                                   TERMINATOR_FF             
0x00000FE2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FE4      2  0053                                                 WORD_00XX                 u16_be=83, low_byte=83
0x00000FE6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000FE8      1  FF                                                   TERMINATOR_FF             
0x00000FE9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FEB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FED      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000FEF      1  FF                                                   TERMINATOR_FF             
0x00000FF0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000FF2      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000FF4      3  F319ED                                               IMM16_F3                  u16_be=6637, u16_le=60697
0x00000FF7      1  FF                                                   TERMINATOR_FF             
0x00000FF8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000FFA      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000FFC      9  8007495A3930303130                                   LEN8_STRING_CP932         length=7, text="IZ90010"
0x00001005      1  FF                                                   TERMINATOR_FF             
0x00001006      1  FF                                                   TERMINATOR_FF             
0x00001007      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001009      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000100B     52  8032817582BE815B82A982E7815B81418DA282E982C182C4...  LEN8_STRING_CP932         length=50, text="「だーからー、困るっていってるじゃない\nですかー」"
0x0000103F      1  FF                                                   TERMINATOR_FF             
0x00001040      1  FF                                                   TERMINATOR_FF             
0x00001041      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001043      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001045      1  FF                                                   TERMINATOR_FF             
0x00001046      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001048      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000104A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000104C      1  FF                                                   TERMINATOR_FF             
0x0000104D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000104F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001051      3  F319EE                                               IMM16_F3                  u16_be=6638, u16_le=60953
0x00001054      1  FF                                                   TERMINATOR_FF             
0x00001055      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001057      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001059     32  801E82A8814182BF82E182F182C6926682EA82BB82A482B6...  LEN8_STRING_CP932         length=30, text="お、ちゃんと断れそうじゃない。"
0x00001079      1  FF                                                   TERMINATOR_FF             
0x0000107A      1  FF                                                   TERMINATOR_FF             
0x0000107B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000107D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000107F      1  FF                                                   TERMINATOR_FF             
0x00001080      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001082      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001084      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001086      1  FF                                                   TERMINATOR_FF             
0x00001087      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001089      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000108B      3  F319EF                                               IMM16_F3                  u16_be=6639, u16_le=61209
0x0000108E      1  FF                                                   TERMINATOR_FF             
0x0000108F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001091      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001093     94  805C82DC82A0814182D982C182C682A282C482E091E58FE4...  LEN8_STRING_CP932         length=92, text="まあ、ほっといても大丈夫……かな。\nなんだかね、変な下心があるみたいに思われる\nのもやだし。"
0x000010F1      1  FF                                                   TERMINATOR_FF             
0x000010F2      1  FF                                                   TERMINATOR_FF             
0x000010F3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000010F5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000010F7      1  FF                                                   TERMINATOR_FF             
0x000010F8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010FA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000010FC      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000010FE      1  FF                                                   TERMINATOR_FF             
0x000010FF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001101      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001103      3  F319F0                                               IMM16_F3                  u16_be=6640, u16_le=61465
0x00001106      1  FF                                                   TERMINATOR_FF             
0x00001107      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001109      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000110B     52  803282BE82A282BD82A282B382A0814182E0815B82BF82E5...  LEN8_STRING_CP932         length=50, text="だいたいさあ、もーちょっと気をつけるべき\nだよね。"
0x0000113F      1  FF                                                   TERMINATOR_FF             
0x00001140      1  FF                                                   TERMINATOR_FF             
0x00001141      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001143      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001145      1  FF                                                   TERMINATOR_FF             
0x00001146      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001148      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000114A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000114C      1  FF                                                   TERMINATOR_FF             
0x0000114D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000114F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001151      3  F319F1                                               IMM16_F3                  u16_be=6641, u16_le=61721
0x00001154      1  FF                                                   TERMINATOR_FF             
0x00001155      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001157      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001159     84  805282A082F182C882C98BB982E082C68A4A82AF82BF82E1...  LEN8_STRING_CP932         length=82, text="あんなに胸もと開けちゃって。\nあれじゃ、見せたいのよーって誘ってるみたい\nだもん。"
0x000011AD      1  FF                                                   TERMINATOR_FF             
0x000011AE      1  FF                                                   TERMINATOR_FF             
0x000011AF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000011B1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000011B3      1  FF                                                   TERMINATOR_FF             
0x000011B4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000011B6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000011B8      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000011BA      1  FF                                                   TERMINATOR_FF             
0x000011BB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000011BD      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000011BF      3  F319F2                                               IMM16_F3                  u16_be=6642, u16_le=61977
0x000011C2      1  FF                                                   TERMINATOR_FF             
0x000011C3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000011C5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000011C7     88  805682A282E281418CA982B982BD82A282CC81485C6E8163...  LEN8_STRING_CP932         length=86, text="いや、見せたいの？\n……そんなこともないか。\nそんな度胸のありそうな人にも見えないし。"
0x0000121F      1  FF                                                   TERMINATOR_FF             
0x00001220      1  FF                                                   TERMINATOR_FF             
0x00001221      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001223      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001225      1  FF                                                   TERMINATOR_FF             
0x00001226      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001228      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000122A      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000122C      1  FF                                                   TERMINATOR_FF             
0x0000122D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000122F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001231      3  F319F3                                               IMM16_F3                  u16_be=6643, u16_le=62233
0x00001234      1  FF                                                   TERMINATOR_FF             
0x00001235      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001237      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001239      9  8007495A3930303230                                   LEN8_STRING_CP932         length=7, text="IZ90020"
0x00001242      1  FF                                                   TERMINATOR_FF             
0x00001243      1  FF                                                   TERMINATOR_FF             
0x00001244      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001246      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001248     52  8032817582BE815B82A9815B82E7815B814282C282A282C4...  LEN8_STRING_CP932         length=50, text="「だーかーらー。ついて来ないで\nくーだーさーいー」"
0x0000127C      1  FF                                                   TERMINATOR_FF             
0x0000127D      1  FF                                                   TERMINATOR_FF             
0x0000127E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001280      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001282      1  FF                                                   TERMINATOR_FF             
0x00001283      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001285      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001287      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x00001289      1  FF                                                   TERMINATOR_FF             
0x0000128A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000128C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000128E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001290      1  FF                                                   TERMINATOR_FF             
0x00001291      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001293      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001295      1  FF                                                   TERMINATOR_FF             
0x00001296      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001298      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000129A      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x0000129C      1  FF                                                   TERMINATOR_FF             
0x0000129D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000129F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000012A1      2  F2FF                                                 IMM8_F2                   u8=255, s8=-1
0x000012A3      1  FF                                                   TERMINATOR_FF             
0x000012A4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000012A6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000012A8      1  FF                                                   TERMINATOR_FF             
0x000012A9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012AB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012AD      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000012AF      1  FF                                                   TERMINATOR_FF             
0x000012B0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012B2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000012B4      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000012B6      1  FF                                                   TERMINATOR_FF             
0x000012B7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012B9      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000012BB      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000012BD      1  FF                                                   TERMINATOR_FF             
0x000012BE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000012C0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000012C2      1  FF                                                   TERMINATOR_FF             
0x000012C3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012C5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012C7      2  F20B                                                 IMM8_F2                   u8=11, s8=11
0x000012C9      1  FF                                                   TERMINATOR_FF             
0x000012CA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000012CC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000012CE      1  FF                                                   TERMINATOR_FF             
0x000012CF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012D1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012D3      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x000012D5      1  FF                                                   TERMINATOR_FF             
0x000012D6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000012D8      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x000012DA     14  800C50533241303032612E62696E                         LEN8_STRING_CP932         length=12, text="PS2A002a.bin"
0x000012E8      1  FF                                                   TERMINATOR_FF             
0x000012E9      1  FF                                                   TERMINATOR_FF             
0x000012EA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000012EC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000012EE      1  FF                                                   TERMINATOR_FF             
0x000012EF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012F1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012F3      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000012F5      1  FF                                                   TERMINATOR_FF             
0x000012F6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000012F8      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000012FA      3  F319F4                                               IMM16_F3                  u16_be=6644, u16_le=62489
0x000012FD      1  FF                                                   TERMINATOR_FF             
0x000012FE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001300      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001302     16  800E81758ADB93A182B382F121218176                     LEN8_STRING_CP932         length=14, text="「丸藤さん!!」"
0x00001312      1  FF                                                   TERMINATOR_FF             
0x00001313      1  FF                                                   TERMINATOR_FF             
0x00001314      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001316      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001318      1  FF                                                   TERMINATOR_FF             
0x00001319      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000131B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000131D      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x0000131F      1  FF                                                   TERMINATOR_FF             
0x00001320      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001322      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001324      2  F229                                                 IMM8_F2                   u8=41, s8=41
0x00001326      1  FF                                                   TERMINATOR_FF             
0x00001327      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001329      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000132B      1  FF                                                   TERMINATOR_FF             
0x0000132C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000132E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001330      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00001332      1  FF                                                   TERMINATOR_FF             
0x00001333      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001335      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001337      3  F300C0                                               IMM16_F3                  u16_be=192, u16_le=49152
0x0000133A      1  FF                                                   TERMINATOR_FF             
0x0000133B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000133D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000133F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001341      1  FF                                                   TERMINATOR_FF             
0x00001342      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001344      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001346      1  FF                                                   TERMINATOR_FF             
0x00001347      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001349      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000134B      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000134D      1  FF                                                   TERMINATOR_FF             
0x0000134E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001350      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001352      2  F240                                                 IMM8_F2                   u8=64, s8=64
0x00001354      1  FF                                                   TERMINATOR_FF             
0x00001355      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001357      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001359      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000135B      1  FF                                                   TERMINATOR_FF             
0x0000135C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000135E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001360      1  FF                                                   TERMINATOR_FF             
0x00001361      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001363      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001365      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00001367      1  FF                                                   TERMINATOR_FF             
0x00001368      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000136A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000136C      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000136E      1  FF                                                   TERMINATOR_FF             
0x0000136F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001371      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001373      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00001375      1  FF                                                   TERMINATOR_FF             
0x00001376      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001378      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000137A      1  FF                                                   TERMINATOR_FF             
0x0000137B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000137D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000137F      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001381      1  FF                                                   TERMINATOR_FF             
0x00001382      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001384      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001386      3  F319F5                                               IMM16_F3                  u16_be=6645, u16_le=62745
0x00001389      1  FF                                                   TERMINATOR_FF             
0x0000138A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000138C      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000138E      9  8007495A3030333030                                   LEN8_STRING_CP932         length=7, text="IZ00300"
0x00001397      1  FF                                                   TERMINATOR_FF             
0x00001398      1  FF                                                   TERMINATOR_FF             
0x00001399      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000139B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000139D     21  8013817582A682C13F212082A682A682C13F3F8176           LEN8_STRING_CP932         length=19, text="「えっ?! ええっ??」"
0x000013B2      1  FF                                                   TERMINATOR_FF             
0x000013B3      1  FF                                                   TERMINATOR_FF             
0x000013B4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000013B6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000013B8      1  FF                                                   TERMINATOR_FF             
0x000013B9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000013BB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000013BD      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000013BF      1  FF                                                   TERMINATOR_FF             
0x000013C0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000013C2      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000013C4      3  F319F6                                               IMM16_F3                  u16_be=6646, u16_le=63001
0x000013C7      1  FF                                                   TERMINATOR_FF             
0x000013C8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000013CA      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000013CC      9  800745583435303530                                   LEN8_STRING_CP932         length=7, text="EX45050"
0x000013D5      1  FF                                                   TERMINATOR_FF             
0x000013D6      1  FF                                                   TERMINATOR_FF             
0x000013D7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000013D9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000013DB     22  8014817582C882C98141926D82E88D8782A281488176         LEN8_STRING_CP932         length=20, text="「なに、知り合い？」"
0x000013F1      1  FF                                                   TERMINATOR_FF             
0x000013F2      1  FF                                                   TERMINATOR_FF             
0x000013F3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000013F5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000013F7      1  FF                                                   TERMINATOR_FF             
0x000013F8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000013FA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000013FC      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000013FE      1  FF                                                   TERMINATOR_FF             
0x000013FF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001401      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001403      3  F319F7                                               IMM16_F3                  u16_be=6647, u16_le=63257
0x00001406      1  FF                                                   TERMINATOR_FF             
0x00001407      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001409      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000140B     72  8046837A83588367816982BE82A98141834C838383628360...  LEN8_STRING_CP932         length=70, text="ホスト（だか、キャッチだか）風お兄さんは、\nぼくにいやな視線を向ける。"
0x00001453      1  FF                                                   TERMINATOR_FF             
0x00001454      1  FF                                                   TERMINATOR_FF             
0x00001455      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001457      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001459      1  FF                                                   TERMINATOR_FF             
0x0000145A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000145C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000145E      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00001460      1  FF                                                   TERMINATOR_FF             
0x00001461      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001463      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001465      3  F300CE                                               IMM16_F3                  u16_be=206, u16_le=52736
0x00001468      1  FF                                                   TERMINATOR_FF             
0x00001469      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000146B      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000146D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000146F      1  FF                                                   TERMINATOR_FF             
0x00001470      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001472      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001474      1  FF                                                   TERMINATOR_FF             
0x00001475      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001477      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001479      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000147B      1  FF                                                   TERMINATOR_FF             
0x0000147C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000147E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001480      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001482      1  FF                                                   TERMINATOR_FF             
0x00001483      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001485      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001487      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00001489      1  FF                                                   TERMINATOR_FF             
0x0000148A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000148C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000148E      1  FF                                                   TERMINATOR_FF             
0x0000148F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001491      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001493      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001495      1  FF                                                   TERMINATOR_FF             
0x00001496      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001498      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000149A      3  F319F8                                               IMM16_F3                  u16_be=6648, u16_le=63513
0x0000149D      1  FF                                                   TERMINATOR_FF             
0x0000149E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000014A0      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000014A2      9  8007495A3030333130                                   LEN8_STRING_CP932         length=7, text="IZ00310"
0x000014AB      1  FF                                                   TERMINATOR_FF             
0x000014AC      1  FF                                                   TERMINATOR_FF             
0x000014AD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000014AF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000014B1     26  8018817582A682C182C68163816382A082EA816381638148...  LEN8_STRING_CP932         length=24, text="「えっと……あれ……？」"
0x000014CB      1  FF                                                   TERMINATOR_FF             
0x000014CC      1  FF                                                   TERMINATOR_FF             
0x000014CD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000014CF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000014D1      1  FF                                                   TERMINATOR_FF             
0x000014D2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000014D4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000014D6      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000014D8      1  FF                                                   TERMINATOR_FF             
0x000014D9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000014DB      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000014DD      3  F319F9                                               IMM16_F3                  u16_be=6649, u16_le=63769
0x000014E0      1  FF                                                   TERMINATOR_FF             
0x000014E1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000014E3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000014E5     66  80408ADB93A182B382F182E0814182B182C182BF82AA8B7E...  LEN8_STRING_CP932         length=64, text="丸藤さんも、こっちが救いの手をさしのべてる\nのに気づいてないし。"
0x00001527      1  FF                                                   TERMINATOR_FF             
0x00001528      1  FF                                                   TERMINATOR_FF             
0x00001529      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000152B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000152D      1  FF                                                   TERMINATOR_FF             
0x0000152E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001530      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001532      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001534      1  FF                                                   TERMINATOR_FF             
0x00001535      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001537      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001539      3  F319FA                                               IMM16_F3                  u16_be=6650, u16_le=64025
0x0000153C      1  FF                                                   TERMINATOR_FF             
0x0000153D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000153F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001541     38  802481758ADB93A182B382F1814182CD82E282AD82B582C8...  LEN8_STRING_CP932         length=36, text="「丸藤さん、はやくしないと遅れる!!」"
0x00001567      1  FF                                                   TERMINATOR_FF             
0x00001568      1  FF                                                   TERMINATOR_FF             
0x00001569      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000156B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000156D      1  FF                                                   TERMINATOR_FF             
0x0000156E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001570      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001572      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00001574      1  FF                                                   TERMINATOR_FF             
0x00001575      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001577      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001579      3  F300C4                                               IMM16_F3                  u16_be=196, u16_le=50176
0x0000157C      1  FF                                                   TERMINATOR_FF             
0x0000157D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000157F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001581      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001583      1  FF                                                   TERMINATOR_FF             
0x00001584      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001586      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001588      1  FF                                                   TERMINATOR_FF             
0x00001589      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000158B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000158D      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000158F      1  FF                                                   TERMINATOR_FF             
0x00001590      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001592      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001594      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001596      1  FF                                                   TERMINATOR_FF             
0x00001597      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001599      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000159B      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000159D      1  FF                                                   TERMINATOR_FF             
0x0000159E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000015A0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000015A2      1  FF                                                   TERMINATOR_FF             
0x000015A3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000015A5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000015A7      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000015A9      1  FF                                                   TERMINATOR_FF             
0x000015AA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000015AC      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000015AE      3  F319FB                                               IMM16_F3                  u16_be=6651, u16_le=64281
0x000015B1      1  FF                                                   TERMINATOR_FF             
0x000015B2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000015B4      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000015B6      9  8007495A3030333230                                   LEN8_STRING_CP932         length=7, text="IZ00320"
0x000015BF      1  FF                                                   TERMINATOR_FF             
0x000015C0      1  FF                                                   TERMINATOR_FF             
0x000015C1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000015C3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000015C5     30  801C817582A08163816382A682C182C68163816382A082CC...  LEN8_STRING_CP932         length=28, text="「あ……えっと……あの……」"
0x000015E3      1  FF                                                   TERMINATOR_FF             
0x000015E4      1  FF                                                   TERMINATOR_FF             
0x000015E5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000015E7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000015E9      1  FF                                                   TERMINATOR_FF             
0x000015EA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000015EC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000015EE      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000015F0      1  FF                                                   TERMINATOR_FF             
0x000015F1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000015F3      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000015F5      3  F319FC                                               IMM16_F3                  u16_be=6652, u16_le=64537
0x000015F8      1  FF                                                   TERMINATOR_FF             
0x000015F9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000015FB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000015FD     42  802881758ADB93A182B382F181418AD482C982A082ED82C8...  LEN8_STRING_CP932         length=40, text="「丸藤さん、間にあわなくなっちゃうよ!!」"
0x00001627      1  FF                                                   TERMINATOR_FF             
0x00001628      1  FF                                                   TERMINATOR_FF             
0x00001629      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000162B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000162D      1  FF                                                   TERMINATOR_FF             
0x0000162E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001630      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001632      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00001634      1  FF                                                   TERMINATOR_FF             
0x00001635      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001637      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001639      3  F300C2                                               IMM16_F3                  u16_be=194, u16_le=49664
0x0000163C      1  FF                                                   TERMINATOR_FF             
0x0000163D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000163F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001641      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001643      1  FF                                                   TERMINATOR_FF             
0x00001644      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001646      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001648      1  FF                                                   TERMINATOR_FF             
0x00001649      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000164B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000164D      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000164F      1  FF                                                   TERMINATOR_FF             
0x00001650      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001652      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001654      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001656      1  FF                                                   TERMINATOR_FF             
0x00001657      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001659      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000165B      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000165D      1  FF                                                   TERMINATOR_FF             
0x0000165E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001660      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001662      1  FF                                                   TERMINATOR_FF             
0x00001663      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001665      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001667      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001669      1  FF                                                   TERMINATOR_FF             
0x0000166A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000166C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000166E      3  F319FD                                               IMM16_F3                  u16_be=6653, u16_le=64793
0x00001671      1  FF                                                   TERMINATOR_FF             
0x00001672      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001674      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001676      9  8007495A3030333330                                   LEN8_STRING_CP932         length=7, text="IZ00330"
0x0000167F      1  FF                                                   TERMINATOR_FF             
0x00001680      1  FF                                                   TERMINATOR_FF             
0x00001681      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001683      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001685     56  8036817582A08163816382A682C182C6814182A082CC8163...  LEN8_STRING_CP932         length=54, text="「あ……えっと、あの……知り合い、来ちゃったんで……」"
0x000016BD      1  FF                                                   TERMINATOR_FF             
0x000016BE      1  FF                                                   TERMINATOR_FF             
0x000016BF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000016C1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000016C3      1  FF                                                   TERMINATOR_FF             
0x000016C4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016C6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016C8      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000016CA      1  FF                                                   TERMINATOR_FF             
0x000016CB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000016CD      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000016CF      3  F319FE                                               IMM16_F3                  u16_be=6654, u16_le=65049
0x000016D2      1  FF                                                   TERMINATOR_FF             
0x000016D3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000016D5      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000016D7      9  800745583435303630                                   LEN8_STRING_CP932         length=7, text="EX45060"
0x000016E0      1  FF                                                   TERMINATOR_FF             
0x000016E1      1  FF                                                   TERMINATOR_FF             
0x000016E2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000016E4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000016E6     26  8018817582A682A58160814182BB815B82C882F182BE815B...  LEN8_STRING_CP932         length=24, text="「えぇ～、そーなんだー」"
0x00001700      1  FF                                                   TERMINATOR_FF             
0x00001701      1  FF                                                   TERMINATOR_FF             
0x00001702      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001704      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001706      1  FF                                                   TERMINATOR_FF             
0x00001707      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001709      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000170B      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000170D      1  FF                                                   TERMINATOR_FF             
0x0000170E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001710      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001712      3  F319FF                                               IMM16_F3                  u16_be=6655, u16_le=65305
0x00001715      1  FF                                                   TERMINATOR_FF             
0x00001716      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001718      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000171A      9  800745583435303730                                   LEN8_STRING_CP932         length=7, text="EX45070"
0x00001723      1  FF                                                   TERMINATOR_FF             
0x00001724      1  FF                                                   TERMINATOR_FF             
0x00001725      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001727      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001729     36  802281758B7D82A282C582C882A282C182C482A282C182C4...  LEN8_STRING_CP932         length=34, text="「急いでないっていってたのになあ」"
0x0000174D      1  FF                                                   TERMINATOR_FF             
0x0000174E      1  FF                                                   TERMINATOR_FF             
0x0000174F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001751      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001753      1  FF                                                   TERMINATOR_FF             
0x00001754      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001756      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001758      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000175A      1  FF                                                   TERMINATOR_FF             
0x0000175B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000175D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000175F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001761      1  FF                                                   TERMINATOR_FF             
0x00001762      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001764      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001766      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001768      1  FF                                                   TERMINATOR_FF             
0x00001769      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000176B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000176D      1  FF                                                   TERMINATOR_FF             
0x0000176E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001770      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001772      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00001774      1  FF                                                   TERMINATOR_FF             
0x00001775      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001777      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001779      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000177B      1  FF                                                   TERMINATOR_FF             
0x0000177C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000177E      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001780      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00001782      1  FF                                                   TERMINATOR_FF             
0x00001783      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001785      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001787      1  FF                                                   TERMINATOR_FF             
0x00001788      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000178A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000178C      2  F20B                                                 IMM8_F2                   u8=11, s8=11
0x0000178E      1  FF                                                   TERMINATOR_FF             
0x0000178F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001791      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001793      1  FF                                                   TERMINATOR_FF             
0x00001794      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001796      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001798      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000179A      1  FF                                                   TERMINATOR_FF             
0x0000179B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000179D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000179F      3  F31A00                                               IMM16_F3                  u16_be=6656, u16_le=26
0x000017A2      1  FF                                                   TERMINATOR_FF             
0x000017A3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000017A5      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000017A7      9  800745583435303830                                   LEN8_STRING_CP932         length=7, text="EX45080"
0x000017B0      1  FF                                                   TERMINATOR_FF             
0x000017B1      1  FF                                                   TERMINATOR_FF             
0x000017B2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000017B4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000017B6     26  8018817582A0814182A282DC8E9E8AD482A082E982A38148...  LEN8_STRING_CP932         length=24, text="「あ、いま時間あるぅ？」"
0x000017D0      1  FF                                                   TERMINATOR_FF             
0x000017D1      1  FF                                                   TERMINATOR_FF             
0x000017D2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000017D4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000017D6      1  FF                                                   TERMINATOR_FF             
0x000017D7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000017D9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000017DB      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000017DD      1  FF                                                   TERMINATOR_FF             
0x000017DE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000017E0      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000017E2      3  F31A01                                               IMM16_F3                  u16_be=6657, u16_le=282
0x000017E5      1  FF                                                   TERMINATOR_FF             
0x000017E6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000017E8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000017EA    110  806C837A83588367816982BE82A9834C838383628360816A...  LEN8_STRING_CP932         length=108, text="ホスト（だかキャッチ）風お兄さんは、\nせっかくの獲物を横取りされたって表情で、\n次の標的へと向かっていった。"
0x00001858      1  FF                                                   TERMINATOR_FF             
0x00001859      1  FF                                                   TERMINATOR_FF             
0x0000185A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000185C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000185E      1  FF                                                   TERMINATOR_FF             
0x0000185F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001861      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001863      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00001865      1  FF                                                   TERMINATOR_FF             
0x00001866      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001868      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000186A      3  F300C5                                               IMM16_F3                  u16_be=197, u16_le=50432
0x0000186D      1  FF                                                   TERMINATOR_FF             
0x0000186E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001870      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001872      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001874      1  FF                                                   TERMINATOR_FF             
0x00001875      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001877      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001879      1  FF                                                   TERMINATOR_FF             
0x0000187A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000187C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000187E      2  F20A                                                 IMM8_F2                   u8=10, s8=10
0x00001880      1  FF                                                   TERMINATOR_FF             
0x00001881      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001883      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001885      2  F210                                                 IMM8_F2                   u8=16, s8=16
0x00001887      1  FF                                                   TERMINATOR_FF             
0x00001888      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000188A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000188C      1  FF                                                   TERMINATOR_FF             
0x0000188D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000188F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001891      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00001893      1  FF                                                   TERMINATOR_FF             
0x00001894      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001896      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001898      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000189A      1  FF                                                   TERMINATOR_FF             
0x0000189B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000189D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000189F      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000018A1      1  FF                                                   TERMINATOR_FF             
0x000018A2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000018A4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000018A6      1  FF                                                   TERMINATOR_FF             
0x000018A7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018A9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018AB      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000018AD      1  FF                                                   TERMINATOR_FF             
0x000018AE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018B0      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000018B2      3  F31A02                                               IMM16_F3                  u16_be=6658, u16_le=538
0x000018B5      1  FF                                                   TERMINATOR_FF             
0x000018B6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000018B8      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000018BA      9  8007495A3030333430                                   LEN8_STRING_CP932         length=7, text="IZ00340"
0x000018C3      1  FF                                                   TERMINATOR_FF             
0x000018C4      1  FF                                                   TERMINATOR_FF             
0x000018C5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000018C7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000018C9     28  801A817582D382A38163816382E682A982C182BD829F8163...  LEN8_STRING_CP932         length=26, text="「ふぅ……よかったぁ……」"
0x000018E5      1  FF                                                   TERMINATOR_FF             
0x000018E6      1  FF                                                   TERMINATOR_FF             
0x000018E7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000018E9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000018EB      1  FF                                                   TERMINATOR_FF             
0x000018EC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018EE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018F0      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000018F2      1  FF                                                   TERMINATOR_FF             
0x000018F3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000018F5      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000018F7      3  F31A03                                               IMM16_F3                  u16_be=6659, u16_le=794
0x000018FA      1  FF                                                   TERMINATOR_FF             
0x000018FB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000018FD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000018FF     66  80408ADB93A182B382F182CD814182D982C182C682B582BD...  LEN8_STRING_CP932         length=64, text="丸藤さんは、ほっとしたように大きなため息を\nつき、肩の力を抜く。"
0x00001941      1  FF                                                   TERMINATOR_FF             
0x00001942      1  FF                                                   TERMINATOR_FF             
0x00001943      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001945      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001947      1  FF                                                   TERMINATOR_FF             
0x00001948      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000194A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000194C      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000194E      1  FF                                                   TERMINATOR_FF             
0x0000194F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001951      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001953      3  F300CB                                               IMM16_F3                  u16_be=203, u16_le=51968
0x00001956      1  FF                                                   TERMINATOR_FF             
0x00001957      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001959      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000195B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000195D      1  FF                                                   TERMINATOR_FF             
0x0000195E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001960      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001962      1  FF                                                   TERMINATOR_FF             
0x00001963      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001965      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001967      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00001969      1  FF                                                   TERMINATOR_FF             
0x0000196A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000196C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000196E      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001970      1  FF                                                   TERMINATOR_FF             
0x00001971      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001973      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001975      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00001977      1  FF                                                   TERMINATOR_FF             
0x00001978      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000197A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000197C      1  FF                                                   TERMINATOR_FF             
0x0000197D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000197F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001981      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001983      1  FF                                                   TERMINATOR_FF             
0x00001984      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001986      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001988      3  F31A04                                               IMM16_F3                  u16_be=6660, u16_le=1050
0x0000198B      1  FF                                                   TERMINATOR_FF             
0x0000198C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000198E      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001990      9  8007495A3030333530                                   LEN8_STRING_CP932         length=7, text="IZ00350"
0x00001999      1  FF                                                   TERMINATOR_FF             
0x0000199A      1  FF                                                   TERMINATOR_FF             
0x0000199B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000199D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000199F     58  8038817582B782C182B282AD82B582C282B182A282F182BE...  LEN8_STRING_CP932         length=56, text="「すっごくしつこいんだもん。やだって、\nいってるのにー」"
0x000019D9      1  FF                                                   TERMINATOR_FF             
0x000019DA      1  FF                                                   TERMINATOR_FF             
0x000019DB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000019DD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000019DF      1  FF                                                   TERMINATOR_FF             
0x000019E0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000019E2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000019E4      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000019E6      1  FF                                                   TERMINATOR_FF             
0x000019E7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000019E9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000019EB      3  F31A05                                               IMM16_F3                  u16_be=6661, u16_le=1306
0x000019EE      1  FF                                                   TERMINATOR_FF             
0x000019EF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000019F1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000019F3     20  801282A282C182C482C882A982C182BD82E68142             LEN8_STRING_CP932         length=18, text="いってなかったよ。"
0x00001A07      1  FF                                                   TERMINATOR_FF             
0x00001A08      1  FF                                                   TERMINATOR_FF             
0x00001A09      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001A0B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001A0D      1  FF                                                   TERMINATOR_FF             
0x00001A0E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A10      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A12      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00001A14      1  FF                                                   TERMINATOR_FF             
0x00001A15      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A17      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001A19      3  F300C0                                               IMM16_F3                  u16_be=192, u16_le=49152
0x00001A1C      1  FF                                                   TERMINATOR_FF             
0x00001A1D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A1F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001A21      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001A23      1  FF                                                   TERMINATOR_FF             
0x00001A24      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001A26      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001A28      1  FF                                                   TERMINATOR_FF             
0x00001A29      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A2B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A2D      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00001A2F      1  FF                                                   TERMINATOR_FF             
0x00001A30      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A32      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001A34      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001A36      1  FF                                                   TERMINATOR_FF             
0x00001A37      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A39      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001A3B      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00001A3D      1  FF                                                   TERMINATOR_FF             
0x00001A3E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001A40      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001A42      1  FF                                                   TERMINATOR_FF             
0x00001A43      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A45      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A47      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001A49      1  FF                                                   TERMINATOR_FF             
0x00001A4A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A4C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001A4E      3  F31A06                                               IMM16_F3                  u16_be=6662, u16_le=1562
0x00001A51      1  FF                                                   TERMINATOR_FF             
0x00001A52      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001A54      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001A56      9  8007495A3030333630                                   LEN8_STRING_CP932         length=7, text="IZ00360"
0x00001A5F      1  FF                                                   TERMINATOR_FF             
0x00001A60      1  FF                                                   TERMINATOR_FF             
0x00001A61      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001A63      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001A65     14  800C81758163816382A081488176                         LEN8_STRING_CP932         length=12, text="「……あ？」"
0x00001A73      1  FF                                                   TERMINATOR_FF             
0x00001A74      1  FF                                                   TERMINATOR_FF             
0x00001A75      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001A77      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001A79      1  FF                                                   TERMINATOR_FF             
0x00001A7A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A7C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A7E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001A80      1  FF                                                   TERMINATOR_FF             
0x00001A81      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001A83      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001A85      3  F31A07                                               IMM16_F3                  u16_be=6663, u16_le=1818
0x00001A88      1  FF                                                   TERMINATOR_FF             
0x00001A89      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001A8B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001A8D    108  806A8ADB93A182B382F182CD814182E682A482E282AD82DA...  LEN8_STRING_CP932         length=106, text="丸藤さんは、ようやくぼくがいることを\n思い出してくれたらしい。\n警戒するゲッシ類のような瞳で見つめられた。"
0x00001AF9      1  FF                                                   TERMINATOR_FF             
0x00001AFA      1  FF                                                   TERMINATOR_FF             
0x00001AFB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001AFD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001AFF      1  FF                                                   TERMINATOR_FF             
0x00001B00      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B02      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B04      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001B06      1  FF                                                   TERMINATOR_FF             
0x00001B07      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B09      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001B0B      3  F31A08                                               IMM16_F3                  u16_be=6664, u16_le=2074
0x00001B0E      1  FF                                                   TERMINATOR_FF             
0x00001B0F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001B11      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001B13     60  803A82B182ED82AA82C182C482E982ED82AF82C582CD82C8...  LEN8_STRING_CP932         length=58, text="こわがってるわけではないけど、ぜんぜん安心\nしてない感じ。"
0x00001B4F      1  FF                                                   TERMINATOR_FF             
0x00001B50      1  FF                                                   TERMINATOR_FF             
0x00001B51      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001B53      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001B55      1  FF                                                   TERMINATOR_FF             
0x00001B56      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B58      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B5A      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00001B5C      1  FF                                                   TERMINATOR_FF             
0x00001B5D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B5F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001B61      3  F300C4                                               IMM16_F3                  u16_be=196, u16_le=50176
0x00001B64      1  FF                                                   TERMINATOR_FF             
0x00001B65      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B67      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001B69      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001B6B      1  FF                                                   TERMINATOR_FF             
0x00001B6C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001B6E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001B70      1  FF                                                   TERMINATOR_FF             
0x00001B71      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B73      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B75      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00001B77      1  FF                                                   TERMINATOR_FF             
0x00001B78      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B7A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001B7C      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001B7E      1  FF                                                   TERMINATOR_FF             
0x00001B7F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B81      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001B83      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00001B85      1  FF                                                   TERMINATOR_FF             
0x00001B86      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001B88      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001B8A      1  FF                                                   TERMINATOR_FF             
0x00001B8B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B8D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B8F      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001B91      1  FF                                                   TERMINATOR_FF             
0x00001B92      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001B94      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001B96      3  F31A09                                               IMM16_F3                  u16_be=6665, u16_le=2330
0x00001B99      1  FF                                                   TERMINATOR_FF             
0x00001B9A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001B9C      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001B9E      9  8007495A3030333730                                   LEN8_STRING_CP932         length=7, text="IZ00370"
0x00001BA7      1  FF                                                   TERMINATOR_FF             
0x00001BA8      1  FF                                                   TERMINATOR_FF             
0x00001BA9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001BAB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001BAD     62  803C817582A08163816382C782A482E0814182A082E882AA...  LEN8_STRING_CP932         length=60, text="「あ……どうも、ありがとう。しつこくされて、困ってたんです」"
0x00001BEB      1  FF                                                   TERMINATOR_FF             
0x00001BEC      1  FF                                                   TERMINATOR_FF             
0x00001BED      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001BEF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001BF1      1  FF                                                   TERMINATOR_FF             
0x00001BF2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001BF4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001BF6      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001BF8      1  FF                                                   TERMINATOR_FF             
0x00001BF9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001BFB      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001BFD      3  F31A0A                                               IMM16_F3                  u16_be=6666, u16_le=2586
0x00001C00      1  FF                                                   TERMINATOR_FF             
0x00001C01      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001C03      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001C05     76  804A82F18163816381485C6E82E082B582A982B582C48141...  LEN8_STRING_CP932         length=74, text="ん……？\nもしかして、昨日のこと忘れちゃった？\nぼくのこと、おぼえてない？"
0x00001C51      1  FF                                                   TERMINATOR_FF             
0x00001C52      1  FF                                                   TERMINATOR_FF             
0x00001C53      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001C55      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001C57      1  FF                                                   TERMINATOR_FF             
0x00001C58      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001C5A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001C5C      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001C5E      1  FF                                                   TERMINATOR_FF             
0x00001C5F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001C61      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001C63      3  F31A0B                                               IMM16_F3                  u16_be=6667, u16_le=2842
0x00001C66      1  FF                                                   TERMINATOR_FF             
0x00001C67      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001C69      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001C6B      9  8007495A3030333831                                   LEN8_STRING_CP932         length=7, text="IZ00381"
0x00001C74      1  FF                                                   TERMINATOR_FF             
0x00001C75      1  FF                                                   TERMINATOR_FF             
0x00001C76      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001C78      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001C7A     68  8042817582A082CC8163816382A682C182C681418E848163...  LEN8_STRING_CP932         length=66, text="「あの……えっと、私……手相とかモニター会員とか興味ないですから」"
0x00001CBE      1  FF                                                   TERMINATOR_FF             
0x00001CBF      1  FF                                                   TERMINATOR_FF             
0x00001CC0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001CC2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001CC4      1  FF                                                   TERMINATOR_FF             
0x00001CC5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001CC7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001CC9      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001CCB      1  FF                                                   TERMINATOR_FF             
0x00001CCC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001CCE      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001CD0      3  F31A0C                                               IMM16_F3                  u16_be=6668, u16_le=3098
0x00001CD3      1  FF                                                   TERMINATOR_FF             
0x00001CD4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001CD6      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001CD8      9  8007495A3030333832                                   LEN8_STRING_CP932         length=7, text="IZ00382"
0x00001CE1      1  FF                                                   TERMINATOR_FF             
0x00001CE2      1  FF                                                   TERMINATOR_FF             
0x00001CE3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001CE5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001CE7     84  8052817582A082C6894896D1957A926382C683828366838B...  LEN8_STRING_CP932         length=82, text="「あと羽毛布団とモデルの仕事と、それから家は仏教だし、バイトは禁止されてるし……」"
0x00001D3B      1  FF                                                   TERMINATOR_FF             
0x00001D3C      1  FF                                                   TERMINATOR_FF             
0x00001D3D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001D3F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001D41      1  FF                                                   TERMINATOR_FF             
0x00001D42      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001D44      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001D46      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001D48      1  FF                                                   TERMINATOR_FF             
0x00001D49      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001D4B      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001D4D      3  F31A0D                                               IMM16_F3                  u16_be=6669, u16_le=3354
0x00001D50      1  FF                                                   TERMINATOR_FF             
0x00001D51      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001D53      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001D55     62  803C82ED82C1814182A882DA82A682C482C882A282C782B1...  LEN8_STRING_CP932         length=60, text="わっ、おぼえてないどころか、警戒されてる。\n……仕方ないか。"
0x00001D93      1  FF                                                   TERMINATOR_FF             
0x00001D94      1  FF                                                   TERMINATOR_FF             
0x00001D95      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001D97      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001D99      1  FF                                                   TERMINATOR_FF             
0x00001D9A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001D9C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001D9E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001DA0      1  FF                                                   TERMINATOR_FF             
0x00001DA1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001DA3      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001DA5      3  F31A0E                                               IMM16_F3                  u16_be=6670, u16_le=3610
0x00001DA8      1  FF                                                   TERMINATOR_FF             
0x00001DA9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001DAB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001DAD     72  80468DF093FA82CD82B882A282D482F1835683878362834E...  LEN8_STRING_CP932         length=70, text="昨日はずいぶんショックうけてたし、そのことを恩にきせたいわけじゃない。"
0x00001DF5      1  FF                                                   TERMINATOR_FF             
0x00001DF6      1  FF                                                   TERMINATOR_FF             
0x00001DF7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001DF9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001DFB      1  FF                                                   TERMINATOR_FF             
0x00001DFC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001DFE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001E00      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001E02      1  FF                                                   TERMINATOR_FF             
0x00001E03      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001E05      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001E07      3  F31A0F                                               IMM16_F3                  u16_be=6671, u16_le=3866
0x00001E0A      1  FF                                                   TERMINATOR_FF             
0x00001E0B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001E0D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001E0F     56  803682BD82BE8141965982EA82E782EA82BF82E182C182C4...  LEN8_STRING_CP932         length=54, text="ただ、忘れられちゃってたのは、少し悲しい\nものがある。"
0x00001E47      1  FF                                                   TERMINATOR_FF             
0x00001E48      1  FF                                                   TERMINATOR_FF             
0x00001E49      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001E4B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001E4D      1  FF                                                   TERMINATOR_FF             
0x00001E4E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001E50      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001E52      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001E54      1  FF                                                   TERMINATOR_FF             
0x00001E55      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001E57      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001E59      3  F31A10                                               IMM16_F3                  u16_be=6672, u16_le=4122
0x00001E5C      1  FF                                                   TERMINATOR_FF             
0x00001E5D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001E5F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001E61     78  804C82C582E0814182BB82F182C882E082F182C882F182BE...  LEN8_STRING_CP932         length=76, text="でも、そんなもんなんだろーな。\n玉なんかだと、おぼえてる方が珍しいわけだし。"
0x00001EAF      1  FF                                                   TERMINATOR_FF             
0x00001EB0      1  FF                                                   TERMINATOR_FF             
0x00001EB1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001EB3      2  0004                                                 WORD_00XX                 u16_be=4, low_byte=4
0x00001EB5     30  801C82BF82E182F182C6926682E782C882AB82E1835F8381...  LEN8_STRING_CP932         length=28, text="ちゃんと断らなきゃダメですよ"
0x00001ED3      1  FF                                                   TERMINATOR_FF             
0x00001ED4      1  FF                                                   TERMINATOR_FF             
0x00001ED5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001ED7      2  000A                                                 WORD_00XX                 u16_be=10, low_byte=10
0x00001ED9      3  F31A11                                               IMM16_F3                  u16_be=6673, u16_le=4378
0x00001EDC      1  FF                                                   TERMINATOR_FF             
0x00001EDD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001EDF      2  0005                                                 WORD_00XX                 u16_be=5, low_byte=5
0x00001EE1     34  80208365834283628356838582C682A982E082E782A282B7...  LEN8_STRING_CP932         length=32, text="ティッシュとかもらいすぎません？"
0x00001F03      1  FF                                                   TERMINATOR_FF             
0x00001F04      1  FF                                                   TERMINATOR_FF             
0x00001F05      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F07      2  000B                                                 WORD_00XX                 u16_be=11, low_byte=11
0x00001F09      3  F31A12                                               IMM16_F3                  u16_be=6674, u16_le=4634
0x00001F0C      1  FF                                                   TERMINATOR_FF             
0x00001F0D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F0F      2  0008                                                 WORD_00XX                 u16_be=8, low_byte=8
0x00001F11      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00001F13      1  FF                                                   TERMINATOR_FF             
0x00001F14      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F16      2  0009                                                 WORD_00XX                 u16_be=9, low_byte=9
0x00001F18      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001F1A      1  FF                                                   TERMINATOR_FF             
0x00001F1B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F1D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F1F      2  F212                                                 IMM8_F2                   u8=18, s8=18
0x00001F21      1  FF                                                   TERMINATOR_FF             
0x00001F22      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001F24      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001F26      1  FF                                                   TERMINATOR_FF             
0x00001F27      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00001F29      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x00001F2C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001F2E      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00001F2F      1  FF                                                   TERMINATOR_FF             
0x00001F30      2  0020                                                 WORD_00XX                 u16_be=32, low_byte=32
0x00001F32      1  7B                                                   OPAQUE_RAW_BYTES          bytes=7B
0x00001F33      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F35      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F37      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00001F39      1  FF                                                   TERMINATOR_FF             
0x00001F3A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F3C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001F3E      3  F300C1                                               IMM16_F3                  u16_be=193, u16_le=49408
0x00001F41      1  FF                                                   TERMINATOR_FF             
0x00001F42      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F44      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001F46      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001F48      1  FF                                                   TERMINATOR_FF             
0x00001F49      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001F4B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001F4D      1  FF                                                   TERMINATOR_FF             
0x00001F4E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F50      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F52      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00001F54      1  FF                                                   TERMINATOR_FF             
0x00001F55      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F57      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001F59      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00001F5B      1  FF                                                   TERMINATOR_FF             
0x00001F5C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F5E      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001F60      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00001F62      1  FF                                                   TERMINATOR_FF             
0x00001F63      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001F65      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001F67      1  FF                                                   TERMINATOR_FF             
0x00001F68      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F6A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F6C      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00001F6E      1  FF                                                   TERMINATOR_FF             
0x00001F6F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001F71      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00001F73      3  F31A13                                               IMM16_F3                  u16_be=6675, u16_le=4890
0x00001F76      1  FF                                                   TERMINATOR_FF             
0x00001F77      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001F79      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00001F7B      9  8007495A3030333930                                   LEN8_STRING_CP932         length=7, text="IZ00390"
0x00001F84      1  FF                                                   TERMINATOR_FF             
0x00001F85      1  FF                                                   TERMINATOR_FF             
0x00001F86      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001F88      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00001F8A    101  8063817582A682C18148208163816382A682C182C6815B81...  LEN8_STRING_CP932         length=99, text="「えっ？ ……えっとー、そうなんですけどー、\nちょっと、悪いかなーとか思っちゃうじゃない\nですかー」"
0x00001FEF      1  FF                                                   TERMINATOR_FF             
0x00001FF0      1  FF                                                   TERMINATOR_FF             
0x00001FF1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00001FF3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00001FF5      1  FF                                                   TERMINATOR_FF             
0x00001FF6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001FF8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001FFA      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00001FFC      1  FF                                                   TERMINATOR_FF             
0x00001FFD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00001FFF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002001      3  F31A14                                               IMM16_F3                  u16_be=6676, u16_le=5146
0x00002004      1  FF                                                   TERMINATOR_FF             
0x00002005      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002007      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002009    102  806482A4815B82F18141906C82AA82A282A282C682E4815B...  LEN8_STRING_CP932         length=100, text="うーん、人がいいとゆーか、優しすぎるとゆーか。そーゆー態度って、勘違いされやすいんだろーな、きっと。"
0x0000206F      1  FF                                                   TERMINATOR_FF             
0x00002070      1  FF                                                   TERMINATOR_FF             
0x00002071      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002073      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002075      1  FF                                                   TERMINATOR_FF             
0x00002076      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00002078      2  0023                                                 WORD_00XX                 u16_be=35, low_byte=35
0x0000207A      1  4F                                                   OPAQUE_RAW_BYTES          bytes=4F
0x0000207B      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x0000207D      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x00002080      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00002082      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00002083      1  FF                                                   TERMINATOR_FF             
0x00002084      2  0023                                                 WORD_00XX                 u16_be=35, low_byte=35
0x00002086      1  4F                                                   OPAQUE_RAW_BYTES          bytes=4F
0x00002087      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002089      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000208B      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000208D      1  FF                                                   TERMINATOR_FF             
0x0000208E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002090      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002092      3  F300C4                                               IMM16_F3                  u16_be=196, u16_le=50176
0x00002095      1  FF                                                   TERMINATOR_FF             
0x00002096      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002098      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000209A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000209C      1  FF                                                   TERMINATOR_FF             
0x0000209D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000209F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000020A1      1  FF                                                   TERMINATOR_FF             
0x000020A2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000020A4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000020A6      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000020A8      1  FF                                                   TERMINATOR_FF             
0x000020A9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000020AB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000020AD      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000020AF      1  FF                                                   TERMINATOR_FF             
0x000020B0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000020B2      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000020B4      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000020B6      1  FF                                                   TERMINATOR_FF             
0x000020B7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000020B9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000020BB      1  FF                                                   TERMINATOR_FF             
0x000020BC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000020BE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000020C0      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000020C2      1  FF                                                   TERMINATOR_FF             
0x000020C3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000020C5      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000020C7      3  F31A15                                               IMM16_F3                  u16_be=6677, u16_le=5402
0x000020CA      1  FF                                                   TERMINATOR_FF             
0x000020CB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000020CD      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000020CF      9  8007495A3030343030                                   LEN8_STRING_CP932         length=7, text="IZ00400"
0x000020D8      1  FF                                                   TERMINATOR_FF             
0x000020D9      1  FF                                                   TERMINATOR_FF             
0x000020DA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000020DC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000020DE     99  8061817582BB815B82C882F182C582B782E682CB815B8142...  LEN8_STRING_CP932         length=97, text="「そーなんですよねー。すーぐいっぱいに\nなっちゃうしー…………えっ？ なんで知って\nるんですか?!」"
0x00002141      1  FF                                                   TERMINATOR_FF             
0x00002142      1  FF                                                   TERMINATOR_FF             
0x00002143      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002145      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002147      1  FF                                                   TERMINATOR_FF             
0x00002148      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000214A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000214C      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000214E      1  FF                                                   TERMINATOR_FF             
0x0000214F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002151      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002153      3  F31A16                                               IMM16_F3                  u16_be=6678, u16_le=5658
0x00002156      1  FF                                                   TERMINATOR_FF             
0x00002157      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002159      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000215B     40  80268ADB93A182B382F182CD814182DA82AD82C982A882D1...  LEN8_STRING_CP932         length=38, text="丸藤さんは、ぼくにおびえた目を向ける。"
0x00002183      1  FF                                                   TERMINATOR_FF             
0x00002184      1  FF                                                   TERMINATOR_FF             
0x00002185      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002187      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002189      1  FF                                                   TERMINATOR_FF             
0x0000218A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000218C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000218E      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00002190      1  FF                                                   TERMINATOR_FF             
0x00002191      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002193      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002195      3  F30215                                               IMM16_F3                  u16_be=533, u16_le=5378
0x00002198      1  FF                                                   TERMINATOR_FF             
0x00002199      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000219B      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000219D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000219F      1  FF                                                   TERMINATOR_FF             
0x000021A0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000021A2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000021A4      1  FF                                                   TERMINATOR_FF             
0x000021A5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000021A7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000021A9      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000021AB      1  FF                                                   TERMINATOR_FF             
0x000021AC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000021AE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000021B0      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000021B2      1  FF                                                   TERMINATOR_FF             
0x000021B3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000021B5      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000021B7      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000021B9      1  FF                                                   TERMINATOR_FF             
0x000021BA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000021BC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000021BE      1  FF                                                   TERMINATOR_FF             
0x000021BF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000021C1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000021C3      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000021C5      1  FF                                                   TERMINATOR_FF             
0x000021C6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000021C8      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000021CA      3  F31A17                                               IMM16_F3                  u16_be=6679, u16_le=5914
0x000021CD      1  FF                                                   TERMINATOR_FF             
0x000021CE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000021D0      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000021D2      9  8007495A3030343130                                   LEN8_STRING_CP932         length=7, text="IZ00410"
0x000021DB      1  FF                                                   TERMINATOR_FF             
0x000021DC      1  FF                                                   TERMINATOR_FF             
0x000021DD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000021DF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000021E1     38  802481758E8482CC82B182C68163816383588367815B834A...  LEN8_STRING_CP932         length=36, text="「私のこと……ストーカーしてます？」"
0x00002207      1  FF                                                   TERMINATOR_FF             
0x00002208      1  FF                                                   TERMINATOR_FF             
0x00002209      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000220B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000220D      1  FF                                                   TERMINATOR_FF             
0x0000220E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002210      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002212      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002214      1  FF                                                   TERMINATOR_FF             
0x00002215      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002217      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002219      3  F31A18                                               IMM16_F3                  u16_be=6680, u16_le=6170
0x0000221C      1  FF                                                   TERMINATOR_FF             
0x0000221D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000221F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002221     32  801E82A0816082A0814182B382C182AB82E682E888AB89BB...  LEN8_STRING_CP932         length=30, text="あ～あ、さっきより悪化してる。"
0x00002241      1  FF                                                   TERMINATOR_FF             
0x00002242      1  FF                                                   TERMINATOR_FF             
0x00002243      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002245      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002247      1  FF                                                   TERMINATOR_FF             
0x00002248      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000224A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000224C      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000224E      1  FF                                                   TERMINATOR_FF             
0x0000224F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002251      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002253      3  F31A19                                               IMM16_F3                  u16_be=6681, u16_le=6426
0x00002256      1  FF                                                   TERMINATOR_FF             
0x00002257      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002259      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000225B     16  800E817582B582C482DC82B982F18176                     LEN8_STRING_CP932         length=14, text="「してません」"
0x0000226B      1  FF                                                   TERMINATOR_FF             
0x0000226C      1  FF                                                   TERMINATOR_FF             
0x0000226D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000226F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002271      1  FF                                                   TERMINATOR_FF             
0x00002272      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002274      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002276      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00002278      1  FF                                                   TERMINATOR_FF             
0x00002279      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000227B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000227D      3  F300C4                                               IMM16_F3                  u16_be=196, u16_le=50176
0x00002280      1  FF                                                   TERMINATOR_FF             
0x00002281      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002283      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002285      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002287      1  FF                                                   TERMINATOR_FF             
0x00002288      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000228A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000228C      1  FF                                                   TERMINATOR_FF             
0x0000228D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000228F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002291      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00002293      1  FF                                                   TERMINATOR_FF             
0x00002294      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002296      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002298      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000229A      1  FF                                                   TERMINATOR_FF             
0x0000229B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000229D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000229F      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000022A1      1  FF                                                   TERMINATOR_FF             
0x000022A2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000022A4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000022A6      1  FF                                                   TERMINATOR_FF             
0x000022A7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000022A9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000022AB      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000022AD      1  FF                                                   TERMINATOR_FF             
0x000022AE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000022B0      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000022B2      3  F31A1A                                               IMM16_F3                  u16_be=6682, u16_le=6682
0x000022B5      1  FF                                                   TERMINATOR_FF             
0x000022B6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000022B8      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000022BA      9  8007495A3030343230                                   LEN8_STRING_CP932         length=7, text="IZ00420"
0x000022C3      1  FF                                                   TERMINATOR_FF             
0x000022C4      1  FF                                                   TERMINATOR_FF             
0x000022C5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000022C7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000022C9     20  8012817582BB815B82C582B782E682CB815B8176             LEN8_STRING_CP932         length=18, text="「そーですよねー」"
0x000022DD      1  FF                                                   TERMINATOR_FF             
0x000022DE      1  FF                                                   TERMINATOR_FF             
0x000022DF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000022E1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000022E3      1  FF                                                   TERMINATOR_FF             
0x000022E4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000022E6      2  0055                                                 WORD_00XX                 u16_be=85, low_byte=85
0x000022E8      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000022EA      1  FF                                                   TERMINATOR_FF             
0x000022EB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000022ED      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000022EF      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000022F1      1  FF                                                   TERMINATOR_FF             
0x000022F2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000022F4      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000022F6      3  F31A1B                                               IMM16_F3                  u16_be=6683, u16_le=6938
0x000022F9      1  FF                                                   TERMINATOR_FF             
0x000022FA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000022FC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000022FE     74  804888AB8B4382CD82C882A282F182BE82C8814182C68E76...  LEN8_STRING_CP932         length=72, text="悪気はないんだな、と思う。\nただ勘違いされるんだろーな、いろんな意味で。"
0x00002348      1  FF                                                   TERMINATOR_FF             
0x00002349      1  FF                                                   TERMINATOR_FF             
0x0000234A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000234C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000234E      1  FF                                                   TERMINATOR_FF             
0x0000234F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002351      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002353      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002355      1  FF                                                   TERMINATOR_FF             
0x00002356      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002358      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000235A      3  F31A1C                                               IMM16_F3                  u16_be=6684, u16_le=7194
0x0000235D      1  FF                                                   TERMINATOR_FF             
0x0000235E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002360      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002362     54  803482A082F182DC82E882B582C282B182AD82B582C482E9...  LEN8_STRING_CP932         length=52, text="あんまりしつこくしてると、ぼくまで誤解され\nそうだ。"
0x00002398      1  FF                                                   TERMINATOR_FF             
0x00002399      1  FF                                                   TERMINATOR_FF             
0x0000239A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000239C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000239E      1  FF                                                   TERMINATOR_FF             
0x0000239F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000023A1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000023A3      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000023A5      1  FF                                                   TERMINATOR_FF             
0x000023A6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000023A8      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000023AA      3  F31A1D                                               IMM16_F3                  u16_be=6685, u16_le=7450
0x000023AD      1  FF                                                   TERMINATOR_FF             
0x000023AE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000023B0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000023B2     28  801A817582B682E182A0814182DA82AD81418B4182E882DC...  LEN8_STRING_CP932         length=26, text="「じゃあ、ぼく、帰ります」"
0x000023CE      1  FF                                                   TERMINATOR_FF             
0x000023CF      1  FF                                                   TERMINATOR_FF             
0x000023D0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000023D2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000023D4      1  FF                                                   TERMINATOR_FF             
0x000023D5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000023D7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000023D9      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000023DB      1  FF                                                   TERMINATOR_FF             
0x000023DC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000023DE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000023E0      3  F300C2                                               IMM16_F3                  u16_be=194, u16_le=49664
0x000023E3      1  FF                                                   TERMINATOR_FF             
0x000023E4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000023E6      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000023E8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000023EA      1  FF                                                   TERMINATOR_FF             
0x000023EB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000023ED      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000023EF      1  FF                                                   TERMINATOR_FF             
0x000023F0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000023F2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000023F4      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000023F6      1  FF                                                   TERMINATOR_FF             
0x000023F7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000023F9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000023FB      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000023FD      1  FF                                                   TERMINATOR_FF             
0x000023FE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002400      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002402      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00002404      1  FF                                                   TERMINATOR_FF             
0x00002405      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002407      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002409      1  FF                                                   TERMINATOR_FF             
0x0000240A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000240C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000240E      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00002410      1  FF                                                   TERMINATOR_FF             
0x00002411      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002413      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002415      3  F31A1E                                               IMM16_F3                  u16_be=6686, u16_le=7706
0x00002418      1  FF                                                   TERMINATOR_FF             
0x00002419      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000241B      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000241D      9  8007495A3030343330                                   LEN8_STRING_CP932         length=7, text="IZ00430"
0x00002426      1  FF                                                   TERMINATOR_FF             
0x00002427      1  FF                                                   TERMINATOR_FF             
0x00002428      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000242A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000242C     76  804A817582A08163816382CD82A2814282BB82A482C582B7...  LEN8_STRING_CP932         length=74, text="「あ……はい。そうですか。えっと、助けて\nくれて、ありがとうございました」"
0x00002478      1  FF                                                   TERMINATOR_FF             
0x00002479      1  FF                                                   TERMINATOR_FF             
0x0000247A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000247C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000247E      1  FF                                                   TERMINATOR_FF             
0x0000247F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002481      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002483      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00002485      1  FF                                                   TERMINATOR_FF             
0x00002486      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002488      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000248A      2  F2FF                                                 IMM8_F2                   u8=255, s8=-1
0x0000248C      1  FF                                                   TERMINATOR_FF             
0x0000248D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000248F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002491      1  FF                                                   TERMINATOR_FF             
0x00002492      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002494      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002496      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00002498      1  FF                                                   TERMINATOR_FF             
0x00002499      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000249B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000249D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000249F      1  FF                                                   TERMINATOR_FF             
0x000024A0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024A2      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000024A4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000024A6      1  FF                                                   TERMINATOR_FF             
0x000024A7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000024A9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000024AB      1  FF                                                   TERMINATOR_FF             
0x000024AC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024AE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024B0      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000024B2      1  FF                                                   TERMINATOR_FF             
0x000024B3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024B5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000024B7      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000024B9      1  FF                                                   TERMINATOR_FF             
0x000024BA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024BC      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000024BE      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000024C0      1  FF                                                   TERMINATOR_FF             
0x000024C1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000024C3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000024C5      1  FF                                                   TERMINATOR_FF             
0x000024C6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024C8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024CA      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x000024CC      1  FF                                                   TERMINATOR_FF             
0x000024CD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024CF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000024D1      2  F264                                                 IMM8_F2                   u8=100, s8=100
0x000024D3      1  FF                                                   TERMINATOR_FF             
0x000024D4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000024D6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000024D8      1  FF                                                   TERMINATOR_FF             
0x000024D9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024DB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024DD      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000024DF      1  FF                                                   TERMINATOR_FF             
0x000024E0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024E2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000024E4      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000024E6      1  FF                                                   TERMINATOR_FF             
0x000024E7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024E9      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000024EB      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000024ED      1  FF                                                   TERMINATOR_FF             
0x000024EE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000024F0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000024F2      1  FF                                                   TERMINATOR_FF             
0x000024F3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024F5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024F7      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000024F9      1  FF                                                   TERMINATOR_FF             
0x000024FA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000024FC      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000024FE      3  F31A1F                                               IMM16_F3                  u16_be=6687, u16_le=7962
0x00002501      1  FF                                                   TERMINATOR_FF             
0x00002502      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002504      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002506    122  807891E5906C82C682CD8E7682A682C882A282AD82E782A2...  LEN8_STRING_CP932         length=120, text="大人とは思えないくらい、おっとりした人だなあ。そんなことを考えていながら歩きだそうとした\nとき、後ろから呼びとめられた。"
0x00002580      1  FF                                                   TERMINATOR_FF             
0x00002581      1  FF                                                   TERMINATOR_FF             
0x00002582      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002584      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002586      1  FF                                                   TERMINATOR_FF             
0x00002587      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002589      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000258B      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000258D      1  FF                                                   TERMINATOR_FF             
0x0000258E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002590      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002592      3  F31A20                                               IMM16_F3                  u16_be=6688, u16_le=8218
0x00002595      1  FF                                                   TERMINATOR_FF             
0x00002596      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002598      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000259A      9  8007495A3030343430                                   LEN8_STRING_CP932         length=7, text="IZ00440"
0x000025A3      1  FF                                                   TERMINATOR_FF             
0x000025A4      1  FF                                                   TERMINATOR_FF             
0x000025A5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000025A7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000025A9     34  8020817582A082CC815B8163816382BF82E582C182C68141...  LEN8_STRING_CP932         length=32, text="「あのー……ちょっと、待って!!」"
0x000025CB      1  FF                                                   TERMINATOR_FF             
0x000025CC      1  FF                                                   TERMINATOR_FF             
0x000025CD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000025CF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000025D1      1  FF                                                   TERMINATOR_FF             
0x000025D2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000025D4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000025D6      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000025D8      1  FF                                                   TERMINATOR_FF             
0x000025D9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000025DB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000025DD      3  F300CE                                               IMM16_F3                  u16_be=206, u16_le=52736
0x000025E0      1  FF                                                   TERMINATOR_FF             
0x000025E1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000025E3      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000025E5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000025E7      1  FF                                                   TERMINATOR_FF             
0x000025E8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000025EA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000025EC      1  FF                                                   TERMINATOR_FF             
0x000025ED      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000025EF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000025F1      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000025F3      1  FF                                                   TERMINATOR_FF             
0x000025F4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000025F6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000025F8      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000025FA      1  FF                                                   TERMINATOR_FF             
0x000025FB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000025FD      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000025FF      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00002601      1  FF                                                   TERMINATOR_FF             
0x00002602      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002604      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002606      1  FF                                                   TERMINATOR_FF             
0x00002607      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002609      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000260B      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000260D      1  FF                                                   TERMINATOR_FF             
0x0000260E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002610      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002612      3  F31A21                                               IMM16_F3                  u16_be=6689, u16_le=8474
0x00002615      1  FF                                                   TERMINATOR_FF             
0x00002616      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002618      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000261A      9  8007495A3030343530                                   LEN8_STRING_CP932         length=7, text="IZ00450"
0x00002623      1  FF                                                   TERMINATOR_FF             
0x00002624      1  FF                                                   TERMINATOR_FF             
0x00002625      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002627      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002629     72  8046817582CB82A6814182AB82DD8163816382C7815B82B5...  LEN8_STRING_CP932         length=70, text="「ねえ、きみ……どーして私の名前知ってるの？どっかで会ったことある？」"
0x00002671      1  FF                                                   TERMINATOR_FF             
0x00002672      1  FF                                                   TERMINATOR_FF             
0x00002673      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002675      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002677      1  FF                                                   TERMINATOR_FF             
0x00002678      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000267A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000267C      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000267E      1  FF                                                   TERMINATOR_FF             
0x0000267F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002681      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002683      3  F31A22                                               IMM16_F3                  u16_be=6690, u16_le=8730
0x00002686      1  FF                                                   TERMINATOR_FF             
0x00002687      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002689      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000268B     56  803682BB82A482A282C182C48ADB93A182B382F182CD8141...  LEN8_STRING_CP932         length=54, text="そういって丸藤さんは、ぼくの顔をまじまじと\n見つめる。"
0x000026C3      1  FF                                                   TERMINATOR_FF             
0x000026C4      1  FF                                                   TERMINATOR_FF             
0x000026C5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000026C7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000026C9      1  FF                                                   TERMINATOR_FF             
0x000026CA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000026CC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000026CE      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000026D0      1  FF                                                   TERMINATOR_FF             
0x000026D1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000026D3      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000026D5      3  F31A23                                               IMM16_F3                  u16_be=6691, u16_le=8986
0x000026D8      1  FF                                                   TERMINATOR_FF             
0x000026D9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000026DB      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000026DD      9  8007495A3030343630                                   LEN8_STRING_CP932         length=7, text="IZ00460"
0x000026E6      1  FF                                                   TERMINATOR_FF             
0x000026E7      1  FF                                                   TERMINATOR_FF             
0x000026E8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000026EA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000026EC     76  804A817582B282DF82F182CB815B81418DA193FA83528393...  LEN8_STRING_CP932         length=74, text="「ごめんねー、今日コンタクト忘れちゃったから、あんまりよく見えないのよー」"
0x00002738      1  FF                                                   TERMINATOR_FF             
0x00002739      1  FF                                                   TERMINATOR_FF             
0x0000273A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000273C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000273E      1  FF                                                   TERMINATOR_FF             
0x0000273F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002741      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002743      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00002745      1  FF                                                   TERMINATOR_FF             
0x00002746      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002748      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000274A      3  F300C0                                               IMM16_F3                  u16_be=192, u16_le=49152
0x0000274D      1  FF                                                   TERMINATOR_FF             
0x0000274E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002750      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002752      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002754      1  FF                                                   TERMINATOR_FF             
0x00002755      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002757      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002759      1  FF                                                   TERMINATOR_FF             
0x0000275A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000275C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000275E      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00002760      1  FF                                                   TERMINATOR_FF             
0x00002761      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002763      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002765      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00002767      1  FF                                                   TERMINATOR_FF             
0x00002768      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000276A      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000276C      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000276E      1  FF                                                   TERMINATOR_FF             
0x0000276F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002771      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002773      1  FF                                                   TERMINATOR_FF             
0x00002774      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002776      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002778      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000277A      1  FF                                                   TERMINATOR_FF             
0x0000277B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000277D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000277F      3  F31A24                                               IMM16_F3                  u16_be=6692, u16_le=9242
0x00002782      1  FF                                                   TERMINATOR_FF             
0x00002783      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002785      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002787      9  8007495A3030343730                                   LEN8_STRING_CP932         length=7, text="IZ00470"
0x00002790      1  FF                                                   TERMINATOR_FF             
0x00002791      1  FF                                                   TERMINATOR_FF             
0x00002792      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002794      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002796     25  8017817582A082C13F21208DF093FA82CC8E7182BE212181...  LEN8_STRING_CP932         length=23, text="「あっ?! 昨日の子だ!!」"
0x000027AF      1  FF                                                   TERMINATOR_FF             
0x000027B0      1  FF                                                   TERMINATOR_FF             
0x000027B1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000027B3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000027B5      1  FF                                                   TERMINATOR_FF             
0x000027B6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000027B8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000027BA      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000027BC      1  FF                                                   TERMINATOR_FF             
0x000027BD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000027BF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000027C1      3  F300C6                                               IMM16_F3                  u16_be=198, u16_le=50688
0x000027C4      1  FF                                                   TERMINATOR_FF             
0x000027C5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000027C7      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000027C9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000027CB      1  FF                                                   TERMINATOR_FF             
0x000027CC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000027CE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000027D0      1  FF                                                   TERMINATOR_FF             
0x000027D1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000027D3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000027D5      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000027D7      1  FF                                                   TERMINATOR_FF             
0x000027D8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000027DA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000027DC      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000027DE      1  FF                                                   TERMINATOR_FF             
0x000027DF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000027E1      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000027E3      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000027E5      1  FF                                                   TERMINATOR_FF             
0x000027E6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000027E8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000027EA      1  FF                                                   TERMINATOR_FF             
0x000027EB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000027ED      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000027EF      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000027F1      1  FF                                                   TERMINATOR_FF             
0x000027F2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000027F4      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000027F6      3  F31A25                                               IMM16_F3                  u16_be=6693, u16_le=9498
0x000027F9      1  FF                                                   TERMINATOR_FF             
0x000027FA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000027FC      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000027FE      9  8007495A3030343830                                   LEN8_STRING_CP932         length=7, text="IZ00480"
0x00002807      1  FF                                                   TERMINATOR_FF             
0x00002808      1  FF                                                   TERMINATOR_FF             
0x00002809      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000280B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000280D    112  806E8175835F838182E6835F8381835F83812121208B4182...  LEN8_STRING_CP932         length=110, text="「ダメよダメダメ!! 帰っちゃダメ!!\nえっ?! 私２回も助けてもらっちゃったの？\nそんなのダメよ、お礼しなくちゃ!!」"
0x0000287D      1  FF                                                   TERMINATOR_FF             
0x0000287E      1  FF                                                   TERMINATOR_FF             
0x0000287F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002881      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002883      1  FF                                                   TERMINATOR_FF             
0x00002884      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002886      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002888      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000288A      1  FF                                                   TERMINATOR_FF             
0x0000288B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000288D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000288F      3  F31A26                                               IMM16_F3                  u16_be=6694, u16_le=9754
0x00002892      1  FF                                                   TERMINATOR_FF             
0x00002893      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002895      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002897     40  802682ED8141906C82AA95CF82ED82C182BD82E682A482C8...  LEN8_STRING_CP932         length=38, text="わ、人が変わったようなハイテンション。"
0x000028BF      1  FF                                                   TERMINATOR_FF             
0x000028C0      1  FF                                                   TERMINATOR_FF             
0x000028C1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000028C3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000028C5      1  FF                                                   TERMINATOR_FF             
0x000028C6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000028C8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000028CA      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000028CC      1  FF                                                   TERMINATOR_FF             
0x000028CD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000028CF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000028D1      3  F300D0                                               IMM16_F3                  u16_be=208, u16_le=53248
0x000028D4      1  FF                                                   TERMINATOR_FF             
0x000028D5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000028D7      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000028D9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000028DB      1  FF                                                   TERMINATOR_FF             
0x000028DC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000028DE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000028E0      1  FF                                                   TERMINATOR_FF             
0x000028E1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000028E3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000028E5      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000028E7      1  FF                                                   TERMINATOR_FF             
0x000028E8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000028EA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000028EC      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000028EE      1  FF                                                   TERMINATOR_FF             
0x000028EF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000028F1      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000028F3      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000028F5      1  FF                                                   TERMINATOR_FF             
0x000028F6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000028F8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000028FA      1  FF                                                   TERMINATOR_FF             
0x000028FB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000028FD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000028FF      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00002901      1  FF                                                   TERMINATOR_FF             
0x00002902      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002904      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002906      3  F31A27                                               IMM16_F3                  u16_be=6695, u16_le=10010
0x00002909      1  FF                                                   TERMINATOR_FF             
0x0000290A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000290C      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000290E      9  8007495A3030343931                                   LEN8_STRING_CP932         length=7, text="IZ00491"
0x00002917      1  FF                                                   TERMINATOR_FF             
0x00002918      1  FF                                                   TERMINATOR_FF             
0x00002919      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000291B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000291D     88  8056817582B782C182B282AD83588365834C82C882A89358...  LEN8_STRING_CP932         length=86, text="「すっごくステキなお店があるの。ね、ね、\nいいでしょ。ちょっとでいーから、つきあって」"
0x00002975      1  FF                                                   TERMINATOR_FF             
0x00002976      1  FF                                                   TERMINATOR_FF             
0x00002977      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002979      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000297B      1  FF                                                   TERMINATOR_FF             
0x0000297C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000297E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002980      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00002982      1  FF                                                   TERMINATOR_FF             
0x00002983      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002985      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002987      3  F31A28                                               IMM16_F3                  u16_be=6696, u16_le=10266
0x0000298A      1  FF                                                   TERMINATOR_FF             
0x0000298B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000298D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000298F      9  8007495A3030343932                                   LEN8_STRING_CP932         length=7, text="IZ00492"
0x00002998      1  FF                                                   TERMINATOR_FF             
0x00002999      1  FF                                                   TERMINATOR_FF             
0x0000299A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000299C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000299E     78  804C817582BE82C182C4814182D982E781418E84814182A0...  LEN8_STRING_CP932         length=76, text="「だって、ほら、私、あやしくないしー。ねー、ホントにちょっとでいーんだから」"
0x000029EC      1  FF                                                   TERMINATOR_FF             
0x000029ED      1  FF                                                   TERMINATOR_FF             
0x000029EE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000029F0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000029F2      1  FF                                                   TERMINATOR_FF             
0x000029F3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000029F5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000029F7      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000029F9      1  FF                                                   TERMINATOR_FF             
0x000029FA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000029FC      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000029FE      3  F31A29                                               IMM16_F3                  u16_be=6697, u16_le=10522
0x00002A01      1  FF                                                   TERMINATOR_FF             
0x00002A02      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002A04      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002A06     72  80468B4382C382A282C482C882A282F182BE82EB82A482AF...  LEN8_STRING_CP932         length=70, text="気づいてないんだろうけど……\nキャッチの兄ちゃんと同じこといってるし。"
0x00002A4E      1  FF                                                   TERMINATOR_FF             
0x00002A4F      1  FF                                                   TERMINATOR_FF             
0x00002A50      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002A52      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002A54      1  FF                                                   TERMINATOR_FF             
0x00002A55      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002A57      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002A59      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002A5B      1  FF                                                   TERMINATOR_FF             
0x00002A5C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002A5E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002A60      3  F31A2A                                               IMM16_F3                  u16_be=6698, u16_le=10778
0x00002A63      1  FF                                                   TERMINATOR_FF             
0x00002A64      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002A66      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002A68     74  8048817582BE82A982E782CB815B814182C882C982A982B2...  LEN8_STRING_CP932         length=72, text="「だからねー、なにかごちそーしたいのよー。\n助けてもらった恩返しだしー」"
0x00002AB2      1  FF                                                   TERMINATOR_FF             
0x00002AB3      1  FF                                                   TERMINATOR_FF             
0x00002AB4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002AB6      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002AB8      9  8007495A3030353030                                   LEN8_STRING_CP932         length=7, text="IZ00500"
0x00002AC1      1  FF                                                   TERMINATOR_FF             
0x00002AC2      1  FF                                                   TERMINATOR_FF             
0x00002AC3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002AC5      2  0004                                                 WORD_00XX                 u16_be=4, low_byte=4
0x00002AC7     28  801A82BB82EA82D982C782CC82B182C682B682E182C882A2...  LEN8_STRING_CP932         length=26, text="それほどのことじゃないから"
0x00002AE3      1  FF                                                   TERMINATOR_FF             
0x00002AE4      1  FF                                                   TERMINATOR_FF             
0x00002AE5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002AE7      2  000A                                                 WORD_00XX                 u16_be=10, low_byte=10
0x00002AE9      3  F31A2B                                               IMM16_F3                  u16_be=6699, u16_le=11034
0x00002AEC      1  FF                                                   TERMINATOR_FF             
0x00002AED      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002AEF      2  0005                                                 WORD_00XX                 u16_be=5, low_byte=5
0x00002AF1     14  800C82A08163816382C782A482E0                         LEN8_STRING_CP932         length=12, text="あ……どうも"
0x00002AFF      1  FF                                                   TERMINATOR_FF             
0x00002B00      1  FF                                                   TERMINATOR_FF             
0x00002B01      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002B03      2  000B                                                 WORD_00XX                 u16_be=11, low_byte=11
0x00002B05      3  F31A2C                                               IMM16_F3                  u16_be=6700, u16_le=11290
0x00002B08      1  FF                                                   TERMINATOR_FF             
0x00002B09      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002B0B      2  0008                                                 WORD_00XX                 u16_be=8, low_byte=8
0x00002B0D      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00002B0F      1  FF                                                   TERMINATOR_FF             
0x00002B10      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002B12      2  0009                                                 WORD_00XX                 u16_be=9, low_byte=9
0x00002B14      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002B16      1  FF                                                   TERMINATOR_FF             
0x00002B17      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002B19      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002B1B      2  F213                                                 IMM8_F2                   u8=19, s8=19
0x00002B1D      1  FF                                                   TERMINATOR_FF             
0x00002B1E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002B20      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002B22      1  FF                                                   TERMINATOR_FF             
0x00002B23      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00002B25      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x00002B28      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002B2A      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00002B2B      1  FF                                                   TERMINATOR_FF             
0x00002B2C      2  002C                                                 WORD_00XX                 u16_be=44, low_byte=44
0x00002B2E      1  E9                                                   OPAQUE_RAW_BYTES          bytes=E9
0x00002B2F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002B31      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002B33      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00002B35      1  FF                                                   TERMINATOR_FF             
0x00002B36      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002B38      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002B3A      3  F300CA                                               IMM16_F3                  u16_be=202, u16_le=51712
0x00002B3D      1  FF                                                   TERMINATOR_FF             
0x00002B3E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002B40      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002B42      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002B44      1  FF                                                   TERMINATOR_FF             
0x00002B45      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002B47      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002B49      1  FF                                                   TERMINATOR_FF             
0x00002B4A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002B4C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002B4E      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00002B50      1  FF                                                   TERMINATOR_FF             
0x00002B51      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002B53      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002B55      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00002B57      1  FF                                                   TERMINATOR_FF             
0x00002B58      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002B5A      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002B5C      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00002B5E      1  FF                                                   TERMINATOR_FF             
0x00002B5F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002B61      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002B63      1  FF                                                   TERMINATOR_FF             
0x00002B64      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002B66      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002B68      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00002B6A      1  FF                                                   TERMINATOR_FF             
0x00002B6B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002B6D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002B6F      3  F31A2D                                               IMM16_F3                  u16_be=6701, u16_le=11546
0x00002B72      1  FF                                                   TERMINATOR_FF             
0x00002B73      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002B75      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002B77      9  8007495A3030353131                                   LEN8_STRING_CP932         length=7, text="IZ00511"
0x00002B80      1  FF                                                   TERMINATOR_FF             
0x00002B81      1  FF                                                   TERMINATOR_FF             
0x00002B82      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002B84      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002B86     30  801C817582A68160814182BB82F182C882B182C682C8815B...  LEN8_STRING_CP932         length=28, text="「え～、そんなことなーいー」"
0x00002BA4      1  FF                                                   TERMINATOR_FF             
0x00002BA5      1  FF                                                   TERMINATOR_FF             
0x00002BA6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002BA8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002BAA      1  FF                                                   TERMINATOR_FF             
0x00002BAB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002BAD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002BAF      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00002BB1      1  FF                                                   TERMINATOR_FF             
0x00002BB2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002BB4      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002BB6      3  F31A2E                                               IMM16_F3                  u16_be=6702, u16_le=11802
0x00002BB9      1  FF                                                   TERMINATOR_FF             
0x00002BBA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002BBC      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002BBE      9  8007495A3030353132                                   LEN8_STRING_CP932         length=7, text="IZ00512"
0x00002BC7      1  FF                                                   TERMINATOR_FF             
0x00002BC8      1  FF                                                   TERMINATOR_FF             
0x00002BC9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002BCB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002BCD    122  8078817582BE82C182C48DA282C182C482C482E082DD82F1...  LEN8_STRING_CP932         length=120, text="「だって困っててもみんな知らんぷりなんだよー。きみだけだもーん、助けてくれたのー。それって、すごーくリッパだと思うなー」"
0x00002C47      1  FF                                                   TERMINATOR_FF             
0x00002C48      1  FF                                                   TERMINATOR_FF             
0x00002C49      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002C4B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002C4D      1  FF                                                   TERMINATOR_FF             
0x00002C4E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002C50      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002C52      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002C54      1  FF                                                   TERMINATOR_FF             
0x00002C55      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002C57      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002C59      3  F31A2F                                               IMM16_F3                  u16_be=6703, u16_le=12058
0x00002C5C      1  FF                                                   TERMINATOR_FF             
0x00002C5D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002C5F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002C61    124  807A8ADB93A182B382F182CD82B182C782E082CC82E682A4...  LEN8_STRING_CP932         length=122, text="丸藤さんはこどものようにはしゃいでる。\nそんなつもりはなかったんだけど、こんなに\n喜ばれると、こっちもうれしくなってくる。"
0x00002CDD      1  FF                                                   TERMINATOR_FF             
0x00002CDE      1  FF                                                   TERMINATOR_FF             
0x00002CDF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002CE1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002CE3      1  FF                                                   TERMINATOR_FF             
0x00002CE4      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00002CE6      2  002F                                                 WORD_00XX                 u16_be=47, low_byte=47
0x00002CE8      1  17                                                   OPAQUE_RAW_BYTES          bytes=17
0x00002CE9      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00002CEB      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x00002CEE      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00002CF0      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00002CF1      1  FF                                                   TERMINATOR_FF             
0x00002CF2      2  002F                                                 WORD_00XX                 u16_be=47, low_byte=47
0x00002CF4      1  17                                                   OPAQUE_RAW_BYTES          bytes=17
0x00002CF5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002CF7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002CF9      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00002CFB      1  FF                                                   TERMINATOR_FF             
0x00002CFC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002CFE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002D00      3  F300C2                                               IMM16_F3                  u16_be=194, u16_le=49664
0x00002D03      1  FF                                                   TERMINATOR_FF             
0x00002D04      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002D06      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002D08      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002D0A      1  FF                                                   TERMINATOR_FF             
0x00002D0B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002D0D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002D0F      1  FF                                                   TERMINATOR_FF             
0x00002D10      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002D12      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002D14      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00002D16      1  FF                                                   TERMINATOR_FF             
0x00002D17      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002D19      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002D1B      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00002D1D      1  FF                                                   TERMINATOR_FF             
0x00002D1E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002D20      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002D22      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00002D24      1  FF                                                   TERMINATOR_FF             
0x00002D25      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002D27      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002D29      1  FF                                                   TERMINATOR_FF             
0x00002D2A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002D2C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002D2E      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00002D30      1  FF                                                   TERMINATOR_FF             
0x00002D31      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002D33      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002D35      3  F31A30                                               IMM16_F3                  u16_be=6704, u16_le=12314
0x00002D38      1  FF                                                   TERMINATOR_FF             
0x00002D39      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002D3B      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002D3D      9  8007495A3030353230                                   LEN8_STRING_CP932         length=7, text="IZ00520"
0x00002D46      1  FF                                                   TERMINATOR_FF             
0x00002D47      1  FF                                                   TERMINATOR_FF             
0x00002D48      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002D4A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002D4C     68  8042817582E682A982C182BD815B814282B182C682ED82E7...  LEN8_STRING_CP932         length=66, text="「よかったー。ことわられちゃったら、\nどーしよーって思ってたのー」"
0x00002D90      1  FF                                                   TERMINATOR_FF             
0x00002D91      1  FF                                                   TERMINATOR_FF             
0x00002D92      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002D94      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002D96      1  FF                                                   TERMINATOR_FF             
0x00002D97      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002D99      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002D9B      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00002D9D      1  FF                                                   TERMINATOR_FF             
0x00002D9E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002DA0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002DA2      3  F300BD                                               IMM16_F3                  u16_be=189, u16_le=48384
0x00002DA5      1  FF                                                   TERMINATOR_FF             
0x00002DA6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002DA8      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002DAA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002DAC      1  FF                                                   TERMINATOR_FF             
0x00002DAD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002DAF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002DB1      1  FF                                                   TERMINATOR_FF             
0x00002DB2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002DB4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002DB6      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00002DB8      1  FF                                                   TERMINATOR_FF             
0x00002DB9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002DBB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002DBD      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00002DBF      1  FF                                                   TERMINATOR_FF             
0x00002DC0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002DC2      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002DC4      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00002DC6      1  FF                                                   TERMINATOR_FF             
0x00002DC7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002DC9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002DCB      1  FF                                                   TERMINATOR_FF             
0x00002DCC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002DCE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002DD0      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00002DD2      1  FF                                                   TERMINATOR_FF             
0x00002DD3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002DD5      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002DD7      3  F31A31                                               IMM16_F3                  u16_be=6705, u16_le=12570
0x00002DDA      1  FF                                                   TERMINATOR_FF             
0x00002DDB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002DDD      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002DDF      9  8007495A3030353330                                   LEN8_STRING_CP932         length=7, text="IZ00530"
0x00002DE8      1  FF                                                   TERMINATOR_FF             
0x00002DE9      1  FF                                                   TERMINATOR_FF             
0x00002DEA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002DEC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002DEE    122  8078817582A08163816382C582E0814182AB82DD8D828D5A...  LEN8_STRING_CP932         length=120, text="「あ……でも、きみ高校生？ 門限とかある？\nお家、きびしー？ でも大丈夫よ、おねーさん、\nちゃ～んと家の人に説明するから」"
0x00002E68      1  FF                                                   TERMINATOR_FF             
0x00002E69      1  FF                                                   TERMINATOR_FF             
0x00002E6A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002E6C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002E6E      1  FF                                                   TERMINATOR_FF             
0x00002E6F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002E71      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002E73      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002E75      1  FF                                                   TERMINATOR_FF             
0x00002E76      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002E78      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002E7A      3  F31A32                                               IMM16_F3                  u16_be=6706, u16_le=12826
0x00002E7D      1  FF                                                   TERMINATOR_FF             
0x00002E7E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002E80      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002E82     60  803A82A082A08141918A8EE882CC8B438E9D82BF82F08D6C...  LEN8_STRING_CP932         length=58, text="ああ、相手の気持ちを考えすぎて、ことわれなくなってしまう。"
0x00002EBE      1  FF                                                   TERMINATOR_FF             
0x00002EBF      1  FF                                                   TERMINATOR_FF             
0x00002EC0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002EC2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002EC4      1  FF                                                   TERMINATOR_FF             
0x00002EC5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002EC7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002EC9      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002ECB      1  FF                                                   TERMINATOR_FF             
0x00002ECC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002ECE      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002ED0      3  F31A33                                               IMM16_F3                  u16_be=6707, u16_le=13082
0x00002ED3      1  FF                                                   TERMINATOR_FF             
0x00002ED4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002ED6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002ED8     56  80368ADB93A182B382F182CD814182BF82E582C182C682E2...  LEN8_STRING_CP932         length=54, text="丸藤さんは、ちょっとやさしすぎる人なのかも\nしれない。"
0x00002F10      1  FF                                                   TERMINATOR_FF             
0x00002F11      1  FF                                                   TERMINATOR_FF             
0x00002F12      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002F14      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002F16      1  FF                                                   TERMINATOR_FF             
0x00002F17      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002F19      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002F1B      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00002F1D      1  FF                                                   TERMINATOR_FF             
0x00002F1E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002F20      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002F22      3  F300CA                                               IMM16_F3                  u16_be=202, u16_le=51712
0x00002F25      1  FF                                                   TERMINATOR_FF             
0x00002F26      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002F28      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002F2A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002F2C      1  FF                                                   TERMINATOR_FF             
0x00002F2D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002F2F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002F31      1  FF                                                   TERMINATOR_FF             
0x00002F32      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002F34      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002F36      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00002F38      1  FF                                                   TERMINATOR_FF             
0x00002F39      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002F3B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002F3D      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00002F3F      1  FF                                                   TERMINATOR_FF             
0x00002F40      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002F42      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00002F44      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00002F46      1  FF                                                   TERMINATOR_FF             
0x00002F47      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002F49      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002F4B      1  FF                                                   TERMINATOR_FF             
0x00002F4C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002F4E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002F50      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002F52      1  FF                                                   TERMINATOR_FF             
0x00002F53      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002F55      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002F57      3  F31A34                                               IMM16_F3                  u16_be=6708, u16_le=13338
0x00002F5A      1  FF                                                   TERMINATOR_FF             
0x00002F5B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002F5D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002F5F     88  805682DA82AD82AA88EA8F8F82C98D7382AD82E082CC82C6...  LEN8_STRING_CP932         length=86, text="ぼくが一緒に行くものと信じきっている丸藤さんの笑顔を見ていると、ＮＯとはいえなかった。"
0x00002FB7      1  FF                                                   TERMINATOR_FF             
0x00002FB8      1  FF                                                   TERMINATOR_FF             
0x00002FB9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002FBB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002FBD      1  FF                                                   TERMINATOR_FF             
0x00002FBE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002FC0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002FC2      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00002FC4      1  FF                                                   TERMINATOR_FF             
0x00002FC5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002FC7      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00002FC9      3  F31A35                                               IMM16_F3                  u16_be=6709, u16_le=13594
0x00002FCC      1  FF                                                   TERMINATOR_FF             
0x00002FCD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002FCF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00002FD1     36  8022817582B682E18163816382A882B182C682CE82C982A0...  LEN8_STRING_CP932         length=34, text="「じゃ……おことばにあまえて……」"
0x00002FF5      1  FF                                                   TERMINATOR_FF             
0x00002FF6      1  FF                                                   TERMINATOR_FF             
0x00002FF7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00002FF9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00002FFB      1  FF                                                   TERMINATOR_FF             
0x00002FFC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00002FFE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003000      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00003002      1  FF                                                   TERMINATOR_FF             
0x00003003      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003005      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003007      3  F300D0                                               IMM16_F3                  u16_be=208, u16_le=53248
0x0000300A      1  FF                                                   TERMINATOR_FF             
0x0000300B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000300D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000300F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003011      1  FF                                                   TERMINATOR_FF             
0x00003012      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003014      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003016      1  FF                                                   TERMINATOR_FF             
0x00003017      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003019      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000301B      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000301D      1  FF                                                   TERMINATOR_FF             
0x0000301E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003020      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003022      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00003024      1  FF                                                   TERMINATOR_FF             
0x00003025      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003027      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003029      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000302B      1  FF                                                   TERMINATOR_FF             
0x0000302C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000302E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003030      1  FF                                                   TERMINATOR_FF             
0x00003031      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003033      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003035      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00003037      1  FF                                                   TERMINATOR_FF             
0x00003038      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000303A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000303C      3  F31A36                                               IMM16_F3                  u16_be=6710, u16_le=13850
0x0000303F      1  FF                                                   TERMINATOR_FF             
0x00003040      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003042      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003044      9  8007495A3030353430                                   LEN8_STRING_CP932         length=7, text="IZ00540"
0x0000304D      1  FF                                                   TERMINATOR_FF             
0x0000304E      1  FF                                                   TERMINATOR_FF             
0x0000304F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003051      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003053     82  8050817582E682A982C182BD82A08160814282A082CC82CB...  LEN8_STRING_CP932         length=80, text="「よかったあ～。あのね、すぐ近くなの。今もね、そのお店行こうとしてたんだけど…」"
0x000030A5      1  FF                                                   TERMINATOR_FF             
0x000030A6      1  FF                                                   TERMINATOR_FF             
0x000030A7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000030A9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000030AB      1  FF                                                   TERMINATOR_FF             
0x000030AC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000030AE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000030B0      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000030B2      1  FF                                                   TERMINATOR_FF             
0x000030B3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000030B5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000030B7      3  F3020B                                               IMM16_F3                  u16_be=523, u16_le=2818
0x000030BA      1  FF                                                   TERMINATOR_FF             
0x000030BB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000030BD      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000030BF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000030C1      1  FF                                                   TERMINATOR_FF             
0x000030C2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000030C4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000030C6      1  FF                                                   TERMINATOR_FF             
0x000030C7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000030C9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000030CB      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000030CD      1  FF                                                   TERMINATOR_FF             
0x000030CE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000030D0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000030D2      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000030D4      1  FF                                                   TERMINATOR_FF             
0x000030D5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000030D7      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000030D9      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000030DB      1  FF                                                   TERMINATOR_FF             
0x000030DC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000030DE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000030E0      1  FF                                                   TERMINATOR_FF             
0x000030E1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000030E3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000030E5      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000030E7      1  FF                                                   TERMINATOR_FF             
0x000030E8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000030EA      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000030EC      3  F31A37                                               IMM16_F3                  u16_be=6711, u16_le=14106
0x000030EF      1  FF                                                   TERMINATOR_FF             
0x000030F0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000030F2      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000030F4      9  8007495A3030353530                                   LEN8_STRING_CP932         length=7, text="IZ00550"
0x000030FD      1  FF                                                   TERMINATOR_FF             
0x000030FE      1  FF                                                   TERMINATOR_FF             
0x000030FF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003101      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003103     54  8034817582BB82A482BB82A481418E8481418ADB93A190F2...  LEN8_STRING_CP932         length=52, text="「そうそう、私、丸藤泉美。公務員で市役所…………？」"
0x00003139      1  FF                                                   TERMINATOR_FF             
0x0000313A      1  FF                                                   TERMINATOR_FF             
0x0000313B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000313D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000313F      1  FF                                                   TERMINATOR_FF             
0x00003140      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003142      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003144      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00003146      1  FF                                                   TERMINATOR_FF             
0x00003147      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003149      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000314B      3  F300CE                                               IMM16_F3                  u16_be=206, u16_le=52736
0x0000314E      1  FF                                                   TERMINATOR_FF             
0x0000314F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003151      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003153      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003155      1  FF                                                   TERMINATOR_FF             
0x00003156      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003158      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000315A      1  FF                                                   TERMINATOR_FF             
0x0000315B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000315D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000315F      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00003161      1  FF                                                   TERMINATOR_FF             
0x00003162      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003164      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003166      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00003168      1  FF                                                   TERMINATOR_FF             
0x00003169      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000316B      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000316D      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000316F      1  FF                                                   TERMINATOR_FF             
0x00003170      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003172      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003174      1  FF                                                   TERMINATOR_FF             
0x00003175      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003177      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003179      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000317B      1  FF                                                   TERMINATOR_FF             
0x0000317C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000317E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003180      3  F31A38                                               IMM16_F3                  u16_be=6712, u16_le=14362
0x00003183      1  FF                                                   TERMINATOR_FF             
0x00003184      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003186      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003188      9  8007495A3030353631                                   LEN8_STRING_CP932         length=7, text="IZ00561"
0x00003191      1  FF                                                   TERMINATOR_FF             
0x00003192      1  FF                                                   TERMINATOR_FF             
0x00003193      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003195      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003197     82  8050817582B382C182AB816381638ADB93A182B382F182C1...  LEN8_STRING_CP932         length=80, text="「さっき……丸藤さんって……いったもんね。\nどうして……私の名前、知ってるの？」"
0x000031E9      1  FF                                                   TERMINATOR_FF             
0x000031EA      1  FF                                                   TERMINATOR_FF             
0x000031EB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000031ED      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000031EF      1  FF                                                   TERMINATOR_FF             
0x000031F0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000031F2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000031F4      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000031F6      1  FF                                                   TERMINATOR_FF             
0x000031F7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000031F9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000031FB      3  F30215                                               IMM16_F3                  u16_be=533, u16_le=5378
0x000031FE      1  FF                                                   TERMINATOR_FF             
0x000031FF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003201      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003203      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003205      1  FF                                                   TERMINATOR_FF             
0x00003206      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003208      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000320A      1  FF                                                   TERMINATOR_FF             
0x0000320B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000320D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000320F      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00003211      1  FF                                                   TERMINATOR_FF             
0x00003212      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003214      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003216      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00003218      1  FF                                                   TERMINATOR_FF             
0x00003219      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000321B      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000321D      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000321F      1  FF                                                   TERMINATOR_FF             
0x00003220      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003222      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003224      1  FF                                                   TERMINATOR_FF             
0x00003225      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003227      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003229      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000322B      1  FF                                                   TERMINATOR_FF             
0x0000322C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000322E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003230      3  F31A39                                               IMM16_F3                  u16_be=6713, u16_le=14618
0x00003233      1  FF                                                   TERMINATOR_FF             
0x00003234      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003236      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003238      9  8007495A3030353632                                   LEN8_STRING_CP932         length=7, text="IZ00562"
0x00003241      1  FF                                                   TERMINATOR_FF             
0x00003242      1  FF                                                   TERMINATOR_FF             
0x00003243      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003245      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003247     32  801E817582E082B582A982B582C48163816383588367815B...  LEN8_STRING_CP932         length=30, text="「もしかして……ストーカー？」"
0x00003267      1  FF                                                   TERMINATOR_FF             
0x00003268      1  FF                                                   TERMINATOR_FF             
0x00003269      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000326B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000326D      1  FF                                                   TERMINATOR_FF             
0x0000326E      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00003270      3  F10055                                               IMM16_F1                  u16_be=85, u16_le=21760
0x00003273      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00003275      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00003276      1  FF                                                   TERMINATOR_FF             
0x00003277      2  0033                                                 WORD_00XX                 u16_be=51, low_byte=51
0x00003279      1  2F                                                   OPAQUE_RAW_BYTES          bytes=2F
0x0000327A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000327C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000327E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00003280      1  FF                                                   TERMINATOR_FF             
0x00003281      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003283      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003285      3  F31A3A                                               IMM16_F3                  u16_be=6714, u16_le=14874
0x00003288      1  FF                                                   TERMINATOR_FF             
0x00003289      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000328B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000328D     34  8020817582B382C182AB82E0814182BB82EA82A282A282DC...  LEN8_STRING_CP932         length=32, text="「さっきも、それいいましたよね」"
0x000032AF      1  FF                                                   TERMINATOR_FF             
0x000032B0      1  FF                                                   TERMINATOR_FF             
0x000032B1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000032B3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000032B5      1  FF                                                   TERMINATOR_FF             
0x000032B6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000032B8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000032BA      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000032BC      1  FF                                                   TERMINATOR_FF             
0x000032BD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000032BF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000032C1      3  F300CE                                               IMM16_F3                  u16_be=206, u16_le=52736
0x000032C4      1  FF                                                   TERMINATOR_FF             
0x000032C5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000032C7      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000032C9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000032CB      1  FF                                                   TERMINATOR_FF             
0x000032CC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000032CE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000032D0      1  FF                                                   TERMINATOR_FF             
0x000032D1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000032D3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000032D5      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000032D7      1  FF                                                   TERMINATOR_FF             
0x000032D8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000032DA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000032DC      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000032DE      1  FF                                                   TERMINATOR_FF             
0x000032DF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000032E1      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000032E3      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000032E5      1  FF                                                   TERMINATOR_FF             
0x000032E6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000032E8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000032EA      1  FF                                                   TERMINATOR_FF             
0x000032EB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000032ED      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000032EF      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000032F1      1  FF                                                   TERMINATOR_FF             
0x000032F2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000032F4      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000032F6      3  F31A3B                                               IMM16_F3                  u16_be=6715, u16_le=15130
0x000032F9      1  FF                                                   TERMINATOR_FF             
0x000032FA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000032FC      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000032FE      9  8007495A3030353730                                   LEN8_STRING_CP932         length=7, text="IZ00570"
0x00003307      1  FF                                                   TERMINATOR_FF             
0x00003308      1  FF                                                   TERMINATOR_FF             
0x00003309      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000330B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000330D     22  8014817582A0814182BB815B82BE82C182AF81488176         LEN8_STRING_CP932         length=20, text="「あ、そーだっけ？」"
0x00003323      1  FF                                                   TERMINATOR_FF             
0x00003324      1  FF                                                   TERMINATOR_FF             
0x00003325      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003327      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003329      1  FF                                                   TERMINATOR_FF             
0x0000332A      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x0000332C      2  0034                                                 WORD_00XX                 u16_be=52, low_byte=52
0x0000332E      1  05                                                   OPAQUE_RAW_BYTES          bytes=05
0x0000332F      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00003331      3  F10055                                               IMM16_F1                  u16_be=85, u16_le=21760
0x00003334      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003336      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00003337      1  FF                                                   TERMINATOR_FF             
0x00003338      2  0034                                                 WORD_00XX                 u16_be=52, low_byte=52
0x0000333A      1  05                                                   OPAQUE_RAW_BYTES          bytes=05
0x0000333B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000333D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000333F      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00003341      1  FF                                                   TERMINATOR_FF             
0x00003342      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003344      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003346      3  F31A3C                                               IMM16_F3                  u16_be=6716, u16_le=15386
0x00003349      1  FF                                                   TERMINATOR_FF             
0x0000334A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000334C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000334E     58  803881758DF093FA82BE82C182C481418E968FEE92AE8EE6...  LEN8_STRING_CP932         length=56, text="「昨日だって、事情聴取のとき、いってたじゃ\nないですか」"
0x00003388      1  FF                                                   TERMINATOR_FF             
0x00003389      1  FF                                                   TERMINATOR_FF             
0x0000338A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000338C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000338E      1  FF                                                   TERMINATOR_FF             
0x0000338F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003391      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003393      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00003395      1  FF                                                   TERMINATOR_FF             
0x00003396      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003398      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000339A      3  F3020B                                               IMM16_F3                  u16_be=523, u16_le=2818
0x0000339D      1  FF                                                   TERMINATOR_FF             
0x0000339E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000033A0      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000033A2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000033A4      1  FF                                                   TERMINATOR_FF             
0x000033A5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000033A7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000033A9      1  FF                                                   TERMINATOR_FF             
0x000033AA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000033AC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000033AE      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000033B0      1  FF                                                   TERMINATOR_FF             
0x000033B1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000033B3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000033B5      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000033B7      1  FF                                                   TERMINATOR_FF             
0x000033B8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000033BA      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000033BC      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000033BE      1  FF                                                   TERMINATOR_FF             
0x000033BF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000033C1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000033C3      1  FF                                                   TERMINATOR_FF             
0x000033C4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000033C6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000033C8      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000033CA      1  FF                                                   TERMINATOR_FF             
0x000033CB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000033CD      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000033CF      3  F31A3D                                               IMM16_F3                  u16_be=6717, u16_le=15642
0x000033D2      1  FF                                                   TERMINATOR_FF             
0x000033D3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000033D5      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000033D7      9  8007495A3030353830                                   LEN8_STRING_CP932         length=7, text="IZ00580"
0x000033E0      1  FF                                                   TERMINATOR_FF             
0x000033E1      1  FF                                                   TERMINATOR_FF             
0x000033E2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000033E4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000033E6     24  8016817582A082C1814182BB815B82BE82E682CB815B8176     LEN8_STRING_CP932         length=22, text="「あっ、そーだよねー」"
0x000033FE      1  FF                                                   TERMINATOR_FF             
0x000033FF      1  FF                                                   TERMINATOR_FF             
0x00003400      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003402      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003404      1  FF                                                   TERMINATOR_FF             
0x00003405      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003407      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003409      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000340B      1  FF                                                   TERMINATOR_FF             
0x0000340C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000340E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003410      3  F300C2                                               IMM16_F3                  u16_be=194, u16_le=49664
0x00003413      1  FF                                                   TERMINATOR_FF             
0x00003414      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003416      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003418      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000341A      1  FF                                                   TERMINATOR_FF             
0x0000341B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000341D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000341F      1  FF                                                   TERMINATOR_FF             
0x00003420      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003422      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003424      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00003426      1  FF                                                   TERMINATOR_FF             
0x00003427      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003429      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000342B      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000342D      1  FF                                                   TERMINATOR_FF             
0x0000342E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003430      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003432      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00003434      1  FF                                                   TERMINATOR_FF             
0x00003435      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003437      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003439      1  FF                                                   TERMINATOR_FF             
0x0000343A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000343C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000343E      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00003440      1  FF                                                   TERMINATOR_FF             
0x00003441      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003443      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003445      3  F31A3E                                               IMM16_F3                  u16_be=6718, u16_le=15898
0x00003448      1  FF                                                   TERMINATOR_FF             
0x00003449      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000344B      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000344D      9  8007495A3030353930                                   LEN8_STRING_CP932         length=7, text="IZ00590"
0x00003456      1  FF                                                   TERMINATOR_FF             
0x00003457      1  FF                                                   TERMINATOR_FF             
0x00003458      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000345A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000345C     96  805E81758E8481418EB897E782C882B182C682A282C182BF...  LEN8_STRING_CP932         length=94, text="「私、失礼なこといっちゃったー。おわびと\n一緒にお礼しなきゃー。ね、ね、じゃ、すぐそこだから」"
0x000034BC      1  FF                                                   TERMINATOR_FF             
0x000034BD      1  FF                                                   TERMINATOR_FF             
0x000034BE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000034C0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000034C2      1  FF                                                   TERMINATOR_FF             
0x000034C3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000034C5      2  0051                                                 WORD_00XX                 u16_be=81, low_byte=81
0x000034C7      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000034C9      1  FF                                                   TERMINATOR_FF             
0x000034CA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000034CC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000034CE      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000034D0      1  FF                                                   TERMINATOR_FF             
0x000034D1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000034D3      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000034D5      3  F31A3F                                               IMM16_F3                  u16_be=6719, u16_le=16154
0x000034D8      1  FF                                                   TERMINATOR_FF             
0x000034D9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000034DB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000034DD     56  80368ADB93A182B382F182CC8FCE8AE782C9814182DA82AD...  LEN8_STRING_CP932         length=54, text="丸藤さんの笑顔に、ぼくはかわいい人なんだな、と思った。"
0x00003515      1  FF                                                   TERMINATOR_FF             
0x00003516      1  FF                                                   TERMINATOR_FF             
0x00003517      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003519      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000351B      1  FF                                                   TERMINATOR_FF             
0x0000351C      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x0000351E      2  0067                                                 WORD_00XX                 u16_be=103, low_byte=103
0x00003520      1  38                                                   OPAQUE_RAW_BYTES          bytes=38
0x00003521      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00003523      3  F10034                                               IMM16_F1                  u16_be=52, u16_le=13312
0x00003526      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003528      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00003529      1  FF                                                   TERMINATOR_FF             
0x0000352A      2  0067                                                 WORD_00XX                 u16_be=103, low_byte=103
0x0000352C      1  38                                                   OPAQUE_RAW_BYTES          bytes=38
0x0000352D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000352F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003531      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00003533      1  FF                                                   TERMINATOR_FF             
0x00003534      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003536      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003538      2  F229                                                 IMM8_F2                   u8=41, s8=41
0x0000353A      1  FF                                                   TERMINATOR_FF             
0x0000353B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000353D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000353F      1  FF                                                   TERMINATOR_FF             
0x00003540      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003542      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003544      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00003546      1  FF                                                   TERMINATOR_FF             
0x00003547      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003549      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000354B      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000354D      1  FF                                                   TERMINATOR_FF             
0x0000354E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003550      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003552      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00003554      1  FF                                                   TERMINATOR_FF             
0x00003555      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003557      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003559      1  FF                                                   TERMINATOR_FF             
0x0000355A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000355C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000355E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00003560      1  FF                                                   TERMINATOR_FF             
0x00003561      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003563      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003565      3  F31A40                                               IMM16_F3                  u16_be=6720, u16_le=16410
0x00003568      1  FF                                                   TERMINATOR_FF             
0x00003569      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000356B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000356D     68  804282DA82AD82CD96DA82F082A082B082E981425C6E8DA2...  LEN8_STRING_CP932         length=66, text="ぼくは目をあげる。\n困惑とも恥じらいともつかない声が、耳についた。"
0x000035B1      1  FF                                                   TERMINATOR_FF             
0x000035B2      1  FF                                                   TERMINATOR_FF             
0x000035B3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000035B5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000035B7      1  FF                                                   TERMINATOR_FF             
0x000035B8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000035BA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000035BC      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000035BE      1  FF                                                   TERMINATOR_FF             
0x000035BF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000035C1      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000035C3      3  F31A41                                               IMM16_F3                  u16_be=6721, u16_le=16666
0x000035C6      1  FF                                                   TERMINATOR_FF             
0x000035C7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000035C9      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000035CB      9  8007495A3030363030                                   LEN8_STRING_CP932         length=7, text="IZ00600"
0x000035D4      1  FF                                                   TERMINATOR_FF             
0x000035D5      1  FF                                                   TERMINATOR_FF             
0x000035D6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000035D8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000035DA     36  8022817582BB815B82C882F182C582B782AF82C7815B8163...  LEN8_STRING_CP932         length=34, text="「そーなんですけどー……でもお～」"
0x000035FE      1  FF                                                   TERMINATOR_FF             
0x000035FF      1  FF                                                   TERMINATOR_FF             
0x00003600      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003602      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003604      1  FF                                                   TERMINATOR_FF             
0x00003605      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003607      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003609      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000360B      1  FF                                                   TERMINATOR_FF             
0x0000360C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000360E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003610      3  F31A42                                               IMM16_F3                  u16_be=6722, u16_le=16922
0x00003613      1  FF                                                   TERMINATOR_FF             
0x00003614      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003616      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003618     24  801682DA82AD82CD95D382E882F08CA989F182B582BD8142     LEN8_STRING_CP932         length=22, text="ぼくは辺りを見回した。"
0x00003630      1  FF                                                   TERMINATOR_FF             
0x00003631      1  FF                                                   TERMINATOR_FF             
0x00003632      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003634      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003636      1  FF                                                   TERMINATOR_FF             
0x00003637      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003639      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000363B      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x0000363D      1  FF                                                   TERMINATOR_FF             
0x0000363E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003640      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003642      3  F30160                                               IMM16_F3                  u16_be=352, u16_le=24577
0x00003645      1  FF                                                   TERMINATOR_FF             
0x00003646      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003648      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000364A      1  FF                                                   TERMINATOR_FF             
0x0000364B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000364D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000364F      2  F20A                                                 IMM8_F2                   u8=10, s8=10
0x00003651      1  FF                                                   TERMINATOR_FF             
0x00003652      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003654      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003656      2  F218                                                 IMM8_F2                   u8=24, s8=24
0x00003658      1  FF                                                   TERMINATOR_FF             
0x00003659      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000365B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000365D      1  FF                                                   TERMINATOR_FF             
0x0000365E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003660      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003662      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00003664      1  FF                                                   TERMINATOR_FF             
0x00003665      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003667      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003669      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000366B      1  FF                                                   TERMINATOR_FF             
0x0000366C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000366E      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003670      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00003672      1  FF                                                   TERMINATOR_FF             
0x00003673      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003675      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003677      1  FF                                                   TERMINATOR_FF             
0x00003678      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000367A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000367C      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000367E      1  FF                                                   TERMINATOR_FF             
0x0000367F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003681      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003683      3  F31A43                                               IMM16_F3                  u16_be=6723, u16_le=17178
0x00003686      1  FF                                                   TERMINATOR_FF             
0x00003687      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003689      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000368B    112  806E837A83588367959782C682E4815B82A9834C83838362...  LEN8_STRING_CP932         length=110, text="ホスト風とゆーかキャッチセールス風とゆーか、そんな感じのお兄さんが、なれなれしく女性の\n肩に手をまわしている。"
0x000036FB      1  FF                                                   TERMINATOR_FF             
0x000036FC      1  FF                                                   TERMINATOR_FF             
0x000036FD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000036FF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003701      1  FF                                                   TERMINATOR_FF             
0x00003702      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003704      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003706      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00003708      1  FF                                                   TERMINATOR_FF             
0x00003709      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000370B      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000370D      3  F31A44                                               IMM16_F3                  u16_be=6724, u16_le=17434
0x00003710      1  FF                                                   TERMINATOR_FF             
0x00003711      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003713      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003715    112  806E8F9782CC906C82CC95FB82CD814182A282E282AA82C1...  LEN8_STRING_CP932         length=110, text="女の人の方は、いやがっているのだろうけど、\n強くこばめないせいか、ふたりの間の距離が\nどんどん近くなっていく。"
0x00003785      1  FF                                                   TERMINATOR_FF             
0x00003786      1  FF                                                   TERMINATOR_FF             
0x00003787      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003789      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000378B      1  FF                                                   TERMINATOR_FF             
0x0000378C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000378E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003790      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00003792      1  FF                                                   TERMINATOR_FF             
0x00003793      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003795      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003797      3  F31A45                                               IMM16_F3                  u16_be=6725, u16_le=17690
0x0000379A      1  FF                                                   TERMINATOR_FF             
0x0000379B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000379D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000379F      9  8007495A3030363130                                   LEN8_STRING_CP932         length=7, text="IZ00610"
0x000037A8      1  FF                                                   TERMINATOR_FF             
0x000037A9      1  FF                                                   TERMINATOR_FF             
0x000037AA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000037AC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000037AE     54  8034817582A6814182BE82C182C4814182BB815B82E4815B...  LEN8_STRING_CP932         length=52, text="「え、だって、そーゆーわけじゃないんです\nけど……」"
0x000037E4      1  FF                                                   TERMINATOR_FF             
0x000037E5      1  FF                                                   TERMINATOR_FF             
0x000037E6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000037E8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000037EA      1  FF                                                   TERMINATOR_FF             
0x000037EB      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x000037ED      3  F10035                                               IMM16_F1                  u16_be=53, u16_le=13568
0x000037F0      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000037F2      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x000037F3      1  FF                                                   TERMINATOR_FF             
0x000037F4      2  0038                                                 WORD_00XX                 u16_be=56, low_byte=56
0x000037F6      1  D4                                                   OPAQUE_RAW_BYTES          bytes=D4
0x000037F7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000037F9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000037FB      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000037FD      1  FF                                                   TERMINATOR_FF             
0x000037FE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003800      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003802      3  F31A46                                               IMM16_F3                  u16_be=6726, u16_le=17946
0x00003805      1  FF                                                   TERMINATOR_FF             
0x00003806      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003808      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000380A    109  806B82A482ED82A081418DA193FA82CD834C838383628360...  LEN8_STRING_CP932         length=107, text="うわあ、今日はキャッチ？ いそがしい人だなあ。\nはっきり『いやです』っていわないから、\nつけこんでくるのに。"
0x00003877      1  FF                                                   TERMINATOR_FF             
0x00003878      1  FF                                                   TERMINATOR_FF             
0x00003879      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000387B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000387D      1  FF                                                   TERMINATOR_FF             
0x0000387E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003880      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003882      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00003884      1  FF                                                   TERMINATOR_FF             
0x00003885      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003887      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003889      3  F31A47                                               IMM16_F3                  u16_be=6727, u16_le=18202
0x0000388C      1  FF                                                   TERMINATOR_FF             
0x0000388D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000388F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003891     60  803A82E082B582A982B782E982C6816381635C6E8DF093FA...  LEN8_STRING_CP932         length=58, text="もしかすると……\n昨日の痴漢も……勘違いじゃなかったとか。"
0x000038CD      1  FF                                                   TERMINATOR_FF             
0x000038CE      1  FF                                                   TERMINATOR_FF             
0x000038CF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000038D1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000038D3      1  FF                                                   TERMINATOR_FF             
0x000038D4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000038D6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000038D8      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x000038DA      1  FF                                                   TERMINATOR_FF             
0x000038DB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000038DD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000038DF      3  F30161                                               IMM16_F3                  u16_be=353, u16_le=24833
0x000038E2      1  FF                                                   TERMINATOR_FF             
0x000038E3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000038E5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000038E7      1  FF                                                   TERMINATOR_FF             
0x000038E8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000038EA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000038EC      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000038EE      1  FF                                                   TERMINATOR_FF             
0x000038EF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000038F1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000038F3      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000038F5      1  FF                                                   TERMINATOR_FF             
0x000038F6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000038F8      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000038FA      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000038FC      1  FF                                                   TERMINATOR_FF             
0x000038FD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000038FF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003901      1  FF                                                   TERMINATOR_FF             
0x00003902      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003904      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003906      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00003908      1  FF                                                   TERMINATOR_FF             
0x00003909      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000390B      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000390D      3  F31A48                                               IMM16_F3                  u16_be=6728, u16_le=18458
0x00003910      1  FF                                                   TERMINATOR_FF             
0x00003911      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003913      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003915      9  8007495A3030363230                                   LEN8_STRING_CP932         length=7, text="IZ00620"
0x0000391E      1  FF                                                   TERMINATOR_FF             
0x0000391F      1  FF                                                   TERMINATOR_FF             
0x00003920      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003922      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003924     44  802A817582A6814182C582E081418DA282E882DC82B78142...  LEN8_STRING_CP932         length=42, text="「え、でも、困ります。本当に興味ないんで」"
0x00003950      1  FF                                                   TERMINATOR_FF             
0x00003951      1  FF                                                   TERMINATOR_FF             
0x00003952      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003954      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003956      1  FF                                                   TERMINATOR_FF             
0x00003957      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003959      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000395B      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000395D      1  FF                                                   TERMINATOR_FF             
0x0000395E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003960      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003962      3  F31A49                                               IMM16_F3                  u16_be=6729, u16_le=18714
0x00003965      1  FF                                                   TERMINATOR_FF             
0x00003966      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003968      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000396A      9  800745583435303130                                   LEN8_STRING_CP932         length=7, text="EX45010"
0x00003973      1  FF                                                   TERMINATOR_FF             
0x00003974      1  FF                                                   TERMINATOR_FF             
0x00003975      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003977      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003979     82  8050817582B7815B82AE814282BF82E582C182C682BE82A9...  LEN8_STRING_CP932         length=80, text="「すーぐ。ちょっとだから。マジすぐ終わるって。オレ、ほら、あやしくないでしょ？」"
0x000039CB      1  FF                                                   TERMINATOR_FF             
0x000039CC      1  FF                                                   TERMINATOR_FF             
0x000039CD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000039CF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000039D1      1  FF                                                   TERMINATOR_FF             
0x000039D2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000039D4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000039D6      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000039D8      1  FF                                                   TERMINATOR_FF             
0x000039D9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000039DB      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000039DD      3  F31A4A                                               IMM16_F3                  u16_be=6730, u16_le=18970
0x000039E0      1  FF                                                   TERMINATOR_FF             
0x000039E1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000039E3      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000039E5      9  8007495A3030363330                                   LEN8_STRING_CP932         length=7, text="IZ00630"
0x000039EE      1  FF                                                   TERMINATOR_FF             
0x000039EF      1  FF                                                   TERMINATOR_FF             
0x000039F0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000039F2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000039F4     38  8024817582A682A581608163816382BB82A482C882F182C5...  LEN8_STRING_CP932         length=36, text="「えぇ～……そうなんですけどお……」"
0x00003A1A      1  FF                                                   TERMINATOR_FF             
0x00003A1B      1  FF                                                   TERMINATOR_FF             
0x00003A1C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003A1E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003A20      1  FF                                                   TERMINATOR_FF             
0x00003A21      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003A23      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003A25      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00003A27      1  FF                                                   TERMINATOR_FF             
0x00003A28      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003A2A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003A2C      3  F31A4B                                               IMM16_F3                  u16_be=6731, u16_le=19226
0x00003A2F      1  FF                                                   TERMINATOR_FF             
0x00003A30      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003A32      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003A34      9  800745583435313030                                   LEN8_STRING_CP932         length=7, text="EX45100"
0x00003A3D      1  FF                                                   TERMINATOR_FF             
0x00003A3E      1  FF                                                   TERMINATOR_FF             
0x00003A3F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003A41      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003A43     45  802B8175975C92E882C882A282F182C582B582E581482098...  LEN8_STRING_CP932         length=43, text="「予定ないんでしょ？ 話だけでいーんだって」"
0x00003A70      1  FF                                                   TERMINATOR_FF             
0x00003A71      1  FF                                                   TERMINATOR_FF             
0x00003A72      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003A74      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003A76      1  FF                                                   TERMINATOR_FF             
0x00003A77      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003A79      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003A7B      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00003A7D      1  FF                                                   TERMINATOR_FF             
0x00003A7E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003A80      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003A82      3  F31A4C                                               IMM16_F3                  u16_be=6732, u16_le=19482
0x00003A85      1  FF                                                   TERMINATOR_FF             
0x00003A86      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003A88      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003A8A     52  8032899F82B582CC8BAD82A282A88C5A82B382F182C98141...  LEN8_STRING_CP932         length=50, text="押しの強いお兄さんに、彼女はもう涙目になっている。"
0x00003ABE      1  FF                                                   TERMINATOR_FF             
0x00003ABF      1  FF                                                   TERMINATOR_FF             
0x00003AC0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003AC2      2  0004                                                 WORD_00XX                 u16_be=4, low_byte=4
0x00003AC4     12  800A90BA82F082A982AF82E9                             LEN8_STRING_CP932         length=10, text="声をかける"
0x00003AD0      1  FF                                                   TERMINATOR_FF             
0x00003AD1      1  FF                                                   TERMINATOR_FF             
0x00003AD2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003AD4      2  000A                                                 WORD_00XX                 u16_be=10, low_byte=10
0x00003AD6      3  F31A4D                                               IMM16_F3                  u16_be=6733, u16_le=19738
0x00003AD9      1  FF                                                   TERMINATOR_FF             
0x00003ADA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003ADC      2  0005                                                 WORD_00XX                 u16_be=5, low_byte=5
0x00003ADE     12  800A976C8E7182F08CA982E9                             LEN8_STRING_CP932         length=10, text="様子を見る"
0x00003AEA      1  FF                                                   TERMINATOR_FF             
0x00003AEB      1  FF                                                   TERMINATOR_FF             
0x00003AEC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003AEE      2  000B                                                 WORD_00XX                 u16_be=11, low_byte=11
0x00003AF0      3  F31A4E                                               IMM16_F3                  u16_be=6734, u16_le=19994
0x00003AF3      1  FF                                                   TERMINATOR_FF             
0x00003AF4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003AF6      2  0006                                                 WORD_00XX                 u16_be=6, low_byte=6
0x00003AF8     10  800882D982C182C682AD                                 LEN8_STRING_CP932         length=8, text="ほっとく"
0x00003B02      1  FF                                                   TERMINATOR_FF             
0x00003B03      1  FF                                                   TERMINATOR_FF             
0x00003B04      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003B06      2  000C                                                 WORD_00XX                 u16_be=12, low_byte=12
0x00003B08      3  F31A4F                                               IMM16_F3                  u16_be=6735, u16_le=20250
0x00003B0B      1  FF                                                   TERMINATOR_FF             
0x00003B0C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003B0E      2  0008                                                 WORD_00XX                 u16_be=8, low_byte=8
0x00003B10      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00003B12      1  FF                                                   TERMINATOR_FF             
0x00003B13      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003B15      2  0009                                                 WORD_00XX                 u16_be=9, low_byte=9
0x00003B17      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003B19      1  FF                                                   TERMINATOR_FF             
0x00003B1A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003B1C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003B1E      2  F212                                                 IMM8_F2                   u8=18, s8=18
0x00003B20      1  FF                                                   TERMINATOR_FF             
0x00003B21      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003B23      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003B25      1  FF                                                   TERMINATOR_FF             
0x00003B26      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00003B28      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x00003B2B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003B2D      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00003B2E      1  FF                                                   TERMINATOR_FF             
0x00003B2F      2  003B                                                 WORD_00XX                 u16_be=59, low_byte=59
0x00003B31      1  37                                                   OPAQUE_RAW_BYTES          bytes=37
0x00003B32      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00003B34      2  0041                                                 WORD_00XX                 u16_be=65, low_byte=65
0x00003B36      1  84                                                   OPAQUE_RAW_BYTES          bytes=84
0x00003B37      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00003B39      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x00003B3C      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00003B3E      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00003B3F      1  FF                                                   TERMINATOR_FF             
0x00003B40      2  003D                                                 WORD_00XX                 u16_be=61, low_byte=61
0x00003B42      1  6A                                                   OPAQUE_RAW_BYTES          bytes=6A
0x00003B43      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003B45      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003B47      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00003B49      1  FF                                                   TERMINATOR_FF             
0x00003B4A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003B4C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003B4E      3  F31A50                                               IMM16_F3                  u16_be=6736, u16_le=20506
0x00003B51      1  FF                                                   TERMINATOR_FF             
0x00003B52      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003B54      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003B56      9  800745583435313130                                   LEN8_STRING_CP932         length=7, text="EX45110"
0x00003B5F      1  FF                                                   TERMINATOR_FF             
0x00003B60      1  FF                                                   TERMINATOR_FF             
0x00003B61      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003B63      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003B65     59  8039817582A6814182C882C98148208349838C82CC82B182...  LEN8_STRING_CP932         length=57, text="「え、なに？ オレのこと、きらいー？\n傷ついちゃうよなぁ」"
0x00003BA0      1  FF                                                   TERMINATOR_FF             
0x00003BA1      1  FF                                                   TERMINATOR_FF             
0x00003BA2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003BA4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003BA6      1  FF                                                   TERMINATOR_FF             
0x00003BA7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003BA9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003BAB      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00003BAD      1  FF                                                   TERMINATOR_FF             
0x00003BAE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003BB0      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003BB2      3  F31A51                                               IMM16_F3                  u16_be=6737, u16_le=20762
0x00003BB5      1  FF                                                   TERMINATOR_FF             
0x00003BB6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003BB8      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003BBA      9  8007495A3030363430                                   LEN8_STRING_CP932         length=7, text="IZ00640"
0x00003BC3      1  FF                                                   TERMINATOR_FF             
0x00003BC4      1  FF                                                   TERMINATOR_FF             
0x00003BC5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003BC7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003BC9     48  802E817582A6815B814182BB82F182C882B182C682A282C1...  LEN8_STRING_CP932         length=46, text="「えー、そんなこといってないじゃないですかあ」"
0x00003BF9      1  FF                                                   TERMINATOR_FF             
0x00003BFA      1  FF                                                   TERMINATOR_FF             
0x00003BFB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003BFD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003BFF      1  FF                                                   TERMINATOR_FF             
0x00003C00      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003C02      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003C04      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00003C06      1  FF                                                   TERMINATOR_FF             
0x00003C07      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003C09      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003C0B      3  F31A52                                               IMM16_F3                  u16_be=6738, u16_le=21018
0x00003C0E      1  FF                                                   TERMINATOR_FF             
0x00003C0F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003C11      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003C13      9  800745583435313230                                   LEN8_STRING_CP932         length=7, text="EX45120"
0x00003C1C      1  FF                                                   TERMINATOR_FF             
0x00003C1D      1  FF                                                   TERMINATOR_FF             
0x00003C1E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003C20      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003C22     55  8035817582B682E1814182A2815B82E682CB81482082BF82...  LEN8_STRING_CP932         length=53, text="「じゃ、いーよね？ ちょっと、そこ。マジすぐ\nだから」"
0x00003C59      1  FF                                                   TERMINATOR_FF             
0x00003C5A      1  FF                                                   TERMINATOR_FF             
0x00003C5B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003C5D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003C5F      1  FF                                                   TERMINATOR_FF             
0x00003C60      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003C62      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003C64      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00003C66      1  FF                                                   TERMINATOR_FF             
0x00003C67      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003C69      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003C6B      3  F31A53                                               IMM16_F3                  u16_be=6739, u16_le=21274
0x00003C6E      1  FF                                                   TERMINATOR_FF             
0x00003C6F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003C71      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003C73      9  8007495A3030363530                                   LEN8_STRING_CP932         length=7, text="IZ00650"
0x00003C7C      1  FF                                                   TERMINATOR_FF             
0x00003C7D      1  FF                                                   TERMINATOR_FF             
0x00003C7E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003C80      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003C82     46  802C817582A682A58160814882C582E082A78163816382BB...  LEN8_STRING_CP932         length=44, text="「えぇ～？でもぉ……そーゆーのじゃないしー」"
0x00003CB0      1  FF                                                   TERMINATOR_FF             
0x00003CB1      1  FF                                                   TERMINATOR_FF             
0x00003CB2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003CB4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003CB6      1  FF                                                   TERMINATOR_FF             
0x00003CB7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003CB9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003CBB      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00003CBD      1  FF                                                   TERMINATOR_FF             
0x00003CBE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003CC0      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003CC2      3  F31A54                                               IMM16_F3                  u16_be=6740, u16_le=21530
0x00003CC5      1  FF                                                   TERMINATOR_FF             
0x00003CC6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003CC8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003CCA     66  80408FF38BB582CD82C782F182C782F188AB82AD82C882C1...  LEN8_STRING_CP932         length=64, text="状況はどんどん悪くなって行く。\n肩に手を回され、顔と顔が近づく。"
0x00003D0C      1  FF                                                   TERMINATOR_FF             
0x00003D0D      1  FF                                                   TERMINATOR_FF             
0x00003D0E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003D10      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003D12      1  FF                                                   TERMINATOR_FF             
0x00003D13      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003D15      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003D17      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00003D19      1  FF                                                   TERMINATOR_FF             
0x00003D1A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003D1C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003D1E      3  F31A55                                               IMM16_F3                  u16_be=6741, u16_le=21786
0x00003D21      1  FF                                                   TERMINATOR_FF             
0x00003D22      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003D24      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003D26     56  803695F882AB82B582DF82E782EA82C482E982CC82C98BDF...  LEN8_STRING_CP932         length=54, text="抱きしめられてるのに近いから、逃げ場は\nどこにもない。"
0x00003D5E      1  FF                                                   TERMINATOR_FF             
0x00003D5F      1  FF                                                   TERMINATOR_FF             
0x00003D60      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003D62      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003D64      1  FF                                                   TERMINATOR_FF             
0x00003D65      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00003D67      2  0041                                                 WORD_00XX                 u16_be=65, low_byte=65
0x00003D69      1  84                                                   OPAQUE_RAW_BYTES          bytes=84
0x00003D6A      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00003D6C      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x00003D6F      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00003D71      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00003D72      1  FF                                                   TERMINATOR_FF             
0x00003D73      2  0041                                                 WORD_00XX                 u16_be=65, low_byte=65
0x00003D75      1  84                                                   OPAQUE_RAW_BYTES          bytes=84
0x00003D76      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003D78      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003D7A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00003D7C      1  FF                                                   TERMINATOR_FF             
0x00003D7D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003D7F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003D81      3  F31A56                                               IMM16_F3                  u16_be=6742, u16_le=22042
0x00003D84      1  FF                                                   TERMINATOR_FF             
0x00003D85      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003D87      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003D89     86  805482A082A0814182BF82E182F182C6926682EA82CE82A2...  LEN8_STRING_CP932         length=84, text="ああ、ちゃんと断ればいいのにね。\nそーすれば、向こうだって、無理強いはしない\nのに。"
0x00003DDF      1  FF                                                   TERMINATOR_FF             
0x00003DE0      1  FF                                                   TERMINATOR_FF             
0x00003DE1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003DE3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003DE5      1  FF                                                   TERMINATOR_FF             
0x00003DE6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003DE8      2  0037                                                 WORD_00XX                 u16_be=55, low_byte=55
0x00003DEA      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00003DEC      1  FF                                                   TERMINATOR_FF             
0x00003DED      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003DEF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003DF1      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00003DF3      1  FF                                                   TERMINATOR_FF             
0x00003DF4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003DF6      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003DF8      3  F31A57                                               IMM16_F3                  u16_be=6743, u16_le=22298
0x00003DFB      1  FF                                                   TERMINATOR_FF             
0x00003DFC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003DFE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003E00    112  806E82DC814182BB82CC82A482BF82BF82E182F182C693A6...  LEN8_STRING_CP932         length=110, text="ま、そのうちちゃんと逃げだすでしょう。\nあぶなくなりそうだったら。\nそれくらいの自己防衛は大人なんだから、ね。"
0x00003E70      1  FF                                                   TERMINATOR_FF             
0x00003E71      1  FF                                                   TERMINATOR_FF             
0x00003E72      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003E74      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003E76      1  FF                                                   TERMINATOR_FF             
0x00003E77      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003E79      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003E7B      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00003E7D      1  FF                                                   TERMINATOR_FF             
0x00003E7E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003E80      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003E82      3  F31A58                                               IMM16_F3                  u16_be=6744, u16_le=22554
0x00003E85      1  FF                                                   TERMINATOR_FF             
0x00003E86      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003E88      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00003E8A      9  8007495A3930303130                                   LEN8_STRING_CP932         length=7, text="IZ90010"
0x00003E93      1  FF                                                   TERMINATOR_FF             
0x00003E94      1  FF                                                   TERMINATOR_FF             
0x00003E95      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003E97      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003E99     52  8032817582BE815B82A982E7815B81418DA282E982C182C4...  LEN8_STRING_CP932         length=50, text="「だーからー、困るっていってるじゃない\nですかー」"
0x00003ECD      1  FF                                                   TERMINATOR_FF             
0x00003ECE      1  FF                                                   TERMINATOR_FF             
0x00003ECF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003ED1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003ED3      1  FF                                                   TERMINATOR_FF             
0x00003ED4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003ED6      2  0053                                                 WORD_00XX                 u16_be=83, low_byte=83
0x00003ED8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003EDA      1  FF                                                   TERMINATOR_FF             
0x00003EDB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003EDD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003EDF      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00003EE1      1  FF                                                   TERMINATOR_FF             
0x00003EE2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003EE4      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003EE6      3  F31A59                                               IMM16_F3                  u16_be=6745, u16_le=22810
0x00003EE9      1  FF                                                   TERMINATOR_FF             
0x00003EEA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003EEC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003EEE     32  801E82A8814182BF82E182F182C6926682EA82BB82A482B6...  LEN8_STRING_CP932         length=30, text="お、ちゃんと断れそうじゃない。"
0x00003F0E      1  FF                                                   TERMINATOR_FF             
0x00003F0F      1  FF                                                   TERMINATOR_FF             
0x00003F10      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003F12      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003F14      1  FF                                                   TERMINATOR_FF             
0x00003F15      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003F17      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003F19      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00003F1B      1  FF                                                   TERMINATOR_FF             
0x00003F1C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003F1E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003F20      3  F31A5A                                               IMM16_F3                  u16_be=6746, u16_le=23066
0x00003F23      1  FF                                                   TERMINATOR_FF             
0x00003F24      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003F26      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003F28     94  805C82DC82A0814182D982C182C682A282C482E091E58FE4...  LEN8_STRING_CP932         length=92, text="まあ、ほっといても大丈夫……かな。\nなんだかね、変な下心があるみたいに思われる\nのもやだし。"
0x00003F86      1  FF                                                   TERMINATOR_FF             
0x00003F87      1  FF                                                   TERMINATOR_FF             
0x00003F88      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003F8A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003F8C      1  FF                                                   TERMINATOR_FF             
0x00003F8D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003F8F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003F91      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00003F93      1  FF                                                   TERMINATOR_FF             
0x00003F94      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003F96      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003F98      3  F31A5B                                               IMM16_F3                  u16_be=6747, u16_le=23322
0x00003F9B      1  FF                                                   TERMINATOR_FF             
0x00003F9C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003F9E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003FA0     52  803282BE82A282BD82A282B382A0814182E0815B82BF82E5...  LEN8_STRING_CP932         length=50, text="だいたいさあ、もーちょっと気をつけるべき\nだよね。"
0x00003FD4      1  FF                                                   TERMINATOR_FF             
0x00003FD5      1  FF                                                   TERMINATOR_FF             
0x00003FD6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00003FD8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00003FDA      1  FF                                                   TERMINATOR_FF             
0x00003FDB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003FDD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003FDF      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00003FE1      1  FF                                                   TERMINATOR_FF             
0x00003FE2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00003FE4      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00003FE6      3  F31A5C                                               IMM16_F3                  u16_be=6748, u16_le=23578
0x00003FE9      1  FF                                                   TERMINATOR_FF             
0x00003FEA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003FEC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00003FEE     84  805282A082F182C882C98BB982E082C68A4A82AF82BF82E1...  LEN8_STRING_CP932         length=82, text="あんなに胸もと開けちゃって。\nあれじゃ、見せたいのよーって誘ってるみたい\nだもん。"
0x00004042      1  FF                                                   TERMINATOR_FF             
0x00004043      1  FF                                                   TERMINATOR_FF             
0x00004044      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004046      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004048      1  FF                                                   TERMINATOR_FF             
0x00004049      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000404B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000404D      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000404F      1  FF                                                   TERMINATOR_FF             
0x00004050      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004052      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004054      3  F31A5D                                               IMM16_F3                  u16_be=6749, u16_le=23834
0x00004057      1  FF                                                   TERMINATOR_FF             
0x00004058      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000405A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000405C     88  805682A282E281418CA982B982BD82A282CC81485C6E8163...  LEN8_STRING_CP932         length=86, text="いや、見せたいの？\n……そんなこともないか。\nそんな度胸のありそうな人にも見えないし。"
0x000040B4      1  FF                                                   TERMINATOR_FF             
0x000040B5      1  FF                                                   TERMINATOR_FF             
0x000040B6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000040B8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000040BA      1  FF                                                   TERMINATOR_FF             
0x000040BB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000040BD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000040BF      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000040C1      1  FF                                                   TERMINATOR_FF             
0x000040C2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000040C4      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000040C6      3  F31A5E                                               IMM16_F3                  u16_be=6750, u16_le=24090
0x000040C9      1  FF                                                   TERMINATOR_FF             
0x000040CA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000040CC      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000040CE      9  8007495A3930303230                                   LEN8_STRING_CP932         length=7, text="IZ90020"
0x000040D7      1  FF                                                   TERMINATOR_FF             
0x000040D8      1  FF                                                   TERMINATOR_FF             
0x000040D9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000040DB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000040DD     52  8032817582BE815B82A9815B82E7815B814282C282A282C4...  LEN8_STRING_CP932         length=50, text="「だーかーらー。ついて来ないで\nくーだーさーいー」"
0x00004111      1  FF                                                   TERMINATOR_FF             
0x00004112      1  FF                                                   TERMINATOR_FF             
0x00004113      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004115      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004117      1  FF                                                   TERMINATOR_FF             
0x00004118      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000411A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000411C      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x0000411E      1  FF                                                   TERMINATOR_FF             
0x0000411F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004121      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004123      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004125      1  FF                                                   TERMINATOR_FF             
0x00004126      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004128      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000412A      1  FF                                                   TERMINATOR_FF             
0x0000412B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000412D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000412F      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00004131      1  FF                                                   TERMINATOR_FF             
0x00004132      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004134      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004136      2  F2FF                                                 IMM8_F2                   u8=255, s8=-1
0x00004138      1  FF                                                   TERMINATOR_FF             
0x00004139      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000413B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000413D      1  FF                                                   TERMINATOR_FF             
0x0000413E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004140      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004142      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00004144      1  FF                                                   TERMINATOR_FF             
0x00004145      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004147      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004149      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000414B      1  FF                                                   TERMINATOR_FF             
0x0000414C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000414E      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004150      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00004152      1  FF                                                   TERMINATOR_FF             
0x00004153      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004155      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004157      1  FF                                                   TERMINATOR_FF             
0x00004158      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000415A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000415C      2  F20B                                                 IMM8_F2                   u8=11, s8=11
0x0000415E      1  FF                                                   TERMINATOR_FF             
0x0000415F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004161      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004163      1  FF                                                   TERMINATOR_FF             
0x00004164      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004166      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004168      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x0000416A      1  FF                                                   TERMINATOR_FF             
0x0000416B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000416D      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x0000416F     14  800C50533241303032612E62696E                         LEN8_STRING_CP932         length=12, text="PS2A002a.bin"
0x0000417D      1  FF                                                   TERMINATOR_FF             
0x0000417E      1  FF                                                   TERMINATOR_FF             
0x0000417F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004181      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004183      1  FF                                                   TERMINATOR_FF             
0x00004184      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004186      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004188      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000418A      1  FF                                                   TERMINATOR_FF             
0x0000418B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000418D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000418F      3  F300C7                                               IMM16_F3                  u16_be=199, u16_le=50944
0x00004192      1  FF                                                   TERMINATOR_FF             
0x00004193      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004195      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004197      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004199      1  FF                                                   TERMINATOR_FF             
0x0000419A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000419C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000419E      1  FF                                                   TERMINATOR_FF             
0x0000419F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000041A1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000041A3      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000041A5      1  FF                                                   TERMINATOR_FF             
0x000041A6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000041A8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000041AA      2  F240                                                 IMM8_F2                   u8=64, s8=64
0x000041AC      1  FF                                                   TERMINATOR_FF             
0x000041AD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000041AF      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000041B1      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000041B3      1  FF                                                   TERMINATOR_FF             
0x000041B4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000041B6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000041B8      1  FF                                                   TERMINATOR_FF             
0x000041B9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000041BB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000041BD      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x000041BF      1  FF                                                   TERMINATOR_FF             
0x000041C0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000041C2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000041C4      2  F229                                                 IMM8_F2                   u8=41, s8=41
0x000041C6      1  FF                                                   TERMINATOR_FF             
0x000041C7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000041C9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000041CB      1  FF                                                   TERMINATOR_FF             
0x000041CC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000041CE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000041D0      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000041D2      1  FF                                                   TERMINATOR_FF             
0x000041D3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000041D5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000041D7      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000041D9      1  FF                                                   TERMINATOR_FF             
0x000041DA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000041DC      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000041DE      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000041E0      1  FF                                                   TERMINATOR_FF             
0x000041E1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000041E3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000041E5      1  FF                                                   TERMINATOR_FF             
0x000041E6      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x000041E8      3  F10035                                               IMM16_F1                  u16_be=53, u16_le=13568
0x000041EB      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000041ED      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x000041EE      1  FF                                                   TERMINATOR_FF             
0x000041EF      2  0042                                                 WORD_00XX                 u16_be=66, low_byte=66
0x000041F1      1  DF                                                   OPAQUE_RAW_BYTES          bytes=DF
0x000041F2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000041F4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000041F6      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000041F8      1  FF                                                   TERMINATOR_FF             
0x000041F9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000041FB      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000041FD      3  F31A5F                                               IMM16_F3                  u16_be=6751, u16_le=24346
0x00004200      1  FF                                                   TERMINATOR_FF             
0x00004201      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004203      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004205    122  807882DA82AD82CD8DF093FA82CC82B182C682C98DDF88AB...  LEN8_STRING_CP932         length=120, text="ぼくは昨日のことに罪悪感をおぼえていた。\n気のせいだったとしても、ほうっておかず、声をかけるくらいはできたんじゃないか。"
0x0000427F      1  FF                                                   TERMINATOR_FF             
0x00004280      1  FF                                                   TERMINATOR_FF             
0x00004281      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004283      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004285      1  FF                                                   TERMINATOR_FF             
0x00004286      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004288      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000428A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000428C      1  FF                                                   TERMINATOR_FF             
0x0000428D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000428F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004291      3  F31A60                                               IMM16_F3                  u16_be=6752, u16_le=24602
0x00004294      1  FF                                                   TERMINATOR_FF             
0x00004295      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004297      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004299     58  803882BB82A48D6C82A682E982C6814182B182CC82DC82DC...  LEN8_STRING_CP932         length=56, text="そう考えると、このままにしておくわけには\nいかなかった。"
0x000042D3      1  FF                                                   TERMINATOR_FF             
0x000042D4      1  FF                                                   TERMINATOR_FF             
0x000042D5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000042D7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000042D9      1  FF                                                   TERMINATOR_FF             
0x000042DA      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x000042DC      2  0042                                                 WORD_00XX                 u16_be=66, low_byte=66
0x000042DE      1  EB                                                   OPAQUE_RAW_BYTES          bytes=EB
0x000042DF      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x000042E1      3  F10035                                               IMM16_F1                  u16_be=53, u16_le=13568
0x000042E4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000042E6      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x000042E7      1  FF                                                   TERMINATOR_FF             
0x000042E8      2  0042                                                 WORD_00XX                 u16_be=66, low_byte=66
0x000042EA      1  EB                                                   OPAQUE_RAW_BYTES          bytes=EB
0x000042EB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000042ED      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000042EF      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000042F1      1  FF                                                   TERMINATOR_FF             
0x000042F2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000042F4      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000042F6      3  F31A61                                               IMM16_F3                  u16_be=6753, u16_le=24858
0x000042F9      1  FF                                                   TERMINATOR_FF             
0x000042FA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000042FC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000042FE     51  8031817582C7815B82B582BD82F182C582B78148208DA282...  LEN8_STRING_CP932         length=49, text="「どーしたんです？ 困ってるんじゃないです\nか？」"
0x00004331      1  FF                                                   TERMINATOR_FF             
0x00004332      1  FF                                                   TERMINATOR_FF             
0x00004333      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004335      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004337      1  FF                                                   TERMINATOR_FF             
0x00004338      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000433A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000433C      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000433E      1  FF                                                   TERMINATOR_FF             
0x0000433F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004341      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004343      3  F300C0                                               IMM16_F3                  u16_be=192, u16_le=49152
0x00004346      1  FF                                                   TERMINATOR_FF             
0x00004347      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004349      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000434B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000434D      1  FF                                                   TERMINATOR_FF             
0x0000434E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004350      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004352      1  FF                                                   TERMINATOR_FF             
0x00004353      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004355      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004357      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00004359      1  FF                                                   TERMINATOR_FF             
0x0000435A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000435C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000435E      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00004360      1  FF                                                   TERMINATOR_FF             
0x00004361      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004363      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004365      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00004367      1  FF                                                   TERMINATOR_FF             
0x00004368      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000436A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000436C      1  FF                                                   TERMINATOR_FF             
0x0000436D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000436F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004371      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00004373      1  FF                                                   TERMINATOR_FF             
0x00004374      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004376      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004378      3  F31A62                                               IMM16_F3                  u16_be=6754, u16_le=25114
0x0000437B      1  FF                                                   TERMINATOR_FF             
0x0000437C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000437E      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004380      9  8007495A3030363630                                   LEN8_STRING_CP932         length=7, text="IZ00660"
0x00004389      1  FF                                                   TERMINATOR_FF             
0x0000438A      1  FF                                                   TERMINATOR_FF             
0x0000438B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000438D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000438F     40  8026817582A682C121212082A682C13F212082A682C182C6...  LEN8_STRING_CP932         length=38, text="「えっ!! えっ?! えっとお……私ぃ……」"
0x000043B7      1  FF                                                   TERMINATOR_FF             
0x000043B8      1  FF                                                   TERMINATOR_FF             
0x000043B9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000043BB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000043BD      1  FF                                                   TERMINATOR_FF             
0x000043BE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000043C0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000043C2      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000043C4      1  FF                                                   TERMINATOR_FF             
0x000043C5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000043C7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000043C9      3  F300CD                                               IMM16_F3                  u16_be=205, u16_le=52480
0x000043CC      1  FF                                                   TERMINATOR_FF             
0x000043CD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000043CF      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000043D1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000043D3      1  FF                                                   TERMINATOR_FF             
0x000043D4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000043D6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000043D8      1  FF                                                   TERMINATOR_FF             
0x000043D9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000043DB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000043DD      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000043DF      1  FF                                                   TERMINATOR_FF             
0x000043E0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000043E2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000043E4      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000043E6      1  FF                                                   TERMINATOR_FF             
0x000043E7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000043E9      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000043EB      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000043ED      1  FF                                                   TERMINATOR_FF             
0x000043EE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000043F0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000043F2      1  FF                                                   TERMINATOR_FF             
0x000043F3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000043F5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000043F7      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000043F9      1  FF                                                   TERMINATOR_FF             
0x000043FA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000043FC      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000043FE      3  F31A63                                               IMM16_F3                  u16_be=6755, u16_le=25370
0x00004401      1  FF                                                   TERMINATOR_FF             
0x00004402      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004404      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004406      9  8007495A3030363730                                   LEN8_STRING_CP932         length=7, text="IZ00670"
0x0000440F      1  FF                                                   TERMINATOR_FF             
0x00004410      1  FF                                                   TERMINATOR_FF             
0x00004411      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004413      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004415     26  801881758163816382C7815B82C882F182BE82EB815B8148...  LEN8_STRING_CP932         length=24, text="「……どーなんだろー？」"
0x0000442F      1  FF                                                   TERMINATOR_FF             
0x00004430      1  FF                                                   TERMINATOR_FF             
0x00004431      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004433      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004435      1  FF                                                   TERMINATOR_FF             
0x00004436      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004438      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000443A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000443C      1  FF                                                   TERMINATOR_FF             
0x0000443D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000443F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004441      3  F31A64                                               IMM16_F3                  u16_be=6756, u16_le=25626
0x00004444      1  FF                                                   TERMINATOR_FF             
0x00004445      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004447      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004449     86  8054905682B582A2936F8FEA906C95A882CC8F6F8CBB82C9...  LEN8_STRING_CP932         length=84, text="新しい登場人物の出現にパニクッてるみたいだ。けれど、ぼくともうひとりはいたって冷静。"
0x0000449F      1  FF                                                   TERMINATOR_FF             
0x000044A0      1  FF                                                   TERMINATOR_FF             
0x000044A1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000044A3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000044A5      1  FF                                                   TERMINATOR_FF             
0x000044A6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000044A8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000044AA      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000044AC      1  FF                                                   TERMINATOR_FF             
0x000044AD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000044AF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000044B1      3  F31A65                                               IMM16_F3                  u16_be=6757, u16_le=25882
0x000044B4      1  FF                                                   TERMINATOR_FF             
0x000044B5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000044B7      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000044B9      9  800745583435313330                                   LEN8_STRING_CP932         length=7, text="EX45130"
0x000044C2      1  FF                                                   TERMINATOR_FF             
0x000044C3      1  FF                                                   TERMINATOR_FF             
0x000044C4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000044C6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000044C8     27  801981758163816382F18160814820926D82E88D8782A281...  LEN8_STRING_CP932         length=25, text="「……ん～？ 知り合い？」"
0x000044E3      1  FF                                                   TERMINATOR_FF             
0x000044E4      1  FF                                                   TERMINATOR_FF             
0x000044E5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000044E7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000044E9      1  FF                                                   TERMINATOR_FF             
0x000044EA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000044EC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000044EE      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000044F0      1  FF                                                   TERMINATOR_FF             
0x000044F1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000044F3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000044F5      3  F300CE                                               IMM16_F3                  u16_be=206, u16_le=52736
0x000044F8      1  FF                                                   TERMINATOR_FF             
0x000044F9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000044FB      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000044FD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000044FF      1  FF                                                   TERMINATOR_FF             
0x00004500      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004502      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004504      1  FF                                                   TERMINATOR_FF             
0x00004505      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004507      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004509      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000450B      1  FF                                                   TERMINATOR_FF             
0x0000450C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000450E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004510      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00004512      1  FF                                                   TERMINATOR_FF             
0x00004513      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004515      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004517      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00004519      1  FF                                                   TERMINATOR_FF             
0x0000451A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000451C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000451E      1  FF                                                   TERMINATOR_FF             
0x0000451F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004521      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004523      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00004525      1  FF                                                   TERMINATOR_FF             
0x00004526      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004528      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000452A      3  F31A66                                               IMM16_F3                  u16_be=6758, u16_le=26138
0x0000452D      1  FF                                                   TERMINATOR_FF             
0x0000452E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004530      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004532      9  8007495A3030363830                                   LEN8_STRING_CP932         length=7, text="IZ00680"
0x0000453B      1  FF                                                   TERMINATOR_FF             
0x0000453C      1  FF                                                   TERMINATOR_FF             
0x0000453D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000453F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004541     35  8021817582A682C181482082A082CC815B814182A6816082...  LEN8_STRING_CP932         length=33, text="「えっ？ あのー、え～っとお……」"
0x00004564      1  FF                                                   TERMINATOR_FF             
0x00004565      1  FF                                                   TERMINATOR_FF             
0x00004566      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004568      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000456A      1  FF                                                   TERMINATOR_FF             
0x0000456B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000456D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000456F      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00004571      1  FF                                                   TERMINATOR_FF             
0x00004572      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004574      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004576      3  F31A67                                               IMM16_F3                  u16_be=6759, u16_le=26394
0x00004579      1  FF                                                   TERMINATOR_FF             
0x0000457A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000457C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000457E    112  806E817582A08141906590CA82C582B7814282B182CC906C...  LEN8_STRING_CP932         length=110, text="「あ、親戚です。この人はいとこの高山めぐみ\nさん。今日、おじいちゃんの法事だから、迎えに行けっていわれたんで」"
0x000045EE      1  FF                                                   TERMINATOR_FF             
0x000045EF      1  FF                                                   TERMINATOR_FF             
0x000045F0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000045F2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000045F4      1  FF                                                   TERMINATOR_FF             
0x000045F5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000045F7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000045F9      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000045FB      1  FF                                                   TERMINATOR_FF             
0x000045FC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000045FE      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004600      3  F31A68                                               IMM16_F3                  u16_be=6760, u16_le=26650
0x00004603      1  FF                                                   TERMINATOR_FF             
0x00004604      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004606      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004608     52  803295BD8B4382C5835883898358838982A482BB82AA8F6F...  LEN8_STRING_CP932         length=50, text="平気でスラスラうそが出る。\n我ながら感心するなあ。"
0x0000463C      1  FF                                                   TERMINATOR_FF             
0x0000463D      1  FF                                                   TERMINATOR_FF             
0x0000463E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004640      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004642      1  FF                                                   TERMINATOR_FF             
0x00004643      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004645      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004647      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00004649      1  FF                                                   TERMINATOR_FF             
0x0000464A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000464C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000464E      3  F300C4                                               IMM16_F3                  u16_be=196, u16_le=50176
0x00004651      1  FF                                                   TERMINATOR_FF             
0x00004652      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004654      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004656      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004658      1  FF                                                   TERMINATOR_FF             
0x00004659      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000465B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000465D      1  FF                                                   TERMINATOR_FF             
0x0000465E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004660      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004662      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00004664      1  FF                                                   TERMINATOR_FF             
0x00004665      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004667      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004669      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000466B      1  FF                                                   TERMINATOR_FF             
0x0000466C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000466E      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004670      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00004672      1  FF                                                   TERMINATOR_FF             
0x00004673      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004675      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004677      1  FF                                                   TERMINATOR_FF             
0x00004678      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000467A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000467C      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000467E      1  FF                                                   TERMINATOR_FF             
0x0000467F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004681      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004683      3  F31A69                                               IMM16_F3                  u16_be=6761, u16_le=26906
0x00004686      1  FF                                                   TERMINATOR_FF             
0x00004687      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004689      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000468B      9  8007495A3030363930                                   LEN8_STRING_CP932         length=7, text="IZ00690"
0x00004694      1  FF                                                   TERMINATOR_FF             
0x00004695      1  FF                                                   TERMINATOR_FF             
0x00004696      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004698      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000469A     42  8028817582A682C121212082A082C1814182A682C182C681...  LEN8_STRING_CP932         length=40, text="「えっ!! あっ、えっと？ ……そうなの？」"
0x000046C4      1  FF                                                   TERMINATOR_FF             
0x000046C5      1  FF                                                   TERMINATOR_FF             
0x000046C6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000046C8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000046CA      1  FF                                                   TERMINATOR_FF             
0x000046CB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000046CD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000046CF      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000046D1      1  FF                                                   TERMINATOR_FF             
0x000046D2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000046D4      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000046D6      3  F31A6A                                               IMM16_F3                  u16_be=6762, u16_le=27162
0x000046D9      1  FF                                                   TERMINATOR_FF             
0x000046DA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000046DC      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000046DE      9  800745583435313430                                   LEN8_STRING_CP932         length=7, text="EX45140"
0x000046E7      1  FF                                                   TERMINATOR_FF             
0x000046E8      1  FF                                                   TERMINATOR_FF             
0x000046E9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000046EB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000046ED     52  8032817582A08160814182BB82A482C882F182BE814282B6...  LEN8_STRING_CP932         length=50, text="「あ～、そうなんだ。じゃあ、また今度\nゆっくりね」"
0x00004721      1  FF                                                   TERMINATOR_FF             
0x00004722      1  FF                                                   TERMINATOR_FF             
0x00004723      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004725      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004727      1  FF                                                   TERMINATOR_FF             
0x00004728      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000472A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000472C      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000472E      1  FF                                                   TERMINATOR_FF             
0x0000472F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004731      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004733      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004735      1  FF                                                   TERMINATOR_FF             
0x00004736      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004738      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000473A      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000473C      1  FF                                                   TERMINATOR_FF             
0x0000473D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000473F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004741      1  FF                                                   TERMINATOR_FF             
0x00004742      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004744      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004746      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00004748      1  FF                                                   TERMINATOR_FF             
0x00004749      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000474B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000474D      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000474F      1  FF                                                   TERMINATOR_FF             
0x00004750      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004752      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004754      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00004756      1  FF                                                   TERMINATOR_FF             
0x00004757      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004759      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000475B      1  FF                                                   TERMINATOR_FF             
0x0000475C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000475E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004760      2  F20B                                                 IMM8_F2                   u8=11, s8=11
0x00004762      1  FF                                                   TERMINATOR_FF             
0x00004763      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004765      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004767      1  FF                                                   TERMINATOR_FF             
0x00004768      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000476A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000476C      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000476E      1  FF                                                   TERMINATOR_FF             
0x0000476F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004771      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004773      3  F31A6B                                               IMM16_F3                  u16_be=6763, u16_le=27418
0x00004776      1  FF                                                   TERMINATOR_FF             
0x00004777      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004779      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000477B    104  8066834C83838362836082CC82A88C5A82B382F182CD8141...  LEN8_STRING_CP932         length=102, text="キャッチのお兄さんは、愛想笑いをお姉さんに\n向け、つぎの獲物をさがしに行ってしまった。\nよかった……。"
0x000047E3      1  FF                                                   TERMINATOR_FF             
0x000047E4      1  FF                                                   TERMINATOR_FF             
0x000047E5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000047E7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000047E9      1  FF                                                   TERMINATOR_FF             
0x000047EA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000047EC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000047EE      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000047F0      1  FF                                                   TERMINATOR_FF             
0x000047F1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000047F3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000047F5      3  F300C6                                               IMM16_F3                  u16_be=198, u16_le=50688
0x000047F8      1  FF                                                   TERMINATOR_FF             
0x000047F9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000047FB      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000047FD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000047FF      1  FF                                                   TERMINATOR_FF             
0x00004800      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004802      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004804      1  FF                                                   TERMINATOR_FF             
0x00004805      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004807      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004809      2  F20A                                                 IMM8_F2                   u8=10, s8=10
0x0000480B      1  FF                                                   TERMINATOR_FF             
0x0000480C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000480E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004810      2  F210                                                 IMM8_F2                   u8=16, s8=16
0x00004812      1  FF                                                   TERMINATOR_FF             
0x00004813      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004815      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004817      1  FF                                                   TERMINATOR_FF             
0x00004818      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000481A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000481C      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000481E      1  FF                                                   TERMINATOR_FF             
0x0000481F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004821      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004823      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00004825      1  FF                                                   TERMINATOR_FF             
0x00004826      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004828      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000482A      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000482C      1  FF                                                   TERMINATOR_FF             
0x0000482D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000482F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004831      1  FF                                                   TERMINATOR_FF             
0x00004832      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004834      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004836      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00004838      1  FF                                                   TERMINATOR_FF             
0x00004839      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000483B      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000483D      3  F31A6C                                               IMM16_F3                  u16_be=6764, u16_le=27674
0x00004840      1  FF                                                   TERMINATOR_FF             
0x00004841      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004843      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004845      9  8007495A3030373030                                   LEN8_STRING_CP932         length=7, text="IZ00700"
0x0000484E      1  FF                                                   TERMINATOR_FF             
0x0000484F      1  FF                                                   TERMINATOR_FF             
0x00004850      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004852      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004854     16  800E817582A082CC815B816381638176                     LEN8_STRING_CP932         length=14, text="「あのー……」"
0x00004864      1  FF                                                   TERMINATOR_FF             
0x00004865      1  FF                                                   TERMINATOR_FF             
0x00004866      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004868      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000486A      1  FF                                                   TERMINATOR_FF             
0x0000486B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000486D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000486F      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00004871      1  FF                                                   TERMINATOR_FF             
0x00004872      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004874      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004876      3  F31A6D                                               IMM16_F3                  u16_be=6765, u16_le=27930
0x00004879      1  FF                                                   TERMINATOR_FF             
0x0000487A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000487C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000487E     44  802A82A88E6F82B382F182AA9053947A8B4382C8955C8FEE...  LEN8_STRING_CP932         length=42, text="お姉さんが心配気な表情で、ぼくを見ていた。"
0x000048AA      1  FF                                                   TERMINATOR_FF             
0x000048AB      1  FF                                                   TERMINATOR_FF             
0x000048AC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000048AE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000048B0      1  FF                                                   TERMINATOR_FF             
0x000048B1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000048B3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000048B5      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000048B7      1  FF                                                   TERMINATOR_FF             
0x000048B8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000048BA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000048BC      3  F300CB                                               IMM16_F3                  u16_be=203, u16_le=51968
0x000048BF      1  FF                                                   TERMINATOR_FF             
0x000048C0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000048C2      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000048C4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000048C6      1  FF                                                   TERMINATOR_FF             
0x000048C7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000048C9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000048CB      1  FF                                                   TERMINATOR_FF             
0x000048CC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000048CE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000048D0      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000048D2      1  FF                                                   TERMINATOR_FF             
0x000048D3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000048D5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000048D7      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000048D9      1  FF                                                   TERMINATOR_FF             
0x000048DA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000048DC      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000048DE      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000048E0      1  FF                                                   TERMINATOR_FF             
0x000048E1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000048E3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000048E5      1  FF                                                   TERMINATOR_FF             
0x000048E6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000048E8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000048EA      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000048EC      1  FF                                                   TERMINATOR_FF             
0x000048ED      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000048EF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000048F1      3  F31A6E                                               IMM16_F3                  u16_be=6766, u16_le=28186
0x000048F4      1  FF                                                   TERMINATOR_FF             
0x000048F5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000048F7      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000048F9      9  8007495A3030373130                                   LEN8_STRING_CP932         length=7, text="IZ00710"
0x00004902      1  FF                                                   TERMINATOR_FF             
0x00004903      1  FF                                                   TERMINATOR_FF             
0x00004904      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004906      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004908     16  800E817582A082CC815B816381638176                     LEN8_STRING_CP932         length=14, text="「あのー……」"
0x00004918      1  FF                                                   TERMINATOR_FF             
0x00004919      1  FF                                                   TERMINATOR_FF             
0x0000491A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000491C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000491E      1  FF                                                   TERMINATOR_FF             
0x0000491F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004921      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004923      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00004925      1  FF                                                   TERMINATOR_FF             
0x00004926      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004928      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000492A      3  F300C4                                               IMM16_F3                  u16_be=196, u16_le=50176
0x0000492D      1  FF                                                   TERMINATOR_FF             
0x0000492E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004930      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004932      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004934      1  FF                                                   TERMINATOR_FF             
0x00004935      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004937      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004939      1  FF                                                   TERMINATOR_FF             
0x0000493A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000493C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000493E      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00004940      1  FF                                                   TERMINATOR_FF             
0x00004941      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004943      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004945      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00004947      1  FF                                                   TERMINATOR_FF             
0x00004948      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000494A      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000494C      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000494E      1  FF                                                   TERMINATOR_FF             
0x0000494F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004951      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004953      1  FF                                                   TERMINATOR_FF             
0x00004954      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004956      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004958      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000495A      1  FF                                                   TERMINATOR_FF             
0x0000495B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000495D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000495F      3  F31A6F                                               IMM16_F3                  u16_be=6767, u16_le=28442
0x00004962      1  FF                                                   TERMINATOR_FF             
0x00004963      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004965      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004967      9  8007495A3030373230                                   LEN8_STRING_CP932         length=7, text="IZ00720"
0x00004970      1  FF                                                   TERMINATOR_FF             
0x00004971      1  FF                                                   TERMINATOR_FF             
0x00004972      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004974      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004976    118  807481758E84815B814182A882B682A282BF82E182F18141...  LEN8_STRING_CP932         length=116, text="「私ー、おじいちゃん、山形にいるんでー、\nぜんぜん元気だしー。たぶんーひと違いなんじゃないかなーって思うんですよー」"
0x000049EC      1  FF                                                   TERMINATOR_FF             
0x000049ED      1  FF                                                   TERMINATOR_FF             
0x000049EE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000049F0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000049F2      1  FF                                                   TERMINATOR_FF             
0x000049F3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000049F5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000049F7      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000049F9      1  FF                                                   TERMINATOR_FF             
0x000049FA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000049FC      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000049FE      3  F31A70                                               IMM16_F3                  u16_be=6768, u16_le=28698
0x00004A01      1  FF                                                   TERMINATOR_FF             
0x00004A02      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004A04      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004A06     14  800C82F181638163816381638148                         LEN8_STRING_CP932         length=12, text="ん…………？"
0x00004A14      1  FF                                                   TERMINATOR_FF             
0x00004A15      1  FF                                                   TERMINATOR_FF             
0x00004A16      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004A18      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004A1A      1  FF                                                   TERMINATOR_FF             
0x00004A1B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004A1D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004A1F      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00004A21      1  FF                                                   TERMINATOR_FF             
0x00004A22      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004A24      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004A26      3  F31A71                                               IMM16_F3                  u16_be=6769, u16_le=28954
0x00004A29      1  FF                                                   TERMINATOR_FF             
0x00004A2A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004A2C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004A2E     76  804A82A0829F814182BB82A482A282A482B182C682A98142...  LEN8_STRING_CP932         length=74, text="あぁ、そういうことか。\n気づいてない。\nぼくのウソをそのまま信じこんでる。"
0x00004A7A      1  FF                                                   TERMINATOR_FF             
0x00004A7B      1  FF                                                   TERMINATOR_FF             
0x00004A7C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004A7E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004A80      1  FF                                                   TERMINATOR_FF             
0x00004A81      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004A83      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004A85      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00004A87      1  FF                                                   TERMINATOR_FF             
0x00004A88      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004A8A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004A8C      3  F31A72                                               IMM16_F3                  u16_be=6770, u16_le=29210
0x00004A8F      1  FF                                                   TERMINATOR_FF             
0x00004A90      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004A92      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004A94     52  803282AB82C182C6814182B782B282AD82A282A2906C82C8...  LEN8_STRING_CP932         length=50, text="きっと、すごくいい人なんだろうな、この\nお姉さん。"
0x00004AC8      1  FF                                                   TERMINATOR_FF             
0x00004AC9      1  FF                                                   TERMINATOR_FF             
0x00004ACA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004ACC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004ACE      1  FF                                                   TERMINATOR_FF             
0x00004ACF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004AD1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004AD3      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00004AD5      1  FF                                                   TERMINATOR_FF             
0x00004AD6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004AD8      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004ADA      3  F31A73                                               IMM16_F3                  u16_be=6771, u16_le=29466
0x00004ADD      1  FF                                                   TERMINATOR_FF             
0x00004ADE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004AE0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004AE2     40  8026817582A0814182A282E2814182BB815B82E4815B82ED...  LEN8_STRING_CP932         length=38, text="「あ、いや、そーゆーわけじゃないんで」"
0x00004B0A      1  FF                                                   TERMINATOR_FF             
0x00004B0B      1  FF                                                   TERMINATOR_FF             
0x00004B0C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004B0E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004B10      1  FF                                                   TERMINATOR_FF             
0x00004B11      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004B13      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004B15      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00004B17      1  FF                                                   TERMINATOR_FF             
0x00004B18      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004B1A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004B1C      3  F31A74                                               IMM16_F3                  u16_be=6772, u16_le=29722
0x00004B1F      1  FF                                                   TERMINATOR_FF             
0x00004B20      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004B22      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004B24      9  8007495A3030373330                                   LEN8_STRING_CP932         length=7, text="IZ00730"
0x00004B2D      1  FF                                                   TERMINATOR_FF             
0x00004B2E      1  FF                                                   TERMINATOR_FF             
0x00004B2F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004B31      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004B33     49  802F817582A682C13F212082C582E0814182A282DC814182...  LEN8_STRING_CP932         length=47, text="「えっ?! でも、いま、そーいってました\nよねー」"
0x00004B64      1  FF                                                   TERMINATOR_FF             
0x00004B65      1  FF                                                   TERMINATOR_FF             
0x00004B66      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004B68      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004B6A      1  FF                                                   TERMINATOR_FF             
0x00004B6B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004B6D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004B6F      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00004B71      1  FF                                                   TERMINATOR_FF             
0x00004B72      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004B74      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004B76      3  F31A75                                               IMM16_F3                  u16_be=6773, u16_le=29978
0x00004B79      1  FF                                                   TERMINATOR_FF             
0x00004B7A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004B7C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004B7E     34  8020817582DC82A0814182BB82A482A282A282DC82B582BD...  LEN8_STRING_CP932         length=32, text="「まあ、そういいましたけど……」"
0x00004BA0      1  FF                                                   TERMINATOR_FF             
0x00004BA1      1  FF                                                   TERMINATOR_FF             
0x00004BA2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004BA4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004BA6      1  FF                                                   TERMINATOR_FF             
0x00004BA7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004BA9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004BAB      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00004BAD      1  FF                                                   TERMINATOR_FF             
0x00004BAE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004BB0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004BB2      3  F30215                                               IMM16_F3                  u16_be=533, u16_le=5378
0x00004BB5      1  FF                                                   TERMINATOR_FF             
0x00004BB6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004BB8      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004BBA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004BBC      1  FF                                                   TERMINATOR_FF             
0x00004BBD      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004BBF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004BC1      1  FF                                                   TERMINATOR_FF             
0x00004BC2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004BC4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004BC6      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00004BC8      1  FF                                                   TERMINATOR_FF             
0x00004BC9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004BCB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004BCD      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00004BCF      1  FF                                                   TERMINATOR_FF             
0x00004BD0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004BD2      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004BD4      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00004BD6      1  FF                                                   TERMINATOR_FF             
0x00004BD7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004BD9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004BDB      1  FF                                                   TERMINATOR_FF             
0x00004BDC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004BDE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004BE0      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00004BE2      1  FF                                                   TERMINATOR_FF             
0x00004BE3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004BE5      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004BE7      3  F31A76                                               IMM16_F3                  u16_be=6774, u16_le=30234
0x00004BEA      1  FF                                                   TERMINATOR_FF             
0x00004BEB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004BED      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004BEF      9  8007495A3030373430                                   LEN8_STRING_CP932         length=7, text="IZ00740"
0x00004BF8      1  FF                                                   TERMINATOR_FF             
0x00004BF9      1  FF                                                   TERMINATOR_FF             
0x00004BFA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004BFC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004BFE     24  8016817582A282C182C482BD82CC82C9815B816381638176     LEN8_STRING_CP932         length=22, text="「いってたのにー……」"
0x00004C16      1  FF                                                   TERMINATOR_FF             
0x00004C17      1  FF                                                   TERMINATOR_FF             
0x00004C18      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004C1A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004C1C      1  FF                                                   TERMINATOR_FF             
0x00004C1D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004C1F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004C21      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00004C23      1  FF                                                   TERMINATOR_FF             
0x00004C24      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004C26      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004C28      3  F31A77                                               IMM16_F3                  u16_be=6775, u16_le=30490
0x00004C2B      1  FF                                                   TERMINATOR_FF             
0x00004C2C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004C2E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004C30     46  802C82A88E6F82B382F182CD814182DA82AD82F082A482B3...  LEN8_STRING_CP932         length=44, text="お姉さんは、ぼくをうさんくさそうに見つめる。"
0x00004C5E      1  FF                                                   TERMINATOR_FF             
0x00004C5F      1  FF                                                   TERMINATOR_FF             
0x00004C60      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004C62      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004C64      1  FF                                                   TERMINATOR_FF             
0x00004C65      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004C67      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004C69      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00004C6B      1  FF                                                   TERMINATOR_FF             
0x00004C6C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004C6E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004C70      3  F300C0                                               IMM16_F3                  u16_be=192, u16_le=49152
0x00004C73      1  FF                                                   TERMINATOR_FF             
0x00004C74      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004C76      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004C78      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004C7A      1  FF                                                   TERMINATOR_FF             
0x00004C7B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004C7D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004C7F      1  FF                                                   TERMINATOR_FF             
0x00004C80      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004C82      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004C84      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00004C86      1  FF                                                   TERMINATOR_FF             
0x00004C87      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004C89      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004C8B      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00004C8D      1  FF                                                   TERMINATOR_FF             
0x00004C8E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004C90      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004C92      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00004C94      1  FF                                                   TERMINATOR_FF             
0x00004C95      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004C97      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004C99      1  FF                                                   TERMINATOR_FF             
0x00004C9A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004C9C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004C9E      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00004CA0      1  FF                                                   TERMINATOR_FF             
0x00004CA1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004CA3      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004CA5      3  F31A78                                               IMM16_F3                  u16_be=6776, u16_le=30746
0x00004CA8      1  FF                                                   TERMINATOR_FF             
0x00004CA9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004CAB      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004CAD      9  8007495A3030373530                                   LEN8_STRING_CP932         length=7, text="IZ00750"
0x00004CB6      1  FF                                                   TERMINATOR_FF             
0x00004CB7      1  FF                                                   TERMINATOR_FF             
0x00004CB8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004CBA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004CBC     30  801C817582A081608163816382E082B582A982B582C48160...  LEN8_STRING_CP932         length=28, text="「あ～……もしかして～……」"
0x00004CDA      1  FF                                                   TERMINATOR_FF             
0x00004CDB      1  FF                                                   TERMINATOR_FF             
0x00004CDC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004CDE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004CE0      1  FF                                                   TERMINATOR_FF             
0x00004CE1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004CE3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004CE5      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00004CE7      1  FF                                                   TERMINATOR_FF             
0x00004CE8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004CEA      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004CEC      3  F31A79                                               IMM16_F3                  u16_be=6777, u16_le=31002
0x00004CEF      1  FF                                                   TERMINATOR_FF             
0x00004CF0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004CF2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004CF4     44  802A82E682A982C182BD81425C6E82E682A482E282AD8B43...  LEN8_STRING_CP932         length=42, text="よかった。\nようやく気づいてくれたみたい。"
0x00004D20      1  FF                                                   TERMINATOR_FF             
0x00004D21      1  FF                                                   TERMINATOR_FF             
0x00004D22      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004D24      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004D26      1  FF                                                   TERMINATOR_FF             
0x00004D27      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004D29      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004D2B      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00004D2D      1  FF                                                   TERMINATOR_FF             
0x00004D2E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004D30      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004D32      3  F3020B                                               IMM16_F3                  u16_be=523, u16_le=2818
0x00004D35      1  FF                                                   TERMINATOR_FF             
0x00004D36      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004D38      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004D3A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004D3C      1  FF                                                   TERMINATOR_FF             
0x00004D3D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004D3F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004D41      1  FF                                                   TERMINATOR_FF             
0x00004D42      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004D44      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004D46      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00004D48      1  FF                                                   TERMINATOR_FF             
0x00004D49      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004D4B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004D4D      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00004D4F      1  FF                                                   TERMINATOR_FF             
0x00004D50      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004D52      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004D54      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00004D56      1  FF                                                   TERMINATOR_FF             
0x00004D57      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004D59      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004D5B      1  FF                                                   TERMINATOR_FF             
0x00004D5C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004D5E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004D60      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00004D62      1  FF                                                   TERMINATOR_FF             
0x00004D63      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004D65      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004D67      3  F31A7A                                               IMM16_F3                  u16_be=6778, u16_le=31258
0x00004D6A      1  FF                                                   TERMINATOR_FF             
0x00004D6B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004D6D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004D6F      9  8007495A3030373630                                   LEN8_STRING_CP932         length=7, text="IZ00760"
0x00004D78      1  FF                                                   TERMINATOR_FF             
0x00004D79      1  FF                                                   TERMINATOR_FF             
0x00004D7A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004D7C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004D7E     56  8036817583698393837082B582C482E982F182C582B582E5...  LEN8_STRING_CP932         length=54, text="「ナンパしてるんでしょー。やだなー、こどものくせにー」"
0x00004DB6      1  FF                                                   TERMINATOR_FF             
0x00004DB7      1  FF                                                   TERMINATOR_FF             
0x00004DB8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004DBA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004DBC      1  FF                                                   TERMINATOR_FF             
0x00004DBD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004DBF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004DC1      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00004DC3      1  FF                                                   TERMINATOR_FF             
0x00004DC4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004DC6      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004DC8      3  F31A7B                                               IMM16_F3                  u16_be=6779, u16_le=31514
0x00004DCB      1  FF                                                   TERMINATOR_FF             
0x00004DCC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004DCE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004DD0     24  801682ED82C1814182B782B282A2834A839388E182A28142     LEN8_STRING_CP932         length=22, text="わっ、すごいカン違い。"
0x00004DE8      1  FF                                                   TERMINATOR_FF             
0x00004DE9      1  FF                                                   TERMINATOR_FF             
0x00004DEA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004DEC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004DEE      1  FF                                                   TERMINATOR_FF             
0x00004DEF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004DF1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004DF3      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00004DF5      1  FF                                                   TERMINATOR_FF             
0x00004DF6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004DF8      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004DFA      3  F31A7C                                               IMM16_F3                  u16_be=6780, u16_le=31770
0x00004DFD      1  FF                                                   TERMINATOR_FF             
0x00004DFE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004E00      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004E02     18  8010817588E182A282DC82B782E621218176                 LEN8_STRING_CP932         length=16, text="「違いますよ!!」"
0x00004E14      1  FF                                                   TERMINATOR_FF             
0x00004E15      1  FF                                                   TERMINATOR_FF             
0x00004E16      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004E18      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004E1A      1  FF                                                   TERMINATOR_FF             
0x00004E1B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004E1D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004E1F      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00004E21      1  FF                                                   TERMINATOR_FF             
0x00004E22      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004E24      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004E26      3  F31A7D                                               IMM16_F3                  u16_be=6781, u16_le=32026
0x00004E29      1  FF                                                   TERMINATOR_FF             
0x00004E2A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004E2C      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004E2E      9  8007495A3030373731                                   LEN8_STRING_CP932         length=7, text="IZ00771"
0x00004E37      1  FF                                                   TERMINATOR_FF             
0x00004E38      1  FF                                                   TERMINATOR_FF             
0x00004E39      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004E3B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004E3D    107  8069817582A68160814182BE82C182C4815B96408E96815B...  LEN8_STRING_CP932         length=105, text="「え～、だってー法事ーとかいってー、話し\nかけて来たじゃなーいー？ ヘンなことゆーなー\nって思ったんだー」"
0x00004EA8      1  FF                                                   TERMINATOR_FF             
0x00004EA9      1  FF                                                   TERMINATOR_FF             
0x00004EAA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004EAC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004EAE      1  FF                                                   TERMINATOR_FF             
0x00004EAF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004EB1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004EB3      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00004EB5      1  FF                                                   TERMINATOR_FF             
0x00004EB6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004EB8      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004EBA      3  F31A7E                                               IMM16_F3                  u16_be=6782, u16_le=32282
0x00004EBD      1  FF                                                   TERMINATOR_FF             
0x00004EBE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004EC0      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004EC2      9  8007495A3030373732                                   LEN8_STRING_CP932         length=7, text="IZ00772"
0x00004ECB      1  FF                                                   TERMINATOR_FF             
0x00004ECC      1  FF                                                   TERMINATOR_FF             
0x00004ECD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004ECF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004ED1    132  8082817596408E9682CC83698393837082C882F182A995B7...  LEN8_STRING_CP932         length=130, text="「法事のナンパなんか聞いたことないよー。\nなーにー、仕出しのお弁当いっしょに食べるの？おかしーよ、そんなのー。ぜーったい、ヘンー」"
0x00004F55      1  FF                                                   TERMINATOR_FF             
0x00004F56      1  FF                                                   TERMINATOR_FF             
0x00004F57      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004F59      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004F5B      1  FF                                                   TERMINATOR_FF             
0x00004F5C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004F5E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004F60      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00004F62      1  FF                                                   TERMINATOR_FF             
0x00004F63      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004F65      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004F67      3  F31A7F                                               IMM16_F3                  u16_be=6783, u16_le=32538
0x00004F6A      1  FF                                                   TERMINATOR_FF             
0x00004F6B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004F6D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004F6F     46  802C82BE82A982E78163816388E182A482C182C482A282C1...  LEN8_STRING_CP932         length=44, text="だから……違うっていってるんだけどなあ……。"
0x00004F9D      1  FF                                                   TERMINATOR_FF             
0x00004F9E      1  FF                                                   TERMINATOR_FF             
0x00004F9F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004FA1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004FA3      1  FF                                                   TERMINATOR_FF             
0x00004FA4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004FA6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004FA8      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00004FAA      1  FF                                                   TERMINATOR_FF             
0x00004FAB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004FAD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004FAF      3  F300C1                                               IMM16_F3                  u16_be=193, u16_le=49408
0x00004FB2      1  FF                                                   TERMINATOR_FF             
0x00004FB3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004FB5      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004FB7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004FB9      1  FF                                                   TERMINATOR_FF             
0x00004FBA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004FBC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004FBE      1  FF                                                   TERMINATOR_FF             
0x00004FBF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004FC1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004FC3      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00004FC5      1  FF                                                   TERMINATOR_FF             
0x00004FC6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004FC8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004FCA      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00004FCC      1  FF                                                   TERMINATOR_FF             
0x00004FCD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004FCF      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004FD1      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00004FD3      1  FF                                                   TERMINATOR_FF             
0x00004FD4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00004FD6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00004FD8      1  FF                                                   TERMINATOR_FF             
0x00004FD9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004FDB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004FDD      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00004FDF      1  FF                                                   TERMINATOR_FF             
0x00004FE0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00004FE2      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00004FE4      3  F31A80                                               IMM16_F3                  u16_be=6784, u16_le=32794
0x00004FE7      1  FF                                                   TERMINATOR_FF             
0x00004FE8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004FEA      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00004FEC      9  8007495A3030373830                                   LEN8_STRING_CP932         length=7, text="IZ00780"
0x00004FF5      1  FF                                                   TERMINATOR_FF             
0x00004FF6      1  FF                                                   TERMINATOR_FF             
0x00004FF7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004FF9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00004FFB     24  8016817582A08163816382DC82B382A9815B816381638176     LEN8_STRING_CP932         length=22, text="「あ……まさかー……」"
0x00005013      1  FF                                                   TERMINATOR_FF             
0x00005014      1  FF                                                   TERMINATOR_FF             
0x00005015      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005017      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005019      1  FF                                                   TERMINATOR_FF             
0x0000501A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000501C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000501E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00005020      1  FF                                                   TERMINATOR_FF             
0x00005021      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005023      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005025      3  F31A81                                               IMM16_F3                  u16_be=6785, u16_le=33050
0x00005028      1  FF                                                   TERMINATOR_FF             
0x00005029      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000502B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000502D     46  802C82E082A482A082AB82E782DF82BD81425C6E82C882C9...  LEN8_STRING_CP932         length=44, text="もうあきらめた。\nなにをいわれても驚かない。"
0x0000505B      1  FF                                                   TERMINATOR_FF             
0x0000505C      1  FF                                                   TERMINATOR_FF             
0x0000505D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000505F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005061      1  FF                                                   TERMINATOR_FF             
0x00005062      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005064      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005066      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00005068      1  FF                                                   TERMINATOR_FF             
0x00005069      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000506B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000506D      3  F300C0                                               IMM16_F3                  u16_be=192, u16_le=49152
0x00005070      1  FF                                                   TERMINATOR_FF             
0x00005071      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005073      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005075      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005077      1  FF                                                   TERMINATOR_FF             
0x00005078      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000507A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000507C      1  FF                                                   TERMINATOR_FF             
0x0000507D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000507F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005081      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00005083      1  FF                                                   TERMINATOR_FF             
0x00005084      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005086      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005088      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000508A      1  FF                                                   TERMINATOR_FF             
0x0000508B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000508D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000508F      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00005091      1  FF                                                   TERMINATOR_FF             
0x00005092      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005094      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005096      1  FF                                                   TERMINATOR_FF             
0x00005097      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005099      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000509B      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000509D      1  FF                                                   TERMINATOR_FF             
0x0000509E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000050A0      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000050A2      3  F31A82                                               IMM16_F3                  u16_be=6786, u16_le=33306
0x000050A5      1  FF                                                   TERMINATOR_FF             
0x000050A6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000050A8      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000050AA      9  8007495A3030373930                                   LEN8_STRING_CP932         length=7, text="IZ00790"
0x000050B3      1  FF                                                   TERMINATOR_FF             
0x000050B4      1  FF                                                   TERMINATOR_FF             
0x000050B5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000050B7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000050B9     38  802481758E8482CC82B182C6816381638F9582AF82C482AD...  LEN8_STRING_CP932         length=36, text="「私のこと……助けてくれたとかー？」"
0x000050DF      1  FF                                                   TERMINATOR_FF             
0x000050E0      1  FF                                                   TERMINATOR_FF             
0x000050E1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000050E3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000050E5      1  FF                                                   TERMINATOR_FF             
0x000050E6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000050E8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000050EA      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000050EC      1  FF                                                   TERMINATOR_FF             
0x000050ED      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000050EF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000050F1      3  F31A83                                               IMM16_F3                  u16_be=6787, u16_le=33562
0x000050F4      1  FF                                                   TERMINATOR_FF             
0x000050F5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000050F7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000050F9     50  803082ED82C1814182E682A482E282AD8B4382C382A282C4...  LEN8_STRING_CP932         length=48, text="わっ、ようやく気づいてくれたよ。\nちょっと感激。"
0x0000512B      1  FF                                                   TERMINATOR_FF             
0x0000512C      1  FF                                                   TERMINATOR_FF             
0x0000512D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000512F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005131      1  FF                                                   TERMINATOR_FF             
0x00005132      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005134      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005136      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00005138      1  FF                                                   TERMINATOR_FF             
0x00005139      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000513B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000513D      3  F300CE                                               IMM16_F3                  u16_be=206, u16_le=52736
0x00005140      1  FF                                                   TERMINATOR_FF             
0x00005141      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005143      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005145      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005147      1  FF                                                   TERMINATOR_FF             
0x00005148      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000514A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000514C      1  FF                                                   TERMINATOR_FF             
0x0000514D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000514F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005151      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00005153      1  FF                                                   TERMINATOR_FF             
0x00005154      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005156      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005158      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000515A      1  FF                                                   TERMINATOR_FF             
0x0000515B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000515D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000515F      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00005161      1  FF                                                   TERMINATOR_FF             
0x00005162      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005164      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005166      1  FF                                                   TERMINATOR_FF             
0x00005167      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005169      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000516B      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000516D      1  FF                                                   TERMINATOR_FF             
0x0000516E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005170      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005172      3  F31A84                                               IMM16_F3                  u16_be=6788, u16_le=33818
0x00005175      1  FF                                                   TERMINATOR_FF             
0x00005176      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005178      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000517A     22  801481758163816382BB815B82C882CC815B81488176         LEN8_STRING_CP932         length=20, text="「……そーなのー？」"
0x00005190      1  FF                                                   TERMINATOR_FF             
0x00005191      1  FF                                                   TERMINATOR_FF             
0x00005192      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005194      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005196      9  8007495A3030383030                                   LEN8_STRING_CP932         length=7, text="IZ00800"
0x0000519F      1  FF                                                   TERMINATOR_FF             
0x000051A0      1  FF                                                   TERMINATOR_FF             
0x000051A1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000051A3      2  0004                                                 WORD_00XX                 u16_be=4, low_byte=4
0x000051A5     24  801682DC82A0814182BB82A482C882F182C582B782AF82C7     LEN8_STRING_CP932         length=22, text="まあ、そうなんですけど"
0x000051BD      1  FF                                                   TERMINATOR_FF             
0x000051BE      1  FF                                                   TERMINATOR_FF             
0x000051BF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000051C1      2  000A                                                 WORD_00XX                 u16_be=10, low_byte=10
0x000051C3      3  F31A85                                               IMM16_F3                  u16_be=6789, u16_le=34074
0x000051C6      1  FF                                                   TERMINATOR_FF             
0x000051C7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000051C9      2  0005                                                 WORD_00XX                 u16_be=5, low_byte=5
0x000051CB     24  801682BB82F182C882B182C682C882A282C582B782AF82C7     LEN8_STRING_CP932         length=22, text="そんなことないですけど"
0x000051E3      1  FF                                                   TERMINATOR_FF             
0x000051E4      1  FF                                                   TERMINATOR_FF             
0x000051E5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000051E7      2  000B                                                 WORD_00XX                 u16_be=11, low_byte=11
0x000051E9      3  F31A86                                               IMM16_F3                  u16_be=6790, u16_le=34330
0x000051EC      1  FF                                                   TERMINATOR_FF             
0x000051ED      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000051EF      2  0008                                                 WORD_00XX                 u16_be=8, low_byte=8
0x000051F1      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x000051F3      1  FF                                                   TERMINATOR_FF             
0x000051F4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000051F6      2  0009                                                 WORD_00XX                 u16_be=9, low_byte=9
0x000051F8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000051FA      1  FF                                                   TERMINATOR_FF             
0x000051FB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000051FD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000051FF      2  F213                                                 IMM8_F2                   u8=19, s8=19
0x00005201      1  FF                                                   TERMINATOR_FF             
0x00005202      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005204      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005206      1  FF                                                   TERMINATOR_FF             
0x00005207      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00005209      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x0000520C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000520E      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x0000520F      1  FF                                                   TERMINATOR_FF             
0x00005210      2  005B                                                 WORD_00XX                 u16_be=91, low_byte=91
0x00005212      1  95                                                   OPAQUE_RAW_BYTES          bytes=95
0x00005213      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00005215      3  F10035                                               IMM16_F1                  u16_be=53, u16_le=13568
0x00005218      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000521A      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x0000521B      1  FF                                                   TERMINATOR_FF             
0x0000521C      2  005B                                                 WORD_00XX                 u16_be=91, low_byte=91
0x0000521E      1  84                                                   OPAQUE_RAW_BYTES          bytes=84
0x0000521F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005221      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005223      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00005225      1  FF                                                   TERMINATOR_FF             
0x00005226      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005228      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000522A      3  F300CB                                               IMM16_F3                  u16_be=203, u16_le=51968
0x0000522D      1  FF                                                   TERMINATOR_FF             
0x0000522E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005230      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005232      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005234      1  FF                                                   TERMINATOR_FF             
0x00005235      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005237      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005239      1  FF                                                   TERMINATOR_FF             
0x0000523A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000523C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000523E      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00005240      1  FF                                                   TERMINATOR_FF             
0x00005241      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005243      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005245      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00005247      1  FF                                                   TERMINATOR_FF             
0x00005248      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000524A      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000524C      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000524E      1  FF                                                   TERMINATOR_FF             
0x0000524F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005251      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005253      1  FF                                                   TERMINATOR_FF             
0x00005254      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005256      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005258      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000525A      1  FF                                                   TERMINATOR_FF             
0x0000525B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000525D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000525F      3  F31A87                                               IMM16_F3                  u16_be=6791, u16_le=34586
0x00005262      1  FF                                                   TERMINATOR_FF             
0x00005263      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005265      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005267      9  8007495A3030383130                                   LEN8_STRING_CP932         length=7, text="IZ00810"
0x00005270      1  FF                                                   TERMINATOR_FF             
0x00005271      1  FF                                                   TERMINATOR_FF             
0x00005272      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005274      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005276     77  804B817582A6816082C13F212082C582E08163816382C781...  LEN8_STRING_CP932         length=75, text="「え～っ?! でも……どーしてー？\n私、助けられるよーなこと、なにかしたー？」"
0x000052C3      1  FF                                                   TERMINATOR_FF             
0x000052C4      1  FF                                                   TERMINATOR_FF             
0x000052C5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000052C7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000052C9      1  FF                                                   TERMINATOR_FF             
0x000052CA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000052CC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000052CE      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000052D0      1  FF                                                   TERMINATOR_FF             
0x000052D1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000052D3      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000052D5      3  F31A88                                               IMM16_F3                  u16_be=6792, u16_le=34842
0x000052D8      1  FF                                                   TERMINATOR_FF             
0x000052D9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000052DB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000052DD     40  802681758DF093FA814195CF82C882B182C6814182A082E8...  LEN8_STRING_CP932         length=38, text="「昨日、変なこと、ありませんでした？」"
0x00005305      1  FF                                                   TERMINATOR_FF             
0x00005306      1  FF                                                   TERMINATOR_FF             
0x00005307      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005309      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000530B      1  FF                                                   TERMINATOR_FF             
0x0000530C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000530E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005310      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00005312      1  FF                                                   TERMINATOR_FF             
0x00005313      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005315      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005317      3  F300CE                                               IMM16_F3                  u16_be=206, u16_le=52736
0x0000531A      1  FF                                                   TERMINATOR_FF             
0x0000531B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000531D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000531F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005321      1  FF                                                   TERMINATOR_FF             
0x00005322      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005324      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005326      1  FF                                                   TERMINATOR_FF             
0x00005327      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005329      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000532B      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000532D      1  FF                                                   TERMINATOR_FF             
0x0000532E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005330      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005332      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00005334      1  FF                                                   TERMINATOR_FF             
0x00005335      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005337      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005339      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000533B      1  FF                                                   TERMINATOR_FF             
0x0000533C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000533E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005340      1  FF                                                   TERMINATOR_FF             
0x00005341      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005343      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005345      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00005347      1  FF                                                   TERMINATOR_FF             
0x00005348      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000534A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000534C      3  F31A89                                               IMM16_F3                  u16_be=6793, u16_le=35098
0x0000534F      1  FF                                                   TERMINATOR_FF             
0x00005350      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005352      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005354      9  8007495A3030383230                                   LEN8_STRING_CP932         length=7, text="IZ00820"
0x0000535D      1  FF                                                   TERMINATOR_FF             
0x0000535E      1  FF                                                   TERMINATOR_FF             
0x0000535F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005361      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005363     52  8032817582F18148208DF093FA8148208163816382A682C1...  LEN8_STRING_CP932         length=50, text="「ん？ 昨日？ ……えっとねー、なんか\nあったっけ」"
0x00005397      1  FF                                                   TERMINATOR_FF             
0x00005398      1  FF                                                   TERMINATOR_FF             
0x00005399      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000539B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000539D      1  FF                                                   TERMINATOR_FF             
0x0000539E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000053A0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000053A2      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000053A4      1  FF                                                   TERMINATOR_FF             
0x000053A5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000053A7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000053A9      3  F300C1                                               IMM16_F3                  u16_be=193, u16_le=49408
0x000053AC      1  FF                                                   TERMINATOR_FF             
0x000053AD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000053AF      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000053B1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000053B3      1  FF                                                   TERMINATOR_FF             
0x000053B4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000053B6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000053B8      1  FF                                                   TERMINATOR_FF             
0x000053B9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000053BB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000053BD      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000053BF      1  FF                                                   TERMINATOR_FF             
0x000053C0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000053C2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000053C4      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000053C6      1  FF                                                   TERMINATOR_FF             
0x000053C7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000053C9      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000053CB      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000053CD      1  FF                                                   TERMINATOR_FF             
0x000053CE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000053D0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000053D2      1  FF                                                   TERMINATOR_FF             
0x000053D3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000053D5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000053D7      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000053D9      1  FF                                                   TERMINATOR_FF             
0x000053DA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000053DC      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000053DE      3  F31A8A                                               IMM16_F3                  u16_be=6794, u16_le=35354
0x000053E1      1  FF                                                   TERMINATOR_FF             
0x000053E2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000053E4      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000053E6      9  8007495A3030383331                                   LEN8_STRING_CP932         length=7, text="IZ00831"
0x000053EF      1  FF                                                   TERMINATOR_FF             
0x000053F0      1  FF                                                   TERMINATOR_FF             
0x000053F1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000053F3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000053F5    112  806E817582A8928B904882D782C982A282C182BD82E79744...  LEN8_STRING_CP932         length=110, text="「お昼食べにいったら優輝ちゃんがおサイフ\n忘れたーってたてかえたでしょー。次長に\nプチセクハラされたでしょー」"
0x00005465      1  FF                                                   TERMINATOR_FF             
0x00005466      1  FF                                                   TERMINATOR_FF             
0x00005467      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005469      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000546B      1  FF                                                   TERMINATOR_FF             
0x0000546C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000546E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005470      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00005472      1  FF                                                   TERMINATOR_FF             
0x00005473      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005475      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005477      3  F31A8B                                               IMM16_F3                  u16_be=6795, u16_le=35610
0x0000547A      1  FF                                                   TERMINATOR_FF             
0x0000547B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000547D      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000547F      9  8007495A3030383332                                   LEN8_STRING_CP932         length=7, text="IZ00832"
0x00005488      1  FF                                                   TERMINATOR_FF             
0x00005489      1  FF                                                   TERMINATOR_FF             
0x0000548A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000548C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000548E    130  808081758B4182E882CC835283938372836A82C5815B82A8...  LEN8_STRING_CP932         length=128, text="「帰りのコンビニでーおつり多くもらいすぎ\nちゃったから、返しにいったでしょー。あとはー……タイマー失敗して野球録画しちゃったー」"
0x00005510      1  FF                                                   TERMINATOR_FF             
0x00005511      1  FF                                                   TERMINATOR_FF             
0x00005512      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005514      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005516      1  FF                                                   TERMINATOR_FF             
0x00005517      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005519      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000551B      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000551D      1  FF                                                   TERMINATOR_FF             
0x0000551E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005520      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005522      3  F31A8C                                               IMM16_F3                  u16_be=6796, u16_le=35866
0x00005525      1  FF                                                   TERMINATOR_FF             
0x00005526      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005528      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000552A      9  8007495A3030383333                                   LEN8_STRING_CP932         length=7, text="IZ00833"
0x00005533      1  FF                                                   TERMINATOR_FF             
0x00005534      1  FF                                                   TERMINATOR_FF             
0x00005535      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005537      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005539     28  801A817582A682C182C6815B82BB82EA82A982E7815B8163...  LEN8_STRING_CP932         length=26, text="「えっとーそれからー……」"
0x00005555      1  FF                                                   TERMINATOR_FF             
0x00005556      1  FF                                                   TERMINATOR_FF             
0x00005557      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005559      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000555B      1  FF                                                   TERMINATOR_FF             
0x0000555C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000555E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005560      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00005562      1  FF                                                   TERMINATOR_FF             
0x00005563      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005565      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005567      3  F31A8D                                               IMM16_F3                  u16_be=6797, u16_le=36122
0x0000556A      1  FF                                                   TERMINATOR_FF             
0x0000556B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000556D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000556F     80  804E81758DF093FA8141897782C58CA982A982AF82BD82F1...  LEN8_STRING_CP932         length=78, text="「昨日、駅で見かけたんですよ。痴漢っぽい人が、ぴったり後ろにくっついてたから」"
0x000055BF      1  FF                                                   TERMINATOR_FF             
0x000055C0      1  FF                                                   TERMINATOR_FF             
0x000055C1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000055C3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000055C5      1  FF                                                   TERMINATOR_FF             
0x000055C6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000055C8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000055CA      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000055CC      1  FF                                                   TERMINATOR_FF             
0x000055CD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000055CF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000055D1      3  F300C6                                               IMM16_F3                  u16_be=198, u16_le=50688
0x000055D4      1  FF                                                   TERMINATOR_FF             
0x000055D5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000055D7      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000055D9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000055DB      1  FF                                                   TERMINATOR_FF             
0x000055DC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000055DE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000055E0      1  FF                                                   TERMINATOR_FF             
0x000055E1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000055E3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000055E5      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000055E7      1  FF                                                   TERMINATOR_FF             
0x000055E8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000055EA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000055EC      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000055EE      1  FF                                                   TERMINATOR_FF             
0x000055EF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000055F1      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000055F3      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000055F5      1  FF                                                   TERMINATOR_FF             
0x000055F6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000055F8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000055FA      1  FF                                                   TERMINATOR_FF             
0x000055FB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000055FD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000055FF      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00005601      1  FF                                                   TERMINATOR_FF             
0x00005602      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005604      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005606      3  F31A8E                                               IMM16_F3                  u16_be=6798, u16_le=36378
0x00005609      1  FF                                                   TERMINATOR_FF             
0x0000560A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000560C      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000560E      9  8007495A3030383431                                   LEN8_STRING_CP932         length=7, text="IZ00841"
0x00005617      1  FF                                                   TERMINATOR_FF             
0x00005618      1  FF                                                   TERMINATOR_FF             
0x00005619      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000561B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000561D     76  804A817582A03F212082A082EA82C13F212082E282C182CF...  LEN8_STRING_CP932         length=74, text="「あ?! あれっ?! やっぱり、痴漢だった？\nなんかね、そっかなーって思ったの」"
0x00005669      1  FF                                                   TERMINATOR_FF             
0x0000566A      1  FF                                                   TERMINATOR_FF             
0x0000566B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000566D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000566F      1  FF                                                   TERMINATOR_FF             
0x00005670      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005672      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005674      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00005676      1  FF                                                   TERMINATOR_FF             
0x00005677      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005679      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000567B      3  F31A8F                                               IMM16_F3                  u16_be=6799, u16_le=36634
0x0000567E      1  FF                                                   TERMINATOR_FF             
0x0000567F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005681      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005683      9  8007495A3030383432                                   LEN8_STRING_CP932         length=7, text="IZ00842"
0x0000568C      1  FF                                                   TERMINATOR_FF             
0x0000568D      1  FF                                                   TERMINATOR_FF             
0x0000568E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005690      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005692    102  8064817582C582E0814183748363815B82C182DB82A282A9...  LEN8_STRING_CP932         length=100, text="「でも、フツーっぽいから、気のせーかなーって思ったのよー……えっ!! えっ?! なに?!\nなにしてたのっ?!」"
0x000056F8      1  FF                                                   TERMINATOR_FF             
0x000056F9      1  FF                                                   TERMINATOR_FF             
0x000056FA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000056FC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000056FE      1  FF                                                   TERMINATOR_FF             
0x000056FF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005701      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005703      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00005705      1  FF                                                   TERMINATOR_FF             
0x00005706      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005708      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000570A      3  F31A90                                               IMM16_F3                  u16_be=6800, u16_le=36890
0x0000570D      1  FF                                                   TERMINATOR_FF             
0x0000570E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005710      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005712     44  802A81758358834A815B836782CC928681418E42896582B5...  LEN8_STRING_CP932         length=42, text="「スカートの中、撮影してるような気がして」"
0x0000573E      1  FF                                                   TERMINATOR_FF             
0x0000573F      1  FF                                                   TERMINATOR_FF             
0x00005740      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005742      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005744      1  FF                                                   TERMINATOR_FF             
0x00005745      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005747      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005749      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000574B      1  FF                                                   TERMINATOR_FF             
0x0000574C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000574E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005750      3  F300C5                                               IMM16_F3                  u16_be=197, u16_le=50432
0x00005753      1  FF                                                   TERMINATOR_FF             
0x00005754      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005756      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005758      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000575A      1  FF                                                   TERMINATOR_FF             
0x0000575B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000575D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000575F      1  FF                                                   TERMINATOR_FF             
0x00005760      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005762      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005764      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00005766      1  FF                                                   TERMINATOR_FF             
0x00005767      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005769      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000576B      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000576D      1  FF                                                   TERMINATOR_FF             
0x0000576E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005770      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005772      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00005774      1  FF                                                   TERMINATOR_FF             
0x00005775      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005777      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005779      1  FF                                                   TERMINATOR_FF             
0x0000577A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000577C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000577E      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00005780      1  FF                                                   TERMINATOR_FF             
0x00005781      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005783      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005785      3  F31A91                                               IMM16_F3                  u16_be=6801, u16_le=37146
0x00005788      1  FF                                                   TERMINATOR_FF             
0x00005789      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000578B      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000578D      9  8007495A3030383530                                   LEN8_STRING_CP932         length=7, text="IZ00850"
0x00005796      1  FF                                                   TERMINATOR_FF             
0x00005797      1  FF                                                   TERMINATOR_FF             
0x00005798      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000579A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000579C     29  801B817582A682A5815B82C13F2120837A8393836782C981...  LEN8_STRING_CP932         length=27, text="「えぇーっ?! ホントにー?!」"
0x000057B9      1  FF                                                   TERMINATOR_FF             
0x000057BA      1  FF                                                   TERMINATOR_FF             
0x000057BB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000057BD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000057BF      1  FF                                                   TERMINATOR_FF             
0x000057C0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000057C2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000057C4      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000057C6      1  FF                                                   TERMINATOR_FF             
0x000057C7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000057C9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000057CB      3  F31A92                                               IMM16_F3                  u16_be=6802, u16_le=37402
0x000057CE      1  FF                                                   TERMINATOR_FF             
0x000057CF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000057D1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000057D3     42  8028817582A282E2814182BB82F182C88B4382AA82B582BD...  LEN8_STRING_CP932         length=40, text="「いや、そんな気がしただけなんですけど」"
0x000057FD      1  FF                                                   TERMINATOR_FF             
0x000057FE      1  FF                                                   TERMINATOR_FF             
0x000057FF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005801      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005803      1  FF                                                   TERMINATOR_FF             
0x00005804      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005806      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005808      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000580A      1  FF                                                   TERMINATOR_FF             
0x0000580B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000580D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000580F      3  F31A93                                               IMM16_F3                  u16_be=6803, u16_le=37658
0x00005812      1  FF                                                   TERMINATOR_FF             
0x00005813      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005815      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005817      9  8007495A3030383630                                   LEN8_STRING_CP932         length=7, text="IZ00860"
0x00005820      1  FF                                                   TERMINATOR_FF             
0x00005821      1  FF                                                   TERMINATOR_FF             
0x00005822      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005824      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005826     78  804C817582A482A482F1814182BB82EA814182BA82C182BD...  LEN8_STRING_CP932         length=76, text="「ううん、それ、ぜったいホント。だって、私もそーかなーって思ったんだもーん」"
0x00005874      1  FF                                                   TERMINATOR_FF             
0x00005875      1  FF                                                   TERMINATOR_FF             
0x00005876      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005878      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000587A      1  FF                                                   TERMINATOR_FF             
0x0000587B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000587D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000587F      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00005881      1  FF                                                   TERMINATOR_FF             
0x00005882      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005884      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005886      3  F31A94                                               IMM16_F3                  u16_be=6804, u16_le=37914
0x00005889      1  FF                                                   TERMINATOR_FF             
0x0000588A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000588C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000588E     44  802A816381638E7682C182BD82F182C882E7814182C882F1...  LEN8_STRING_CP932         length=42, text="……思ったんなら、なんとかすればいいのに。"
0x000058BA      1  FF                                                   TERMINATOR_FF             
0x000058BB      1  FF                                                   TERMINATOR_FF             
0x000058BC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000058BE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000058C0      1  FF                                                   TERMINATOR_FF             
0x000058C1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000058C3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000058C5      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000058C7      1  FF                                                   TERMINATOR_FF             
0x000058C8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000058CA      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000058CC      3  F31A95                                               IMM16_F3                  u16_be=6805, u16_le=38170
0x000058CF      1  FF                                                   TERMINATOR_FF             
0x000058D0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000058D2      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000058D4      9  8007495A3030383731                                   LEN8_STRING_CP932         length=7, text="IZ00871"
0x000058DD      1  FF                                                   TERMINATOR_FF             
0x000058DE      1  FF                                                   TERMINATOR_FF             
0x000058DF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000058E1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000058E3    132  8082817582A0829F8163816382E0815B835683878362834E...  LEN8_STRING_CP932         length=130, text="「あぁ……もーショックー。後ろピターッて\nくっついてるから、ちょっとヘンーとか思った\nんだけど、混んでるから仕方ないのかなーって」"
0x00005967      1  FF                                                   TERMINATOR_FF             
0x00005968      1  FF                                                   TERMINATOR_FF             
0x00005969      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000596B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000596D      1  FF                                                   TERMINATOR_FF             
0x0000596E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005970      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005972      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00005974      1  FF                                                   TERMINATOR_FF             
0x00005975      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005977      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005979      3  F31A96                                               IMM16_F3                  u16_be=6806, u16_le=38426
0x0000597C      1  FF                                                   TERMINATOR_FF             
0x0000597D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000597F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005981      9  8007495A3030383732                                   LEN8_STRING_CP932         length=7, text="IZ00872"
0x0000598A      1  FF                                                   TERMINATOR_FF             
0x0000598B      1  FF                                                   TERMINATOR_FF             
0x0000598C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000598E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005990    100  8062817582C582E0814182BB82F182C882C9969E88F582C1...  LEN8_STRING_CP932         length=98, text="「でも、そんなに満員ってわけじゃなかったし、ヘンなんだけどなーって……あー、くやしくって死にそー」"
0x000059F4      1  FF                                                   TERMINATOR_FF             
0x000059F5      1  FF                                                   TERMINATOR_FF             
0x000059F6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000059F8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000059FA      1  FF                                                   TERMINATOR_FF             
0x000059FB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000059FD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000059FF      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00005A01      1  FF                                                   TERMINATOR_FF             
0x00005A02      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005A04      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005A06      3  F30215                                               IMM16_F3                  u16_be=533, u16_le=5378
0x00005A09      1  FF                                                   TERMINATOR_FF             
0x00005A0A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005A0C      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005A0E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005A10      1  FF                                                   TERMINATOR_FF             
0x00005A11      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005A13      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005A15      1  FF                                                   TERMINATOR_FF             
0x00005A16      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005A18      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005A1A      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00005A1C      1  FF                                                   TERMINATOR_FF             
0x00005A1D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005A1F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005A21      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00005A23      1  FF                                                   TERMINATOR_FF             
0x00005A24      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005A26      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005A28      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00005A2A      1  FF                                                   TERMINATOR_FF             
0x00005A2B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005A2D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005A2F      1  FF                                                   TERMINATOR_FF             
0x00005A30      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005A32      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005A34      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00005A36      1  FF                                                   TERMINATOR_FF             
0x00005A37      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005A39      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005A3B      3  F31A97                                               IMM16_F3                  u16_be=6807, u16_le=38682
0x00005A3E      1  FF                                                   TERMINATOR_FF             
0x00005A3F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005A41      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005A43      9  8007495A3030383830                                   LEN8_STRING_CP932         length=7, text="IZ00880"
0x00005A4C      1  FF                                                   TERMINATOR_FF             
0x00005A4D      1  FF                                                   TERMINATOR_FF             
0x00005A4E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005A50      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005A52     42  8028817582A082C18163816382B682E182A0814182E282C1...  LEN8_STRING_CP932         length=40, text="「あっ……じゃあ、やっぱりあれって……」"
0x00005A7C      1  FF                                                   TERMINATOR_FF             
0x00005A7D      1  FF                                                   TERMINATOR_FF             
0x00005A7E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005A80      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005A82      1  FF                                                   TERMINATOR_FF             
0x00005A83      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005A85      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005A87      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00005A89      1  FF                                                   TERMINATOR_FF             
0x00005A8A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005A8C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005A8E      3  F31A98                                               IMM16_F3                  u16_be=6808, u16_le=38938
0x00005A91      1  FF                                                   TERMINATOR_FF             
0x00005A92      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005A94      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005A96     30  801C82A88E6F82B382F182CD955C8FEE82F082AD82E082E7...  LEN8_STRING_CP932         length=28, text="お姉さんは表情をくもらせる。"
0x00005AB4      1  FF                                                   TERMINATOR_FF             
0x00005AB5      1  FF                                                   TERMINATOR_FF             
0x00005AB6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005AB8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005ABA      1  FF                                                   TERMINATOR_FF             
0x00005ABB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005ABD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005ABF      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00005AC1      1  FF                                                   TERMINATOR_FF             
0x00005AC2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005AC4      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005AC6      3  F31A99                                               IMM16_F3                  u16_be=6809, u16_le=39194
0x00005AC9      1  FF                                                   TERMINATOR_FF             
0x00005ACA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005ACC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005ACE     38  8024817582BE82A982E7814182B382C182AB82E093AF82B6...  LEN8_STRING_CP932         length=36, text="「だから、さっきも同じに思えたんで」"
0x00005AF4      1  FF                                                   TERMINATOR_FF             
0x00005AF5      1  FF                                                   TERMINATOR_FF             
0x00005AF6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005AF8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005AFA      1  FF                                                   TERMINATOR_FF             
0x00005AFB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005AFD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005AFF      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00005B01      1  FF                                                   TERMINATOR_FF             
0x00005B02      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005B04      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005B06      3  F300CB                                               IMM16_F3                  u16_be=203, u16_le=51968
0x00005B09      1  FF                                                   TERMINATOR_FF             
0x00005B0A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005B0C      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005B0E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005B10      1  FF                                                   TERMINATOR_FF             
0x00005B11      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005B13      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005B15      1  FF                                                   TERMINATOR_FF             
0x00005B16      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005B18      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005B1A      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00005B1C      1  FF                                                   TERMINATOR_FF             
0x00005B1D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005B1F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005B21      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00005B23      1  FF                                                   TERMINATOR_FF             
0x00005B24      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005B26      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005B28      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00005B2A      1  FF                                                   TERMINATOR_FF             
0x00005B2B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005B2D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005B2F      1  FF                                                   TERMINATOR_FF             
0x00005B30      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005B32      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005B34      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00005B36      1  FF                                                   TERMINATOR_FF             
0x00005B37      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005B39      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005B3B      3  F31A9A                                               IMM16_F3                  u16_be=6810, u16_le=39450
0x00005B3E      1  FF                                                   TERMINATOR_FF             
0x00005B3F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005B41      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005B43      9  8007495A3030383930                                   LEN8_STRING_CP932         length=7, text="IZ00890"
0x00005B4C      1  FF                                                   TERMINATOR_FF             
0x00005B4D      1  FF                                                   TERMINATOR_FF             
0x00005B4E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005B50      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005B52     38  8024817582A08163816382BE82A982E781638F9582AF82C4...  LEN8_STRING_CP932         length=36, text="「あ……だから…助けてくれたんだー」"
0x00005B78      1  FF                                                   TERMINATOR_FF             
0x00005B79      1  FF                                                   TERMINATOR_FF             
0x00005B7A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005B7C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005B7E      1  FF                                                   TERMINATOR_FF             
0x00005B7F      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00005B81      2  005B                                                 WORD_00XX                 u16_be=91, low_byte=91
0x00005B83      1  90                                                   OPAQUE_RAW_BYTES          bytes=90
0x00005B84      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00005B86      3  F10035                                               IMM16_F1                  u16_be=53, u16_le=13568
0x00005B89      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005B8B      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00005B8C      1  FF                                                   TERMINATOR_FF             
0x00005B8D      2  005B                                                 WORD_00XX                 u16_be=91, low_byte=91
0x00005B8F      1  90                                                   OPAQUE_RAW_BYTES          bytes=90
0x00005B90      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00005B92      2  0062                                                 WORD_00XX                 u16_be=98, low_byte=98
0x00005B94      1  65                                                   OPAQUE_RAW_BYTES          bytes=65
0x00005B95      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00005B97      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x00005B9A      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00005B9C      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00005B9D      1  FF                                                   TERMINATOR_FF             
0x00005B9E      2  0062                                                 WORD_00XX                 u16_be=98, low_byte=98
0x00005BA0      1  65                                                   OPAQUE_RAW_BYTES          bytes=65
0x00005BA1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005BA3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005BA5      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00005BA7      1  FF                                                   TERMINATOR_FF             
0x00005BA8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005BAA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005BAC      3  F300CA                                               IMM16_F3                  u16_be=202, u16_le=51712
0x00005BAF      1  FF                                                   TERMINATOR_FF             
0x00005BB0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005BB2      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005BB4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005BB6      1  FF                                                   TERMINATOR_FF             
0x00005BB7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005BB9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005BBB      1  FF                                                   TERMINATOR_FF             
0x00005BBC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005BBE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005BC0      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00005BC2      1  FF                                                   TERMINATOR_FF             
0x00005BC3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005BC5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005BC7      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00005BC9      1  FF                                                   TERMINATOR_FF             
0x00005BCA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005BCC      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005BCE      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00005BD0      1  FF                                                   TERMINATOR_FF             
0x00005BD1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005BD3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005BD5      1  FF                                                   TERMINATOR_FF             
0x00005BD6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005BD8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005BDA      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00005BDC      1  FF                                                   TERMINATOR_FF             
0x00005BDD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005BDF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005BE1      3  F31A9B                                               IMM16_F3                  u16_be=6811, u16_le=39706
0x00005BE4      1  FF                                                   TERMINATOR_FF             
0x00005BE5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005BE7      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005BE9      9  8007495A3030393030                                   LEN8_STRING_CP932         length=7, text="IZ00900"
0x00005BF2      1  FF                                                   TERMINATOR_FF             
0x00005BF3      1  FF                                                   TERMINATOR_FF             
0x00005BF4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005BF6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005BF8     48  802E817582A6816082C1814182BB82C182A982C8815B8142...  LEN8_STRING_CP932         length=46, text="「え～っ、そっかなー。そんな気ーするけどなー」"
0x00005C28      1  FF                                                   TERMINATOR_FF             
0x00005C29      1  FF                                                   TERMINATOR_FF             
0x00005C2A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005C2C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005C2E      1  FF                                                   TERMINATOR_FF             
0x00005C2F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005C31      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005C33      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00005C35      1  FF                                                   TERMINATOR_FF             
0x00005C36      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005C38      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005C3A      3  F31A9C                                               IMM16_F3                  u16_be=6812, u16_le=39962
0x00005C3D      1  FF                                                   TERMINATOR_FF             
0x00005C3E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005C40      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005C42     46  802C82A88E6F82B382F182CD82BF82E582C182C68D6C82A6...  LEN8_STRING_CP932         length=44, text="お姉さんはちょっと考えこみ、にやっと笑った。"
0x00005C70      1  FF                                                   TERMINATOR_FF             
0x00005C71      1  FF                                                   TERMINATOR_FF             
0x00005C72      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005C74      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005C76      1  FF                                                   TERMINATOR_FF             
0x00005C77      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005C79      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005C7B      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00005C7D      1  FF                                                   TERMINATOR_FF             
0x00005C7E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005C80      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005C82      3  F300C9                                               IMM16_F3                  u16_be=201, u16_le=51456
0x00005C85      1  FF                                                   TERMINATOR_FF             
0x00005C86      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005C88      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005C8A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005C8C      1  FF                                                   TERMINATOR_FF             
0x00005C8D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005C8F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005C91      1  FF                                                   TERMINATOR_FF             
0x00005C92      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005C94      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005C96      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00005C98      1  FF                                                   TERMINATOR_FF             
0x00005C99      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005C9B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005C9D      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00005C9F      1  FF                                                   TERMINATOR_FF             
0x00005CA0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005CA2      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005CA4      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00005CA6      1  FF                                                   TERMINATOR_FF             
0x00005CA7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005CA9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005CAB      1  FF                                                   TERMINATOR_FF             
0x00005CAC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005CAE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005CB0      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00005CB2      1  FF                                                   TERMINATOR_FF             
0x00005CB3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005CB5      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005CB7      3  F31A9D                                               IMM16_F3                  u16_be=6813, u16_le=40218
0x00005CBA      1  FF                                                   TERMINATOR_FF             
0x00005CBB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005CBD      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005CBF     10  8008495A303039313052                                 LEN8_STRING_CP932         length=8, text="IZ00910R"
0x00005CC9      1  FF                                                   TERMINATOR_FF             
0x00005CCA      1  FF                                                   TERMINATOR_FF             
0x00005CCB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005CCD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005CCF     62  803C817582B682E1815B82A0815B814182AB82DD82CD815B...  LEN8_STRING_CP932         length=60, text="「じゃーあー、きみはー、どーして声かけて\nくれたのかなー？」"
0x00005D0D      1  FF                                                   TERMINATOR_FF             
0x00005D0E      1  FF                                                   TERMINATOR_FF             
0x00005D0F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005D11      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005D13      1  FF                                                   TERMINATOR_FF             
0x00005D14      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005D16      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005D18      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00005D1A      1  FF                                                   TERMINATOR_FF             
0x00005D1B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005D1D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005D1F      3  F300C1                                               IMM16_F3                  u16_be=193, u16_le=49408
0x00005D22      1  FF                                                   TERMINATOR_FF             
0x00005D23      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005D25      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005D27      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005D29      1  FF                                                   TERMINATOR_FF             
0x00005D2A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005D2C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005D2E      1  FF                                                   TERMINATOR_FF             
0x00005D2F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005D31      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005D33      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00005D35      1  FF                                                   TERMINATOR_FF             
0x00005D36      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005D38      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005D3A      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00005D3C      1  FF                                                   TERMINATOR_FF             
0x00005D3D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005D3F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005D41      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00005D43      1  FF                                                   TERMINATOR_FF             
0x00005D44      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005D46      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005D48      1  FF                                                   TERMINATOR_FF             
0x00005D49      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005D4B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005D4D      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00005D4F      1  FF                                                   TERMINATOR_FF             
0x00005D50      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005D52      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005D54      3  F31A9E                                               IMM16_F3                  u16_be=6814, u16_le=40474
0x00005D57      1  FF                                                   TERMINATOR_FF             
0x00005D58      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005D5A      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005D5C     10  8008495A303039323052                                 LEN8_STRING_CP932         length=8, text="IZ00920R"
0x00005D66      1  FF                                                   TERMINATOR_FF             
0x00005D67      1  FF                                                   TERMINATOR_FF             
0x00005D68      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005D6A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005D6C     74  8048817582A88E6F82B382F182AA82B182DC82C182C482E9...  LEN8_STRING_CP932         length=72, text="「お姉さんがこまってるーとか思ってー、助けてくれたんじゃないのかなー？」"
0x00005DB6      1  FF                                                   TERMINATOR_FF             
0x00005DB7      1  FF                                                   TERMINATOR_FF             
0x00005DB8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005DBA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005DBC      1  FF                                                   TERMINATOR_FF             
0x00005DBD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005DBF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005DC1      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00005DC3      1  FF                                                   TERMINATOR_FF             
0x00005DC4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005DC6      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005DC8      3  F31A9F                                               IMM16_F3                  u16_be=6815, u16_le=40730
0x00005DCB      1  FF                                                   TERMINATOR_FF             
0x00005DCC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005DCE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005DD0     40  80268163816382B782B282AD82B182C782E088B582A282B3...  LEN8_STRING_CP932         length=38, text="……すごくこども扱いされてる気がする。"
0x00005DF8      1  FF                                                   TERMINATOR_FF             
0x00005DF9      1  FF                                                   TERMINATOR_FF             
0x00005DFA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005DFC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005DFE      1  FF                                                   TERMINATOR_FF             
0x00005DFF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005E01      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005E03      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00005E05      1  FF                                                   TERMINATOR_FF             
0x00005E06      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005E08      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005E0A      3  F300C8                                               IMM16_F3                  u16_be=200, u16_le=51200
0x00005E0D      1  FF                                                   TERMINATOR_FF             
0x00005E0E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005E10      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005E12      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005E14      1  FF                                                   TERMINATOR_FF             
0x00005E15      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005E17      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005E19      1  FF                                                   TERMINATOR_FF             
0x00005E1A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005E1C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005E1E      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00005E20      1  FF                                                   TERMINATOR_FF             
0x00005E21      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005E23      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005E25      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00005E27      1  FF                                                   TERMINATOR_FF             
0x00005E28      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005E2A      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005E2C      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00005E2E      1  FF                                                   TERMINATOR_FF             
0x00005E2F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005E31      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005E33      1  FF                                                   TERMINATOR_FF             
0x00005E34      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005E36      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005E38      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00005E3A      1  FF                                                   TERMINATOR_FF             
0x00005E3B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005E3D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005E3F      3  F31AA0                                               IMM16_F3                  u16_be=6816, u16_le=40986
0x00005E42      1  FF                                                   TERMINATOR_FF             
0x00005E43      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005E45      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005E47     10  8008495A303039333052                                 LEN8_STRING_CP932         length=8, text="IZ00930R"
0x00005E51      1  FF                                                   TERMINATOR_FF             
0x00005E52      1  FF                                                   TERMINATOR_FF             
0x00005E53      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005E55      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005E57     92  805A817582A682C182D6816082F1814183568387815B8357...  LEN8_STRING_CP932         length=90, text="「えっへ～ん、ショージキにいってごらん\nなさーい。ぜーんぜん、はずかしくないん\nだからー」"
0x00005EB3      1  FF                                                   TERMINATOR_FF             
0x00005EB4      1  FF                                                   TERMINATOR_FF             
0x00005EB5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005EB7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005EB9      1  FF                                                   TERMINATOR_FF             
0x00005EBA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005EBC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005EBE      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00005EC0      1  FF                                                   TERMINATOR_FF             
0x00005EC1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005EC3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005EC5      3  F300C2                                               IMM16_F3                  u16_be=194, u16_le=49664
0x00005EC8      1  FF                                                   TERMINATOR_FF             
0x00005EC9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005ECB      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005ECD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005ECF      1  FF                                                   TERMINATOR_FF             
0x00005ED0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005ED2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005ED4      1  FF                                                   TERMINATOR_FF             
0x00005ED5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005ED7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005ED9      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00005EDB      1  FF                                                   TERMINATOR_FF             
0x00005EDC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005EDE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005EE0      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00005EE2      1  FF                                                   TERMINATOR_FF             
0x00005EE3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005EE5      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005EE7      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00005EE9      1  FF                                                   TERMINATOR_FF             
0x00005EEA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005EEC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005EEE      1  FF                                                   TERMINATOR_FF             
0x00005EEF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005EF1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005EF3      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00005EF5      1  FF                                                   TERMINATOR_FF             
0x00005EF6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005EF8      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005EFA      3  F31AA1                                               IMM16_F3                  u16_be=6817, u16_le=41242
0x00005EFD      1  FF                                                   TERMINATOR_FF             
0x00005EFE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005F00      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00005F02     10  8008495A303039343052                                 LEN8_STRING_CP932         length=8, text="IZ00940R"
0x00005F0C      1  FF                                                   TERMINATOR_FF             
0x00005F0D      1  FF                                                   TERMINATOR_FF             
0x00005F0E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005F10      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005F12    108  806A817582CB82A682CB82A681418F9582AF82C482AD82EA...  LEN8_STRING_CP932         length=106, text="「ねえねえ、助けてくれたんだよねー？\nわかってるんだからー。照れないでー、\nホントのこといっちゃいなよー」"
0x00005F7E      1  FF                                                   TERMINATOR_FF             
0x00005F7F      1  FF                                                   TERMINATOR_FF             
0x00005F80      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005F82      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005F84      1  FF                                                   TERMINATOR_FF             
0x00005F85      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005F87      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005F89      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00005F8B      1  FF                                                   TERMINATOR_FF             
0x00005F8C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005F8E      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00005F90      3  F31AA2                                               IMM16_F3                  u16_be=6818, u16_le=41498
0x00005F93      1  FF                                                   TERMINATOR_FF             
0x00005F94      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005F96      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00005F98     86  805482C882F182BE82A9814182B382C182AB82DC82C582C6...  LEN8_STRING_CP932         length=84, text="なんだか、さっきまでとテンションが違うなあ。年下だと思って安心されてるんだろうけど。"
0x00005FEE      1  FF                                                   TERMINATOR_FF             
0x00005FEF      1  FF                                                   TERMINATOR_FF             
0x00005FF0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00005FF2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00005FF4      1  FF                                                   TERMINATOR_FF             
0x00005FF5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005FF7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005FF9      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00005FFB      1  FF                                                   TERMINATOR_FF             
0x00005FFC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00005FFE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006000      3  F300C8                                               IMM16_F3                  u16_be=200, u16_le=51200
0x00006003      1  FF                                                   TERMINATOR_FF             
0x00006004      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006006      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00006008      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000600A      1  FF                                                   TERMINATOR_FF             
0x0000600B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000600D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000600F      1  FF                                                   TERMINATOR_FF             
0x00006010      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006012      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006014      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00006016      1  FF                                                   TERMINATOR_FF             
0x00006017      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006019      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000601B      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000601D      1  FF                                                   TERMINATOR_FF             
0x0000601E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006020      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00006022      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00006024      1  FF                                                   TERMINATOR_FF             
0x00006025      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006027      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006029      1  FF                                                   TERMINATOR_FF             
0x0000602A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000602C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000602E      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00006030      1  FF                                                   TERMINATOR_FF             
0x00006031      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006033      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006035      3  F31AA3                                               IMM16_F3                  u16_be=6819, u16_le=41754
0x00006038      1  FF                                                   TERMINATOR_FF             
0x00006039      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000603B      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000603D     10  8008495A303039353052                                 LEN8_STRING_CP932         length=8, text="IZ00950R"
0x00006047      1  FF                                                   TERMINATOR_FF             
0x00006048      1  FF                                                   TERMINATOR_FF             
0x00006049      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000604B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000604D    104  8066817582A282E2815B82CB815B81418FC682EA82BF82E1...  LEN8_STRING_CP932         length=102, text="「いやーねー、照れちゃってるんだからー。\n女の子に親切にしたからって、はずかしく\nないんですからねー」"
0x000060B5      1  FF                                                   TERMINATOR_FF             
0x000060B6      1  FF                                                   TERMINATOR_FF             
0x000060B7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000060B9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000060BB      1  FF                                                   TERMINATOR_FF             
0x000060BC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000060BE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000060C0      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000060C2      1  FF                                                   TERMINATOR_FF             
0x000060C3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000060C5      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000060C7      3  F31AA4                                               IMM16_F3                  u16_be=6820, u16_le=42010
0x000060CA      1  FF                                                   TERMINATOR_FF             
0x000060CB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000060CD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000060CF     48  802E82A482ED815B81418FAC8A7790B682DD82BD82A282C8...  LEN8_STRING_CP932         length=46, text="うわー、小学生みたいなこといわれちゃったよー。"
0x000060FF      1  FF                                                   TERMINATOR_FF             
0x00006100      1  FF                                                   TERMINATOR_FF             
0x00006101      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006103      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006105      1  FF                                                   TERMINATOR_FF             
0x00006106      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006108      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000610A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000610C      1  FF                                                   TERMINATOR_FF             
0x0000610D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000610F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006111      3  F31AA5                                               IMM16_F3                  u16_be=6821, u16_le=42266
0x00006114      1  FF                                                   TERMINATOR_FF             
0x00006115      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006117      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006119     30  801C817582DC82A08163816382BB82A482C882F182C582B7...  LEN8_STRING_CP932         length=28, text="「まあ……そうなんですけど」"
0x00006137      1  FF                                                   TERMINATOR_FF             
0x00006138      1  FF                                                   TERMINATOR_FF             
0x00006139      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000613B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000613D      1  FF                                                   TERMINATOR_FF             
0x0000613E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006140      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006142      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00006144      1  FF                                                   TERMINATOR_FF             
0x00006145      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006147      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006149      3  F31AA6                                               IMM16_F3                  u16_be=6822, u16_le=42522
0x0000614C      1  FF                                                   TERMINATOR_FF             
0x0000614D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000614F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00006151     10  8008495A303039363052                                 LEN8_STRING_CP932         length=8, text="IZ00960R"
0x0000615B      1  FF                                                   TERMINATOR_FF             
0x0000615C      1  FF                                                   TERMINATOR_FF             
0x0000615D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000615F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006161     26  8018817582C582B582E582C582B582E582C582B582E5815B...  LEN8_STRING_CP932         length=24, text="「でしょでしょでしょー」"
0x0000617B      1  FF                                                   TERMINATOR_FF             
0x0000617C      1  FF                                                   TERMINATOR_FF             
0x0000617D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000617F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006181      1  FF                                                   TERMINATOR_FF             
0x00006182      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006184      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006186      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00006188      1  FF                                                   TERMINATOR_FF             
0x00006189      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000618B      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000618D      3  F31AA7                                               IMM16_F3                  u16_be=6823, u16_le=42778
0x00006190      1  FF                                                   TERMINATOR_FF             
0x00006191      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006193      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00006195     10  8008495A303039373152                                 LEN8_STRING_CP932         length=8, text="IZ00971R"
0x0000619F      1  FF                                                   TERMINATOR_FF             
0x000061A0      1  FF                                                   TERMINATOR_FF             
0x000061A1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000061A3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000061A5     99  8061817582DE82D382D38160814182C8815B82C9815B8148...  LEN8_STRING_CP932         length=97, text="「むふふ～、なーにー？ きみはー、お姉さんが\n困ってるからー、助けなきゃーって思ったん\nだよねー」"
0x00006208      1  FF                                                   TERMINATOR_FF             
0x00006209      1  FF                                                   TERMINATOR_FF             
0x0000620A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000620C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000620E      1  FF                                                   TERMINATOR_FF             
0x0000620F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006211      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006213      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00006215      1  FF                                                   TERMINATOR_FF             
0x00006216      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006218      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000621A      3  F31AA8                                               IMM16_F3                  u16_be=6824, u16_le=43034
0x0000621D      1  FF                                                   TERMINATOR_FF             
0x0000621E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006220      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00006222     10  8008495A303039373252                                 LEN8_STRING_CP932         length=8, text="IZ00972R"
0x0000622C      1  FF                                                   TERMINATOR_FF             
0x0000622D      1  FF                                                   TERMINATOR_FF             
0x0000622E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006230      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006232     44  802A817582B782B2816082A2814182A682E7816082A28142...  LEN8_STRING_CP932         length=42, text="「すご～い、えら～い。や～さしーんだね～」"
0x0000625E      1  FF                                                   TERMINATOR_FF             
0x0000625F      1  FF                                                   TERMINATOR_FF             
0x00006260      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006262      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006264      1  FF                                                   TERMINATOR_FF             
0x00006265      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006267      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006269      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000626B      1  FF                                                   TERMINATOR_FF             
0x0000626C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000626E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006270      3  F300CA                                               IMM16_F3                  u16_be=202, u16_le=51712
0x00006273      1  FF                                                   TERMINATOR_FF             
0x00006274      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006276      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00006278      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000627A      1  FF                                                   TERMINATOR_FF             
0x0000627B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000627D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000627F      1  FF                                                   TERMINATOR_FF             
0x00006280      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006282      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006284      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00006286      1  FF                                                   TERMINATOR_FF             
0x00006287      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006289      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000628B      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000628D      1  FF                                                   TERMINATOR_FF             
0x0000628E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006290      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00006292      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00006294      1  FF                                                   TERMINATOR_FF             
0x00006295      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006297      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006299      1  FF                                                   TERMINATOR_FF             
0x0000629A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000629C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000629E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000062A0      1  FF                                                   TERMINATOR_FF             
0x000062A1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000062A3      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000062A5      3  F31AA9                                               IMM16_F3                  u16_be=6825, u16_le=43290
0x000062A8      1  FF                                                   TERMINATOR_FF             
0x000062A9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000062AB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000062AD     44  802A82A88E6F82B382F182CD81418FAC8EF182F082A982B5...  LEN8_STRING_CP932         length=42, text="お姉さんは、小首をかしげ、ぼくを見つめる。"
0x000062D9      1  FF                                                   TERMINATOR_FF             
0x000062DA      1  FF                                                   TERMINATOR_FF             
0x000062DB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000062DD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000062DF      1  FF                                                   TERMINATOR_FF             
0x000062E0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000062E2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000062E4      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000062E6      1  FF                                                   TERMINATOR_FF             
0x000062E7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000062E9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000062EB      3  F31AAA                                               IMM16_F3                  u16_be=6826, u16_le=43546
0x000062EE      1  FF                                                   TERMINATOR_FF             
0x000062EF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000062F1      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000062F3     10  8008495A303039383052                                 LEN8_STRING_CP932         length=8, text="IZ00980R"
0x000062FD      1  FF                                                   TERMINATOR_FF             
0x000062FE      1  FF                                                   TERMINATOR_FF             
0x000062FF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006301      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006303     44  802A817582B682E182A08163816382A897E7816382B582C8...  LEN8_STRING_CP932         length=42, text="「じゃあ……お礼…しなきゃ、ダメだよねー」"
0x0000632F      1  FF                                                   TERMINATOR_FF             
0x00006330      1  FF                                                   TERMINATOR_FF             
0x00006331      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006333      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006335      1  FF                                                   TERMINATOR_FF             
0x00006336      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006338      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000633A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000633C      1  FF                                                   TERMINATOR_FF             
0x0000633D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000633F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006341      3  F31AAB                                               IMM16_F3                  u16_be=6827, u16_le=43802
0x00006344      1  FF                                                   TERMINATOR_FF             
0x00006345      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006347      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006349     48  802E817582A282E2814195CA82C98163816382BB82F182C8...  LEN8_STRING_CP932         length=46, text="「いや、別に……そんなつもりじゃなかったから」"
0x00006379      1  FF                                                   TERMINATOR_FF             
0x0000637A      1  FF                                                   TERMINATOR_FF             
0x0000637B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000637D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000637F      1  FF                                                   TERMINATOR_FF             
0x00006380      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006382      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006384      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00006386      1  FF                                                   TERMINATOR_FF             
0x00006387      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006389      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000638B      3  F3020B                                               IMM16_F3                  u16_be=523, u16_le=2818
0x0000638E      1  FF                                                   TERMINATOR_FF             
0x0000638F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006391      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00006393      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006395      1  FF                                                   TERMINATOR_FF             
0x00006396      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006398      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000639A      1  FF                                                   TERMINATOR_FF             
0x0000639B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000639D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000639F      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000063A1      1  FF                                                   TERMINATOR_FF             
0x000063A2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000063A4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000063A6      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000063A8      1  FF                                                   TERMINATOR_FF             
0x000063A9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000063AB      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000063AD      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000063AF      1  FF                                                   TERMINATOR_FF             
0x000063B0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000063B2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000063B4      1  FF                                                   TERMINATOR_FF             
0x000063B5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000063B7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000063B9      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000063BB      1  FF                                                   TERMINATOR_FF             
0x000063BC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000063BE      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000063C0      3  F31AAC                                               IMM16_F3                  u16_be=6828, u16_le=44058
0x000063C3      1  FF                                                   TERMINATOR_FF             
0x000063C4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000063C6      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000063C8     10  8008495A303039393052                                 LEN8_STRING_CP932         length=8, text="IZ00990R"
0x000063D2      1  FF                                                   TERMINATOR_FF             
0x000063D3      1  FF                                                   TERMINATOR_FF             
0x000063D4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000063D6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000063D8     82  8050817582A682A58160814182BB815B82E4815B82ED82AF...  LEN8_STRING_CP932         length=80, text="「えぇ～、そーゆーわけにはいかないよー。\nだあ～って、助けてもらったんだもーん」"
0x0000642A      1  FF                                                   TERMINATOR_FF             
0x0000642B      1  FF                                                   TERMINATOR_FF             
0x0000642C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000642E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006430      1  FF                                                   TERMINATOR_FF             
0x00006431      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006433      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006435      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00006437      1  FF                                                   TERMINATOR_FF             
0x00006438      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000643A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000643C      3  F300BE                                               IMM16_F3                  u16_be=190, u16_le=48640
0x0000643F      1  FF                                                   TERMINATOR_FF             
0x00006440      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006442      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00006444      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006446      1  FF                                                   TERMINATOR_FF             
0x00006447      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006449      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000644B      1  FF                                                   TERMINATOR_FF             
0x0000644C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000644E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006450      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00006452      1  FF                                                   TERMINATOR_FF             
0x00006453      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006455      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006457      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00006459      1  FF                                                   TERMINATOR_FF             
0x0000645A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000645C      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000645E      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00006460      1  FF                                                   TERMINATOR_FF             
0x00006461      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006463      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006465      1  FF                                                   TERMINATOR_FF             
0x00006466      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006468      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000646A      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000646C      1  FF                                                   TERMINATOR_FF             
0x0000646D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000646F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006471      3  F31AAD                                               IMM16_F3                  u16_be=6829, u16_le=44314
0x00006474      1  FF                                                   TERMINATOR_FF             
0x00006475      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006477      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00006479     10  8008495A303130303052                                 LEN8_STRING_CP932         length=8, text="IZ01000R"
0x00006483      1  FF                                                   TERMINATOR_FF             
0x00006484      1  FF                                                   TERMINATOR_FF             
0x00006485      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006487      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006489     57  8037817582BB82C182BE82C121212082CB82A6814182A882...  LEN8_STRING_CP932         length=55, text="「そっだっ!! ねえ、おなかすいてる？\nすいてるでしょ?!」"
0x000064C2      1  FF                                                   TERMINATOR_FF             
0x000064C3      1  FF                                                   TERMINATOR_FF             
0x000064C4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000064C6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000064C8      1  FF                                                   TERMINATOR_FF             
0x000064C9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000064CB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000064CD      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000064CF      1  FF                                                   TERMINATOR_FF             
0x000064D0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000064D2      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000064D4      3  F31AAE                                               IMM16_F3                  u16_be=6830, u16_le=44570
0x000064D7      1  FF                                                   TERMINATOR_FF             
0x000064D8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000064DA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000064DC     26  8018817582A682C18163816382BB82EA82D982C782C582E0...  LEN8_STRING_CP932         length=24, text="「えっ……それほどでも」"
0x000064F6      1  FF                                                   TERMINATOR_FF             
0x000064F7      1  FF                                                   TERMINATOR_FF             
0x000064F8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000064FA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000064FC      1  FF                                                   TERMINATOR_FF             
0x000064FD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000064FF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006501      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00006503      1  FF                                                   TERMINATOR_FF             
0x00006504      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006506      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006508      3  F31AAF                                               IMM16_F3                  u16_be=6831, u16_le=44826
0x0000650B      1  FF                                                   TERMINATOR_FF             
0x0000650C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000650E      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00006510     10  8008495A303130313152                                 LEN8_STRING_CP932         length=8, text="IZ01011R"
0x0000651A      1  FF                                                   TERMINATOR_FF             
0x0000651B      1  FF                                                   TERMINATOR_FF             
0x0000651C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000651E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006520     40  8026817582E282BE82C882A0814183478393838A838782B5...  LEN8_STRING_CP932         length=38, text="「やだなあ、エンリョしてるんでしょー」"
0x00006548      1  FF                                                   TERMINATOR_FF             
0x00006549      1  FF                                                   TERMINATOR_FF             
0x0000654A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000654C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000654E      1  FF                                                   TERMINATOR_FF             
0x0000654F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006551      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006553      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00006555      1  FF                                                   TERMINATOR_FF             
0x00006556      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006558      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000655A      3  F31AB0                                               IMM16_F3                  u16_be=6832, u16_le=45082
0x0000655D      1  FF                                                   TERMINATOR_FF             
0x0000655E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006560      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00006562     10  8008495A303130313252                                 LEN8_STRING_CP932         length=8, text="IZ01012R"
0x0000656C      1  FF                                                   TERMINATOR_FF             
0x0000656D      1  FF                                                   TERMINATOR_FF             
0x0000656E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006570      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006572    116  8072817582A0814182ED82A982C182BD814282A882B182C3...  LEN8_STRING_CP932         length=114, text="「あ、わかった。おこづかいないんだー。\nだいじょーぶよー。お姉さん、おとななんだし、それくらいおごってあげるよー」"
0x000065E6      1  FF                                                   TERMINATOR_FF             
0x000065E7      1  FF                                                   TERMINATOR_FF             
0x000065E8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000065EA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000065EC      1  FF                                                   TERMINATOR_FF             
0x000065ED      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000065EF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000065F1      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000065F3      1  FF                                                   TERMINATOR_FF             
0x000065F4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000065F6      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000065F8      3  F31AB1                                               IMM16_F3                  u16_be=6833, u16_le=45338
0x000065FB      1  FF                                                   TERMINATOR_FF             
0x000065FC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000065FE      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00006600     10  8008495A303130313352                                 LEN8_STRING_CP932         length=8, text="IZ01013R"
0x0000660A      1  FF                                                   TERMINATOR_FF             
0x0000660B      1  FF                                                   TERMINATOR_FF             
0x0000660C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000660E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006610     50  8030817582BE815B82C182C4815B81418F9582AF82C482AD...  LEN8_STRING_CP932         length=48, text="「だーってー、助けてくれたお礼なんだ\nもーん!!」"
0x00006642      1  FF                                                   TERMINATOR_FF             
0x00006643      1  FF                                                   TERMINATOR_FF             
0x00006644      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006646      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006648      1  FF                                                   TERMINATOR_FF             
0x00006649      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000664B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000664D      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000664F      1  FF                                                   TERMINATOR_FF             
0x00006650      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006652      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006654      3  F3020B                                               IMM16_F3                  u16_be=523, u16_le=2818
0x00006657      1  FF                                                   TERMINATOR_FF             
0x00006658      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000665A      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000665C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000665E      1  FF                                                   TERMINATOR_FF             
0x0000665F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006661      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006663      1  FF                                                   TERMINATOR_FF             
0x00006664      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006666      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006668      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000666A      1  FF                                                   TERMINATOR_FF             
0x0000666B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000666D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000666F      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00006671      1  FF                                                   TERMINATOR_FF             
0x00006672      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006674      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00006676      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00006678      1  FF                                                   TERMINATOR_FF             
0x00006679      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000667B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000667D      1  FF                                                   TERMINATOR_FF             
0x0000667E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006680      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006682      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00006684      1  FF                                                   TERMINATOR_FF             
0x00006685      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006687      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006689      3  F31AB2                                               IMM16_F3                  u16_be=6834, u16_le=45594
0x0000668C      1  FF                                                   TERMINATOR_FF             
0x0000668D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000668F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00006691      9  8007495A3130343830                                   LEN8_STRING_CP932         length=7, text="IZ10480"
0x0000669A      1  FF                                                   TERMINATOR_FF             
0x0000669B      1  FF                                                   TERMINATOR_FF             
0x0000669C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000669E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000066A0     42  8028817582A0814182BB82A482BB82A481418EA98CC88FD0...  LEN8_STRING_CP932         length=40, text="「あ、そうそう、自己紹介しなくっちゃね」"
0x000066CA      1  FF                                                   TERMINATOR_FF             
0x000066CB      1  FF                                                   TERMINATOR_FF             
0x000066CC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000066CE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000066D0      1  FF                                                   TERMINATOR_FF             
0x000066D1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000066D3      2  0051                                                 WORD_00XX                 u16_be=81, low_byte=81
0x000066D5      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000066D7      1  FF                                                   TERMINATOR_FF             
0x000066D8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000066DA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000066DC      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000066DE      1  FF                                                   TERMINATOR_FF             
0x000066DF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000066E1      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000066E3      3  F31AB3                                               IMM16_F3                  u16_be=6835, u16_le=45850
0x000066E6      1  FF                                                   TERMINATOR_FF             
0x000066E7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000066E9      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000066EB      9  8007495A3130343930                                   LEN8_STRING_CP932         length=7, text="IZ10490"
0x000066F4      1  FF                                                   TERMINATOR_FF             
0x000066F5      1  FF                                                   TERMINATOR_FF             
0x000066F6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000066F8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000066FA     55  803581758ADB93A12090F294FC82C582B7814296F08F8A82...  LEN8_STRING_CP932         length=53, text="「丸藤 泉美です。役所で働いてまっす。\nよろしくね～」"
0x00006731      1  FF                                                   TERMINATOR_FF             
0x00006732      1  FF                                                   TERMINATOR_FF             
0x00006733      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00006735      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006737      1  FF                                                   TERMINATOR_FF             
0x00006738      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000673A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000673C      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000673E      1  FF                                                   TERMINATOR_FF             
0x0000673F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006741      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00006743      3  F31AB4                                               IMM16_F3                  u16_be=6836, u16_le=46106
0x00006746      1  FF                                                   TERMINATOR_FF             
0x00006747      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006749      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000674B     14  800C82C782A482B582E682A48148                         LEN8_STRING_CP932         length=12, text="どうしよう？"
0x00006759      1  FF                                                   TERMINATOR_FF             
0x0000675A      1  FF                                                   TERMINATOR_FF             
0x0000675B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000675D      2  0004                                                 WORD_00XX                 u16_be=4, low_byte=4
0x0000675F      6  80048D7382AD                                         LEN8_STRING_CP932         length=4, text="行く"
0x00006765      1  FF                                                   TERMINATOR_FF             
0x00006766      1  FF                                                   TERMINATOR_FF             
0x00006767      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006769      2  000A                                                 WORD_00XX                 u16_be=10, low_byte=10
0x0000676B      3  F31AB5                                               IMM16_F3                  u16_be=6837, u16_le=46362
0x0000676E      1  FF                                                   TERMINATOR_FF             
0x0000676F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00006771      2  0005                                                 WORD_00XX                 u16_be=5, low_byte=5
0x00006773     10  80088D7382A982C882A2                                 LEN8_STRING_CP932         length=8, text="行かない"
0x0000677D      1  FF                                                   TERMINATOR_FF             
0x0000677E      1  FF                                                   TERMINATOR_FF             
0x0000677F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006781      2  000B                                                 WORD_00XX                 u16_be=11, low_byte=11
0x00006783      3  F31AB6                                               IMM16_F3                  u16_be=6838, u16_le=46618
0x00006786      1  FF                                                   TERMINATOR_FF             
0x00006787      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006789      2  0008                                                 WORD_00XX                 u16_be=8, low_byte=8
0x0000678B      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x0000678D      1  FF                                                   TERMINATOR_FF             
0x0000678E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006790      2  0009                                                 WORD_00XX                 u16_be=9, low_byte=9
0x00006792      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00006794      1  FF                                                   TERMINATOR_FF             
0x00006795      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006797      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00006799      2  F212                                                 IMM8_F2                   u8=18, s8=18
0x0000679B      1  FF                                                   TERMINATOR_FF             
0x0000679C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000679E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000067A0      1  FF                                                   TERMINATOR_FF             
0x000067A1      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x000067A3      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x000067A6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000067A8      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x000067A9      1  FF                                                   TERMINATOR_FF             
0x000067AA      2  0067                                                 WORD_00XX                 u16_be=103, low_byte=103
0x000067AC      1  D2                                                   OPAQUE_RAW_BYTES          bytes=D2
0x000067AD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000067AF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000067B1      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x000067B3      1  FF                                                   TERMINATOR_FF             
0x000067B4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000067B6      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x000067B8     14  800C50533249303035612E62696E                         LEN8_STRING_CP932         length=12, text="PS2I005a.bin"
0x000067C6      1  FF                                                   TERMINATOR_FF             
0x000067C7      1  FF                                                   TERMINATOR_FF             
0x000067C8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000067CA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000067CC      1  FF                                                   TERMINATOR_FF             
0x000067CD      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x000067CF      2  0067                                                 WORD_00XX                 u16_be=103, low_byte=103
0x000067D1      1  FE                                                   OPAQUE_RAW_BYTES          bytes=FE
0x000067D2      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x000067D4      3  F10088                                               IMM16_F1                  u16_be=136, u16_le=34816
0x000067D7      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000067D9      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x000067DA      1  FF                                                   TERMINATOR_FF             
0x000067DB      2  0067                                                 WORD_00XX                 u16_be=103, low_byte=103
0x000067DD      1  FE                                                   OPAQUE_RAW_BYTES          bytes=FE
0x000067DE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000067E0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000067E2      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x000067E4      1  FF                                                   TERMINATOR_FF             
0x000067E5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000067E7      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x000067E9     14  800C50533246303031612E62696E                         LEN8_STRING_CP932         length=12, text="PS2F001a.bin"
0x000067F7      1  FF                                                   TERMINATOR_FF             
0x000067F8      1  FF                                                   TERMINATOR_FF             
0x000067F9      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000067FB      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000067FD      1  FF                                                   TERMINATOR_FF             
0x000067FE      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00006800      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00006802      1  FF                                                   TERMINATOR_FF             
0x00006803      2  0068                                                 WORD_00XX                 u16_be=104, low_byte=104
0x00006805      1  0D                                                   OPAQUE_RAW_BYTES          bytes=0D
0x00006806      2  0021                                                 WORD_00XX                 u16_be=33, low_byte=33
0x00006808      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x0000680A      2  0067                                                 WORD_00XX                 u16_be=103, low_byte=103
0x0000680C      1  FE                                                   OPAQUE_RAW_BYTES          bytes=FE
0x0000680D      1  FF                                                   TERMINATOR_FF             
0x0000680E      1  FF                                                   TERMINATOR_FF             
