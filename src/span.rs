use std::ops::Range;

pub type Span = Range<usize>;

pub trait TweakSpan {
    fn tweak(&mut self, shift: usize, extend: isize);
    fn shift(&mut self, shift: usize);
}

impl TweakSpan for Span {
    fn tweak(&mut self, shift: usize, extend: isize) {
        self.start += shift;
        self.end += shift;
        if extend < 0 {
            self.start -= (-extend) as usize;
        } else {
            self.end += extend as usize;
        }
    }
    fn shift(&mut self, shift: usize) {
        self.tweak(shift, 0);
    }
}

