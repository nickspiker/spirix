#[derive(Clone, Copy)]
pub struct ColourScheme {
    pub colour: [u8; 3],
}

#[derive(Clone, Copy)]
pub struct ScalarColours {
    pub normal_positive: ColourScheme,
    pub normal_negative: ColourScheme,
    pub exploded_positive: ColourScheme,
    pub exploded_negative: ColourScheme,
    pub vanished_positive: ColourScheme,
    pub vanished_negative: ColourScheme,
    pub zero: ColourScheme,
    pub undefined: ColourScheme,
    pub integer_exponent: ColourScheme,
    pub fractional_exponent: ColourScheme,
}

pub const COLOURS: ScalarColours = ScalarColours {
    normal_positive: ColourScheme {
        colour: [0xff, 0x80, 0x80],
    },
    normal_negative: ColourScheme {
        colour: [0x80, 0x80, 0xff],
    },
    exploded_positive: ColourScheme {
        colour: [0xff, 0x00, 0x00],
    },
    exploded_negative: ColourScheme {
        colour: [0x00, 0x00, 0xff],
    },
    vanished_positive: ColourScheme {
        colour: [0xff, 0x00, 0x00],
    },
    vanished_negative: ColourScheme {
        colour: [0x00, 0x00, 0xff],
    },
    zero: ColourScheme {
        colour: [0x80, 0xff, 0x80],
    },
    undefined: ColourScheme {
        colour: [0xff, 0x80, 0xff],
    },
    integer_exponent: ColourScheme {
        colour: [0xff, 0xff, 0x80],
    },
    fractional_exponent: ColourScheme {
        colour: [0x80, 0xff, 0xff],
    },
};