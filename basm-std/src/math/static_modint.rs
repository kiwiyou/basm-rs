// Currently, no work has been done for preventing correctness issues from overflowing and
// performance problems. This should only be considered as a sort of skeleton code.

use core::{fmt::Display, ops::*};

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct ModInt<const M: u64>(u64);

impl<const M: u64> ModInt<M> {
    pub fn get(self) -> u64 {
        self.0
    }
}

impl<const M: u64> Display for ModInt<M> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl<const M: u64> From<u64> for ModInt<M> {
    fn from(value: u64) -> Self {
        Self(value % M)
    }
}

impl<const M: u64> From<ModInt<M>> for u64 {
    fn from(value: ModInt<M>) -> Self {
        value.0
    }
}

impl<const M: u64> Add for ModInt<M> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let (val, carry) = self.0.overflowing_add(rhs.0);
        Self(if val >= M || carry {
            val.wrapping_sub(M)
        } else {
            val
        })
    }
}

impl<const M: u64> AddAssign for ModInt<M> {
    fn add_assign(&mut self, rhs: Self) {
        let (val, carry) = self.0.overflowing_add(rhs.0);
        self.0 = if val >= M || carry {
            val.wrapping_sub(M)
        } else {
            val
        }
    }
}

impl<const M: u64> Div for ModInt<M> {
    type Output = Self;
    /// May panic if M == 0
    fn div(self, rhs: Self) -> Self::Output {
        let inv = super::egcd(rhs.0 as i128, M as i128).1;
        let inv = (M as i128 + inv) as u64 % M;
        Self(self.0) * Self(inv)
    }
}

impl<const M: u64> DivAssign for ModInt<M> {
    fn div_assign(&mut self, rhs: Self) {
        let inv = super::egcd(rhs.0 as i128, M as i128).1;
        let inv = (M as i128 + inv) as u64 % M;
        *self *= Self(inv);
    }
}

impl<const M: u64> Mul for ModInt<M> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(((self.0 as u128 * rhs.0 as u128) % (M as u128)) as u64)
    }
}

impl<const M: u64> MulAssign for ModInt<M> {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 = ((self.0 as u128 * rhs.0 as u128) % (M as u128)) as u64;
    }
}

impl<const M: u64> Neg for ModInt<M> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self((M - self.0) % M)
    }
}

impl<const M: u64> Sub for ModInt<M> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}

