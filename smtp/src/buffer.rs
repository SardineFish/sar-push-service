use std::{ops::{Deref, DerefMut, Index, Range, RangeFrom, RangeTo}, slice::SliceIndex};

pub struct Buffer{
    buf: Vec<u8>,
    range: Range<usize>,
}

impl Buffer {
    pub fn new(size: usize) -> Self {
        Buffer {
            buf: vec![0; size],
            range: 0..size
        }
    }
    // pub fn slice(self, range: Range<usize>) -> Self {
    //     Self {
    //         range: (self.range.start + range.start)..(self.range.start + range.end),
    //         ..self
    //     }
    // }
    pub fn extend_head_uncheck(self, size: usize) -> Self {
        Self {
            range: (self.range.start - size)..self.range.end,
            ..self
        }
    }
    pub fn extend_tail_uncheck(self, size: usize) -> Self {
        Self {
            range: self.range.start..(self.range.end + size),
            ..self
        }
    }
    pub fn shrink_head_uncheck(self, size: usize) -> Self {
        Self {
            range: (self.range.start + size)..self.range.end,
            ..self
        }
    }
    pub fn shrink_tail_uncheck(self, size: usize) -> Self {
        Self {
            range: self.range.start..(self.range.end - size),
            ..self
        }
    }
    pub fn reset(self) -> Self {
        Self {
            range: 0..self.buf.len(),
            ..self
        }
    }
    pub fn raw(&self) -> &[u8] {
        &self.buf
    }
}

impl Deref for Buffer {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.buf[self.range.clone()]
    }
}

impl DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buf[self.range.clone()]
    }
}

pub trait SliceFrom {
    fn slice(self, range: RangeFrom<usize>) -> Self;
}
pub trait SliceTo {
    fn slice(self, range: RangeTo<usize>) -> Self;
}

impl SliceFrom for Buffer {
    fn slice(self, range: RangeFrom<usize>) -> Self {
        Self {
            range: (self.range.start + range.start)..self.range.end,
            ..self
        }
    }
}

impl SliceTo for Buffer {
    fn slice(self, range: RangeTo<usize>) -> Self {
        Self {
            range: self.range.start..range.end,
            ..self
        }
    }
}