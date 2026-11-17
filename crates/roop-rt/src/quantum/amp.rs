use std::ops::Mul;

/// A complex amplitude.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Amp {
    pub re: f64,
    pub im: f64,
}

impl Mul for Amp {
    type Output = Amp;
    fn mul(self, o: Amp) -> Amp {
        Amp {
            re: self.re * o.re - self.im * o.im,
            im: self.re * o.im + self.im * o.re,
        }
    }
}

impl Amp {
    pub fn norm(self) -> f64 {
        self.re * self.re + self.im * self.im
    }
}
