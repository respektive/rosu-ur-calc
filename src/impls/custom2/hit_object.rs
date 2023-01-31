use std::fmt::{Debug, Formatter, Result as FmtResult};

use rosu_pp::{
    osu::{OsuObject, OsuObjectKind},
    parse::Pos2,
};

pub struct HitObject<'h> {
    pub found_hit: bool,
    h: &'h dyn HitObjectExt,
}

impl<'h> HitObject<'h> {
    pub fn new(h: &'h dyn HitObjectExt) -> Self {
        Self {
            h,
            found_hit: false,
        }
    }
}

impl HitObject<'_> {
    pub fn start_time(&self) -> i32 {
        self.h.start_time()
    }

    pub fn pos(&self) -> Pos2 {
        self.h.pos()
    }

    pub fn is_slider(&self) -> bool {
        matches!(self.h.kind(), HitObjectKind::Slider)
    }

    pub fn ignore(&self) -> bool {
        self.h.ignore()
    }

    /// If the note is a slider, return its endtime
    pub fn slider_end_time(&self) -> Option<i32> {
        self.h.slider_end_time()
    }
}

#[derive(Debug)]
pub enum HitObjectKind {
    Circle,
    Slider,
    Spinner,
}

pub trait HitObjectExt {
    fn start_time(&self) -> i32;
    fn slider_end_time(&self) -> Option<i32>;
    fn pos(&self) -> Pos2;
    fn kind(&self) -> HitObjectKind;
    fn ignore(&self) -> bool;
}

impl HitObjectExt for OsuObject {
    #[inline]
    fn start_time(&self) -> i32 {
        self.start_time as i32
    }

    #[inline]
    fn slider_end_time(&self) -> Option<i32> {
        match self.kind {
            OsuObjectKind::Slider(ref slider) => Some(slider.end_time as i32),
            OsuObjectKind::Circle | OsuObjectKind::Spinner { .. } => None,
        }
    }

    #[inline]
    fn pos(&self) -> Pos2 {
        self.stacked_pos()
    }

    #[inline]
    fn kind(&self) -> HitObjectKind {
        match self.kind {
            OsuObjectKind::Circle => HitObjectKind::Circle,
            OsuObjectKind::Slider(_) => HitObjectKind::Slider,
            OsuObjectKind::Spinner { .. } => HitObjectKind::Spinner,
        }
    }

    #[inline]
    fn ignore(&self) -> bool {
        self.is_spinner()
    }
}

impl Debug for HitObject<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        #[derive(Debug)]
        #[allow(unused)]
        struct HitObject {
            time: i32,
            pos: Pos2,
            kind: HitObjectKind,
            matched_frame: bool,
        }

        let h = HitObject {
            time: self.h.start_time(),
            pos: self.h.pos(),
            kind: self.h.kind(),
            matched_frame: self.found_hit,
        };

        Debug::fmt(&h, f)
    }
}
