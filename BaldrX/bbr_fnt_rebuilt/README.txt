BBR FNT rebuilt package

Files:
  FNT10X10.FNT
  FNT12X12.FNT
  FNT14X14.FNT
  hgo10x10.fnt
  hgo12x12.fnt
  hgo14x14.fnt

Method:
  1. Decode BBR FNT LZSS bitmap font.
  2. Locate each target CP932 full-width character slot according to the game's hard-coded SJIS ranges.
  3. Render the source Chinese character into that target slot.
  4. Rebuild FNT using the same container header and safe literal compression.

Note:
  Mapping target 凜 was automatically changed to 凛 because 凜 = CP932 EAA3 is outside the game's drawable range ending at EAA2.

Command used in this container:
  python3 bbr_rebuild_fnt.py /mnt/data /mnt/data/subs_cn_jp.json /mnt/data/bbr_fnt_rebuilt --ttf /usr/share/fonts/truetype/arphic/uming.ttc --size 14 --threshold 80

For Windows, use for example:
  python bbr_rebuild_fnt.py . subs_cn_jp.json font_out --ttf C:\Windows\Fonts\simsun.ttc --size 14 --threshold 80
