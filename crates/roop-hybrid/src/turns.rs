use std::ops::Add;

/// A phase of `num / 2^log` of a full turn, reduced, with `num < 2^log`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Turns {
    pub num: u64,
    pub log: u32,
}

impl Turns {
    pub fn new(num: u64, log: u32) -> Self {
        let num = num & ((1u64 << log) - 1);
        let shift = num.trailing_zeros().min(log);
        match num {
            0 => Self { num: 0, log: 0 },
            _ => Self {
                num: num >> shift,
                log: log - shift,
            },
        }
    }

    pub fn is_zero(self) -> bool {
        self.num == 0
    }

    pub fn radians(self) -> f64 {
        std::f64::consts::TAU * self.num as f64 / (1u64 << self.log) as f64
    }
}

impl Add for Turns {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        let log = self.log.max(other.log);
        Self::new(
            (self.num << (log - self.log)) + (other.num << (log - other.log)),
            log,
        )
    }
}
