use std::fmt::{Debug, Formatter, Result as FmtResult};

#[derive(Debug)]
pub struct ReplayData {
    pub timestamp: f64,
    pub x: f32,
    pub y: f32,
    pub keys: Buttons,
}

#[derive(Copy, Clone, Default)]
pub struct Buttons(u8);

impl Buttons {
    const M1: u8 = 1 << 0;
    const M2: u8 = 1 << 1;
    const K1: u8 = 1 << 2;
    const K2: u8 = 1 << 3;

    pub fn from_f32(float: f32) -> Self {
        let mut bits = float as u8;

        if (bits & Self::K1) > 0 {
            bits &= !Self::M1;
        }

        if (bits & Self::K2) > 0 {
            bits &= !Self::M2;
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

    pub fn is_new_press(self, prev: Self) -> bool {
        let m1 = self.m1() && !prev.m1();
        let m2 = self.m2() && !prev.m2();
        let k1 = self.k1() && !prev.k1();
        let k2 = self.k2() && !prev.k2();

        m1 || m2 || k1 || k2
    }
}

impl Debug for Buttons {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Debug::fmt(&self.0, f)
    }
}
