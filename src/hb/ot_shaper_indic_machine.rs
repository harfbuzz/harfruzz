#![allow(
    dead_code,
    non_upper_case_globals,
    unused_assignments,
    unused_parens,
    while_true,
    clippy::assign_op_pattern,
    clippy::comparison_chain,
    clippy::double_parens,
    clippy::unnecessary_cast,
    clippy::single_match,
    clippy::never_loop
)]

use super::buffer::{hb_buffer_t, HB_BUFFER_SCRATCH_FLAG_HAS_BROKEN_SYLLABLE};

static _indic_syllable_machine_trans_keys: [u8; 278] = [
    7, 19, 3, 19, 4, 19, 4, 19, 12, 12, 3, 19, 3, 19, 3, 19, 7, 19, 4, 19, 4, 19, 12, 12, 3, 19, 3,
    19, 3, 19, 3, 19, 7, 19, 4, 19, 4, 19, 12, 12, 3, 19, 3, 19, 3, 19, 7, 19, 4, 19, 4, 19, 12,
    12, 3, 19, 3, 19, 4, 19, 7, 19, 0, 19, 2, 19, 2, 19, 3, 19, 0, 19, 4, 19, 4, 19, 8, 8, 4, 8, 0,
    19, 0, 19, 0, 19, 2, 19, 3, 19, 4, 19, 4, 19, 3, 19, 4, 19, 2, 19, 4, 19, 2, 19, 2, 19, 2, 19,
    2, 19, 3, 19, 0, 19, 2, 19, 2, 19, 3, 19, 0, 19, 4, 19, 8, 8, 4, 8, 0, 19, 0, 19, 2, 19, 3, 19,
    4, 19, 4, 19, 3, 19, 4, 19, 4, 19, 2, 19, 4, 19, 2, 19, 2, 19, 3, 19, 2, 19, 2, 19, 3, 19, 0,
    19, 2, 19, 0, 19, 4, 19, 8, 8, 4, 8, 0, 19, 0, 19, 2, 19, 3, 19, 4, 19, 4, 19, 2, 19, 3, 19, 4,
    19, 4, 19, 2, 19, 4, 19, 2, 19, 3, 19, 3, 19, 2, 19, 2, 19, 3, 19, 0, 19, 2, 19, 0, 19, 4, 19,
    8, 8, 4, 8, 0, 19, 0, 19, 2, 19, 3, 19, 4, 19, 4, 19, 2, 19, 3, 19, 4, 19, 4, 19, 2, 19, 4, 19,
    0, 19, 2, 19, 0, 19, 3, 19, 4, 19, 4, 19, 8, 8, 4, 8, 0, 19, 2, 19, 4, 19, 4, 19, 8, 8, 4, 8,
    0, 14, 0, 0,
];
static _indic_syllable_machine_char_class: [i8; 59] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 18, 18, 18, 18, 18, 18, 18,
    18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18,
    18, 18, 18, 18, 18, 18, 19, 0, 0,
];
static _indic_syllable_machine_index_offsets: [i16; 140] = [
    0, 13, 30, 46, 62, 63, 80, 97, 114, 127, 143, 159, 160, 177, 194, 211, 228, 241, 257, 273, 274,
    291, 308, 325, 338, 354, 370, 371, 388, 405, 421, 434, 454, 472, 490, 507, 527, 543, 559, 560,
    565, 585, 605, 625, 643, 660, 676, 692, 709, 725, 743, 759, 777, 795, 813, 831, 848, 868, 886,
    904, 921, 941, 957, 958, 963, 983, 1003, 1021, 1038, 1054, 1070, 1087, 1103, 1119, 1137, 1153,
    1171, 1189, 1206, 1224, 1242, 1259, 1279, 1297, 1317, 1333, 1334, 1339, 1359, 1379, 1397, 1414,
    1430, 1446, 1464, 1481, 1497, 1513, 1531, 1547, 1565, 1582, 1599, 1617, 1635, 1652, 1672, 1690,
    1710, 1726, 1727, 1732, 1752, 1772, 1790, 1807, 1823, 1839, 1857, 1874, 1890, 1906, 1924, 1940,
    1960, 1978, 1998, 2015, 2031, 2047, 2048, 2053, 2073, 2091, 2107, 2123, 2124, 2129, 0, 0,
];
static _indic_syllable_machine_indices: [i16; 2146] = [
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 3, 4, 5, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 5, 3, 3,
    4, 6, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 6, 3, 3, 4, 5, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 5, 4, 7,
    3, 3, 4, 5, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 5, 2, 3, 3, 4, 5, 0, 0, 0, 8, 4, 0, 0, 0, 0, 0, 0,
    5, 10, 11, 11, 12, 13, 9, 9, 9, 9, 12, 9, 9, 9, 9, 9, 9, 13, 14, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    9, 14, 11, 11, 12, 15, 9, 9, 9, 9, 12, 9, 9, 9, 9, 9, 9, 15, 11, 11, 12, 13, 9, 9, 9, 9, 12, 9,
    9, 9, 9, 9, 9, 13, 12, 16, 11, 11, 12, 13, 9, 9, 9, 9, 12, 9, 9, 9, 9, 9, 9, 13, 10, 11, 11,
    12, 13, 9, 9, 9, 17, 12, 9, 9, 9, 9, 9, 9, 13, 10, 11, 11, 12, 13, 9, 9, 9, 18, 12, 9, 9, 9, 9,
    9, 9, 13, 20, 21, 21, 22, 23, 19, 19, 19, 24, 22, 19, 19, 19, 19, 19, 19, 23, 25, 19, 19, 19,
    19, 19, 19, 19, 19, 19, 19, 19, 25, 21, 21, 22, 27, 26, 26, 26, 26, 22, 26, 26, 26, 26, 26, 26,
    27, 21, 21, 22, 23, 19, 19, 19, 19, 22, 19, 19, 19, 19, 19, 19, 23, 22, 20, 21, 21, 22, 23, 19,
    19, 19, 19, 22, 19, 19, 19, 19, 19, 19, 23, 28, 21, 21, 22, 23, 19, 19, 19, 19, 22, 19, 19, 19,
    19, 19, 19, 23, 30, 31, 31, 32, 33, 29, 29, 29, 34, 32, 29, 29, 29, 29, 29, 29, 33, 35, 29, 29,
    29, 29, 29, 29, 29, 29, 29, 29, 29, 35, 31, 31, 32, 36, 29, 29, 29, 29, 32, 29, 29, 29, 29, 29,
    29, 36, 31, 31, 32, 33, 29, 29, 29, 29, 32, 29, 29, 29, 29, 29, 29, 33, 32, 30, 31, 31, 32, 33,
    29, 29, 29, 29, 32, 29, 29, 29, 29, 29, 29, 33, 37, 31, 31, 32, 33, 29, 29, 29, 29, 32, 29, 29,
    29, 29, 29, 29, 33, 21, 21, 22, 38, 0, 0, 0, 0, 22, 0, 0, 0, 0, 0, 0, 38, 40, 39, 39, 39, 39,
    39, 39, 39, 39, 39, 39, 39, 40, 43, 44, 45, 46, 47, 48, 22, 23, 49, 50, 50, 24, 22, 51, 52, 53,
    54, 55, 42, 56, 58, 59, 60, 61, 4, 5, 62, 57, 57, 8, 4, 57, 57, 63, 57, 57, 57, 5, 64, 59, 65,
    65, 4, 5, 62, 57, 57, 57, 4, 57, 57, 63, 57, 57, 57, 5, 59, 65, 65, 4, 5, 62, 57, 57, 57, 4,
    57, 57, 63, 57, 57, 57, 5, 43, 57, 57, 57, 66, 67, 57, 1, 62, 57, 57, 57, 57, 57, 43, 57, 57,
    57, 57, 1, 68, 68, 57, 1, 62, 57, 57, 57, 57, 57, 57, 57, 57, 57, 57, 1, 62, 57, 57, 69, 62,
    57, 57, 57, 57, 57, 57, 57, 57, 57, 57, 69, 62, 62, 57, 57, 57, 62, 43, 57, 70, 57, 68, 68, 57,
    1, 62, 57, 57, 57, 57, 57, 43, 57, 57, 57, 57, 1, 43, 57, 57, 57, 68, 68, 57, 1, 62, 57, 57,
    57, 57, 57, 43, 57, 57, 57, 57, 1, 43, 57, 57, 57, 68, 67, 57, 1, 62, 57, 57, 57, 57, 57, 43,
    57, 57, 57, 57, 1, 71, 72, 73, 73, 4, 5, 62, 57, 57, 57, 4, 57, 57, 57, 57, 57, 57, 5, 72, 73,
    73, 4, 5, 62, 57, 57, 57, 4, 57, 57, 57, 57, 57, 57, 5, 73, 73, 4, 5, 62, 57, 57, 57, 4, 57,
    57, 57, 57, 57, 57, 5, 62, 57, 57, 69, 62, 57, 57, 57, 4, 57, 57, 57, 57, 57, 57, 69, 74, 75,
    75, 4, 5, 62, 57, 57, 57, 4, 57, 57, 57, 57, 57, 57, 5, 66, 76, 57, 1, 62, 57, 57, 57, 57, 57,
    57, 57, 57, 57, 57, 1, 66, 57, 68, 68, 57, 1, 62, 57, 57, 57, 57, 57, 57, 57, 57, 57, 57, 1,
    68, 76, 57, 1, 62, 57, 57, 57, 57, 57, 57, 57, 57, 57, 57, 1, 58, 59, 65, 65, 4, 5, 62, 57, 57,
    57, 4, 57, 57, 63, 57, 57, 57, 5, 58, 59, 60, 65, 4, 5, 62, 57, 57, 8, 4, 57, 57, 63, 57, 57,
    57, 5, 78, 79, 80, 81, 12, 13, 82, 77, 77, 18, 12, 77, 77, 83, 77, 77, 77, 13, 84, 79, 85, 81,
    12, 13, 82, 77, 77, 77, 12, 77, 77, 83, 77, 77, 77, 13, 79, 85, 81, 12, 13, 82, 77, 77, 77, 12,
    77, 77, 83, 77, 77, 77, 13, 86, 77, 77, 77, 87, 88, 77, 14, 82, 77, 77, 77, 77, 77, 86, 77, 77,
    77, 77, 14, 89, 79, 90, 91, 12, 13, 82, 77, 77, 17, 12, 77, 77, 83, 77, 77, 77, 13, 92, 79, 85,
    85, 12, 13, 82, 77, 77, 77, 12, 77, 77, 83, 77, 77, 77, 13, 79, 85, 85, 12, 13, 82, 77, 77, 77,
    12, 77, 77, 83, 77, 77, 77, 13, 86, 77, 77, 77, 93, 88, 77, 14, 82, 77, 77, 77, 77, 77, 86, 77,
    77, 77, 77, 14, 82, 77, 77, 94, 82, 77, 77, 77, 77, 77, 77, 77, 77, 77, 77, 94, 82, 82, 77, 77,
    77, 82, 86, 77, 95, 77, 93, 93, 77, 14, 82, 77, 77, 77, 77, 77, 86, 77, 77, 77, 77, 14, 86, 77,
    77, 77, 93, 93, 77, 14, 82, 77, 77, 77, 77, 77, 86, 77, 77, 77, 77, 14, 96, 97, 98, 98, 12, 13,
    82, 77, 77, 77, 12, 77, 77, 77, 77, 77, 77, 13, 97, 98, 98, 12, 13, 82, 77, 77, 77, 12, 77, 77,
    77, 77, 77, 77, 13, 98, 98, 12, 13, 82, 77, 77, 77, 12, 77, 77, 77, 77, 77, 77, 13, 82, 77, 77,
    94, 82, 77, 77, 77, 12, 77, 77, 77, 77, 77, 77, 94, 99, 100, 100, 12, 13, 82, 77, 77, 77, 12,
    77, 77, 77, 77, 77, 77, 13, 87, 101, 77, 14, 82, 77, 77, 77, 77, 77, 77, 77, 77, 77, 77, 14,
    93, 93, 77, 14, 82, 77, 77, 77, 77, 77, 77, 77, 77, 77, 77, 14, 87, 77, 93, 93, 77, 14, 82, 77,
    77, 77, 77, 77, 77, 77, 77, 77, 77, 14, 93, 101, 77, 14, 82, 77, 77, 77, 77, 77, 77, 77, 77,
    77, 77, 14, 89, 79, 85, 85, 12, 13, 82, 77, 77, 77, 12, 77, 77, 83, 77, 77, 77, 13, 89, 79, 90,
    85, 12, 13, 82, 77, 77, 17, 12, 77, 77, 83, 77, 77, 77, 13, 10, 11, 11, 12, 13, 77, 77, 77, 77,
    12, 77, 77, 77, 77, 77, 77, 13, 78, 79, 85, 81, 12, 13, 82, 77, 77, 77, 12, 77, 77, 83, 77, 77,
    77, 13, 103, 46, 104, 104, 22, 23, 49, 102, 102, 102, 22, 102, 102, 53, 102, 102, 102, 23, 46,
    104, 104, 22, 23, 49, 102, 102, 102, 22, 102, 102, 53, 102, 102, 102, 23, 105, 102, 102, 102,
    106, 107, 102, 25, 49, 102, 102, 102, 102, 102, 105, 102, 102, 102, 102, 25, 45, 46, 108, 109,
    22, 23, 49, 102, 102, 24, 22, 102, 102, 53, 102, 102, 102, 23, 105, 102, 102, 102, 110, 107,
    102, 25, 49, 102, 102, 102, 102, 102, 105, 102, 102, 102, 102, 25, 49, 102, 102, 111, 49, 102,
    102, 102, 102, 102, 102, 102, 102, 102, 102, 111, 49, 49, 102, 102, 102, 49, 105, 102, 112,
    102, 110, 110, 102, 25, 49, 102, 102, 102, 102, 102, 105, 102, 102, 102, 102, 25, 105, 102,
    102, 102, 110, 110, 102, 25, 49, 102, 102, 102, 102, 102, 105, 102, 102, 102, 102, 25, 113,
    114, 115, 115, 22, 23, 49, 102, 102, 102, 22, 102, 102, 102, 102, 102, 102, 23, 114, 115, 115,
    22, 23, 49, 102, 102, 102, 22, 102, 102, 102, 102, 102, 102, 23, 115, 115, 22, 23, 49, 102,
    102, 102, 22, 102, 102, 102, 102, 102, 102, 23, 49, 26, 26, 111, 49, 26, 26, 26, 22, 26, 26,
    26, 26, 26, 26, 111, 45, 46, 104, 104, 22, 23, 49, 102, 102, 102, 22, 102, 102, 53, 102, 102,
    102, 23, 116, 117, 117, 22, 23, 49, 102, 102, 102, 22, 102, 102, 102, 102, 102, 102, 23, 106,
    118, 102, 25, 49, 102, 102, 102, 102, 102, 102, 102, 102, 102, 102, 25, 110, 110, 102, 25, 49,
    102, 102, 102, 102, 102, 102, 102, 102, 102, 102, 25, 106, 102, 110, 110, 102, 25, 49, 102,
    102, 102, 102, 102, 102, 102, 102, 102, 102, 25, 110, 118, 102, 25, 49, 102, 102, 102, 102,
    102, 102, 102, 102, 102, 102, 25, 45, 46, 108, 104, 22, 23, 49, 102, 102, 24, 22, 102, 102, 53,
    102, 102, 102, 23, 20, 21, 21, 22, 23, 119, 119, 119, 24, 22, 119, 119, 119, 119, 119, 119, 23,
    20, 21, 21, 22, 23, 119, 119, 119, 119, 22, 119, 119, 119, 119, 119, 119, 23, 121, 122, 123,
    124, 32, 33, 125, 120, 120, 34, 32, 120, 120, 126, 120, 120, 120, 33, 127, 122, 124, 124, 32,
    33, 125, 120, 120, 120, 32, 120, 120, 126, 120, 120, 120, 33, 122, 124, 124, 32, 33, 125, 120,
    120, 120, 32, 120, 120, 126, 120, 120, 120, 33, 128, 120, 120, 120, 129, 130, 120, 35, 125,
    120, 120, 120, 120, 120, 128, 120, 120, 120, 120, 35, 121, 122, 123, 50, 32, 33, 125, 120, 120,
    34, 32, 120, 120, 126, 120, 120, 120, 33, 128, 120, 120, 120, 131, 130, 120, 35, 125, 120, 120,
    120, 120, 120, 128, 120, 120, 120, 120, 35, 125, 120, 120, 132, 125, 120, 120, 120, 120, 120,
    120, 120, 120, 120, 120, 132, 125, 125, 120, 120, 120, 125, 128, 120, 133, 120, 131, 131, 120,
    35, 125, 120, 120, 120, 120, 120, 128, 120, 120, 120, 120, 35, 128, 120, 120, 120, 131, 131,
    120, 35, 125, 120, 120, 120, 120, 120, 128, 120, 120, 120, 120, 35, 134, 135, 136, 136, 32, 33,
    125, 120, 120, 120, 32, 120, 120, 120, 120, 120, 120, 33, 135, 136, 136, 32, 33, 125, 120, 120,
    120, 32, 120, 120, 120, 120, 120, 120, 33, 136, 136, 32, 33, 125, 120, 120, 120, 32, 120, 120,
    120, 120, 120, 120, 33, 125, 120, 120, 132, 125, 120, 120, 120, 32, 120, 120, 120, 120, 120,
    120, 132, 121, 122, 124, 124, 32, 33, 125, 120, 120, 120, 32, 120, 120, 126, 120, 120, 120, 33,
    137, 138, 138, 32, 33, 125, 120, 120, 120, 32, 120, 120, 120, 120, 120, 120, 33, 129, 139, 120,
    35, 125, 120, 120, 120, 120, 120, 120, 120, 120, 120, 120, 35, 131, 131, 120, 35, 125, 120,
    120, 120, 120, 120, 120, 120, 120, 120, 120, 35, 129, 120, 131, 131, 120, 35, 125, 120, 120,
    120, 120, 120, 120, 120, 120, 120, 120, 35, 131, 139, 120, 35, 125, 120, 120, 120, 120, 120,
    120, 120, 120, 120, 120, 35, 43, 44, 45, 46, 108, 104, 22, 23, 49, 50, 50, 24, 22, 102, 43, 53,
    102, 102, 102, 23, 58, 140, 60, 61, 4, 5, 62, 57, 57, 8, 4, 57, 57, 63, 57, 57, 57, 5, 43, 44,
    45, 46, 141, 142, 22, 143, 144, 57, 50, 24, 22, 57, 43, 53, 57, 57, 57, 143, 20, 145, 145, 22,
    143, 62, 57, 57, 24, 22, 57, 57, 57, 57, 57, 57, 143, 62, 57, 57, 69, 62, 57, 57, 57, 22, 57,
    57, 57, 57, 57, 57, 69, 144, 57, 57, 146, 144, 57, 57, 57, 22, 57, 57, 57, 57, 57, 57, 146,
    144, 144, 57, 57, 57, 144, 43, 57, 70, 20, 145, 145, 22, 143, 62, 57, 57, 57, 22, 57, 43, 57,
    57, 57, 57, 143, 148, 147, 149, 149, 147, 40, 150, 147, 147, 147, 147, 147, 147, 147, 147, 147,
    147, 40, 149, 149, 147, 40, 150, 147, 147, 147, 147, 147, 147, 147, 147, 147, 147, 40, 150,
    147, 147, 151, 150, 147, 147, 147, 147, 147, 147, 147, 147, 147, 147, 151, 150, 150, 147, 147,
    147, 150, 43, 119, 119, 119, 119, 119, 119, 119, 119, 50, 119, 119, 119, 119, 43, 0, 0,
];
static _indic_syllable_machine_index_defaults: [i16; 140] = [
    0, 0, 0, 0, 0, 0, 0, 9, 9, 9, 9, 9, 9, 9, 9, 19, 19, 26, 19, 26, 19, 19, 29, 29, 29, 29, 29,
    29, 29, 0, 39, 42, 57, 57, 57, 57, 57, 57, 57, 57, 57, 57, 57, 57, 57, 57, 57, 57, 57, 57, 57,
    57, 57, 77, 77, 77, 77, 77, 77, 77, 77, 77, 77, 77, 77, 77, 77, 77, 77, 77, 77, 77, 77, 77, 77,
    77, 77, 77, 77, 102, 102, 102, 102, 102, 102, 102, 102, 102, 102, 102, 102, 102, 26, 102, 102,
    102, 102, 102, 102, 102, 119, 119, 120, 120, 120, 120, 120, 120, 120, 120, 120, 120, 120, 120,
    120, 120, 120, 120, 120, 120, 120, 120, 120, 102, 57, 57, 57, 57, 57, 57, 57, 57, 147, 147,
    147, 147, 147, 119, 0, 0,
];
static _indic_syllable_machine_cond_targs: [i16; 154] = [
    31, 37, 42, 2, 43, 46, 4, 50, 51, 31, 60, 9, 66, 69, 61, 11, 74, 75, 78, 31, 83, 17, 89, 92,
    93, 84, 31, 19, 98, 31, 107, 24, 113, 116, 117, 108, 26, 122, 127, 31, 134, 31, 31, 32, 53, 79,
    81, 100, 101, 85, 102, 123, 124, 94, 132, 137, 92, 31, 33, 35, 6, 52, 38, 47, 34, 1, 36, 40, 0,
    39, 41, 44, 45, 3, 48, 5, 49, 31, 54, 56, 14, 77, 62, 70, 55, 7, 57, 72, 64, 58, 13, 76, 59, 8,
    63, 65, 67, 68, 10, 71, 12, 73, 31, 80, 20, 82, 96, 87, 15, 99, 16, 86, 88, 90, 91, 18, 95, 21,
    97, 31, 31, 103, 105, 22, 27, 109, 118, 104, 106, 120, 111, 23, 110, 112, 114, 115, 25, 119,
    28, 121, 125, 126, 131, 128, 129, 29, 130, 31, 133, 30, 135, 136, 0, 0,
];
static _indic_syllable_machine_cond_actions: [i8; 154] = [
    1, 0, 2, 0, 2, 0, 0, 2, 2, 3, 2, 0, 2, 0, 0, 0, 2, 2, 2, 4, 2, 0, 5, 5, 5, 0, 6, 0, 2, 7, 2, 0,
    2, 0, 2, 0, 0, 2, 0, 8, 0, 0, 11, 2, 2, 5, 0, 12, 12, 0, 2, 5, 2, 5, 2, 0, 13, 14, 2, 0, 0, 2,
    0, 2, 2, 0, 2, 2, 0, 0, 2, 2, 2, 0, 0, 0, 2, 15, 2, 0, 0, 2, 0, 2, 2, 0, 2, 2, 2, 2, 0, 2, 2,
    0, 0, 2, 2, 2, 0, 0, 0, 2, 16, 5, 0, 5, 2, 2, 0, 5, 0, 0, 2, 5, 5, 0, 0, 0, 2, 17, 18, 2, 0, 0,
    0, 0, 2, 2, 2, 2, 2, 0, 0, 2, 2, 2, 0, 0, 0, 2, 0, 19, 19, 0, 0, 0, 0, 20, 2, 0, 0, 0, 0, 0,
];
static _indic_syllable_machine_to_state_actions: [i8; 140] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static _indic_syllable_machine_from_state_actions: [i8; 140] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
static _indic_syllable_machine_eof_trans: [i16; 140] = [
    1, 1, 1, 1, 1, 1, 1, 10, 10, 10, 10, 10, 10, 10, 10, 20, 20, 27, 20, 27, 20, 20, 30, 30, 30,
    30, 30, 30, 30, 1, 40, 42, 58, 58, 58, 58, 58, 58, 58, 58, 58, 58, 58, 58, 58, 58, 58, 58, 58,
    58, 58, 58, 58, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78, 78,
    78, 78, 78, 78, 78, 78, 103, 103, 103, 103, 103, 103, 103, 103, 103, 103, 103, 103, 103, 27,
    103, 103, 103, 103, 103, 103, 103, 120, 120, 121, 121, 121, 121, 121, 121, 121, 121, 121, 121,
    121, 121, 121, 121, 121, 121, 121, 121, 121, 121, 121, 103, 58, 58, 58, 58, 58, 58, 58, 58,
    148, 148, 148, 148, 148, 120, 0, 0,
];
static indic_syllable_machine_start: i32 = 31;
static indic_syllable_machine_first_final: i32 = 31;
static indic_syllable_machine_error: i32 = -1;
static indic_syllable_machine_en_main: i32 = 31;
#[derive(Clone, Copy)]
pub enum SyllableType {
    ConsonantSyllable = 0,
    VowelSyllable,
    StandaloneCluster,
    SymbolCluster,
    BrokenCluster,
    NonIndicCluster,
}

