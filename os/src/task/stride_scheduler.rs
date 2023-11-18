use core::{cmp::Ordering, ops::AddAssign};
use alloc::sync::Arc;
use super::TaskControlBlock;

pub static BIG_STRIDE: u8 = 255;
pub static HALF_BIG_STRIDE: u8 = 127;

#[derive(Clone, Copy)]
pub struct Stride(pub u8);


impl PartialOrd for Stride {
    // STRIDE_MAX - STRIDE_MIN <= BigStride / 2
    // Need reverse on the size to pop minimum everytime
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0 >= other.0 && self.0 - other.0 <= HALF_BIG_STRIDE
            || self.0 < other.0 && other.0 - self.0 > HALF_BIG_STRIDE{
            Some(Ordering::Less)
        }else{
            Some(Ordering::Greater)
        }
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        let _ = other;
        false
    }
}

impl Eq for Stride{}

impl Ord for Stride{
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap()
    }
}

impl AddAssign<u8> for Stride{
    fn add_assign(&mut self, other: u8) {
        self.0 += other;
    }
}

pub struct StrideWrapTcb(pub Stride, pub Arc<TaskControlBlock>);

impl PartialOrd for StrideWrapTcb{
    // Need reverse on the size to pop minimum everytime
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let res = self.0.partial_cmp(&other.0);
        match res {
            Some(Ordering::Less) => Some(Ordering::Greater),
            _ => Some(Ordering::Less)
        }
    }
}

impl PartialEq for StrideWrapTcb{
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}