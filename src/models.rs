#[derive(Debug)]
pub struct ReplayData {
    pub timestamp: f64,
    pub x: f32,
    pub y: f32,
    pub keys: Buttons,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Buttons(u8);

impl Buttons {
    const M1: u8 = 1 << 0;
    const M2: u8 = 1 << 1;
    const K1: u8 = 1 << 2;
    const K2: u8 = 1 << 3;

    pub fn from_f32(float: f32) -> Self {
        let mut bits = float as u8;

        if (bits & Self::K1) > 0 {
            assert!((bits & Self::M1) > 0);
            bits -= Self::M1;
        }

        if (bits & Self::K2) > 0 {
            assert!((bits & Self::M2) > 0);
            bits -= Self::M2;
        }

        Self(bits)
    }

    pub fn m1(self) -> bool {
        (self.0 & Self::M1) > 0
    }

    pub fn m2(self) -> bool {
        (self.0 & Self::M2) > 0
    }

    pub fn k1(self) -> bool {
        (self.0 & Self::K1) > 0
    }

    pub fn k2(self) -> bool {
        (self.0 & Self::K2) > 0
    }
}
