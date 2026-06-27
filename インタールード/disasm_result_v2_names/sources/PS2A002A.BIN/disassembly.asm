; Full conservative disassembly for PS2A002A.BIN
; Columns: offset | size | raw bytes | mnemonic | operands/preview
; Unknown or ambiguous VM semantics are preserved as typed atoms / opaque bytes.

0x00000000      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000002      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000004      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000006      1  FF                                                   TERMINATOR_FF             
0x00000007      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000009      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000000B      2  F231                                                 IMM8_F2                   u8=49, s8=49
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
0x0000004B      3  F301F3                                               IMM16_F3                  u16_be=499, u16_le=62209
0x0000004E      1  FF                                                   TERMINATOR_FF             
0x0000004F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000051      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000053     60  803A9581926982CD8B4382C382A982C882A282AF82EA82C7...  LEN8_STRING_CP932         length=58, text="普段は気づかないけれど、街ではいろんなことが\n起きている。"
0x0000008F      1  FF                                                   TERMINATOR_FF             
0x00000090      1  FF                                                   TERMINATOR_FF             
0x00000091      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000093      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000095      1  FF                                                   TERMINATOR_FF             
0x00000096      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000098      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000009A      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000009C      1  FF                                                   TERMINATOR_FF             
0x0000009D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000009F      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000000A1      3  F33E9E                                               IMM16_F3                  u16_be=16030, u16_le=40510
0x000000A4      1  FF                                                   TERMINATOR_FF             
0x000000A5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000000A7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000000A9     28  801A82DA82AD82CD82BB82F182C882B182C682F08D6C82A6...  LEN8_STRING_CP932         length=26, text="ぼくはそんなことを考える。"
0x000000C5      1  FF                                                   TERMINATOR_FF             
0x000000C6      1  FF                                                   TERMINATOR_FF             
0x000000C7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000000C9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000000CB      1  FF                                                   TERMINATOR_FF             
0x000000CC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000CE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000D0      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000000D2      1  FF                                                   TERMINATOR_FF             
0x000000D3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000D5      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000000D7      3  F301F4                                               IMM16_F3                  u16_be=500, u16_le=62465
0x000000DA      1  FF                                                   TERMINATOR_FF             
0x000000DB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000000DD      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000000DF     90  805882B382C182AB82CC82B182C682E082BB82A481425C6E...  LEN8_STRING_CP932         length=88, text="さっきのこともそう。\nぼくたちは、大抵の場合、自分に関係のある\nことにしか目を向けない。"
0x00000139      1  FF                                                   TERMINATOR_FF             
0x0000013A      1  FF                                                   TERMINATOR_FF             
0x0000013B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000013D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000013F      1  FF                                                   TERMINATOR_FF             
0x00000140      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000142      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000144      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000146      1  FF                                                   TERMINATOR_FF             
0x00000147      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000149      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000014B      3  F301F5                                               IMM16_F3                  u16_be=501, u16_le=62721
0x0000014E      1  FF                                                   TERMINATOR_FF             
0x0000014F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000151      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000153    122  807882DC82A081418BCA82C882F182A982DD82BD82A282C9...  LEN8_STRING_CP932         length=120, text="まあ、玉なんかみたいに関係あろうとなかろうと、ありとあらゆることに余計な興味を向け続ける\nヤツもいることはいるんだけど。"
0x000001CD      1  FF                                                   TERMINATOR_FF             
0x000001CE      1  FF                                                   TERMINATOR_FF             
0x000001CF      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000001D1      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000001D3      1  FF                                                   TERMINATOR_FF             
0x000001D4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001D6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001D8      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000001DA      1  FF                                                   TERMINATOR_FF             
0x000001DB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001DD      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000001DF      3  F301F6                                               IMM16_F3                  u16_be=502, u16_le=62977
0x000001E2      1  FF                                                   TERMINATOR_FF             
0x000001E3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001E5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001E7     78  804C91E592EF82CC8FEA8D8782CD814182BE81425C6E82BB...  LEN8_STRING_CP932         length=76, text="大抵の場合は、だ。\nそうすると、街の多くの部分を見ないですごす\nことになる。"
0x00000235      1  FF                                                   TERMINATOR_FF             
0x00000236      1  FF                                                   TERMINATOR_FF             
0x00000237      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000239      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000023B      1  FF                                                   TERMINATOR_FF             
0x0000023C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000023E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000240      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000242      1  FF                                                   TERMINATOR_FF             
0x00000243      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000245      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000247      3  F301F7                                               IMM16_F3                  u16_be=503, u16_le=63233
0x0000024A      1  FF                                                   TERMINATOR_FF             
0x0000024B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000024D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000024F     20  8012924E82A982AA82A282C182C482A282BD8142             LEN8_STRING_CP932         length=18, text="誰かがいっていた。"
0x00000263      1  FF                                                   TERMINATOR_FF             
0x00000264      1  FF                                                   TERMINATOR_FF             
0x00000265      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000267      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000269      1  FF                                                   TERMINATOR_FF             
0x0000026A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000026C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000026E      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000270      1  FF                                                   TERMINATOR_FF             
0x00000271      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000273      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000275      3  F301F8                                               IMM16_F3                  u16_be=504, u16_le=63489
0x00000278      1  FF                                                   TERMINATOR_FF             
0x00000279      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000027B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000027D     98  806082DA82AD82BD82BF82CC90A28A4582CD965C97CD82C9...  LEN8_STRING_CP932         length=96, text="ぼくたちの世界は暴力に満ちあふれている。\nただ、それは目につかないよう隠されている\nだけだ、と。"
0x000002DF      1  FF                                                   TERMINATOR_FF             
0x000002E0      1  FF                                                   TERMINATOR_FF             
0x000002E1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000002E3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000002E5      1  FF                                                   TERMINATOR_FF             
0x000002E6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002E8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002EA      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000002EC      1  FF                                                   TERMINATOR_FF             
0x000002ED      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002EF      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000002F1      3  F301F9                                               IMM16_F3                  u16_be=505, u16_le=63745
0x000002F4      1  FF                                                   TERMINATOR_FF             
0x000002F5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002F7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002F9     64  803E82BB82A4814182DA82AD82BD82BF82AA8CA982C482A2...  LEN8_STRING_CP932         length=62, text="そう、ぼくたちが見ているこの場所は、見ているままの姿ではない。"
0x00000339      1  FF                                                   TERMINATOR_FF             
0x0000033A      1  FF                                                   TERMINATOR_FF             
0x0000033B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000033D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000033F      1  FF                                                   TERMINATOR_FF             
0x00000340      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000342      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000344      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000346      1  FF                                                   TERMINATOR_FF             
0x00000347      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000349      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000034B      3  F301FA                                               IMM16_F3                  u16_be=506, u16_le=64001
0x0000034E      1  FF                                                   TERMINATOR_FF             
0x0000034F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000351      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000353    110  806C82B182C782E082CC82C682AB82C982CD917A919C82B7...  LEN8_STRING_CP932         length=108, text="こどものときには想像することもできなかった。\nあの頃は、目にするものがすべてだったから\n（玉は今でもそう）。"
0x000003C1      1  FF                                                   TERMINATOR_FF             
0x000003C2      1  FF                                                   TERMINATOR_FF             
0x000003C3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000003C5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000003C7      1  FF                                                   TERMINATOR_FF             
0x000003C8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003CA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003CC      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000003CE      1  FF                                                   TERMINATOR_FF             
0x000003CF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003D1      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000003D3      3  F301FB                                               IMM16_F3                  u16_be=507, u16_le=64257
0x000003D6      1  FF                                                   TERMINATOR_FF             
0x000003D7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000003D9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000003DB     54  803482C582E081418CA982C482A282E982E082CC82C982BB...  LEN8_STRING_CP932         length=52, text="でも、見ているものにそれ以上の意味があると\nしたら。"
0x00000411      1  FF                                                   TERMINATOR_FF             
0x00000412      1  FF                                                   TERMINATOR_FF             
0x00000413      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000415      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000417      1  FF                                                   TERMINATOR_FF             
0x00000418      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000041A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000041C      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000041E      1  FF                                                   TERMINATOR_FF             
0x0000041F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000421      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000423      3  F301FC                                               IMM16_F3                  u16_be=508, u16_le=64513
0x00000426      1  FF                                                   TERMINATOR_FF             
0x00000427      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000429      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000042B     70  8044979D89F082C582AB82B88141917A919C82B782E982B1...  LEN8_STRING_CP932         length=68, text="理解できず、想像することさえ不可能な\nことがらにふれ続けるとしたら。"
0x00000471      1  FF                                                   TERMINATOR_FF             
0x00000472      1  FF                                                   TERMINATOR_FF             
0x00000473      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000475      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000477      1  FF                                                   TERMINATOR_FF             
0x00000478      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000047A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000047C      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000047E      1  FF                                                   TERMINATOR_FF             
0x0000047F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000481      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000483      3  F301FD                                               IMM16_F3                  u16_be=509, u16_le=64769
0x00000486      1  FF                                                   TERMINATOR_FF             
0x00000487      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000489      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000048B     56  803682BB82EA82CD814182A982C882E8836E815B836882C8...  LEN8_STRING_CP932         length=54, text="それは、かなりハードな毎日になるのだろう、\nとも思う。"
0x000004C3      1  FF                                                   TERMINATOR_FF             
0x000004C4      1  FF                                                   TERMINATOR_FF             
0x000004C5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000004C7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000004C9      1  FF                                                   TERMINATOR_FF             
0x000004CA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004CC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004CE      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000004D0      1  FF                                                   TERMINATOR_FF             
0x000004D1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000004D3      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000004D5      3  F301FE                                               IMM16_F3                  u16_be=510, u16_le=65025
0x000004D8      1  FF                                                   TERMINATOR_FF             
0x000004D9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000004DB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000004DD     48  802E82E082BF82EB82F1814182BB82F182C882B182C682C9...  LEN8_STRING_CP932         length=46, text="もちろん、そんなことにならないのは知っていた。"
0x0000050D      1  FF                                                   TERMINATOR_FF             
0x0000050E      1  FF                                                   TERMINATOR_FF             
0x0000050F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000511      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000513      1  FF                                                   TERMINATOR_FF             
0x00000514      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000516      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000518      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000051A      1  FF                                                   TERMINATOR_FF             
0x0000051B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000051D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000051F      3  F301FF                                               IMM16_F3                  u16_be=511, u16_le=65281
0x00000522      1  FF                                                   TERMINATOR_FF             
0x00000523      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000525      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000527     96  805E91E5906C82C982C882E982C682A282A482B182C682CD...  LEN8_STRING_CP932         length=94, text="大人になるということは、そんな理解できない\nさまざまな事柄と妥協し、慣れるということ\nだから。"
0x00000587      1  FF                                                   TERMINATOR_FF             
0x00000588      1  FF                                                   TERMINATOR_FF             
0x00000589      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000058B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000058D      1  FF                                                   TERMINATOR_FF             
0x0000058E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000590      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000592      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000594      1  FF                                                   TERMINATOR_FF             
0x00000595      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000597      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000599      3  F30200                                               IMM16_F3                  u16_be=512, u16_le=2
0x0000059C      1  FF                                                   TERMINATOR_FF             
0x0000059D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000059F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000005A1     64  803E82BB82A48D6C82A682E982C681418BCA82CD88EA90B6...  LEN8_STRING_CP932         length=62, text="そう考えると、玉は一生こどものままなわけ\nだけど、アレは例外。"
0x000005E1      1  FF                                                   TERMINATOR_FF             
0x000005E2      1  FF                                                   TERMINATOR_FF             
0x000005E3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000005E5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000005E7      1  FF                                                   TERMINATOR_FF             
0x000005E8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005EA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005EC      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000005EE      1  FF                                                   TERMINATOR_FF             
0x000005EF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005F1      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000005F3      3  F30201                                               IMM16_F3                  u16_be=513, u16_le=258
0x000005F6      1  FF                                                   TERMINATOR_FF             
0x000005F7      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000005F9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000005FB     80  804E82A082A282C282CD8AB582EA82E982B182C682F08BC9...  LEN8_STRING_CP932         length=78, text="あいつは慣れることを極端に嫌うから。\nでも、普通の人間は、慣れなきゃいけない。"
0x0000064B      1  FF                                                   TERMINATOR_FF             
0x0000064C      1  FF                                                   TERMINATOR_FF             
0x0000064D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000064F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000651      1  FF                                                   TERMINATOR_FF             
0x00000652      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000654      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000656      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000658      1  FF                                                   TERMINATOR_FF             
0x00000659      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000065B      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000065D      3  F30202                                               IMM16_F3                  u16_be=514, u16_le=514
0x00000660      1  FF                                                   TERMINATOR_FF             
0x00000661      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000663      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000665    100  8062968893FA82AA82A282C282C582E09056914E82C882E7...  LEN8_STRING_CP932         length=98, text="毎日がいつでも新鮮なら、それはそれでいいこと\nなのだろうけど、時間がいくらあっても足りなく\nなる。"
0x000006C9      1  FF                                                   TERMINATOR_FF             
0x000006CA      1  FF                                                   TERMINATOR_FF             
0x000006CB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000006CD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000006CF      1  FF                                                   TERMINATOR_FF             
0x000006D0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006D2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006D4      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000006D6      1  FF                                                   TERMINATOR_FF             
0x000006D7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006D9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000006DB      3  F30203                                               IMM16_F3                  u16_be=515, u16_le=770
0x000006DE      1  FF                                                   TERMINATOR_FF             
0x000006DF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006E1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006E3     72  804682DA82AD82E782CD8141968893FA82F082AB82BF82F1...  LEN8_STRING_CP932         length=70, text="ぼくらは、毎日をきちんと過ごすために、目を\nつぶらなきゃいけなくなる。"
0x0000072B      1  FF                                                   TERMINATOR_FF             
0x0000072C      1  FF                                                   TERMINATOR_FF             
0x0000072D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000072F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000731      1  FF                                                   TERMINATOR_FF             
0x00000732      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000734      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000736      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000738      1  FF                                                   TERMINATOR_FF             
0x00000739      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000073B      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000073D      3  F30204                                               IMM16_F3                  u16_be=516, u16_le=1026
0x00000740      1  FF                                                   TERMINATOR_FF             
0x00000741      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000743      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000745     28  801A82BB82EA82CD81418E6495FB82CC82C882A282B182C6...  LEN8_STRING_CP932         length=26, text="それは、仕方のないことだ。"
0x00000761      1  FF                                                   TERMINATOR_FF             
0x00000762      1  FF                                                   TERMINATOR_FF             
0x00000763      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000765      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000767      1  FF                                                   TERMINATOR_FF             
0x00000768      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000076A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000076C      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000076E      1  FF                                                   TERMINATOR_FF             
0x0000076F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000771      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000773      3  F30205                                               IMM16_F3                  u16_be=517, u16_le=1282
0x00000776      1  FF                                                   TERMINATOR_FF             
0x00000777      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000779      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000077B    104  80668A778D5A8163816382C682A282A48FEA8F8A82AA8141...  LEN8_STRING_CP932         length=102, text="学校……という場所が、ぼくたちに慣れることを\n教えてくれる。\nそして、ぼくたちは社会に出て行くのだし。"
0x000007E3      1  FF                                                   TERMINATOR_FF             
0x000007E4      1  FF                                                   TERMINATOR_FF             
0x000007E5      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000007E7      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000007E9      1  FF                                                   TERMINATOR_FF             
0x000007EA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007EC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007EE      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000007F0      1  FF                                                   TERMINATOR_FF             
0x000007F1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007F3      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000007F5      3  F30206                                               IMM16_F3                  u16_be=518, u16_le=1538
0x000007F8      1  FF                                                   TERMINATOR_FF             
0x000007F9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000007FB      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000007FD     64  803E82DA82AD82CD82BB82A48D6C82A682C482A282BD82B5...  LEN8_STRING_CP932         length=62, text="ぼくはそう考えていたし、ぼく以外の連中も\n薄々は勘づいている。"
0x0000083D      1  FF                                                   TERMINATOR_FF             
0x0000083E      1  FF                                                   TERMINATOR_FF             
0x0000083F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000841      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000843      1  FF                                                   TERMINATOR_FF             
0x00000844      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000846      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000848      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000084A      1  FF                                                   TERMINATOR_FF             
0x0000084B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000084D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000084F      3  F30207                                               IMM16_F3                  u16_be=519, u16_le=1794
0x00000852      1  FF                                                   TERMINATOR_FF             
0x00000853      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000855      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000857     90  805882E082BF82EB82F1814182DA82AD88C88A4F82CC9841...  LEN8_STRING_CP932         length=88, text="もちろん、ぼく以外の連中に玉は含まれない\n（ま、念のため）。\nずっと……そう思ってきた。"
0x000008B1      1  FF                                                   TERMINATOR_FF             
0x000008B2      1  FF                                                   TERMINATOR_FF             
0x000008B3      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000008B5      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000008B7      1  FF                                                   TERMINATOR_FF             
0x000008B8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008BA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008BC      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000008BE      1  FF                                                   TERMINATOR_FF             
0x000008BF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008C1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000008C3      3  F3027D                                               IMM16_F3                  u16_be=637, u16_le=32002
0x000008C6      1  FF                                                   TERMINATOR_FF             
0x000008C7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008C9      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000008CB      2  F231                                                 IMM8_F2                   u8=49, s8=49
0x000008CD      1  FF                                                   TERMINATOR_FF             
0x000008CE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008D0      2  0004                                                 WORD_00XX                 u16_be=4, low_byte=4
0x000008D2      2  F20A                                                 IMM8_F2                   u8=10, s8=10
0x000008D4      1  FF                                                   TERMINATOR_FF             
0x000008D5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008D7      2  0005                                                 WORD_00XX                 u16_be=5, low_byte=5
0x000008D9      2  F23C                                                 IMM8_F2                   u8=60, s8=60
0x000008DB      1  FF                                                   TERMINATOR_FF             
0x000008DC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008DE      2  0006                                                 WORD_00XX                 u16_be=6, low_byte=6
0x000008E0      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000008E2      1  FF                                                   TERMINATOR_FF             
0x000008E3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000008E5      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000008E7      8  800673652D653333                                     LEN8_STRING_CP932         length=6, text="se-e33"
0x000008EF      1  FF                                                   TERMINATOR_FF             
0x000008F0      1  FF                                                   TERMINATOR_FF             
0x000008F1      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000008F3      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000008F5      1  FF                                                   TERMINATOR_FF             
0x000008F6      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008F8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008FA      2  F20A                                                 IMM8_F2                   u8=10, s8=10
0x000008FC      1  FF                                                   TERMINATOR_FF             
0x000008FD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008FF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000901      2  F22A                                                 IMM8_F2                   u8=42, s8=42
0x00000903      1  FF                                                   TERMINATOR_FF             
0x00000904      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000906      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000908      1  FF                                                   TERMINATOR_FF             
0x00000909      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000090B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000090D      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000090F      1  FF                                                   TERMINATOR_FF             
0x00000910      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000912      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000914      3  F30208                                               IMM16_F3                  u16_be=520, u16_le=2050
0x00000917      1  FF                                                   TERMINATOR_FF             
0x00000918      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000091A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000091C    112  806E82BE82AF82C7814182BB82CC8D6C82A682AA814182A0...  LEN8_STRING_CP932         length=110, text="だけど、その考えが、あの朝、揺らいだ。\nあまりに異質な瞬間だった。\n混雑しているはずのホームが無人になり……。"
0x0000098C      1  FF                                                   TERMINATOR_FF             
0x0000098D      1  FF                                                   TERMINATOR_FF             
0x0000098E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000990      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000992      1  FF                                                   TERMINATOR_FF             
0x00000993      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000995      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000997      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000999      1  FF                                                   TERMINATOR_FF             
0x0000099A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000099C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000099E      3  F30209                                               IMM16_F3                  u16_be=521, u16_le=2306
0x000009A1      1  FF                                                   TERMINATOR_FF             
0x000009A2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009A4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009A6     46  802C8CB682C682A282A482C982CD814182A082DC82E882C9...  LEN8_STRING_CP932         length=44, text="幻というには、あまりに鮮明すぎる光景だった。"
0x000009D4      1  FF                                                   TERMINATOR_FF             
0x000009D5      1  FF                                                   TERMINATOR_FF             
0x000009D6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000009D8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000009DA      1  FF                                                   TERMINATOR_FF             
0x000009DB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009DD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009DF      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000009E1      1  FF                                                   TERMINATOR_FF             
0x000009E2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009E4      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000009E6      3  F3020A                                               IMM16_F3                  u16_be=522, u16_le=2562
0x000009E9      1  FF                                                   TERMINATOR_FF             
0x000009EA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009EC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009EE    100  806282DA82AD82CD94DE8F9782A982E796DA82F082CD82C8...  LEN8_STRING_CP932         length=98, text="ぼくは彼女から目をはなすことができなくなった。\nそして、単語帳から目をあげた彼女が、ぼくを\n見た。"
0x00000A52      1  FF                                                   TERMINATOR_FF             
0x00000A53      1  FF                                                   TERMINATOR_FF             
0x00000A54      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000A56      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000A58      1  FF                                                   TERMINATOR_FF             
0x00000A59      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A5B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A5D      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000A5F      1  FF                                                   TERMINATOR_FF             
0x00000A60      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000A62      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000A64      3  F3020B                                               IMM16_F3                  u16_be=523, u16_le=2818
0x00000A67      1  FF                                                   TERMINATOR_FF             
0x00000A68      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A6A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000A6C    120  807694DE8F9782CC93B582CC899C82C9814182DA82AD82CD...  LEN8_STRING_CP932         length=118, text="彼女の瞳の奥に、ぼくはなにかを見た、と思った。\nそれは、ぼくたちの毎日の中では見つけることの\nできない、なにかだった。"
0x00000AE4      1  FF                                                   TERMINATOR_FF             
0x00000AE5      1  FF                                                   TERMINATOR_FF             
0x00000AE6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000AE8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000AEA      1  FF                                                   TERMINATOR_FF             
0x00000AEB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AED      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AEF      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000AF1      1  FF                                                   TERMINATOR_FF             
0x00000AF2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000AF4      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000AF6      3  F3020C                                               IMM16_F3                  u16_be=524, u16_le=3074
0x00000AF9      1  FF                                                   TERMINATOR_FF             
0x00000AFA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000AFC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000AFE     46  802C94DE8F9782CD814182C882BA8163816382A082F182C8...  LEN8_STRING_CP932         length=44, text="彼女は、なぜ……あんな目をしていたのだろう。"
0x00000B2C      1  FF                                                   TERMINATOR_FF             
0x00000B2D      1  FF                                                   TERMINATOR_FF             
0x00000B2E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000B30      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000B32      1  FF                                                   TERMINATOR_FF             
0x00000B33      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B35      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B37      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000B39      1  FF                                                   TERMINATOR_FF             
0x00000B3A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B3C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000B3E      3  F3020D                                               IMM16_F3                  u16_be=525, u16_le=3330
0x00000B41      1  FF                                                   TERMINATOR_FF             
0x00000B42      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B44      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B46     58  803882BB82EA82AA8B4382C982C882C182BD81425C6E94DE...  LEN8_STRING_CP932         length=56, text="それが気になった。\n彼女はなにを……考えていたのだろう。"
0x00000B80      1  FF                                                   TERMINATOR_FF             
0x00000B81      1  FF                                                   TERMINATOR_FF             
0x00000B82      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000B84      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000B86      1  FF                                                   TERMINATOR_FF             
0x00000B87      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B89      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B8B      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000B8D      1  FF                                                   TERMINATOR_FF             
0x00000B8E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000B90      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000B92      3  F3020E                                               IMM16_F3                  u16_be=526, u16_le=3586
0x00000B95      1  FF                                                   TERMINATOR_FF             
0x00000B96      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B98      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000B9A     58  8038968893FA82CC97A091A482C9814194DE8F9782CC93B5...  LEN8_STRING_CP932         length=56, text="毎日の裏側に、彼女の瞳が隠されているのだと\nしたら……。"
0x00000BD4      1  FF                                                   TERMINATOR_FF             
0x00000BD5      1  FF                                                   TERMINATOR_FF             
0x00000BD6      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000BD8      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000BDA      1  FF                                                   TERMINATOR_FF             
0x00000BDB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BDD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BDF      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000BE1      1  FF                                                   TERMINATOR_FF             
0x00000BE2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000BE4      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000BE6      3  F3020F                                               IMM16_F3                  u16_be=527, u16_le=3842
0x00000BE9      1  FF                                                   TERMINATOR_FF             
0x00000BEA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000BEC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000BEE    104  806682BB82EA82C98AB582EA82E982B182C682C882C78141...  LEN8_STRING_CP932         length=102, text="それに慣れることなど、できそうになかった。\n彼女の瞳は、それくらい強い意志をひめている\nように思えた。"
0x00000C56      1  FF                                                   TERMINATOR_FF             
0x00000C57      1  FF                                                   TERMINATOR_FF             
0x00000C58      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000C5A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000C5C      1  FF                                                   TERMINATOR_FF             
0x00000C5D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C5F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C61      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000C63      1  FF                                                   TERMINATOR_FF             
0x00000C64      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000C66      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000C68      3  F33E9F                                               IMM16_F3                  u16_be=16031, u16_le=40766
0x00000C6B      1  FF                                                   TERMINATOR_FF             
0x00000C6C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C6E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000C70    124  807A82BB82F182C882B182C682F08D6C82A682C882AA82E7...  LEN8_STRING_CP932         length=122, text="そんなことを考えながら、ぼくは街を歩き続けた。無駄なことかもしれないけれど、それなりに\n気持ちが落ちついていくのを感じる。"
0x00000CEC      1  FF                                                   TERMINATOR_FF             
0x00000CED      1  FF                                                   TERMINATOR_FF             
0x00000CEE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000CF0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000CF2      1  FF                                                   TERMINATOR_FF             
0x00000CF3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CF5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CF7      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000CF9      1  FF                                                   TERMINATOR_FF             
0x00000CFA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000CFC      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000CFE      3  F33EA0                                               IMM16_F3                  u16_be=16032, u16_le=41022
0x00000D01      1  FF                                                   TERMINATOR_FF             
0x00000D02      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D04      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D06     50  803082E082A48FAD82B595E082B182A481425C6E8DA19378...  LEN8_STRING_CP932         length=48, text="もう少し歩こう。\n今度は頭をからっぽにして……。"
0x00000D38      1  FF                                                   TERMINATOR_FF             
0x00000D39      1  FF                                                   TERMINATOR_FF             
0x00000D3A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000D3C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000D3E      1  FF                                                   TERMINATOR_FF             
0x00000D3F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D41      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D43      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x00000D45      1  FF                                                   TERMINATOR_FF             
0x00000D46      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D48      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D4A      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000D4C      1  FF                                                   TERMINATOR_FF             
0x00000D4D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000D4F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000D51      1  FF                                                   TERMINATOR_FF             
0x00000D52      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D54      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D56      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000D58      1  FF                                                   TERMINATOR_FF             
0x00000D59      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D5B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D5D      2  F2FF                                                 IMM8_F2                   u8=255, s8=-1
0x00000D5F      1  FF                                                   TERMINATOR_FF             
0x00000D60      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000D62      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000D64      1  FF                                                   TERMINATOR_FF             
0x00000D65      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D67      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D69      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x00000D6B      1  FF                                                   TERMINATOR_FF             
0x00000D6C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D6E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000D70      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000D72      1  FF                                                   TERMINATOR_FF             
0x00000D73      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D75      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000D77      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x00000D79      1  FF                                                   TERMINATOR_FF             
0x00000D7A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000D7C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000D7E      1  FF                                                   TERMINATOR_FF             
0x00000D7F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D81      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D83      2  F20B                                                 IMM8_F2                   u8=11, s8=11
0x00000D85      1  FF                                                   TERMINATOR_FF             
0x00000D86      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000D88      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000D8A      1  FF                                                   TERMINATOR_FF             
0x00000D8B      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00000D8D      3  F10041                                               IMM16_F1                  u16_be=65, u16_le=16640
0x00000D90      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000D92      1  35                                                   OPAQUE_RAW_BYTES          bytes=35
0x00000D93      1  FF                                                   TERMINATOR_FF             
0x00000D94      2  000D                                                 WORD_00XX                 u16_be=13, low_byte=13
0x00000D96      1  BC                                                   OPAQUE_RAW_BYTES          bytes=BC
0x00000D97      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D99      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000D9B      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000D9D      1  FF                                                   TERMINATOR_FF             
0x00000D9E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000DA0      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x00000DA2     14  800C50533254303130612E62696E                         LEN8_STRING_CP932         length=12, text="PS2T010a.bin"
0x00000DB0      1  FF                                                   TERMINATOR_FF             
0x00000DB1      1  FF                                                   TERMINATOR_FF             
0x00000DB2      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000DB4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000DB6      1  FF                                                   TERMINATOR_FF             
0x00000DB7      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00000DB9      2  000D                                                 WORD_00XX                 u16_be=13, low_byte=13
0x00000DBB      1  DC                                                   OPAQUE_RAW_BYTES          bytes=DC
0x00000DBC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000DBE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000DC0      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x00000DC2      1  FF                                                   TERMINATOR_FF             
0x00000DC3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000DC5      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x00000DC7     14  800C50533246303032612E62696E                         LEN8_STRING_CP932         length=12, text="PS2F002a.bin"
0x00000DD5      1  FF                                                   TERMINATOR_FF             
0x00000DD6      1  FF                                                   TERMINATOR_FF             
0x00000DD7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000DD9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000DDB      1  FF                                                   TERMINATOR_FF             
0x00000DDC      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x00000DDE      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000DE0      1  FF                                                   TERMINATOR_FF             
0x00000DE1      2  000D                                                 WORD_00XX                 u16_be=13, low_byte=13
0x00000DE3      1  EB                                                   OPAQUE_RAW_BYTES          bytes=EB
0x00000DE4      2  0021                                                 WORD_00XX                 u16_be=33, low_byte=33
0x00000DE6      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x00000DE8      2  000D                                                 WORD_00XX                 u16_be=13, low_byte=13
0x00000DEA      1  DC                                                   OPAQUE_RAW_BYTES          bytes=DC
0x00000DEB      1  FF                                                   TERMINATOR_FF             
0x00000DEC      1  FF                                                   TERMINATOR_FF             
