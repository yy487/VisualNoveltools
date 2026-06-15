#!/usr/bin/env python3
"""
ACTGS 引擎 .scr 脚本文本提取工具
从 arc.scr 档案中提取可翻译文本为 GalTransl 兼容 JSON

用法: python scr_extract.py <ACTGS.exe> <arc.scr> [输出目录]
"""

import json
import os
import sys
import re

from scr_crypto import auto_find_key, parse_archive


# ============================================================
# 常量
# ============================================================
PURE_COMMANDS = {
    'N', 'ret',
    'bgm1', 'bgm_fo',
    'bg', 'bg1_fi',
    'sp1', 'sp1_cf', 'sp2_cf', 'sp3', 'sp3_cf', 'sp_fo',
    'ev', 'ev1_cf', 'ev1_fi', 'ev1',
    'fi', 'fo',
    'change', 'change_kaisou',
    'define', 'def_cg', 'def_indent', 'def_kinsoku',
    'def_sp_left', 'def_sp_center', 'def_sp_right',
    'select2', 'select_center_on',
    'def_selmes2',
    'flag_update', 'param_calc',
    'map', 'menu', 'movie', 'title',
    'move_xyposi', 'posi',
    'shake', 'sleep', 'wait',
    'window', 'window_off', 'window_on', 'window_sel',
    'kaisou_end', 'auto_ret_off',
    # 扩展: 音频/视觉/时序/精灵控制
    'vo_wait', 'vo_sel', 'bgm_stop', 'bgm_wait', 'bgm2',
    'se1', 'se2', 'se3', 'se_bgm1', 'se_bgm2', 'se_fo', 'se_stop', 'se_wait', 'sex',
    'flash1', 'flash2', 'shake2',
    'moji', 'pan',
    'move_xyset', 'move_xywait',
    'sp1_cf', 'sp2_cf', 'sp_fade_in', 'sp_loop',
    'def_sp_left3', 'def_sp_right3',
    'exchg',
    # 精灵显示/控制
    'sp', 'sp2', 'sp1_', 'sp2_', 'sp3_', 'spx_', 'sp2_fi',
    # 效果/转场
    'set_effect', 'zoom', 'bg1_', 'ev1_', 'fade_sp4', 'trans_sp4',
}

# 对话触发指令: 这些指令后跟随文本行
VOICE_TRIGGERS = {'vo', 'vo2', 'vox'}

# 消息块内可安全跳过的控制指令 (不中断文本收集)
# 这些是视觉效果/音频/时序指令, 常出现在同一对话块的文本行之间
SKIP_IN_MSG = {
    # 音频
    'vo_wait', 'bgm_stop', 'bgm_wait', 'se_stop', 'se_wait',
    'se_fo', 'bgm_fo', 'se1', 'se2', 'se3', 'sex', 'se_bgm2',
    # 视觉特效
    'flash1', 'flash2', 'shake', 'shake2',
    # 时序
    'wait', 'moji',
    # 镜头/位置
    'pan', 'move_xyset', 'move_xywait', 'posi',
    # 精灵动画/渐变 (同一角色表情变化)
    'sp1_cf', 'sp2_cf', 'sp3_cf', 'sp_fade_in', 'sp_loop', 'sp2_fi',
    # 精灵位置定义
    'def_sp_left', 'def_sp_center', 'def_sp_right',
    'def_sp_left3', 'def_sp_right3',
    # 精灵显示/交换 (非入场/退场)
    'sp', 'sp1', 'sp2', 'sp3', 'sp1_', 'sp2_', 'sp3_', 'spx_', 'exchg',
    # 效果
    'set_effect', 'zoom',
}

# 可含日文字符串参数的命令 (参数以引号包裹)
TEXT_ARG_COMMANDS = {'ev1_fi', 'ev1', 'ev', 'ev1_cf'}

SEL_PREFIX_RE = re.compile(r'^[１２３４５６７８９０]+[．.]')
INLINE_NAME_RE = re.compile(r'^([^\x00-\x7F]+?)\s*(「.+)', re.DOTALL)


# ============================================================
# 文本判断
# ============================================================
def has_japanese(s):
    for ch in s:
        cp = ord(ch)
        if (0x3040 <= cp <= 0x309F or 0x30A0 <= cp <= 0x30FF or
            0x4E00 <= cp <= 0x9FFF or 0xFF00 <= cp <= 0xFFEF or
            0x3000 <= cp <= 0x303F):
            return True
        # CJK 标点: 省略号 (…), 破折号等
        if cp in (0x2026, 0x2014, 0x2015):
            return True
    return False


def is_pure_command(line):
    s = line.strip()
    if not s or s.startswith(';') or s.startswith('[') or s in ('{', '}'):
        return True
    if re.match(r'^F\d+', s):
        return True
    cmd = s.split(None, 1)[0]
    if cmd in PURE_COMMANDS or cmd in ('vo', 'vo2', 'vox', 'msg2'):
        return True
    return False


def strip_sel_prefix(text):
    m = SEL_PREFIX_RE.match(text)
    if m:
        return m.group(0), text[m.end():]
    return "", text


