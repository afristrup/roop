use std::ops::{Add, Mul, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub const ZERO: Self = Self { re: 0.0, im: 0.0 };
    pub const ONE: Self = Self { re: 1.0, im: 0.0 };

    pub fn polar(radians: f64) -> Self {
        Self {
            re: radians.cos(),
            im: radians.sin(),
        }
    }

    pub fn conj(self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }

    pub fn scale(self, k: f64) -> Self {
        Self {
            re: self.re * k,
            im: self.im * k,
        }
    }
}

impl Add for Complex {
    type Output = Self;
    fn add(self, o: Self) -> Self {
        Self {
            re: self.re + o.re,
            im: self.im + o.im,
        }
    }
}

impl Sub for Complex {
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        Self {
            re: self.re - o.re,
            im: self.im - o.im,
        }
    }
}

impl Mul for Complex {
    type Output = Self;
    fn mul(self, o: Self) -> Self {
        Self {
            re: self.re * o.re - self.im * o.im,
            im: self.re * o.im + self.im * o.re,
        }
    }
}
