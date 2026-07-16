#![allow(non_snake_case)]

use std::collections::HashSet;
use std::env;
use std::ffi::c_void;
use std::fs::{self, File};
use std::io::{self, Write};
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::ptr;

const FONT_FILE_BYTES: usize = 2_544_768;
const GLYPH_COUNT: usize = 94 * 94;
const GLYPH_WIDTH: usize = 24;
const GLYPH_HEIGHT: usize = 24;
const GLYPH_BYTES: usize = GLYPH_WIDTH * GLYPH_HEIGHT / 2;
const EXPECTED_MAP_COUNT: usize = 3_018;
const FONT_PIXEL_HEIGHT: i32 = 24;
const RASTER_WIDTH: usize = 64;
const RASTER_HEIGHT: usize = 64;
const RASTER_ORIGIN_X: i32 = 16;
const RASTER_ORIGIN_Y: i32 = 16;

#[derive(Clone, Copy)]
struct MapEntry {
    target: char,
    source: char,
    glyph_index: usize,
}

const CN_JP_MAP: &[MapEntry] = &[
    MapEntry {
        target: '这',
        source: '這',
        glyph_index: 3642,
    },
    MapEntry {
        target: '说',
        source: '説',
        glyph_index: 2979,
    },
    MapEntry {
        target: '为',
        source: '為',
        glyph_index: 1466,
    },
    MapEntry {
        target: '你',
        source: '凜',
        glyph_index: 7806,
    },
    MapEntry {
        target: '们',
        source: '們',
        glyph_index: 4495,
    },
    MapEntry {
        target: '时',
        source: '時',
        glyph_index: 2537,
    },
    MapEntry {
        target: '过',
        source: '過',
        glyph_index: 1662,
    },
    MapEntry {
        target: '对',
        source: '対',
        glyph_index: 3149,
    },
    MapEntry {
        target: '么',
        source: '麼',
        glyph_index: 5068,
    },
    MapEntry {
        target: '她',
        source: '瑤',
        glyph_index: 7805,
    },
    MapEntry {
        target: '还',
        source: '還',
        glyph_index: 1837,
    },
    MapEntry {
        target: '现',
        source: '現',
        glyph_index: 2190,
    },
    MapEntry {
        target: '发',
        source: '髪',
        glyph_index: 3682,
    },
    MapEntry {
        target: '种',
        source: '種',
        glyph_index: 2616,
    },
    MapEntry {
        target: '样',
        source: '樣',
        glyph_index: 5620,
    },
    MapEntry {
        target: '见',
        source: '見',
        glyph_index: 2172,
    },
    MapEntry {
        target: '经',
        source: '経',
        glyph_index: 2115,
    },
    MapEntry {
        target: '头',
        source: '頭',
        glyph_index: 3489,
    },
    MapEntry {
        target: '书',
        source: '書',
        glyph_index: 2712,
    },
    MapEntry {
        target: '实',
        source: '実',
        glyph_index: 2571,
    },
    MapEntry {
        target: '开',
        source: '開',
        glyph_index: 1702,
    },
    MapEntry {
        target: '两',
        source: '両',
        glyph_index: 4259,
    },
    MapEntry {
        target: '动',
        source: '動',
        glyph_index: 3493,
    },
    MapEntry {
        target: '话',
        source: '話',
        glyph_index: 4358,
    },
    MapEntry {
        target: '问',
        source: '問',
        glyph_index: 4109,
    },
    MapEntry {
        target: '间',
        source: '間',
        glyph_index: 1839,
    },
    MapEntry {
        target: '论',
        source: '論',
        glyph_index: 4355,
    },
    MapEntry {
        target: '长',
        source: '長',
        glyph_index: 3314,
    },
    MapEntry {
        target: '进',
        source: '進',
        glyph_index: 2861,
    },
    MapEntry {
        target: '义',
        source: '義',
        glyph_index: 1912,
    },
    MapEntry {
        target: '给',
        source: '給',
        glyph_index: 1954,
    },
    MapEntry {
        target: '关',
        source: '関',
        glyph_index: 1841,
    },
    MapEntry {
        target: '别',
        source: '別',
        glyph_index: 3897,
    },
    MapEntry {
        target: '门',
        source: '門',
        glyph_index: 4112,
    },
    MapEntry {
        target: '觉',
        source: '覚',
        glyph_index: 1739,
    },
    MapEntry {
        target: '复',
        source: '復',
        glyph_index: 3851,
    },
    MapEntry {
        target: '难',
        source: '難',
        glyph_index: 3558,
    },
    MapEntry {
        target: '东',
        source: '東',
        glyph_index: 3459,
    },
    MapEntry {
        target: '应',
        source: '応',
        glyph_index: 1597,
    },
    MapEntry {
        target: '风',
        source: '風',
        glyph_index: 3846,
    },
    MapEntry {
        target: '张',
        source: '張',
        glyph_index: 3294,
    },
    MapEntry {
        target: '认',
        source: '認',
        glyph_index: 3578,
    },
    MapEntry {
        target: '传',
        source: '伝',
        glyph_index: 3416,
    },
    MapEntry {
        target: '题',
        source: '題',
        glyph_index: 3175,
    },
    MapEntry {
        target: '处',
        source: '処',
        glyph_index: 2703,
    },
    MapEntry {
        target: '马',
        source: '馬',
        glyph_index: 3618,
    },
    MapEntry {
        target: '诗',
        source: '詩',
        glyph_index: 2520,
    },
    MapEntry {
        target: '让',
        source: '讓',
        glyph_index: 7059,
    },
    MapEntry {
        target: '爱',
        source: '愛',
        glyph_index: 1415,
    },
    MapEntry {
        target: '师',
        source: '師',
        glyph_index: 2496,
    },
    MapEntry {
        target: '语',
        source: '語',
        glyph_index: 2237,
    },
    MapEntry {
        target: '变',
        source: '変',
        glyph_index: 3902,
    },
    MapEntry {
        target: '尔',
        source: '爾',
        glyph_index: 2541,
    },
    MapEntry {
        target: '许',
        source: '許',
        glyph_index: 1965,
    },
    MapEntry {
        target: '结',
        source: '結',
        glyph_index: 2142,
    },
    MapEntry {
        target: '边',
        source: '邊',
        glyph_index: 7257,
    },
    MapEntry {
        target: '历',
        source: '歴',
        glyph_index: 4311,
    },
    MapEntry {
        target: '记',
        source: '記',
        glyph_index: 1892,
    },
    MapEntry {
        target: '读',
        source: '読',
        glyph_index: 3518,
    },
    MapEntry {
        target: '识',
        source: '識',
        glyph_index: 2554,
    },
    MapEntry {
        target: '战',
        source: '戰',
        glyph_index: 5268,
    },
    MapEntry {
        target: '德',
        source: '徳',
        glyph_index: 3510,
    },
    MapEntry {
        target: '术',
        source: '術',
        glyph_index: 2680,
    },
    MapEntry {
        target: '远',
        source: '遠',
        glyph_index: 1586,
    },
    MapEntry {
        target: '总',
        source: '総',
        glyph_index: 3084,
    },
    MapEntry {
        target: '观',
        source: '観',
        glyph_index: 1834,
    },
    MapEntry {
        target: '呢',
        source: '遙',
        glyph_index: 7804,
    },
    MapEntry {
        target: '吧',
        source: '槇',
        glyph_index: 7803,
    },
    MapEntry {
        target: '连',
        source: '連',
        glyph_index: 4325,
    },
    MapEntry {
        target: '强',
        source: '強',
        glyph_index: 1988,
    },
    MapEntry {
        target: '则',
        source: '則',
        glyph_index: 3108,
    },
    MapEntry {
        target: '该',
        source: '該',
        glyph_index: 1717,
    },
    MapEntry {
        target: '啊',
        source: '堯',
        glyph_index: 7802,
    },
    MapEntry {
        target: '场',
        source: '場',
        glyph_index: 2801,
    },
    MapEntry {
        target: '轻',
        source: '軽',
        glyph_index: 2125,
    },
    MapEntry {
        target: '亲',
        source: '親',
        glyph_index: 2857,
    },
    MapEntry {
        target: '统',
        source: '統',
        glyph_index: 3476,
    },
    MapEntry {
        target: '吗',
        source: '龜',
        glyph_index: 7800,
    },
    MapEntry {
        target: '带',
        source: '帯',
        glyph_index: 3152,
    },
    MapEntry {
        target: '业',
        source: '業',
        glyph_index: 2013,
    },
    MapEntry {
        target: '产',
        source: '産',
        glyph_index: 2469,
    },
    MapEntry {
        target: '脸',
        source: '臉',
        glyph_index: 6611,
    },
    MapEntry {
        target: '虽',
        source: '雖',
        glyph_index: 6874,
    },
    MapEntry {
        target: '军',
        source: '軍',
        glyph_index: 2086,
    },
    MapEntry {
        target: '极',
        source: '極',
        glyph_index: 2016,
    },
    MapEntry {
        target: '类',
        source: '類',
        glyph_index: 4293,
    },
    MapEntry {
        target: '满',
        source: '満',
        glyph_index: 4041,
    },
    MapEntry {
        target: '转',
        source: '転',
        glyph_index: 3413,
    },
    MapEntry {
        target: '步',
        source: '歩',
        glyph_index: 3919,
    },
    MapEntry {
        target: '剑',
        source: '剣',
        glyph_index: 2152,
    },
    MapEntry {
        target: '黑',
        source: '黒',
        glyph_index: 2340,
    },
    MapEntry {
        target: '离',
        source: '離',
        glyph_index: 4234,
    },
    MapEntry {
        target: '电',
        source: '電',
        glyph_index: 3420,
    },
    MapEntry {
        target: '每',
        source: '毎',
        glyph_index: 4019,
    },
    MapEntry {
        target: '飞',
        source: '飛',
        glyph_index: 3749,
    },
    MapEntry {
        target: '济',
        source: '済',
        glyph_index: 2398,
    },
    MapEntry {
        target: '艺',
        source: '藝',
        glyph_index: 6793,
    },
    MapEntry {
        target: '众',
        source: '衆',
        glyph_index: 2647,
    },
    MapEntry {
        target: '杀',
        source: '殺',
        glyph_index: 2449,
    },
    MapEntry {
        target: '报',
        source: '報',
        glyph_index: 3936,
    },
    MapEntry {
        target: '车',
        source: '車',
        glyph_index: 2591,
    },
    MapEntry {
        target: '终',
        source: '終',
        glyph_index: 2641,
    },
    MapEntry {
        target: '红',
        source: '紅',
        glyph_index: 2295,
    },
    MapEntry {
        target: '视',
        source: '視',
        glyph_index: 2518,
    },
    MapEntry {
        target: '谁',
        source: '誰',
        glyph_index: 3210,
    },
    MapEntry {
        target: '绝',
        source: '絶',
        glyph_index: 2981,
    },
    MapEntry {
        target: '确',
        source: '確',
        glyph_index: 1737,
    },
    MapEntry {
        target: '达',
        source: '達',
        glyph_index: 3198,
    },
    MapEntry {
        target: '惊',
        source: '驚',
        glyph_index: 2008,
    },
    MapEntry {
        target: '罗',
        source: '羅',
        glyph_index: 4204,
    },
    MapEntry {
        target: '乐',
        source: '楽',
        glyph_index: 1749,
    },
    MapEntry {
        target: '资',
        source: '資',
        glyph_index: 2524,
    },
    MapEntry {
        target: '运',
        source: '運',
        glyph_index: 1534,
    },
    MapEntry {
        target: '华',
        source: '華',
        glyph_index: 1655,
    },
    MapEntry {
        target: '伤',
        source: '傷',
        glyph_index: 2724,
    },
    MapEntry {
        target: '评',
        source: '評',
        glyph_index: 3789,
    },
    MapEntry {
        target: '响',
        source: '響',
        glyph_index: 2006,
    },
    MapEntry {
        target: '刚',
        source: '剛',
        glyph_index: 2323,
    },
    MapEntry {
        target: '谈',
        source: '談',
        glyph_index: 3239,
    },
    MapEntry {
        target: '领',
        source: '領',
        glyph_index: 4275,
    },
    MapEntry {
        target: '权',
        source: '権',
        glyph_index: 2163,
    },
    MapEntry {
        target: '晚',
        source: '晩',
        glyph_index: 3718,
    },
    MapEntry {
        target: '兴',
        source: '興',
        glyph_index: 2002,
    },
    MapEntry {
        target: '哪',
        source: '齧',
        glyph_index: 7793,
    },
    MapEntry {
        target: '显',
        source: '顕',
        glyph_index: 2179,
    },
    MapEntry {
        target: '图',
        source: '圖',
        glyph_index: 4799,
    },
    MapEntry {
        target: '钱',
        source: '銭',
        glyph_index: 3019,
    },
    MapEntry {
        target: '杨',
        source: '楊',
        glyph_index: 4179,
    },
    MapEntry {
        target: '态',
        source: '態',
        glyph_index: 3155,
    },
    MapEntry {
        target: '灵',
        source: '霊',
        glyph_index: 4307,
    },
    MapEntry {
        target: '译',
        source: '訳',
        glyph_index: 4126,
    },
    MapEntry {
        target: '另',
        source: '鄰',
        glyph_index: 7272,
    },
    MapEntry {
        target: '欢',
        source: '歓',
        glyph_index: 1816,
    },
    MapEntry {
        target: '错',
        source: '錯',
        glyph_index: 2437,
    },
    MapEntry {
        target: '联',
        source: '聯',
        glyph_index: 4323,
    },
    MapEntry {
        target: '计',
        source: '計',
        glyph_index: 2122,
    },
    MapEntry {
        target: '纪',
        source: '紀',
        glyph_index: 1889,
    },
    MapEntry {
        target: '请',
        source: '請',
        glyph_index: 2946,
    },
    MapEntry {
        target: '创',
        source: '創',
        glyph_index: 3054,
    },
    MapEntry {
        target: '阳',
        source: '陽',
        glyph_index: 4194,
    },
    MapEntry {
        target: '选',
        source: '選',
        glyph_index: 3017,
    },
    MapEntry {
        target: '紧',
        source: '緊',
        glyph_index: 2032,
    },
    MapEntry {
        target: '单',
        source: '単',
        glyph_index: 3212,
    },
    MapEntry {
        target: '词',
        source: '詞',
        glyph_index: 2519,
    },
    MapEntry {
        target: '调',
        source: '調',
        glyph_index: 3309,
    },
    MapEntry {
        target: '编',
        source: '編',
        glyph_index: 3905,
    },
    MapEntry {
        target: '异',
        source: '異',
        glyph_index: 1468,
    },
    MapEntry {
        target: '员',
        source: '員',
        glyph_index: 1496,
    },
    MapEntry {
        target: '仅',
        source: '僅',
        glyph_index: 2020,
    },
    MapEntry {
        target: '势',
        source: '勢',
        glyph_index: 2923,
    },
    MapEntry {
        target: '讲',
        source: '講',
        glyph_index: 2309,
    },
    MapEntry {
        target: '办',
        source: '齡',
        glyph_index: 7791,
    },
    MapEntry {
        target: '汉',
        source: '漢',
        glyph_index: 1818,
    },
    MapEntry {
        target: '证',
        source: '証',
        glyph_index: 2783,
    },
    MapEntry {
        target: '较',
        source: '較',
        glyph_index: 1742,
    },
    MapEntry {
        target: '击',
        source: '撃',
        glyph_index: 2133,
    },
    MapEntry {
        target: '严',
        source: '厳',
        glyph_index: 2184,
    },
    MapEntry {
        target: '龙',
        source: '竜',
        glyph_index: 4250,
    },
    MapEntry {
        target: '约',
        source: '約',
        glyph_index: 4124,
    },
    MapEntry {
        target: '兰',
        source: '蘭',
        glyph_index: 4221,
    },
    MapEntry {
        target: '质',
        source: '質',
        glyph_index: 2570,
    },
    MapEntry {
        target: '够',
        source: '鉤',
        glyph_index: 7313,
    },
    MapEntry {
        target: '帮',
        source: '幇',
        glyph_index: 5064,
    },
    MapEntry {
        target: '陆',
        source: '陸',
        glyph_index: 4235,
    },
    MapEntry {
        target: '铁',
        source: '鉄',
        glyph_index: 3403,
    },
    MapEntry {
        target: '节',
        source: '節',
        glyph_index: 2978,
    },
    MapEntry {
        target: '细',
        source: '細',
        glyph_index: 2406,
    },
    MapEntry {
        target: '笔',
        source: '筆',
        glyph_index: 3773,
    },
    MapEntry {
        target: '值',
        source: '値',
        glyph_index: 3240,
    },
    MapEntry {
        target: '陈',
        source: '陳',
        glyph_index: 3325,
    },
    MapEntry {
        target: '热',
        source: '熱',
        glyph_index: 3585,
    },
    MapEntry {
        target: '苏',
        source: '蘇',
        glyph_index: 3048,
    },
    MapEntry {
        target: '沉',
        source: '齣',
        glyph_index: 7788,
    },
    MapEntry {
        target: '怀',
        source: '懐',
        glyph_index: 1688,
    },
    MapEntry {
        target: '妈',
        source: '媽',
        glyph_index: 4919,
    },
    MapEntry {
        target: '际',
        source: '際',
        glyph_index: 2410,
    },
    MapEntry {
        target: '级',
        source: '級',
        glyph_index: 1952,
    },
    MapEntry {
        target: '恶',
        source: '悪',
        glyph_index: 1422,
    },
    MapEntry {
        target: '须',
        source: '須',
        glyph_index: 2879,
    },
    MapEntry {
        target: '维',
        source: '維',
        glyph_index: 1470,
    },
    MapEntry {
        target: '举',
        source: '挙',
        glyph_index: 1962,
    },
    MapEntry {
        target: '专',
        source: '専',
        glyph_index: 2989,
    },
    MapEntry {
        target: '导',
        source: '導',
        glyph_index: 3496,
    },
    MapEntry {
        target: '备',
        source: '備',
        glyph_index: 3752,
    },
    MapEntry {
        target: '务',
        source: '務',
        glyph_index: 4060,
    },
    MapEntry {
        target: '简',
        source: '簡',
        glyph_index: 1827,
    },
    MapEntry {
        target: '费',
        source: '費',
        glyph_index: 3746,
    },
    MapEntry {
        target: '脑',
        source: '脳',
        glyph_index: 3601,
    },
    MapEntry {
        target: '杂',
        source: '雑',
        glyph_index: 2451,
    },
    MapEntry {
        target: '亚',
        source: '亜',
        glyph_index: 1410,
    },
    MapEntry {
        target: '顾',
        source: '顧',
        glyph_index: 2221,
    },
    MapEntry {
        target: '岁',
        source: '歳',
        glyph_index: 2397,
    },
    MapEntry {
        target: '构',
        source: '構',
        glyph_index: 2284,
    },
    MapEntry {
        target: '队',
        source: '隊',
        glyph_index: 3167,
    },
    MapEntry {
        target: '鲁',
        source: '魯',
        glyph_index: 4328,
    },
    MapEntry {
        target: '续',
        source: '続',
        glyph_index: 3120,
    },
    MapEntry {
        target: '剧',
        source: '劇',
        glyph_index: 2131,
    },
    MapEntry {
        target: '闻',
        source: '聞',
        glyph_index: 3878,
    },
    MapEntry {
        target: '爷',
        source: '爺',
        glyph_index: 4117,
    },
    MapEntry {
        target: '敌',
        source: '敵',
        glyph_index: 3391,
    },
    MapEntry {
        target: '规',
        source: '規',
        glyph_index: 1891,
    },
    MapEntry {
        target: '阵',
        source: '陣',
        glyph_index: 2875,
    },
    MapEntry {
        target: '习',
        source: '習',
        glyph_index: 2643,
    },
    MapEntry {
        target: '线',
        source: '線',
        glyph_index: 3007,
    },
    MapEntry {
        target: '树',
        source: '樹',
        glyph_index: 2626,
    },
    MapEntry {
        target: '阶',
        source: '階',
        glyph_index: 1703,
    },
    MapEntry {
        target: '诉',
        source: '訴',
        glyph_index: 3049,
    },
    MapEntry {
        target: '设',
        source: '設',
        glyph_index: 2976,
    },
    MapEntry {
        target: '谓',
        source: '謂',
        glyph_index: 1475,
    },
    MapEntry {
        target: '围',
        source: '囲',
        glyph_index: 1456,
    },
    MapEntry {
        target: '议',
        source: '議',
        glyph_index: 1915,
    },
    MapEntry {
        target: '谢',
        source: '謝',
        glyph_index: 2590,
    },
    MapEntry {
        target: '录',
        source: '録',
        glyph_index: 4354,
    },
    MapEntry {
        target: '层',
        source: '層',
        glyph_index: 3063,
    },
    MapEntry {
        target: '标',
        source: '標',
        glyph_index: 3783,
    },
    MapEntry {
        target: '摇',
        source: '揺',
        glyph_index: 4176,
    },
    MapEntry {
        target: '归',
        source: '帰',
        glyph_index: 1881,
    },
    MapEntry {
        target: '刘',
        source: '劉',
        glyph_index: 4242,
    },
    MapEntry {
        target: '齐',
        source: '斉',
        glyph_index: 2951,
    },
    MapEntry {
        target: '戏',
        source: '戯',
        glyph_index: 1905,
    },
    MapEntry {
        target: '隐',
        source: '隠',
        glyph_index: 1506,
    },
    MapEntry {
        target: '跑',
        source: '鉋',
        glyph_index: 7317,
    },
    MapEntry {
        target: '寻',
        source: '尋',
        glyph_index: 2869,
    },
    MapEntry {
        target: '查',
        source: '査',
        glyph_index: 2375,
    },
    MapEntry {
        target: '试',
        source: '試',
        glyph_index: 2521,
    },
    MapEntry {
        target: '饭',
        source: '飯',
        glyph_index: 3716,
    },
    MapEntry {
        target: '渐',
        source: '漸',
        glyph_index: 3025,
    },
    MapEntry {
        target: '护',
        source: '護',
        glyph_index: 2239,
    },
    MapEntry {
        target: '组',
        source: '組',
        glyph_index: 3047,
    },
    MapEntry {
        target: '胜',
        source: '勝',
        glyph_index: 2726,
    },
    MapEntry {
        target: '瞧',
        source: '喬',
        glyph_index: 1985,
    },
    MapEntry {
        target: '释',
        source: '釈',
        glyph_index: 2602,
    },
    MapEntry {
        target: '讨',
        source: '討',
        glyph_index: 3481,
    },
    MapEntry {
        target: '验',
        source: '験',
        glyph_index: 2180,
    },
    MapEntry {
        target: '买',
        source: '買',
        glyph_index: 3638,
    },
    MapEntry {
        target: '贵',
        source: '貴',
        glyph_index: 1893,
    },
    MapEntry {
        target: '继',
        source: '継',
        glyph_index: 2116,
    },
    MapEntry {
        target: '团',
        source: '団',
        glyph_index: 3231,
    },
    MapEntry {
        target: '适',
        source: '適',
        glyph_index: 3395,
    },
    MapEntry {
        target: '农',
        source: '農',
        glyph_index: 3603,
    },
    MapEntry {
        target: '叹',
        source: '嘆',
        glyph_index: 3213,
    },
    MapEntry {
        target: '卫',
        source: '衛',
        glyph_index: 1553,
    },
    MapEntry {
        target: '丽',
        source: '麗',
        glyph_index: 4308,
    },
    MapEntry {
        target: '块',
        source: '塊',
        glyph_index: 1681,
    },
    MapEntry {
        target: '宫',
        source: '宮',
        glyph_index: 1939,
    },
    MapEntry {
        target: '环',
        source: '環',
        glyph_index: 1821,
    },
    MapEntry {
        target: '份',
        source: '齒',
        glyph_index: 7786,
    },
    MapEntry {
        target: '换',
        source: '換',
        glyph_index: 1810,
    },
    MapEntry {
        target: '伦',
        source: '倫',
        glyph_index: 4278,
    },
    MapEntry {
        target: '败',
        source: '敗',
        glyph_index: 3623,
    },
    MapEntry {
        target: '压',
        source: '圧',
        glyph_index: 1430,
    },
    MapEntry {
        target: '圣',
        source: '聖',
        glyph_index: 2940,
    },
    MapEntry {
        target: '责',
        source: '責',
        glyph_index: 2966,
    },
    MapEntry {
        target: '丝',
        source: '絲',
        glyph_index: 6406,
    },
    MapEntry {
        target: '赶',
        source: '骭',
        glyph_index: 7595,
    },
    MapEntry {
        target: '页',
        source: '頁',
        glyph_index: 3892,
    },
    MapEntry {
        target: '险',
        source: '険',
        glyph_index: 2178,
    },
    MapEntry {
        target: '顿',
        source: '頓',
        glyph_index: 3537,
    },
    MapEntry {
        target: '孙',
        source: '孫',
        glyph_index: 3126,
    },
    MapEntry {
        target: '凤',
        source: '鳳',
        glyph_index: 3964,
    },
    MapEntry {
        target: '药',
        source: '薬',
        glyph_index: 4125,
    },
    MapEntry {
        target: '妇',
        source: '婦',
        glyph_index: 3815,
    },
    MapEntry {
        target: '韦',
        source: '韋',
        glyph_index: 7499,
    },
    MapEntry {
        target: '养',
        source: '養',
        glyph_index: 4195,
    },
    MapEntry {
        target: '卡',
        source: '裃',
        glyph_index: 6926,
    },
    MapEntry {
        target: '园',
        source: '園',
        glyph_index: 1567,
    },
    MapEntry {
        target: '纸',
        source: '紙',
        glyph_index: 2513,
    },
    MapEntry {
        target: '劳',
        source: '労',
        glyph_index: 4334,
    },
    MapEntry {
        target: '钟',
        source: '鐘',
        glyph_index: 2791,
    },
    MapEntry {
        target: '扬',
        source: '揚',
        glyph_index: 4175,
    },
    MapEntry {
        target: '误',
        source: '誤',
        glyph_index: 2238,
    },
    MapEntry {
        target: '咱',
        source: '齊',
        glyph_index: 7785,
    },
    MapEntry {
        target: '划',
        source: '劃',
        glyph_index: 1727,
    },
    MapEntry {
        target: '挥',
        source: '揮',
        glyph_index: 1873,
    },
    MapEntry {
        target: '毕',
        source: '畢',
        glyph_index: 3772,
    },
    MapEntry {
        target: '负',
        source: '負',
        glyph_index: 3832,
    },
    MapEntry {
        target: '宁',
        source: '寧',
        glyph_index: 3582,
    },
    MapEntry {
        target: '您',
        source: '鼡',
        glyph_index: 7782,
    },
    MapEntry {
        target: '银',
        source: '銀',
        glyph_index: 2041,
    },
    MapEntry {
        target: '诸',
        source: '諸',
        glyph_index: 2715,
    },
    MapEntry {
        target: '阴',
        source: '陰',
        glyph_index: 1505,
    },
    MapEntry {
        target: '吴',
        source: '呉',
        glyph_index: 2227,
    },
    MapEntry {
        target: '赵',
        source: '趙',
        glyph_index: 7117,
    },
    MapEntry {
        target: '闪',
        source: '閃',
        glyph_index: 3021,
    },
    MapEntry {
        target: '纳',
        source: '納',
        glyph_index: 3599,
    },
    MapEntry {
        target: '罢',
        source: '罷',
        glyph_index: 3742,
    },
    MapEntry {
        target: '遗',
        source: '遺',
        glyph_index: 1477,
    },
    MapEntry {
        target: '顺',
        source: '順',
        glyph_index: 2702,
    },
    MapEntry {
        target: '辑',
        source: '輯',
        glyph_index: 2651,
    },
    MapEntry {
        target: '乡',
        source: '郷',
        glyph_index: 2004,
    },
    MapEntry {
        target: '临',
        source: '臨',
        glyph_index: 4284,
    },
    MapEntry {
        target: '坚',
        source: '堅',
        glyph_index: 2155,
    },
    MapEntry {
        target: '顶',
        source: '頂',
        glyph_index: 3315,
    },
    MapEntry {
        target: '练',
        source: '練',
        glyph_index: 4322,
    },
    MapEntry {
        target: '缓',
        source: '緩',
        glyph_index: 1828,
    },
    MapEntry {
        target: '增',
        source: '増',
        glyph_index: 3100,
    },
    MapEntry {
        target: '职',
        source: '職',
        glyph_index: 2825,
    },
    MapEntry {
        target: '啦',
        source: '鼕',
        glyph_index: 7781,
    },
    MapEntry {
        target: '馆',
        source: '館',
        glyph_index: 1844,
    },
    MapEntry {
        target: '伟',
        source: '偉',
        glyph_index: 1455,
    },
    MapEntry {
        target: '获',
        source: '獲',
        glyph_index: 1736,
    },
    MapEntry {
        target: '劲',
        source: '勁',
        glyph_index: 4611,
    },
    MapEntry {
        target: '枪',
        source: '槍',
        glyph_index: 3075,
    },
    MapEntry {
        target: '灭',
        source: '滅',
        glyph_index: 4080,
    },
    MapEntry {
        target: '懂',
        source: '皷',
        glyph_index: 7780,
    },
    MapEntry {
        target: '骂',
        source: '罵',
        glyph_index: 3616,
    },
    MapEntry {
        target: '辈',
        source: '輩',
        glyph_index: 3629,
    },
    MapEntry {
        target: '忆',
        source: '憶',
        glyph_index: 1614,
    },
    MapEntry {
        target: '优',
        source: '優',
        glyph_index: 4140,
    },
    MapEntry {
        target: '预',
        source: '預',
        glyph_index: 4169,
    },
    MapEntry {
        target: '择',
        source: '擇',
        glyph_index: 5361,
    },
    MapEntry {
        target: '补',
        source: '補',
        glyph_index: 3921,
    },
    MapEntry {
        target: '贝',
        source: '貝',
        glyph_index: 1704,
    },
    MapEntry {
        target: '鲜',
        source: '鮮',
        glyph_index: 3022,
    },
    MapEntry {
        target: '织',
        source: '織',
        glyph_index: 2824,
    },
    MapEntry {
        target: '积',
        source: '積',
        glyph_index: 2962,
    },
    MapEntry {
        target: '晓',
        source: '暁',
        glyph_index: 2012,
    },
    MapEntry {
        target: '卖',
        source: '売',
        glyph_index: 3639,
    },
    MapEntry {
        target: '仿',
        source: '倣',
        glyph_index: 3932,
    },
    MapEntry {
        target: '载',
        source: '載',
        glyph_index: 2409,
    },
    MapEntry {
        target: '桌',
        source: '鼈',
        glyph_index: 7779,
    },
    MapEntry {
        target: '卢',
        source: '盧',
        glyph_index: 6135,
    },
    MapEntry {
        target: '诺',
        source: '諾',
        glyph_index: 3191,
    },
    MapEntry {
        target: '偷',
        source: '偸',
        glyph_index: 4506,
    },
    MapEntry {
        target: '伙',
        source: '夥',
        glyph_index: 4871,
    },
    MapEntry {
        target: '纯',
        source: '純',
        glyph_index: 2698,
    },
    MapEntry {
        target: '课',
        source: '課',
        glyph_index: 1658,
    },
    MapEntry {
        target: '厉',
        source: '鼇',
        glyph_index: 7778,
    },
    MapEntry {
        target: '营',
        source: '営',
        glyph_index: 1539,
    },
    MapEntry {
        target: '奶',
        source: '黨',
        glyph_index: 7769,
    },
    MapEntry {
        target: '启',
        source: '啓',
        glyph_index: 2095,
    },
    MapEntry {
        target: '纷',
        source: '紛',
        glyph_index: 3875,
    },
    MapEntry {
        target: '摆',
        source: '擺',
        glyph_index: 5377,
    },
    MapEntry {
        target: '弹',
        source: '弾',
        glyph_index: 3233,
    },
    MapEntry {
        target: '财',
        source: '財',
        glyph_index: 2415,
    },
    MapEntry {
        target: '赏',
        source: '賞',
        glyph_index: 2787,
    },
    MapEntry {
        target: '盘',
        source: '盤',
        glyph_index: 3720,
    },
    MapEntry {
        target: '墙',
        source: '牆',
        glyph_index: 5936,
    },
    MapEntry {
        target: '爸',
        source: '點',
        glyph_index: 7765,
    },
    MapEntry {
        target: '丰',
        source: '豐',
        glyph_index: 7069,
    },
    MapEntry {
        target: '颇',
        source: '頗',
        glyph_index: 2911,
    },
    MapEntry {
        target: '赞',
        source: '賛',
        glyph_index: 2474,
    },
    MapEntry {
        target: '圆',
        source: '圓',
        glyph_index: 4797,
    },
    MapEntry {
        target: '绍',
        source: '紹',
        glyph_index: 2775,
    },
    MapEntry {
        target: '纵',
        source: '縦',
        glyph_index: 2667,
    },
    MapEntry {
        target: '贾',
        source: '賈',
        glyph_index: 7092,
    },
    MapEntry {
        target: '执',
        source: '執',
        glyph_index: 2562,
    },
    MapEntry {
        target: '虑',
        source: '慮',
        glyph_index: 4253,
    },
    MapEntry {
        target: '烦',
        source: '煩',
        glyph_index: 3714,
    },
    MapEntry {
        target: '爹',
        source: '黐',
        glyph_index: 7762,
    },
    MapEntry {
        target: '吓',
        source: '嚇',
        glyph_index: 1728,
    },
    MapEntry {
        target: '闹',
        source: '鬧',
        glyph_index: 7622,
    },
    MapEntry {
        target: '鱼',
        source: '魚',
        glyph_index: 1970,
    },
    MapEntry {
        target: '缘',
        source: '縁',
        glyph_index: 1582,
    },
    MapEntry {
        target: '闲',
        source: '閑',
        glyph_index: 1840,
    },
    MapEntry {
        target: '妮',
        source: '黌',
        glyph_index: 7759,
    },
    MapEntry {
        target: '骑',
        source: '騎',
        glyph_index: 1898,
    },
    MapEntry {
        target: '镜',
        source: '鏡',
        glyph_index: 2005,
    },
    MapEntry {
        target: '厅',
        source: '庁',
        glyph_index: 3292,
    },
    MapEntry {
        target: '诚',
        source: '誠',
        glyph_index: 2944,
    },
    MapEntry {
        target: '烧',
        source: '焼',
        glyph_index: 2763,
    },
    MapEntry {
        target: '倾',
        source: '傾',
        glyph_index: 2092,
    },
    MapEntry {
        target: '韩',
        source: '韓',
        glyph_index: 1843,
    },
    MapEntry {
        target: '户',
        source: '戸',
        glyph_index: 2206,
    },
    MapEntry {
        target: '丛',
        source: '叢',
        glyph_index: 3056,
    },
    MapEntry {
        target: '绿',
        source: '緑',
        glyph_index: 4277,
    },
    MapEntry {
        target: '惯',
        source: '慣',
        glyph_index: 1808,
    },
    MapEntry {
        target: '抢',
        source: '搶',
        glyph_index: 5342,
    },
    MapEntry {
        target: '项',
        source: '項',
        glyph_index: 2319,
    },
    MapEntry {
        target: '萧',
        source: '蕭',
        glyph_index: 6778,
    },
    MapEntry {
        target: '镇',
        source: '鎮',
        glyph_index: 3324,
    },
    MapEntry {
        target: '岛',
        source: '島',
        glyph_index: 3454,
    },
    MapEntry {
        target: '荣',
        source: '栄',
        glyph_index: 1544,
    },
    MapEntry {
        target: '搞',
        source: '鎬',
        glyph_index: 7345,
    },
    MapEntry {
        target: '跃',
        source: '躍',
        glyph_index: 4127,
    },
    MapEntry {
        target: '颜',
        source: '顔',
        glyph_index: 1858,
    },
    MapEntry {
        target: '审',
        source: '審',
        glyph_index: 2838,
    },
    MapEntry {
        target: '软',
        source: '軟',
        glyph_index: 3557,
    },
    MapEntry {
        target: '拥',
        source: '擁',
        glyph_index: 4177,
    },
    MapEntry {
        target: '贴',
        source: '貼',
        glyph_index: 3412,
    },
    MapEntry {
        target: '础',
        source: '礎',
        glyph_index: 3042,
    },
    MapEntry {
        target: '谋',
        source: '謀',
        glyph_index: 3984,
    },
    MapEntry {
        target: '飘',
        source: '飄',
        glyph_index: 7527,
    },
    MapEntry {
        target: '训',
        source: '訓',
        glyph_index: 2084,
    },
    MapEntry {
        target: '闭',
        source: '閉',
        glyph_index: 3889,
    },
    MapEntry {
        target: '尘',
        source: '塵',
        glyph_index: 2867,
    },
    MapEntry {
        target: '穷',
        source: '窮',
        glyph_index: 1950,
    },
    MapEntry {
        target: '汤',
        source: '湯',
        glyph_index: 3465,
    },
    MapEntry {
        target: '忧',
        source: '憂',
        glyph_index: 4146,
    },
    MapEntry {
        target: '荡',
        source: '蕩',
        glyph_index: 3479,
    },
    MapEntry {
        target: '碰',
        source: '麭',
        glyph_index: 7757,
    },
    MapEntry {
        target: '蓝',
        source: '藍',
        glyph_index: 4220,
    },
    MapEntry {
        target: '净',
        source: '淨',
        glyph_index: 5771,
    },
    MapEntry {
        target: '辩',
        source: '辯',
        glyph_index: 7214,
    },
    MapEntry {
        target: '访',
        source: '訪',
        glyph_index: 3959,
    },
    MapEntry {
        target: '苍',
        source: '蒼',
        glyph_index: 3090,
    },
    MapEntry {
        target: '愤',
        source: '憤',
        glyph_index: 3869,
    },
    MapEntry {
        target: '扑',
        source: '撲',
        glyph_index: 3995,
    },
    MapEntry {
        target: '萨',
        source: '薩',
        glyph_index: 2450,
    },
    MapEntry {
        target: '县',
        source: '県',
        glyph_index: 2170,
    },
    MapEntry {
        target: '岂',
        source: '豈',
        glyph_index: 7066,
    },
    MapEntry {
        target: '奋',
        source: '奮',
        glyph_index: 3872,
    },
    MapEntry {
        target: '夺',
        source: '奪',
        glyph_index: 3200,
    },
    MapEntry {
        target: '庆',
        source: '慶',
        glyph_index: 2103,
    },
    MapEntry {
        target: '泽',
        source: '澤',
        glyph_index: 5850,
    },
    MapEntry {
        target: '轮',
        source: '輪',
        glyph_index: 4285,
    },
    MapEntry {
        target: '针',
        source: '針',
        glyph_index: 2862,
    },
    MapEntry {
        target: '监',
        source: '監',
        glyph_index: 1823,
    },
    MapEntry {
        target: '宽',
        source: '寛',
        glyph_index: 1803,
    },
    MapEntry {
        target: '锋',
        source: '鋒',
        glyph_index: 3962,
    },
    MapEntry {
        target: '绪',
        source: '緒',
        glyph_index: 2710,
    },
    MapEntry {
        target: '迟',
        source: '遅',
        glyph_index: 3252,
    },
    MapEntry {
        target: '侧',
        source: '側',
        glyph_index: 3107,
    },
    MapEntry {
        target: '肃',
        source: '粛',
        glyph_index: 2676,
    },
    MapEntry {
        target: '鸡',
        source: '鶏',
        glyph_index: 2127,
    },
    MapEntry {
        target: '洁',
        source: '潔',
        glyph_index: 2140,
    },
    MapEntry {
        target: '仪',
        source: '儀',
        glyph_index: 1902,
    },
    MapEntry {
        target: '稳',
        source: '穏',
        glyph_index: 1623,
    },
    MapEntry {
        target: '乌',
        source: '烏',
        glyph_index: 1511,
    },
    MapEntry {
        target: '躲',
        source: '躱',
        glyph_index: 7173,
    },
    MapEntry {
        target: '郑',
        source: '鄭',
        glyph_index: 3385,
    },
    MapEntry {
        target: '彻',
        source: '徹',
        glyph_index: 3399,
    },
    MapEntry {
        target: '销',
        source: '銷',
        glyph_index: 7326,
    },
    MapEntry {
        target: '滚',
        source: '滾',
        glyph_index: 5821,
    },
    MapEntry {
        target: '鸟',
        source: '鳥',
        glyph_index: 3316,
    },
    MapEntry {
        target: '废',
        source: '廃',
        glyph_index: 3620,
    },
    MapEntry {
        target: '疯',
        source: '瘋',
        glyph_index: 6085,
    },
    MapEntry {
        target: '聪',
        source: '聡',
        glyph_index: 3086,
    },
    MapEntry {
        target: '测',
        source: '測',
        glyph_index: 3113,
    },
    MapEntry {
        target: '涉',
        source: '渉',
        glyph_index: 2761,
    },
    MapEntry {
        target: '姊',
        source: '姉',
        glyph_index: 2491,
    },
    MapEntry {
        target: '袭',
        source: '襲',
        glyph_index: 2648,
    },
    MapEntry {
        target: '狱',
        source: '獄',
        glyph_index: 2341,
    },
    MapEntry {
        target: '毁',
        source: '毀',
        glyph_index: 4837,
    },
    MapEntry {
        target: '饮',
        source: '飲',
        glyph_index: 1500,
    },
    MapEntry {
        target: '阅',
        source: '閲',
        glyph_index: 1563,
    },
    MapEntry {
        target: '检',
        source: '検',
        glyph_index: 2162,
    },
    MapEntry {
        target: '倘',
        source: '麪',
        glyph_index: 7756,
    },
    MapEntry {
        target: '傻',
        source: '麩',
        glyph_index: 7754,
    },
    MapEntry {
        target: '缩',
        source: '縮',
        glyph_index: 2675,
    },
    MapEntry {
        target: '尝',
        source: '嘗',
        glyph_index: 2733,
    },
    MapEntry {
        target: '浓',
        source: '濃',
        glyph_index: 3598,
    },
    MapEntry {
        target: '丫',
        source: '麥',
        glyph_index: 7753,
    },
    MapEntry {
        target: '乔',
        source: '麕',
        glyph_index: 7750,
    },
    MapEntry {
        target: '挡',
        source: '麌',
        glyph_index: 7748,
    },
    MapEntry {
        target: '赛',
        source: '賽',
        glyph_index: 7097,
    },
    MapEntry {
        target: '货',
        source: '貨',
        glyph_index: 1660,
    },
    MapEntry {
        target: '揭',
        source: '掲',
        glyph_index: 2106,
    },
    MapEntry {
        target: '协',
        source: '協',
        glyph_index: 1981,
    },
    MapEntry {
        target: '贼',
        source: '賊',
        glyph_index: 3118,
    },
    MapEntry {
        target: '厌',
        source: '厭',
        glyph_index: 1565,
    },
    MapEntry {
        target: '输',
        source: '輸',
        glyph_index: 4137,
    },
    MapEntry {
        target: '骗',
        source: '騙',
        glyph_index: 7576,
    },
    MapEntry {
        target: '丧',
        source: '喪',
        glyph_index: 3058,
    },
    MapEntry {
        target: '躺',
        source: '麁',
        glyph_index: 7745,
    },
    MapEntry {
        target: '奖',
        source: '奨',
        glyph_index: 2734,
    },
    MapEntry {
        target: '桥',
        source: '橋',
        glyph_index: 1995,
    },
    MapEntry {
        target: '玛',
        source: '瑪',
        glyph_index: 6006,
    },
    MapEntry {
        target: '劝',
        source: '勧',
        glyph_index: 1796,
    },
    MapEntry {
        target: '逻',
        source: '邏',
        glyph_index: 7259,
    },
    MapEntry {
        target: '鸣',
        source: '鳴',
        glyph_index: 4077,
    },
    MapEntry {
        target: '拖',
        source: '鹽',
        glyph_index: 7744,
    },
    MapEntry {
        target: '减',
        source: '減',
        glyph_index: 2187,
    },
    MapEntry {
        target: '扫',
        source: '掃',
        glyph_index: 3068,
    },
    MapEntry {
        target: '损',
        source: '損',
        glyph_index: 3128,
    },
    MapEntry {
        target: '闷',
        source: '悶',
        glyph_index: 4110,
    },
    MapEntry {
        target: '详',
        source: '詳',
        glyph_index: 2785,
    },
    MapEntry {
        target: '烂',
        source: '爛',
        glyph_index: 5926,
    },
    MapEntry {
        target: '库',
        source: '庫',
        glyph_index: 2204,
    },
    MapEntry {
        target: '朵',
        source: '朶',
        glyph_index: 5471,
    },
    MapEntry {
        target: '嘿',
        source: '鹹',
        glyph_index: 7743,
    },
    MapEntry {
        target: '绕',
        source: '繞',
        glyph_index: 6460,
    },
    MapEntry {
        target: '冯',
        source: '馮',
        glyph_index: 7559,
    },
    MapEntry {
        target: '吕',
        source: '呂',
        glyph_index: 4327,
    },
    MapEntry {
        target: '绘',
        source: '絵',
        glyph_index: 1699,
    },
    MapEntry {
        target: '赖',
        source: '頼',
        glyph_index: 4209,
    },
    MapEntry {
        target: '哼',
        source: '鷽',
        glyph_index: 7738,
    },
    MapEntry {
        target: '码',
        source: '碼',
        glyph_index: 6200,
    },
    MapEntry {
        target: '颤',
        source: '顫',
        glyph_index: 7517,
    },
    MapEntry {
        target: '竞',
        source: '競',
        glyph_index: 1978,
    },
    MapEntry {
        target: '贫',
        source: '貧',
        glyph_index: 3806,
    },
    MapEntry {
        target: '帅',
        source: '帥',
        glyph_index: 2886,
    },
    MapEntry {
        target: '阁',
        source: '閣',
        glyph_index: 1744,
    },
    MapEntry {
        target: '姬',
        source: '鷭',
        glyph_index: 7736,
    },
    MapEntry {
        target: '贯',
        source: '貫',
        glyph_index: 1836,
    },
    MapEntry {
        target: '浑',
        source: '渾',
        glyph_index: 5786,
    },
    MapEntry {
        target: '丢',
        source: '鷏',
        glyph_index: 7730,
    },
    MapEntry {
        target: '暂',
        source: '暫',
        glyph_index: 2478,
    },
    MapEntry {
        target: '舰',
        source: '艦',
        glyph_index: 1832,
    },
    MapEntry {
        target: '瞪',
        source: '鷆',
        glyph_index: 7729,
    },
    MapEntry {
        target: '夹',
        source: '夾',
        glyph_index: 4876,
    },
    MapEntry {
        target: '饰',
        source: '飾',
        glyph_index: 2819,
    },
    MapEntry {
        target: '俩',
        source: '倆',
        glyph_index: 4496,
    },
    MapEntry {
        target: '兽',
        source: '獸',
        glyph_index: 5978,
    },
    MapEntry {
        target: '坛',
        source: '壇',
        glyph_index: 3232,
    },
    MapEntry {
        target: '贤',
        source: '賢',
        glyph_index: 2174,
    },
    MapEntry {
        target: '宾',
        source: '賓',
        glyph_index: 3807,
    },
    MapEntry {
        target: '锁',
        source: '鎖',
        glyph_index: 2380,
    },
    MapEntry {
        target: '莲',
        source: '蓮',
        glyph_index: 4324,
    },
    MapEntry {
        target: '牵',
        source: '牽',
        glyph_index: 2164,
    },
    MapEntry {
        target: '睁',
        source: '諍',
        glyph_index: 7013,
    },
    MapEntry {
        target: '轰',
        source: '轟',
        glyph_index: 2331,
    },
    MapEntry {
        target: '汇',
        source: '匯',
        glyph_index: 4632,
    },
    MapEntry {
        target: '腾',
        source: '騰',
        glyph_index: 3490,
    },
    MapEntry {
        target: '额',
        source: '額',
        glyph_index: 1750,
    },
    MapEntry {
        target: '鉴',
        source: '鑑',
        glyph_index: 1838,
    },
    MapEntry {
        target: '辉',
        source: '輝',
        glyph_index: 1896,
    },
    MapEntry {
        target: '缠',
        source: '纏',
        glyph_index: 3410,
    },
    MapEntry {
        target: '唤',
        source: '喚',
        glyph_index: 1798,
    },
    MapEntry {
        target: '讯',
        source: '訊',
        glyph_index: 2873,
    },
    MapEntry {
        target: '雾',
        source: '霧',
        glyph_index: 4065,
    },
    MapEntry {
        target: '迁',
        source: '遷',
        glyph_index: 3018,
    },
    MapEntry {
        target: '恼',
        source: '悩',
        glyph_index: 3597,
    },
    MapEntry {
        target: '拼',
        source: '並',
        glyph_index: 3887,
    },
    MapEntry {
        target: '铺',
        source: '舗',
        glyph_index: 3915,
    },
    MapEntry {
        target: '订',
        source: '訂',
        glyph_index: 3380,
    },
    MapEntry {
        target: '阔',
        source: '闊',
        glyph_index: 7403,
    },
    MapEntry {
        target: '腊',
        source: '臘',
        glyph_index: 6615,
    },
    MapEntry {
        target: '龄',
        source: '齢',
        glyph_index: 4309,
    },
    MapEntry {
        target: '帐',
        source: '帳',
        glyph_index: 3291,
    },
    MapEntry {
        target: '皱',
        source: '皺',
        glyph_index: 6127,
    },
    MapEntry {
        target: '挣',
        source: '錚',
        glyph_index: 7334,
    },
    MapEntry {
        target: '扭',
        source: '紐',
        glyph_index: 3778,
    },
    MapEntry {
        target: '赋',
        source: '賦',
        glyph_index: 3833,
    },
    MapEntry {
        target: '驾',
        source: '駕',
        glyph_index: 1676,
    },
    MapEntry {
        target: '购',
        source: '購',
        glyph_index: 2311,
    },
    MapEntry {
        target: '涂',
        source: '塗',
        glyph_index: 3424,
    },
    MapEntry {
        target: '嗯',
        source: '鶺',
        glyph_index: 7728,
    },
    MapEntry {
        target: '颗',
        source: '顆',
        glyph_index: 7514,
    },
    MapEntry {
        target: '扩',
        source: '拡',
        glyph_index: 1731,
    },
    MapEntry {
        target: '艳',
        source: '艶',
        glyph_index: 1583,
    },
    MapEntry {
        target: '瘦',
        source: '痩',
        glyph_index: 3080,
    },
    MapEntry {
        target: '喻',
        source: '喩',
        glyph_index: 4739,
    },
    MapEntry {
        target: '祸',
        source: '禍',
        glyph_index: 1647,
    },
    MapEntry {
        target: '艰',
        source: '艱',
        glyph_index: 6648,
    },
    MapEntry {
        target: '扯',
        source: '鶸',
        glyph_index: 7727,
    },
    MapEntry {
        target: '裤',
        source: '鷁',
        glyph_index: 7725,
    },
    MapEntry {
        target: '踢',
        source: '鷄',
        glyph_index: 7724,
    },
    MapEntry {
        target: '凯',
        source: '凱',
        glyph_index: 1705,
    },
    MapEntry {
        target: '逊',
        source: '遜',
        glyph_index: 3130,
    },
    MapEntry {
        target: '钢',
        source: '鋼',
        glyph_index: 2316,
    },
    MapEntry {
        target: '脏',
        source: '臓',
        glyph_index: 3102,
    },
    MapEntry {
        target: '庙',
        source: '廟',
        glyph_index: 3791,
    },
    MapEntry {
        target: '递',
        source: '逓',
        glyph_index: 3383,
    },
    MapEntry {
        target: '锐',
        source: '鋭',
        glyph_index: 1555,
    },
    MapEntry {
        target: '摔',
        source: '鶲',
        glyph_index: 7723,
    },
    MapEntry {
        target: '抚',
        source: '撫',
        glyph_index: 3838,
    },
    MapEntry {
        target: '鸿',
        source: '鴻',
        glyph_index: 2322,
    },
    MapEntry {
        target: '砍',
        source: '鶤',
        glyph_index: 7721,
    },
    MapEntry {
        target: '盯',
        source: '鵺',
        glyph_index: 7719,
    },
    MapEntry {
        target: '趋',
        source: '趨',
        glyph_index: 2905,
    },
    MapEntry {
        target: '晕',
        source: '暈',
        glyph_index: 5441,
    },
    MapEntry {
        target: '娇',
        source: '嬌',
        glyph_index: 4927,
    },
    MapEntry {
        target: '挤',
        source: '擠',
        glyph_index: 5368,
    },
    MapEntry {
        target: '钻',
        source: '鑚',
        glyph_index: 7383,
    },
    MapEntry {
        target: '纽',
        source: '鈕',
        glyph_index: 7307,
    },
    MapEntry {
        target: '键',
        source: '鍵',
        glyph_index: 2177,
    },
    MapEntry {
        target: '俱',
        source: '倶',
        glyph_index: 2043,
    },
    MapEntry {
        target: '铜',
        source: '銅',
        glyph_index: 3505,
    },
    MapEntry {
        target: '绳',
        source: '縄',
        glyph_index: 3553,
    },
    MapEntry {
        target: '悬',
        source: '懸',
        glyph_index: 2159,
    },
    MapEntry {
        target: '络',
        source: '絡',
        glyph_index: 4212,
    },
    MapEntry {
        target: '贡',
        source: '貢',
        glyph_index: 2310,
    },
    MapEntry {
        target: '鹏',
        source: '鵬',
        glyph_index: 3965,
    },
    MapEntry {
        target: '怔',
        source: '鵯',
        glyph_index: 7718,
    },
    MapEntry {
        target: '喂',
        source: '鶫',
        glyph_index: 7717,
    },
    MapEntry {
        target: '讽',
        source: '諷',
        glyph_index: 7024,
    },
    MapEntry {
        target: '兹',
        source: '茲',
        glyph_index: 6677,
    },
    MapEntry {
        target: '违',
        source: '違',
        glyph_index: 1476,
    },
    MapEntry {
        target: '吞',
        source: '呑',
        glyph_index: 3538,
    },
    MapEntry {
        target: '炼',
        source: '煉',
        glyph_index: 4320,
    },
    MapEntry {
        target: '嘻',
        source: '鶇',
        glyph_index: 7716,
    },
    MapEntry {
        target: '驱',
        source: '駆',
        glyph_index: 2051,
    },
    MapEntry {
        target: '灾',
        source: '災',
        glyph_index: 2399,
    },
    MapEntry {
        target: '谱',
        source: '譜',
        glyph_index: 3831,
    },
    MapEntry {
        target: '颂',
        source: '頌',
        glyph_index: 7508,
    },
    MapEntry {
        target: '扰',
        source: '擾',
        glyph_index: 2806,
    },
    MapEntry {
        target: '伪',
        source: '偽',
        glyph_index: 1901,
    },
    MapEntry {
        target: '笼',
        source: '篭',
        glyph_index: 4345,
    },
    MapEntry {
        target: '辆',
        source: '輌',
        glyph_index: 7193,
    },
    MapEntry {
        target: '猎',
        source: '猟',
        glyph_index: 4265,
    },
    MapEntry {
        target: '饿',
        source: '餓',
        glyph_index: 1675,
    },
    MapEntry {
        target: '肤',
        source: '膚',
        glyph_index: 3829,
    },
    MapEntry {
        target: '绩',
        source: '績',
        glyph_index: 2964,
    },
    MapEntry {
        target: '亏',
        source: '虧',
        glyph_index: 6811,
    },
    MapEntry {
        target: '喷',
        source: '噴',
        glyph_index: 3867,
    },
    MapEntry {
        target: '枫',
        source: '楓',
        glyph_index: 3845,
    },
    MapEntry {
        target: '污',
        source: '汚',
        glyph_index: 1591,
    },
    MapEntry {
        target: '赌',
        source: '賭',
        glyph_index: 3433,
    },
    MapEntry {
        target: '扔',
        source: '鵙',
        glyph_index: 7713,
    },
    MapEntry {
        target: '吵',
        source: '鵐',
        glyph_index: 7712,
    },
    MapEntry {
        target: '诞',
        source: '誕',
        glyph_index: 3229,
    },
    MapEntry {
        target: '畅',
        source: '暢',
        glyph_index: 3299,
    },
    MapEntry {
        target: '阐',
        source: '闡',
        glyph_index: 7412,
    },
    MapEntry {
        target: '赢',
        source: '贏',
        glyph_index: 7104,
    },
    MapEntry {
        target: '拦',
        source: '攬',
        glyph_index: 5373,
    },
    MapEntry {
        target: '润',
        source: '潤',
        glyph_index: 2696,
    },
    MapEntry {
        target: '唉',
        source: '鵤',
        glyph_index: 7710,
    },
    MapEntry {
        target: '锦',
        source: '錦',
        glyph_index: 2024,
    },
    MapEntry {
        target: '懒',
        source: '懶',
        glyph_index: 5250,
    },
    MapEntry {
        target: '斩',
        source: '斬',
        glyph_index: 2477,
    },
    MapEntry {
        target: '签',
        source: '簽',
        glyph_index: 6350,
    },
    MapEntry {
        target: '撑',
        source: '鵞',
        glyph_index: 7709,
    },
    MapEntry {
        target: '齿',
        source: '歯',
        glyph_index: 2528,
    },
    MapEntry {
        target: '罚',
        source: '罰',
        glyph_index: 3684,
    },
    MapEntry {
        target: '拨',
        source: '撥',
        glyph_index: 5354,
    },
    MapEntry {
        target: '盼',
        source: '鵈',
        glyph_index: 7707,
    },
    MapEntry {
        target: '拟',
        source: '擬',
        glyph_index: 1907,
    },
    MapEntry {
        target: '诱',
        source: '誘',
        glyph_index: 4157,
    },
    MapEntry {
        target: '琼',
        source: '瓊',
        glyph_index: 6012,
    },
    MapEntry {
        target: '驰',
        source: '馳',
        glyph_index: 3253,
    },
    MapEntry {
        target: '卧',
        source: '臥',
        glyph_index: 1670,
    },
    MapEntry {
        target: '宪',
        source: '憲',
        glyph_index: 2158,
    },
    MapEntry {
        target: '邮',
        source: '郵',
        glyph_index: 4160,
    },
    MapEntry {
        target: '樱',
        source: '桜',
        glyph_index: 2438,
    },
    MapEntry {
        target: '缝',
        source: '縫',
        glyph_index: 3952,
    },
    MapEntry {
        target: '饶',
        source: '饒',
        glyph_index: 7552,
    },
    MapEntry {
        target: '贺',
        source: '賀',
        glyph_index: 1673,
    },
    MapEntry {
        target: '讶',
        source: '訝',
        glyph_index: 6990,
    },
    MapEntry {
        target: '综',
        source: '綜',
        glyph_index: 3085,
    },
    MapEntry {
        target: '栏',
        source: '欄',
        glyph_index: 4218,
    },
    MapEntry {
        target: '沟',
        source: '溝',
        glyph_index: 2289,
    },
    MapEntry {
        target: '谨',
        source: '謹',
        glyph_index: 2037,
    },
    MapEntry {
        target: '庞',
        source: '厖',
        glyph_index: 4650,
    },
    MapEntry {
        target: '疗',
        source: '療',
        glyph_index: 4266,
    },
    MapEntry {
        target: '迈',
        source: '邁',
        glyph_index: 7255,
    },
    MapEntry {
        target: '挖',
        source: '窪',
        glyph_index: 2073,
    },
    MapEntry {
        target: '摄',
        source: '摂',
        glyph_index: 2974,
    },
    MapEntry {
        target: '渴',
        source: '渇',
        glyph_index: 1764,
    },
    MapEntry {
        target: '铃',
        source: '鈴',
        glyph_index: 4304,
    },
    MapEntry {
        target: '饱',
        source: '飽',
        glyph_index: 3963,
    },
    MapEntry {
        target: '凑',
        source: '湊',
        glyph_index: 4052,
    },
    MapEntry {
        target: '纠',
        source: '糾',
        glyph_index: 1953,
    },
    MapEntry {
        target: '柜',
        source: '櫃',
        glyph_index: 5643,
    },
    MapEntry {
        target: '烛',
        source: '燭',
        glyph_index: 2823,
    },
    MapEntry {
        target: '颠',
        source: '顛',
        glyph_index: 3414,
    },
    MapEntry {
        target: '婴',
        source: '嬰',
        glyph_index: 1540,
    },
    MapEntry {
        target: '啸',
        source: '嘯',
        glyph_index: 4765,
    },
    MapEntry {
        target: '牺',
        source: '犠',
        glyph_index: 1909,
    },
    MapEntry {
        target: '帕',
        source: '袙',
        glyph_index: 6919,
    },
    MapEntry {
        target: '览',
        source: '覧',
        glyph_index: 4222,
    },
    MapEntry {
        target: '贪',
        source: '貪',
        glyph_index: 7086,
    },
    MapEntry {
        target: '绵',
        source: '綿',
        glyph_index: 4083,
    },
    MapEntry {
        target: '婷',
        source: '鵆',
        glyph_index: 7706,
    },
    MapEntry {
        target: '闯',
        source: '闖',
        glyph_index: 7410,
    },
    MapEntry {
        target: '啡',
        source: '鴾',
        glyph_index: 7705,
    },
    MapEntry {
        target: '鹰',
        source: '鷹',
        glyph_index: 3176,
    },
    MapEntry {
        target: '谦',
        source: '謙',
        glyph_index: 2173,
    },
    MapEntry {
        target: '焰',
        source: '焔',
        glyph_index: 1578,
    },
    MapEntry {
        target: '邻',
        source: '隣',
        glyph_index: 4286,
    },
    MapEntry {
        target: '窝',
        source: '窩',
        glyph_index: 6264,
    },
    MapEntry {
        target: '胁',
        source: '脇',
        glyph_index: 4361,
    },
    MapEntry {
        target: '谅',
        source: '諒',
        glyph_index: 4271,
    },
    MapEntry {
        target: '币',
        source: '幣',
        glyph_index: 3883,
    },
    MapEntry {
        target: '肠',
        source: '腸',
        glyph_index: 3307,
    },
    MapEntry {
        target: '顽',
        source: '頑',
        glyph_index: 1857,
    },
    MapEntry {
        target: '骚',
        source: '騒',
        glyph_index: 3098,
    },
    MapEntry {
        target: '纹',
        source: '紋',
        glyph_index: 4111,
    },
    MapEntry {
        target: '纲',
        source: '綱',
        glyph_index: 2298,
    },
    MapEntry {
        target: '咖',
        source: '鵁',
        glyph_index: 7703,
    },
    MapEntry {
        target: '斋',
        source: '斎',
        glyph_index: 2405,
    },
    MapEntry {
        target: '呜',
        source: '嗚',
        glyph_index: 4742,
    },
    MapEntry {
        target: '脖',
        source: '鴒',
        glyph_index: 7702,
    },
    MapEntry {
        target: '删',
        source: '刪',
        glyph_index: 4583,
    },
    MapEntry {
        target: '泼',
        source: '溌',
        glyph_index: 3679,
    },
    MapEntry {
        target: '掷',
        source: '擲',
        glyph_index: 5376,
    },
    MapEntry {
        target: '频',
        source: '頻',
        glyph_index: 3808,
    },
    MapEntry {
        target: '镖',
        source: '驃',
        glyph_index: 7582,
    },
    MapEntry {
        target: '沪',
        source: '滬',
        glyph_index: 5819,
    },
    MapEntry {
        target: '歧',
        source: '鵄',
        glyph_index: 7700,
    },
    MapEntry {
        target: '哎',
        source: '鶯',
        glyph_index: 7697,
    },
    MapEntry {
        target: '龟',
        source: '亀',
        glyph_index: 1900,
    },
    MapEntry {
        target: '赠',
        source: '贈',
        glyph_index: 3104,
    },
    MapEntry {
        target: '仑',
        source: '侖',
        glyph_index: 4470,
    },
    MapEntry {
        target: '颈',
        source: '頚',
        glyph_index: 2126,
    },
    MapEntry {
        target: '晌',
        source: '鴪',
        glyph_index: 7695,
    },
    MapEntry {
        target: '耸',
        source: '聳',
        glyph_index: 6549,
    },
    MapEntry {
        target: '衬',
        source: '襯',
        glyph_index: 6960,
    },
    MapEntry {
        target: '坠',
        source: '墜',
        glyph_index: 3327,
    },
    MapEntry {
        target: '绣',
        source: '綉',
        glyph_index: 6412,
    },
    MapEntry {
        target: '愣',
        source: '鴃',
        glyph_index: 7693,
    },
    MapEntry {
        target: '驶',
        source: '駛',
        glyph_index: 7562,
    },
    MapEntry {
        target: '玫',
        source: '鳫',
        glyph_index: 7692,
    },
    MapEntry {
        target: '询',
        source: '詢',
        glyph_index: 7001,
    },
    MapEntry {
        target: '钉',
        source: '釘',
        glyph_index: 3386,
    },
    MapEntry {
        target: '蕴',
        source: '蘊',
        glyph_index: 6797,
    },
    MapEntry {
        target: '赐',
        source: '賜',
        glyph_index: 2525,
    },
    MapEntry {
        target: '谐',
        source: '諧',
        glyph_index: 7018,
    },
    MapEntry {
        target: '邓',
        source: '鴈',
        glyph_index: 7691,
    },
    MapEntry {
        target: '砸',
        source: '鳰',
        glyph_index: 7689,
    },
    MapEntry {
        target: '锅',
        source: '鍋',
        glyph_index: 3550,
    },
    MapEntry {
        target: '鹤',
        source: '鶴',
        glyph_index: 3354,
    },
    MapEntry {
        target: '诡',
        source: '詭',
        glyph_index: 6999,
    },
    MapEntry {
        target: '眨',
        source: '詐',
        glyph_index: 2379,
    },
    MapEntry {
        target: '驻',
        source: '駐',
        glyph_index: 3278,
    },
    MapEntry {
        target: '贱',
        source: '賎',
        glyph_index: 3015,
    },
    MapEntry {
        target: '驳',
        source: '駁',
        glyph_index: 3664,
    },
    MapEntry {
        target: '宠',
        source: '寵',
        glyph_index: 3289,
    },
    MapEntry {
        target: '厕',
        source: '厠',
        glyph_index: 4651,
    },
    MapEntry {
        target: '哑',
        source: '唖',
        glyph_index: 1411,
    },
    MapEntry {
        target: '莹',
        source: '瑩',
        glyph_index: 6003,
    },
    MapEntry {
        target: '亿',
        source: '億',
        glyph_index: 1612,
    },
    MapEntry {
        target: '叠',
        source: '畳',
        glyph_index: 2811,
    },
    MapEntry {
        target: '骇',
        source: '駭',
        glyph_index: 7566,
    },
    MapEntry {
        target: '渔',
        source: '漁',
        glyph_index: 1968,
    },
    MapEntry {
        target: '颊',
        source: '頬',
        glyph_index: 3990,
    },
    MapEntry {
        target: '摊',
        source: '攤',
        glyph_index: 5383,
    },
    MapEntry {
        target: '谎',
        source: '鳧',
        glyph_index: 7687,
    },
    MapEntry {
        target: '缚',
        source: '縛',
        glyph_index: 3662,
    },
    MapEntry {
        target: '帘',
        source: '簾',
        glyph_index: 4321,
    },
    MapEntry {
        target: '钦',
        source: '欽',
        glyph_index: 2027,
    },
    MapEntry {
        target: '轨',
        source: '軌',
        glyph_index: 1895,
    },
    MapEntry {
        target: '筹',
        source: '籌',
        glyph_index: 6351,
    },
    MapEntry {
        target: '窜',
        source: '竄',
        glyph_index: 6269,
    },
    MapEntry {
        target: '钧',
        source: '鈞',
        glyph_index: 7303,
    },
    MapEntry {
        target: '嚷',
        source: '鱶',
        glyph_index: 7685,
    },
    MapEntry {
        target: '溃',
        source: '潰',
        glyph_index: 3346,
    },
    MapEntry {
        target: '煞',
        source: '鱧',
        glyph_index: 7684,
    },
    MapEntry {
        target: '辽',
        source: '遼',
        glyph_index: 4272,
    },
    MapEntry {
        target: '贬',
        source: '貶',
        glyph_index: 7091,
    },
    MapEntry {
        target: '轩',
        source: '軒',
        glyph_index: 2175,
    },
    MapEntry {
        target: '尬',
        source: '鱚',
        glyph_index: 7682,
    },
    MapEntry {
        target: '瞒',
        source: '瞞',
        glyph_index: 6163,
    },
    MapEntry {
        target: '鸦',
        source: '鴉',
        glyph_index: 7690,
    },
    MapEntry {
        target: '搂',
        source: '僂',
        glyph_index: 4515,
    },
    MapEntry {
        target: '尴',
        source: '鰾',
        glyph_index: 7681,
    },
    MapEntry {
        target: '钩',
        source: '鈎',
        glyph_index: 1726,
    },
    MapEntry {
        target: '锡',
        source: '錫',
        glyph_index: 2603,
    },
    MapEntry {
        target: '绑',
        source: '鱆',
        glyph_index: 7680,
    },
    MapEntry {
        target: '囊',
        source: '嚢',
        glyph_index: 3596,
    },
    MapEntry {
        target: '饥',
        source: '飢',
        glyph_index: 1897,
    },
    MapEntry {
        target: '谊',
        source: '誼',
        glyph_index: 1914,
    },
    MapEntry {
        target: '谬',
        source: '謬',
        glyph_index: 3780,
    },
    MapEntry {
        target: '赚',
        source: '賺',
        glyph_index: 7098,
    },
    MapEntry {
        target: '甩',
        source: '鱇',
        glyph_index: 7678,
    },
    MapEntry {
        target: '窥',
        source: '窺',
        glyph_index: 1517,
    },
    MapEntry {
        target: '糕',
        source: '鰰',
        glyph_index: 7677,
    },
    MapEntry {
        target: '盐',
        source: '塩',
        glyph_index: 1589,
    },
    MapEntry {
        target: '咕',
        source: '鰡',
        glyph_index: 7676,
    },
    MapEntry {
        target: '竖',
        source: '竪',
        glyph_index: 3203,
    },
    MapEntry {
        target: '狮',
        source: '獅',
        glyph_index: 2509,
    },
    MapEntry {
        target: '胀',
        source: '脹',
        glyph_index: 3306,
    },
    MapEntry {
        target: '贞',
        source: '貞',
        glyph_index: 3360,
    },
    MapEntry {
        target: '辅',
        source: '輔',
        glyph_index: 3922,
    },
    MapEntry {
        target: '沧',
        source: '滄',
        glyph_index: 5808,
    },
    MapEntry {
        target: '诵',
        source: '誦',
        glyph_index: 7009,
    },
    MapEntry {
        target: '纤',
        source: '繊',
        glyph_index: 3008,
    },
    MapEntry {
        target: '铭',
        source: '銘',
        glyph_index: 4076,
    },
    MapEntry {
        target: '偿',
        source: '償',
        glyph_index: 2725,
    },
    MapEntry {
        target: '骄',
        source: '驕',
        glyph_index: 7584,
    },
    MapEntry {
        target: '谭',
        source: '譚',
        glyph_index: 7048,
    },
    MapEntry {
        target: '潇',
        source: '瀟',
        glyph_index: 5874,
    },
    MapEntry {
        target: '坟',
        source: '墳',
        glyph_index: 3868,
    },
    MapEntry {
        target: '哗',
        source: '嘩',
        glyph_index: 1659,
    },
    MapEntry {
        target: '纱',
        source: '紗',
        glyph_index: 2588,
    },
    MapEntry {
        target: '妆',
        source: '粧',
        glyph_index: 2774,
    },
    MapEntry {
        target: '荆',
        source: '荊',
        glyph_index: 2120,
    },
    MapEntry {
        target: '羡',
        source: '羨',
        glyph_index: 3009,
    },
    MapEntry {
        target: '搁',
        source: '擱',
        glyph_index: 5365,
    },
    MapEntry {
        target: '俞',
        source: '兪',
        glyph_index: 4544,
    },
    MapEntry {
        target: '哟',
        source: '鰤',
        glyph_index: 7675,
    },
    MapEntry {
        target: '诧',
        source: '詫',
        glyph_index: 4368,
    },
    MapEntry {
        target: '链',
        source: '鏈',
        glyph_index: 7357,
    },
    MapEntry {
        target: '壶',
        source: '壷',
        glyph_index: 3348,
    },
    MapEntry {
        target: '贸',
        source: '貿',
        glyph_index: 3986,
    },
    MapEntry {
        target: '滥',
        source: '濫',
        glyph_index: 4219,
    },
    MapEntry {
        target: '赔',
        source: '賠',
        glyph_index: 3640,
    },
    MapEntry {
        target: '耍',
        source: '鰛',
        glyph_index: 7673,
    },
    MapEntry {
        target: '钗',
        source: '釵',
        glyph_index: 7301,
    },
    MapEntry {
        target: '仓',
        source: '倉',
        glyph_index: 3057,
    },
    MapEntry {
        target: '债',
        source: '債',
        glyph_index: 2385,
    },
    MapEntry {
        target: '喔',
        source: '鰮',
        glyph_index: 7672,
    },
    MapEntry {
        target: '渗',
        source: '滲',
        glyph_index: 5823,
    },
    MapEntry {
        target: '涩',
        source: '渋',
        glyph_index: 2665,
    },
    MapEntry {
        target: '冻',
        source: '凍',
        glyph_index: 3447,
    },
    MapEntry {
        target: '灿',
        source: '燦',
        glyph_index: 2467,
    },
    MapEntry {
        target: '沦',
        source: '淪',
        glyph_index: 5778,
    },
    MapEntry {
        target: '踩',
        source: '採',
        glyph_index: 2395,
    },
    MapEntry {
        target: '棵',
        source: '鰄',
        glyph_index: 7671,
    },
    MapEntry {
        target: '谣',
        source: '謡',
        glyph_index: 4191,
    },
    MapEntry {
        target: '谜',
        source: '謎',
        glyph_index: 3547,
    },
    MapEntry {
        target: '聂',
        source: '聶',
        glyph_index: 6552,
    },
    MapEntry {
        target: '趟',
        source: '鰊',
        glyph_index: 7670,
    },
    MapEntry {
        target: '琐',
        source: '瑣',
        glyph_index: 6005,
    },
    MapEntry {
        target: '颖',
        source: '穎',
        glyph_index: 1550,
    },
    MapEntry {
        target: '磕',
        source: '鰒',
        glyph_index: 7669,
    },
    MapEntry {
        target: '砰',
        source: '鰈',
        glyph_index: 7668,
    },
    MapEntry {
        target: '腻',
        source: '膩',
        glyph_index: 6602,
    },
    MapEntry {
        target: '钥',
        source: '鑰',
        glyph_index: 7379,
    },
    MapEntry {
        target: '侄',
        source: '姪',
        glyph_index: 4078,
    },
    MapEntry {
        target: '砖',
        source: '甎',
        glyph_index: 6030,
    },
    MapEntry {
        target: '涨',
        source: '漲',
        glyph_index: 5826,
    },
    MapEntry {
        target: '啥',
        source: '鰆',
        glyph_index: 7667,
    },
    MapEntry {
        target: '惩',
        source: '懲',
        glyph_index: 3297,
    },
    MapEntry {
        target: '陡',
        source: '闘',
        glyph_index: 3491,
    },
    MapEntry {
        target: '饼',
        source: '餅',
        glyph_index: 4104,
    },
    MapEntry {
        target: '逛',
        source: '絋',
        glyph_index: 6397,
    },
    MapEntry {
        target: '讳',
        source: '諱',
        glyph_index: 7020,
    },
    MapEntry {
        target: '诀',
        source: '訣',
        glyph_index: 2144,
    },
    MapEntry {
        target: '毙',
        source: '斃',
        glyph_index: 5402,
    },
    MapEntry {
        target: '滩',
        source: '灘',
        glyph_index: 3548,
    },
    MapEntry {
        target: '贩',
        source: '販',
        glyph_index: 3711,
    },
    MapEntry {
        target: '嗓',
        source: '鰌',
        glyph_index: 7666,
    },
    MapEntry {
        target: '媳',
        source: '鰉',
        glyph_index: 7664,
    },
    MapEntry {
        target: '岗',
        source: '崗',
        glyph_index: 5011,
    },
    MapEntry {
        target: '叁',
        source: '鰔',
        glyph_index: 7663,
    },
    MapEntry {
        target: '捡',
        source: '揀',
        glyph_index: 5330,
    },
    MapEntry {
        target: '厢',
        source: '廂',
        glyph_index: 5072,
    },
    MapEntry {
        target: '尧',
        source: '尭',
        glyph_index: 2011,
    },
    MapEntry {
        target: '铸',
        source: '鋳',
        glyph_index: 3277,
    },
    MapEntry {
        target: '讥',
        source: '譏',
        glyph_index: 7043,
    },
    MapEntry {
        target: '伞',
        source: '傘',
        glyph_index: 2460,
    },
    MapEntry {
        target: '骤',
        source: '驟',
        glyph_index: 7588,
    },
    MapEntry {
        target: '塌',
        source: '鰕',
        glyph_index: 7662,
    },
    MapEntry {
        target: '轿',
        source: '轎',
        glyph_index: 7204,
    },
    MapEntry {
        target: '旷',
        source: '昿',
        glyph_index: 5457,
    },
    MapEntry {
        target: '俏',
        source: '鯰',
        glyph_index: 7661,
    },
    MapEntry {
        target: '岭',
        source: '嶺',
        glyph_index: 4299,
    },
    MapEntry {
        target: '芷',
        source: '鯱',
        glyph_index: 7660,
    },
    MapEntry {
        target: '缕',
        source: '縷',
        glyph_index: 6454,
    },
    MapEntry {
        target: '壳',
        source: '殻',
        glyph_index: 1735,
    },
    MapEntry {
        target: '秃',
        source: '禿',
        glyph_index: 3514,
    },
    MapEntry {
        target: '储',
        source: '儲',
        glyph_index: 4098,
    },
    MapEntry {
        target: '娱',
        source: '娯',
        glyph_index: 2229,
    },
    MapEntry {
        target: '咪',
        source: '鯲',
        glyph_index: 7659,
    },
    MapEntry {
        target: '肿',
        source: '腫',
        glyph_index: 2617,
    },
    MapEntry {
        target: '账',
        source: '鰺',
        glyph_index: 7658,
    },
    MapEntry {
        target: '侣',
        source: '侶',
        glyph_index: 4252,
    },
    MapEntry {
        target: '蜡',
        source: '蝋',
        glyph_index: 4348,
    },
    MapEntry {
        target: '栋',
        source: '棟',
        glyph_index: 3462,
    },
    MapEntry {
        target: '绽',
        source: '綻',
        glyph_index: 3225,
    },
    MapEntry {
        target: '咦',
        source: '鯔',
        glyph_index: 7656,
    },
    MapEntry {
        target: '胧',
        source: '朧',
        glyph_index: 5467,
    },
    MapEntry {
        target: '胳',
        source: '鯣',
        glyph_index: 7653,
    },
    MapEntry {
        target: '侦',
        source: '偵',
        glyph_index: 3358,
    },
    MapEntry {
        target: '钰',
        source: '鯒',
        glyph_index: 7652,
    },
    MapEntry {
        target: '妒',
        source: '妬',
        glyph_index: 3425,
    },
    MapEntry {
        target: '鲍',
        source: '鮑',
        glyph_index: 7639,
    },
    MapEntry {
        target: '澜',
        source: '瀾',
        glyph_index: 5876,
    },
    MapEntry {
        target: '矿',
        source: '鉱',
        glyph_index: 2314,
    },
    MapEntry {
        target: '郝',
        source: '鯑',
        glyph_index: 7651,
    },
    MapEntry {
        target: '恳',
        source: '懇',
        glyph_index: 2358,
    },
    MapEntry {
        target: '桢',
        source: '鯏',
        glyph_index: 7650,
    },
    MapEntry {
        target: '焕',
        source: '煥',
        glyph_index: 5896,
    },
    MapEntry {
        target: '缪',
        source: '繆',
        glyph_index: 6448,
    },
    MapEntry {
        target: '趴',
        source: '鯆',
        glyph_index: 7649,
    },
    MapEntry {
        target: '溅',
        source: '濺',
        glyph_index: 5864,
    },
    MapEntry {
        target: '撇',
        source: '鮹',
        glyph_index: 7648,
    },
    MapEntry {
        target: '挪',
        source: '鮴',
        glyph_index: 7645,
    },
    MapEntry {
        target: '霉',
        source: '黴',
        glyph_index: 7771,
    },
    MapEntry {
        target: '骆',
        source: '駱',
        glyph_index: 7568,
    },
    MapEntry {
        target: '垃',
        source: '鮨',
        glyph_index: 7644,
    },
    MapEntry {
        target: '啪',
        source: '鮠',
        glyph_index: 7643,
    },
    MapEntry {
        target: '圾',
        source: '鮗',
        glyph_index: 7641,
    },
    MapEntry {
        target: '烁',
        source: '爍',
        glyph_index: 5924,
    },
    MapEntry {
        target: '胶',
        source: '膠',
        glyph_index: 6596,
    },
    MapEntry {
        target: '挚',
        source: '摯',
        glyph_index: 5348,
    },
    MapEntry {
        target: '凳',
        source: '鮖',
        glyph_index: 7640,
    },
    MapEntry {
        target: '惭',
        source: '慚',
        glyph_index: 5217,
    },
    MapEntry {
        target: '绮',
        source: '綺',
        glyph_index: 6417,
    },
    MapEntry {
        target: '岚',
        source: '嵐',
        glyph_index: 4217,
    },
    MapEntry {
        target: '烤',
        source: '栲',
        glyph_index: 5513,
    },
    MapEntry {
        target: '鸳',
        source: '鴛',
        glyph_index: 1588,
    },
    MapEntry {
        target: '驴',
        source: '驢',
        glyph_index: 7589,
    },
    MapEntry {
        target: '驼',
        source: '駝',
        glyph_index: 7563,
    },
    MapEntry {
        target: '眯',
        source: '鮓',
        glyph_index: 7637,
    },
    MapEntry {
        target: '颓',
        source: '頽',
        glyph_index: 7513,
    },
    MapEntry {
        target: '笺',
        source: '箋',
        glyph_index: 6318,
    },
    MapEntry {
        target: '剂',
        source: '剤',
        glyph_index: 2411,
    },
    MapEntry {
        target: '舆',
        source: '輿',
        glyph_index: 4168,
    },
    MapEntry {
        target: '桩',
        source: '鬪',
        glyph_index: 7625,
    },
    MapEntry {
        target: '呐',
        source: '吶',
        glyph_index: 4674,
    },
    MapEntry {
        target: '拢',
        source: '隴',
        glyph_index: 7440,
    },
    MapEntry {
        target: '庐',
        source: '廬',
        glyph_index: 5085,
    },
    MapEntry {
        target: '闺',
        source: '閨',
        glyph_index: 7396,
    },
    MapEntry {
        target: '阎',
        source: '閻',
        glyph_index: 7400,
    },
    MapEntry {
        target: '诊',
        source: '診',
        glyph_index: 2858,
    },
    MapEntry {
        target: '捣',
        source: '擣',
        glyph_index: 5371,
    },
    MapEntry {
        target: '篮',
        source: '籃',
        glyph_index: 6352,
    },
    MapEntry {
        target: '舱',
        source: '艙',
        glyph_index: 6637,
    },
    MapEntry {
        target: '顷',
        source: '頃',
        glyph_index: 2351,
    },
    MapEntry {
        target: '衔',
        source: '銜',
        glyph_index: 7319,
    },
    MapEntry {
        target: '鸭',
        source: '鴨',
        glyph_index: 1782,
    },
    MapEntry {
        target: '吱',
        source: '鬨',
        glyph_index: 7623,
    },
    MapEntry {
        target: '觅',
        source: '覓',
        glyph_index: 6967,
    },
    MapEntry {
        target: '骏',
        source: '駿',
        glyph_index: 2688,
    },
    MapEntry {
        target: '佣',
        source: '傭',
        glyph_index: 4170,
    },
    MapEntry {
        target: '浆',
        source: '漿',
        glyph_index: 5822,
    },
    MapEntry {
        target: '贷',
        source: '貸',
        glyph_index: 3164,
    },
    MapEntry {
        target: '诈',
        source: '鬥',
        glyph_index: 7621,
    },
    MapEntry {
        target: '碟',
        source: '鬚',
        glyph_index: 7617,
    },
    MapEntry {
        target: '孽',
        source: '鬘',
        glyph_index: 7616,
    },
    MapEntry {
        target: '绸',
        source: '綢',
        glyph_index: 6425,
    },
    MapEntry {
        target: '鹅',
        source: '鵝',
        glyph_index: 7708,
    },
    MapEntry {
        target: '詹',
        source: '鬆',
        glyph_index: 7615,
    },
    MapEntry {
        target: '冈',
        source: '岡',
        glyph_index: 1609,
    },
    MapEntry {
        target: '琪',
        source: '髷',
        glyph_index: 7613,
    },
    MapEntry {
        target: '钞',
        source: '鈔',
        glyph_index: 7305,
    },
    MapEntry {
        target: '巩',
        source: '鞏',
        glyph_index: 7488,
    },
    MapEntry {
        target: '鸯',
        source: '鴦',
        glyph_index: 7696,
    },
    MapEntry {
        target: '呕',
        source: '嘔',
        glyph_index: 4749,
    },
    MapEntry {
        target: '烫',
        source: '髱',
        glyph_index: 7612,
    },
    MapEntry {
        target: '稣',
        source: '髴',
        glyph_index: 7611,
    },
    MapEntry {
        target: '怅',
        source: '悵',
        glyph_index: 5185,
    },
    MapEntry {
        target: '搅',
        source: '撹',
        glyph_index: 1732,
    },
    MapEntry {
        target: '咚',
        source: '髮',
        glyph_index: 7610,
    },
    MapEntry {
        target: '筷',
        source: '髣',
        glyph_index: 7606,
    },
    MapEntry {
        target: '嗡',
        source: '髢',
        glyph_index: 7605,
    },
    MapEntry {
        target: '蓦',
        source: '驀',
        glyph_index: 7581,
    },
    MapEntry {
        target: '鹊',
        source: '鵲',
        glyph_index: 7714,
    },
    MapEntry {
        target: '巢',
        source: '巣',
        glyph_index: 3074,
    },
    MapEntry {
        target: '莺',
        source: '鴬',
        glyph_index: 1606,
    },
    MapEntry {
        target: '硕',
        source: '碩',
        glyph_index: 2970,
    },
    MapEntry {
        target: '凿',
        source: '鑿',
        glyph_index: 7387,
    },
    MapEntry {
        target: '膛',
        source: '髞',
        glyph_index: 7603,
    },
    MapEntry {
        target: '蚀',
        source: '蝕',
        glyph_index: 2829,
    },
    MapEntry {
        target: '啤',
        source: '體',
        glyph_index: 7602,
    },
    MapEntry {
        target: '垫',
        source: '驫',
        glyph_index: 7593,
    },
    MapEntry {
        target: '嘟',
        source: '驗',
        glyph_index: 7587,
    },
    MapEntry {
        target: '窦',
        source: '竇',
        glyph_index: 6272,
    },
    MapEntry {
        target: '烘',
        source: '驛',
        glyph_index: 7586,
    },
    MapEntry {
        target: '酿',
        source: '釀',
        glyph_index: 7291,
    },
    MapEntry {
        target: '铮',
        source: '驅',
        glyph_index: 7579,
    },
    MapEntry {
        target: '鸠',
        source: '鳩',
        glyph_index: 3688,
    },
    MapEntry {
        target: '盏',
        source: '盞',
        glyph_index: 6132,
    },
    MapEntry {
        target: '拣',
        source: '騷',
        glyph_index: 7578,
    },
    MapEntry {
        target: '咧',
        source: '駲',
        glyph_index: 7569,
    },
    MapEntry {
        target: '绅',
        source: '紳',
        glyph_index: 2853,
    },
    MapEntry {
        target: '锤',
        source: '錘',
        glyph_index: 2897,
    },
    MapEntry {
        target: '揪',
        source: '駮',
        glyph_index: 7567,
    },
    MapEntry {
        target: '嚣',
        source: '囂',
        glyph_index: 4778,
    },
    MapEntry {
        target: '眶',
        source: '馼',
        glyph_index: 7560,
    },
    MapEntry {
        target: '垒',
        source: '塁',
        glyph_index: 4290,
    },
    MapEntry {
        target: '陕',
        source: '陜',
        glyph_index: 7424,
    },
    MapEntry {
        target: '琏',
        source: '饑',
        glyph_index: 7551,
    },
    MapEntry {
        target: '虏',
        source: '虜',
        glyph_index: 4255,
    },
    MapEntry {
        target: '勋',
        source: '勲',
        glyph_index: 2081,
    },
    MapEntry {
        target: '萝',
        source: '蘿',
        glyph_index: 6806,
    },
    MapEntry {
        target: '轴',
        source: '軸',
        glyph_index: 2557,
    },
    MapEntry {
        target: '垄',
        source: '壟',
        glyph_index: 4859,
    },
    MapEntry {
        target: '锻',
        source: '鍛',
        glyph_index: 3230,
    },
    MapEntry {
        target: '狈',
        source: '狽',
        glyph_index: 3637,
    },
    MapEntry {
        target: '诬',
        source: '誣',
        glyph_index: 7011,
    },
    MapEntry {
        target: '矫',
        source: '矯',
        glyph_index: 1999,
    },
    MapEntry {
        target: '敛',
        source: '歛',
        glyph_index: 5673,
    },
    MapEntry {
        target: '绷',
        source: '繃',
        glyph_index: 6453,
    },
    MapEntry {
        target: '滨',
        source: '濱',
        glyph_index: 5859,
    },
    MapEntry {
        target: '绰',
        source: '綽',
        glyph_index: 6422,
    },
    MapEntry {
        target: '娅',
        source: '讚',
        glyph_index: 7062,
    },
    MapEntry {
        target: '诛',
        source: '誅',
        glyph_index: 7002,
    },
    MapEntry {
        target: '瞄',
        source: '饐',
        glyph_index: 7549,
    },
    MapEntry {
        target: '祯',
        source: '禎',
        glyph_index: 3376,
    },
    MapEntry {
        target: '筠',
        source: '饂',
        glyph_index: 7546,
    },
    MapEntry {
        target: '蚁',
        source: '蟻',
        glyph_index: 1913,
    },
    MapEntry {
        target: '粪',
        source: '糞',
        glyph_index: 3874,
    },
    MapEntry {
        target: '汹',
        source: '洶',
        glyph_index: 5741,
    },
    MapEntry {
        target: '荫',
        source: '蔭',
        glyph_index: 1503,
    },
    MapEntry {
        target: '垮',
        source: '誇',
        glyph_index: 2217,
    },
    MapEntry {
        target: '诏',
        source: '詔',
        glyph_index: 2784,
    },
    MapEntry {
        target: '绒',
        source: '絨',
        glyph_index: 6407,
    },
    MapEntry {
        target: '咙',
        source: '龍',
        glyph_index: 4251,
    },
    MapEntry {
        target: '诅',
        source: '詛',
        glyph_index: 6994,
    },
    MapEntry {
        target: '捂',
        source: '餽',
        glyph_index: 7544,
    },
    MapEntry {
        target: '芜',
        source: '蕪',
        glyph_index: 3842,
    },
    MapEntry {
        target: '颐',
        source: '頤',
        glyph_index: 7510,
    },
    MapEntry {
        target: '辫',
        source: '辮',
        glyph_index: 6471,
    },
    MapEntry {
        target: '姗',
        source: '餬',
        glyph_index: 7542,
    },
    MapEntry {
        target: '阀',
        source: '閥',
        glyph_index: 3687,
    },
    MapEntry {
        target: '宓',
        source: '餠',
        glyph_index: 7541,
    },
    MapEntry {
        target: '摹',
        source: '餤',
        glyph_index: 7540,
    },
    MapEntry {
        target: '浊',
        source: '濁',
        glyph_index: 3190,
    },
    MapEntry {
        target: '惕',
        source: '餝',
        glyph_index: 7538,
    },
    MapEntry {
        target: '拎',
        source: '餘',
        glyph_index: 7536,
    },
    MapEntry {
        target: '薰',
        source: '薫',
        glyph_index: 2083,
    },
    MapEntry {
        target: '阱',
        source: '餔',
        glyph_index: 7535,
    },
    MapEntry {
        target: '笃',
        source: '篤',
        glyph_index: 3515,
    },
    MapEntry {
        target: '秽',
        source: '穢',
        glyph_index: 6253,
    },
    MapEntry {
        target: '砚',
        source: '硯',
        glyph_index: 2168,
    },
    MapEntry {
        target: '钓',
        source: '釣',
        glyph_index: 3353,
    },
    MapEntry {
        target: '蹦',
        source: '飃',
        glyph_index: 7528,
    },
    MapEntry {
        target: '搔',
        source: '掻',
        glyph_index: 3070,
    },
    MapEntry {
        target: '捞',
        source: '撈',
        glyph_index: 5356,
    },
    MapEntry {
        target: '袜',
        source: '襪',
        glyph_index: 6959,
    },
    MapEntry {
        target: '龚',
        source: '颱',
        glyph_index: 7525,
    },
    MapEntry {
        target: '觑',
        source: '颪',
        glyph_index: 7523,
    },
    MapEntry {
        target: '缎',
        source: '緞',
        glyph_index: 6434,
    },
    MapEntry {
        target: '噢',
        source: '顳',
        glyph_index: 7522,
    },
    MapEntry {
        target: '嗦',
        source: '顯',
        glyph_index: 7518,
    },
    MapEntry {
        target: '煜',
        source: '顋',
        glyph_index: 7516,
    },
    MapEntry {
        target: '酱',
        source: '醤',
        glyph_index: 2788,
    },
    MapEntry {
        target: '帧',
        source: '幀',
        glyph_index: 5057,
    },
    MapEntry {
        target: '帜',
        source: '幟',
        glyph_index: 5061,
    },
    MapEntry {
        target: '窍',
        source: '竅',
        glyph_index: 6268,
    },
    MapEntry {
        target: '噩',
        source: '顏',
        glyph_index: 7515,
    },
    MapEntry {
        target: '侨',
        source: '僑',
        glyph_index: 1976,
    },
    MapEntry {
        target: '谴',
        source: '譴',
        glyph_index: 7053,
    },
    MapEntry {
        target: '箫',
        source: '簫',
        glyph_index: 6349,
    },
    MapEntry {
        target: '鹃',
        source: '鵑',
        glyph_index: 7711,
    },
    MapEntry {
        target: '扳',
        source: '鈑',
        glyph_index: 7308,
    },
    MapEntry {
        target: '蝇',
        source: '蝿',
        glyph_index: 3643,
    },
    MapEntry {
        target: '悯',
        source: '憫',
        glyph_index: 5235,
    },
    MapEntry {
        target: '铅',
        source: '鉛',
        glyph_index: 1587,
    },
    MapEntry {
        target: '赎',
        source: '贖',
        glyph_index: 7111,
    },
    MapEntry {
        target: '惫',
        source: '憊',
        glyph_index: 5233,
    },
    MapEntry {
        target: '缀',
        source: '綴',
        glyph_index: 3343,
    },
    MapEntry {
        target: '绎',
        source: '繹',
        glyph_index: 6463,
    },
    MapEntry {
        target: '嚎',
        source: '頸',
        glyph_index: 7509,
    },
    MapEntry {
        target: '圳',
        source: '韲',
        glyph_index: 7503,
    },
    MapEntry {
        target: '绞',
        source: '絞',
        glyph_index: 2297,
    },
    MapEntry {
        target: '炜',
        source: '韈',
        glyph_index: 7498,
    },
    MapEntry {
        target: '诠',
        source: '詮',
        glyph_index: 3014,
    },
    MapEntry {
        target: '妞',
        source: '韆',
        glyph_index: 7497,
    },
    MapEntry {
        target: '瘾',
        source: '鞴',
        glyph_index: 7495,
    },
    MapEntry {
        target: '掐',
        source: '鞐',
        glyph_index: 7489,
    },
    MapEntry {
        target: '羁',
        source: '羈',
        glyph_index: 6504,
    },
    MapEntry {
        target: '锥',
        source: '錐',
        glyph_index: 2896,
    },
    MapEntry {
        target: '捆',
        source: '梱',
        glyph_index: 2362,
    },
    MapEntry {
        target: '渲',
        source: '鞦',
        glyph_index: 7492,
    },
    MapEntry {
        target: '诫',
        source: '誡',
        glyph_index: 7006,
    },
    MapEntry {
        target: '颁',
        source: '頒',
        glyph_index: 3715,
    },
    MapEntry {
        target: '谛',
        source: '諦',
        glyph_index: 3381,
    },
    MapEntry {
        target: '翘',
        source: '翹',
        glyph_index: 6528,
    },
    MapEntry {
        target: '嘎',
        source: '鞨',
        glyph_index: 7491,
    },
    MapEntry {
        target: '浇',
        source: '澆',
        glyph_index: 5831,
    },
    MapEntry {
        target: '狞',
        source: '獰',
        glyph_index: 5977,
    },
    MapEntry {
        target: '炽',
        source: '熾',
        glyph_index: 5912,
    },
    MapEntry {
        target: '涟',
        source: '漣',
        glyph_index: 4319,
    },
    MapEntry {
        target: '聋',
        source: '聾',
        glyph_index: 4347,
    },
    MapEntry {
        target: '绊',
        source: '絆',
        glyph_index: 6402,
    },
    MapEntry {
        target: '泻',
        source: '瀉',
        glyph_index: 5862,
    },
    MapEntry {
        target: '衮',
        source: '袞',
        glyph_index: 6910,
    },
    MapEntry {
        target: '诃',
        source: '訶',
        glyph_index: 6992,
    },
    MapEntry {
        target: '橱',
        source: '鞜',
        glyph_index: 7490,
    },
    MapEntry {
        target: '瘫',
        source: '鞆',
        glyph_index: 7486,
    },
    MapEntry {
        target: '忏',
        source: '懴',
        glyph_index: 5252,
    },
    MapEntry {
        target: '睬',
        source: '靺',
        glyph_index: 7485,
    },
    MapEntry {
        target: '舔',
        source: '鞁',
        glyph_index: 7484,
    },
    MapEntry {
        target: '沅',
        source: '靹',
        glyph_index: 7481,
    },
    MapEntry {
        target: '泸',
        source: '瀘',
        glyph_index: 5873,
    },
    MapEntry {
        target: '疤',
        source: '靱',
        glyph_index: 7480,
    },
    MapEntry {
        target: '惮',
        source: '憚',
        glyph_index: 5232,
    },
    MapEntry {
        target: '晖',
        source: '暉',
        glyph_index: 5443,
    },
    MapEntry {
        target: '镶',
        source: '驤',
        glyph_index: 7591,
    },
    MapEntry {
        target: '挠',
        source: '撓',
        glyph_index: 5353,
    },
    MapEntry {
        target: '谕',
        source: '諭',
        glyph_index: 4136,
    },
    MapEntry {
        target: '澈',
        source: '靤',
        glyph_index: 7475,
    },
    MapEntry {
        target: '噜',
        source: '櫓',
        glyph_index: 4329,
    },
    MapEntry {
        target: '鳄',
        source: '鰐',
        glyph_index: 4367,
    },
    MapEntry {
        target: '噗',
        source: '靜',
        glyph_index: 7473,
    },
    MapEntry {
        target: '闵',
        source: '閔',
        glyph_index: 7391,
    },
    MapEntry {
        target: '辙',
        source: '轍',
        glyph_index: 3401,
    },
    MapEntry {
        target: '珑',
        source: '瓏',
        glyph_index: 6013,
    },
    MapEntry {
        target: '谏',
        source: '諌',
        glyph_index: 1835,
    },
    MapEntry {
        target: '呸',
        source: '靉',
        glyph_index: 7472,
    },
    MapEntry {
        target: '虾',
        source: '蝦',
        glyph_index: 1657,
    },
    MapEntry {
        target: '婧',
        source: '靈',
        glyph_index: 7470,
    },
    MapEntry {
        target: '跄',
        source: '蹌',
        glyph_index: 7146,
    },
    MapEntry {
        target: '娓',
        source: '靆',
        glyph_index: 7469,
    },
    MapEntry {
        target: '揽',
        source: '霤',
        glyph_index: 7462,
    },
    MapEntry {
        target: '迳',
        source: '逕',
        glyph_index: 7227,
    },
    MapEntry {
        target: '璐',
        source: '霙',
        glyph_index: 7461,
    },
    MapEntry {
        target: '兑',
        source: '兌',
        glyph_index: 4539,
    },
    MapEntry {
        target: '惋',
        source: '輓',
        glyph_index: 7189,
    },
    MapEntry {
        target: '唬',
        source: '雜',
        glyph_index: 7449,
    },
    MapEntry {
        target: '讼',
        source: '訟',
        glyph_index: 2782,
    },
    MapEntry {
        target: '嘀',
        source: '襍',
        glyph_index: 7448,
    },
    MapEntry {
        target: '雯',
        source: '隸',
        glyph_index: 7442,
    },
    MapEntry {
        target: '韧',
        source: '靭',
        glyph_index: 2876,
    },
    MapEntry {
        target: '鬓',
        source: '鬢',
        glyph_index: 7619,
    },
    MapEntry {
        target: '匀',
        source: '隱',
        glyph_index: 7437,
    },
    MapEntry {
        target: '阙',
        source: '闕',
        glyph_index: 7408,
    },
    MapEntry {
        target: '贿',
        source: '賄',
        glyph_index: 4360,
    },
    MapEntry {
        target: '崭',
        source: '嶄',
        glyph_index: 5029,
    },
    MapEntry {
        target: '轼',
        source: '軾',
        glyph_index: 7183,
    },
    MapEntry {
        target: '盔',
        source: '險',
        glyph_index: 7435,
    },
    MapEntry {
        target: '蚂',
        source: '陦',
        glyph_index: 7428,
    },
    MapEntry {
        target: '抨',
        source: '陝',
        glyph_index: 7426,
    },
    MapEntry {
        target: '鳞',
        source: '鱗',
        glyph_index: 4287,
    },
    MapEntry {
        target: '嬷',
        source: '陞',
        glyph_index: 7425,
    },
    MapEntry {
        target: '钝',
        source: '鈍',
        glyph_index: 3540,
    },
    MapEntry {
        target: '浒',
        source: '滸',
        glyph_index: 5820,
    },
    MapEntry {
        target: '啧',
        source: '嘖',
        glyph_index: 4751,
    },
    MapEntry {
        target: '闽',
        source: '陏',
        glyph_index: 7421,
    },
    MapEntry {
        target: '屉',
        source: '阯',
        glyph_index: 7418,
    },
    MapEntry {
        target: '鞑',
        source: '韃',
        glyph_index: 7496,
    },
    MapEntry {
        target: '鳌',
        source: '鰲',
        glyph_index: 7679,
    },
    MapEntry {
        target: '铲',
        source: '阨',
        glyph_index: 7416,
    },
    MapEntry {
        target: '呛',
        source: '闢',
        glyph_index: 7414,
    },
    MapEntry {
        target: '恺',
        source: '關',
        glyph_index: 7411,
    },
    MapEntry {
        target: '拽',
        source: '闍',
        glyph_index: 7406,
    },
    MapEntry {
        target: '驯',
        source: '馴',
        glyph_index: 3552,
    },
    MapEntry {
        target: '肮',
        source: '濶',
        glyph_index: 7404,
    },
    MapEntry {
        target: '憋',
        source: '閧',
        glyph_index: 7397,
    },
    MapEntry {
        target: '颔',
        source: '頷',
        glyph_index: 7512,
    },
    MapEntry {
        target: '踌',
        source: '躊',
        glyph_index: 7163,
    },
    MapEntry {
        target: '邢',
        source: '閠',
        glyph_index: 7395,
    },
    MapEntry {
        target: '娄',
        source: '婁',
        glyph_index: 4335,
    },
    MapEntry {
        target: '踱',
        source: '鈬',
        glyph_index: 7306,
    },
    MapEntry {
        target: '炫',
        source: '鉉',
        glyph_index: 7312,
    },
    MapEntry {
        target: '喽',
        source: '閙',
        glyph_index: 7394,
    },
    MapEntry {
        target: '鸽',
        source: '鴿',
        glyph_index: 7704,
    },
    MapEntry {
        target: '飕',
        source: '閖',
        glyph_index: 7392,
    },
    MapEntry {
        target: '汴',
        source: '閊',
        glyph_index: 7390,
    },
    MapEntry {
        target: '涡',
        source: '渦',
        glyph_index: 1521,
    },
    MapEntry {
        target: '慑',
        source: '懾',
        glyph_index: 5256,
    },
    MapEntry {
        target: '讪',
        source: '閇',
        glyph_index: 7389,
    },
    MapEntry {
        target: '氧',
        source: '钁',
        glyph_index: 7386,
    },
    MapEntry {
        target: '驿',
        source: '駅',
        glyph_index: 1559,
    },
    MapEntry {
        target: '棱',
        source: '薐',
        glyph_index: 6786,
    },
    MapEntry {
        target: '篱',
        source: '籬',
        glyph_index: 6362,
    },
    MapEntry {
        target: '萤',
        source: '蛍',
        glyph_index: 2121,
    },
    MapEntry {
        target: '辗',
        source: '輾',
        glyph_index: 7200,
    },
    MapEntry {
        target: '啃',
        source: '墾',
        glyph_index: 2355,
    },
    MapEntry {
        target: '婶',
        source: '鑽',
        glyph_index: 7382,
    },
    MapEntry {
        target: '谟',
        source: '謨',
        glyph_index: 7040,
    },
    MapEntry {
        target: '颅',
        source: '顱',
        glyph_index: 7520,
    },
    MapEntry {
        target: '缴',
        source: '鑵',
        glyph_index: 7380,
    },
    MapEntry {
        target: '扒',
        source: '釟',
        glyph_index: 7297,
    },
    MapEntry {
        target: '俨',
        source: '儼',
        glyph_index: 4534,
    },
    MapEntry {
        target: '拧',
        source: '檸',
        glyph_index: 5645,
    },
    MapEntry {
        target: '栈',
        source: '桟',
        glyph_index: 2466,
    },
    MapEntry {
        target: '绢',
        source: '絹',
        glyph_index: 2169,
    },
    MapEntry {
        target: '谤',
        source: '謗',
        glyph_index: 7033,
    },
    MapEntry {
        target: '灶',
        source: '竃',
        glyph_index: 1777,
    },
    MapEntry {
        target: '哆',
        source: '鈩',
        glyph_index: 7378,
    },
    MapEntry {
        target: '诣',
        source: '詣',
        glyph_index: 2123,
    },
    MapEntry {
        target: '俭',
        source: '倹',
        glyph_index: 2147,
    },
    MapEntry {
        target: '踹',
        source: '鑪',
        glyph_index: 7377,
    },
    MapEntry {
        target: '栅',
        source: '柵',
        glyph_index: 2433,
    },
    MapEntry {
        target: '馒',
        source: '饅',
        glyph_index: 7548,
    },
    MapEntry {
        target: '辖',
        source: '轄',
        glyph_index: 1768,
    },
    MapEntry {
        target: '枣',
        source: '棗',
        glyph_index: 5552,
    },
    MapEntry {
        target: '叽',
        source: '鑢',
        glyph_index: 7375,
    },
    MapEntry {
        target: '袅',
        source: '嫋',
        glyph_index: 4917,
    },
    MapEntry {
        target: '钊',
        source: '鑛',
        glyph_index: 7373,
    },
    MapEntry {
        target: '瞅',
        source: '鑄',
        glyph_index: 7372,
    },
    MapEntry {
        target: '赘',
        source: '贅',
        glyph_index: 7101,
    },
    MapEntry {
        target: '漩',
        source: '鑒',
        glyph_index: 7371,
    },
    MapEntry {
        target: '婊',
        source: '鑁',
        glyph_index: 7370,
    },
    MapEntry {
        target: '盎',
        source: '鐡',
        glyph_index: 7368,
    },
    MapEntry {
        target: '揍',
        source: '輳',
        glyph_index: 7195,
    },
    MapEntry {
        target: '阑',
        source: '闌',
        glyph_index: 7407,
    },
    MapEntry {
        target: '纨',
        source: '鐵',
        glyph_index: 7367,
    },
    MapEntry {
        target: '唠',
        source: '癆',
        glyph_index: 6099,
    },
    MapEntry {
        target: '缅',
        source: '緬',
        glyph_index: 4084,
    },
    MapEntry {
        target: '佬',
        source: '鐶',
        glyph_index: 7365,
    },
    MapEntry {
        target: '皂',
        source: '鹸',
        glyph_index: 2181,
    },
    MapEntry {
        target: '辄',
        source: '輒',
        glyph_index: 7187,
    },
    MapEntry {
        target: '驭',
        source: '馭',
        glyph_index: 7558,
    },
    MapEntry {
        target: '韬',
        source: '韜',
        glyph_index: 7500,
    },
    MapEntry {
        target: '唧',
        source: '喞',
        glyph_index: 4735,
    },
    MapEntry {
        target: '糙',
        source: '鐇',
        glyph_index: 7363,
    },
    MapEntry {
        target: '炕',
        source: '鐓',
        glyph_index: 7361,
    },
    MapEntry {
        target: '亩',
        source: '畝',
        glyph_index: 2919,
    },
    MapEntry {
        target: '呃',
        source: '鐔',
        glyph_index: 7360,
    },
    MapEntry {
        target: '唷',
        source: '鐚',
        glyph_index: 7359,
    },
    MapEntry {
        target: '缭',
        source: '繚',
        glyph_index: 6462,
    },
    MapEntry {
        target: '绛',
        source: '絳',
        glyph_index: 6403,
    },
    MapEntry {
        target: '侥',
        source: '僥',
        glyph_index: 4518,
    },
    MapEntry {
        target: '诲',
        source: '誨',
        glyph_index: 7005,
    },
    MapEntry {
        target: '槛',
        source: '檻',
        glyph_index: 5642,
    },
    MapEntry {
        target: '亵',
        source: '褻',
        glyph_index: 6949,
    },
    MapEntry {
        target: '馈',
        source: '饋',
        glyph_index: 7550,
    },
    MapEntry {
        target: '坞',
        source: '塢',
        glyph_index: 4834,
    },
    MapEntry {
        target: '碳',
        source: '鏝',
        glyph_index: 7355,
    },
    MapEntry {
        target: '纣',
        source: '紂',
        glyph_index: 6392,
    },
    MapEntry {
        target: '吨',
        source: '噸',
        glyph_index: 3530,
    },
    MapEntry {
        target: '抿',
        source: '鏥',
        glyph_index: 7352,
    },
    MapEntry {
        target: '铐',
        source: '鎹',
        glyph_index: 7348,
    },
    MapEntry {
        target: '渎',
        source: '涜',
        glyph_index: 3511,
    },
    MapEntry {
        target: '谍',
        source: '諜',
        glyph_index: 3310,
    },
    MapEntry {
        target: '锯',
        source: '鋸',
        glyph_index: 1967,
    },
    MapEntry {
        target: '坷',
        source: '鎔',
        glyph_index: 7347,
    },
    MapEntry {
        target: '邹',
        source: '鄒',
        glyph_index: 7269,
    },
    MapEntry {
        target: '讹',
        source: '訛',
        glyph_index: 6989,
    },
    MapEntry {
        target: '铎',
        source: '鐸',
        glyph_index: 3189,
    },
    MapEntry {
        target: '蔷',
        source: '薔',
        glyph_index: 6779,
    },
    MapEntry {
        target: '浏',
        source: '瀏',
        glyph_index: 5867,
    },
    MapEntry {
        target: '蹬',
        source: '鐙',
        glyph_index: 3487,
    },
    MapEntry {
        target: '攒',
        source: '攅',
        glyph_index: 5382,
    },
    MapEntry {
        target: '纺',
        source: '紡',
        glyph_index: 3981,
    },
    MapEntry {
        target: '榷',
        source: '鎭',
        glyph_index: 7346,
    },
    MapEntry {
        target: '吆',
        source: '鍖',
        glyph_index: 7343,
    },
    MapEntry {
        target: '隽',
        source: '雋',
        glyph_index: 7445,
    },
    MapEntry {
        target: '恻',
        source: '惻',
        glyph_index: 5197,
    },
    MapEntry {
        target: '纭',
        source: '紜',
        glyph_index: 6393,
    },
    MapEntry {
        target: '嬴',
        source: '鍮',
        glyph_index: 7342,
    },
    MapEntry {
        target: '萦',
        source: '鍼',
        glyph_index: 7341,
    },
    MapEntry {
        target: '嗳',
        source: '鍠',
        glyph_index: 7340,
    },
    MapEntry {
        target: '钮',
        source: '鍜',
        glyph_index: 7339,
    },
    MapEntry {
        target: '纬',
        source: '緯',
        glyph_index: 1471,
    },
    MapEntry {
        target: '狰',
        source: '錻',
        glyph_index: 7338,
    },
    MapEntry {
        target: '榆',
        source: '楡',
        glyph_index: 5577,
    },
    MapEntry {
        target: '疮',
        source: '瘡',
        glyph_index: 6091,
    },
    MapEntry {
        target: '搀',
        source: '讒',
        glyph_index: 7058,
    },
    MapEntry {
        target: '锣',
        source: '鑼',
        glyph_index: 7384,
    },
    MapEntry {
        target: '缉',
        source: '緝',
        glyph_index: 6432,
    },
    MapEntry {
        target: '蹑',
        source: '躡',
        glyph_index: 7169,
    },
    MapEntry {
        target: '掳',
        source: '錵',
        glyph_index: 7337,
    },
    MapEntry {
        target: '簌',
        source: '錺',
        glyph_index: 7336,
    },
    MapEntry {
        target: '衅',
        source: '釁',
        glyph_index: 7292,
    },
    MapEntry {
        target: '绥',
        source: '綏',
        glyph_index: 6414,
    },
    MapEntry {
        target: '瞩',
        source: '矚',
        glyph_index: 6174,
    },
    MapEntry {
        target: '桨',
        source: '錣',
        glyph_index: 7335,
    },
    MapEntry {
        target: '驹',
        source: '駒',
        glyph_index: 2053,
    },
    MapEntry {
        target: '瞟',
        source: '錢',
        glyph_index: 7333,
    },
    MapEntry {
        target: '喳',
        source: '鍄',
        glyph_index: 7330,
    },
    MapEntry {
        target: '蛊',
        source: '蠱',
        glyph_index: 6896,
    },
    MapEntry {
        target: '蹭',
        source: '鋺',
        glyph_index: 7329,
    },
    MapEntry {
        target: '啷',
        source: '錏',
        glyph_index: 7328,
    },
    MapEntry {
        target: '湄',
        source: '鋩',
        glyph_index: 7327,
    },
    MapEntry {
        target: '哧',
        source: '銛',
        glyph_index: 7322,
    },
    MapEntry {
        target: '嘈',
        source: '獎',
        glyph_index: 5971,
    },
    MapEntry {
        target: '缔',
        source: '締',
        glyph_index: 3378,
    },
    MapEntry {
        target: '癫',
        source: '癲',
        glyph_index: 6110,
    },
    MapEntry {
        target: '沥',
        source: '瀝',
        glyph_index: 5872,
    },
    MapEntry {
        target: '刁',
        source: '鉐',
        glyph_index: 7318,
    },
    MapEntry {
        target: '哝',
        source: '儂',
        glyph_index: 4526,
    },
    MapEntry {
        target: '鸥',
        source: '鴎',
        glyph_index: 1607,
    },
    MapEntry {
        target: '锄',
        source: '鋤',
        glyph_index: 2722,
    },
    MapEntry {
        target: '崽',
        source: '銕',
        glyph_index: 7315,
    },
    MapEntry {
        target: '骷',
        source: '鉈',
        glyph_index: 7314,
    },
    MapEntry {
        target: '忡',
        source: '鉅',
        glyph_index: 7311,
    },
    MapEntry {
        target: '掺',
        source: '參',
        glyph_index: 4657,
    },
    MapEntry {
        target: '袄',
        source: '襖',
        glyph_index: 1605,
    },
    MapEntry {
        target: '诘',
        source: '詰',
        glyph_index: 1924,
    },
    MapEntry {
        target: '嗒',
        source: '釿',
        glyph_index: 7304,
    },
    MapEntry {
        target: '贻',
        source: '貽',
        glyph_index: 7087,
    },
    MapEntry {
        target: '雳',
        source: '靂',
        glyph_index: 7471,
    },
    MapEntry {
        target: '烬',
        source: '燼',
        glyph_index: 5921,
    },
    MapEntry {
        target: '酝',
        source: '釶',
        glyph_index: 7302,
    },
    MapEntry {
        target: '苇',
        source: '葦',
        glyph_index: 1426,
    },
    MapEntry {
        target: '啬',
        source: '嗇',
        glyph_index: 4800,
    },
    MapEntry {
        target: '茧',
        source: '繭',
        glyph_index: 4037,
    },
    MapEntry {
        target: '颉',
        source: '頡',
        glyph_index: 7511,
    },
    MapEntry {
        target: '篡',
        source: '簒',
        glyph_index: 4658,
    },
    MapEntry {
        target: '褚',
        source: '釼',
        glyph_index: 7300,
    },
    MapEntry {
        target: '锢',
        source: '錮',
        glyph_index: 7331,
    },
    MapEntry {
        target: '蜕',
        source: '蛻',
        glyph_index: 6839,
    },
    MapEntry {
        target: '砆',
        source: '釛',
        glyph_index: 7299,
    },
    MapEntry {
        target: '碔',
        source: '釡',
        glyph_index: 7298,
    },
    MapEntry {
        target: '偎',
        source: '釖',
        glyph_index: 7296,
    },
    MapEntry {
        target: '饵',
        source: '餌',
        glyph_index: 1537,
    },
    MapEntry {
        target: '钳',
        source: '鉗',
        glyph_index: 7310,
    },
    MapEntry {
        target: '疙',
        source: '釐',
        glyph_index: 7295,
    },
    MapEntry {
        target: '阖',
        source: '闔',
        glyph_index: 7409,
    },
    MapEntry {
        target: '诂',
        source: '詁',
        glyph_index: 6993,
    },
    MapEntry {
        target: '绫',
        source: '綾',
        glyph_index: 1438,
    },
    MapEntry {
        target: '鲸',
        source: '鯨',
        glyph_index: 2130,
    },
    MapEntry {
        target: '蕙',
        source: '釋',
        glyph_index: 7294,
    },
    MapEntry {
        target: '璜',
        source: '醯',
        glyph_index: 7286,
    },
    MapEntry {
        target: '瘩',
        source: '醫',
        glyph_index: 7285,
    },
    MapEntry {
        target: '摒',
        source: '醂',
        glyph_index: 7283,
    },
    MapEntry {
        target: '谒',
        source: '謁',
        glyph_index: 1561,
    },
    MapEntry {
        target: '娴',
        source: '嫻',
        glyph_index: 4926,
    },
    MapEntry {
        target: '嵇',
        source: '酳',
        glyph_index: 7279,
    },
    MapEntry {
        target: '缰',
        source: '酘',
        glyph_index: 7275,
    },
    MapEntry {
        target: '轶',
        source: '軼',
        glyph_index: 7180,
    },
    MapEntry {
        target: '骋',
        source: '騁',
        glyph_index: 7572,
    },
    MapEntry {
        target: '烨',
        source: '酖',
        glyph_index: 7274,
    },
    MapEntry {
        target: '怆',
        source: '愴',
        glyph_index: 5209,
    },
    MapEntry {
        target: '髅',
        source: '髏',
        glyph_index: 7599,
    },
    MapEntry {
        target: '涧',
        source: '澗',
        glyph_index: 1819,
    },
    MapEntry {
        target: '烩',
        source: '邉',
        glyph_index: 7258,
    },
    MapEntry {
        target: '偌',
        source: '遲',
        glyph_index: 7252,
    },
    MapEntry {
        target: '肾',
        source: '腎',
        glyph_index: 2872,
    },
    MapEntry {
        target: '跺',
        source: '隨',
        glyph_index: 7251,
    },
    MapEntry {
        target: '毡',
        source: '氈',
        glyph_index: 5704,
    },
    MapEntry {
        target: '巅',
        source: '巓',
        glyph_index: 5041,
    },
    MapEntry {
        target: '玺',
        source: '璽',
        glyph_index: 2542,
    },
    MapEntry {
        target: '钵',
        source: '鉢',
        glyph_index: 3678,
    },
    MapEntry {
        target: '辕',
        source: '轅',
        glyph_index: 7198,
    },
    MapEntry {
        target: '绚',
        source: '絢',
        glyph_index: 1437,
    },
    MapEntry {
        target: '飙',
        source: '飆',
        glyph_index: 7529,
    },
    MapEntry {
        target: '愫',
        source: '遶',
        glyph_index: 7250,
    },
    MapEntry {
        target: '赃',
        source: '賍',
        glyph_index: 7109,
    },
    MapEntry {
        target: '渍',
        source: '漬',
        glyph_index: 3339,
    },
    MapEntry {
        target: '瑷',
        source: '遯',
        glyph_index: 7249,
    },
    MapEntry {
        target: '憨',
        source: '遞',
        glyph_index: 7247,
    },
    MapEntry {
        target: '谚',
        source: '諺',
        glyph_index: 2194,
    },
    MapEntry {
        target: '诽',
        source: '誹',
        glyph_index: 3745,
    },
    MapEntry {
        target: '骡',
        source: '騾',
        glyph_index: 7583,
    },
    MapEntry {
        target: '蛰',
        source: '蟄',
        glyph_index: 6876,
    },
    MapEntry {
        target: '缥',
        source: '縹',
        glyph_index: 6452,
    },
    MapEntry {
        target: '荃',
        source: '遖',
        glyph_index: 7245,
    },
    MapEntry {
        target: '擎',
        source: '遉',
        glyph_index: 7243,
    },
    MapEntry {
        target: '楣',
        source: '逎',
        glyph_index: 7242,
    },
    MapEntry {
        target: '捅',
        source: '逹',
        glyph_index: 7236,
    },
    MapEntry {
        target: '癞',
        source: '癩',
        glyph_index: 6105,
    },
    MapEntry {
        target: '蔼',
        source: '藹',
        glyph_index: 6796,
    },
    MapEntry {
        target: '乓',
        source: '逧',
        glyph_index: 7233,
    },
    MapEntry {
        target: '籁',
        source: '藾',
        glyph_index: 6800,
    },
    MapEntry {
        target: '窑',
        source: '窯',
        glyph_index: 4185,
    },
    MapEntry {
        target: '晗',
        source: '迴',
        glyph_index: 7222,
    },
    MapEntry {
        target: '悻',
        source: '邇',
        glyph_index: 7221,
    },
    MapEntry {
        target: '铛',
        source: '鐺',
        glyph_index: 7369,
    },
    MapEntry {
        target: '跷',
        source: '磽',
        glyph_index: 6206,
    },
    MapEntry {
        target: '歼',
        source: '殱',
        glyph_index: 5691,
    },
    MapEntry {
        target: '濒',
        source: '瀕',
        glyph_index: 3805,
    },
    MapEntry {
        target: '琦',
        source: '迯',
        glyph_index: 7220,
    },
    MapEntry {
        target: '睽',
        source: '迚',
        glyph_index: 7216,
    },
    MapEntry {
        target: '滢',
        source: '辷',
        glyph_index: 7215,
    },
    MapEntry {
        target: '骈',
        source: '駢',
        glyph_index: 7575,
    },
    MapEntry {
        target: '忒',
        source: '辭',
        glyph_index: 7213,
    },
    MapEntry {
        target: '镑',
        source: '轣',
        glyph_index: 7208,
    },
    MapEntry {
        target: '雏',
        source: '雛',
        glyph_index: 2906,
    },
    MapEntry {
        target: '啰',
        source: '轢',
        glyph_index: 7207,
    },
    MapEntry {
        target: '伫',
        source: '佇',
        glyph_index: 4459,
    },
    MapEntry {
        target: '谑',
        source: '謔',
        glyph_index: 7021,
    },
    MapEntry {
        target: '缈',
        source: '緲',
        glyph_index: 6436,
    },
    MapEntry {
        target: '涤',
        source: '滌',
        glyph_index: 5827,
    },
    MapEntry {
        target: '匾',
        source: '轜',
        glyph_index: 7206,
    },
    MapEntry {
        target: '锈',
        source: '銹',
        glyph_index: 7325,
    },
    MapEntry {
        target: '炅',
        source: '轗',
        glyph_index: 7205,
    },
    MapEntry {
        target: '乒',
        source: '轉',
        glyph_index: 7202,
    },
    MapEntry {
        target: '蹋',
        source: '轌',
        glyph_index: 7201,
    },
    MapEntry {
        target: '鲨',
        source: '鯊',
        glyph_index: 7647,
    },
    MapEntry {
        target: '谙',
        source: '諳',
        glyph_index: 7017,
    },
    MapEntry {
        target: '戾',
        source: '輹',
        glyph_index: 7197,
    },
    MapEntry {
        target: '谆',
        source: '諄',
        glyph_index: 7012,
    },
    MapEntry {
        target: '矶',
        source: '磯',
        glyph_index: 1484,
    },
    MapEntry {
        target: '霭',
        source: '靄',
        glyph_index: 7468,
    },
    MapEntry {
        target: '飒',
        source: '颯',
        glyph_index: 7524,
    },
    MapEntry {
        target: '谪',
        source: '謫',
        glyph_index: 7038,
    },
    MapEntry {
        target: '痪',
        source: '渙',
        glyph_index: 5783,
    },
    MapEntry {
        target: '跤',
        source: '輛',
        glyph_index: 7192,
    },
    MapEntry {
        target: '拴',
        source: '輙',
        glyph_index: 7188,
    },
    MapEntry {
        target: '嗨',
        source: '軣',
        glyph_index: 7179,
    },
    MapEntry {
        target: '礴',
        source: '軈',
        glyph_index: 7176,
    },
    MapEntry {
        target: '屿',
        source: '嶼',
        glyph_index: 5038,
    },
    MapEntry {
        target: '赂',
        source: '賂',
        glyph_index: 4331,
    },
    MapEntry {
        target: '涔',
        source: '軅',
        glyph_index: 7175,
    },
    MapEntry {
        target: '啐',
        source: '躾',
        glyph_index: 7174,
    },
    MapEntry {
        target: '暧',
        source: '曖',
        glyph_index: 5454,
    },
    MapEntry {
        target: '纾',
        source: '軆',
        glyph_index: 7172,
    },
    MapEntry {
        target: '鸾',
        source: '鸞',
        glyph_index: 7741,
    },
    MapEntry {
        target: '轧',
        source: '軋',
        glyph_index: 7177,
    },
    MapEntry {
        target: '绯',
        source: '緋',
        glyph_index: 3741,
    },
    MapEntry {
        target: '坝',
        source: '躰',
        glyph_index: 7171,
    },
    MapEntry {
        target: '贰',
        source: '貳',
        glyph_index: 7089,
    },
    MapEntry {
        target: '铿',
        source: '鏗',
        glyph_index: 7350,
    },
    MapEntry {
        target: '嚅',
        source: '躙',
        glyph_index: 7167,
    },
    MapEntry {
        target: '翟',
        source: '躔',
        glyph_index: 7166,
    },
    MapEntry {
        target: '纶',
        source: '綸',
        glyph_index: 6428,
    },
    MapEntry {
        target: '抡',
        source: '躄',
        glyph_index: 7161,
    },
    MapEntry {
        target: '鹦',
        source: '鸚',
        glyph_index: 7739,
    },
    MapEntry {
        target: '闸',
        source: '閘',
        glyph_index: 7393,
    },
    MapEntry {
        target: '搐',
        source: '蹠',
        glyph_index: 7151,
    },
    MapEntry {
        target: '滤',
        source: '濾',
        glyph_index: 5868,
    },
    MapEntry {
        target: '彤',
        source: '蹤',
        glyph_index: 7150,
    },
    MapEntry {
        target: '榨',
        source: '搾',
        glyph_index: 2430,
    },
    MapEntry {
        target: '胭',
        source: '蹐',
        glyph_index: 7147,
    },
    MapEntry {
        target: '镯',
        source: '踴',
        glyph_index: 7142,
    },
    MapEntry {
        target: '魇',
        source: '魘',
        glyph_index: 7635,
    },
    MapEntry {
        target: '陇',
        source: '踰',
        glyph_index: 7141,
    },
    MapEntry {
        target: '恸',
        source: '慟',
        glyph_index: 5223,
    },
    MapEntry {
        target: '呗',
        source: '唄',
        glyph_index: 1523,
    },
    MapEntry {
        target: '墩',
        source: '踐',
        glyph_index: 7137,
    },
    MapEntry {
        target: '枭',
        source: '梟',
        glyph_index: 5521,
    },
    MapEntry {
        target: '贮',
        source: '貯',
        glyph_index: 3284,
    },
    MapEntry {
        target: '瘪',
        source: '跿',
        glyph_index: 7134,
    },
    MapEntry {
        target: '惦',
        source: '踈',
        glyph_index: 7132,
    },
    MapEntry {
        target: '缨',
        source: '纓',
        glyph_index: 6478,
    },
    MapEntry {
        target: '攥',
        source: '跼',
        glyph_index: 7131,
    },
    MapEntry {
        target: '靶',
        source: '赱',
        glyph_index: 7114,
    },
    MapEntry {
        target: '姒',
        source: '贔',
        glyph_index: 7110,
    },
    MapEntry {
        target: '潆',
        source: '贓',
        glyph_index: 7108,
    },
    MapEntry {
        target: '妩',
        source: '贐',
        glyph_index: 7106,
    },
    MapEntry {
        target: '夔',
        source: '贊',
        glyph_index: 7102,
    },
    MapEntry {
        target: '匮',
        source: '匱',
        glyph_index: 4633,
    },
    MapEntry {
        target: '脍',
        source: '膾',
        glyph_index: 6605,
    },
    MapEntry {
        target: '汩',
        source: '賻',
        glyph_index: 7099,
    },
    MapEntry {
        target: '辍',
        source: '輟',
        glyph_index: 7191,
    },
    MapEntry {
        target: '筛',
        source: '篩',
        glyph_index: 6330,
    },
    MapEntry {
        target: '锭',
        source: '錠',
        glyph_index: 2816,
    },
    MapEntry {
        target: '睐',
        source: '賣',
        glyph_index: 7095,
    },
    MapEntry {
        target: '欤',
        source: '歟',
        glyph_index: 5674,
    },
    MapEntry {
        target: '翎',
        source: '賤',
        glyph_index: 7094,
    },
    MapEntry {
        target: '咝',
        source: '貮',
        glyph_index: 7090,
    },
    MapEntry {
        target: '挞',
        source: '撻',
        glyph_index: 5362,
    },
    MapEntry {
        target: '讷',
        source: '訥',
        glyph_index: 6991,
    },
    MapEntry {
        target: '嗖',
        source: '貭',
        glyph_index: 7085,
    },
    MapEntry {
        target: '饲',
        source: '飼',
        glyph_index: 2527,
    },
    MapEntry {
        target: '邺',
        source: '戝',
        glyph_index: 7084,
    },
    MapEntry {
        target: '脓',
        source: '膿',
        glyph_index: 3602,
    },
    MapEntry {
        target: '钏',
        source: '釧',
        glyph_index: 2065,
    },
    MapEntry {
        target: '漪',
        source: '豼',
        glyph_index: 7082,
    },
    MapEntry {
        target: '喵',
        source: '貎',
        glyph_index: 7080,
    },
    MapEntry {
        target: '洼',
        source: '貍',
        glyph_index: 7079,
    },
    MapEntry {
        target: '玷',
        source: '霑',
        glyph_index: 7458,
    },
    MapEntry {
        target: '诙',
        source: '詼',
        glyph_index: 6998,
    },
    MapEntry {
        target: '缆',
        source: '纜',
        glyph_index: 6483,
    },
    MapEntry {
        target: '霁',
        source: '霽',
        glyph_index: 7466,
    },
    MapEntry {
        target: '饺',
        source: '餃',
        glyph_index: 7532,
    },
    MapEntry {
        target: '殡',
        source: '殯',
        glyph_index: 5689,
    },
    MapEntry {
        target: '荧',
        source: '塋',
        glyph_index: 4835,
    },
    MapEntry {
        target: '闩',
        source: '閂',
        glyph_index: 7388,
    },
    MapEntry {
        target: '鹗',
        source: '鶚',
        glyph_index: 7720,
    },
    MapEntry {
        target: '锚',
        source: '錨',
        glyph_index: 3796,
    },
    MapEntry {
        target: '缤',
        source: '繽',
        glyph_index: 6470,
    },
    MapEntry {
        target: '锵',
        source: '鏘',
        glyph_index: 7353,
    },
    MapEntry {
        target: '弈',
        source: '豬',
        glyph_index: 7072,
    },
    MapEntry {
        target: '陨',
        source: '隕',
        glyph_index: 7433,
    },
    MapEntry {
        target: '驸',
        source: '豎',
        glyph_index: 7068,
    },
    MapEntry {
        target: '晾',
        source: '谺',
        glyph_index: 7063,
    },
    MapEntry {
        target: '撂',
        source: '讙',
        glyph_index: 7061,
    },
    MapEntry {
        target: '阉',
        source: '閹',
        glyph_index: 7401,
    },
    MapEntry {
        target: '痹',
        source: '痺',
        glyph_index: 6082,
    },
    MapEntry {
        target: '嚓',
        source: '讌',
        glyph_index: 7056,
    },
    MapEntry {
        target: '泾',
        source: '讀',
        glyph_index: 7055,
    },
    MapEntry {
        target: '蓓',
        source: '譽',
        glyph_index: 7054,
    },
    MapEntry {
        target: '璇',
        source: '譯',
        glyph_index: 7052,
    },
    MapEntry {
        target: '缄',
        source: '緘',
        glyph_index: 6431,
    },
    MapEntry {
        target: '缜',
        source: '譟',
        glyph_index: 7050,
    },
    MapEntry {
        target: '卤',
        source: '鹵',
        glyph_index: 7742,
    },
    MapEntry {
        target: '饷',
        source: '餉',
        glyph_index: 7533,
    },
    MapEntry {
        target: '荤',
        source: '葷',
        glyph_index: 6729,
    },
    MapEntry {
        target: '懵',
        source: '譖',
        glyph_index: 7046,
    },
    MapEntry {
        target: '辐',
        source: '輻',
        glyph_index: 7196,
    },
    MapEntry {
        target: '惬',
        source: '證',
        glyph_index: 7045,
    },
    MapEntry {
        target: '炖',
        source: '燉',
        glyph_index: 5914,
    },
    MapEntry {
        target: '忑',
        source: '譌',
        glyph_index: 7042,
    },
    MapEntry {
        target: '谥',
        source: '謚',
        glyph_index: 7029,
    },
    MapEntry {
        target: '谗',
        source: '譁',
        glyph_index: 7041,
    },
    MapEntry {
        target: '罡',
        source: '謠',
        glyph_index: 7034,
    },
    MapEntry {
        target: '鹉',
        source: '鵡',
        glyph_index: 4066,
    },
    MapEntry {
        target: '涣',
        source: '諡',
        glyph_index: 7030,
    },
    MapEntry {
        target: '铠',
        source: '鎧',
        glyph_index: 1718,
    },
    MapEntry {
        target: '馋',
        source: '謌',
        glyph_index: 7027,
    },
    MapEntry {
        target: '镣',
        source: '鐐',
        glyph_index: 7364,
    },
    MapEntry {
        target: '忐',
        source: '諠',
        glyph_index: 7022,
    },
    MapEntry {
        target: '绀',
        source: '紺',
        glyph_index: 2365,
    },
    MapEntry {
        target: '掂',
        source: '諫',
        glyph_index: 7016,
    },
    MapEntry {
        target: '膑',
        source: '諚',
        glyph_index: 7015,
    },
    MapEntry {
        target: '掰',
        source: '誂',
        glyph_index: 7003,
    },
    MapEntry {
        target: '峦',
        source: '巒',
        glyph_index: 5042,
    },
    MapEntry {
        target: '叼',
        source: '觸',
        glyph_index: 6984,
    },
    MapEntry {
        target: '谧',
        source: '謐',
        glyph_index: 7032,
    },
    MapEntry {
        target: '茬',
        source: '靫',
        glyph_index: 7479,
    },
    MapEntry {
        target: '撬',
        source: '觧',
        glyph_index: 6982,
    },
    MapEntry {
        target: '滇',
        source: '觝',
        glyph_index: 6981,
    },
    MapEntry {
        target: '菀',
        source: '觀',
        glyph_index: 6978,
    },
    MapEntry {
        target: '蹒',
        source: '蹣',
        glyph_index: 7153,
    },
    MapEntry {
        target: '靳',
        source: '覽',
        glyph_index: 6976,
    },
    MapEntry {
        target: '猬',
        source: '蝟',
        glyph_index: 6853,
    },
    MapEntry {
        target: '篓',
        source: '簍',
        glyph_index: 6342,
    },
    MapEntry {
        target: '虬',
        source: '覺',
        glyph_index: 6975,
    },
    MapEntry {
        target: '囔',
        source: '覩',
        glyph_index: 6970,
    },
    MapEntry {
        target: '剁',
        source: '覊',
        glyph_index: 6966,
    },
    MapEntry {
        target: '垅',
        source: '覈',
        glyph_index: 6965,
    },
    MapEntry {
        target: '撅',
        source: '襾',
        glyph_index: 6963,
    },
    MapEntry {
        target: '垦',
        source: '襷',
        glyph_index: 6962,
    },
    MapEntry {
        target: '躏',
        source: '躪',
        glyph_index: 7168,
    },
    MapEntry {
        target: '鼢',
        source: '襴',
        glyph_index: 6961,
    },
    MapEntry {
        target: '鹫',
        source: '鷲',
        glyph_index: 4364,
    },
    MapEntry {
        target: '怂',
        source: '慫',
        glyph_index: 5218,
    },
    MapEntry {
        target: '汶',
        source: '襭',
        glyph_index: 6958,
    },
    MapEntry {
        target: '懑',
        source: '懣',
        glyph_index: 5249,
    },
    MapEntry {
        target: '嗫',
        source: '囁',
        glyph_index: 4780,
    },
    MapEntry {
        target: '觞',
        source: '觴',
        glyph_index: 6983,
    },
    MapEntry {
        target: '咔',
        source: '哢',
        glyph_index: 4714,
    },
    MapEntry {
        target: '阕',
        source: '褝',
        glyph_index: 6953,
    },
    MapEntry {
        target: '牍',
        source: '牘',
        glyph_index: 5938,
    },
    MapEntry {
        target: '诩',
        source: '襌',
        glyph_index: 6952,
    },
    MapEntry {
        target: '琨',
        source: '褞',
        glyph_index: 6943,
    },
    MapEntry {
        target: '鲤',
        source: '鯉',
        glyph_index: 2242,
    },
    MapEntry {
        target: '樨',
        source: '褌',
        glyph_index: 6939,
    },
    MapEntry {
        target: '撵',
        source: '褄',
        glyph_index: 6938,
    },
    MapEntry {
        target: '甭',
        source: '裲',
        glyph_index: 6937,
    },
    MapEntry {
        target: '媲',
        source: '裝',
        glyph_index: 6931,
    },
    MapEntry {
        target: '菇',
        source: '裄',
        glyph_index: 6927,
    },
    MapEntry {
        target: '砾',
        source: '礫',
        glyph_index: 6213,
    },
    MapEntry {
        target: '挛',
        source: '攣',
        glyph_index: 5384,
    },
    MapEntry {
        target: '讴',
        source: '謳',
        glyph_index: 7035,
    },
    MapEntry {
        target: '酗',
        source: '袿',
        glyph_index: 6924,
    },
    MapEntry {
        target: '轲',
        source: '軻',
        glyph_index: 7181,
    },
    MapEntry {
        target: '拄',
        source: '袰',
        glyph_index: 6923,
    },
    MapEntry {
        target: '蘸',
        source: '袮',
        glyph_index: 6918,
    },
    MapEntry {
        target: '熠',
        source: '袗',
        glyph_index: 6916,
    },
    MapEntry {
        target: '镰',
        source: '鎌',
        glyph_index: 1780,
    },
    MapEntry {
        target: '谄',
        source: '諂',
        glyph_index: 7014,
    },
    MapEntry {
        target: '诋',
        source: '詆',
        glyph_index: 6996,
    },
    MapEntry {
        target: '荪',
        source: '袵',
        glyph_index: 6913,
    },
    MapEntry {
        target: '咻',
        source: '衵',
        glyph_index: 6911,
    },
    MapEntry {
        target: '鹂',
        source: '衞',
        glyph_index: 6905,
    },
    MapEntry {
        target: '栾',
        source: '欒',
        glyph_index: 5658,
    },
    MapEntry {
        target: '荔',
        source: '衒',
        glyph_index: 6903,
    },
    MapEntry {
        target: '馁',
        source: '餒',
        glyph_index: 7534,
    },
    MapEntry {
        target: '骥',
        source: '驥',
        glyph_index: 7590,
    },
    MapEntry {
        target: '峒',
        source: '衂',
        glyph_index: 6902,
    },
    MapEntry {
        target: '骁',
        source: '驍',
        glyph_index: 7585,
    },
    MapEntry {
        target: '桅',
        source: '衄',
        glyph_index: 6901,
    },
    MapEntry {
        target: '毗',
        source: '蠻',
        glyph_index: 6900,
    },
    MapEntry {
        target: '馅',
        source: '餡',
        glyph_index: 7537,
    },
    MapEntry {
        target: '瘸',
        source: '蠧',
        glyph_index: 6899,
    },
    MapEntry {
        target: '龊',
        source: '齪',
        glyph_index: 7795,
    },
    MapEntry {
        target: '饯',
        source: '餞',
        glyph_index: 7539,
    },
    MapEntry {
        target: '窠',
        source: '蠶',
        glyph_index: 6897,
    },
    MapEntry {
        target: '抠',
        source: '蠑',
        glyph_index: 6891,
    },
    MapEntry {
        target: '剐',
        source: '蠎',
        glyph_index: 6889,
    },
    MapEntry {
        target: '龌',
        source: '齷',
        glyph_index: 7796,
    },
    MapEntry {
        target: '柠',
        source: '蟷',
        glyph_index: 6888,
    },
    MapEntry {
        target: '斓',
        source: '蠍',
        glyph_index: 6885,
    },
    MapEntry {
        target: '犷',
        source: '蠏',
        glyph_index: 6884,
    },
    MapEntry {
        target: '颍',
        source: '潁',
        glyph_index: 5816,
    },
    MapEntry {
        target: '稹',
        source: '蟲',
        glyph_index: 6882,
    },
    MapEntry {
        target: '囱',
        source: '蟇',
        glyph_index: 6878,
    },
    MapEntry {
        target: '琛',
        source: '蟐',
        glyph_index: 6873,
    },
    MapEntry {
        target: '靓',
        source: '螢',
        glyph_index: 6866,
    },
    MapEntry {
        target: '蛐',
        source: '蠅',
        glyph_index: 6865,
    },
    MapEntry {
        target: '呓',
        source: '囈',
        glyph_index: 4783,
    },
    MapEntry {
        target: '刨',
        source: '蝪',
        glyph_index: 6864,
    },
    MapEntry {
        target: '哒',
        source: '蝨',
        glyph_index: 6859,
    },
    MapEntry {
        target: '岖',
        source: '嶇',
        glyph_index: 5028,
    },
    MapEntry {
        target: '捋',
        source: '蜑',
        glyph_index: 6840,
    },
    MapEntry {
        target: '闾',
        source: '閭',
        glyph_index: 7398,
    },
    MapEntry {
        target: '嗬',
        source: '蛯',
        glyph_index: 6833,
    },
    MapEntry {
        target: '勰',
        source: '蛬',
        glyph_index: 6830,
    },
    MapEntry {
        target: '骧',
        source: '蠣',
        glyph_index: 6825,
    },
    MapEntry {
        target: '腼',
        source: '靦',
        glyph_index: 7476,
    },
    MapEntry {
        target: '刽',
        source: '蚰',
        glyph_index: 6823,
    },
    MapEntry {
        target: '榭',
        source: '號',
        glyph_index: 6810,
    },
    MapEntry {
        target: '啕',
        source: '乕',
        glyph_index: 6808,
    },
    MapEntry {
        target: '羿',
        source: '虍',
        glyph_index: 6807,
    },
    MapEntry {
        target: '喏',
        source: '蘰',
        glyph_index: 6805,
    },
    MapEntry {
        target: '坍',
        source: '蘆',
        glyph_index: 6802,
    },
    MapEntry {
        target: '鹭',
        source: '鷺',
        glyph_index: 2426,
    },
    MapEntry {
        target: '捎',
        source: '蘋',
        glyph_index: 6799,
    },
    MapEntry {
        target: '婺',
        source: '蘓',
        glyph_index: 6798,
    },
    MapEntry {
        target: '焘',
        source: '藥',
        glyph_index: 6794,
    },
    MapEntry {
        target: '谀',
        source: '諛',
        glyph_index: 7026,
    },
    MapEntry {
        target: '镐',
        source: '薹',
        glyph_index: 6790,
    },
    MapEntry {
        target: '颦',
        source: '顰',
        glyph_index: 7519,
    },
    MapEntry {
        target: '痉',
        source: '痙',
        glyph_index: 6074,
    },
    MapEntry {
        target: '崆',
        source: '蕷',
        glyph_index: 6784,
    },
    MapEntry {
        target: '梆',
        source: '藪',
        glyph_index: 6781,
    },
    MapEntry {
        target: '镂',
        source: '鏤',
        glyph_index: 7358,
    },
    MapEntry {
        target: '掇',
        source: '薑',
        glyph_index: 6775,
    },
    MapEntry {
        target: '戕',
        source: '薀',
        glyph_index: 6772,
    },
    MapEntry {
        target: '脐',
        source: '臍',
        glyph_index: 6612,
    },
    MapEntry {
        target: '蜗',
        source: '蝸',
        glyph_index: 6854,
    },
    MapEntry {
        target: '咿',
        source: '蕋',
        glyph_index: 6770,
    },
    MapEntry {
        target: '瞌',
        source: '蘂',
        glyph_index: 6769,
    },
    MapEntry {
        target: '翱',
        source: '蕣',
        glyph_index: 6765,
    },
    MapEntry {
        target: '褛',
        source: '褸',
        glyph_index: 6951,
    },
    MapEntry {
        target: '熵',
        source: '蕀',
        glyph_index: 6764,
    },
    MapEntry {
        target: '臊',
        source: '蔔',
        glyph_index: 6762,
    },
    MapEntry {
        target: '谩',
        source: '謾',
        glyph_index: 7039,
    },
    MapEntry {
        target: '龛',
        source: '龕',
        glyph_index: 7799,
    },
    MapEntry {
        target: '恿',
        source: '慂',
        glyph_index: 5211,
    },
    MapEntry {
        target: '搪',
        source: '溏',
        glyph_index: 5812,
    },
    MapEntry {
        target: '璀',
        source: '蔕',
        glyph_index: 6761,
    },
    MapEntry {
        target: '诌',
        source: '蔟',
        glyph_index: 6760,
    },
    MapEntry {
        target: '骞',
        source: '騫',
        glyph_index: 7577,
    },
    MapEntry {
        target: '鸪',
        source: '鴣',
        glyph_index: 7698,
    },
    MapEntry {
        target: '嘭',
        source: '蔘',
        glyph_index: 6758,
    },
    MapEntry {
        target: '褴',
        source: '襤',
        glyph_index: 6957,
    },
    MapEntry {
        target: '胰',
        source: '蓆',
        glyph_index: 6751,
    },
    MapEntry {
        target: '妪',
        source: '嫗',
        glyph_index: 4921,
    },
    MapEntry {
        target: '盹',
        source: '蓚',
        glyph_index: 6748,
    },
    MapEntry {
        target: '啮',
        source: '噛',
        glyph_index: 1781,
    },
    MapEntry {
        target: '娣',
        source: '蒻',
        glyph_index: 6747,
    },
    MapEntry {
        target: '榄',
        source: '欖',
        glyph_index: 5659,
    },
    MapEntry {
        target: '泞',
        source: '濘',
        glyph_index: 5858,
    },
    MapEntry {
        target: '坳',
        source: '蓙',
        glyph_index: 6745,
    },
    MapEntry {
        target: '侬',
        source: '蒟',
        glyph_index: 6744,
    },
    MapEntry {
        target: '炀',
        source: '煬',
        glyph_index: 5903,
    },
    MapEntry {
        target: '皑',
        source: '皚',
        glyph_index: 6122,
    },
    MapEntry {
        target: '娲',
        source: '葢',
        glyph_index: 6741,
    },
    MapEntry {
        target: '辇',
        source: '輦',
        glyph_index: 7194,
    },
    MapEntry {
        target: '鳖',
        source: '葹',
        glyph_index: 6738,
    },
    MapEntry {
        target: '骊',
        source: '驪',
        glyph_index: 7594,
    },
    MapEntry {
        target: '鹧',
        source: '鷓',
        glyph_index: 7733,
    },
    MapEntry {
        target: '姝',
        source: '葯',
        glyph_index: 6737,
    },
    MapEntry {
        target: '饨',
        source: '飩',
        glyph_index: 7530,
    },
    MapEntry {
        target: '腌',
        source: '萬',
        glyph_index: 6736,
    },
    MapEntry {
        target: '辘',
        source: '轆',
        glyph_index: 7203,
    },
    MapEntry {
        target: '婕',
        source: '葮',
        glyph_index: 6732,
    },
    MapEntry {
        target: '玑',
        source: '蒭',
        glyph_index: 6731,
    },
    MapEntry {
        target: '撷',
        source: '蒄',
        glyph_index: 6728,
    },
    MapEntry {
        target: '暨',
        source: '曁',
        glyph_index: 5447,
    },
    MapEntry {
        target: '阂',
        source: '蕚',
        glyph_index: 6727,
    },
    MapEntry {
        target: '绾',
        source: '綰',
        glyph_index: 6430,
    },
    MapEntry {
        target: '铨',
        source: '銓',
        glyph_index: 7321,
    },
    MapEntry {
        target: '钺',
        source: '鉞',
        glyph_index: 7309,
    },
    MapEntry {
        target: '俪',
        source: '儷',
        glyph_index: 4533,
    },
    MapEntry {
        target: '昙',
        source: '曇',
        glyph_index: 3539,
    },
    MapEntry {
        target: '驮',
        source: '駄',
        glyph_index: 3145,
    },
    MapEntry {
        target: '赡',
        source: '贍',
        glyph_index: 7105,
    },
    MapEntry {
        target: '嵘',
        source: '萪',
        glyph_index: 6725,
    },
    MapEntry {
        target: '孪',
        source: '菻',
        glyph_index: 6723,
    },
    MapEntry {
        target: '傥',
        source: '儻',
        glyph_index: 4535,
    },
    MapEntry {
        target: '驷',
        source: '駟',
        glyph_index: 7561,
    },
    MapEntry {
        target: '跻',
        source: '躋',
        glyph_index: 7162,
    },
    MapEntry {
        target: '淄',
        source: '蔆',
        glyph_index: 6722,
    },
    MapEntry {
        target: '淖',
        source: '萠',
        glyph_index: 6719,
    },
    MapEntry {
        target: '佟',
        source: '萢',
        glyph_index: 6718,
    },
    MapEntry {
        target: '虢',
        source: '菷',
        glyph_index: 6713,
    },
    MapEntry {
        target: '峥',
        source: '崢',
        glyph_index: 5017,
    },
    MapEntry {
        target: '砭',
        source: '菎',
        glyph_index: 6707,
    },
    MapEntry {
        target: '桦',
        source: '枠',
        glyph_index: 4363,
    },
    MapEntry {
        target: '璨',
        source: '菫',
        glyph_index: 6706,
    },
    MapEntry {
        target: '擞',
        source: '萓',
        glyph_index: 6705,
    },
    MapEntry {
        target: '昕',
        source: '菴',
        glyph_index: 6704,
    },
    MapEntry {
        target: '贽',
        source: '贄',
        glyph_index: 7100,
    },
    MapEntry {
        target: '舀',
        source: '荵',
        glyph_index: 6700,
    },
    MapEntry {
        target: '蹿',
        source: '荳',
        glyph_index: 6699,
    },
    MapEntry {
        target: '鹄',
        source: '鵠',
        glyph_index: 2339,
    },
    MapEntry {
        target: '谶',
        source: '讖',
        glyph_index: 7060,
    },
    MapEntry {
        target: '倌',
        source: '莵',
        glyph_index: 6698,
    },
    MapEntry {
        target: '咛',
        source: '嚀',
        glyph_index: 4769,
    },
    MapEntry {
        target: '嫔',
        source: '嬪',
        glyph_index: 4932,
    },
    MapEntry {
        target: '螃',
        source: '莊',
        glyph_index: 6696,
    },
    MapEntry {
        target: '磷',
        source: '莇',
        glyph_index: 6695,
    },
    MapEntry {
        target: '镌',
        source: '鐫',
        glyph_index: 7366,
    },
    MapEntry {
        target: '獠',
        source: '茣',
        glyph_index: 6693,
    },
    MapEntry {
        target: '挎',
        source: '莖',
        glyph_index: 6692,
    },
    MapEntry {
        target: '氲',
        source: '莟',
        glyph_index: 6690,
    },
    MapEntry {
        target: '龇',
        source: '莚',
        glyph_index: 6688,
    },
    MapEntry {
        target: '闰',
        source: '閏',
        glyph_index: 1531,
    },
    MapEntry {
        target: '胫',
        source: '脛',
        glyph_index: 6575,
    },
    MapEntry {
        target: '诰',
        source: '誥',
        glyph_index: 7008,
    },
    MapEntry {
        target: '摞',
        source: '茘',
        glyph_index: 6686,
    },
    MapEntry {
        target: '谲',
        source: '譎',
        glyph_index: 7044,
    },
    MapEntry {
        target: '唰',
        source: '荅',
        glyph_index: 6682,
    },
    MapEntry {
        target: '噼',
        source: '茖',
        glyph_index: 6676,
    },
    MapEntry {
        target: '噔',
        source: '苙',
        glyph_index: 6673,
    },
    MapEntry {
        target: '粼',
        source: '茆',
        glyph_index: 6670,
    },
    MapEntry {
        target: '嘤',
        source: '嚶',
        glyph_index: 4776,
    },
    MapEntry {
        target: '呦',
        source: '苺',
        glyph_index: 6664,
    },
    MapEntry {
        target: '裆',
        source: '襠',
        glyph_index: 6954,
    },
    MapEntry {
        target: '蛀',
        source: '苳',
        glyph_index: 6663,
    },
    MapEntry {
        target: '怵',
        source: '艸',
        glyph_index: 6650,
    },
    MapEntry {
        target: '馄',
        source: '鯤',
        glyph_index: 7655,
    },
    MapEntry {
        target: '庾',
        source: '艷',
        glyph_index: 6649,
    },
    MapEntry {
        target: '溉',
        source: '漑',
        glyph_index: 5817,
    },
    MapEntry {
        target: '涮',
        source: '舮',
        glyph_index: 6647,
    },
    MapEntry {
        target: '荟',
        source: '薈',
        glyph_index: 6774,
    },
    MapEntry {
        target: '殇',
        source: '殤',
        glyph_index: 5686,
    },
    MapEntry {
        target: '摈',
        source: '擯',
        glyph_index: 5372,
    },
    MapEntry {
        target: '俅',
        source: '艪',
        glyph_index: 6645,
    },
    MapEntry {
        target: '噶',
        source: '艨',
        glyph_index: 6644,
    },
    MapEntry {
        target: '睑',
        source: '瞼',
        glyph_index: 6168,
    },
    MapEntry {
        target: '觎',
        source: '覦',
        glyph_index: 6971,
    },
    MapEntry {
        target: '镳',
        source: '艢',
        glyph_index: 6643,
    },
    MapEntry {
        target: '刍',
        source: '芻',
        glyph_index: 6656,
    },
    MapEntry {
        target: '嬗',
        source: '艟',
        glyph_index: 6641,
    },
    MapEntry {
        target: '磺',
        source: '艚',
        glyph_index: 6640,
    },
    MapEntry {
        target: '犊',
        source: '犢',
        glyph_index: 5946,
    },
    MapEntry {
        target: '锲',
        source: '艝',
        glyph_index: 6639,
    },
    MapEntry {
        target: '绶',
        source: '綬',
        glyph_index: 2627,
    },
    MapEntry {
        target: '郸',
        source: '鄲',
        glyph_index: 7271,
    },
    MapEntry {
        target: '蜓',
        source: '艀',
        glyph_index: 6636,
    },
    MapEntry {
        target: '莘',
        source: '舩',
        glyph_index: 6632,
    },
    MapEntry {
        target: '漳',
        source: '舖',
        glyph_index: 6631,
    },
    MapEntry {
        target: '鸨',
        source: '鴇',
        glyph_index: 3507,
    },
    MapEntry {
        target: '殁',
        source: '歿',
        glyph_index: 5678,
    },
    MapEntry {
        target: '咭',
        source: '舊',
        glyph_index: 6628,
    },
    MapEntry {
        target: '铉',
        source: '與',
        glyph_index: 6627,
    },
    MapEntry {
        target: '鹳',
        source: '鸛',
        glyph_index: 7740,
    },
    MapEntry {
        target: '槟',
        source: '檳',
        glyph_index: 5646,
    },
    MapEntry {
        target: '搽',
        source: '臺',
        glyph_index: 6621,
    },
    MapEntry {
        target: '坨',
        source: '臟',
        glyph_index: 6618,
    },
    MapEntry {
        target: '觊',
        source: '覬',
        glyph_index: 6972,
    },
    MapEntry {
        target: '饬',
        source: '飭',
        glyph_index: 4617,
    },
    MapEntry {
        target: '堑',
        source: '塹',
        glyph_index: 4840,
    },
    MapEntry {
        target: '灏',
        source: '臈',
        glyph_index: 6616,
    },
    MapEntry {
        target: '闳',
        source: '臙',
        glyph_index: 6614,
    },
    MapEntry {
        target: '颢',
        source: '臑',
        glyph_index: 6613,
    },
    MapEntry {
        target: '琮',
        source: '膽',
        glyph_index: 6607,
    },
    MapEntry {
        target: '嗑',
        source: '膸',
        glyph_index: 6606,
    },
    MapEntry {
        target: '蟑',
        source: '膵',
        glyph_index: 6604,
    },
    MapEntry {
        target: '愠',
        source: '慍',
        glyph_index: 5187,
    },
    MapEntry {
        target: '榈',
        source: '櫚',
        glyph_index: 5652,
    },
    MapEntry {
        target: '咂',
        source: '膰',
        glyph_index: 6603,
    },
    MapEntry {
        target: '糗',
        source: '膓',
        glyph_index: 6601,
    },
    MapEntry {
        target: '贲',
        source: '賁',
        glyph_index: 7093,
    },
    MapEntry {
        target: '膻',
        source: '羶',
        glyph_index: 6516,
    },
    MapEntry {
        target: '鸢',
        source: '鳶',
        glyph_index: 3525,
    },
    MapEntry {
        target: '掸',
        source: '腟',
        glyph_index: 6600,
    },
    MapEntry {
        target: '绉',
        source: '膣',
        glyph_index: 6599,
    },
    MapEntry {
        target: '泠',
        source: '膤',
        glyph_index: 6598,
    },
    MapEntry {
        target: '偻',
        source: '膕',
        glyph_index: 6597,
    },
    MapEntry {
        target: '碉',
        source: '膃',
        glyph_index: 6591,
    },
    MapEntry {
        target: '诨',
        source: '諢',
        glyph_index: 7023,
    },
    MapEntry {
        target: '盅',
        source: '腦',
        glyph_index: 6589,
    },
    MapEntry {
        target: '睢',
        source: '脣',
        glyph_index: 6577,
    },
    MapEntry {
        target: '臃',
        source: '脩',
        glyph_index: 6576,
    },
    MapEntry {
        target: '蹩',
        source: '肬',
        glyph_index: 6564,
    },
    MapEntry {
        target: '嶙',
        source: '冐',
        glyph_index: 6563,
    },
    MapEntry {
        target: '蘑',
        source: '肭',
        glyph_index: 6562,
    },
    MapEntry {
        target: '嗥',
        source: '肅',
        glyph_index: 6558,
    },
    MapEntry {
        target: '仨',
        source: '聽',
        glyph_index: 6554,
    },
    MapEntry {
        target: '赣',
        source: '聹',
        glyph_index: 6553,
    },
    MapEntry {
        target: '绌',
        source: '聰',
        glyph_index: 6551,
    },
    MapEntry {
        target: '扪',
        source: '捫',
        glyph_index: 5326,
    },
    MapEntry {
        target: '忪',
        source: '聲',
        glyph_index: 6550,
    },
    MapEntry {
        target: '峋',
        source: '聨',
        glyph_index: 6548,
    },
    MapEntry {
        target: '祜',
        source: '聢',
        glyph_index: 6547,
    },
    MapEntry {
        target: '镀',
        source: '鍍',
        glyph_index: 3436,
    },
    MapEntry {
        target: '颚',
        source: '顎',
        glyph_index: 1751,
    },
    MapEntry {
        target: '倜',
        source: '聟',
        glyph_index: 6546,
    },
    MapEntry {
        target: '镬',
        source: '耡',
        glyph_index: 6537,
    },
    MapEntry {
        target: '鹪',
        source: '鷦',
        glyph_index: 7735,
    },
    MapEntry {
        target: '靥',
        source: '靨',
        glyph_index: 7477,
    },
    MapEntry {
        target: '圯',
        source: '飜',
        glyph_index: 6529,
    },
    MapEntry {
        target: '叵',
        source: '翆',
        glyph_index: 6520,
    },
    MapEntry {
        target: '揿',
        source: '譱',
        glyph_index: 6518,
    },
    MapEntry {
        target: '茁',
        source: '羮',
        glyph_index: 6515,
    },
    MapEntry {
        target: '觐',
        source: '覲',
        glyph_index: 6974,
    },
    MapEntry {
        target: '仃',
        source: '羣',
        glyph_index: 6511,
    },
    MapEntry {
        target: '挲',
        source: '羇',
        glyph_index: 6505,
    },
    MapEntry {
        target: '剜',
        source: '羃',
        glyph_index: 6503,
    },
    MapEntry {
        target: '铢',
        source: '銖',
        glyph_index: 7320,
    },
    MapEntry {
        target: '鑫',
        source: '羂',
        glyph_index: 6501,
    },
    MapEntry {
        target: '篙',
        source: '罸',
        glyph_index: 6500,
    },
    MapEntry {
        target: '氢',
        source: '輕',
        glyph_index: 7186,
    },
    MapEntry {
        target: '泵',
        source: '罧',
        glyph_index: 6499,
    },
    MapEntry {
        target: '罂',
        source: '罌',
        glyph_index: 6487,
    },
    MapEntry {
        target: '蓟',
        source: '薊',
        glyph_index: 6776,
    },
    MapEntry {
        target: '卟',
        source: '罠',
        glyph_index: 6496,
    },
    MapEntry {
        target: '芮',
        source: '罎',
        glyph_index: 6489,
    },
    MapEntry {
        target: '氨',
        source: '鮟',
        glyph_index: 7642,
    },
    MapEntry {
        target: '骅',
        source: '纎',
        glyph_index: 6481,
    },
    MapEntry {
        target: '辔',
        source: '轡',
        glyph_index: 2072,
    },
    MapEntry {
        target: '赊',
        source: '纖',
        glyph_index: 6480,
    },
    MapEntry {
        target: '苯',
        source: '纔',
        glyph_index: 6479,
    },
    MapEntry {
        target: '迤',
        source: '纐',
        glyph_index: 6477,
    },
    MapEntry {
        target: '徜',
        source: '纒',
        glyph_index: 6476,
    },
    MapEntry {
        target: '鹘',
        source: '鶻',
        glyph_index: 7726,
    },
    MapEntry {
        target: '邕',
        source: '續',
        glyph_index: 6475,
    },
    MapEntry {
        target: '坯',
        source: '繿',
        glyph_index: 6472,
    },
    MapEntry {
        target: '潢',
        source: '緕',
        glyph_index: 6469,
    },
    MapEntry {
        target: '菡',
        source: '纃',
        glyph_index: 6468,
    },
    MapEntry {
        target: '邋',
        source: '鑞',
        glyph_index: 7376,
    },
    MapEntry {
        target: '歆',
        source: '繻',
        glyph_index: 6467,
    },
    MapEntry {
        target: '娆',
        source: '繼',
        glyph_index: 6466,
    },
    MapEntry {
        target: '芊',
        source: '繩',
        glyph_index: 6465,
    },
    MapEntry {
        target: '踮',
        source: '鈿',
        glyph_index: 7316,
    },
    MapEntry {
        target: '碱',
        source: '繪',
        glyph_index: 6464,
    },
    MapEntry {
        target: '鬃',
        source: '繙',
        glyph_index: 6461,
    },
    MapEntry {
        target: '籽',
        source: '繖',
        glyph_index: 6459,
    },
    MapEntry {
        target: '牯',
        source: '繧',
        glyph_index: 6457,
    },
    MapEntry {
        target: '鲧',
        source: '鯀',
        glyph_index: 7646,
    },
    MapEntry {
        target: '遢',
        source: '縺',
        glyph_index: 6456,
    },
    MapEntry {
        target: '聩',
        source: '繦',
        glyph_index: 6449,
    },
    MapEntry {
        target: '颏',
        source: '縱',
        glyph_index: 6443,
    },
    MapEntry {
        target: '跎',
        source: '縒',
        glyph_index: 6442,
    },
    MapEntry {
        target: '垛',
        source: '縡',
        glyph_index: 6441,
    },
    MapEntry {
        target: '埂',
        source: '縣',
        glyph_index: 6440,
    },
    MapEntry {
        target: '耷',
        source: '縅',
        glyph_index: 6438,
    },
    MapEntry {
        target: '诮',
        source: '誚',
        glyph_index: 7010,
    },
    MapEntry {
        target: '镛',
        source: '緻',
        glyph_index: 6435,
    },
    MapEntry {
        target: '咣',
        source: '緤',
        glyph_index: 6433,
    },
    MapEntry {
        target: '飓',
        source: '颶',
        glyph_index: 7526,
    },
    MapEntry {
        target: '铀',
        source: '綟',
        glyph_index: 6429,
    },
    MapEntry {
        target: '殓',
        source: '緜',
        glyph_index: 6427,
    },
    MapEntry {
        target: '殒',
        source: '殞',
        glyph_index: 5685,
    },
    MapEntry {
        target: '徉',
        source: '綯',
        glyph_index: 6426,
    },
    MapEntry {
        target: '侩',
        source: '總',
        glyph_index: 6424,
    },
    MapEntry {
        target: '饪',
        source: '綫',
        glyph_index: 6423,
    },
    MapEntry {
        target: '昀',
        source: '綵',
        glyph_index: 6420,
    },
    MapEntry {
        target: '锴',
        source: '綛',
        glyph_index: 6416,
    },
    MapEntry {
        target: '箩',
        source: '絽',
        glyph_index: 6415,
    },
    MapEntry {
        target: '煲',
        source: '襃',
        glyph_index: 6942,
    },
    MapEntry {
        target: '徕',
        source: '徠',
        glyph_index: 5125,
    },
    MapEntry {
        target: '焱',
        source: '經',
        glyph_index: 6411,
    },
    MapEntry {
        target: '岜',
        source: '絣',
        glyph_index: 6410,
    },
    MapEntry {
        target: '飨',
        source: '饗',
        glyph_index: 2007,
    },
    MapEntry {
        target: '摁',
        source: '絏',
        glyph_index: 6409,
    },
    MapEntry {
        target: '嗲',
        source: '絎',
        glyph_index: 6405,
    },
    MapEntry {
        target: '缙',
        source: '縉',
        glyph_index: 6445,
    },
    MapEntry {
        target: '邈',
        source: '絖',
        glyph_index: 6404,
    },
    MapEntry {
        target: '铝',
        source: '紵',
        glyph_index: 6401,
    },
    MapEntry {
        target: '钙',
        source: '紮',
        glyph_index: 6398,
    },
    MapEntry {
        target: '纫',
        source: '絅',
        glyph_index: 6396,
    },
    MapEntry {
        target: '诟',
        source: '詬',
        glyph_index: 7000,
    },
    MapEntry {
        target: '鼹',
        source: '糺',
        glyph_index: 6390,
    },
    MapEntry {
        target: '凫',
        source: '鳬',
        glyph_index: 7688,
    },
    MapEntry {
        target: '椭',
        source: '橢',
        glyph_index: 5628,
    },
    MapEntry {
        target: '骛',
        source: '糴',
        glyph_index: 6388,
    },
    MapEntry {
        target: '缢',
        source: '縊',
        glyph_index: 6439,
    },
    MapEntry {
        target: '傣',
        source: '糒',
        glyph_index: 6382,
    },
    MapEntry {
        target: '噙',
        source: '糘',
        glyph_index: 6381,
    },
    MapEntry {
        target: '鹜',
        source: '鶩',
        glyph_index: 7722,
    },
    MapEntry {
        target: '淼',
        source: '糂',
        glyph_index: 6380,
    },
    MapEntry {
        target: '锹',
        source: '鍬',
        glyph_index: 2080,
    },
    MapEntry {
        target: '呤',
        source: '糀',
        glyph_index: 6378,
    },
    MapEntry {
        target: '滟',
        source: '粨',
        glyph_index: 6371,
    },
    MapEntry {
        target: '踽',
        source: '粡',
        glyph_index: 6370,
    },
    MapEntry {
        target: '聃',
        source: '粫',
        glyph_index: 6369,
    },
    MapEntry {
        target: '钿',
        source: '粢',
        glyph_index: 6368,
    },
    MapEntry {
        target: '炷',
        source: '粭',
        glyph_index: 6367,
    },
    MapEntry {
        target: '杈',
        source: '粐',
        glyph_index: 6365,
    },
    MapEntry {
        target: '浃',
        source: '浹',
        glyph_index: 5754,
    },
    MapEntry {
        target: '搡',
        source: '粃',
        glyph_index: 6364,
    },
    MapEntry {
        target: '赅',
        source: '籵',
        glyph_index: 6363,
    },
    MapEntry {
        target: '晔',
        source: '曄',
        glyph_index: 5452,
    },
    MapEntry {
        target: '讦',
        source: '訐',
        glyph_index: 6987,
    },
    MapEntry {
        target: '旎',
        source: '籖',
        glyph_index: 6360,
    },
    MapEntry {
        target: '恹',
        source: '籤',
        glyph_index: 6359,
    },
    MapEntry {
        target: '玮',
        source: '籟',
        glyph_index: 6358,
    },
    MapEntry {
        target: '焯',
        source: '籘',
        glyph_index: 6357,
    },
    MapEntry {
        target: '轸',
        source: '軫',
        glyph_index: 7182,
    },
    MapEntry {
        target: '奁',
        source: '奩',
        glyph_index: 4887,
    },
    MapEntry {
        target: '玟',
        source: '籐',
        glyph_index: 6356,
    },
    MapEntry {
        target: '跆',
        source: '籏',
        glyph_index: 6354,
    },
    MapEntry {
        target: '恽',
        source: '籔',
        glyph_index: 6353,
    },
    MapEntry {
        target: '噱',
        source: '簷',
        glyph_index: 6348,
    },
    MapEntry {
        target: '钎',
        source: '篶',
        glyph_index: 6343,
    },
    MapEntry {
        target: '鸵',
        source: '鴕',
        glyph_index: 7701,
    },
    MapEntry {
        target: '橐',
        source: '簗',
        glyph_index: 6341,
    },
    MapEntry {
        target: '蘅',
        source: '簓',
        glyph_index: 6338,
    },
    MapEntry {
        target: '灞',
        source: '簀',
        glyph_index: 6336,
    },
    MapEntry {
        target: '藓',
        source: '蘚',
        glyph_index: 6804,
    },
    MapEntry {
        target: '殚',
        source: '殫',
        glyph_index: 5688,
    },
    MapEntry {
        target: '诿',
        source: '籠',
        glyph_index: 6335,
    },
    MapEntry {
        target: '侪',
        source: '儕',
        glyph_index: 4528,
    },
    MapEntry {
        target: '忾',
        source: '愾',
        glyph_index: 5202,
    },
    MapEntry {
        target: '旖',
        source: '簔',
        glyph_index: 6332,
    },
    MapEntry {
        target: '趄',
        source: '簑',
        glyph_index: 6331,
    },
    MapEntry {
        target: '缛',
        source: '縟',
        glyph_index: 6444,
    },
    MapEntry {
        target: '矸',
        source: '篏',
        glyph_index: 6326,
    },
    MapEntry {
        target: '绡',
        source: '箙',
        glyph_index: 6322,
    },
    MapEntry {
        target: '帏',
        source: '幃',
        glyph_index: 5056,
    },
    MapEntry {
        target: '碴',
        source: '箏',
        glyph_index: 6320,
    },
    MapEntry {
        target: '祛',
        source: '箒',
        glyph_index: 6319,
    },
    MapEntry {
        target: '谌',
        source: '箚',
        glyph_index: 6317,
    },
    MapEntry {
        target: '鼐',
        source: '箟',
        glyph_index: 6314,
    },
    MapEntry {
        target: '銮',
        source: '鑾',
        glyph_index: 7385,
    },
    MapEntry {
        target: '瘀',
        source: '箘',
        glyph_index: 6313,
    },
    MapEntry {
        target: '嘁',
        source: '筬',
        glyph_index: 6310,
    },
    MapEntry {
        target: '狯',
        source: '獪',
        glyph_index: 5975,
    },
    MapEntry {
        target: '裱',
        source: '筰',
        glyph_index: 6308,
    },
    MapEntry {
        target: '呲',
        source: '筴',
        glyph_index: 6306,
    },
    MapEntry {
        target: '嗝',
        source: '筥',
        glyph_index: 6305,
    },
    MapEntry {
        target: '囫',
        source: '筅',
        glyph_index: 6303,
    },
    MapEntry {
        target: '馔',
        source: '饌',
        glyph_index: 7553,
    },
    MapEntry {
        target: '罄',
        source: '筍',
        glyph_index: 6300,
    },
    MapEntry {
        target: '嘹',
        source: '筺',
        glyph_index: 6298,
    },
    MapEntry {
        target: '骓',
        source: '騅',
        glyph_index: 7574,
    },
    MapEntry {
        target: '囵',
        source: '笶',
        glyph_index: 6296,
    },
    MapEntry {
        target: '牖',
        source: '笵',
        glyph_index: 6294,
    },
    MapEntry {
        target: '沏',
        source: '笘',
        glyph_index: 6291,
    },
    MapEntry {
        target: '阗',
        source: '笂',
        glyph_index: 6286,
    },
    MapEntry {
        target: '瓯',
        source: '甌',
        glyph_index: 6029,
    },
    MapEntry {
        target: '蚱',
        source: '竢',
        glyph_index: 6282,
    },
    MapEntry {
        target: '遛',
        source: '竡',
        glyph_index: 6281,
    },
    MapEntry {
        target: '樾',
        source: '竝',
        glyph_index: 6280,
    },
    MapEntry {
        target: '痨',
        source: '竚',
        glyph_index: 6279,
    },
    MapEntry {
        target: '蒯',
        source: '竓',
        glyph_index: 6277,
    },
    MapEntry {
        target: '堇',
        source: '竏',
        glyph_index: 6275,
    },
    MapEntry {
        target: '誊',
        source: '謄',
        glyph_index: 3482,
    },
    MapEntry {
        target: '讫',
        source: '訖',
        glyph_index: 6986,
    },
    MapEntry {
        target: '诳',
        source: '誑',
        glyph_index: 7007,
    },
    MapEntry {
        target: '馊',
        source: '竍',
        glyph_index: 6274,
    },
    MapEntry {
        target: '郦',
        source: '竊',
        glyph_index: 6273,
    },
    MapEntry {
        target: '螅',
        source: '窰',
        glyph_index: 6266,
    },
    MapEntry {
        target: '俦',
        source: '儔',
        glyph_index: 4529,
    },
    MapEntry {
        target: '鲲',
        source: '竈',
        glyph_index: 6265,
    },
    MapEntry {
        target: '膘',
        source: '穽',
        glyph_index: 6258,
    },
    MapEntry {
        target: '韪',
        source: '龝',
        glyph_index: 6255,
    },
    MapEntry {
        target: '悱',
        source: '穩',
        glyph_index: 6254,
    },
    MapEntry {
        target: '桠',
        source: '椏',
        glyph_index: 5534,
    },
    MapEntry {
        target: '乩',
        source: '穉',
        glyph_index: 6251,
    },
    MapEntry {
        target: '衩',
        source: '穃',
        glyph_index: 6249,
    },
    MapEntry {
        target: '姘',
        source: '稾',
        glyph_index: 6247,
    },
    MapEntry {
        target: '怄',
        source: '稱',
        glyph_index: 6245,
    },
    MapEntry {
        target: '钠',
        source: '稟',
        glyph_index: 6243,
    },
    MapEntry {
        target: '醮',
        source: '稙',
        glyph_index: 6241,
    },
    MapEntry {
        target: '溧',
        source: '稘',
        glyph_index: 6240,
    },
    MapEntry {
        target: '铄',
        source: '鑠',
        glyph_index: 7374,
    },
    MapEntry {
        target: '颧',
        source: '顴',
        glyph_index: 7521,
    },
    MapEntry {
        target: '畹',
        source: '秡',
        glyph_index: 6236,
    },
    MapEntry {
        target: '浔',
        source: '潯',
        glyph_index: 5836,
    },
    MapEntry {
        target: '栊',
        source: '槞',
        glyph_index: 5602,
    },
    MapEntry {
        target: '姣',
        source: '秬',
        glyph_index: 6235,
    },
    MapEntry {
        target: '觥',
        source: '禮',
        glyph_index: 6228,
    },
    MapEntry {
        target: '氯',
        source: '禪',
        glyph_index: 6227,
    },
    MapEntry {
        target: '噘',
        source: '齋',
        glyph_index: 6226,
    },
    MapEntry {
        target: '锏',
        source: '繝',
        glyph_index: 6458,
    },
    MapEntry {
        target: '赝',
        source: '贋',
        glyph_index: 1855,
    },
    MapEntry {
        target: '缮',
        source: '繕',
        glyph_index: 3029,
    },
    MapEntry {
        target: '秸',
        source: '禝',
        glyph_index: 6224,
    },
    MapEntry {
        target: '疟',
        source: '瘧',
        glyph_index: 6089,
    },
    MapEntry {
        target: '缦',
        source: '縵',
        glyph_index: 6451,
    },
    MapEntry {
        target: '馍',
        source: '祿',
        glyph_index: 6222,
    },
    MapEntry {
        target: '哔',
        source: '祕',
        glyph_index: 6219,
    },
    MapEntry {
        target: '钤',
        source: '礙',
        glyph_index: 6211,
    },
    MapEntry {
        target: '箧',
        source: '篋',
        glyph_index: 6323,
    },
    MapEntry {
        target: '鄱',
        source: '礑',
        glyph_index: 6210,
    },
    MapEntry {
        target: '赁',
        source: '賃',
        glyph_index: 3323,
    },
    MapEntry {
        target: '雠',
        source: '讎',
        glyph_index: 7057,
    },
    MapEntry {
        target: '飏',
        source: '礒',
        glyph_index: 6209,
    },
    MapEntry {
        target: '趔',
        source: '礇',
        glyph_index: 6208,
    },
    MapEntry {
        target: '铖',
        source: '磚',
        glyph_index: 6205,
    },
    MapEntry {
        target: '奂',
        source: '奐',
        glyph_index: 4879,
    },
    MapEntry {
        target: '傩',
        source: '儺',
        glyph_index: 4532,
    },
    MapEntry {
        target: '迨',
        source: '磑',
        glyph_index: 6195,
    },
    MapEntry {
        target: '蔺',
        source: '藺',
        glyph_index: 6801,
    },
    MapEntry {
        target: '篑',
        source: '簣',
        glyph_index: 6344,
    },
    MapEntry {
        target: '婵',
        source: '嬋',
        glyph_index: 4928,
    },
    MapEntry {
        target: '氖',
        source: '碯',
        glyph_index: 6194,
    },
    MapEntry {
        target: '獐',
        source: '碪',
        glyph_index: 6193,
    },
    MapEntry {
        target: '赈',
        source: '賑',
        glyph_index: 3565,
    },
    MapEntry {
        target: '缁',
        source: '緇',
        glyph_index: 6421,
    },
    MapEntry {
        target: '烜',
        source: '碵',
        glyph_index: 6192,
    },
    MapEntry {
        target: '绺',
        source: '碆',
        glyph_index: 6187,
    },
    MapEntry {
        target: '侗',
        source: '硴',
        glyph_index: 6186,
    },
    MapEntry {
        target: '镗',
        source: '礪',
        glyph_index: 6183,
    },
    MapEntry {
        target: '邝',
        source: '砠',
        glyph_index: 6182,
    },
    MapEntry {
        target: '飚',
        source: '礦',
        glyph_index: 6181,
    },
    MapEntry {
        target: '擤',
        source: '矼',
        glyph_index: 6178,
    },
    MapEntry {
        target: '涿',
        source: '矇',
        glyph_index: 6171,
    },
    MapEntry {
        target: '饴',
        source: '飴',
        glyph_index: 1436,
    },
    MapEntry {
        target: '骠',
        source: '瞹',
        glyph_index: 6166,
    },
    MapEntry {
        target: '闱',
        source: '瞶',
        glyph_index: 6165,
    },
    MapEntry {
        target: '骢',
        source: '眛',
        glyph_index: 6147,
    },
    MapEntry {
        target: '橹',
        source: '眥',
        glyph_index: 6145,
    },
    MapEntry {
        target: '酮',
        source: '眞',
        glyph_index: 6144,
    },
    MapEntry {
        target: '谯',
        source: '眤',
        glyph_index: 6143,
    },
    MapEntry {
        target: '颀',
        source: '盻',
        glyph_index: 6138,
    },
    MapEntry {
        target: '馏',
        source: '餾',
        glyph_index: 7545,
    },
    MapEntry {
        target: '诤',
        source: '蘯',
        glyph_index: 6137,
    },
    MapEntry {
        target: '烊',
        source: '盪',
        glyph_index: 6136,
    },
    MapEntry {
        target: '荚',
        source: '莢',
        glyph_index: 6691,
    },
    MapEntry {
        target: '煨',
        source: '盡',
        glyph_index: 6133,
    },
    MapEntry {
        target: '囡',
        source: '皹',
        glyph_index: 6126,
    },
    MapEntry {
        target: '哐',
        source: '皰',
        glyph_index: 6123,
    },
    MapEntry {
        target: '琰',
        source: '皃',
        glyph_index: 6115,
    },
    MapEntry {
        target: '啭',
        source: '囀',
        glyph_index: 4782,
    },
    MapEntry {
        target: '鸩',
        source: '鴆',
        glyph_index: 7694,
    },
    MapEntry {
        target: '绦',
        source: '絛',
        glyph_index: 6413,
    },
    MapEntry {
        target: '掼',
        source: '皀',
        glyph_index: 6114,
    },
    MapEntry {
        target: '鳅',
        source: '鰍',
        glyph_index: 1757,
    },
    MapEntry {
        target: '颌',
        source: '發',
        glyph_index: 6113,
    },
    MapEntry {
        target: '讧',
        source: '訌',
        glyph_index: 6988,
    },
    MapEntry {
        target: '遴',
        source: '癶',
        glyph_index: 6111,
    },
    MapEntry {
        target: '媪',
        source: '媼',
        glyph_index: 4915,
    },
    MapEntry {
        target: '铤',
        source: '癧',
        glyph_index: 6107,
    },
    MapEntry {
        target: '蔻',
        source: '癪',
        glyph_index: 6106,
    },
    MapEntry {
        target: '镭',
        source: '癨',
        glyph_index: 6104,
    },
    MapEntry {
        target: '簦',
        source: '癢',
        glyph_index: 6103,
    },
    MapEntry {
        target: '纥',
        source: '癡',
        glyph_index: 6102,
    },
    MapEntry {
        target: '鲈',
        source: '鱸',
        glyph_index: 7686,
    },
    MapEntry {
        target: '澍',
        source: '癜',
        glyph_index: 6100,
    },
    MapEntry {
        target: '摭',
        source: '癈',
        glyph_index: 6098,
    },
    MapEntry {
        target: '穑',
        source: '穡',
        glyph_index: 6252,
    },
    MapEntry {
        target: '诓',
        source: '瘻',
        glyph_index: 6096,
    },
    MapEntry {
        target: '屣',
        source: '瘉',
        glyph_index: 6087,
    },
    MapEntry {
        target: '荦',
        source: '犖',
        glyph_index: 5945,
    },
    MapEntry {
        target: '钹',
        source: '痳',
        glyph_index: 6084,
    },
    MapEntry {
        target: '玎',
        source: '痲',
        glyph_index: 6083,
    },
    MapEntry {
        target: '茈',
        source: '痃',
        glyph_index: 6065,
    },
    MapEntry {
        target: '囤',
        source: '疳',
        glyph_index: 6064,
    },
    MapEntry {
        target: '怼',
        source: '疂',
        glyph_index: 6057,
    },
    MapEntry {
        target: '栉',
        source: '櫛',
        glyph_index: 2064,
    },
    MapEntry {
        target: '蝼',
        source: '螻',
        glyph_index: 6880,
    },
    MapEntry {
        target: '劢',
        source: '疉',
        glyph_index: 6056,
    },
    MapEntry {
        target: '邛',
        source: '疊',
        glyph_index: 6055,
    },
    MapEntry {
        target: '昱',
        source: '疇',
        glyph_index: 6053,
    },
    MapEntry {
        target: '镊',
        source: '鑷',
        glyph_index: 7381,
    },
    MapEntry {
        target: '瓴',
        source: '當',
        glyph_index: 6051,
    },
    MapEntry {
        target: '汜',
        source: '畭',
        glyph_index: 6049,
    },
    MapEntry {
        target: '蹰',
        source: '畫',
        glyph_index: 6048,
    },
    MapEntry {
        target: '芾',
        source: '畧',
        glyph_index: 6047,
    },
    MapEntry {
        target: '缟',
        source: '縞',
        glyph_index: 2579,
    },
    MapEntry {
        target: '谖',
        source: '畩',
        glyph_index: 6045,
    },
    MapEntry {
        target: '椠',
        source: '槧',
        glyph_index: 5610,
    },
    MapEntry {
        target: '镫',
        source: '畆',
        glyph_index: 6043,
    },
    MapEntry {
        target: '珏',
        source: '畉',
        glyph_index: 6041,
    },
    MapEntry {
        target: '犟',
        source: '畊',
        glyph_index: 6040,
    },
    MapEntry {
        target: '殛',
        source: '畍',
        glyph_index: 6039,
    },
    MapEntry {
        target: '妫',
        source: '畄',
        glyph_index: 6038,
    },
    MapEntry {
        target: '樯',
        source: '檣',
        glyph_index: 5639,
    },
    MapEntry {
        target: '纰',
        source: '紕',
        glyph_index: 6394,
    },
    MapEntry {
        target: '龃',
        source: '齟',
        glyph_index: 7789,
    },
    MapEntry {
        target: '鱿',
        source: '尢',
        glyph_index: 4974,
    },
    MapEntry {
        target: '锌',
        source: '甼',
        glyph_index: 6037,
    },
    MapEntry {
        target: '濑',
        source: '瀬',
        glyph_index: 2918,
    },
    MapEntry {
        target: '骜',
        source: '甦',
        glyph_index: 6035,
    },
    MapEntry {
        target: '峤',
        source: '甞',
        glyph_index: 6034,
    },
    MapEntry {
        target: '丏',
        source: '甕',
        glyph_index: 6032,
    },
    MapEntry {
        target: '鄢',
        source: '甅',
        glyph_index: 6028,
    },
    MapEntry {
        target: '毂',
        source: '轂',
        glyph_index: 7199,
    },
    MapEntry {
        target: '夯',
        source: '甃',
        glyph_index: 6027,
    },
    MapEntry {
        target: '碜',
        source: '瓸',
        glyph_index: 6024,
    },
    MapEntry {
        target: '龉',
        source: '齬',
        glyph_index: 7794,
    },
    MapEntry {
        target: '讣',
        source: '訃',
        glyph_index: 6985,
    },
    MapEntry {
        target: '哙',
        source: '瓱',
        glyph_index: 6023,
    },
    MapEntry {
        target: '崧',
        source: '瓰',
        glyph_index: 6022,
    },
    MapEntry {
        target: '咩',
        source: '瓲',
        glyph_index: 6021,
    },
    MapEntry {
        target: '蕤',
        source: '瓩',
        glyph_index: 6019,
    },
    MapEntry {
        target: '苕',
        source: '瓧',
        glyph_index: 6018,
    },
    MapEntry {
        target: '蕻',
        source: '瓔',
        glyph_index: 6014,
    },
    MapEntry {
        target: '蝈',
        source: '幗',
        glyph_index: 5059,
    },
    MapEntry {
        target: '唁',
        source: '珸',
        glyph_index: 5994,
    },
    MapEntry {
        target: '臬',
        source: '瑯',
        glyph_index: 5992,
    },
    MapEntry {
        target: '篾',
        source: '璢',
        glyph_index: 5990,
    },
    MapEntry {
        target: '佥',
        source: '僉',
        glyph_index: 4512,
    },
    MapEntry {
        target: '撸',
        source: '珎',
        glyph_index: 5984,
    },
    MapEntry {
        target: '逦',
        source: '獻',
        glyph_index: 5980,
    },
    MapEntry {
        target: '锉',
        source: '獵',
        glyph_index: 5979,
    },
    MapEntry {
        target: '蜇',
        source: '獨',
        glyph_index: 5976,
    },
    MapEntry {
        target: '魉',
        source: '魎',
        glyph_index: 7633,
    },
    MapEntry {
        target: '栀',
        source: '梔',
        glyph_index: 5524,
    },
    MapEntry {
        target: '蔫',
        source: '猯',
        glyph_index: 5967,
    },
    MapEntry {
        target: '喑',
        source: '狹',
        glyph_index: 5958,
    },
    MapEntry {
        target: '诶',
        source: '狢',
        glyph_index: 5955,
    },
    MapEntry {
        target: '酶',
        source: '狆',
        glyph_index: 5951,
    },
    MapEntry {
        target: '胺',
        source: '犲',
        glyph_index: 5949,
    },
    MapEntry {
        target: '莼',
        source: '蓴',
        glyph_index: 6756,
    },
    MapEntry {
        target: '鹑',
        source: '鶉',
        glyph_index: 7715,
    },
    MapEntry {
        target: '趼',
        source: '犧',
        glyph_index: 5947,
    },
    MapEntry {
        target: '獭',
        source: '獺',
        glyph_index: 5981,
    },
    MapEntry {
        target: '庋',
        source: '犇',
        glyph_index: 5943,
    },
    MapEntry {
        target: '赓',
        source: '犂',
        glyph_index: 5941,
    },
    MapEntry {
        target: '鸱',
        source: '鴟',
        glyph_index: 7699,
    },
    MapEntry {
        target: '繇',
        source: '牴',
        glyph_index: 5939,
    },
    MapEntry {
        target: '儆',
        source: '牋',
        glyph_index: 5937,
    },
    MapEntry {
        target: '绻',
        source: '綣',
        glyph_index: 6419,
    },
    MapEntry {
        target: '饽',
        source: '牀',
        glyph_index: 5935,
    },
    MapEntry {
        target: '艄',
        source: '爼',
        glyph_index: 5933,
    },
    MapEntry {
        target: '讵',
        source: '爲',
        glyph_index: 5931,
    },
    MapEntry {
        target: '铡',
        source: '爭',
        glyph_index: 5928,
    },
    MapEntry {
        target: '趿',
        source: '爐',
        glyph_index: 5925,
    },
    MapEntry {
        target: '蒺',
        source: '燿',
        glyph_index: 5923,
    },
    MapEntry {
        target: '颛',
        source: '燵',
        glyph_index: 5920,
    },
    MapEntry {
        target: '荠',
        source: '薺',
        glyph_index: 6788,
    },
    MapEntry {
        target: '嘏',
        source: '燬',
        glyph_index: 5918,
    },
    MapEntry {
        target: '娈',
        source: '燒',
        glyph_index: 5913,
    },
    MapEntry {
        target: '骘',
        source: '隲',
        glyph_index: 7438,
    },
    MapEntry {
        target: '岘',
        source: '燗',
        glyph_index: 5910,
    },
    MapEntry {
        target: '鬈',
        source: '熕',
        glyph_index: 5907,
    },
    MapEntry {
        target: '锩',
        source: '燻',
        glyph_index: 5905,
    },
    MapEntry {
        target: '纡',
        source: '紆',
        glyph_index: 6391,
    },
    MapEntry {
        target: '秆',
        source: '稈',
        glyph_index: 6238,
    },
    MapEntry {
        target: '黩',
        source: '黷',
        glyph_index: 7773,
    },
    MapEntry {
        target: '棂',
        source: '櫺',
        glyph_index: 5657,
    },
    MapEntry {
        target: '埙',
        source: '煖',
        glyph_index: 5902,
    },
    MapEntry {
        target: '顼',
        source: '熈',
        glyph_index: 5898,
    },
    MapEntry {
        target: '帼',
        source: '煕',
        glyph_index: 5897,
    },
    MapEntry {
        target: '锷',
        source: '鍔',
        glyph_index: 3344,
    },
    MapEntry {
        target: '祂',
        source: '烋',
        glyph_index: 5889,
    },
    MapEntry {
        target: '啵',
        source: '烱',
        glyph_index: 5883,
    },
    MapEntry {
        target: '诒',
        source: '詒',
        glyph_index: 6995,
    },
    MapEntry {
        target: '缱',
        source: '灣',
        glyph_index: 5879,
    },
    MapEntry {
        target: '鸷',
        source: '鷙',
        glyph_index: 7732,
    },
    MapEntry {
        target: '馑',
        source: '饉',
        glyph_index: 7547,
    },
    MapEntry {
        target: '喱',
        source: '竰',
        glyph_index: 6285,
    },
    MapEntry {
        target: '鳗',
        source: '鰻',
        glyph_index: 1526,
    },
    MapEntry {
        target: '暍',
        source: '灑',
        glyph_index: 5878,
    },
    MapEntry {
        target: '茕',
        source: '煢',
        glyph_index: 5900,
    },
    MapEntry {
        target: '骐',
        source: '騏',
        glyph_index: 7573,
    },
    MapEntry {
        target: '郴',
        source: '瀰',
        glyph_index: 5875,
    },
    MapEntry {
        target: '狲',
        source: '瀁',
        glyph_index: 5866,
    },
    MapEntry {
        target: '髡',
        source: '瀋',
        glyph_index: 5863,
    },
    MapEntry {
        target: '涝',
        source: '濛',
        glyph_index: 5861,
    },
    MapEntry {
        target: '谔',
        source: '諤',
        glyph_index: 7019,
    },
    MapEntry {
        target: '猢',
        source: '濔',
        glyph_index: 5857,
    },
    MapEntry {
        target: '焊',
        source: '駻',
        glyph_index: 7570,
    },
    MapEntry {
        target: '伧',
        source: '濕',
        glyph_index: 5855,
    },
    MapEntry {
        target: '淝',
        source: '濟',
        glyph_index: 5854,
    },
    MapEntry {
        target: '踯',
        source: '躑',
        glyph_index: 7165,
    },
    MapEntry {
        target: '赍',
        source: '齎',
        glyph_index: 7107,
    },
    MapEntry {
        target: '谡',
        source: '謖',
        glyph_index: 7031,
    },
    MapEntry {
        target: '轫',
        source: '澪',
        glyph_index: 5853,
    },
    MapEntry {
        target: '靛',
        source: '濆',
        glyph_index: 5852,
    },
    MapEntry {
        target: '廪',
        source: '廩',
        glyph_index: 5084,
    },
    MapEntry {
        target: '猱',
        source: '澣',
        glyph_index: 5848,
    },
    MapEntry {
        target: '抟',
        source: '摶',
        glyph_index: 5349,
    },
    MapEntry {
        target: '垩',
        source: '堊',
        glyph_index: 4826,
    },
    MapEntry {
        target: '渌',
        source: '澑',
        glyph_index: 5844,
    },
    MapEntry {
        target: '唿',
        source: '濳',
        glyph_index: 5838,
    },
    MapEntry {
        target: '镞',
        source: '鏃',
        glyph_index: 7354,
    },
    MapEntry {
        target: '瓒',
        source: '潛',
        glyph_index: 5837,
    },
    MapEntry {
        target: '嘣',
        source: '澀',
        glyph_index: 5835,
    },
    MapEntry {
        target: '钾',
        source: '澁',
        glyph_index: 5834,
    },
    MapEntry {
        target: '砝',
        source: '滷',
        glyph_index: 5830,
    },
    MapEntry {
        target: '铩',
        source: '滯',
        glyph_index: 5825,
    },
    MapEntry {
        target: '酰',
        source: '滉',
        glyph_index: 5803,
    },
    MapEntry {
        target: '挹',
        source: '溂',
        glyph_index: 5800,
    },
    MapEntry {
        target: '碛',
        source: '磧',
        glyph_index: 6204,
    },
    MapEntry {
        target: '掮',
        source: '滿',
        glyph_index: 5797,
    },
    MapEntry {
        target: '洚',
        source: '渟',
        glyph_index: 5792,
    },
    MapEntry {
        target: '蜢',
        source: '湶',
        glyph_index: 5790,
    },
    MapEntry {
        target: '吖',
        source: '渮',
        glyph_index: 5782,
    },
    MapEntry {
        target: '煊',
        source: '淕',
        glyph_index: 5777,
    },
    MapEntry {
        target: '洱',
        source: '淺',
        glyph_index: 5774,
    },
    MapEntry {
        target: '驽',
        source: '駑',
        glyph_index: 7565,
    },
    MapEntry {
        target: '喁',
        source: '淒',
        glyph_index: 5772,
    },
    MapEntry {
        target: '邬',
        source: '渕',
        glyph_index: 5761,
    },
    MapEntry {
        target: '捭',
        source: '濤',
        glyph_index: 5758,
    },
    MapEntry {
        target: '鳏',
        source: '鰥',
        glyph_index: 7674,
    },
    MapEntry {
        target: '谝',
        source: '諞',
        glyph_index: 7025,
    },
    MapEntry {
        target: '耦',
        source: '浤',
        glyph_index: 5752,
    },
    MapEntry {
        target: '镔',
        source: '洳',
        glyph_index: 5747,
    },
    MapEntry {
        target: '餍',
        source: '洟',
        glyph_index: 5739,
    },
    MapEntry {
        target: '觏',
        source: '覯',
        glyph_index: 6973,
    },
    MapEntry {
        target: '妤',
        source: '泙',
        glyph_index: 5737,
    },
    MapEntry {
        target: '鲫',
        source: '沺',
        glyph_index: 5734,
    },
    MapEntry {
        target: '璩',
        source: '泝',
        glyph_index: 5730,
    },
    MapEntry {
        target: '毽',
        source: '沒',
        glyph_index: 5722,
    },
    MapEntry {
        target: '嗵',
        source: '汳',
        glyph_index: 5721,
    },
    MapEntry {
        target: '筚',
        source: '篳',
        glyph_index: 6339,
    },
    MapEntry {
        target: '铰',
        source: '沚',
        glyph_index: 5716,
    },
    MapEntry {
        target: '姹',
        source: '沍',
        glyph_index: 5715,
    },
    MapEntry {
        target: '秾',
        source: '汢',
        glyph_index: 5712,
    },
    MapEntry {
        target: '潞',
        source: '氣',
        glyph_index: 5709,
    },
    MapEntry {
        target: '竽',
        source: '毬',
        glyph_index: 5699,
    },
    MapEntry {
        target: '撺',
        source: '毟',
        glyph_index: 5698,
    },
    MapEntry {
        target: '枰',
        source: '毆',
        glyph_index: 5695,
    },
    MapEntry {
        target: '莒',
        source: '殼',
        glyph_index: 5694,
    },
    MapEntry {
        target: '氮',
        source: '殲',
        glyph_index: 5690,
    },
    MapEntry {
        target: '阊',
        source: '殪',
        glyph_index: 5687,
    },
    MapEntry {
        target: '庑',
        source: '廡',
        glyph_index: 5082,
    },
    MapEntry {
        target: '阆',
        source: '殕',
        glyph_index: 5684,
    },
    MapEntry {
        target: '珩',
        source: '殘',
        glyph_index: 5683,
    },
    MapEntry {
        target: '犄',
        source: '殀',
        glyph_index: 5679,
    },
    MapEntry {
        target: '獾',
        source: '歸',
        glyph_index: 5676,
    },
    MapEntry {
        target: '猡',
        source: '歡',
        glyph_index: 5675,
    },
    MapEntry {
        target: '摀',
        source: '歐',
        glyph_index: 5670,
    },
    MapEntry {
        target: '龈',
        source: '齦',
        glyph_index: 7792,
    },
    MapEntry {
        target: '勖',
        source: '飮',
        glyph_index: 5666,
    },
    MapEntry {
        target: '崴',
        source: '盜',
        glyph_index: 5664,
    },
    MapEntry {
        target: '锱',
        source: '錙',
        glyph_index: 7332,
    },
    MapEntry {
        target: '莆',
        source: '欟',
        glyph_index: 5661,
    },
    MapEntry {
        target: '孬',
        source: '鬱',
        glyph_index: 5660,
    },
    MapEntry {
        target: '辋',
        source: '欅',
        glyph_index: 5655,
    },
    MapEntry {
        target: '茏',
        source: '蘢',
        glyph_index: 6803,
    },
    MapEntry {
        target: '阄',
        source: '鬮',
        glyph_index: 7626,
    },
    MapEntry {
        target: '伥',
        source: '櫻',
        glyph_index: 5654,
    },
    MapEntry {
        target: '焖',
        source: '櫟',
        glyph_index: 5650,
    },
    MapEntry {
        target: '乜',
        source: '櫑',
        glyph_index: 5649,
    },
    MapEntry {
        target: '訇',
        source: '櫞',
        glyph_index: 5648,
    },
    MapEntry {
        target: '佘',
        source: '蘗',
        glyph_index: 5641,
    },
    MapEntry {
        target: '阏',
        source: '閼',
        glyph_index: 7399,
    },
    MapEntry {
        target: '饫',
        source: '飫',
        glyph_index: 7531,
    },
    MapEntry {
        target: '滦',
        source: '檢',
        glyph_index: 5638,
    },
    MapEntry {
        target: '砉',
        source: '檍',
        glyph_index: 5635,
    },
    MapEntry {
        target: '鹞',
        source: '鷂',
        glyph_index: 7731,
    },
    MapEntry {
        target: '蜮',
        source: '樢',
        glyph_index: 5633,
    },
    MapEntry {
        target: '訾',
        source: '樸',
        glyph_index: 5632,
    },
    MapEntry {
        target: '嫫',
        source: '橸',
        glyph_index: 5626,
    },
    MapEntry {
        target: '锃',
        source: '樶',
        glyph_index: 5625,
    },
    MapEntry {
        target: '谠',
        source: '橲',
        glyph_index: 5624,
    },
    MapEntry {
        target: '悭',
        source: '慳',
        glyph_index: 5213,
    },
    MapEntry {
        target: '钡',
        source: '樌',
        glyph_index: 5623,
    },
    MapEntry {
        target: '绔',
        source: '樓',
        glyph_index: 5621,
    },
    MapEntry {
        target: '汛',
        source: '櫁',
        glyph_index: 5619,
    },
    MapEntry {
        target: '雒',
        source: '樒',
        glyph_index: 5618,
    },
    MapEntry {
        target: '觋',
        source: '覡',
        glyph_index: 6969,
    },
    MapEntry {
        target: '氐',
        source: '槫',
        glyph_index: 5616,
    },
    MapEntry {
        target: '瑭',
        source: '樔',
        glyph_index: 5615,
    },
    MapEntry {
        target: '荞',
        source: '蕎',
        glyph_index: 2003,
    },
    MapEntry {
        target: '龢',
        source: '槭',
        glyph_index: 5614,
    },
    MapEntry {
        target: '诔',
        source: '誄',
        glyph_index: 7004,
    },
    MapEntry {
        target: '榫',
        source: '樞',
        glyph_index: 5613,
    },
    MapEntry {
        target: '綦',
        source: '槹',
        glyph_index: 5608,
    },
    MapEntry {
        target: '夤',
        source: '權',
        glyph_index: 5607,
    },
    MapEntry {
        target: '鲠',
        source: '樛',
        glyph_index: 5605,
    },
    MapEntry {
        target: '兖',
        source: '樂',
        glyph_index: 5604,
    },
    MapEntry {
        target: '滁',
        source: '槨',
        glyph_index: 5603,
    },
    MapEntry {
        target: '荑',
        source: '榠',
        glyph_index: 5598,
    },
    MapEntry {
        target: '泮',
        source: '榑',
        glyph_index: 5597,
    },
    MapEntry {
        target: '朓',
        source: '樮',
        glyph_index: 5596,
    },
    MapEntry {
        target: '邳',
        source: '槝',
        glyph_index: 5592,
    },
    MapEntry {
        target: '挝',
        source: '榾',
        glyph_index: 5588,
    },
    MapEntry {
        target: '佤',
        source: '槓',
        glyph_index: 5587,
    },
    MapEntry {
        target: '橛',
        source: '榿',
        glyph_index: 5585,
    },
    MapEntry {
        target: '撄',
        source: '榮',
        glyph_index: 5583,
    },
    MapEntry {
        target: '彀',
        source: '榲',
        glyph_index: 5582,
    },
    MapEntry {
        target: '仵',
        source: '楪',
        glyph_index: 5581,
    },
    MapEntry {
        target: '荛',
        source: '蕘',
        glyph_index: 6766,
    },
    MapEntry {
        target: '栎',
        source: '檪',
        glyph_index: 5651,
    },
    MapEntry {
        target: '圮',
        source: '榁',
        glyph_index: 5580,
    },
    MapEntry {
        target: '崤',
        source: '楙',
        glyph_index: 5575,
    },
    MapEntry {
        target: '郜',
        source: '楴',
        glyph_index: 5573,
    },
    MapEntry {
        target: '逯',
        source: '椹',
        glyph_index: 5572,
    },
    MapEntry {
        target: '铳',
        source: '銃',
        glyph_index: 2669,
    },
    MapEntry {
        target: '犍',
        source: '楾',
        glyph_index: 5570,
    },
    MapEntry {
        target: '剡',
        source: '楜',
        glyph_index: 5566,
    },
    MapEntry {
        target: '轾',
        source: '輊',
        glyph_index: 7184,
    },
    MapEntry {
        target: '郇',
        source: '棆',
        glyph_index: 5563,
    },
    MapEntry {
        target: '杌',
        source: '椡',
        glyph_index: 5562,
    },
    MapEntry {
        target: '蕲',
        source: '椣',
        glyph_index: 5561,
    },
    MapEntry {
        target: '涪',
        source: '椚',
        glyph_index: 5560,
    },
    MapEntry {
        target: '荨',
        source: '蕁',
        glyph_index: 6768,
    },
    MapEntry {
        target: '狻',
        source: '椪',
        glyph_index: 5559,
    },
    MapEntry {
        target: '阒',
        source: '闃',
        glyph_index: 7405,
    },
    MapEntry {
        target: '牦',
        source: '椨',
        glyph_index: 5558,
    },
    MapEntry {
        target: '缒',
        source: '縋',
        glyph_index: 6446,
    },
    MapEntry {
        target: '荥',
        source: '棯',
        glyph_index: 5557,
    },
    MapEntry {
        target: '扃',
        source: '椥',
        glyph_index: 5554,
    },
    MapEntry {
        target: '洄',
        source: '椄',
        glyph_index: 5551,
    },
    MapEntry {
        target: '黢',
        source: '齲',
        glyph_index: 7797,
    },
    MapEntry {
        target: '辏',
        source: '椶',
        glyph_index: 5549,
    },
    MapEntry {
        target: '畀',
        source: '棧',
        glyph_index: 5547,
    },
    MapEntry {
        target: '嚭',
        source: '棔',
        glyph_index: 5546,
    },
    MapEntry {
        target: '沤',
        source: '椌',
        glyph_index: 5544,
    },
    MapEntry {
        target: '癣',
        source: '癬',
        glyph_index: 6108,
    },
    MapEntry {
        target: '洹',
        source: '棡',
        glyph_index: 5543,
    },
    MapEntry {
        target: '屌',
        source: '椦',
        glyph_index: 5542,
    },
    MapEntry {
        target: '琊',
        source: '椢',
        glyph_index: 5541,
    },
    MapEntry {
        target: '蠲',
        source: '椈',
        glyph_index: 5539,
    },
    MapEntry {
        target: '桡',
        source: '橈',
        glyph_index: 5631,
    },
    MapEntry {
        target: '瓤',
        source: '棊',
        glyph_index: 5538,
    },
    MapEntry {
        target: '唢',
        source: '桾',
        glyph_index: 5536,
    },
    MapEntry {
        target: '怃',
        source: '憮',
        glyph_index: 5236,
    },
    MapEntry {
        target: '沆',
        source: '梍',
        glyph_index: 5535,
    },
    MapEntry {
        target: '泫',
        source: '梺',
        glyph_index: 5533,
    },
    MapEntry {
        target: '鲛',
        source: '鮫',
        glyph_index: 2456,
    },
    MapEntry {
        target: '骝',
        source: '梠',
        glyph_index: 5532,
    },
    MapEntry {
        target: '蚍',
        source: '梹',
        glyph_index: 5529,
    },
    MapEntry {
        target: '亍',
        source: '檮',
        glyph_index: 5528,
    },
    MapEntry {
        target: '嚯',
        source: '梛',
        glyph_index: 5526,
    },
    MapEntry {
        target: '旮',
        source: '條',
        glyph_index: 5525,
    },
    MapEntry {
        target: '砣',
        source: '桿',
        glyph_index: 5520,
    },
    MapEntry {
        target: '疡',
        source: '瘍',
        glyph_index: 6086,
    },
    MapEntry {
        target: '帔',
        source: '桙',
        glyph_index: 5517,
    },
    MapEntry {
        target: '猕',
        source: '栫',
        glyph_index: 5516,
    },
    MapEntry {
        target: '呖',
        source: '桍',
        glyph_index: 5512,
    },
    MapEntry {
        target: '骖',
        source: '驂',
        glyph_index: 7580,
    },
    MapEntry {
        target: '疴',
        source: '痾',
        glyph_index: 6077,
    },
    MapEntry {
        target: '懔',
        source: '懍',
        glyph_index: 5247,
    },
    MapEntry {
        target: '崐',
        source: '栞',
        glyph_index: 5508,
    },
    MapEntry {
        target: '郓',
        source: '檜',
        glyph_index: 5507,
    },
    MapEntry {
        target: '刿',
        source: '柧',
        glyph_index: 5506,
    },
    MapEntry {
        target: '舢',
        source: '柆',
        glyph_index: 5505,
    },
    MapEntry {
        target: '缃',
        source: '柎',
        glyph_index: 5504,
    },
    MapEntry {
        target: '桉',
        source: '枹',
        glyph_index: 5503,
    },
    MapEntry {
        target: '赀',
        source: '貲',
        glyph_index: 7088,
    },
    MapEntry {
        target: '屺',
        source: '柮',
        glyph_index: 5502,
    },
    MapEntry {
        target: '癯',
        source: '柤',
        glyph_index: 5498,
    },
    MapEntry {
        target: '龋',
        source: '枴',
        glyph_index: 5493,
    },
    MapEntry {
        target: '搠',
        source: '枅',
        glyph_index: 5490,
    },
    MapEntry {
        target: '铣',
        source: '銑',
        glyph_index: 3020,
    },
    MapEntry {
        target: '冼',
        source: '枡',
        glyph_index: 5489,
    },
    MapEntry {
        target: '辎',
        source: '輜',
        glyph_index: 7190,
    },
    MapEntry {
        target: '胪',
        source: '臚',
        glyph_index: 6617,
    },
    MapEntry {
        target: '矾',
        source: '礬',
        glyph_index: 6212,
    },
    MapEntry {
        target: '堞',
        source: '枦',
        glyph_index: 5488,
    },
    MapEntry {
        target: '茔',
        source: '枌',
        glyph_index: 5486,
    },
    MapEntry {
        target: '橥',
        source: '枩',
        glyph_index: 5483,
    },
    MapEntry {
        target: '氅',
        source: '杤',
        glyph_index: 5480,
    },
    MapEntry {
        target: '沔',
        source: '杣',
        glyph_index: 5479,
    },
    MapEntry {
        target: '镁',
        source: '杙',
        glyph_index: 5478,
    },
    MapEntry {
        target: '澶',
        source: '朷',
        glyph_index: 5474,
    },
    MapEntry {
        target: '觯',
        source: '朸',
        glyph_index: 5473,
    },
    MapEntry {
        target: '阃',
        source: '杁',
        glyph_index: 5472,
    },
    MapEntry {
        target: '娌',
        source: '朿',
        glyph_index: 5470,
    },
    MapEntry {
        target: '嫘',
        source: '朮',
        glyph_index: 5469,
    },
    MapEntry {
        target: '喈',
        source: '朞',
        glyph_index: 5465,
    },
    MapEntry {
        target: '鼍',
        source: '朖',
        glyph_index: 5464,
    },
    MapEntry {
        target: '泷',
        source: '滝',
        glyph_index: 3177,
    },
    MapEntry {
        target: '鲇',
        source: '鮎',
        glyph_index: 1439,
    },
    MapEntry {
        target: '粑',
        source: '朏',
        glyph_index: 5463,
    },
    MapEntry {
        target: '偓',
        source: '曵',
        glyph_index: 5461,
    },
    MapEntry {
        target: '殽',
        source: '曠',
        glyph_index: 5456,
    },
    MapEntry {
        target: '蘼',
        source: '曚',
        glyph_index: 5455,
    },
    MapEntry {
        target: '厝',
        source: '暸',
        glyph_index: 5453,
    },
    MapEntry {
        target: '俇',
        source: '暼',
        glyph_index: 5451,
    },
    MapEntry {
        target: '鼙',
        source: '曉',
        glyph_index: 5449,
    },
    MapEntry {
        target: '侔',
        source: '暘',
        glyph_index: 5445,
    },
    MapEntry {
        target: '戆',
        source: '暎',
        glyph_index: 5442,
    },
    MapEntry {
        target: '墉',
        source: '暃',
        glyph_index: 5440,
    },
    MapEntry {
        target: '虺',
        source: '晢',
        glyph_index: 5438,
    },
    MapEntry {
        target: '椤',
        source: '晧',
        glyph_index: 5435,
    },
    MapEntry {
        target: '旯',
        source: '晝',
        glyph_index: 5433,
    },
    MapEntry {
        target: '岿',
        source: '晉',
        glyph_index: 5430,
    },
    MapEntry {
        target: '酽',
        source: '晄',
        glyph_index: 5429,
    },
    MapEntry {
        target: '擀',
        source: '昜',
        glyph_index: 5427,
    },
    MapEntry {
        target: '鹌',
        source: '旡',
        glyph_index: 5417,
    },
    MapEntry {
        target: '硌',
        source: '旙',
        glyph_index: 5415,
    },
    MapEntry {
        target: '聱',
        source: '旛',
        glyph_index: 5414,
    },
    MapEntry {
        target: '垆',
        source: '斷',
        glyph_index: 5407,
    },
    MapEntry {
        target: '鳝',
        source: '變',
        glyph_index: 5403,
    },
    MapEntry {
        target: '妯',
        source: '斂',
        glyph_index: 5401,
    },
    MapEntry {
        target: '飧',
        source: '喰',
        glyph_index: 2057,
    },
    MapEntry {
        target: '馐',
        source: '數',
        glyph_index: 5400,
    },
    MapEntry {
        target: '阋',
        source: '鬩',
        glyph_index: 7624,
    },
    MapEntry {
        target: '韫',
        source: '敘',
        glyph_index: 5396,
    },
    MapEntry {
        target: '焐',
        source: '敍',
        glyph_index: 5395,
    },
    MapEntry {
        target: '暌',
        source: '攷',
        glyph_index: 5388,
    },
    MapEntry {
        target: '蚝',
        source: '攴',
        glyph_index: 5386,
    },
    MapEntry {
        target: '笤',
        source: '攜',
        glyph_index: 5381,
    },
    MapEntry {
        target: '锟',
        source: '擽',
        glyph_index: 5379,
    },
    MapEntry {
        target: '髹',
        source: '擴',
        glyph_index: 5375,
    },
    MapEntry {
        target: '轭',
        source: '軛',
        glyph_index: 7178,
    },
    MapEntry {
        target: '藿',
        source: '擶',
        glyph_index: 5374,
    },
    MapEntry {
        target: '赜',
        source: '擡',
        glyph_index: 5369,
    },
    MapEntry {
        target: '芰',
        source: '舉',
        glyph_index: 5367,
    },
    MapEntry {
        target: '畲',
        source: '擧',
        glyph_index: 5366,
    },
    MapEntry {
        target: '傧',
        source: '據',
        glyph_index: 5358,
    },
    MapEntry {
        target: '嫽',
        source: '攪',
        glyph_index: 5351,
    },
    MapEntry {
        target: '猓',
        source: '摎',
        glyph_index: 5350,
    },
    MapEntry {
        target: '脔',
        source: '臠',
        glyph_index: 6619,
    },
    MapEntry {
        target: '潋',
        source: '瀲',
        glyph_index: 5877,
    },
    MapEntry {
        target: '伛',
        source: '傴',
        glyph_index: 4510,
    },
    MapEntry {
        target: '羼',
        source: '搨',
        glyph_index: 5345,
    },
    MapEntry {
        target: '珰',
        source: '搗',
        glyph_index: 5344,
    },
    MapEntry {
        target: '郅',
        source: '攝',
        glyph_index: 5343,
    },
    MapEntry {
        target: '鲵',
        source: '鯢',
        glyph_index: 7654,
    },
    MapEntry {
        target: '鳍',
        source: '鰭',
        glyph_index: 3800,
    },
    MapEntry {
        target: '坼',
        source: '搆',
        glyph_index: 5339,
    },
    MapEntry {
        target: '坻',
        source: '搖',
        glyph_index: 5337,
    },
    MapEntry {
        target: '琬',
        source: '掵',
        glyph_index: 5325,
    },
    MapEntry {
        target: '痫',
        source: '癇',
        glyph_index: 6097,
    },
    MapEntry {
        target: '鲂',
        source: '魴',
        glyph_index: 7636,
    },
    MapEntry {
        target: '洇',
        source: '掟',
        glyph_index: 5324,
    },
    MapEntry {
        target: '阈',
        source: '閾',
        glyph_index: 7402,
    },
    MapEntry {
        target: '烷',
        source: '掫',
        glyph_index: 5319,
    },
    MapEntry {
        target: '锰',
        source: '掎',
        glyph_index: 5317,
    },
    MapEntry {
        target: '枥',
        source: '櫪',
        glyph_index: 5653,
    },
    MapEntry {
        target: '惝',
        source: '挾',
        glyph_index: 5312,
    },
    MapEntry {
        target: '铙',
        source: '鐃',
        glyph_index: 7362,
    },
    MapEntry {
        target: '痱',
        source: '拵',
        glyph_index: 5310,
    },
    MapEntry {
        target: '鄯',
        source: '挧',
        glyph_index: 5306,
    },
    MapEntry {
        target: '愦',
        source: '挌',
        glyph_index: 5303,
    },
    MapEntry {
        target: '苌',
        source: '萇',
        glyph_index: 6714,
    },
    MapEntry {
        target: '痈',
        source: '癰',
        glyph_index: 6109,
    },
    MapEntry {
        target: '璎',
        source: '珱',
        glyph_index: 6015,
    },
    MapEntry {
        target: '顸',
        source: '擔',
        glyph_index: 5294,
    },
    MapEntry {
        target: '鳎',
        source: '拑',
        glyph_index: 5289,
    },
    MapEntry {
        target: '珉',
        source: '抃',
        glyph_index: 5286,
    },
    MapEntry {
        target: '碘',
        source: '抂',
        glyph_index: 5279,
    },
    MapEntry {
        target: '趸',
        source: '扨',
        glyph_index: 5277,
    },
    MapEntry {
        target: '洮',
        source: '扠',
        glyph_index: 5276,
    },
    MapEntry {
        target: '翮',
        source: '扞',
        glyph_index: 5273,
    },
    MapEntry {
        target: '苎',
        source: '苧',
        glyph_index: 3282,
    },
    MapEntry {
        target: '瑗',
        source: '戲',
        glyph_index: 5269,
    },
    MapEntry {
        target: '氡',
        source: '戞',
        glyph_index: 5264,
    },
    MapEntry {
        target: '跹',
        source: '戉',
        glyph_index: 5259,
    },
    MapEntry {
        target: '芡',
        source: '戀',
        glyph_index: 5257,
    },
    MapEntry {
        target: '蚜',
        source: '懼',
        glyph_index: 5255,
    },
    MapEntry {
        target: '锨',
        source: '懽',
        glyph_index: 5254,
    },
    MapEntry {
        target: '朊',
        source: '懺',
        glyph_index: 5251,
    },
    MapEntry {
        target: '媸',
        source: '憺',
        glyph_index: 5244,
    },
    MapEntry {
        target: '鋈',
        source: '懆',
        glyph_index: 5243,
    },
    MapEntry {
        target: '嘞',
        source: '懃',
        glyph_index: 5242,
    },
    MapEntry {
        target: '彘',
        source: '懷',
        glyph_index: 5240,
    },
    MapEntry {
        target: '郧',
        source: '應',
        glyph_index: 5239,
    },
    MapEntry {
        target: '缣',
        source: '憑',
        glyph_index: 5234,
    },
    MapEntry {
        target: '曛',
        source: '憇',
        glyph_index: 5229,
    },
    MapEntry {
        target: '茀',
        source: '憖',
        glyph_index: 5228,
    },
    MapEntry {
        target: '嬛',
        source: '憙',
        glyph_index: 5227,
    },
    MapEntry {
        target: '潍',
        source: '慓',
        glyph_index: 5225,
    },
    MapEntry {
        target: '伢',
        source: '慱',
        glyph_index: 5222,
    },
    MapEntry {
        target: '颟',
        source: '慥',
        glyph_index: 5221,
    },
    MapEntry {
        target: '佾',
        source: '慯',
        glyph_index: 5220,
    },
    MapEntry {
        target: '谰',
        source: '慴',
        glyph_index: 5219,
    },
    MapEntry {
        target: '搿',
        source: '慙',
        glyph_index: 5216,
    },
    MapEntry {
        target: '笾',
        source: '慘',
        glyph_index: 5215,
    },
    MapEntry {
        target: '豳',
        source: '慄',
        glyph_index: 5212,
    },
    MapEntry {
        target: '轱',
        source: '愽',
        glyph_index: 5210,
    },
    MapEntry {
        target: '戋',
        source: '戔',
        glyph_index: 5262,
    },
    MapEntry {
        target: '舨',
        source: '愼',
        glyph_index: 5207,
    },
    MapEntry {
        target: '蕖',
        source: '惱',
        glyph_index: 5198,
    },
    MapEntry {
        target: '闼',
        source: '闥',
        glyph_index: 7413,
    },
    MapEntry {
        target: '裰',
        source: '愡',
        glyph_index: 5196,
    },
    MapEntry {
        target: '浞',
        source: '愃',
        glyph_index: 5195,
    },
    MapEntry {
        target: '诎',
        source: '惷',
        glyph_index: 5191,
    },
    MapEntry {
        target: '妗',
        source: '悽',
        glyph_index: 5183,
    },
    MapEntry {
        target: '囝',
        source: '忰',
        glyph_index: 5182,
    },
    MapEntry {
        target: '炔',
        source: '惓',
        glyph_index: 5180,
    },
    MapEntry {
        target: '踬',
        source: '躓',
        glyph_index: 7164,
    },
    MapEntry {
        target: '葚',
        source: '惡',
        glyph_index: 5177,
    },
    MapEntry {
        target: '窳',
        source: '悋',
        glyph_index: 5176,
    },
    MapEntry {
        target: '窣',
        source: '悧',
        glyph_index: 5175,
    },
    MapEntry {
        target: '绂',
        source: '悗',
        glyph_index: 5173,
    },
    MapEntry {
        target: '邗',
        source: '悁',
        glyph_index: 5165,
    },
    MapEntry {
        target: '髌',
        source: '恆',
        glyph_index: 5156,
    },
    MapEntry {
        target: '戢',
        source: '恊',
        glyph_index: 5155,
    },
    MapEntry {
        target: '邶',
        source: '恟',
        glyph_index: 5154,
    },
    MapEntry {
        target: '兕',
        source: '恷',
        glyph_index: 5153,
    },
    MapEntry {
        target: '鹬',
        source: '鷸',
        glyph_index: 7734,
    },
    MapEntry {
        target: '蕡',
        source: '怺',
        glyph_index: 5149,
    },
    MapEntry {
        target: '喾',
        source: '怱',
        glyph_index: 5143,
    },
    MapEntry {
        target: '缗',
        source: '緡',
        glyph_index: 6437,
    },
    MapEntry {
        target: '楂',
        source: '怐',
        glyph_index: 5140,
    },
    MapEntry {
        target: '屙',
        source: '恠',
        glyph_index: 5138,
    },
    MapEntry {
        target: '饧',
        source: '悳',
        glyph_index: 5135,
    },
    MapEntry {
        target: '氰',
        source: '從',
        glyph_index: 5122,
    },
    MapEntry {
        target: '掊',
        source: '徑',
        glyph_index: 5120,
    },
    MapEntry {
        target: '鎏',
        source: '彿',
        glyph_index: 5117,
    },
    MapEntry {
        target: '辂',
        source: '輅',
        glyph_index: 7185,
    },
    MapEntry {
        target: '烯',
        source: '徃',
        glyph_index: 5115,
    },
    MapEntry {
        target: '狺',
        source: '彡',
        glyph_index: 5111,
    },
    MapEntry {
        target: '墀',
        source: '彙',
        glyph_index: 5110,
    },
    MapEntry {
        target: '戗',
        source: '彑',
        glyph_index: 5107,
    },
    MapEntry {
        target: '崂',
        source: '彎',
        glyph_index: 5105,
    },
    MapEntry {
        target: '晷',
        source: '彌',
        glyph_index: 5104,
    },
    MapEntry {
        target: '遄',
        source: '彈',
        glyph_index: 5103,
    },
    MapEntry {
        target: '蘧',
        source: '彁',
        glyph_index: 5102,
    },
    MapEntry {
        target: '罴',
        source: '羆',
        glyph_index: 6502,
    },
    MapEntry {
        target: '隳',
        source: '弸',
        glyph_index: 5101,
    },
    MapEntry {
        target: '媵',
        source: '弖',
        glyph_index: 5098,
    },
    MapEntry {
        target: '鸬',
        source: '彜',
        glyph_index: 5095,
    },
    MapEntry {
        target: '捃',
        source: '弉',
        glyph_index: 5093,
    },
    MapEntry {
        target: '傈',
        source: '廸',
        glyph_index: 5090,
    },
    MapEntry {
        target: '渑',
        source: '廴',
        glyph_index: 5089,
    },
    MapEntry {
        target: '酆',
        source: '廰',
        glyph_index: 5088,
    },
    MapEntry {
        target: '椟',
        source: '廳',
        glyph_index: 5087,
    },
    MapEntry {
        target: '睃',
        source: '廱',
        glyph_index: 5086,
    },
    MapEntry {
        target: '鞯',
        source: '廢',
        glyph_index: 5081,
    },
    MapEntry {
        target: '缇',
        source: '廚',
        glyph_index: 5079,
    },
    MapEntry {
        target: '葳',
        source: '廝',
        glyph_index: 5078,
    },
    MapEntry {
        target: '醚',
        source: '廣',
        glyph_index: 5077,
    },
    MapEntry {
        target: '嘬',
        source: '廏',
        glyph_index: 5075,
    },
    MapEntry {
        target: '酹',
        source: '廐',
        glyph_index: 5074,
    },
    MapEntry {
        target: '诼',
        source: '廈',
        glyph_index: 5073,
    },
    MapEntry {
        target: '嫠',
        source: '廁',
        glyph_index: 5071,
    },
    MapEntry {
        target: '狍',
        source: '幵',
        glyph_index: 5065,
    },
    MapEntry {
        target: '鳃',
        source: '鰓',
        glyph_index: 7665,
    },
    MapEntry {
        target: '裒',
        source: '幤',
        glyph_index: 5063,
    },
    MapEntry {
        target: '悝',
        source: '幎',
        glyph_index: 5058,
    },
    MapEntry {
        target: '黾',
        source: '黽',
        glyph_index: 7777,
    },
    MapEntry {
        target: '毖',
        source: '帶',
        glyph_index: 5053,
    },
    MapEntry {
        target: '唣',
        source: '帋',
        glyph_index: 5048,
    },
    MapEntry {
        target: '澧',
        source: '巵',
        glyph_index: 5047,
    },
    MapEntry {
        target: '圩',
        source: '巛',
        glyph_index: 5044,
    },
    MapEntry {
        target: '鸶',
        source: '巖',
        glyph_index: 5043,
    },
    MapEntry {
        target: '枘',
        source: '巉',
        glyph_index: 5039,
    },
    MapEntry {
        target: '荇',
        source: '嶐',
        glyph_index: 5036,
    },
    MapEntry {
        target: '瀣',
        source: '嶽',
        glyph_index: 5035,
    },
    MapEntry {
        target: '狁',
        source: '嶮',
        glyph_index: 5034,
    },
    MapEntry {
        target: '螭',
        source: '嶬',
        glyph_index: 5033,
    },
    MapEntry {
        target: '幛',
        source: '嶝',
        glyph_index: 5032,
    },
    MapEntry {
        target: '踔',
        source: '嵶',
        glyph_index: 5027,
    },
    MapEntry {
        target: '棻',
        source: '嵳',
        glyph_index: 5026,
    },
    MapEntry {
        target: '蟊',
        source: '嵎',
        glyph_index: 5023,
    },
    MapEntry {
        target: '锒',
        source: '嵒',
        glyph_index: 5022,
    },
    MapEntry {
        target: '铗',
        source: '鋏',
        glyph_index: 7324,
    },
    MapEntry {
        target: '蛱',
        source: '崘',
        glyph_index: 5020,
    },
    MapEntry {
        target: '鲑',
        source: '鮭',
        glyph_index: 2439,
    },
    MapEntry {
        target: '哞',
        source: '崙',
        glyph_index: 5019,
    },
    MapEntry {
        target: '阍',
        source: '崚',
        glyph_index: 5018,
    },
    MapEntry {
        target: '陔',
        source: '崑',
        glyph_index: 5015,
    },
    MapEntry {
        target: '麂',
        source: '崟',
        glyph_index: 5013,
    },
    MapEntry {
        target: '酡',
        source: '嵜',
        glyph_index: 5012,
    },
    MapEntry {
        target: '棰',
        source: '崕',
        glyph_index: 5010,
    },
    MapEntry {
        target: '伕',
        source: '崋',
        glyph_index: 5009,
    },
    MapEntry {
        target: '辚',
        source: '嶌',
        glyph_index: 5007,
    },
    MapEntry {
        target: '谳',
        source: '峺',
        glyph_index: 5005,
    },
    MapEntry {
        target: '祧',
        source: '峽',
        glyph_index: 5004,
    },
    MapEntry {
        target: '鍪',
        source: '峩',
        glyph_index: 5003,
    },
    MapEntry {
        target: '莴',
        source: '萵',
        glyph_index: 6739,
    },
    MapEntry {
        target: '邾',
        source: '峇',
        glyph_index: 5001,
    },
    MapEntry {
        target: '迕',
        source: '岾',
        glyph_index: 5000,
    },
    MapEntry {
        target: '颃',
        source: '頏',
        glyph_index: 7507,
    },
    MapEntry {
        target: '獒',
        source: '峅',
        glyph_index: 4999,
    },
    MapEntry {
        target: '郏',
        source: '岼',
        glyph_index: 4997,
    },
    MapEntry {
        target: '邙',
        source: '岶',
        glyph_index: 4996,
    },
    MapEntry {
        target: '龅',
        source: '岻',
        glyph_index: 4995,
    },
    MapEntry {
        target: '涞',
        source: '妛',
        glyph_index: 4993,
    },
    MapEntry {
        target: '疠',
        source: '癘',
        glyph_index: 6101,
    },
    MapEntry {
        target: '舄',
        source: '潟',
        glyph_index: 1758,
    },
    MapEntry {
        target: '褰',
        source: '屶',
        glyph_index: 4988,
    },
    MapEntry {
        target: '莳',
        source: '蒔',
        glyph_index: 2549,
    },
    MapEntry {
        target: '璟',
        source: '乢',
        glyph_index: 4987,
    },
    MapEntry {
        target: '禛',
        source: '屮',
        glyph_index: 4986,
    },
    MapEntry {
        target: '垭',
        source: '屬',
        glyph_index: 4985,
    },
    MapEntry {
        target: '铧',
        source: '屓',
        glyph_index: 4981,
    },
    MapEntry {
        target: '呒',
        source: '嘸',
        glyph_index: 4762,
    },
    MapEntry {
        target: '铬',
        source: '屆',
        glyph_index: 4979,
    },
    MapEntry {
        target: '觇',
        source: '覘',
        glyph_index: 6968,
    },
    MapEntry {
        target: '赉',
        source: '賚',
        glyph_index: 7096,
    },
    MapEntry {
        target: '蟮',
        source: '尨',
        glyph_index: 4975,
    },
    MapEntry {
        target: '哓',
        source: '尠',
        glyph_index: 4973,
    },
    MapEntry {
        target: '凇',
        source: '尓',
        glyph_index: 4972,
    },
    MapEntry {
        target: '幂',
        source: '冪',
        glyph_index: 4560,
    },
    MapEntry {
        target: '迮',
        source: '對',
        glyph_index: 4971,
    },
    MapEntry {
        target: '镒',
        source: '鎰',
        glyph_index: 7344,
    },
    MapEntry {
        target: '鲋',
        source: '鮒',
        glyph_index: 3864,
    },
    MapEntry {
        target: '蛴',
        source: '專',
        glyph_index: 4970,
    },
    MapEntry {
        target: '蝰',
        source: '將',
        glyph_index: 4969,
    },
    MapEntry {
        target: '饔',
        source: '尅',
        glyph_index: 4968,
    },
    MapEntry {
        target: '轳',
        source: '轤',
        glyph_index: 7209,
    },
    MapEntry {
        target: '咴',
        source: '寳',
        glyph_index: 4967,
    },
    MapEntry {
        target: '谵',
        source: '譫',
        glyph_index: 7049,
    },
    MapEntry {
        target: '鹈',
        source: '鵜',
        glyph_index: 1516,
    },
    MapEntry {
        target: '齑',
        source: '齏',
        glyph_index: 7502,
    },
    MapEntry {
        target: '鄜',
        source: '寶',
        glyph_index: 4966,
    },
    MapEntry {
        target: '厍',
        source: '寫',
        glyph_index: 4964,
    },
    MapEntry {
        target: '褡',
        source: '寢',
        glyph_index: 4961,
    },
    MapEntry {
        target: '跸',
        source: '蹕',
        glyph_index: 7154,
    },
    MapEntry {
        target: '荩',
        source: '實',
        glyph_index: 4960,
    },
    MapEntry {
        target: '洎',
        source: '寉',
        glyph_index: 4956,
    },
    MapEntry {
        target: '麇',
        source: '寃',
        glyph_index: 4954,
    },
    MapEntry {
        target: '絷',
        source: '斈',
        glyph_index: 4948,
    },
    MapEntry {
        target: '僳',
        source: '學',
        glyph_index: 4947,
    },
    MapEntry {
        target: '芗',
        source: '孅',
        glyph_index: 4936,
    },
    MapEntry {
        target: '酚',
        source: '竕',
        glyph_index: 6276,
    },
    MapEntry {
        target: '骀',
        source: '駘',
        glyph_index: 7564,
    },
    MapEntry {
        target: '蕞',
        source: '孃',
        glyph_index: 4935,
    },
    MapEntry {
        target: '溆',
        source: '嬾',
        glyph_index: 4934,
    },
    MapEntry {
        target: '鳕',
        source: '鱈',
        glyph_index: 3208,
    },
    MapEntry {
        target: '爝',
        source: '嬶',
        glyph_index: 4933,
    },
    MapEntry {
        target: '峄',
        source: '嫐',
        glyph_index: 4931,
    },
    MapEntry {
        target: '粜',
        source: '糶',
        glyph_index: 6389,
    },
    MapEntry {
        target: '瓿',
        source: '嫺',
        glyph_index: 4925,
    },
    MapEntry {
        target: '颙',
        source: '娵',
        glyph_index: 4910,
    },
    MapEntry {
        target: '沣',
        source: '婬',
        glyph_index: 4908,
    },
    MapEntry {
        target: '郯',
        source: '娚',
        glyph_index: 4906,
    },
    MapEntry {
        target: '盱',
        source: '姙',
        glyph_index: 4899,
    },
    MapEntry {
        target: '峁',
        source: '侫',
        glyph_index: 4892,
    },
    MapEntry {
        target: '鸹',
        source: '妝',
        glyph_index: 4890,
    },
    MapEntry {
        target: '琯',
        source: '奬',
        glyph_index: 4886,
    },
    MapEntry {
        target: '艿',
        source: '奧',
        glyph_index: 4885,
    },
    MapEntry {
        target: '茍',
        source: '竒',
        glyph_index: 4877,
    },
    MapEntry {
        target: '墒',
        source: '夲',
        glyph_index: 4874,
    },
    MapEntry {
        target: '薏',
        source: '夬',
        glyph_index: 4872,
    },
    MapEntry {
        target: '舻',
        source: '艫',
        glyph_index: 6646,
    },
    MapEntry {
        target: '缑',
        source: '夛',
        glyph_index: 4869,
    },
    MapEntry {
        target: '谮',
        source: '譛',
        glyph_index: 7047,
    },
    MapEntry {
        target: '驺',
        source: '夐',
        glyph_index: 4868,
    },
    MapEntry {
        target: '鹚',
        source: '壽',
        glyph_index: 4865,
    },
    MapEntry {
        target: '踅',
        source: '壼',
        glyph_index: 4864,
    },
    MapEntry {
        target: '眬',
        source: '壻',
        glyph_index: 4863,
    },
    MapEntry {
        target: '怿',
        source: '懌',
        glyph_index: 5237,
    },
    MapEntry {
        target: '屄',
        source: '壺',
        glyph_index: 4861,
    },
    MapEntry {
        target: '鹆',
        source: '壯',
        glyph_index: 4860,
    },
    MapEntry {
        target: '犴',
        source: '壜',
        glyph_index: 4857,
    },
    MapEntry {
        target: '儋',
        source: '壥',
        glyph_index: 4856,
    },
    MapEntry {
        target: '逄',
        source: '壘',
        glyph_index: 4855,
    },
    MapEntry {
        target: '嫱',
        source: '壗',
        glyph_index: 4853,
    },
    MapEntry {
        target: '迓',
        source: '壓',
        glyph_index: 4851,
    },
    MapEntry {
        target: '侂',
        source: '墮',
        glyph_index: 4849,
    },
    MapEntry {
        target: '秫',
        source: '墸',
        glyph_index: 4848,
    },
    MapEntry {
        target: '滹',
        source: '墻',
        glyph_index: 4847,
    },
    MapEntry {
        target: '蔌',
        source: '壞',
        glyph_index: 4846,
    },
    MapEntry {
        target: '噻',
        source: '墺',
        glyph_index: 4845,
    },
    MapEntry {
        target: '澌',
        source: '墫',
        glyph_index: 4844,
    },
    MapEntry {
        target: '洧',
        source: '墹',
        glyph_index: 4842,
    },
    MapEntry {
        target: '缯',
        source: '堽',
        glyph_index: 4839,
    },
    MapEntry {
        target: '扦',
        source: '塰',
        glyph_index: 4836,
    },
    MapEntry {
        target: '裎',
        source: '塲',
        glyph_index: 4832,
    },
    MapEntry {
        target: '邠',
        source: '堝',
        glyph_index: 4831,
    },
    MapEntry {
        target: '镏',
        source: '埣',
        glyph_index: 4828,
    },
    MapEntry {
        target: '琚',
        source: '埖',
        glyph_index: 4827,
    },
    MapEntry {
        target: '秭',
        source: '埓',
        glyph_index: 4825,
    },
    MapEntry {
        target: '錾',
        source: '鏨',
        glyph_index: 7351,
    },
    MapEntry {
        target: '棼',
        source: '埆',
        glyph_index: 4822,
    },
    MapEntry {
        target: '髋',
        source: '垰',
        glyph_index: 4820,
    },
    MapEntry {
        target: '戬',
        source: '垪',
        glyph_index: 4819,
    },
    MapEntry {
        target: '枨',
        source: '垤',
        glyph_index: 4818,
    },
    MapEntry {
        target: '葸',
        source: '垳',
        glyph_index: 4817,
    },
    MapEntry {
        target: '漭',
        source: '垉',
        glyph_index: 4814,
    },
    MapEntry {
        target: '荸',
        source: '坿',
        glyph_index: 4813,
    },
    MapEntry {
        target: '馕',
        source: '垈',
        glyph_index: 4811,
    },
    MapEntry {
        target: '铆',
        source: '鉚',
        glyph_index: 7323,
    },
    MapEntry {
        target: '堠',
        source: '埀',
        glyph_index: 4810,
    },
    MapEntry {
        target: '濉',
        source: '坩',
        glyph_index: 4809,
    },
    MapEntry {
        target: '葑',
        source: '圸',
        glyph_index: 4804,
    },
    MapEntry {
        target: '璘',
        source: '圷',
        glyph_index: 4803,
    },
    MapEntry {
        target: '蹓',
        source: '圦',
        glyph_index: 4802,
    },
    MapEntry {
        target: '鳟',
        source: '鱒',
        glyph_index: 4027,
    },
    MapEntry {
        target: '氟',
        source: '團',
        glyph_index: 4798,
    },
    MapEntry {
        target: '圹',
        source: '壙',
        glyph_index: 4854,
    },
    MapEntry {
        target: '榉',
        source: '圍',
        glyph_index: 4796,
    },
    MapEntry {
        target: '伲',
        source: '國',
        glyph_index: 4795,
    },
    MapEntry {
        target: '锞',
        source: '圀',
        glyph_index: 4790,
    },
    MapEntry {
        target: '珐',
        source: '琺',
        glyph_index: 5996,
    },
    MapEntry {
        target: '豉',
        source: '囮',
        glyph_index: 4788,
    },
    MapEntry {
        target: '鄞',
        source: '囓',
        glyph_index: 4786,
    },
    MapEntry {
        target: '溱',
        source: '囑',
        glyph_index: 4785,
    },
    MapEntry {
        target: '诜',
        source: '囎',
        glyph_index: 4784,
    },
    MapEntry {
        target: '悫',
        source: '愨',
        glyph_index: 5203,
    },
    MapEntry {
        target: '芩',
        source: '囃',
        glyph_index: 4781,
    },
    MapEntry {
        target: '跽',
        source: '嚴',
        glyph_index: 4777,
    },
    MapEntry {
        target: '硎',
        source: '嚮',
        glyph_index: 4775,
    },
    MapEntry {
        target: '鹕',
        source: '嚥',
        glyph_index: 4774,
    },
    MapEntry {
        target: '肏',
        source: '嚔',
        glyph_index: 4772,
    },
    MapEntry {
        target: '蠓',
        source: '嚠',
        glyph_index: 4771,
    },
    MapEntry {
        target: '麀',
        source: '嚊',
        glyph_index: 4770,
    },
    MapEntry {
        target: '缡',
        source: '營',
        glyph_index: 4758,
    },
    MapEntry {
        target: '剀',
        source: '剴',
        glyph_index: 4594,
    },
    MapEntry {
        target: '揎',
        source: '噐',
        glyph_index: 4757,
    },
    MapEntry {
        target: '瘐',
        source: '嗹',
        glyph_index: 4755,
    },
    MapEntry {
        target: '呙',
        source: '咼',
        glyph_index: 4704,
    },
    MapEntry {
        target: '绲',
        source: '喨',
        glyph_index: 4741,
    },
    MapEntry {
        target: '酴',
        source: '單',
        glyph_index: 4736,
    },
    MapEntry {
        target: '屦',
        source: '啝',
        glyph_index: 4726,
    },
    MapEntry {
        target: '芈',
        source: '唸',
        glyph_index: 4724,
    },
    MapEntry {
        target: '蚴',
        source: '啗',
        glyph_index: 4723,
    },
    MapEntry {
        target: '埽',
        source: '啅',
        glyph_index: 4721,
    },
    MapEntry {
        target: '诖',
        source: '啌',
        glyph_index: 4718,
    },
    MapEntry {
        target: '跶',
        source: '啣',
        glyph_index: 4717,
    },
    MapEntry {
        target: '浥',
        source: '啀',
        glyph_index: 4716,
    },
    MapEntry {
        target: '菔',
        source: '唹',
        glyph_index: 4715,
    },
    MapEntry {
        target: '痖',
        source: '哘',
        glyph_index: 4705,
    },
    MapEntry {
        target: '鲣',
        source: '鰹',
        glyph_index: 1770,
    },
    MapEntry {
        target: '瘗',
        source: '咾',
        glyph_index: 4703,
    },
    MapEntry {
        target: '蛳',
        source: '咥',
        glyph_index: 4695,
    },
    MapEntry {
        target: '镝',
        source: '鏑',
        glyph_index: 3396,
    },
    MapEntry {
        target: '埸',
        source: '咢',
        glyph_index: 4693,
    },
    MapEntry {
        target: '雩',
        source: '呰',
        glyph_index: 4684,
    },
    MapEntry {
        target: '虿',
        source: '呟',
        glyph_index: 4681,
    },
    MapEntry {
        target: '笱',
        source: '吽',
        glyph_index: 4668,
    },
    MapEntry {
        target: '鹓',
        source: '叺',
        glyph_index: 4666,
    },
    MapEntry {
        target: '煅',
        source: '雙',
        glyph_index: 4659,
    },
    MapEntry {
        target: '郾',
        source: '厰',
        glyph_index: 4655,
    },
    MapEntry {
        target: '刭',
        source: '剄',
        glyph_index: 4588,
    },
    MapEntry {
        target: '泚',
        source: '卻',
        glyph_index: 4647,
    },
    MapEntry {
        target: '籼',
        source: '夘',
        glyph_index: 4646,
    },
    MapEntry {
        target: '挢',
        source: '卩',
        glyph_index: 4644,
    },
    MapEntry {
        target: '觌',
        source: '覿',
        glyph_index: 6977,
    },
    MapEntry {
        target: '砷',
        source: '凖',
        glyph_index: 4642,
    },
    MapEntry {
        target: '肽',
        source: '卍',
        glyph_index: 4641,
    },
    MapEntry {
        target: '尕',
        source: '丗',
        glyph_index: 4639,
    },
    MapEntry {
        target: '镍',
        source: '卆',
        glyph_index: 4637,
    },
    MapEntry {
        target: '臁',
        source: '區',
        glyph_index: 4636,
    },
    MapEntry {
        target: '绁',
        source: '紲',
        glyph_index: 6399,
    },
    MapEntry {
        target: '糁',
        source: '匸',
        glyph_index: 4635,
    },
    MapEntry {
        target: '郫',
        source: '匳',
        glyph_index: 4634,
    },
    MapEntry {
        target: '湜',
        source: '勹',
        glyph_index: 4622,
    },
    MapEntry {
        target: '鹩',
        source: '鷯',
        glyph_index: 7737,
    },
    MapEntry {
        target: '蚨',
        source: '勸',
        glyph_index: 4621,
    },
    MapEntry {
        target: '芃',
        source: '勵',
        glyph_index: 4620,
    },
    MapEntry {
        target: '缵',
        source: '纉',
        glyph_index: 6474,
    },
    MapEntry {
        target: '铱',
        source: '勳',
        glyph_index: 4619,
    },
    MapEntry {
        target: '鲞',
        source: '勠',
        glyph_index: 4618,
    },
    MapEntry {
        target: '窆',
        source: '勦',
        glyph_index: 4616,
    },
    MapEntry {
        target: '敉',
        source: '勣',
        glyph_index: 4615,
    },
    MapEntry {
        target: '菏',
        source: '勞',
        glyph_index: 4614,
    },
    MapEntry {
        target: '萁',
        source: '勗',
        glyph_index: 4613,
    },
    MapEntry {
        target: '湣',
        source: '勍',
        glyph_index: 4612,
    },
    MapEntry {
        target: '筇',
        source: '劵',
        glyph_index: 4610,
    },
    MapEntry {
        target: '瞀',
        source: '劼',
        glyph_index: 4609,
    },
    MapEntry {
        target: '笫',
        source: '辧',
        glyph_index: 4606,
    },
    MapEntry {
        target: '黧',
        source: '劑',
        glyph_index: 4604,
    },
    MapEntry {
        target: '腚',
        source: '剱',
        glyph_index: 4602,
    },
    MapEntry {
        target: '勐',
        source: '劒',
        glyph_index: 4601,
    },
    MapEntry {
        target: '觱',
        source: '劔',
        glyph_index: 4600,
    },
    MapEntry {
        target: '皤',
        source: '劍',
        glyph_index: 4599,
    },
    MapEntry {
        target: '瘼',
        source: '剳',
        glyph_index: 4596,
    },
    MapEntry {
        target: '笕',
        source: '筧',
        glyph_index: 6307,
    },
    MapEntry {
        target: '嵊',
        source: '剋',
        glyph_index: 4589,
    },
    MapEntry {
        target: '姮',
        source: '剏',
        glyph_index: 4587,
    },
    MapEntry {
        target: '痤',
        source: '刧',
        glyph_index: 4582,
    },
    MapEntry {
        target: '匄',
        source: '刔',
        glyph_index: 4580,
    },
    MapEntry {
        target: '闶',
        source: '刋',
        glyph_index: 4579,
    },
    MapEntry {
        target: '鲆',
        source: '鮃',
        glyph_index: 7638,
    },
    MapEntry {
        target: '阚',
        source: '刄',
        glyph_index: 4578,
    },
    MapEntry {
        target: '蒉',
        source: '凾',
        glyph_index: 4577,
    },
    MapEntry {
        target: '吣',
        source: '凵',
        glyph_index: 4576,
    },
    MapEntry {
        target: '珲',
        source: '琿',
        glyph_index: 5998,
    },
    MapEntry {
        target: '蟪',
        source: '凩',
        glyph_index: 4573,
    },
    MapEntry {
        target: '窸',
        source: '處',
        glyph_index: 4572,
    },
    MapEntry {
        target: '邰',
        source: '凅',
        glyph_index: 4568,
    },
    MapEntry {
        target: '蠛',
        source: '冱',
        glyph_index: 4563,
    },
    MapEntry {
        target: '痧',
        source: '冫',
        glyph_index: 4561,
    },
    MapEntry {
        target: '锛',
        source: '冩',
        glyph_index: 4559,
    },
    MapEntry {
        target: '埤',
        source: '冦',
        glyph_index: 4557,
    },
    MapEntry {
        target: '殂',
        source: '冖',
        glyph_index: 4555,
    },
    MapEntry {
        target: '趵',
        source: '冓',
        glyph_index: 4553,
    },
    MapEntry {
        target: '鄄',
        source: '冑',
        glyph_index: 4552,
    },
    MapEntry {
        target: '熳',
        source: '冏',
        glyph_index: 4551,
    },
    MapEntry {
        target: '癔',
        source: '囘',
        glyph_index: 4548,
    },
    MapEntry {
        target: '锸',
        source: '冂',
        glyph_index: 4547,
    },
    MapEntry {
        target: '珙',
        source: '兩',
        glyph_index: 4543,
    },
    MapEntry {
        target: '箓',
        source: '竸',
        glyph_index: 4542,
    },
    MapEntry {
        target: '蘩',
        source: '兒',
        glyph_index: 4538,
    },
    MapEntry {
        target: '鏊',
        source: '儚',
        glyph_index: 4530,
    },
    MapEntry {
        target: '鲔',
        source: '鮪',
        glyph_index: 4025,
    },
    MapEntry {
        target: '恧',
        source: '儖',
        glyph_index: 4527,
    },
    MapEntry {
        target: '簏',
        source: '儁',
        glyph_index: 4525,
    },
    MapEntry {
        target: '枞',
        source: '樅',
        glyph_index: 5611,
    },
    MapEntry {
        target: '咮',
        source: '儉',
        glyph_index: 4524,
    },
    MapEntry {
        target: '哏',
        source: '價',
        glyph_index: 4522,
    },
    MapEntry {
        target: '仳',
        source: '僣',
        glyph_index: 4520,
    },
    MapEntry {
        target: '岵',
        source: '僞',
        glyph_index: 4517,
    },
    MapEntry {
        target: '炝',
        source: '傳',
        glyph_index: 4514,
    },
    MapEntry {
        target: '呔',
        source: '僊',
        glyph_index: 4513,
    },
    MapEntry {
        target: '裣',
        source: '傚',
        glyph_index: 4508,
    },
    MapEntry {
        target: '颡',
        source: '偖',
        glyph_index: 4504,
    },
    MapEntry {
        target: '昉',
        source: '偐',
        glyph_index: 4501,
    },
    MapEntry {
        target: '苁',
        source: '會',
        glyph_index: 4499,
    },
    MapEntry {
        target: '鼯',
        source: '伜',
        glyph_index: 4488,
    },
    MapEntry {
        target: '荜',
        source: '倅',
        glyph_index: 4487,
    },
    MapEntry {
        target: '翚',
        source: '俥',
        glyph_index: 4481,
    },
    MapEntry {
        target: '鹁',
        source: '俤',
        glyph_index: 4480,
    },
    MapEntry {
        target: '奡',
        source: '俛',
        glyph_index: 4476,
    },
    MapEntry {
        target: '獯',
        source: '俔',
        glyph_index: 4472,
    },
    MapEntry {
        target: '黟',
        source: '儘',
        glyph_index: 4471,
    },
    MapEntry {
        target: '礌',
        source: '來',
        glyph_index: 4469,
    },
    MapEntry {
        target: '栘',
        source: '侘',
        glyph_index: 4463,
    },
    MapEntry {
        target: '摅',
        source: '仭',
        glyph_index: 4450,
    },
    MapEntry {
        target: '黡',
        source: '黶',
        glyph_index: 7772,
    },
    MapEntry {
        target: '忮',
        source: '仂',
        glyph_index: 4447,
    },
    MapEntry {
        target: '刖',
        source: '亰',
        glyph_index: 4440,
    },
    MapEntry {
        target: '侉',
        source: '亠',
        glyph_index: 4438,
    },
    MapEntry {
        target: '稞',
        source: '亞',
        glyph_index: 4436,
    },
    MapEntry {
        target: '锼',
        source: '弍',
        glyph_index: 4434,
    },
    MapEntry {
        target: '瘛',
        source: '亊',
        glyph_index: 4432,
    },
    MapEntry {
        target: '汊',
        source: '亅',
        glyph_index: 4430,
    },
    MapEntry {
        target: '嗉',
        source: '亂',
        glyph_index: 4429,
    },
    MapEntry {
        target: '醅',
        source: '乂',
        glyph_index: 4426,
    },
    MapEntry {
        target: '踣',
        source: '丿',
        glyph_index: 4425,
    },
    MapEntry {
        target: '庥',
        source: '丼',
        glyph_index: 4424,
    },
    MapEntry {
        target: '缫',
        source: '丶',
        glyph_index: 4423,
    },
    MapEntry {
        target: '钲',
        source: '鉦',
        glyph_index: 2789,
    },
    MapEntry {
        target: '愔',
        source: '丱',
        glyph_index: 4422,
    },
    MapEntry {
        target: '菹',
        source: '弌',
        glyph_index: 4418,
    },
    MapEntry {
        target: '廑',
        source: '椀',
        glyph_index: 4371,
    },
    MapEntry {
        target: '撙',
        source: '亙',
        glyph_index: 4365,
    },
    MapEntry {
        target: '舣',
        source: '艤',
        glyph_index: 6642,
    },
    MapEntry {
        target: '椐',
        source: '錬',
        glyph_index: 4326,
    },
    MapEntry {
        target: '哕',
        source: '憐',
        glyph_index: 4318,
    },
    MapEntry {
        target: '闫',
        source: '暦',
        glyph_index: 4310,
    },
    MapEntry {
        target: '锶',
        source: '隷',
        glyph_index: 4305,
    },
    MapEntry {
        target: '濞',
        source: '涙',
        glyph_index: 4291,
    },
    MapEntry {
        target: '龆',
        source: '齠',
        glyph_index: 7790,
    },
    MapEntry {
        target: '缳',
        source: '瑠',
        glyph_index: 4289,
    },
    MapEntry {
        target: '谂',
        source: '糧',
        glyph_index: 4269,
    },
    MapEntry {
        target: '氚',
        source: '稜',
        glyph_index: 4268,
    },
    MapEntry {
        target: '塍',
        source: '瞭',
        glyph_index: 4267,
    },
    MapEntry {
        target: '箨',
        source: '涼',
        glyph_index: 4264,
    },
    MapEntry {
        target: '倮',
        source: '葎',
        glyph_index: 4239,
    },
    MapEntry {
        target: '阌',
        source: '裡',
        glyph_index: 4232,
    },
    MapEntry {
        target: '镪',
        source: '裏',
        glyph_index: 4231,
    },
    MapEntry {
        target: '厣',
        source: '慾',
        glyph_index: 4196,
    },
    MapEntry {
        target: '芑',
        source: '葉',
        glyph_index: 4188,
    },
    MapEntry {
        target: '吡',
        source: '遊',
        glyph_index: 4158,
    },
    MapEntry {
        target: '缧',
        source: '縲',
        glyph_index: 6455,
    },
    MapEntry {
        target: '帻',
        source: '猶',
        glyph_index: 4152,
    },
    MapEntry {
        target: '郗',
        source: '湧',
        glyph_index: 4150,
    },
    MapEntry {
        target: '钋',
        source: '癒',
        glyph_index: 4135,
    },
    MapEntry {
        target: '柽',
        source: '鑓',
        glyph_index: 4131,
    },
    MapEntry {
        target: '岀',
        source: '匁',
        glyph_index: 4113,
    },
    MapEntry {
        target: '脞',
        source: '籾',
        glyph_index: 4107,
    },
    MapEntry {
        target: '氦',
        source: '戻',
        glyph_index: 4106,
    },
    MapEntry {
        target: '糍',
        source: '杢',
        glyph_index: 4102,
    },
    MapEntry {
        target: '棨',
        source: '黙',
        glyph_index: 4100,
    },
    MapEntry {
        target: '鄣',
        source: '網',
        glyph_index: 4095,
    },
    MapEntry {
        target: '逭',
        source: '麺',
        glyph_index: 4086,
    },
    MapEntry {
        target: '忭',
        source: '無',
        glyph_index: 4062,
    },
    MapEntry {
        target: '钍',
        source: '夢',
        glyph_index: 4061,
    },
    MapEntry {
        target: '眙',
        source: '粍',
        glyph_index: 4057,
    },
    MapEntry {
        target: '羑',
        source: '脈',
        glyph_index: 4055,
    },
    MapEntry {
        target: '慆',
        source: '麿',
        glyph_index: 4038,
    },
    MapEntry {
        target: '觫',
        source: '侭',
        glyph_index: 4036,
    },
    MapEntry {
        target: '邲',
        source: '桝',
        glyph_index: 4028,
    },
    MapEntry {
        target: '眚',
        source: '柾',
        glyph_index: 4026,
    },
    MapEntry {
        target: '硖',
        source: '槙',
        glyph_index: 4021,
    },
    MapEntry {
        target: '搛',
        source: '釦',
        glyph_index: 4000,
    },
    MapEntry {
        target: '獬',
        source: '僕',
        glyph_index: 3992,
    },
    MapEntry {
        target: '岢',
        source: '鉾',
        glyph_index: 3987,
    },
    MapEntry {
        target: '徬',
        source: '豊',
        glyph_index: 3960,
    },
    MapEntry {
        target: '轺',
        source: '砲',
        glyph_index: 3951,
    },
    MapEntry {
        target: '蒗',
        source: '峯',
        glyph_index: 3940,
    },
    MapEntry {
        target: '绱',
        source: '穂',
        glyph_index: 3923,
    },
    MapEntry {
        target: '诹',
        source: '諏',
        glyph_index: 2878,
    },
    MapEntry {
        target: '攮',
        source: '鋪',
        glyph_index: 3916,
    },
    MapEntry {
        target: '稊',
        source: '辺',
        glyph_index: 3906,
    },
    MapEntry {
        target: '湔',
        source: '箆',
        glyph_index: 3900,
    },
    MapEntry {
        target: '蜾',
        source: '塀',
        glyph_index: 3882,
    },
    MapEntry {
        target: '煳',
        source: '併',
        glyph_index: 3880,
    },
    MapEntry {
        target: '釄',
        source: '雰',
        glyph_index: 3876,
    },
    MapEntry {
        target: '钴',
        source: '鈷',
        glyph_index: 2219,
    },
    MapEntry {
        target: '饩',
        source: '仏',
        glyph_index: 3862,
    },
    MapEntry {
        target: '莸',
        source: '蕕',
        glyph_index: 6771,
    },
    MapEntry {
        target: '鳜',
        source: '払',
        glyph_index: 3860,
    },
    MapEntry {
        target: '憯',
        source: '淵',
        glyph_index: 3858,
    },
    MapEntry {
        target: '虼',
        source: '複',
        glyph_index: 3856,
    },
    MapEntry {
        target: '璠',
        source: '鋲',
        glyph_index: 3797,
    },
    MapEntry {
        target: '祎',
        source: '氷',
        glyph_index: 3784,
    },
    MapEntry {
        target: '髁',
        source: '俵',
        glyph_index: 3781,
    },
    MapEntry {
        target: '桕',
        source: '柊',
        glyph_index: 3761,
    },
    MapEntry {
        target: '憷',
        source: '毘',
        glyph_index: 3756,
    },
    MapEntry {
        target: '砀',
        source: '樋',
        glyph_index: 3750,
    },
    MapEntry {
        target: '贶',
        source: '釆',
        glyph_index: 3713,
    },
    MapEntry {
        target: '鲱',
        source: '鯡',
        glyph_index: 7657,
    },
    MapEntry {
        target: '僦',
        source: '範',
        glyph_index: 3712,
    },
    MapEntry {
        target: '鲥',
        source: '汎',
        glyph_index: 3703,
    },
    MapEntry {
        target: '魈',
        source: '氾',
        glyph_index: 3702,
    },
    MapEntry {
        target: '觔',
        source: '塙',
        glyph_index: 3690,
    },
    MapEntry {
        target: '荽',
        source: '噺',
        glyph_index: 3689,
    },
    MapEntry {
        target: '熘',
        source: '抜',
        glyph_index: 3685,
    },
    MapEntry {
        target: '崾',
        source: '醗',
        glyph_index: 3681,
    },
    MapEntry {
        target: '桬',
        source: '発',
        glyph_index: 3680,
    },
    MapEntry {
        target: '搧',
        source: '畠',
        glyph_index: 3676,
    },
    MapEntry {
        target: '遹',
        source: '畑',
        glyph_index: 3675,
    },
    MapEntry {
        target: '孓',
        source: '筈',
        glyph_index: 3671,
    },
    MapEntry {
        target: '昺',
        source: '硲',
        glyph_index: 3668,
    },
    MapEntry {
        target: '砻',
        source: '萩',
        glyph_index: 3646,
    },
    MapEntry {
        target: '囟',
        source: '楳',
        glyph_index: 3635,
    },
    MapEntry {
        target: '笪',
        source: '盃',
        glyph_index: 3625,
    },
    MapEntry {
        target: '垡',
        source: '拝',
        glyph_index: 3621,
    },
    MapEntry {
        target: '佺',
        source: '覇',
        glyph_index: 3609,
    },
    MapEntry {
        target: '酤',
        source: '覗',
        glyph_index: 3604,
    },
    MapEntry {
        target: '晬',
        source: '廼',
        glyph_index: 3593,
    },
    MapEntry {
        target: '偾',
        source: '禰',
        glyph_index: 3580,
    },
    MapEntry {
        target: '蹁',
        source: '韮',
        glyph_index: 3574,
    },
    MapEntry {
        target: '骎',
        source: '駸',
        glyph_index: 7571,
    },
    MapEntry {
        target: '柒',
        source: '匂',
        glyph_index: 3564,
    },
    MapEntry {
        target: '叻',
        source: '弐',
        glyph_index: 3562,
    },
    MapEntry {
        target: '贳',
        source: '貰',
        glyph_index: 4108,
    },
    MapEntry {
        target: '艹',
        source: '畷',
        glyph_index: 3554,
    },
    MapEntry {
        target: '糇',
        source: '楢',
        glyph_index: 3551,
    },
    MapEntry {
        target: '枵',
        source: '薙',
        glyph_index: 3546,
    },
    MapEntry {
        target: '腭',
        source: '齶',
        glyph_index: 7798,
    },
    MapEntry {
        target: '锜',
        source: '瀞',
        glyph_index: 3529,
    },
    MapEntry {
        target: '忉',
        source: '椴',
        glyph_index: 3523,
    },
    MapEntry {
        target: '舡',
        source: '栃',
        glyph_index: 3519,
    },
    MapEntry {
        target: '胗',
        source: '峠',
        glyph_index: 3506,
    },
    MapEntry {
        target: '泐',
        source: '働',
        glyph_index: 3492,
    },
    MapEntry {
        target: '浼',
        source: '燈',
        glyph_index: 3468,
    },
    MapEntry {
        target: '艏',
        source: '梼',
        glyph_index: 3461,
    },
    MapEntry {
        target: '斁',
        source: '嶋',
        glyph_index: 3455,
    },
    MapEntry {
        target: '跗',
        source: '兎',
        glyph_index: 3421,
    },
    MapEntry {
        target: '劂',
        source: '澱',
        glyph_index: 3418,
    },
    MapEntry {
        target: '涑',
        source: '紬',
        glyph_index: 3350,
    },
    MapEntry {
        target: '鹨',
        source: '嬬',
        glyph_index: 3349,
    },
    MapEntry {
        target: '跩',
        source: '辻',
        glyph_index: 3341,
    },
    MapEntry {
        target: '闿',
        source: '槻',
        glyph_index: 3337,
    },
    MapEntry {
        target: '蒌',
        source: '栂',
        glyph_index: 3335,
    },
    MapEntry {
        target: '蹀',
        source: '塚',
        glyph_index: 3334,
    },
    MapEntry {
        target: '趱',
        source: '鎚',
        glyph_index: 3331,
    },
    MapEntry {
        target: '绠',
        source: '捗',
        glyph_index: 3318,
    },
    MapEntry {
        target: '崃',
        source: '勅',
        glyph_index: 3317,
    },
    MapEntry {
        target: '柙',
        source: '銚',
        glyph_index: 3313,
    },
    MapEntry {
        target: '瘳',
        source: '聴',
        glyph_index: 3305,
    },
    MapEntry {
        target: '歊',
        source: '徴',
        glyph_index: 3296,
    },
    MapEntry {
        target: '嘒',
        source: '彫',
        glyph_index: 3295,
    },
    MapEntry {
        target: '沬',
        source: '弔',
        glyph_index: 3293,
    },
    MapEntry {
        target: '箬',
        source: '瀦',
        glyph_index: 3280,
    },
    MapEntry {
        target: '酯',
        source: '酎',
        glyph_index: 3276,
    },
    MapEntry {
        target: '馓',
        source: '註',
        glyph_index: 3275,
    },
    MapEntry {
        target: '茭',
        source: '築',
        glyph_index: 3254,
    },
    MapEntry {
        target: '獍',
        source: '恥',
        glyph_index: 3244,
    },
    MapEntry {
        target: '锑',
        source: '歎',
        glyph_index: 3218,
    },
    MapEntry {
        target: '甙',
        source: '蛸',
        glyph_index: 3194,
    },
    MapEntry {
        target: '皲',
        source: '皸',
        glyph_index: 6125,
    },
    MapEntry {
        target: '煸',
        source: '凧',
        glyph_index: 3193,
    },
    MapEntry {
        target: '芪',
        source: '託',
        glyph_index: 3188,
    },
    MapEntry {
        target: '鹖',
        source: '択',
        glyph_index: 3183,
    },
    MapEntry {
        target: '萏',
        source: '瀧',
        glyph_index: 3178,
    },
    MapEntry {
        target: '苷',
        source: '辿',
        glyph_index: 3204,
    },
    MapEntry {
        target: '鲷',
        source: '鯛',
        glyph_index: 3169,
    },
    MapEntry {
        target: '慉',
        source: '騨',
        glyph_index: 3146,
    },
    MapEntry {
        target: '嫪',
        source: '楕',
        glyph_index: 3143,
    },
    MapEntry {
        target: '嗌',
        source: '詑',
        glyph_index: 3135,
    },
    MapEntry {
        target: '毐',
        source: '揃',
        glyph_index: 3124,
    },
    MapEntry {
        target: '蚧',
        source: '蔵',
        glyph_index: 3103,
    },
    MapEntry {
        target: '朐',
        source: '鎗',
        glyph_index: 3096,
    },
    MapEntry {
        target: '湓',
        source: '荘',
        glyph_index: 3088,
    },
    MapEntry {
        target: '陉',
        source: '窓',
        glyph_index: 3082,
    },
    MapEntry {
        target: '蛏',
        source: '蟶',
        glyph_index: 6887,
    },
    MapEntry {
        target: '谫',
        source: '捜',
        glyph_index: 3067,
    },
    MapEntry {
        target: '畈',
        source: '惣',
        glyph_index: 3065,
    },
    MapEntry {
        target: '锳',
        source: '遡',
        glyph_index: 3051,
    },
    MapEntry {
        target: '埏',
        source: '疎',
        glyph_index: 3041,
    },
    MapEntry {
        target: '旰',
        source: '曽',
        glyph_index: 3037,
    },
    MapEntry {
        target: '芨',
        source: '岨',
        glyph_index: 3034,
    },
    MapEntry {
        target: '瞢',
        source: '糎',
        glyph_index: 3031,
    },
    MapEntry {
        target: '芄',
        source: '薦',
        glyph_index: 3013,
    },
    MapEntry {
        target: '鹇',
        source: '栴',
        glyph_index: 2996,
    },
    MapEntry {
        target: '仡',
        source: '蹟',
        glyph_index: 2969,
    },
    MapEntry {
        target: '挼',
        source: '跡',
        glyph_index: 2968,
    },
    MapEntry {
        target: '埘',
        source: '塒',
        glyph_index: 4838,
    },
    MapEntry {
        target: '倊',
        source: '隻',
        glyph_index: 2954,
    },
    MapEntry {
        target: '窨',
        source: '製',
        glyph_index: 2942,
    },
    MapEntry {
        target: '湑',
        source: '棲',
        glyph_index: 2932,
    },
    MapEntry {
        target: '醛',
        source: '摺',
        glyph_index: 2915,
    },
    MapEntry {
        target: '沭',
        source: '椙',
        glyph_index: 2909,
    },
    MapEntry {
        target: '芏',
        source: '髄',
        glyph_index: 2900,
    },
    MapEntry {
        target: '囷',
        source: '酔',
        glyph_index: 2895,
    },
    MapEntry {
        target: '硷',
        source: '粋',
        glyph_index: 2891,
    },
    MapEntry {
        target: '哚',
        source: '醸',
        glyph_index: 2815,
    },
    MapEntry {
        target: '汔',
        source: '譲',
        glyph_index: 2814,
    },
    MapEntry {
        target: '铂',
        source: '穣',
        glyph_index: 2812,
    },
    MapEntry {
        target: '猃',
        source: '壌',
        glyph_index: 2802,
    },
    MapEntry {
        target: '嗞',
        source: '剰',
        glyph_index: 2799,
    },
    MapEntry {
        target: '窭',
        source: '窶',
        glyph_index: 6267,
    },
    MapEntry {
        target: '涘',
        source: '乗',
        glyph_index: 2797,
    },
    MapEntry {
        target: '嵝',
        source: '鍾',
        glyph_index: 2790,
    },
    MapEntry {
        target: '伃',
        source: '衝',
        glyph_index: 2780,
    },
    MapEntry {
        target: '缬',
        source: '纈',
        glyph_index: 6473,
    },
    MapEntry {
        target: '莜',
        source: '昇',
        glyph_index: 2751,
    },
    MapEntry {
        target: '珦',
        source: '廠',
        glyph_index: 2744,
    },
    MapEntry {
        target: '豨',
        source: '藷',
        glyph_index: 2714,
    },
    MapEntry {
        target: '绨',
        source: '準',
        glyph_index: 2695,
    },
    MapEntry {
        target: '锬',
        source: '楯',
        glyph_index: 2692,
    },
    MapEntry {
        target: '鞮',
        source: '従',
        glyph_index: 2661,
    },
    MapEntry {
        target: '趑',
        source: '醜',
        glyph_index: 2656,
    },
    MapEntry {
        target: '跬',
        source: '週',
        glyph_index: 2652,
    },
    MapEntry {
        target: '劓',
        source: '讐',
        glyph_index: 2649,
    },
    MapEntry {
        target: '螨',
        source: '蒐',
        glyph_index: 2646,
    },
    MapEntry {
        target: '皦',
        source: '繍',
        glyph_index: 2642,
    },
    MapEntry {
        target: '氵',
        source: '収',
        glyph_index: 2630,
    },
    MapEntry {
        target: '锗',
        source: '呪',
        glyph_index: 2623,
    },
    MapEntry {
        target: '嗐',
        source: '捨',
        glyph_index: 2583,
    },
    MapEntry {
        target: '疃',
        source: '舎',
        glyph_index: 2580,
    },
    MapEntry {
        target: '肷',
        source: '偲',
        glyph_index: 2574,
    },
    MapEntry {
        target: '镠',
        source: '鏐',
        glyph_index: 7356,
    },
    MapEntry {
        target: '硐',
        source: '篠',
        glyph_index: 2573,
    },
    MapEntry {
        target: '焓',
        source: '蔀',
        glyph_index: 2572,
    },
    MapEntry {
        target: '俜',
        source: '雫',
        glyph_index: 2559,
    },
    MapEntry {
        target: '钛',
        source: '宍',
        glyph_index: 2558,
    },
    MapEntry {
        target: '玠',
        source: '鴫',
        glyph_index: 2555,
    },
    MapEntry {
        target: '倓',
        source: '児',
        glyph_index: 2532,
    },
    MapEntry {
        target: '鄘',
        source: '諮',
        glyph_index: 2523,
    },
    MapEntry {
        target: '驵',
        source: '誌',
        glyph_index: 2522,
    },
    MapEntry {
        target: '苋',
        source: '糸',
        glyph_index: 2512,
    },
    MapEntry {
        target: '狶',
        source: '屍',
        glyph_index: 2494,
    },
    MapEntry {
        target: '芤',
        source: '讃',
        glyph_index: 2473,
    },
    MapEntry {
        target: '傺',
        source: '錆',
        glyph_index: 2455,
    },
    MapEntry {
        target: '骱',
        source: '鯖',
        glyph_index: 2453,
    },
    MapEntry {
        target: '薅',
        source: '皐',
        glyph_index: 2452,
    },
    MapEntry {
        target: '钭',
        source: '冊',
        glyph_index: 2442,
    },
    MapEntry {
        target: '粝',
        source: '糲',
        glyph_index: 6387,
    },
    MapEntry {
        target: '鞒',
        source: '笹',
        glyph_index: 2440,
    },
    MapEntry {
        target: '尥',
        source: '碕',
        glyph_index: 2425,
    },
    MapEntry {
        target: '樘',
        source: '埼',
        glyph_index: 2424,
    },
    MapEntry {
        target: '筲',
        source: '榊',
        glyph_index: 2420,
    },
    MapEntry {
        target: '钕',
        source: '堺',
        glyph_index: 2419,
    },
    MapEntry {
        target: '沩',
        source: '冴',
        glyph_index: 2416,
    },
    MapEntry {
        target: '榇',
        source: '砕',
        glyph_index: 2402,
    },
    MapEntry {
        target: '猰',
        source: '瑳',
        glyph_index: 2377,
    },
    MapEntry {
        target: '裥',
        source: '込',
        glyph_index: 2349,
    },
    MapEntry {
        target: '砟',
        source: '狛',
        glyph_index: 2348,
    },
    MapEntry {
        target: '骟',
        source: '穀',
        glyph_index: 2337,
    },
    MapEntry {
        target: '醁',
        source: '麹',
        glyph_index: 2332,
    },
    MapEntry {
        target: '崦',
        source: '閤',
        glyph_index: 2317,
    },
    MapEntry {
        target: '嫒',
        source: '砿',
        glyph_index: 2315,
    },
    MapEntry {
        target: '痀',
        source: '紘',
        glyph_index: 2296,
    },
    MapEntry {
        target: '螬',
        source: '広',
        glyph_index: 2268,
    },
    MapEntry {
        target: '绐',
        source: '紿',
        glyph_index: 6400,
    },
    MapEntry {
        target: '甏',
        source: '効',
        glyph_index: 2251,
    },
    MapEntry {
        target: '浠',
        source: '倖',
        glyph_index: 2247,
    },
    MapEntry {
        target: '蔾',
        source: '檎',
        glyph_index: 2234,
    },
    MapEntry {
        target: '蛘',
        source: '後',
        glyph_index: 2230,
    },
    MapEntry {
        target: '鲟',
        source: '個',
        glyph_index: 2197,
    },
    MapEntry {
        target: '缂',
        source: '絃',
        glyph_index: 2191,
    },
    MapEntry {
        target: '楦',
        source: '捲',
        glyph_index: 2161,
    },
    MapEntry {
        target: '揸',
        source: '圏',
        glyph_index: 2154,
    },
    MapEntry {
        target: '悰',
        source: '決',
        glyph_index: 2139,
    },
    MapEntry {
        target: '镆',
        source: '傑',
        glyph_index: 2137,
    },
    MapEntry {
        target: '旐',
        source: '罫',
        glyph_index: 2118,
    },
    MapEntry {
        target: '珧',
        source: '繋',
        glyph_index: 2117,
    },
    MapEntry {
        target: '鳣',
        source: '渓',
        glyph_index: 2111,
    },
    MapEntry {
        target: '啴',
        source: '恵',
        glyph_index: 2102,
    },
    MapEntry {
        target: '肫',
        source: '係',
        glyph_index: 2091,
    },
    MapEntry {
        target: '玕',
        source: '繰',
        glyph_index: 2078,
    },
    MapEntry {
        target: '猹',
        source: '粂',
        glyph_index: 2076,
    },
    MapEntry {
        target: '菑',
        source: '駈',
        glyph_index: 2052,
    },
    MapEntry {
        target: '摽',
        source: '粁',
        glyph_index: 2019,
    },
    MapEntry {
        target: '坫',
        source: '脅',
        glyph_index: 2001,
    },
    MapEntry {
        target: '糨',
        source: '況',
        glyph_index: 1996,
    },
    MapEntry {
        target: '絜',
        source: '兇',
        glyph_index: 1977,
    },
    MapEntry {
        target: '钒',
        source: '禦',
        glyph_index: 1969,
    },
    MapEntry {
        target: '疖',
        source: '拠',
        glyph_index: 1961,
    },
    MapEntry {
        target: '瀼',
        source: '喫',
        glyph_index: 1921,
    },
    MapEntry {
        target: '檩',
        source: '気',
        glyph_index: 1883,
    },
    MapEntry {
        target: '锔',
        source: '機',
        glyph_index: 1880,
    },
    MapEntry {
        target: '菪',
        source: '棄',
        glyph_index: 1879,
    },
    MapEntry {
        target: '蛲',
        source: '蟯',
        glyph_index: 6881,
    },
    MapEntry {
        target: '肟',
        source: '幾',
        glyph_index: 1871,
    },
    MapEntry {
        target: '芎',
        source: '願',
        glyph_index: 1859,
    },
    MapEntry {
        target: '嗍',
        source: '翫',
        glyph_index: 1854,
    },
    MapEntry {
        target: '蚵',
        source: '巌',
        glyph_index: 1849,
    },
    MapEntry {
        target: '瘆',
        source: '舘',
        glyph_index: 1845,
    },
    MapEntry {
        target: '猞',
        source: '陥',
        glyph_index: 1842,
    },
    MapEntry {
        target: '栌',
        source: '櫨',
        glyph_index: 3672,
    },
    MapEntry {
        target: '腩',
        source: '潅',
        glyph_index: 1820,
    },
    MapEntry {
        target: '苤',
        source: '幹',
        glyph_index: 1805,
    },
    MapEntry {
        target: '晡',
        source: '姦',
        glyph_index: 1800,
    },
    MapEntry {
        target: '锓',
        source: '巻',
        glyph_index: 1797,
    },
    MapEntry {
        target: '骍',
        source: '乾',
        glyph_index: 1790,
    },
    MapEntry {
        target: '钚',
        source: '苅',
        glyph_index: 1788,
    },
    MapEntry {
        target: '钨',
        source: '栢',
        glyph_index: 1783,
    },
    MapEntry {
        target: '孢',
        source: '蚫',
        glyph_index: 6826,
    },
    MapEntry {
        target: '裢',
        source: '鞄',
        glyph_index: 1774,
    },
    MapEntry {
        target: '琎',
        source: '樺',
        glyph_index: 1773,
    },
    MapEntry {
        target: '轵',
        source: '椛',
        glyph_index: 1772,
    },
    MapEntry {
        target: '呯',
        source: '梶',
        glyph_index: 1756,
    },
    MapEntry {
        target: '栝',
        source: '橿',
        glyph_index: 1755,
    },
    MapEntry {
        target: '螓',
        source: '樫',
        glyph_index: 1754,
    },
    MapEntry {
        target: '鬒',
        source: '掛',
        glyph_index: 1752,
    },
    MapEntry {
        target: '裀',
        source: '穫',
        glyph_index: 1738,
    },
    MapEntry {
        target: '怍',
        source: '浬',
        glyph_index: 1720,
    },
    MapEntry {
        target: '鲅',
        source: '蓋',
        glyph_index: 1715,
    },
    MapEntry {
        target: '砹',
        source: '廻',
        glyph_index: 1683,
    },
    MapEntry {
        target: '虮',
        source: '壊',
        glyph_index: 1682,
    },
    MapEntry {
        target: '茑',
        source: '蔦',
        glyph_index: 3342,
    },
    MapEntry {
        target: '嘌',
        source: '菓',
        glyph_index: 1656,
    },
    MapEntry {
        target: '玦',
        source: '箇',
        glyph_index: 1650,
    },
    MapEntry {
        target: '艋',
        source: '価',
        glyph_index: 1630,
    },
    MapEntry {
        target: '骶',
        source: '仮',
        glyph_index: 1627,
    },
    MapEntry {
        target: '赟',
        source: '贇',
        glyph_index: 7103,
    },
    MapEntry {
        target: '馇',
        source: '沖',
        glyph_index: 1610,
    },
    MapEntry {
        target: '嘧',
        source: '於',
        glyph_index: 1590,
    },
    MapEntry {
        target: '矬',
        source: '薗',
        glyph_index: 1585,
    },
    MapEntry {
        target: '鲒',
        source: '煙',
        glyph_index: 1579,
    },
    MapEntry {
        target: '骙',
        source: '詠',
        glyph_index: 1554,
    },
    MapEntry {
        target: '畯',
        source: '榎',
        glyph_index: 1564,
    },
    MapEntry {
        target: '鲦',
        source: '頴',
        glyph_index: 1551,
    },
    MapEntry {
        target: '掹',
        source: '洩',
        glyph_index: 1547,
    },
    MapEntry {
        target: '棬',
        source: '叡',
        glyph_index: 1538,
    },
    MapEntry {
        target: '峣',
        source: '嶢',
        glyph_index: 5031,
    },
    MapEntry {
        target: '喤',
        source: '雲',
        glyph_index: 1535,
    },
    MapEntry {
        target: '猁',
        source: '噂',
        glyph_index: 1532,
    },
    MapEntry {
        target: '揠',
        source: '欝',
        glyph_index: 1524,
    },
    MapEntry {
        target: '胂',
        source: '韻',
        glyph_index: 1507,
    },
    MapEntry {
        target: '蠊',
        source: '鰯',
        glyph_index: 1492,
    },
    MapEntry {
        target: '鞲',
        source: '稲',
        glyph_index: 1489,
    },
    MapEntry {
        target: '镈',
        source: '壱',
        glyph_index: 1486,
    },
    MapEntry {
        target: '唪',
        source: '闇',
        glyph_index: 1448,
    },
    MapEntry {
        target: '砊',
        source: '扱',
        glyph_index: 1432,
    },
    MapEntry {
        target: '蚬',
        source: '蜆',
        glyph_index: 6835,
    },
    MapEntry {
        target: '崞',
        source: '鯵',
        glyph_index: 1428,
    },
    MapEntry {
        target: '垸',
        source: '穐',
        glyph_index: 1421,
    },
    MapEntry {
        target: '龀',
        source: '齔',
        glyph_index: 7787,
    },
    MapEntry {
        target: '胨',
        source: '姶',
        glyph_index: 1417,
    },
    MapEntry {
        target: '鲙',
        source: '鱠',
        glyph_index: 7683,
    },
    MapEntry {
        target: '·',
        source: '・',
        glyph_index: 5,
    },
    MapEntry {
        target: '—',
        source: '―',
        glyph_index: 28,
    },
];

type Handle = *mut c_void;
type Hdc = Handle;
type Hfont = Handle;
type Hbitmap = Handle;
type Hgdiobj = Handle;

const FR_PRIVATE: u32 = 0x10;
const DEFAULT_CHARSET: u32 = 1;
const OUT_DEFAULT_PRECIS: u32 = 0;
const CLIP_DEFAULT_PRECIS: u32 = 0;
const ANTIALIASED_QUALITY: u32 = 4;
const DEFAULT_PITCH: u32 = 0;
const GGI_MARK_NONEXISTING_GLYPHS: u32 = 1;
const GDI_ERROR: u32 = 0xFFFF_FFFF;
const DIB_RGB_COLORS: u32 = 0;
const BI_RGB: u32 = 0;
const TRANSPARENT: i32 = 1;
const TA_LEFT_TOP_NO_UPDATE_CP: u32 = 0;
const CP932_CODE_PAGE: u32 = 932;
const MB_ERR_INVALID_CHARS: u32 = 0x0000_0008;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct BitmapInfoHeader {
    biSize: u32,
    biWidth: i32,
    biHeight: i32,
    biPlanes: u16,
    biBitCount: u16,
    biCompression: u32,
    biSizeImage: u32,
    biXPelsPerMeter: i32,
    biYPelsPerMeter: i32,
    biClrUsed: u32,
    biClrImportant: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct BitmapInfo {
    bmiHeader: BitmapInfoHeader,
    bmiColors: [u32; 1],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct InkBounds {
    left: usize,
    top: usize,
    right: usize,
    bottom: usize,
}

#[link(name = "gdi32")]
extern "system" {
    fn AddFontResourceExW(name: *const u16, flags: u32, reserved: *mut c_void) -> i32;
    fn RemoveFontResourceExW(name: *const u16, flags: u32, reserved: *mut c_void) -> i32;
    fn CreateCompatibleDC(hdc: Hdc) -> Hdc;
    fn DeleteDC(hdc: Hdc) -> i32;
    fn CreateDIBSection(
        hdc: Hdc,
        bitmap_info: *const BitmapInfo,
        usage: u32,
        bits: *mut *mut c_void,
        section: Handle,
        offset: u32,
    ) -> Hbitmap;
    fn CreateFontW(
        height: i32,
        width: i32,
        escapement: i32,
        orientation: i32,
        weight: i32,
        italic: u32,
        underline: u32,
        strike_out: u32,
        char_set: u32,
        out_precision: u32,
        clip_precision: u32,
        quality: u32,
        pitch_and_family: u32,
        face: *const u16,
    ) -> Hfont;
    fn SelectObject(hdc: Hdc, object: Hgdiobj) -> Hgdiobj;
    fn DeleteObject(object: Hgdiobj) -> i32;
    fn SetBkMode(hdc: Hdc, mode: i32) -> i32;
    fn SetBkColor(hdc: Hdc, color: u32) -> u32;
    fn SetTextColor(hdc: Hdc, color: u32) -> u32;
    fn SetTextAlign(hdc: Hdc, align: u32) -> u32;
    fn TextOutW(hdc: Hdc, x: i32, y: i32, text: *const u16, count: i32) -> i32;
    fn GetTextFaceW(hdc: Hdc, count: i32, face_name: *mut u16) -> i32;
    fn GetGlyphIndicesW(
        hdc: Hdc,
        text: *const u16,
        count: i32,
        glyph_indices: *mut u16,
        flags: u32,
    ) -> u32;
}

#[link(name = "kernel32")]
extern "system" {
    fn MultiByteToWideChar(
        code_page: u32,
        flags: u32,
        input: *const u8,
        input_length: i32,
        output: *mut u16,
        output_length: i32,
    ) -> i32;
}

struct PrivateFont {
    wide_path: Vec<u16>,
}

impl PrivateFont {
    fn load(path: &Path) -> Result<Self, String> {
        let wide_path = wide_path(path);
        let added = unsafe { AddFontResourceExW(wide_path.as_ptr(), FR_PRIVATE, ptr::null_mut()) };
        if added <= 0 {
            return Err(format!(
                "无法加载字体资源：{}\nWindows 错误：{}",
                path.display(),
                io::Error::last_os_error()
            ));
        }
        Ok(Self { wide_path })
    }
}

impl Drop for PrivateFont {
    fn drop(&mut self) {
        unsafe {
            RemoveFontResourceExW(self.wide_path.as_ptr(), FR_PRIVATE, ptr::null_mut());
        }
    }
}

struct GdiFont {
    hdc: Hdc,
    font: Hfont,
    old_font: Hgdiobj,
    bitmap: Hbitmap,
    old_bitmap: Hgdiobj,
    bits: *mut u32,
    selected_face: String,
}

impl GdiFont {
    fn create(info: &FontInfo, pixel_height: i32) -> Result<Self, String> {
        let face = wide_string(&info.family);
        let hdc = unsafe { CreateCompatibleDC(ptr::null_mut()) };
        if hdc.is_null() {
            return Err(format!(
                "CreateCompatibleDC 失败：{}",
                io::Error::last_os_error()
            ));
        }

        let bitmap_info = BitmapInfo {
            bmiHeader: BitmapInfoHeader {
                biSize: std::mem::size_of::<BitmapInfoHeader>() as u32,
                biWidth: RASTER_WIDTH as i32,
                biHeight: -(RASTER_HEIGHT as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                biSizeImage: (RASTER_WIDTH * RASTER_HEIGHT * 4) as u32,
                ..BitmapInfoHeader::default()
            },
            bmiColors: [0],
        };
        let mut bits = ptr::null_mut();
        let bitmap = unsafe {
            CreateDIBSection(
                hdc,
                &bitmap_info,
                DIB_RGB_COLORS,
                &mut bits,
                ptr::null_mut(),
                0,
            )
        };
        if bitmap.is_null() || bits.is_null() {
            unsafe {
                DeleteDC(hdc);
            }
            return Err(format!(
                "CreateDIBSection 失败：{}",
                io::Error::last_os_error()
            ));
        }

        let old_bitmap = unsafe { SelectObject(hdc, bitmap) };
        if old_bitmap.is_null() || old_bitmap as isize == -1 {
            unsafe {
                DeleteObject(bitmap);
                DeleteDC(hdc);
            }
            return Err("SelectObject 选择扩展栅格位图失败".to_string());
        }

        let font = unsafe {
            CreateFontW(
                -pixel_height,
                0,
                0,
                0,
                i32::from(info.weight),
                u32::from(info.italic),
                0,
                0,
                DEFAULT_CHARSET,
                OUT_DEFAULT_PRECIS,
                CLIP_DEFAULT_PRECIS,
                ANTIALIASED_QUALITY,
                DEFAULT_PITCH,
                face.as_ptr(),
            )
        };
        if font.is_null() {
            unsafe {
                SelectObject(hdc, old_bitmap);
                DeleteObject(bitmap);
                DeleteDC(hdc);
            }
            return Err(format!(
                "CreateFontW 失败：family={}，{}",
                info.family,
                io::Error::last_os_error()
            ));
        }

        let old_font = unsafe { SelectObject(hdc, font) };
        if old_font.is_null() || old_font as isize == -1 {
            unsafe {
                DeleteObject(font);
                SelectObject(hdc, old_bitmap);
                DeleteObject(bitmap);
                DeleteDC(hdc);
            }
            return Err("SelectObject 选择字体失败".to_string());
        }

        unsafe {
            SetBkMode(hdc, TRANSPARENT);
            SetBkColor(hdc, 0x000000);
            SetTextColor(hdc, 0xFFFFFF);
            SetTextAlign(hdc, TA_LEFT_TOP_NO_UPDATE_CP);
        }

        let mut face_buffer = [0u16; 128];
        let face_len =
            unsafe { GetTextFaceW(hdc, face_buffer.len() as i32, face_buffer.as_mut_ptr()) };
        let selected_face = if face_len > 0 {
            String::from_utf16_lossy(&face_buffer[..face_len as usize])
                .trim_matches('\0')
                .to_string()
        } else {
            info.family.clone()
        };

        Ok(Self {
            hdc,
            font,
            old_font,
            bitmap,
            old_bitmap,
            bits: bits.cast(),
            selected_face,
        })
    }

    fn glyph_index(&self, ch: char) -> Result<u16, String> {
        let code = bmp_code_unit(ch)?;
        let mut glyph = 0u16;
        let result = unsafe {
            GetGlyphIndicesW(self.hdc, &code, 1, &mut glyph, GGI_MARK_NONEXISTING_GLYPHS)
        };
        if result == GDI_ERROR {
            return Err(format!(
                "GetGlyphIndicesW({ch}) 失败：{}",
                io::Error::last_os_error()
            ));
        }
        Ok(glyph)
    }

    fn render_glyph(
        &mut self,
        ch: char,
        target_bounds: InkBounds,
    ) -> Result<Option<[u8; GLYPH_BYTES]>, String> {
        let code = bmp_code_unit(ch)?;
        let pixels =
            unsafe { std::slice::from_raw_parts_mut(self.bits, RASTER_WIDTH * RASTER_HEIGHT) };
        pixels.fill(0);
        let result = unsafe { TextOutW(self.hdc, RASTER_ORIGIN_X, RASTER_ORIGIN_Y, &code, 1) };
        if result == 0 {
            return Err(format!(
                "TextOutW({ch}) 失败：{}",
                io::Error::last_os_error()
            ));
        }

        let mut raster = [15u8; RASTER_WIDTH * RASTER_HEIGHT];
        for (index, pixel) in pixels.iter().copied().enumerate() {
            let blue = pixel & 0xFF;
            let green = (pixel >> 8) & 0xFF;
            let red = (pixel >> 16) & 0xFF;
            let intensity = ((red + green + blue + 1) / 3) as u8;
            raster[index] = intensity_to_transparency(intensity);
        }
        let Some(rendered_bounds) = transparency_ink_bounds(&raster, RASTER_WIDTH, RASTER_HEIGHT)
        else {
            return Ok(None);
        };

        let mut canvas = [15u8; GLYPH_WIDTH * GLYPH_HEIGHT];
        let source_width = rendered_bounds.right - rendered_bounds.left + 1;
        let source_height = rendered_bounds.bottom - rendered_bounds.top + 1;
        let target_width = target_bounds.right - target_bounds.left + 1;
        let target_height = target_bounds.bottom - target_bounds.top + 1;

        for target_y in target_bounds.top..=target_bounds.bottom {
            let source_y = rendered_bounds.top
                + scale_coordinate(target_y - target_bounds.top, target_height, source_height);
            for target_x in target_bounds.left..=target_bounds.right {
                let source_x = rendered_bounds.left
                    + scale_coordinate(target_x - target_bounds.left, target_width, source_width);
                canvas[target_y * GLYPH_WIDTH + target_x] =
                    raster[source_y * RASTER_WIDTH + source_x];
            }
        }

        for source_y in rendered_bounds.top..=rendered_bounds.bottom {
            let target_y = target_bounds.top
                + scale_coordinate(source_y - rendered_bounds.top, source_height, target_height);
            for source_x in rendered_bounds.left..=rendered_bounds.right {
                let transparency = raster[source_y * RASTER_WIDTH + source_x];
                if transparency >= 15 {
                    continue;
                }
                let target_x = target_bounds.left
                    + scale_coordinate(source_x - rendered_bounds.left, source_width, target_width);
                let target = &mut canvas[target_y * GLYPH_WIDTH + target_x];
                *target = (*target).min(transparency);
            }
        }
        let Some(placed_bounds) = transparency_ink_bounds(&canvas, GLYPH_WIDTH, GLYPH_HEIGHT)
        else {
            return Ok(None);
        };
        if placed_bounds != target_bounds {
            return Err(format!(
                "字符 {ch} 坐标对齐失败：目标={target_bounds:?}，实际={placed_bounds:?}"
            ));
        }
        Ok(Some(pack_canvas(&canvas)))
    }
}

impl Drop for GdiFont {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.hdc, self.old_font);
            SelectObject(self.hdc, self.old_bitmap);
            DeleteObject(self.font);
            DeleteObject(self.bitmap);
            DeleteDC(self.hdc);
        }
    }
}

#[derive(Debug)]
struct FontInfo {
    family: String,
    full_name: String,
    subfamily: String,
    weight: u16,
    italic: bool,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("\n[error] {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_help();
        return Ok(());
    }
    if args.len() > 1 {
        return Err("参数过多。用法：fn.exe [字体文件]".to_string());
    }

    validate_embedded_map()?;
    let exe_path = env::current_exe().map_err(|e| format!("无法取得 EXE 路径：{e}"))?;
    let base_dir = exe_path
        .parent()
        .ok_or_else(|| "无法取得 EXE 所在目录".to_string())?;
    let input_path = find_case_insensitive_file(base_dir, "FN.DAT")?
        .ok_or_else(|| format!("EXE 同目录没有找到原始 FN.DAT：{}", base_dir.display()))?;
    let output_path = base_dir.join("fn_chs.dat");
    if output_path.exists() {
        return Err(format!(
            "输出已存在，未覆盖：{}\n请先改名或移走旧文件。",
            output_path.display()
        ));
    }

    let font_path = if let Some(arg) = args.first() {
        resolve_user_path(arg, base_dir)?
    } else {
        prompt_font_path(base_dir)?
    };
    let extension = font_path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !matches!(extension.as_str(), "ttf" | "ttc" | "otf") {
        return Err(format!(
            "不支持的字体扩展名：{}\n仅支持 TTF/TTC/OTF。",
            font_path.display()
        ));
    }

    let font_data =
        fs::read(&font_path).map_err(|e| format!("读取字体失败 {}：{e}", font_path.display()))?;
    let font_info = parse_font_info(&font_data)?;
    println!("[font] path={}", font_path.display());
    println!(
        "[font] family={} full_name={} subfamily={} weight={}",
        font_info.family, font_info.full_name, font_info.subfamily, font_info.weight
    );

    let original =
        fs::read(&input_path).map_err(|e| format!("读取 {} 失败：{e}", input_path.display()))?;
    if original.len() != FONT_FILE_BYTES {
        return Err(format!(
            "FN.DAT 大小不正确：实际 {}，预期 {}",
            original.len(),
            FONT_FILE_BYTES
        ));
    }
    let slot_characters = build_slot_characters(&original)?;
    let redraw_slots = slot_characters.iter().flatten().count();
    let blank_slots = GLYPH_COUNT - redraw_slots;

    let _private_font = PrivateFont::load(&font_path)?;
    let mut gdi_font = GdiFont::create(&font_info, FONT_PIXEL_HEIGHT)?;
    println!(
        "[font] selected_face={} pixel_height={} raster={}x{} final={}x{}",
        gdi_font.selected_face,
        FONT_PIXEL_HEIGHT,
        RASTER_WIDTH,
        RASTER_HEIGHT,
        GLYPH_WIDTH,
        GLYPH_HEIGHT
    );

    let missing = collect_missing_glyphs(&gdi_font, &slot_characters)?;
    if !missing.is_empty() {
        let preview: String = missing.iter().take(80).collect();
        println!(
            "[font] warning=所选字体缺少 {} 个字符，将逐槽回退到原 FN.DAT 字形",
            missing.len()
        );
        println!("[font] missing_preview={preview}");
    }
    let mut fallback_characters: HashSet<char> = missing.into_iter().collect();
    let unique_characters = slot_characters
        .iter()
        .flatten()
        .copied()
        .collect::<HashSet<_>>()
        .len();

    let mut output = vec![0xFFu8; FONT_FILE_BYTES];
    let mut changed_slots = 0usize;
    let mut redrawn_slots = 0usize;
    let mut fallback_slots = 0usize;
    for (slot, character) in slot_characters.iter().copied().enumerate() {
        let Some(character) = character else {
            continue;
        };
        let offset = slot * GLYPH_BYTES;
        if fallback_characters.contains(&character) {
            output[offset..offset + GLYPH_BYTES]
                .copy_from_slice(&original[offset..offset + GLYPH_BYTES]);
            fallback_slots += 1;
            continue;
        }
        let target_bounds = packed_ink_bounds(&original[offset..offset + GLYPH_BYTES])
            .ok_or_else(|| format!("有效槽位 {slot} 字符 {character} 的原字形为空"))?;
        let Some(glyph) = gdi_font
            .render_glyph(character, target_bounds)
            .map_err(|e| format!("重绘槽位 {slot} 字符 {character}：{e}"))?
        else {
            fallback_characters.insert(character);
            output[offset..offset + GLYPH_BYTES]
                .copy_from_slice(&original[offset..offset + GLYPH_BYTES]);
            fallback_slots += 1;
            continue;
        };
        let packed_bounds = packed_ink_bounds(&glyph)
            .ok_or_else(|| format!("重绘槽位 {slot} 字符 {character} 打包后为空"))?;
        if packed_bounds != target_bounds {
            return Err(format!(
                "重绘槽位 {slot} 字符 {character} 打包坐标不一致：目标={target_bounds:?}，实际={packed_bounds:?}"
            ));
        }
        redrawn_slots += 1;
        if original[offset..offset + GLYPH_BYTES] != glyph {
            changed_slots += 1;
        }
        output[offset..offset + GLYPH_BYTES].copy_from_slice(&glyph);
    }

    verify_output(&slot_characters, &output)?;
    write_transactional(&output_path, &output)?;

    println!("[fn] source={}", input_path.display());
    println!("[fn] total_slots={GLYPH_COUNT}");
    println!("[fn] embedded_map={}", CN_JP_MAP.len());
    println!("[fn] defined={redraw_slots}");
    println!("[fn] redrawn={redrawn_slots}");
    println!("[fn] fallback_original={fallback_slots}");
    println!("[fn] fallback_characters={}", fallback_characters.len());
    println!("[fn] blank={blank_slots}");
    println!("[fn] unique_characters={unique_characters}");
    println!("[fn] changed_slots={changed_slots}");
    println!("[fn] output_bytes={}", output.len());
    println!("[fn] output={}", output_path.display());
    println!("\n生成完成。");
    Ok(())
}

fn print_help() {
    println!(
        "FN.DAT 中文字库重画工具\n\
         \n\
         用法：\n\
           fn.exe\n\
           fn.exe <字体.ttf|字体.ttc|字体.otf>\n\
         \n\
         要求：\n\
           1. fn.exe 同目录必须有原始 FN.DAT。\n\
           2. 不传参数时，程序提示输入字体路径；直接回车会使用同目录字体。\n\
           3. 内置 3018 条 subs_cn_jp 中文→CP932 借码映射。\n\
           4. 按原 FN.DAT 的 8836 个槽位整体重建：有效字形统一重画，未定义槽位保持透明。\n\
           5. 字形先按 24px 原生栅格化，再重采样到原槽位的完整墨迹矩形；标点的上下左右坐标保持与原 FN 一致。\n\
           6. 所选字体缺少字符时，该字符的槽位回退复制原 FN.DAT 字形，不使用系统字体替代。\n\
           7. 输出同目录 fn_chs.dat；已有输出时拒绝覆盖。\n"
    );
}

fn validate_embedded_map() -> Result<(), String> {
    if CN_JP_MAP.len() != EXPECTED_MAP_COUNT {
        return Err(format!(
            "内置映射数量错误：实际 {}，预期 {}",
            CN_JP_MAP.len(),
            EXPECTED_MAP_COUNT
        ));
    }
    let mut targets = HashSet::with_capacity(CN_JP_MAP.len());
    let mut sources = HashSet::with_capacity(CN_JP_MAP.len());
    let mut slots = HashSet::with_capacity(CN_JP_MAP.len());
    for entry in CN_JP_MAP {
        if entry.glyph_index >= GLYPH_COUNT {
            return Err(format!(
                "内置映射越界：{}→{}，slot={}",
                entry.target, entry.source, entry.glyph_index
            ));
        }
        if !targets.insert(entry.target) {
            return Err(format!("内置映射中文字符重复：{}", entry.target));
        }
        if !sources.insert(entry.source) {
            return Err(format!("内置映射借码字符重复：{}", entry.source));
        }
        if !slots.insert(entry.glyph_index) {
            return Err(format!("内置映射槽位重复：{}", entry.glyph_index));
        }
    }
    Ok(())
}

fn build_slot_characters(original: &[u8]) -> Result<Vec<Option<char>>, String> {
    if original.len() != FONT_FILE_BYTES {
        return Err(format!(
            "FN.DAT 大小不正确：实际 {}，预期 {}",
            original.len(),
            FONT_FILE_BYTES
        ));
    }

    let mut overrides = vec![None; GLYPH_COUNT];
    for entry in CN_JP_MAP {
        let offset = entry.glyph_index * GLYPH_BYTES;
        if original[offset..offset + GLYPH_BYTES]
            .iter()
            .all(|byte| *byte == 0xFF)
        {
            return Err(format!(
                "内置映射槽位 {}（{}→{}）在原始 FN.DAT 中是空槽",
                entry.glyph_index, entry.target, entry.source
            ));
        }
        let decoded = decode_slot_character(entry.glyph_index)?;
        if decoded != entry.source {
            let (lead, trail) = slot_to_sjis(entry.glyph_index)?;
            return Err(format!(
                "内置映射与 JIS 槽位不一致：slot={} SJIS={lead:02X}{trail:02X}，表内源字符={}，实际解码={decoded}",
                entry.glyph_index, entry.source
            ));
        }
        overrides[entry.glyph_index] = Some(entry.target);
    }

    let mut characters = Vec::with_capacity(GLYPH_COUNT);
    for (slot, override_character) in overrides.into_iter().enumerate() {
        if let Some(character) = override_character {
            characters.push(Some(character));
            continue;
        }
        let offset = slot * GLYPH_BYTES;
        if original[offset..offset + GLYPH_BYTES]
            .iter()
            .all(|byte| *byte == 0xFF)
        {
            characters.push(None);
        } else {
            characters.push(Some(decode_slot_character(slot)?));
        }
    }
    Ok(characters)
}

fn slot_to_sjis(slot: usize) -> Result<(u8, u8), String> {
    if slot >= GLYPH_COUNT {
        return Err(format!("JIS 槽位越界：{slot}"));
    }
    let row = 0x21u16 + (slot / 94) as u16;
    let cell = 0x21u16 + (slot % 94) as u16;
    let mut lead = 0x81u16 + (row - 0x21) / 2;
    if lead >= 0xA0 {
        lead += 0x40;
    }
    let mut trail = cell + if row & 1 == 1 { 0x1F } else { 0x7D };
    if trail >= 0x7F {
        trail += 1;
    }
    Ok((lead as u8, trail as u8))
}

fn decode_slot_character(slot: usize) -> Result<char, String> {
    let (lead, trail) = slot_to_sjis(slot)?;
    if (lead, trail) == (0x87, 0x9E) {
        return Ok('☞');
    }

    let input = [lead, trail];
    let mut output = [0u16; 2];
    let count = unsafe {
        MultiByteToWideChar(
            CP932_CODE_PAGE,
            MB_ERR_INVALID_CHARS,
            input.as_ptr(),
            input.len() as i32,
            output.as_mut_ptr(),
            output.len() as i32,
        )
    };
    if count != 1 {
        return Err(format!(
            "槽位 {slot} 的 Shift-JIS 字节 {lead:02X}{trail:02X} 无法按 CP932 解码"
        ));
    }
    char::from_u32(u32::from(output[0])).ok_or_else(|| {
        format!("槽位 {slot} 的 Shift-JIS 字节 {lead:02X}{trail:02X} 解码为非法 Unicode")
    })
}

fn collect_missing_glyphs(
    font: &GdiFont,
    slot_characters: &[Option<char>],
) -> Result<Vec<char>, String> {
    let mut missing = Vec::new();
    let mut checked = HashSet::new();
    for character in slot_characters.iter().flatten().copied() {
        if checked.insert(character) && font.glyph_index(character)? == 0xFFFF {
            missing.push(character);
        }
    }
    Ok(missing)
}

fn verify_output(slot_characters: &[Option<char>], after: &[u8]) -> Result<(), String> {
    if slot_characters.len() != GLYPH_COUNT || after.len() != FONT_FILE_BYTES {
        return Err("输出验证失败：文件长度改变".to_string());
    }
    for slot in 0..GLYPH_COUNT {
        let offset = slot * GLYPH_BYTES;
        let end = offset + GLYPH_BYTES;
        let is_blank = after[offset..end].iter().all(|byte| *byte == 0xFF);
        match slot_characters[slot] {
            Some(character) if is_blank => {
                return Err(format!(
                    "输出验证失败：槽位 {slot} 字符 {character} 是空字形"
                ));
            }
            None if !is_blank => {
                return Err(format!("输出验证失败：未定义槽位 {slot} 不是透明空槽"));
            }
            _ => {}
        }
    }
    Ok(())
}

fn write_transactional(output: &Path, data: &[u8]) -> Result<(), String> {
    let file_name = output
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("非法输出路径：{}", output.display()))?;
    let temp = output.with_file_name(format!(".{file_name}.tmp-{}", std::process::id()));
    let result = (|| -> Result<(), String> {
        let mut file = File::create(&temp)
            .map_err(|e| format!("创建临时输出 {} 失败：{e}", temp.display()))?;
        file.write_all(data)
            .map_err(|e| format!("写入临时输出 {} 失败：{e}", temp.display()))?;
        file.sync_all()
            .map_err(|e| format!("同步临时输出 {} 失败：{e}", temp.display()))?;
        drop(file);
        fs::rename(&temp, output).map_err(|e| {
            format!(
                "提交输出 {} -> {} 失败：{e}",
                temp.display(),
                output.display()
            )
        })?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

fn prompt_font_path(base_dir: &Path) -> Result<PathBuf, String> {
    let candidates = find_font_files(base_dir)?;
    println!("请输入字体文件路径（TTF/TTC/OTF）：");
    if candidates.is_empty() {
        println!("  同目录没有检测到字体，请输入完整路径。");
    } else {
        println!("  已检测到同目录字体，直接回车自动使用：");
        for (index, path) in candidates.iter().enumerate() {
            println!("    {}. {}", index + 1, path.display());
        }
    }
    print!("> ");
    io::stdout()
        .flush()
        .map_err(|e| format!("刷新输入提示失败：{e}"))?;
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| format!("读取字体路径失败：{e}"))?;
    let input = trim_path_input(&input);
    if input.is_empty() {
        return candidates
            .into_iter()
            .next()
            .ok_or_else(|| "没有输入字体路径，且 EXE 同目录没有可自动使用的字体".to_string());
    }
    resolve_user_path(input, base_dir)
}

fn find_font_files(base_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut result = Vec::new();
    let entries = fs::read_dir(base_dir)
        .map_err(|e| format!("读取 EXE 目录 {} 失败：{e}", base_dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("读取 EXE 目录项失败：{e}"))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        if matches!(extension.as_str(), "ttf" | "ttc" | "otf") {
            result.push(path);
        }
    }
    result.sort_by_key(|path| {
        path.file_name()
            .map(|name| name.to_string_lossy().to_lowercase())
            .unwrap_or_default()
    });
    Ok(result)
}

fn find_case_insensitive_file(base_dir: &Path, wanted: &str) -> Result<Option<PathBuf>, String> {
    let exact = base_dir.join(wanted);
    if exact.is_file() {
        return Ok(Some(exact));
    }
    let entries =
        fs::read_dir(base_dir).map_err(|e| format!("读取目录 {} 失败：{e}", base_dir.display()))?;
    for entry in entries {
        let path = entry.map_err(|e| format!("读取目录项失败：{e}"))?.path();
        let is_match = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case(wanted));
        if is_match && path.is_file() {
            return Ok(Some(path));
        }
    }
    Ok(None)
}

fn resolve_user_path(input: &str, base_dir: &Path) -> Result<PathBuf, String> {
    let value = PathBuf::from(trim_path_input(input));
    let candidates = if value.is_absolute() {
        vec![value]
    } else {
        let mut paths = Vec::with_capacity(2);
        if let Ok(current_dir) = env::current_dir() {
            paths.push(current_dir.join(&value));
        }
        paths.push(base_dir.join(value));
        paths
    };
    for path in candidates {
        if path.is_file() {
            return fs::canonicalize(&path)
                .map_err(|e| format!("解析字体路径 {} 失败：{e}", path.display()));
        }
    }
    Err(format!("字体文件不存在：{}", input.trim()))
}

fn trim_path_input(input: &str) -> &str {
    input.trim().trim_matches('"').trim_matches('\'')
}

fn intensity_to_transparency(intensity: u8) -> u8 {
    let opacity = (u16::from(intensity) * 15 + 127) / 255;
    15 - opacity as u8
}

fn scale_coordinate(index: usize, from_length: usize, to_length: usize) -> usize {
    if from_length <= 1 || to_length <= 1 {
        return 0;
    }
    (index * (to_length - 1) + (from_length - 1) / 2) / (from_length - 1)
}

fn transparency_ink_bounds(pixels: &[u8], width: usize, height: usize) -> Option<InkBounds> {
    if pixels.len() != width.checked_mul(height)? {
        return None;
    }
    let mut bounds: Option<InkBounds> = None;
    for y in 0..height {
        for x in 0..width {
            if pixels[y * width + x] >= 15 {
                continue;
            }
            match &mut bounds {
                Some(bounds) => {
                    bounds.left = bounds.left.min(x);
                    bounds.top = bounds.top.min(y);
                    bounds.right = bounds.right.max(x);
                    bounds.bottom = bounds.bottom.max(y);
                }
                None => {
                    bounds = Some(InkBounds {
                        left: x,
                        top: y,
                        right: x,
                        bottom: y,
                    });
                }
            }
        }
    }
    bounds
}

fn packed_ink_bounds(glyph: &[u8]) -> Option<InkBounds> {
    if glyph.len() != GLYPH_BYTES {
        return None;
    }
    let mut canvas = [15u8; GLYPH_WIDTH * GLYPH_HEIGHT];
    for y in 0..GLYPH_HEIGHT {
        for x in 0..GLYPH_WIDTH {
            let packed = glyph[y * (GLYPH_WIDTH / 2) + x / 2];
            canvas[y * GLYPH_WIDTH + x] = if x & 1 == 0 {
                packed & 0x0F
            } else {
                packed >> 4
            };
        }
    }
    transparency_ink_bounds(&canvas, GLYPH_WIDTH, GLYPH_HEIGHT)
}

fn pack_canvas(canvas: &[u8; GLYPH_WIDTH * GLYPH_HEIGHT]) -> [u8; GLYPH_BYTES] {
    let mut packed = [0xFFu8; GLYPH_BYTES];
    for (index, pair) in canvas.chunks_exact(2).enumerate() {
        packed[index] = (pair[0] & 0x0F) | ((pair[1] & 0x0F) << 4);
    }
    packed
}

fn bmp_code_unit(ch: char) -> Result<u16, String> {
    let code = u32::from(ch);
    u16::try_from(code).map_err(|_| format!("暂不支持非 BMP 字符：{ch} U+{code:05X}"))
}

fn wide_string(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn wide_path(path: &Path) -> Vec<u16> {
    path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn parse_font_info(data: &[u8]) -> Result<FontInfo, String> {
    let sfnt_offset = first_sfnt_offset(data)?;
    let name_table = find_sfnt_table(data, sfnt_offset, b"name")?
        .ok_or_else(|| "字体没有 name 表".to_string())?;
    let os2_table = find_sfnt_table(data, sfnt_offset, b"OS/2")?;

    let family_typographic = read_best_name(data, name_table, 16)?;
    let family_legacy = read_best_name(data, name_table, 1)?;
    let family = family_typographic
        .filter(|name| name.encode_utf16().count() < 32)
        .or(family_legacy)
        .ok_or_else(|| "无法从字体 name 表读取 family 名称".to_string())?;
    let full_name = read_best_name(data, name_table, 4)?.unwrap_or_else(|| family.clone());
    let subfamily = read_best_name(data, name_table, 17)?
        .or(read_best_name(data, name_table, 2)?)
        .unwrap_or_else(|| "Regular".to_string());

    let weight = if let Some((offset, length)) = os2_table {
        if length >= 6 {
            read_be_u16(data, offset + 4)?.clamp(100, 900)
        } else {
            infer_weight(&subfamily)
        }
    } else {
        infer_weight(&subfamily)
    };
    let lower_subfamily = subfamily.to_lowercase();
    let italic = lower_subfamily.contains("italic") || lower_subfamily.contains("oblique");
    Ok(FontInfo {
        family,
        full_name,
        subfamily,
        weight,
        italic,
    })
}

fn infer_weight(subfamily: &str) -> u16 {
    let value = subfamily.to_lowercase();
    if value.contains("black") || value.contains("heavy") {
        900
    } else if value.contains("extra bold") || value.contains("extrabold") {
        800
    } else if value.contains("bold") {
        700
    } else if value.contains("semi bold") || value.contains("semibold") {
        600
    } else if value.contains("medium") {
        500
    } else if value.contains("light") {
        300
    } else if value.contains("thin") {
        100
    } else {
        400
    }
}

fn first_sfnt_offset(data: &[u8]) -> Result<usize, String> {
    if data.len() < 12 {
        return Err("字体文件过小".to_string());
    }
    if data.get(0..4) == Some(b"ttcf") {
        let count = read_be_u32(data, 8)? as usize;
        if count == 0 {
            return Err("TTC 中没有字体".to_string());
        }
        let offset = read_be_u32(data, 12)? as usize;
        check_range(data, offset, 12, "TTC 第一个字体")?;
        Ok(offset)
    } else {
        Ok(0)
    }
}

fn find_sfnt_table(
    data: &[u8],
    sfnt_offset: usize,
    wanted_tag: &[u8; 4],
) -> Result<Option<(usize, usize)>, String> {
    check_range(data, sfnt_offset, 12, "SFNT 表头")?;
    let table_count = read_be_u16(data, sfnt_offset + 4)? as usize;
    let records_offset = sfnt_offset + 12;
    check_range(
        data,
        records_offset,
        table_count
            .checked_mul(16)
            .ok_or_else(|| "SFNT 表数量溢出".to_string())?,
        "SFNT 表目录",
    )?;
    for index in 0..table_count {
        let record = records_offset + index * 16;
        if data.get(record..record + 4) == Some(wanted_tag) {
            let offset = read_be_u32(data, record + 8)? as usize;
            let length = read_be_u32(data, record + 12)? as usize;
            check_range(data, offset, length, "SFNT 子表")?;
            return Ok(Some((offset, length)));
        }
    }
    Ok(None)
}

fn read_best_name(
    data: &[u8],
    table: (usize, usize),
    wanted_name_id: u16,
) -> Result<Option<String>, String> {
    let (table_offset, table_length) = table;
    if table_length < 6 {
        return Err("字体 name 表过小".to_string());
    }
    let count = read_be_u16(data, table_offset + 2)? as usize;
    let strings_offset = table_offset + read_be_u16(data, table_offset + 4)? as usize;
    let records_offset = table_offset + 6;
    check_range(
        data,
        records_offset,
        count
            .checked_mul(12)
            .ok_or_else(|| "字体 name 记录数量溢出".to_string())?,
        "字体 name 记录",
    )?;

    let mut best: Option<(i32, String)> = None;
    for index in 0..count {
        let record = records_offset + index * 12;
        let platform = read_be_u16(data, record)?;
        let language = read_be_u16(data, record + 4)?;
        let name_id = read_be_u16(data, record + 6)?;
        if name_id != wanted_name_id || !matches!(platform, 0 | 3) {
            continue;
        }
        let length = read_be_u16(data, record + 8)? as usize;
        let relative = read_be_u16(data, record + 10)? as usize;
        let start = strings_offset
            .checked_add(relative)
            .ok_or_else(|| "字体 name 字符串偏移溢出".to_string())?;
        check_range(data, start, length, "字体 name 字符串")?;
        if length % 2 != 0 {
            continue;
        }
        let mut units = Vec::with_capacity(length / 2);
        for chunk in data[start..start + length].chunks_exact(2) {
            units.push(u16::from_be_bytes([chunk[0], chunk[1]]));
        }
        let value = String::from_utf16_lossy(&units)
            .trim_matches('\0')
            .trim()
            .to_string();
        if value.is_empty() {
            continue;
        }
        let platform_score = if platform == 3 { 100 } else { 90 };
        let language_score = match language {
            0x0804 | 0x0404 | 0x0C04 | 0x1004 => 30,
            0x0411 => 20,
            0x0409 | 0 => 10,
            _ => 0,
        };
        let score = platform_score + language_score;
        if best
            .as_ref()
            .is_none_or(|(best_score, _)| score > *best_score)
        {
            best = Some((score, value));
        }
    }
    Ok(best.map(|(_, value)| value))
}

fn check_range(data: &[u8], offset: usize, length: usize, what: &str) -> Result<(), String> {
    let end = offset
        .checked_add(length)
        .ok_or_else(|| format!("{what} 范围溢出"))?;
    if end > data.len() {
        return Err(format!(
            "{what} 越界：offset=0x{offset:X}, length=0x{length:X}, file=0x{:X}",
            data.len()
        ));
    }
    Ok(())
}

fn read_be_u16(data: &[u8], offset: usize) -> Result<u16, String> {
    check_range(data, offset, 2, "u16")?;
    Ok(u16::from_be_bytes([data[offset], data[offset + 1]]))
}

fn read_be_u32(data: &[u8], offset: usize) -> Result<u32, String> {
    check_range(data, offset, 4, "u32")?;
    Ok(u32::from_be_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_map_is_valid() {
        validate_embedded_map().unwrap();
    }

    #[test]
    fn opacity_conversion_matches_fn_format() {
        assert_eq!(intensity_to_transparency(0), 15);
        assert_eq!(intensity_to_transparency(255), 0);
    }

    #[test]
    fn low_nibble_is_left_pixel() {
        let mut canvas = [15u8; GLYPH_WIDTH * GLYPH_HEIGHT];
        canvas[0] = 1;
        canvas[1] = 2;
        assert_eq!(pack_canvas(&canvas)[0], 0x21);
    }

    #[test]
    fn packed_bounds_preserve_original_coordinates() {
        let mut canvas = [15u8; GLYPH_WIDTH * GLYPH_HEIGHT];
        canvas[17 * GLYPH_WIDTH + 2] = 0;
        canvas[22 * GLYPH_WIDTH + 9] = 1;
        assert_eq!(
            packed_ink_bounds(&pack_canvas(&canvas)),
            Some(InkBounds {
                left: 2,
                top: 17,
                right: 9,
                bottom: 22,
            })
        );
    }

    #[test]
    fn coordinate_scaling_keeps_both_edges() {
        assert_eq!(scale_coordinate(0, 13, 7), 0);
        assert_eq!(scale_coordinate(12, 13, 7), 6);
        assert_eq!(scale_coordinate(0, 7, 13), 0);
        assert_eq!(scale_coordinate(6, 7, 13), 12);
    }

    #[test]
    fn jis_slots_convert_to_expected_shift_jis() {
        assert_eq!(slot_to_sjis(0).unwrap(), (0x81, 0x40));
        assert_eq!(slot_to_sjis(283).unwrap(), (0x82, 0xA0));
        assert_eq!(slot_to_sjis(1221).unwrap(), (0x87, 0x9E));
    }

    #[test]
    fn exceptional_original_symbol_is_preserved() {
        assert_eq!(decode_slot_character(1221).unwrap(), '☞');
    }
}
