// Many statics that index one table (ts-rust diag.rs: `static X: &Message = &CATALOG[i]`).
// Each read takes one element in place; the table is not copied per static. Also the
// const table, repeat, byte string, and nested static forms of the same read.
#[derive(Clone, Copy, Debug)]
pub struct Message { code: u32, text: &'static str }
impl Message { pub const fn new(code: u32, text: &'static str) -> Self { Self { code, text } } }
pub static CATALOG: &[Message] = &[
    Message::new(0, "message text 0"),
    Message::new(1, "message text 1"),
    Message::new(2, "message text 2"),
    Message::new(3, "message text 3"),
    Message::new(4, "message text 4"),
    Message::new(5, "message text 5"),
    Message::new(6, "message text 6"),
    Message::new(7, "message text 7"),
    Message::new(8, "message text 8"),
    Message::new(9, "message text 9"),
    Message::new(10, "message text 10"),
    Message::new(11, "message text 11"),
    Message::new(12, "message text 12"),
    Message::new(13, "message text 13"),
    Message::new(14, "message text 14"),
    Message::new(15, "message text 15"),
    Message::new(16, "message text 16"),
    Message::new(17, "message text 17"),
    Message::new(18, "message text 18"),
    Message::new(19, "message text 19"),
    Message::new(20, "message text 20"),
    Message::new(21, "message text 21"),
    Message::new(22, "message text 22"),
    Message::new(23, "message text 23"),
    Message::new(24, "message text 24"),
    Message::new(25, "message text 25"),
    Message::new(26, "message text 26"),
    Message::new(27, "message text 27"),
    Message::new(28, "message text 28"),
    Message::new(29, "message text 29"),
    Message::new(30, "message text 30"),
    Message::new(31, "message text 31"),
    Message::new(32, "message text 32"),
    Message::new(33, "message text 33"),
    Message::new(34, "message text 34"),
    Message::new(35, "message text 35"),
    Message::new(36, "message text 36"),
    Message::new(37, "message text 37"),
    Message::new(38, "message text 38"),
    Message::new(39, "message text 39"),
    Message::new(40, "message text 40"),
    Message::new(41, "message text 41"),
    Message::new(42, "message text 42"),
    Message::new(43, "message text 43"),
    Message::new(44, "message text 44"),
    Message::new(45, "message text 45"),
    Message::new(46, "message text 46"),
    Message::new(47, "message text 47"),
    Message::new(48, "message text 48"),
    Message::new(49, "message text 49"),
    Message::new(50, "message text 50"),
    Message::new(51, "message text 51"),
    Message::new(52, "message text 52"),
    Message::new(53, "message text 53"),
    Message::new(54, "message text 54"),
    Message::new(55, "message text 55"),
    Message::new(56, "message text 56"),
    Message::new(57, "message text 57"),
    Message::new(58, "message text 58"),
    Message::new(59, "message text 59"),
    Message::new(60, "message text 60"),
    Message::new(61, "message text 61"),
    Message::new(62, "message text 62"),
    Message::new(63, "message text 63"),
    Message::new(64, "message text 64"),
    Message::new(65, "message text 65"),
    Message::new(66, "message text 66"),
    Message::new(67, "message text 67"),
    Message::new(68, "message text 68"),
    Message::new(69, "message text 69"),
    Message::new(70, "message text 70"),
    Message::new(71, "message text 71"),
    Message::new(72, "message text 72"),
    Message::new(73, "message text 73"),
    Message::new(74, "message text 74"),
    Message::new(75, "message text 75"),
    Message::new(76, "message text 76"),
    Message::new(77, "message text 77"),
    Message::new(78, "message text 78"),
    Message::new(79, "message text 79"),
    Message::new(80, "message text 80"),
    Message::new(81, "message text 81"),
    Message::new(82, "message text 82"),
    Message::new(83, "message text 83"),
    Message::new(84, "message text 84"),
    Message::new(85, "message text 85"),
    Message::new(86, "message text 86"),
    Message::new(87, "message text 87"),
    Message::new(88, "message text 88"),
    Message::new(89, "message text 89"),
    Message::new(90, "message text 90"),
    Message::new(91, "message text 91"),
    Message::new(92, "message text 92"),
    Message::new(93, "message text 93"),
    Message::new(94, "message text 94"),
    Message::new(95, "message text 95"),
    Message::new(96, "message text 96"),
    Message::new(97, "message text 97"),
    Message::new(98, "message text 98"),
    Message::new(99, "message text 99"),
    Message::new(100, "message text 100"),
    Message::new(101, "message text 101"),
    Message::new(102, "message text 102"),
    Message::new(103, "message text 103"),
    Message::new(104, "message text 104"),
    Message::new(105, "message text 105"),
    Message::new(106, "message text 106"),
    Message::new(107, "message text 107"),
    Message::new(108, "message text 108"),
    Message::new(109, "message text 109"),
    Message::new(110, "message text 110"),
    Message::new(111, "message text 111"),
    Message::new(112, "message text 112"),
    Message::new(113, "message text 113"),
    Message::new(114, "message text 114"),
    Message::new(115, "message text 115"),
    Message::new(116, "message text 116"),
    Message::new(117, "message text 117"),
    Message::new(118, "message text 118"),
    Message::new(119, "message text 119"),
    Message::new(120, "message text 120"),
    Message::new(121, "message text 121"),
    Message::new(122, "message text 122"),
    Message::new(123, "message text 123"),
    Message::new(124, "message text 124"),
    Message::new(125, "message text 125"),
    Message::new(126, "message text 126"),
    Message::new(127, "message text 127"),
    Message::new(128, "message text 128"),
    Message::new(129, "message text 129"),
    Message::new(130, "message text 130"),
    Message::new(131, "message text 131"),
    Message::new(132, "message text 132"),
    Message::new(133, "message text 133"),
    Message::new(134, "message text 134"),
    Message::new(135, "message text 135"),
    Message::new(136, "message text 136"),
    Message::new(137, "message text 137"),
    Message::new(138, "message text 138"),
    Message::new(139, "message text 139"),
    Message::new(140, "message text 140"),
    Message::new(141, "message text 141"),
    Message::new(142, "message text 142"),
    Message::new(143, "message text 143"),
    Message::new(144, "message text 144"),
    Message::new(145, "message text 145"),
    Message::new(146, "message text 146"),
    Message::new(147, "message text 147"),
    Message::new(148, "message text 148"),
    Message::new(149, "message text 149"),
    Message::new(150, "message text 150"),
    Message::new(151, "message text 151"),
    Message::new(152, "message text 152"),
    Message::new(153, "message text 153"),
    Message::new(154, "message text 154"),
    Message::new(155, "message text 155"),
    Message::new(156, "message text 156"),
    Message::new(157, "message text 157"),
    Message::new(158, "message text 158"),
    Message::new(159, "message text 159"),
    Message::new(160, "message text 160"),
    Message::new(161, "message text 161"),
    Message::new(162, "message text 162"),
    Message::new(163, "message text 163"),
    Message::new(164, "message text 164"),
    Message::new(165, "message text 165"),
    Message::new(166, "message text 166"),
    Message::new(167, "message text 167"),
    Message::new(168, "message text 168"),
    Message::new(169, "message text 169"),
    Message::new(170, "message text 170"),
    Message::new(171, "message text 171"),
    Message::new(172, "message text 172"),
    Message::new(173, "message text 173"),
    Message::new(174, "message text 174"),
    Message::new(175, "message text 175"),
    Message::new(176, "message text 176"),
    Message::new(177, "message text 177"),
    Message::new(178, "message text 178"),
    Message::new(179, "message text 179"),
    Message::new(180, "message text 180"),
    Message::new(181, "message text 181"),
    Message::new(182, "message text 182"),
    Message::new(183, "message text 183"),
    Message::new(184, "message text 184"),
    Message::new(185, "message text 185"),
    Message::new(186, "message text 186"),
    Message::new(187, "message text 187"),
    Message::new(188, "message text 188"),
    Message::new(189, "message text 189"),
    Message::new(190, "message text 190"),
    Message::new(191, "message text 191"),
    Message::new(192, "message text 192"),
    Message::new(193, "message text 193"),
    Message::new(194, "message text 194"),
    Message::new(195, "message text 195"),
    Message::new(196, "message text 196"),
    Message::new(197, "message text 197"),
    Message::new(198, "message text 198"),
    Message::new(199, "message text 199"),
    Message::new(200, "message text 200"),
    Message::new(201, "message text 201"),
    Message::new(202, "message text 202"),
    Message::new(203, "message text 203"),
    Message::new(204, "message text 204"),
    Message::new(205, "message text 205"),
    Message::new(206, "message text 206"),
    Message::new(207, "message text 207"),
    Message::new(208, "message text 208"),
    Message::new(209, "message text 209"),
    Message::new(210, "message text 210"),
    Message::new(211, "message text 211"),
    Message::new(212, "message text 212"),
    Message::new(213, "message text 213"),
    Message::new(214, "message text 214"),
    Message::new(215, "message text 215"),
    Message::new(216, "message text 216"),
    Message::new(217, "message text 217"),
    Message::new(218, "message text 218"),
    Message::new(219, "message text 219"),
    Message::new(220, "message text 220"),
    Message::new(221, "message text 221"),
    Message::new(222, "message text 222"),
    Message::new(223, "message text 223"),
    Message::new(224, "message text 224"),
    Message::new(225, "message text 225"),
    Message::new(226, "message text 226"),
    Message::new(227, "message text 227"),
    Message::new(228, "message text 228"),
    Message::new(229, "message text 229"),
    Message::new(230, "message text 230"),
    Message::new(231, "message text 231"),
    Message::new(232, "message text 232"),
    Message::new(233, "message text 233"),
    Message::new(234, "message text 234"),
    Message::new(235, "message text 235"),
    Message::new(236, "message text 236"),
    Message::new(237, "message text 237"),
    Message::new(238, "message text 238"),
    Message::new(239, "message text 239"),
    Message::new(240, "message text 240"),
    Message::new(241, "message text 241"),
    Message::new(242, "message text 242"),
    Message::new(243, "message text 243"),
    Message::new(244, "message text 244"),
    Message::new(245, "message text 245"),
    Message::new(246, "message text 246"),
    Message::new(247, "message text 247"),
    Message::new(248, "message text 248"),
    Message::new(249, "message text 249"),
    Message::new(250, "message text 250"),
    Message::new(251, "message text 251"),
    Message::new(252, "message text 252"),
    Message::new(253, "message text 253"),
    Message::new(254, "message text 254"),
    Message::new(255, "message text 255"),
    Message::new(256, "message text 256"),
    Message::new(257, "message text 257"),
    Message::new(258, "message text 258"),
    Message::new(259, "message text 259"),
    Message::new(260, "message text 260"),
    Message::new(261, "message text 261"),
    Message::new(262, "message text 262"),
    Message::new(263, "message text 263"),
    Message::new(264, "message text 264"),
    Message::new(265, "message text 265"),
    Message::new(266, "message text 266"),
    Message::new(267, "message text 267"),
    Message::new(268, "message text 268"),
    Message::new(269, "message text 269"),
    Message::new(270, "message text 270"),
    Message::new(271, "message text 271"),
    Message::new(272, "message text 272"),
    Message::new(273, "message text 273"),
    Message::new(274, "message text 274"),
    Message::new(275, "message text 275"),
    Message::new(276, "message text 276"),
    Message::new(277, "message text 277"),
    Message::new(278, "message text 278"),
    Message::new(279, "message text 279"),
    Message::new(280, "message text 280"),
    Message::new(281, "message text 281"),
    Message::new(282, "message text 282"),
    Message::new(283, "message text 283"),
    Message::new(284, "message text 284"),
    Message::new(285, "message text 285"),
    Message::new(286, "message text 286"),
    Message::new(287, "message text 287"),
    Message::new(288, "message text 288"),
    Message::new(289, "message text 289"),
    Message::new(290, "message text 290"),
    Message::new(291, "message text 291"),
    Message::new(292, "message text 292"),
    Message::new(293, "message text 293"),
    Message::new(294, "message text 294"),
    Message::new(295, "message text 295"),
    Message::new(296, "message text 296"),
    Message::new(297, "message text 297"),
    Message::new(298, "message text 298"),
    Message::new(299, "message text 299"),
    Message::new(300, "message text 300"),
    Message::new(301, "message text 301"),
    Message::new(302, "message text 302"),
    Message::new(303, "message text 303"),
    Message::new(304, "message text 304"),
    Message::new(305, "message text 305"),
    Message::new(306, "message text 306"),
    Message::new(307, "message text 307"),
    Message::new(308, "message text 308"),
    Message::new(309, "message text 309"),
    Message::new(310, "message text 310"),
    Message::new(311, "message text 311"),
    Message::new(312, "message text 312"),
    Message::new(313, "message text 313"),
    Message::new(314, "message text 314"),
    Message::new(315, "message text 315"),
    Message::new(316, "message text 316"),
    Message::new(317, "message text 317"),
    Message::new(318, "message text 318"),
    Message::new(319, "message text 319"),
    Message::new(320, "message text 320"),
    Message::new(321, "message text 321"),
    Message::new(322, "message text 322"),
    Message::new(323, "message text 323"),
    Message::new(324, "message text 324"),
    Message::new(325, "message text 325"),
    Message::new(326, "message text 326"),
    Message::new(327, "message text 327"),
    Message::new(328, "message text 328"),
    Message::new(329, "message text 329"),
    Message::new(330, "message text 330"),
    Message::new(331, "message text 331"),
    Message::new(332, "message text 332"),
    Message::new(333, "message text 333"),
    Message::new(334, "message text 334"),
    Message::new(335, "message text 335"),
    Message::new(336, "message text 336"),
    Message::new(337, "message text 337"),
    Message::new(338, "message text 338"),
    Message::new(339, "message text 339"),
    Message::new(340, "message text 340"),
    Message::new(341, "message text 341"),
    Message::new(342, "message text 342"),
    Message::new(343, "message text 343"),
    Message::new(344, "message text 344"),
    Message::new(345, "message text 345"),
    Message::new(346, "message text 346"),
    Message::new(347, "message text 347"),
    Message::new(348, "message text 348"),
    Message::new(349, "message text 349"),
    Message::new(350, "message text 350"),
    Message::new(351, "message text 351"),
    Message::new(352, "message text 352"),
    Message::new(353, "message text 353"),
    Message::new(354, "message text 354"),
    Message::new(355, "message text 355"),
    Message::new(356, "message text 356"),
    Message::new(357, "message text 357"),
    Message::new(358, "message text 358"),
    Message::new(359, "message text 359"),
    Message::new(360, "message text 360"),
    Message::new(361, "message text 361"),
    Message::new(362, "message text 362"),
    Message::new(363, "message text 363"),
    Message::new(364, "message text 364"),
    Message::new(365, "message text 365"),
    Message::new(366, "message text 366"),
    Message::new(367, "message text 367"),
    Message::new(368, "message text 368"),
    Message::new(369, "message text 369"),
    Message::new(370, "message text 370"),
    Message::new(371, "message text 371"),
    Message::new(372, "message text 372"),
    Message::new(373, "message text 373"),
    Message::new(374, "message text 374"),
    Message::new(375, "message text 375"),
    Message::new(376, "message text 376"),
    Message::new(377, "message text 377"),
    Message::new(378, "message text 378"),
    Message::new(379, "message text 379"),
    Message::new(380, "message text 380"),
    Message::new(381, "message text 381"),
    Message::new(382, "message text 382"),
    Message::new(383, "message text 383"),
    Message::new(384, "message text 384"),
    Message::new(385, "message text 385"),
    Message::new(386, "message text 386"),
    Message::new(387, "message text 387"),
    Message::new(388, "message text 388"),
    Message::new(389, "message text 389"),
    Message::new(390, "message text 390"),
    Message::new(391, "message text 391"),
    Message::new(392, "message text 392"),
    Message::new(393, "message text 393"),
    Message::new(394, "message text 394"),
    Message::new(395, "message text 395"),
    Message::new(396, "message text 396"),
    Message::new(397, "message text 397"),
    Message::new(398, "message text 398"),
    Message::new(399, "message text 399"),
];
pub static M_0: &Message = &CATALOG[0];
pub static M_2: &Message = &CATALOG[2];
pub static M_4: &Message = &CATALOG[4];
pub static M_6: &Message = &CATALOG[6];
pub static M_8: &Message = &CATALOG[8];
pub static M_10: &Message = &CATALOG[10];
pub static M_12: &Message = &CATALOG[12];
pub static M_14: &Message = &CATALOG[14];
pub static M_16: &Message = &CATALOG[16];
pub static M_18: &Message = &CATALOG[18];
pub static M_20: &Message = &CATALOG[20];
pub static M_22: &Message = &CATALOG[22];
pub static M_24: &Message = &CATALOG[24];
pub static M_26: &Message = &CATALOG[26];
pub static M_28: &Message = &CATALOG[28];
pub static M_30: &Message = &CATALOG[30];
pub static M_32: &Message = &CATALOG[32];
pub static M_34: &Message = &CATALOG[34];
pub static M_36: &Message = &CATALOG[36];
pub static M_38: &Message = &CATALOG[38];
pub static M_40: &Message = &CATALOG[40];
pub static M_42: &Message = &CATALOG[42];
pub static M_44: &Message = &CATALOG[44];
pub static M_46: &Message = &CATALOG[46];
pub static M_48: &Message = &CATALOG[48];
pub static M_50: &Message = &CATALOG[50];
pub static M_52: &Message = &CATALOG[52];
pub static M_54: &Message = &CATALOG[54];
pub static M_56: &Message = &CATALOG[56];
pub static M_58: &Message = &CATALOG[58];
pub static M_60: &Message = &CATALOG[60];
pub static M_62: &Message = &CATALOG[62];
pub static M_64: &Message = &CATALOG[64];
pub static M_66: &Message = &CATALOG[66];
pub static M_68: &Message = &CATALOG[68];
pub static M_70: &Message = &CATALOG[70];
pub static M_72: &Message = &CATALOG[72];
pub static M_74: &Message = &CATALOG[74];
pub static M_76: &Message = &CATALOG[76];
pub static M_78: &Message = &CATALOG[78];
pub static M_80: &Message = &CATALOG[80];
pub static M_82: &Message = &CATALOG[82];
pub static M_84: &Message = &CATALOG[84];
pub static M_86: &Message = &CATALOG[86];
pub static M_88: &Message = &CATALOG[88];
pub static M_90: &Message = &CATALOG[90];
pub static M_92: &Message = &CATALOG[92];
pub static M_94: &Message = &CATALOG[94];
pub static M_96: &Message = &CATALOG[96];
pub static M_98: &Message = &CATALOG[98];
pub static M_100: &Message = &CATALOG[100];
pub static M_102: &Message = &CATALOG[102];
pub static M_104: &Message = &CATALOG[104];
pub static M_106: &Message = &CATALOG[106];
pub static M_108: &Message = &CATALOG[108];
pub static M_110: &Message = &CATALOG[110];
pub static M_112: &Message = &CATALOG[112];
pub static M_114: &Message = &CATALOG[114];
pub static M_116: &Message = &CATALOG[116];
pub static M_118: &Message = &CATALOG[118];
pub static M_120: &Message = &CATALOG[120];
pub static M_122: &Message = &CATALOG[122];
pub static M_124: &Message = &CATALOG[124];
pub static M_126: &Message = &CATALOG[126];
pub static M_128: &Message = &CATALOG[128];
pub static M_130: &Message = &CATALOG[130];
pub static M_132: &Message = &CATALOG[132];
pub static M_134: &Message = &CATALOG[134];
pub static M_136: &Message = &CATALOG[136];
pub static M_138: &Message = &CATALOG[138];
pub static M_140: &Message = &CATALOG[140];
pub static M_142: &Message = &CATALOG[142];
pub static M_144: &Message = &CATALOG[144];
pub static M_146: &Message = &CATALOG[146];
pub static M_148: &Message = &CATALOG[148];
pub static M_150: &Message = &CATALOG[150];
pub static M_152: &Message = &CATALOG[152];
pub static M_154: &Message = &CATALOG[154];
pub static M_156: &Message = &CATALOG[156];
pub static M_158: &Message = &CATALOG[158];
pub static M_160: &Message = &CATALOG[160];
pub static M_162: &Message = &CATALOG[162];
pub static M_164: &Message = &CATALOG[164];
pub static M_166: &Message = &CATALOG[166];
pub static M_168: &Message = &CATALOG[168];
pub static M_170: &Message = &CATALOG[170];
pub static M_172: &Message = &CATALOG[172];
pub static M_174: &Message = &CATALOG[174];
pub static M_176: &Message = &CATALOG[176];
pub static M_178: &Message = &CATALOG[178];
pub static M_180: &Message = &CATALOG[180];
pub static M_182: &Message = &CATALOG[182];
pub static M_184: &Message = &CATALOG[184];
pub static M_186: &Message = &CATALOG[186];
pub static M_188: &Message = &CATALOG[188];
pub static M_190: &Message = &CATALOG[190];
pub static M_192: &Message = &CATALOG[192];
pub static M_194: &Message = &CATALOG[194];
pub static M_196: &Message = &CATALOG[196];
pub static M_198: &Message = &CATALOG[198];
pub static M_200: &Message = &CATALOG[200];
pub static M_202: &Message = &CATALOG[202];
pub static M_204: &Message = &CATALOG[204];
pub static M_206: &Message = &CATALOG[206];
pub static M_208: &Message = &CATALOG[208];
pub static M_210: &Message = &CATALOG[210];
pub static M_212: &Message = &CATALOG[212];
pub static M_214: &Message = &CATALOG[214];
pub static M_216: &Message = &CATALOG[216];
pub static M_218: &Message = &CATALOG[218];
pub static M_220: &Message = &CATALOG[220];
pub static M_222: &Message = &CATALOG[222];
pub static M_224: &Message = &CATALOG[224];
pub static M_226: &Message = &CATALOG[226];
pub static M_228: &Message = &CATALOG[228];
pub static M_230: &Message = &CATALOG[230];
pub static M_232: &Message = &CATALOG[232];
pub static M_234: &Message = &CATALOG[234];
pub static M_236: &Message = &CATALOG[236];
pub static M_238: &Message = &CATALOG[238];
pub static M_240: &Message = &CATALOG[240];
pub static M_242: &Message = &CATALOG[242];
pub static M_244: &Message = &CATALOG[244];
pub static M_246: &Message = &CATALOG[246];
pub static M_248: &Message = &CATALOG[248];
pub static M_250: &Message = &CATALOG[250];
pub static M_252: &Message = &CATALOG[252];
pub static M_254: &Message = &CATALOG[254];
pub static M_256: &Message = &CATALOG[256];
pub static M_258: &Message = &CATALOG[258];
pub static M_260: &Message = &CATALOG[260];
pub static M_262: &Message = &CATALOG[262];
pub static M_264: &Message = &CATALOG[264];
pub static M_266: &Message = &CATALOG[266];
pub static M_268: &Message = &CATALOG[268];
pub static M_270: &Message = &CATALOG[270];
pub static M_272: &Message = &CATALOG[272];
pub static M_274: &Message = &CATALOG[274];
pub static M_276: &Message = &CATALOG[276];
pub static M_278: &Message = &CATALOG[278];
pub static M_280: &Message = &CATALOG[280];
pub static M_282: &Message = &CATALOG[282];
pub static M_284: &Message = &CATALOG[284];
pub static M_286: &Message = &CATALOG[286];
pub static M_288: &Message = &CATALOG[288];
pub static M_290: &Message = &CATALOG[290];
pub static M_292: &Message = &CATALOG[292];
pub static M_294: &Message = &CATALOG[294];
pub static M_296: &Message = &CATALOG[296];
pub static M_298: &Message = &CATALOG[298];
pub static M_300: &Message = &CATALOG[300];
pub static M_302: &Message = &CATALOG[302];
pub static M_304: &Message = &CATALOG[304];
pub static M_306: &Message = &CATALOG[306];
pub static M_308: &Message = &CATALOG[308];
pub static M_310: &Message = &CATALOG[310];
pub static M_312: &Message = &CATALOG[312];
pub static M_314: &Message = &CATALOG[314];
pub static M_316: &Message = &CATALOG[316];
pub static M_318: &Message = &CATALOG[318];
pub static M_320: &Message = &CATALOG[320];
pub static M_322: &Message = &CATALOG[322];
pub static M_324: &Message = &CATALOG[324];
pub static M_326: &Message = &CATALOG[326];
pub static M_328: &Message = &CATALOG[328];
pub static M_330: &Message = &CATALOG[330];
pub static M_332: &Message = &CATALOG[332];
pub static M_334: &Message = &CATALOG[334];
pub static M_336: &Message = &CATALOG[336];
pub static M_338: &Message = &CATALOG[338];
pub static M_340: &Message = &CATALOG[340];
pub static M_342: &Message = &CATALOG[342];
pub static M_344: &Message = &CATALOG[344];
pub static M_346: &Message = &CATALOG[346];
pub static M_348: &Message = &CATALOG[348];
pub static M_350: &Message = &CATALOG[350];
pub static M_352: &Message = &CATALOG[352];
pub static M_354: &Message = &CATALOG[354];
pub static M_356: &Message = &CATALOG[356];
pub static M_358: &Message = &CATALOG[358];
pub static M_360: &Message = &CATALOG[360];
pub static M_362: &Message = &CATALOG[362];
pub static M_364: &Message = &CATALOG[364];
pub static M_366: &Message = &CATALOG[366];
pub static M_368: &Message = &CATALOG[368];
pub static M_370: &Message = &CATALOG[370];
pub static M_372: &Message = &CATALOG[372];
pub static M_374: &Message = &CATALOG[374];
pub static M_376: &Message = &CATALOG[376];
pub static M_378: &Message = &CATALOG[378];
pub static M_380: &Message = &CATALOG[380];
pub static M_382: &Message = &CATALOG[382];
pub static M_384: &Message = &CATALOG[384];
pub static M_386: &Message = &CATALOG[386];
pub static M_388: &Message = &CATALOG[388];
pub static M_390: &Message = &CATALOG[390];
pub static M_392: &Message = &CATALOG[392];
pub static M_394: &Message = &CATALOG[394];
pub static M_396: &Message = &CATALOG[396];
pub static M_398: &Message = &CATALOG[398];
const TABLE: [u32; 5] = [10, 20, 30, 40, 50];
static T3: u32 = TABLE[3];
const REP: [u16; 8] = [7; 8];
static R5: u16 = REP[5];
const BYTES: &[u8] = b"lust";
static B2: u8 = BYTES[2];
static ARR: [i64; 3] = [-1, -2, -3];
static A1: i64 = ARR[1];

fn main() {
    let block_item = {
        static INNER: [u8; 3] = [4, 5, 6];
        static PICK: u8 = INNER[2];
        PICK
    };
    println!("{} {:?} {:?} {}", CATALOG.len(), M_0, M_398, M_200.code + M_200.text.len() as u32);
    println!("{} {} {} {} {}", T3, R5, B2, A1, block_item);
}
