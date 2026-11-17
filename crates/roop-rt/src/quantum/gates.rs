//! The single-qubit gates and their inverses. Each `_inv` is the adjoint, so
//! `uncall` of a circuit runs it backward exactly.

use crate::quantum::{Amp, gate, phase};
use std::f64::consts::{FRAC_1_SQRT_2, FRAC_PI_2, FRAC_PI_4, PI};

const ONE: Amp = Amp { re: 1.0, im: 0.0 };
const ZERO: Amp = Amp { re: 0.0, im: 0.0 };
const H: [[Amp; 2]; 2] = [
    [
        Amp {
            re: FRAC_1_SQRT_2,
            im: 0.0,
        },
        Amp {
            re: FRAC_1_SQRT_2,
            im: 0.0,
        },
    ],
    [
        Amp {
            re: FRAC_1_SQRT_2,
            im: 0.0,
        },
        Amp {
            re: -FRAC_1_SQRT_2,
            im: 0.0,
        },
    ],
];
const X: [[Amp; 2]; 2] = [[ZERO, ONE], [ONE, ZERO]];
const Y: [[Amp; 2]; 2] = [
    [ZERO, Amp { re: 0.0, im: -1.0 }],
    [Amp { re: 0.0, im: 1.0 }, ZERO],
];

macro_rules! gates {
    ($($name:ident $inv:ident => $fwd:expr, $back:expr;)*) => {$(
        /// # Safety
        /// `q` must be valid.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(q: *const i64) {
            unsafe { gate(q, $fwd) }
        }

        /// # Safety
        /// `q` must be valid.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $inv(q: *const i64) {
            unsafe { gate(q, $back) }
        }
    )*};
}

gates! {
    roop_q_h roop_q_h_inv => H, H;
    roop_q_x roop_q_x_inv => X, X;
    roop_q_y roop_q_y_inv => Y, Y;
    roop_q_z roop_q_z_inv => phase(PI), phase(PI);
    roop_q_s roop_q_s_inv => phase(FRAC_PI_2), phase(-FRAC_PI_2);
    roop_q_t roop_q_t_inv => phase(FRAC_PI_4), phase(-FRAC_PI_4);
}
