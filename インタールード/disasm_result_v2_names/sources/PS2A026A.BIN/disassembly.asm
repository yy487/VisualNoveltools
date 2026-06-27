; Full conservative disassembly for PS2A026A.BIN
; Columns: offset | size | raw bytes | mnemonic | operands/preview
; Unknown or ambiguous VM semantics are preserved as typed atoms / opaque bytes.

0x00000000      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000002      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000004      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000006      1  FF                                                   TERMINATOR_FF             
0x00000007      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000009      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000000B      2  F254                                                 IMM8_F2                   u8=84, s8=84
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
0x0000001E      2  F202                                                 IMM8_F2                   u8=2, s8=2
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
0x0000004B      3  F33F3E                                               IMM16_F3                  u16_be=16190, u16_le=15935
0x0000004E      1  FF                                                   TERMINATOR_FF             
0x0000004F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000051      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000053     24  801682A082EA82A982E7909493FA82AA82B782AC82BD8142     LEN8_STRING_CP932         length=22, text="あれから数日がすぎた。"
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
0x0000007D      3  F33F3F                                               IMM16_F3                  u16_be=16191, u16_le=16191
0x00000080      1  FF                                                   TERMINATOR_FF             
0x00000081      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000083      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000085    112  806E834A83458393835A8389815B82A982E782CC8CC482D1...  LEN8_STRING_CP932         length=110, text="カウンセラーからの呼び出しはもうなかった。\nうわべの平穏な毎日は続き、ぼくも表面だけは\nそれにつきあっていた。"
0x000000F5      1  FF                                                   TERMINATOR_FF             
0x000000F6      1  FF                                                   TERMINATOR_FF             
0x000000F7      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000000F9      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000000FB      1  FF                                                   TERMINATOR_FF             
0x000000FC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000000FE      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000100      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000102      1  FF                                                   TERMINATOR_FF             
0x00000103      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000105      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000107      2  F24D                                                 IMM8_F2                   u8=77, s8=77
0x00000109      1  FF                                                   TERMINATOR_FF             
0x0000010A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000010C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000010E      1  FF                                                   TERMINATOR_FF             
0x0000010F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000111      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000113      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000115      1  FF                                                   TERMINATOR_FF             
0x00000116      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000118      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000011A      2  F243                                                 IMM8_F2                   u8=67, s8=67
0x0000011C      1  FF                                                   TERMINATOR_FF             
0x0000011D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000011F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000121      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000123      1  FF                                                   TERMINATOR_FF             
0x00000124      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000126      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000128      1  FF                                                   TERMINATOR_FF             
0x00000129      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000012B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000012D      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000012F      1  FF                                                   TERMINATOR_FF             
0x00000130      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000132      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000134      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000136      1  FF                                                   TERMINATOR_FF             
0x00000137      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000139      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000013B      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000013D      1  FF                                                   TERMINATOR_FF             
0x0000013E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000140      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000142      1  FF                                                   TERMINATOR_FF             
0x00000143      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000145      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000147      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000149      1  FF                                                   TERMINATOR_FF             
0x0000014A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000014C      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000014E      3  F30551                                               IMM16_F3                  u16_be=1361, u16_le=20741
0x00000151      1  FF                                                   TERMINATOR_FF             
0x00000152      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000154      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000156     34  80208343838F836D83728362836082CC8CC395B682CD91B1...  LEN8_STRING_CP932         length=32, text="イワノビッチの古文は続いていた。"
0x00000178      1  FF                                                   TERMINATOR_FF             
0x00000179      1  FF                                                   TERMINATOR_FF             
0x0000017A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000017C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000017E      1  FF                                                   TERMINATOR_FF             
0x0000017F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000181      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000183      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000185      1  FF                                                   TERMINATOR_FF             
0x00000186      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000188      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000018A      3  F30552                                               IMM16_F3                  u16_be=1362, u16_le=20997
0x0000018D      1  FF                                                   TERMINATOR_FF             
0x0000018E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000190      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000192     80  804E92868AD483658358836782CD8BDF82A282CC82BE82AA...  LEN8_STRING_CP932         length=78, text="中間テストは近いのだが、熱心に聞いているのはほんの数人。強烈な眠気を誘う時間。"
0x000001E2      1  FF                                                   TERMINATOR_FF             
0x000001E3      1  FF                                                   TERMINATOR_FF             
0x000001E4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000001E6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000001E8      1  FF                                                   TERMINATOR_FF             
0x000001E9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001EB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001ED      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000001EF      1  FF                                                   TERMINATOR_FF             
0x000001F0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000001F2      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000001F4      3  F30553                                               IMM16_F3                  u16_be=1363, u16_le=21253
0x000001F7      1  FF                                                   TERMINATOR_FF             
0x000001F8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001FA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000001FC     82  805082DA82AD82CD95CA82C888D396A182C58EF68BC682C9...  LEN8_STRING_CP932         length=80, text="ぼくは別な意味で授業にははいりこめない。\n考えはいつでもあの瞬間に戻ってしまう。"
0x0000024E      1  FF                                                   TERMINATOR_FF             
0x0000024F      1  FF                                                   TERMINATOR_FF             
0x00000250      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000252      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000254      1  FF                                                   TERMINATOR_FF             
0x00000255      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000257      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000259      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x0000025B      1  FF                                                   TERMINATOR_FF             
0x0000025C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000025E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000260      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000262      1  FF                                                   TERMINATOR_FF             
0x00000263      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000265      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000267      1  FF                                                   TERMINATOR_FF             
0x00000268      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000026A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000026C      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x0000026E      1  FF                                                   TERMINATOR_FF             
0x0000026F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000271      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000273      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000275      1  FF                                                   TERMINATOR_FF             
0x00000276      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000278      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000027A      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x0000027C      1  FF                                                   TERMINATOR_FF             
0x0000027D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000027F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000281      1  FF                                                   TERMINATOR_FF             
0x00000282      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000284      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000286      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000288      1  FF                                                   TERMINATOR_FF             
0x00000289      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000028B      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000028D      3  F30096                                               IMM16_F3                  u16_be=150, u16_le=38400
0x00000290      1  FF                                                   TERMINATOR_FF             
0x00000291      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000293      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000295      1  FF                                                   TERMINATOR_FF             
0x00000296      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000298      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000029A      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000029C      1  FF                                                   TERMINATOR_FF             
0x0000029D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000029F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002A1      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000002A3      1  FF                                                   TERMINATOR_FF             
0x000002A4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002A6      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000002A8      2  F20A                                                 IMM8_F2                   u8=10, s8=10
0x000002AA      1  FF                                                   TERMINATOR_FF             
0x000002AB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000002AD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000002AF      1  FF                                                   TERMINATOR_FF             
0x000002B0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002B2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002B4      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x000002B6      1  FF                                                   TERMINATOR_FF             
0x000002B7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002B9      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002BB      2  F24D                                                 IMM8_F2                   u8=77, s8=77
0x000002BD      1  FF                                                   TERMINATOR_FF             
0x000002BE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000002C0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000002C2      1  FF                                                   TERMINATOR_FF             
0x000002C3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002C5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002C7      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000002C9      1  FF                                                   TERMINATOR_FF             
0x000002CA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002CC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002CE      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000002D0      1  FF                                                   TERMINATOR_FF             
0x000002D1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002D3      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000002D5      2  F23C                                                 IMM8_F2                   u8=60, s8=60
0x000002D7      1  FF                                                   TERMINATOR_FF             
0x000002D8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000002DA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000002DC      1  FF                                                   TERMINATOR_FF             
0x000002DD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002DF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002E1      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x000002E3      1  FF                                                   TERMINATOR_FF             
0x000002E4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002E6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002E8      3  F300A4                                               IMM16_F3                  u16_be=164, u16_le=41984
0x000002EB      1  FF                                                   TERMINATOR_FF             
0x000002EC      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000002EE      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000002F0      1  FF                                                   TERMINATOR_FF             
0x000002F1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002F3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002F5      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000002F7      1  FF                                                   TERMINATOR_FF             
0x000002F8      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000002FA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000002FC      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000002FE      1  FF                                                   TERMINATOR_FF             
0x000002FF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000301      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000303      2  F20A                                                 IMM8_F2                   u8=10, s8=10
0x00000305      1  FF                                                   TERMINATOR_FF             
0x00000306      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000308      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000030A      1  FF                                                   TERMINATOR_FF             
0x0000030B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000030D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000030F      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x00000311      1  FF                                                   TERMINATOR_FF             
0x00000312      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000314      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000316      2  F24D                                                 IMM8_F2                   u8=77, s8=77
0x00000318      1  FF                                                   TERMINATOR_FF             
0x00000319      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000031B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000031D      1  FF                                                   TERMINATOR_FF             
0x0000031E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000320      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000322      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x00000324      1  FF                                                   TERMINATOR_FF             
0x00000325      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000327      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000329      2  F243                                                 IMM8_F2                   u8=67, s8=67
0x0000032B      1  FF                                                   TERMINATOR_FF             
0x0000032C      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000032E      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000330      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000332      1  FF                                                   TERMINATOR_FF             
0x00000333      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000335      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000337      1  FF                                                   TERMINATOR_FF             
0x00000338      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000033A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000033C      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000033E      1  FF                                                   TERMINATOR_FF             
0x0000033F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000341      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000343      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000345      1  FF                                                   TERMINATOR_FF             
0x00000346      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000348      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000034A      2  F23C                                                 IMM8_F2                   u8=60, s8=60
0x0000034C      1  FF                                                   TERMINATOR_FF             
0x0000034D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000034F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000351      1  FF                                                   TERMINATOR_FF             
0x00000352      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000354      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000356      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000358      1  FF                                                   TERMINATOR_FF             
0x00000359      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000035B      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000035D      3  F30554                                               IMM16_F3                  u16_be=1364, u16_le=21509
0x00000360      1  FF                                                   TERMINATOR_FF             
0x00000361      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000363      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000365     52  8032837A815B838082C58CA982BD817794DE8F9781788142...  LEN8_STRING_CP932         length=50, text="ホームで見た『彼女』。\n幻の中で見た自分自身の姿。"
0x00000399      1  FF                                                   TERMINATOR_FF             
0x0000039A      1  FF                                                   TERMINATOR_FF             
0x0000039B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000039D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000039F      1  FF                                                   TERMINATOR_FF             
0x000003A0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003A2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003A4      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000003A6      1  FF                                                   TERMINATOR_FF             
0x000003A7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000003A9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000003AB      3  F30555                                               IMM16_F3                  u16_be=1365, u16_le=21765
0x000003AE      1  FF                                                   TERMINATOR_FF             
0x000003AF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000003B1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000003B3    104  80668EA995AA82AA95E982E782B78FEA8F8A82C582CD82C8...  LEN8_STRING_CP932         length=102, text="自分が暮らす場所ではなく、『彼女』の場所こそ現実かもしれない。\nそんな疑問が何度でもよみがえってくる。"
0x0000041B      1  FF                                                   TERMINATOR_FF             
0x0000041C      1  FF                                                   TERMINATOR_FF             
0x0000041D      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000041F      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000421      1  FF                                                   TERMINATOR_FF             
0x00000422      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000424      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000426      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000428      1  FF                                                   TERMINATOR_FF             
0x00000429      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000042B      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000042D      3  F33F40                                               IMM16_F3                  u16_be=16192, u16_le=16447
0x00000430      1  FF                                                   TERMINATOR_FF             
0x00000431      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000433      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000435     84  805294DE8F9782CC8B4C89AF82AA814182DA82AD82F08F95...  LEN8_STRING_CP932         length=82, text="彼女の記憶が、ぼくを助けてくれたのだと\nしたら……。\n彼女と彼女の場所こそが……。"
0x00000489      1  FF                                                   TERMINATOR_FF             
0x0000048A      1  FF                                                   TERMINATOR_FF             
0x0000048B      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000048D      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000048F      1  FF                                                   TERMINATOR_FF             
0x00000490      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000492      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000494      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000496      1  FF                                                   TERMINATOR_FF             
0x00000497      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000499      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000049B      3  F30556                                               IMM16_F3                  u16_be=1366, u16_le=22021
0x0000049E      1  FF                                                   TERMINATOR_FF             
0x0000049F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000004A1      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000004A3      9  800749573030313131                                   LEN8_STRING_CP932         length=7, text="IW00111"
0x000004AC      1  FF                                                   TERMINATOR_FF             
0x000004AD      1  FF                                                   TERMINATOR_FF             
0x000004AE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000004B0      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000004B2     88  8056817582B182C182A982E782E082ED82A982F182BE82AF...  LEN8_STRING_CP932         length=86, text="「こっからもわかんだけどよー、昔の人はなー、あの世を別世界とは思ってなかったんだなー」"
0x0000050A      1  FF                                                   TERMINATOR_FF             
0x0000050B      1  FF                                                   TERMINATOR_FF             
0x0000050C      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000050E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000510      1  FF                                                   TERMINATOR_FF             
0x00000511      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000513      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000515      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000517      1  FF                                                   TERMINATOR_FF             
0x00000518      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000051A      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000051C      3  F30557                                               IMM16_F3                  u16_be=1367, u16_le=22277
0x0000051F      1  FF                                                   TERMINATOR_FF             
0x00000520      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000522      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000524      9  800749573030313132                                   LEN8_STRING_CP932         length=7, text="IW00112"
0x0000052D      1  FF                                                   TERMINATOR_FF             
0x0000052E      1  FF                                                   TERMINATOR_FF             
0x0000052F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000531      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000533    118  8074817589A990F295BD8DE282C182C493BB82F0897A82A6...  LEN8_STRING_CP932         length=116, text="「黄泉平坂って峠を越えると死後の世界になるんで、今だとなー、国境とか赤道とか日付変更線\nみたいに考えてたわけだなー」"
0x000005A9      1  FF                                                   TERMINATOR_FF             
0x000005AA      1  FF                                                   TERMINATOR_FF             
0x000005AB      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000005AD      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000005AF      1  FF                                                   TERMINATOR_FF             
0x000005B0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005B2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005B4      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000005B6      1  FF                                                   TERMINATOR_FF             
0x000005B7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005B9      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000005BB      3  F30558                                               IMM16_F3                  u16_be=1368, u16_le=22533
0x000005BE      1  FF                                                   TERMINATOR_FF             
0x000005BF      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000005C1      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000005C3     40  80268343838F836D83728362836082CC90BA82AA8EA882C9...  LEN8_STRING_CP932         length=38, text="イワノビッチの声が耳に飛び込んで来た。"
0x000005EB      1  FF                                                   TERMINATOR_FF             
0x000005EC      1  FF                                                   TERMINATOR_FF             
0x000005ED      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000005EF      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000005F1      1  FF                                                   TERMINATOR_FF             
0x000005F2      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005F4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005F6      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000005F8      1  FF                                                   TERMINATOR_FF             
0x000005F9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000005FB      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000005FD      3  F30559                                               IMM16_F3                  u16_be=1369, u16_le=22789
0x00000600      1  FF                                                   TERMINATOR_FF             
0x00000601      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000603      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000605      8  8006816381638148                                     LEN8_STRING_CP932         length=6, text="……？"
0x0000060D      1  FF                                                   TERMINATOR_FF             
0x0000060E      1  FF                                                   TERMINATOR_FF             
0x0000060F      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000611      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000613      1  FF                                                   TERMINATOR_FF             
0x00000614      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000616      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000618      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x0000061A      1  FF                                                   TERMINATOR_FF             
0x0000061B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000061D      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x0000061F      3  F3055A                                               IMM16_F3                  u16_be=1370, u16_le=23045
0x00000622      1  FF                                                   TERMINATOR_FF             
0x00000623      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000625      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000627      9  800749573030313231                                   LEN8_STRING_CP932         length=7, text="IW00121"
0x00000630      1  FF                                                   TERMINATOR_FF             
0x00000631      1  FF                                                   TERMINATOR_FF             
0x00000632      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000634      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000636    120  80768175986593B982BE82AF82C782C8815B814182BD82C6...  LEN8_STRING_CP932         length=118, text="「脇道だけどなー、たとえばよーオーストラリアの先住民はだ、『夢の世界が現実で、目覚めてるときは幻だ』と思ってたしよー」"
0x000006AE      1  FF                                                   TERMINATOR_FF             
0x000006AF      1  FF                                                   TERMINATOR_FF             
0x000006B0      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000006B2      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000006B4      1  FF                                                   TERMINATOR_FF             
0x000006B5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006B7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006B9      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x000006BB      1  FF                                                   TERMINATOR_FF             
0x000006BC      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000006BE      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000006C0      3  F3055B                                               IMM16_F3                  u16_be=1371, u16_le=23301
0x000006C3      1  FF                                                   TERMINATOR_FF             
0x000006C4      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006C6      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000006C8      9  800749573030313232                                   LEN8_STRING_CP932         length=7, text="IW00122"
0x000006D1      1  FF                                                   TERMINATOR_FF             
0x000006D2      1  FF                                                   TERMINATOR_FF             
0x000006D3      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006D5      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000006D7     96  805E817589A295C482C582E0834A8381838982AA90B682DC...  LEN8_STRING_CP932         length=94, text="「欧米でもカメラが生まれたときにはー、目には見えない世界を写せる機械だって考えたわけ\nだしな」"
0x00000737      1  FF                                                   TERMINATOR_FF             
0x00000738      1  FF                                                   TERMINATOR_FF             
0x00000739      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000073B      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000073D      1  FF                                                   TERMINATOR_FF             
0x0000073E      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000740      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000742      2  F208                                                 IMM8_F2                   u8=8, s8=8
0x00000744      1  FF                                                   TERMINATOR_FF             
0x00000745      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000747      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000749      3  F3055C                                               IMM16_F3                  u16_be=1372, u16_le=23557
0x0000074C      1  FF                                                   TERMINATOR_FF             
0x0000074D      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000074F      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x00000751      9  800749573030313233                                   LEN8_STRING_CP932         length=7, text="IW00123"
0x0000075A      1  FF                                                   TERMINATOR_FF             
0x0000075B      1  FF                                                   TERMINATOR_FF             
0x0000075C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000075E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000760     82  8050817582C782B182A982C982B182B182B682E182C882A2...  LEN8_STRING_CP932         length=80, text="「どこかにここじゃない場所があるってーのは\n洋の東西を問わずに共通してたわけだ」"
0x000007B2      1  FF                                                   TERMINATOR_FF             
0x000007B3      1  FF                                                   TERMINATOR_FF             
0x000007B4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000007B6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000007B8      1  FF                                                   TERMINATOR_FF             
0x000007B9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007BB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007BD      2  F204                                                 IMM8_F2                   u8=4, s8=4
0x000007BF      1  FF                                                   TERMINATOR_FF             
0x000007C0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007C2      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000007C4      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000007C6      1  FF                                                   TERMINATOR_FF             
0x000007C7      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007C9      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000007CB      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000007CD      1  FF                                                   TERMINATOR_FF             
0x000007CE      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000007D0      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000007D2      1  FF                                                   TERMINATOR_FF             
0x000007D3      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007D5      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007D7      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x000007D9      1  FF                                                   TERMINATOR_FF             
0x000007DA      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007DC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000007DE      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000007E0      1  FF                                                   TERMINATOR_FF             
0x000007E1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007E3      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x000007E5      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x000007E7      1  FF                                                   TERMINATOR_FF             
0x000007E8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000007EA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000007EC      1  FF                                                   TERMINATOR_FF             
0x000007ED      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007EF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007F1      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000007F3      1  FF                                                   TERMINATOR_FF             
0x000007F4      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000007F6      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000007F8      3  F3055D                                               IMM16_F3                  u16_be=1373, u16_le=23813
0x000007FB      1  FF                                                   TERMINATOR_FF             
0x000007FC      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000007FE      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000800     66  80408343838F836D83728362836082CD8946928882C682A9...  LEN8_STRING_CP932         length=64, text="イワノビッチは宇宙とか深海のことを話し、\n古事記の表記にもどる。"
0x00000842      1  FF                                                   TERMINATOR_FF             
0x00000843      1  FF                                                   TERMINATOR_FF             
0x00000844      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000846      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000848      1  FF                                                   TERMINATOR_FF             
0x00000849      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000084B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000084D      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x0000084F      1  FF                                                   TERMINATOR_FF             
0x00000850      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000852      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000854      3  F3055E                                               IMM16_F3                  u16_be=1374, u16_le=24069
0x00000857      1  FF                                                   TERMINATOR_FF             
0x00000858      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000085A      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000085C     70  804482C582E0814182DA82AD82CC8EA882C982CD81778CBB...  LEN8_STRING_CP932         length=68, text="でも、ぼくの耳には『現実に重なる世界』ということばだけが残っていた。"
0x000008A2      1  FF                                                   TERMINATOR_FF             
0x000008A3      1  FF                                                   TERMINATOR_FF             
0x000008A4      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000008A6      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000008A8      1  FF                                                   TERMINATOR_FF             
0x000008A9      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008AB      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008AD      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x000008AF      1  FF                                                   TERMINATOR_FF             
0x000008B0      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008B2      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x000008B4      3  F3055F                                               IMM16_F3                  u16_be=1375, u16_le=24325
0x000008B7      1  FF                                                   TERMINATOR_FF             
0x000008B8      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000008BA      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000008BC     58  803882B182CC8FEA8F8A82C98F6482C882E982C782B182A9...  LEN8_STRING_CP932         length=56, text="この場所に重なるどこか。\nそんな場所があるとしたら……。"
0x000008F6      1  FF                                                   TERMINATOR_FF             
0x000008F7      1  FF                                                   TERMINATOR_FF             
0x000008F8      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000008FA      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000008FC      1  FF                                                   TERMINATOR_FF             
0x000008FD      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000008FF      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000901      2  F207                                                 IMM8_F2                   u8=7, s8=7
0x00000903      1  FF                                                   TERMINATOR_FF             
0x00000904      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000906      2  0007                                                 WORD_00XX                 u16_be=7, low_byte=7
0x00000908      3  F33F41                                               IMM16_F3                  u16_be=16193, u16_le=16703
0x0000090B      1  FF                                                   TERMINATOR_FF             
0x0000090C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000090E      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000910     60  803A82BB82EA82CD8163816394DE8F9782CC8FEA8F8A82C9...  LEN8_STRING_CP932         length=58, text="それは……彼女の場所につながる答えとなるの\nかもしれない。"
0x0000094C      1  FF                                                   TERMINATOR_FF             
0x0000094D      1  FF                                                   TERMINATOR_FF             
0x0000094E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000950      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000952      1  FF                                                   TERMINATOR_FF             
0x00000953      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000955      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000957      2  F217                                                 IMM8_F2                   u8=23, s8=23
0x00000959      1  FF                                                   TERMINATOR_FF             
0x0000095A      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000095C      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x0000095E      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000960      1  FF                                                   TERMINATOR_FF             
0x00000961      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000963      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000965      1  FF                                                   TERMINATOR_FF             
0x00000966      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000968      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000096A      2  F203                                                 IMM8_F2                   u8=3, s8=3
0x0000096C      1  FF                                                   TERMINATOR_FF             
0x0000096D      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000096F      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000971      2  F2FF                                                 IMM8_F2                   u8=255, s8=-1
0x00000973      1  FF                                                   TERMINATOR_FF             
0x00000974      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000976      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000978      1  FF                                                   TERMINATOR_FF             
0x00000979      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000097B      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x0000097D      2  F205                                                 IMM8_F2                   u8=5, s8=5
0x0000097F      1  FF                                                   TERMINATOR_FF             
0x00000980      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000982      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x00000984      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x00000986      1  FF                                                   TERMINATOR_FF             
0x00000987      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000989      2  0003                                                 WORD_00XX                 u16_be=3, low_byte=3
0x0000098B      2  F21E                                                 IMM8_F2                   u8=30, s8=30
0x0000098D      1  FF                                                   TERMINATOR_FF             
0x0000098E      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x00000990      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x00000992      1  FF                                                   TERMINATOR_FF             
0x00000993      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000995      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x00000997      2  F20B                                                 IMM8_F2                   u8=11, s8=11
0x00000999      1  FF                                                   TERMINATOR_FF             
0x0000099A      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x0000099C      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x0000099E      1  FF                                                   TERMINATOR_FF             
0x0000099F      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009A1      2  0001                                                 WORD_00XX                 u16_be=1, low_byte=1
0x000009A3      2  F202                                                 IMM8_F2                   u8=2, s8=2
0x000009A5      1  FF                                                   TERMINATOR_FF             
0x000009A6      2  0002                                                 WORD_00XX                 u16_be=2, low_byte=2
0x000009A8      2  0000                                                 WORD_00XX                 u16_be=0, low_byte=0
0x000009AA     14  800C50533241303237612E62696E                         LEN8_STRING_CP932         length=12, text="PS2A027a.bin"
0x000009B8      1  FF                                                   TERMINATOR_FF             
0x000009B9      1  FF                                                   TERMINATOR_FF             
0x000009BA      2  0016                                                 WORD_00XX                 u16_be=22, low_byte=22
0x000009BC      2  F200                                                 IMM8_F2                   u8=0, s8=0
0x000009BE      1  FF                                                   TERMINATOR_FF             
0x000009BF      2  001C                                                 WORD_00XX                 u16_be=28, low_byte=28
0x000009C1      2  F201                                                 IMM8_F2                   u8=1, s8=1
0x000009C3      1  FF                                                   TERMINATOR_FF             
0x000009C4      2  0009                                                 WORD_00XX                 u16_be=9, low_byte=9
0x000009C6      1  CE                                                   OPAQUE_RAW_BYTES          bytes=CE
0x000009C7      2  0021                                                 WORD_00XX                 u16_be=33, low_byte=33
0x000009C9      2  0013                                                 WORD_00XX                 u16_be=19, low_byte=19
0x000009CB      2  0009                                                 WORD_00XX                 u16_be=9, low_byte=9
0x000009CD      1  BF                                                   OPAQUE_RAW_BYTES          bytes=BF
0x000009CE      1  FF                                                   TERMINATOR_FF             
0x000009CF      1  FF                                                   TERMINATOR_FF             
