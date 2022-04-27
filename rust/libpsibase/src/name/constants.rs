pub const CODE_VALUE_BITS: u32 = ((u32::BITS + 3) / 2) as u32;

pub const MAX_CODE: u32 = (1_u32 << CODE_VALUE_BITS) - 1;

pub const ONE_FOURTH: u32 = 1_u32 << (CODE_VALUE_BITS - 2);

pub const ONE_HALF: u32 = 2 * ONE_FOURTH;

pub const THREE_FOURTHS: u32 = 3 * ONE_FOURTH;

pub const MODEL_WIDTH: usize = 38;

pub const SYMBOL_TO_CHAR: [u8; MODEL_WIDTH] = [
    0, b'e', b'a', b'o', b'r', b'i', b'n', b't', b's', b'l', b'd', b'h', b'm', b'c', b'u', b'y',
    b'g', b'b', b'p', b'k', b'w', b'-', b'f', b'1', b'2', b'v', b'0', b'j', b'3', b'z', b'9', b'x',
    b'4', b'7', b'8', b'5', b'6', b'q',
];

pub const CHAR_TO_SYMBOL: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 21, 0, 0, 26, 23, 24, 28, 32, 35, 36, 33, 34, 30, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 2, 17, 13, 10, 1, 22, 16, 11, 5, 27, 19, 9, 12, 6, 3, 18, 37, 4, 8, 7, 14, 25,
    20, 31, 15, 29, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub const MODEL_CF: [[u16; 39]; MODEL_WIDTH] = [
    [
        0, 1, 862, 2908, 3513, 4989, 6103, 7218, 9965, 13074, 14397, 16325, 17420, 19736, 21713,
        22054, 22398, 23522, 25457, 26970, 28069, 28930, 28930, 30084, 30084, 30084, 30545, 30545,
        31987, 31987, 32323, 32350, 32620, 32620, 32620, 32620, 32620, 32620, 32767,
    ],
    [
        0, 2970, 4543, 6487, 6925, 12439, 12956, 15504, 16967, 19523, 21497, 23208, 23513, 24517,
        25327, 25530, 26230, 26817, 27435, 28069, 28429, 28969, 29806, 30287, 30557, 30715, 31284,
        31358, 31508, 31585, 31835, 31919, 32396, 32477, 32538, 32607, 32656, 32711, 32767,
    ],
    [
        0, 1374, 1626, 1999, 2143, 5560, 6497, 11473, 13950, 15929, 18544, 19981, 20449, 22327,
        23821, 24325, 25836, 26671, 27478, 28292, 29068, 30170, 30499, 30848, 30992, 31070, 31636,
        31675, 31851, 31888, 32300, 32344, 32560, 32594, 32627, 32662, 32689, 32715, 32767,
    ],
    [
        0, 1564, 1978, 2446, 4579, 7810, 8289, 12895, 14707, 16261, 18122, 19108, 19564, 21212,
        22096, 23951, 24391, 25080, 26020, 27000, 27679, 29772, 30101, 30949, 31111, 31200, 31798,
        31847, 31986, 32032, 32197, 32249, 32561, 32601, 32639, 32680, 32710, 32741, 32767,
    ],
    [
        0, 2907, 7236, 10894, 14971, 15898, 18953, 19736, 21142, 22320, 23011, 24214, 24427, 25092,
        25644, 26675, 27607, 28157, 28642, 29089, 29845, 30036, 30647, 30954, 31278, 31453, 31642,
        31788, 31911, 32031, 32150, 32252, 32330, 32434, 32506, 32595, 32656, 32731, 32767,
    ],
    [
        0, 1057, 2826, 4202, 5416, 6850, 7227, 13303, 16130, 19092, 21221, 22456, 22606, 23952,
        26558, 26769, 26850, 28177, 28572, 29178, 29933, 30053, 30340, 30895, 30985, 31035, 31613,
        31649, 31764, 31798, 32248, 32280, 32578, 32603, 32625, 32652, 32671, 32690, 32767,
    ],
    [
        0, 3420, 6470, 8938, 11134, 11400, 13626, 14759, 17307, 18772, 19106, 21674, 21926, 22277,
        23313, 23821, 24535, 27595, 27993, 28230, 29224, 29399, 30084, 30446, 30848, 31070, 31232,
        31375, 31666, 31792, 32024, 32151, 32263, 32371, 32463, 32569, 32642, 32722, 32767,
    ],
    [
        0, 2631, 6465, 9246, 12000, 13826, 16438, 16663, 18266, 19589, 20155, 20388, 26137, 26538,
        27122, 27990, 29067, 29281, 29609, 29826, 29964, 30358, 31071, 31344, 31580, 31715, 31812,
        31904, 32013, 32101, 32328, 32395, 32468, 32549, 32597, 32653, 32698, 32747, 32767,
    ],
    [
        0, 4601, 7052, 9239, 10945, 11204, 12968, 13469, 18236, 20065, 20745, 21180, 23716, 24545,
        25548, 26644, 27068, 27332, 27668, 28711, 29443, 29924, 30683, 31005, 31352, 31550, 31642,
        31744, 31883, 31980, 32044, 32149, 32211, 32321, 32395, 32480, 32536, 32602, 32767,
    ],
    [
        0, 1854, 7276, 11228, 14640, 14858, 18704, 18852, 19618, 20487, 24125, 25101, 25257, 25715,
        26035, 27365, 28845, 29073, 29425, 29868, 30266, 30421, 30911, 31393, 31584, 31691, 31946,
        32040, 32146, 32233, 32400, 32460, 32517, 32579, 32621, 32674, 32708, 32745, 32767,
    ],
    [
        0, 3064, 7929, 11787, 15109, 16895, 20069, 20413, 20870, 22044, 22641, 24010, 24378, 24930,
        25323, 26536, 27562, 28098, 28579, 28890, 29127, 29470, 30289, 30805, 31107, 31284, 31495,
        31618, 31921, 32045, 32185, 32278, 32373, 32476, 32543, 32619, 32674, 32736, 32767,
    ],
    [
        0, 1963, 9365, 14894, 18210, 20909, 24587, 25164, 26225, 26600, 26914, 27189, 27529, 27941,
        28123, 29379, 30071, 30259, 30532, 30691, 30849, 31027, 31400, 31577, 31791, 31911, 31974,
        32076, 32204, 32309, 32354, 32417, 32473, 32552, 32599, 32663, 32703, 32742, 32767,
    ],
    [
        0, 1528, 6875, 14214, 17481, 18268, 21728, 22062, 22396, 23176, 23398, 23672, 23822, 24981,
        25645, 26757, 28009, 28230, 29351, 30529, 30683, 30806, 31385, 31561, 31754, 31857, 31926,
        32025, 32175, 32268, 32367, 32425, 32488, 32565, 32606, 32675, 32710, 32745, 32767,
    ],
    [
        0, 1181, 3832, 7968, 12348, 14055, 15358, 15503, 16705, 17174, 18245, 18459, 24081, 24331,
        25091, 26139, 26567, 26744, 26944, 27140, 31302, 31388, 31666, 31808, 31954, 32044, 32123,
        32210, 32312, 32372, 32452, 32498, 32554, 32606, 32640, 32678, 32706, 32733, 32767,
    ],
    [
        0, 1067, 2429, 3271, 3443, 7106, 8245, 12093, 14270, 18945, 20816, 22038, 22323, 24071,
        25799, 26074, 26848, 27851, 28920, 30174, 30771, 30879, 31059, 31579, 31668, 31726, 31882,
        31908, 32022, 32056, 32356, 32384, 32600, 32629, 32654, 32681, 32715, 32735, 32767,
    ],
    [
        0, 6050, 7445, 9167, 11306, 12106, 12666, 13745, 14916, 16756, 17885, 18822, 19274, 20361,
        21338, 21859, 22497, 23084, 24271, 25221, 25620, 26091, 27776, 28424, 29375, 29913, 30074,
        30333, 30647, 30910, 31143, 31443, 31615, 31884, 32089, 32321, 32506, 32705, 32767,
    ],
    [
        0, 2679, 6956, 10400, 13634, 15952, 18416, 19038, 19621, 20629, 21786, 22182, 24335, 24789,
        25050, 27247, 27683, 29110, 29571, 29820, 29966, 30239, 30879, 31254, 31528, 31684, 31768,
        31882, 32031, 32138, 32258, 32339, 32395, 32498, 32558, 32628, 32681, 32739, 32767,
    ],
    [
        0, 1114, 5932, 10662, 15286, 18135, 21137, 21279, 21537, 22258, 25401, 25644, 25853, 26058,
        26298, 28722, 29632, 29765, 30895, 30997, 31108, 31215, 31414, 31546, 31734, 31834, 31943,
        32064, 32212, 32299, 32409, 32460, 32502, 32574, 32613, 32662, 32698, 32732, 32767,
    ],
    [
        0, 1746, 6320, 10438, 13833, 16087, 19082, 19259, 20327, 21545, 23659, 23900, 25815, 26373,
        26659, 27901, 28677, 28818, 29032, 30766, 30939, 31102, 31432, 31627, 31832, 31944, 32057,
        32155, 32243, 32338, 32419, 32474, 32512, 32583, 32622, 32667, 32704, 32741, 32767,
    ],
    [
        0, 3312, 9511, 12974, 14670, 15553, 20214, 21091, 21640, 23232, 24116, 24462, 24973, 25465,
        25782, 26721, 28129, 28318, 28692, 28937, 29497, 29814, 30463, 30743, 31132, 31339, 31430,
        31557, 31813, 31969, 32094, 32214, 32278, 32397, 32482, 32592, 32661, 32733, 32767,
    ],
    [
        0, 1760, 4960, 15877, 18514, 19158, 22852, 24093, 24586, 25371, 25814, 26141, 28194, 28536,
        28715, 29018, 29373, 29619, 30002, 30189, 30533, 30927, 31341, 31538, 31762, 31904, 31955,
        32054, 32149, 32239, 32319, 32382, 32428, 32524, 32569, 32627, 32671, 32712, 32767,
    ],
    [
        0, 1962, 2657, 4725, 5743, 6965, 8078, 9093, 11298, 13716, 14863, 16412, 17403, 19661,
        21279, 21751, 22219, 23348, 25242, 26704, 27396, 28319, 28319, 29530, 30017, 30319, 30663,
        30850, 31428, 31560, 31727, 31894, 32078, 32241, 32342, 32464, 32544, 32649, 32767,
    ],
    [
        0, 1619, 4706, 8870, 11995, 14927, 18474, 18670, 19986, 20581, 22796, 23232, 23499, 23821,
        24170, 26674, 27092, 27503, 27768, 27974, 28180, 28456, 29023, 31461, 31661, 31781, 31862,
        31943, 32160, 32248, 32300, 32356, 32464, 32546, 32595, 32649, 32692, 32735, 32767,
    ],
    [
        0, 8697, 8777, 8908, 8970, 9047, 9096, 9293, 9425, 9595, 9680, 9774, 9811, 9905, 10027,
        10064, 10092, 10174, 10242, 10305, 10378, 10414, 10514, 10572, 13817, 19018, 19054, 21820,
        21848, 23999, 24036, 26746, 26801, 28130, 29410, 30455, 31691, 32743, 32767,
    ],
    [
        0, 8819, 8884, 8978, 9055, 9132, 9181, 9281, 9418, 9555, 9630, 9767, 9834, 9945, 10053,
        10109, 10149, 10235, 10377, 10473, 10682, 10746, 10874, 10961, 13794, 16344, 16372, 20413,
        20458, 25815, 25831, 26762, 26824, 28561, 29652, 30627, 31892, 32740, 32767,
    ],
    [
        0, 1447, 13594, 18104, 20353, 20786, 27618, 27847, 28061, 28506, 28841, 29067, 29185,
        29364, 29580, 29875, 30355, 30517, 30716, 30908, 31027, 31106, 31274, 31390, 31587, 31728,
        32011, 32092, 32174, 32313, 32386, 32434, 32510, 32576, 32616, 32673, 32711, 32747, 32767,
    ],
    [
        0, 8241, 8310, 8419, 8555, 8954, 9032, 9554, 9824, 10139, 10353, 10563, 10647, 10891,
        11045, 11183, 11264, 11380, 11609, 11791, 11945, 12191, 12379, 12515, 17069, 18607, 18685,
        24852, 24900, 26012, 26068, 27495, 27685, 28657, 29984, 31075, 31985, 32742, 32767,
    ],
    [
        0, 1141, 4960, 12209, 18241, 18880, 20789, 21056, 21405, 22130, 22460, 23114, 23639, 24188,
        24631, 28135, 28264, 28564, 29020, 29410, 29978, 30267, 30556, 30943, 31128, 31259, 31386,
        31517, 32073, 32181, 32260, 32327, 32401, 32504, 32558, 32619, 32676, 32718, 32767,
    ],
    [
        0, 12167, 12274, 12500, 12577, 13281, 13362, 13655, 13877, 14102, 14299, 14627, 14720,
        14883, 15017, 15071, 15173, 15285, 15402, 15528, 15659, 15755, 15914, 16026, 17978, 20258,
        20344, 21885, 21945, 24472, 24528, 25336, 25471, 28636, 29765, 30638, 31770, 32729, 32767,
    ],
    [
        0, 5472, 10004, 13943, 16168, 16494, 18855, 19238, 19615, 19933, 20747, 21018, 21424,
        21923, 22164, 23067, 25043, 25245, 25583, 25769, 26043, 26245, 26665, 26818, 27287, 27518,
        27614, 27813, 27922, 28095, 31808, 31962, 32220, 32351, 32452, 32568, 32638, 32718, 32767,
    ],
    [
        0, 9975, 10040, 10106, 10139, 10183, 10223, 10264, 10336, 10402, 10434, 10486, 10519,
        10566, 10610, 10637, 10659, 10705, 10752, 10793, 10830, 10857, 10949, 10993, 13156, 15085,
        15100, 17344, 17374, 18996, 19012, 23469, 23540, 25016, 26800, 29793, 31178, 32754, 32767,
    ],
    [
        0, 7536, 9234, 10748, 11865, 12350, 14855, 15212, 16672, 17494, 18034, 18809, 19151, 19823,
        20719, 21185, 22265, 22601, 23162, 24261, 24537, 24798, 25680, 26080, 26730, 27131, 27371,
        27611, 27838, 28129, 28410, 28623, 31836, 32047, 32226, 32401, 32537, 32685, 32767,
    ],
    [
        0, 10773, 11042, 11229, 11301, 11669, 11767, 12075, 12434, 12694, 13130, 13372, 13518,
        13790, 14018, 14290, 14458, 14630, 14784, 14968, 15116, 15246, 15452, 15626, 17197, 20518,
        20591, 21841, 21927, 23565, 23615, 24516, 24611, 26441, 27774, 28783, 31723, 32729, 32767,
    ],
    [
        0, 14426, 14532, 14652, 14714, 14796, 14865, 14924, 15041, 15156, 15213, 15292, 15383,
        15467, 15550, 15618, 15680, 15747, 15826, 15885, 15945, 15993, 16117, 16186, 17811, 19298,
        19339, 20534, 20586, 21818, 21842, 23144, 23246, 24453, 27792, 29965, 31245, 32740, 32767,
    ],
    [
        0, 10296, 10393, 10497, 10566, 10691, 10769, 10835, 10992, 11107, 11166, 11260, 11329,
        11425, 11501, 11569, 11623, 11686, 11808, 11870, 11930, 11984, 12102, 12179, 13865, 15605,
        15642, 17333, 17385, 18879, 18899, 21974, 22036, 23604, 26227, 29545, 31075, 32739, 32767,
    ],
    [
        0, 13101, 13222, 13362, 13432, 13535, 13621, 13687, 13915, 14036, 14100, 14193, 14331,
        14435, 14524, 14591, 14670, 14751, 14831, 14912, 14997, 15055, 15194, 15281, 17048, 18640,
        18681, 20670, 20756, 22068, 22087, 23158, 23231, 25115, 26474, 27559, 30112, 32732, 32767,
    ],
    [
        0, 12082, 12164, 12266, 12321, 12397, 12473, 12528, 12648, 12757, 12810, 12894, 12958,
        13035, 13116, 13171, 13235, 13303, 13388, 13452, 13519, 13569, 13680, 13772, 15279, 16565,
        16603, 17957, 18021, 19069, 19087, 22955, 23024, 24461, 26471, 27610, 29268, 32736, 32767,
    ],
    [
        0, 2750, 3286, 4170, 4546, 4935, 5546, 5762, 6296, 6745, 7026, 7244, 7440, 7672, 7943,
        26911, 27065, 27190, 27484, 27743, 27956, 29557, 29810, 29969, 30288, 30499, 30629, 30741,
        30866, 31039, 31201, 31282, 31444, 31574, 31675, 31806, 31900, 31985, 32767,
    ],
];