impl<const M: u64> SubAssign for ModInt<M> {
    fn sub_assign(&mut self, rhs: Self) {
        *self += -rhs;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn static_small_mod_add() {
        let proper = |a: u64, b: u64, m: u64| ((a as u128 + b as u128) % (m as u128)) as u64;
        macro_rules! test {
            ($($m:expr),*) => {$(
                {
                    const M: u64 = $m;
                    for mut a in 0..100 {
                        if a > 50 {
                            a = a * 29 + 18;
                        }
                        let am = ModInt::<{ M }>::from(a);
                        for mut b in 0..100 {
                            if b > 50 {
                                b = b * 29 + 18;
                            }
                            let bm = ModInt::<{ M }>::from(b);
                            let t = am + bm;
                            assert_eq!(proper(a, b, M), t.0);
                        }
                    }
                }
            )*};
        }
        test!(2, 10, 593, 11729378, 2343246813781979);
    }

    #[test]
    fn static_large_mod_add() {
        let proper = |a: u64, b: u64, m: u64| ((a as u128 + b as u128) % (m as u128)) as u64;
        macro_rules! test {
            ($($m:expr),*) => {$(
                {
                    const M: u64 = $m;
                    let m = M / 3 * 2;
                    for a in [m, m.wrapping_add(1), m.wrapping_add(2), m.wrapping_add(30)] {
                        let am = ModInt::<{ M }>::from(a);
                        for b in [m, m.wrapping_add(1), m.wrapping_add(2), m.wrapping_add(30)] {
                            let bm = ModInt::<{ M }>::from(b);
                            let t = am + bm;
                            assert_eq!(proper(a, b, M), t.0);
                        }
                    }
                }
            )*};
        }
        test!(
            u64::MAX / 2,
            u64::MAX / 2 + 1,
            u64::MAX / 2 - 1000,
            u64::MAX / 3 * 2 + 40,
            u64::MAX - 1,
            u64::MAX
        );
    }

    #[test]
    fn static_small_mod_sub() {
        let proper =
            |a: u64, b: u64, m: u64| ((a as i128 - b as i128).rem_euclid(m as i128)) as u64;
        macro_rules! test {
            ($($m:expr),*) => {$(
                {
                    const M: u64 = $m;
                    for mut a in 0..100 {
                        if a > 50 {
                            a = a * 29 + 18;
                        }
                        let am = ModInt::<{ M }>::from(a);
                        for mut b in 0..100 {
                            if b > 50 {
                                b = b * 29 + 18;
                            }
                            let bm = ModInt::<{ M }>::from(b);
                            let t = am - bm;
                            assert_eq!(proper(a, b, M), t.0);
                        }
                    }
                }
            )*};
        }
        test!(2, 10, 593, 11729378, 2343246813781979);
    }

    #[test]
    fn static_large_mod_sub() {
        let proper =
            |a: u64, b: u64, m: u64| ((a as i128 - b as i128).rem_euclid(m as i128)) as u64;
        macro_rules! test {
            ($($m:expr),*) => {$(
                {
                    const M: u64 = $m;
                    let m = M / 3 * 2;
                    for a in [m, m.wrapping_add(1), m.wrapping_add(2), m.wrapping_add(30)] {
                        let am = ModInt::<{ M }>::from(a);
                        for b in [m, m.wrapping_add(1), m.wrapping_add(2), m.wrapping_add(30)] {
                            let bm = ModInt::<{ M }>::from(b);
                            let t = am - bm;
                            assert_eq!(proper(a, b, M), t.0);
                        }
                    }
                }
            )*};
        }
        test!(
            u64::MAX / 2,
            u64::MAX / 2 + 1,
            u64::MAX / 2 - 1000,
            u64::MAX / 3 * 2 + 40,
            u64::MAX - 1,
            u64::MAX
        );
    }

    #[test]
    fn static_small_mod_mul() {
        let proper = |a: u64, b: u64, m: u64| ((a as u128 * b as u128) % (m as u128)) as u64;
        macro_rules! test {
            ($($m:expr),*) => {$(
                {
                    const M: u64 = $m;
                    for mut a in 0..100 {
                        if a > 50 {
                            a = a * 29 + 18;
                        }
                        let am = ModInt::<{ M }>::from(a);
                        for mut b in 0..100 {
                            if b > 50 {
                                b = b * 29 + 18;
                            }
                            let bm = ModInt::<{ M }>::from(b);
                            let t = am * bm;
                            assert_eq!(proper(a, b, M), t.0);
                        }
                    }
                }
            )*};
        }
        test!(2, 10, 593, 11729378, 2343246813781979);
    }

    #[test]
    fn static_large_mod_mul() {
        let proper = |a: u64, b: u64, m: u64| ((a as u128 * b as u128) % (m as u128)) as u64;
        macro_rules! test {
            ($($m:expr),*) => {$(
                {
                    const M: u64 = $m;
                    let m = M / 3 * 2;
                    for a in [m, m.wrapping_add(1), m.wrapping_add(2), m.wrapping_add(30)] {
                        let am = ModInt::<{ M }>::from(a);
                        for b in [m, m.wrapping_add(1), m.wrapping_add(2), m.wrapping_add(30)] {
                            let bm = ModInt::<{ M }>::from(b);
                            let t = am * bm;
                            assert_eq!(proper(a, b, M), t.0);
                        }
                    }
                }
            )*};
        }
        test!(
            u64::MAX / 2,
            u64::MAX / 2 + 1,
            u64::MAX / 2 - 1000,
            u64::MAX / 3 * 2 + 40,
            u64::MAX - 1,
            u64::MAX
        );
    }
}
