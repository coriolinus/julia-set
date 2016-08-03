use std::iter;
use std::iter::{Iterator, Skip, Chain, Once};
use std::ops::{Add, Sub, Mul};
use num::{Float};

pub trait Lerp<F> {
    /// Interpolate / extrapolate between `self` and `other` using `t` as the parameter.
    ///
    /// At `t == 0.0`, the result is equal to `self`.
    /// At `t == 1.0`, the result is equal to `other`.
    /// At all other points, the result is a mix of `self` and `other`, proportional to `t`.
    fn lerp(self, other: Self, t: F) -> Self;

    /// Create an iterator which lerps from `self` to `other`.
    ///
    /// The iterator is half-open: it includes `self`, but not `other`
    ///
    /// # Example
    ///
    /// ```
    /// # use julia_set::lerp::Lerp;
    /// // lerp between 3 and 5, collecting two items
    /// let items: Vec<f64> = 3.0_f64.lerp_iter(5.0, 2).collect();
    /// assert_eq!(vec![3.0, 4.0], items);
    /// ```
    fn lerp_iter(self, other: Self, steps: usize) -> LerpIterator<Self> where Self: Sized;

    /// Create an iterator which lerps from `self` to `other`.
    ///
    /// The iterator is closed: it returns both `self` and `other`.
    ///
    /// Note when `steps == 1`, `other` is returned instead of `self`.
    ///
    /// # Example
    ///
    /// ```
    /// # use julia_set::lerp::Lerp;
    /// assert_eq!(vec![3.0, 5.0], 3.0_f64.lerp_iter(5.0, 2).collect::<Vec<f64>>());
    /// ```
    fn lerp_iter_closed(self,
                        other: Self,
                        mut steps: usize)
                        -> Skip<Chain<LerpIterator<Self>, Once<Self>>>
        where Self: Sized + Copy +
                Add<Output = Self> + Sub<Output = Self> + Mul<f64, Output = Self>,
              F: Float
    {
        // reduce the number of times we consume the sub-iterator,
        // because we unconditionally add an element to the end.
        let mut skipn = 0;
        if steps > 0 {
            steps -= 1;
            skipn = 1;
        }
        self.lerp_iter(other, steps).chain(iter::once(other)).skip(skipn)
    }
}

impl<T, F> Lerp<F> for T
    where T: Copy + Add<Output = T> + Sub<Output = T> + Mul<F, Output = T>,
          F: Float
{
    fn lerp(self, other: T, t: F) -> T {
        self + ((other - self) * t)
    }

    fn lerp_iter(self, other: Self, steps: usize) -> LerpIterator<Self>
        where Self: Sized
    {
        LerpIterator::new(self, other, steps)
    }
}

/// An iterator across a range defined by its endpoints and the number of intermediate steps.
pub struct LerpIterator<T> {
    begin: T,
    end: T,
    steps: usize,
    current_step: usize, //    float_type: PhantomData<F>,
}

impl<T> LerpIterator<T> {
    fn new(begin: T, end: T, steps: usize) -> LerpIterator<T> {
        LerpIterator {
            begin: begin,
            end: end,
            steps: steps,
            current_step: 0, //            float_type: PhantomData,
        }
    }
}

impl<T> Iterator for LerpIterator<T>
    where T: Copy + Add<Output = T> + Sub<Output = T> + Mul<f64, Output = T>
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.current_step >= self.steps {
            None
        } else {
            let t = self.current_step as f64 / self.steps as f64;
            self.current_step += 1;
            Some(self.begin.lerp(self.end, t))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = if self.current_step >= self.steps {
            0
        } else {
            self.steps - self.current_step
        };
        (remaining, Some(remaining))
    }
}

impl<T> ExactSizeIterator for LerpIterator<T>
    where T: Copy + Add<Output = T> + Sub<Output = T> + Mul<f64, Output = T>
{
}
