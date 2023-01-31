use std::fmt::{Debug, Formatter, Result as FmtResult};

use rosu_pp::{
    osu::{OsuObject, OsuObjectKind},
    parse::Pos2,
};

pub struct HitObject<'h> {
    pub matched_frame: bool,
    h: &'h dyn HitObjectExt,
}

impl<'h> HitObject<'h> {
    pub fn new(h: &'h dyn HitObjectExt) -> Self {
        Self {
            h,
            matched_frame: false,
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
        self.h.is_slider()
    }

    /// If the note is a slider, return its endtime
    pub fn slider_end_time(&self) -> Option<i32> {
        self.h.slider_end_time()
    }
}

pub trait HitObjectExt {
    fn start_time(&self) -> i32;
    fn slider_end_time(&self) -> Option<i32>;
    fn pos(&self) -> Pos2;
    fn stack_count(&self) -> i32;
    fn is_normal(&self) -> bool;
    fn is_slider(&self) -> bool;
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
    fn stack_count(&self) -> i32 {
        self.stack_height as i32
    }

    #[inline]
    fn is_normal(&self) -> bool {
        self.is_circle()
    }

    #[inline]
    fn is_slider(&self) -> bool {
        self.is_slider()
    }
}

impl Debug for HitObject<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        #[derive(Debug)]
        enum HitObjectKind {
            Circle,
            Slider,
            Spinner,
        }

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
            kind: if self.h.is_normal() {
                HitObjectKind::Circle
            } else if self.h.is_slider() {
                HitObjectKind::Slider
            } else {
                HitObjectKind::Spinner
            },
            matched_frame: self.matched_frame,
        };

        Debug::fmt(&h, f)
    }
}