pub fn find_syllables_indic(buffer: &mut hb_buffer_t) {
    let mut cs = 0;
    let mut ts = 0;
    let mut te = 0;
    let mut act = 0;
    let mut p = 0;
    let pe = buffer.len;
    let eof = buffer.len;
    let mut syllable_serial = 1u8;

    macro_rules! found_syllable {
        ($kind:expr) => {{
            found_syllable(ts, te, &mut syllable_serial, $kind, buffer)
        }};
    }

    {
        cs = (indic_syllable_machine_start) as i32;
        ts = 0;
        te = 0;
        act = 0;
    }

    {
        let mut _trans = 0;
        let mut _keys: i32 = 0;
        let mut _inds: i32 = 0;
        let mut _ic = 0;
        '_resume: while (p != pe || p == eof) {
            '_again: while (true) {
                match (_indic_syllable_machine_from_state_actions[(cs) as usize]) {
                    10 => {
                        ts = p;
                    }

                    _ => {}
                }
                if (p == eof) {
                    {
                        if (_indic_syllable_machine_eof_trans[(cs) as usize] > 0) {
                            {
                                _trans =
                                    (_indic_syllable_machine_eof_trans[(cs) as usize]) as u32 - 1;
                            }
                        }
                    }
                } else {
                    {
                        _keys = (cs << 1) as i32;
                        _inds = (_indic_syllable_machine_index_offsets[(cs) as usize]) as i32;
                        if ((buffer.info[p].indic_category() as u8) <= 57
                            && (buffer.info[p].indic_category() as u8) >= 1)
                        {
                            {
                                _ic = (_indic_syllable_machine_char_class
                                    [((buffer.info[p].indic_category() as u8) as i32 - 1) as usize])
                                    as i32;
                                if (_ic
                                    <= (_indic_syllable_machine_trans_keys[(_keys + 1) as usize])
                                        as i32
                                    && _ic
                                        >= (_indic_syllable_machine_trans_keys[(_keys) as usize])
                                            as i32)
                                {
                                    _trans = (_indic_syllable_machine_indices[(_inds
                                        + (_ic
                                            - (_indic_syllable_machine_trans_keys[(_keys) as usize])
                                                as i32)
                                            as i32)
                                        as usize])
                                        as u32;
                                } else {
                                    _trans = (_indic_syllable_machine_index_defaults[(cs) as usize])
                                        as u32;
                                }
                            }
                        } else {
                            {
                                _trans =
                                    (_indic_syllable_machine_index_defaults[(cs) as usize]) as u32;
                            }
                        }
                    }
                }
                cs = (_indic_syllable_machine_cond_targs[(_trans) as usize]) as i32;
                if (_indic_syllable_machine_cond_actions[(_trans) as usize] != 0) {
                    {
                        match (_indic_syllable_machine_cond_actions[(_trans) as usize]) {
                            2 => {
                                te = p + 1;
                            }
                            11 => {
                                te = p + 1;
                                {
                                    found_syllable!(SyllableType::NonIndicCluster);
                                }
                            }
                            14 => {
                                te = p;
                                p = p - 1;
                                {
                                    found_syllable!(SyllableType::ConsonantSyllable);
                                }
                            }
                            15 => {
                                te = p;
                                p = p - 1;
                                {
                                    found_syllable!(SyllableType::VowelSyllable);
                                }
                            }
                            18 => {
                                te = p;
                                p = p - 1;
                                {
                                    found_syllable!(SyllableType::StandaloneCluster);
                                }
                            }
                            20 => {
                                te = p;
                                p = p - 1;
                                {
                                    found_syllable!(SyllableType::SymbolCluster);
                                }
                            }
                            16 => {
                                te = p;
                                p = p - 1;
                                {
                                    found_syllable!(SyllableType::BrokenCluster);
                                    buffer.scratch_flags |=
                                        HB_BUFFER_SCRATCH_FLAG_HAS_BROKEN_SYLLABLE;
                                }
                            }
                            17 => {
                                te = p;
                                p = p - 1;
                                {
                                    found_syllable!(SyllableType::NonIndicCluster);
                                }
                            }
                            1 => {
                                p = (te) - 1;
                                {
                                    found_syllable!(SyllableType::ConsonantSyllable);
                                }
                            }
                            3 => {
                                p = (te) - 1;
                                {
                                    found_syllable!(SyllableType::VowelSyllable);
                                }
                            }
                            7 => {
                                p = (te) - 1;
                                {
                                    found_syllable!(SyllableType::StandaloneCluster);
                                }
                            }
                            8 => {
                                p = (te) - 1;
                                {
                                    found_syllable!(SyllableType::SymbolCluster);
                                }
                            }
                            4 => {
                                p = (te) - 1;
                                {
                                    found_syllable!(SyllableType::BrokenCluster);
                                    buffer.scratch_flags |=
                                        HB_BUFFER_SCRATCH_FLAG_HAS_BROKEN_SYLLABLE;
                                }
                            }
                            6 => match (act) {
                                1 => {
                                    p = (te) - 1;
                                    {
                                        found_syllable!(SyllableType::ConsonantSyllable);
                                    }
                                }
                                5 => {
                                    p = (te) - 1;
                                    {
                                        found_syllable!(SyllableType::NonIndicCluster);
                                    }
                                }
                                6 => {
                                    p = (te) - 1;
                                    {
                                        found_syllable!(SyllableType::BrokenCluster);
                                        buffer.scratch_flags |=
                                            HB_BUFFER_SCRATCH_FLAG_HAS_BROKEN_SYLLABLE;
                                    }
                                }
                                7 => {
                                    p = (te) - 1;
                                    {
                                        found_syllable!(SyllableType::NonIndicCluster);
                                    }
                                }

                                _ => {}
                            },
                            19 => {
                                {
                                    {
                                        te = p + 1;
                                    }
                                }
                                {
                                    {
                                        act = 1;
                                    }
                                }
                            }
                            13 => {
                                {
                                    {
                                        te = p + 1;
                                    }
                                }
                                {
                                    {
                                        act = 5;
                                    }
                                }
                            }
                            5 => {
                                {
                                    {
                                        te = p + 1;
                                    }
                                }
                                {
                                    {
                                        act = 6;
                                    }
                                }
                            }
                            12 => {
                                {
                                    {
                                        te = p + 1;
                                    }
                                }
                                {
                                    {
                                        act = 7;
                                    }
                                }
                            }

                            _ => {}
                        }
                    }
                }
                break '_again;
            }
            if (p == eof) {
                {
                    if (cs >= 31) {
                        break '_resume;
                    }
                }
            } else {
                {
                    match (_indic_syllable_machine_to_state_actions[(cs) as usize]) {
                        9 => {
                            ts = 0;
                        }

                        _ => {}
                    }
                    p += 1;
                    continue '_resume;
                }
            }
            break '_resume;
        }
    }
}

#[inline]
fn found_syllable(
    start: usize,
    end: usize,
    syllable_serial: &mut u8,
    kind: SyllableType,
    buffer: &mut hb_buffer_t,
) {
    for i in start..end {
        buffer.info[i].set_syllable((*syllable_serial << 4) | kind as u8);
    }

    *syllable_serial += 1;

    if *syllable_serial == 16 {
        *syllable_serial = 1;
    }
}