# ============================================================
# 文本提取
# ============================================================
def extract_text(name, scr_bytes):
    text = scr_bytes.decode('cp932', errors='replace')
    lines = text.split('\r\n')
    entries = []
    basename = name.replace('.scr', '')
    text_id = 0
    i = 0

    while i < len(lines):
        stripped = lines[i].strip()

        if stripped.startswith('def_sel '):
            sel_text = stripped[8:]
            if has_japanese(sel_text):
                _, body = strip_sel_prefix(sel_text)
                entries.append({
                    "name": "",
                    "message": body,
                    "id": f"{basename}/{text_id}/sel"
                })
                text_id += 1
            i += 1
            continue

        if stripped.startswith('def_selmes ') and not stripped.startswith('def_selmes2'):
            sel_text = stripped[11:]
            if has_japanese(sel_text):
                entries.append({
                    "name": "",
                    "message": sel_text,
                    "id": f"{basename}/{text_id}/selmes"
                })
                text_id += 1
            i += 1
            continue

        # ── 对话块: vo / vo2 / vox ──
        voice_match = None
        for vt in VOICE_TRIGGERS:
            if stripped.startswith(vt + ' ') or stripped == vt:
                voice_match = vt
                break

        if voice_match:
            i += 1
            speaker = ""
            msg_lines = []

            while i < len(lines):
                cur = lines[i].strip()
                if not cur:
                    i += 1; continue
                # 注释行: 消息块内跳过, 不中断收集
                if cur.startswith(';'):
                    i += 1; continue
                if cur.startswith('msg2 '):
                    speaker = cur[5:].strip(); i += 1; continue
                # 嵌套的 vo/vo2/vox → 中断当前块
                if any(cur.startswith(vt + ' ') or cur == vt for vt in VOICE_TRIGGERS):
                    break
                if (cur.startswith('def_sel ') or
                    (cur.startswith('def_selmes ') and not cur.startswith('def_selmes2'))):
                    break
                if cur == 'ret':
                    i += 1; break
                if is_pure_command(cur):
                    # 消息块内可安全跳过的控制指令
                    cmd = cur.split(None, 1)[0] if cur.split(None, 1) else cur
                    if cmd in SKIP_IN_MSG:
                        i += 1; continue
                    break
                if has_japanese(cur) or cur.startswith('「'):
                    msg_lines.append(cur); i += 1; continue
                # 检查是否为含日文参数的指令 (如 ev1_fi "_文本_")
                cmd = cur.split(None, 1)[0] if cur.split(None, 1) else ''
                if cmd in TEXT_ARG_COMMANDS:
                    m = re.search(r'\"([^\"]*[぀-ゟ゠-ヿ一-鿿][^\"]*)\"', cur)
                    if m:
                        msg_lines.append(m.group(1)); i += 1; continue
                i += 1; break

            if msg_lines:
                message = '\\n'.join(msg_lines)
                if not speaker and len(msg_lines) == 1:
                    m = INLINE_NAME_RE.match(msg_lines[0])
                    if m:
                        speaker = m.group(1)
                        message = msg_lines[0][m.start(2):]
                # 若有 msg2 已设说话人, 且首文本行以该名字开头 (如 "稔「..."),
                # 则剥离行内冗余名字, 避免译文中出现重复说话人
                if speaker and msg_lines:
                    m = INLINE_NAME_RE.match(msg_lines[0])
                    if m and m.group(1) == speaker:
                        msg_lines[0] = msg_lines[0][m.start(2):]
                        message = '\\n'.join(msg_lines)
                entries.append({
                    "name": speaker,
                    "message": message,
                    "id": f"{basename}/{text_id}"
                })
                text_id += 1
            continue

        # ── 独立含日文参数指令 (非对话块内) ──
        cmd = stripped.split(None, 1)[0] if stripped.split(None, 1) else ''
        if cmd in TEXT_ARG_COMMANDS:
            m = re.search(r'\"([^\"]*[぀-ゟ゠-ヿ一-鿿][^\"]*)\"', stripped)
            if m:
                entries.append({
                    "name": "",
                    "message": m.group(1),
                    "id": f"{basename}/{text_id}/cmd_{cmd}"
                })
                text_id += 1

        i += 1

    return entries


# ============================================================
# 主程序
# ============================================================
def main():
    if len(sys.argv) < 3:
        print(f"用法: {sys.argv[0]} <ACTGS.exe> <arc.scr> [输出目录]")
        sys.exit(1)

    exe_path = sys.argv[1]
    arc_path = sys.argv[2]
    out_dir  = sys.argv[3] if len(sys.argv) > 3 else 'scr_json'

    print(f"搜索密钥: {exe_path}")
    key = auto_find_key(exe_path)
    if not key:
        print("错误: 未能从 EXE 中找到 XOR 密钥")
        sys.exit(1)
    print(f"密钥: {key.hex()} (长度 {len(key)})")

    os.makedirs(out_dir, exist_ok=True)
    print(f"解析档案: {arc_path}")
    scripts, _header, _enc, _gaps, _trailing = parse_archive(arc_path, key)
    print(f"脚本数量: {len(scripts)}")

    total_entries = 0
    total_files = 0
    for name, scr in scripts:
        entries = extract_text(name, scr)
        if entries:
            json_path = os.path.join(out_dir, name.replace('.scr', '.json'))
            with open(json_path, 'w', encoding='utf-8') as f:
                json.dump(entries, f, ensure_ascii=False, indent=2)
            total_entries += len(entries)
            total_files += 1
            print(f"  {name}: {len(entries)} 条")

    print(f"\n完成! 共 {total_files} 个文件, {total_entries} 条文本")
    print(f"输出目录: {out_dir}")


if __name__ == '__main__':
    main()
