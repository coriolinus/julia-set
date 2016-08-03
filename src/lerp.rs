use std::ops::{Add, Sub, Mul};
use num::Float;

pub trait Lerp<F> {
    fn lerp(self, other: Self, t: F) -> Self;
}

impl<T, F> Lerp<F> for T
    where F: Float,
          T: Copy + Add<Output = T> + Sub<Output = T> + Mul<F, Output = T>
{
    fn lerp(self, other: T, t: F) -> T {
        self + ((other - self) * t)
    }
}