pub const MODEL_WIDTH_METHOD: usize = 27;

pub const SYMBOL_TO_CHAR_METHOD: [u8; MODEL_WIDTH_METHOD] = [
    0, b'e', b'a', b'i', b'o', b't', b'n', b'r', b's', b'l', b'c', b'u', b'h', b'd', b'p', b'm',
    b'y', b'g', b'b', b'f', b'w', b'v', b'k', b'z', b'x', b'q', b'j',
];

pub const CHAR_TO_SYMBOL_METHOD: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0, 0, 0, 0, 0,
    0, 0, 2, 18, 10, 13, 1, 19, 17, 12, 3, 26, 22, 9, 15, 6, 4, 14, 25, 7, 8, 5, 11, 21, 20, 24,
    16, 23, 0, 0, 0, 0, 25, 0, 2, 18, 10, 13, 1, 19, 17, 12, 3, 26, 22, 9, 15, 6, 4, 14, 25, 7, 8,
    5, 11, 21, 20, 24, 16, 23, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub const MODEL_CF_METHOD: [[u16; 28]; MODEL_WIDTH_METHOD] = [
    [
        0, 0, 1956, 4146, 6063, 7423, 9305, 10276, 11226, 13687, 14907, 16131, 17056, 18369, 19922,
        21644, 23210, 24604, 25065, 26398, 28475, 30408, 31395, 31725, 32367, 32402, 32623, 32767,
    ],
    [
        0, 4880, 5243, 6669, 7714, 8472, 9634, 11619, 15233, 16738, 18031, 18650, 18993, 20913,
        22653, 25217, 25802, 26404, 26589, 28244, 29312, 31398, 31845, 32080, 32183, 32478, 32695,
        32767,
    ],
    [
        0, 1737, 2901, 3020, 3666, 3734, 8543, 12634, 15523, 17208, 21367, 22949, 23407, 24602,
        25562, 26470, 28001, 28323, 29045, 30727, 31040, 31251, 32084, 32387, 32496, 32598, 32745,
        32767,
    ],
    [
        0, 1420, 3454, 4922, 5132, 6751, 9959, 13993, 14547, 18653, 20482, 23055, 23598, 23956,
        25067, 25566, 27966, 27973, 28483, 29334, 29844, 30620, 31685, 31882, 32375, 32425, 32757,
        32767,
    ],
    [
        0, 4209, 5209, 5503, 5994, 6779, 9809, 13734, 16394, 17728, 20734, 21636, 23733, 24071,
        24785, 26194, 28482, 28566, 29403, 30186, 30894, 31383, 32100, 32294, 32382, 32531, 32744,
        32767,
    ],
    [
        0, 6157, 9213, 10380, 13294, 15931, 16600, 16829, 17987, 18142, 18574, 18667, 19055, 21787,
        22406, 24449, 24772, 25604, 25663, 29173, 30781, 31791, 31995, 32219, 32350, 32610, 32748,
        32767,
    ],
    [
        0, 4891, 8913, 10772, 13035, 15218, 19118, 19893, 20080, 21202, 22114, 23568, 23895, 24392,
        26546, 26760, 27744, 28021, 30534, 31037, 31376, 31650, 32250, 32490, 32548, 32565, 32709,
        32767,
    ],
    [
        0, 4689, 9532, 12839, 16144, 19335, 21427, 21880, 22583, 23728, 24811, 25404, 25957, 26777,
        27443, 28035, 29213, 30179, 30481, 31447, 31666, 31782, 32303, 32484, 32510, 32511, 32751,
        32767,
    ],
    [
        0, 5750, 9495, 10402, 12127, 13082, 19468, 19684, 19710, 21681, 23775, 24600, 25576, 26982,
        27084, 27953, 30214, 30492, 30515, 30947, 31019, 31446, 31709, 31847, 31884, 31884, 32761,
        32767,
    ],
    [
        0, 1827, 5418, 7243, 11598, 13806, 14484, 14972, 15432, 15532, 16785, 16863, 17441, 18465,
        19298, 23066, 24626, 26449, 26523, 27067, 29601, 31460, 31683, 32257, 32266, 32312, 32318,
        32767,
    ],
    [
        0, 3085, 6902, 11386, 13498, 18692, 21202, 21246, 22818, 22950, 23980, 24484, 25962, 30168,
        30176, 30181, 30391, 31287, 31288, 31293, 31294, 31297, 31592, 32703, 32712, 32712, 32767,
        32767,
    ],
    [
        0, 1460, 2488, 3382, 4190, 4382, 7997, 13589, 16710, 21209, 25118, 26178, 26204, 26432,
        27181, 28387, 30216, 30257, 30864, 31867, 32177, 32219, 32374, 32455, 32551, 32617, 32754,
        32767,
    ],
    [
        0, 3497, 9469, 11581, 13649, 15750, 19606, 19858, 20362, 20409, 23728, 23738, 24110, 26020,
        26170, 26284, 28241, 29240, 29247, 29296, 29442, 29819, 30442, 30559, 30604, 30604, 32763,
        32767,
    ],
    [
        0, 7725, 13656, 15552, 18212, 19436, 22322, 22566, 23251, 23443, 26247, 26267, 26704,
        26809, 27460, 27622, 29567, 30839, 30969, 31225, 31379, 31448, 32114, 32127, 32143, 32143,
        32739, 32767,
    ],
    [
        0, 3654, 6677, 7904, 9941, 12843, 16232, 16550, 19915, 20300, 22842, 22898, 23406, 25127,
        25246, 25931, 27743, 28101, 28153, 28321, 29252, 31975, 32243, 32396, 32480, 32494, 32721,
        32767,
    ],
    [
        0, 2259, 4242, 6358, 8676, 10453, 10663, 11361, 12343, 12503, 13022, 13049, 13568, 15061,
        16673, 21204, 22114, 23775, 23801, 26743, 28089, 29382, 31518, 31992, 32482, 32634, 32711,
        32767,
    ],
    [
        0, 8569, 10926, 11269, 11458, 12219, 15419, 15811, 16168, 19632, 21864, 22233, 22272,
        23584, 23897, 24563, 27086, 27677, 27838, 30347, 30897, 31720, 32305, 32316, 32374, 32410,
        32766, 32767,
    ],
    [
        0, 5421, 10228, 13261, 16435, 18453, 18538, 19596, 22477, 22749, 25180, 25189, 26615,
        28325, 28370, 28391, 28704, 29610, 30232, 30341, 30376, 30446, 32750, 32760, 32764, 32764,
        32765, 32767,
    ],
    [
        0, 3536, 8233, 9248, 10667, 11916, 16198, 16366, 17262, 17620, 22341, 22384, 23103, 23432,
        24178, 24653, 27955, 28446, 28462, 29156, 29914, 30860, 31440, 31618, 31718, 31718, 32711,
        32767,
    ],
    [
        0, 6957, 8777, 9282, 11114, 12083, 14679, 14746, 15172, 16789, 19423, 19431, 20076, 20456,
        21353, 21603, 22831, 23934, 24047, 24077, 26986, 30996, 32258, 32271, 32421, 32421, 32764,
        32767,
    ],
    [
        0, 4391, 7270, 8148, 9018, 9499, 14426, 14654, 14752, 18571, 23671, 23681, 23696, 24591,
        24714, 24801, 27775, 27825, 27830, 30016, 30215, 30862, 31659, 31682, 31823, 31824, 32740,
        32767,
    ],
    [
        0, 508, 5145, 6952, 9251, 10376, 10985, 11140, 11725, 11743, 12502, 12513, 12662, 15602,
        17187, 22455, 22627, 23556, 23562, 26519, 30666, 32564, 32574, 32641, 32646, 32708, 32730,
        32767,
    ],
    [
        0, 2561, 9484, 10526, 12712, 13144, 24077, 24752, 24913, 25311, 29557, 29584, 29814, 30171,
        30199, 30250, 31654, 31986, 32003, 32134, 32255, 32412, 32625, 32686, 32687, 32687, 32751,
        32767,
    ],
    [
        0, 1902, 10556, 12815, 13501, 14918, 18520, 18600, 18633, 21631, 26209, 26218, 26391,
        26794, 26889, 27833, 30807, 31246, 31254, 31774, 32060, 32183, 32247, 32252, 32655, 32655,
        32756, 32767,
    ],
    [
        0, 3478, 6497, 8308, 12600, 13818, 17038, 17122, 17152, 17528, 20748, 21906, 22255, 22599,
        22677, 24481, 25735, 28777, 28815, 28891, 29073, 31075, 31424, 31433, 31485, 31485, 32764,
        32767,
    ],
    [
        0, 17596, 18281, 18455, 19256, 19875, 20521, 20599, 20949, 21038, 21413, 21416, 23479,
        24932, 25804, 26610, 28270, 29555, 29557, 30312, 31584, 32451, 32516, 32525, 32623, 32701,
        32703, 32767,
    ],
    [
        0, 392, 8308, 10995, 11504, 13348, 22180, 22194, 22231, 22248, 23981, 23981, 26646, 26978,
        27000, 27014, 30409, 30435, 30435, 30457, 30474, 30503, 30697, 30704, 30818, 30818, 31861,
        32767,
    ],
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assert_parameters() {
        assert_eq!(CODE_VALUE_BITS, 17);
        assert_eq!(MAX_CODE, 131071);
        assert_eq!(ONE_FOURTH, 32768);
        assert_eq!(ONE_HALF, 65536);
        assert_eq!(THREE_FOURTHS, 98304);
    }
}
