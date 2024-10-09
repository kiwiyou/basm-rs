pub mod miller_rabin;
pub use miller_rabin::*;
mod sieve;
pub use sieve::LinearSieve;
mod pollard_rho;
pub use pollard_rho::factorize;
mod reeds_sloane;
pub use reeds_sloane::{linear_fit, reeds_sloane};
pub mod static_modint;
pub mod dynamic_modint;

pub mod ntt;
pub use ntt::*;

mod modmul;

// reference: https://nyaannyaan.github.io/library/trial/fast-gcd.hpp.html

use core::ops::*;

pub trait GcdOps:
    Copy
    + From<u8>
    + PartialOrd
    + Add<Output = Self>
    + Sub<Output = Self>
    + Shl<u32, Output = Self>
    + Shr<u32, Output = Self>
    + ShrAssign<u32>
{
    fn trailing_zeros(self) -> u32;
    fn wrapping_sub(self, rhs: Self) -> Self;
}

macro_rules! impl_gcd_ops {
    ($($t:ty),*) => { $(
        impl GcdOps for $t {
            fn trailing_zeros(self) -> u32 {
                self.trailing_zeros()
            }
            fn wrapping_sub(self, rhs: Self) -> Self {
                self.wrapping_sub(rhs)
            }
        }
    )* };
}
impl_gcd_ops!(u8, u16, u32, u64, u128, usize);

/// Returns the greatest common divisor (GCD) of `a` and `b` if neither is zero, otherwise returns `a + b`.
pub fn gcd<T: GcdOps>(mut a: T, mut b: T) -> T {
    if a == 0.into() || b == 0.into() {
        a + b
    } else {
        let n = a.trailing_zeros();
        let m = b.trailing_zeros();
        a >>= n;
        b >>= m;
        while a != b {
            let m = a.wrapping_sub(b).trailing_zeros();
            let f = a > b;
            let c = if f { a } else { b };
            b = if f { b } else { a };
            a = (c - b) >> m;
        }
        a << n.min(m)
    }
}

/// Returns the least common multiplier (LCM) of `a` and `b` if neither is zero, otherwise returns `0`.
pub fn lcm<T: GcdOps + Mul<Output = T> + Div<Output = T>>(a: T, b: T) -> T {
    if a == 0.into() && b == 0.into() {
        0.into()
    } else {
        a / gcd(a, b) * b
    }
}

pub trait EgcdOps:
    Copy
    + From<i8>
    + PartialOrd
    + Neg<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Rem<Output = Self>
{
}

macro_rules! impl_egcd_ops {
    ($($t:ty),*) => { $(
        impl EgcdOps for $t {}
    )* };
}
impl_egcd_ops!(i8, i16, i32, i64, i128, isize);

/// Returns `(g, x, y)`, where `g` is the GCD of `a.abs()` and `b.abs()`, and `x`, `y` are integers satisfying `a*x + b*y = g`.
pub fn egcd<T: EgcdOps>(mut a: T, mut b: T) -> (T, T, T) {
    let mut c: [T; 4] = if a > b {
        (a, b) = (b, a);
        [0, 1, 1, 0].map(|x| x.into())
    } else {
        [1, 0, 0, 1].map(|x| x.into())
    }; // treat as a row-major 2x2 matrix
    loop {
        if a == 0.into() {
            break if b < 0.into() {
                (-b, -c[1], -c[3])
            } else {
                (b, c[1], c[3])
            };
        }
        let (q, r) = (b / a, b % a);
        (a, b) = (r, a);
        c = [c[1] - q * c[0], c[0], c[3] - q * c[2], c[2]];
    }
}

pub trait ModOps<T>:
    Copy
    + PartialOrd
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Rem<Output = Self>
where
    T: PartialEq,
{
    fn zero() -> T;
    fn one() -> T;
    fn two() -> T;
    fn my_wrapping_sub(&self, other: T) -> T;
    fn modadd(x: T, y: T, modulo: T) -> T;
    fn modsub(x: T, y: T, modulo: T) -> T;
    fn modinv(&self, modulo: T) -> Option<T>;
    fn modmul(x: T, y: T, modulo: T) -> T;
}

macro_rules! impl_mod_ops_signed {
    ($($t:ty),*) => { $(
        impl ModOps<$t> for $t {
            fn zero() -> $t { 0 }
            fn one() -> $t { 1 }
            fn two() -> $t { 2 }
            fn my_wrapping_sub(&self, other: $t) -> $t { self.wrapping_sub(other) }
            fn modinv(&self, modulo: $t) -> Option<$t> {
                assert!(modulo >= 0);
                if modulo == 0 {
                    if self & 1 == 1 {
                        let mut out = *self;
                        for _ in 0..6 {
                            out = out.wrapping_mul((2 as $t).wrapping_sub(self.wrapping_mul(out)));
                        }
                        return Some(out);
                    } else {
                        return None;
                    }
                }
                let (g, x, _y) = egcd(*self, modulo);
                if g == 1 {
                    let out = if modulo != 0 { x % modulo } else { x };
                    Some(if out < 0 { out + modulo } else { out })
                } else {
                    None
                }
            }
            fn modadd(x: $t, y: $t, modulo: $t) -> $t {
                if modulo == 0 {
                    x.wrapping_add(y)
                } else {
                    Self::modsub(x, Self::modsub(0, y, modulo), modulo)
                }
            }
            fn modsub(x: $t, y: $t, modulo: $t) -> $t {
                debug_assert!(modulo >= 0);
                if modulo == 0 {
                    x.wrapping_sub(y)
                } else {
                    let (mut x, mut y) = (x % modulo, y % modulo);
                    if x < 0 { x += modulo; }
                    if y < 0 { y += modulo; }
                    let out = x - y;
                    if out < 0 { out + modulo } else { out }
                }
            }
            fn modmul(x: $t, y: $t, modulo: $t) -> $t {
                debug_assert!(modulo >= 0);
                if modulo == 0 {
                    x.wrapping_mul(y)
                } else if <$t>::BITS <= 16 {
                    ((x as i32) * (y as i32) % (modulo as i32)) as $t
                } else if <$t>::BITS <= 32 {
                    ((x as i64) * (y as i64) % (modulo as i64)) as $t
                } else if <$t>::BITS <= 64 {
                    ((x as i128) * (y as i128) % (modulo as i128)) as $t
                } else if <$t>::BITS <= 128 {
                    let mut x_tmp = x % modulo;
                    if x_tmp < 0 { x_tmp += modulo; }
                    let mut y_tmp = y % modulo;
                    if y_tmp < 0 { y_tmp += modulo; }
                    modmul::modmul128(x_tmp as u128, y_tmp as u128, modulo as u128) as $t
                } else {
                    panic!("Unsupported number of bits: {}", <$t>::BITS)
                }
            }
        }
    )* };
}
impl_mod_ops_signed!(i8, i16, i32, i64, i128, isize);

macro_rules! impl_mod_ops_unsigned {
    ($($t:ty),*) => { $(
        impl ModOps<$t> for $t {
            fn zero() -> $t { 0 }
            fn one() -> $t { 1 }
            fn two() -> $t { 2 }
            fn my_wrapping_sub(&self, other: $t) -> $t { self.wrapping_sub(other) }
            fn modadd(x: $t, y: $t, modulo: $t) -> $t {
                if modulo == 0 {
                    x.wrapping_add(y)
                } else {
                    Self::modsub(x, Self::modsub(0, y, modulo), modulo)
                }
            }
            fn modsub(x: $t, y: $t, modulo: $t) -> $t {
                if modulo == 0 {
                    x.wrapping_sub(y)
                } else {
                    let (x, y) = (x % modulo, y % modulo);
                    let (out, overflow) = x.overflowing_sub(y);
                    if overflow { out.wrapping_add(modulo) } else { out }
                }
            }
            fn modinv(&self, modulo: $t) -> Option<$t> {
                if modulo == 1 {
                    return None;
                }
                if modulo == 0 {
                    if self & 1 == 1 {
                        let mut out = *self;
                        for _ in 0..6 {
                            out = out.wrapping_mul(2.wrapping_sub(self.wrapping_mul(out)));
                        }
                        return Some(out);
                    } else {
                        return None;
                    }
                }
                let (mut a, mut b) = (*self, modulo);
                let mut c: [$t; 4] = if a > b {
                    (a, b) = (b, a);
                    [0, 1, 1, 0]
                } else {
                    [1, 0, 0, 1]
                }; // treat as a row-major 2x2 matrix
                loop {
                    if a == 0 {
                        if b == 1 {
                            break Some(c[1]);
                        } else if b == modulo - 1 {
                            break Some(modsub(0, c[1], modulo));
                        } else {
                            break None;
                        }
                    }
                    let (q, r) = (b / a, b % a);
                    (a, b) = (r, a);
                    c = [modsub(c[1], modmul(q, c[0], modulo), modulo), c[0], modsub(c[3], modmul(q, c[2], modulo), modulo), c[2]];
                }
            }
            fn modmul(x: $t, y: $t, modulo: $t) -> $t {
                if modulo == 0 {
                    x.wrapping_mul(y)
                } else if <$t>::BITS <= 16 {
                    ((x as u32) * (y as u32) % (modulo as u32)) as $t
                } else if <$t>::BITS <= 32 {
                    ((x as u64) * (y as u64) % (modulo as u64)) as $t
                } else if <$t>::BITS <= 64 {
                    ((x as u128) * (y as u128) % (modulo as u128)) as $t
                } else if <$t>::BITS <= 128 {
                    modmul::modmul128(x as u128, y as u128, modulo as u128) as $t
                } else {
                    panic!("Unsupported number of bits: {}", <$t>::BITS)
                }
            }
        }
    )* };
}
impl_mod_ops_unsigned!(u8, u16, u32, u64, u128, usize);

/// Computes the modular addition `x + y`.
///
/// This function will panic if `modulo` is negative.
pub fn modadd<T: ModOps<T>>(x: T, y: T, modulo: T) -> T {
    T::modadd(x, y, modulo)
}

/// Computes the modular subtraction `x - y`.
///
/// This function will panic if `modulo` is negative.
pub fn modsub<T: ModOps<T>>(x: T, y: T, modulo: T) -> T {
    T::modsub(x, y, modulo)
}

/// Computes the modular multiplication `x * y`.
///
/// This function will panic if `modulo` is negative.
pub fn modmul<T: ModOps<T>>(x: T, y: T, modulo: T) -> T {
    T::modmul(x, y, modulo)
}

/// Computes the inverse of `x` mod `modulo`, if it exists.
/// Returns `None` if the inverse does not exist.
///
/// This function will panic if `modulo` is negative.
pub fn modinv<T: ModOps<T>>(x: T, modulo: T) -> Option<T> {
    x.modinv(modulo)
}

/// Computes `base ** exponent` mod `modulo` in `O(lg exponent)` time.
/// Returns `None` if the exponent is negative and `base` is not invertible mod `modulo`.
///
/// This function will panic if `modulo` is negative.
pub fn modpow<T: ModOps<T>>(mut base: T, mut exponent: T, modulo: T) -> Option<T> {
    assert!(modulo >= T::zero());
    let mut out = T::one();
    if exponent < T::zero() {
        /* check for invertibility of base with respect to mod modulo */
        if let Some(x) = modinv(base, modulo) {
            base = x;
        } else {
            return None;
        }
        exponent = T::zero()
            .my_wrapping_sub(exponent)
            .my_wrapping_sub(T::one());
        if modulo != T::zero() {
            out = base % modulo;
        }
    }
    let mut base_pow = if modulo != T::zero() {
        base % modulo
    } else {
        base
    };
    while exponent > T::zero() {
        if (exponent % T::two()) != T::zero() {
            out = T::modmul(out, base_pow, modulo);
        }
        base_pow = T::modmul(base_pow, base_pow, modulo);
        exponent = exponent / T::two();
    }
    Some(out)
}

/// Computes `x * y^-1 mod modulo`.
/// If `y^{-1} mod modulo` does not exist, the result is `None`.
/// If it exists, the result is returned.
///
/// This function will panic if `modulo` is negative.
pub fn moddiv<T: ModOps<T>>(x: T, y: T, modulo: T) -> Option<T> {
    modinv(y, modulo).map(|yinv| modmul(x, yinv, modulo))
}

/// Computes the modular addition `x + y`.
///
/// This function will panic if `modulo` is zero or negative.
pub fn checked_modadd<T: ModOps<T>>(x: T, y: T, modulo: T) -> T {
    assert!(modulo != T::zero());
    modadd(x, y, modulo)
}

/// Computes the modular subtraction `x - y`.
///
/// This function will panic if `modulo` is zero or negative.
pub fn checked_modsub<T: ModOps<T>>(x: T, y: T, modulo: T) -> T {
    assert!(modulo != T::zero());
    modsub(x, y, modulo)
}

/// Computes the modular multiplication `x * y`.
///
/// This function will panic if `modulo` is zero or negative.
pub fn checked_modmul<T: ModOps<T>>(x: T, y: T, modulo: T) -> T {
    assert!(modulo != T::zero());
    modmul(x, y, modulo)
}

/// Computes the inverse of `x` mod `modulo`, if it exists.
/// Returns `None` if the inverse does not exist.
///
/// This function will panic if `modulo` is zero or negative.
pub fn checked_modinv<T: ModOps<T>>(x: T, modulo: T) -> Option<T> {
    assert!(modulo != T::zero());
    modinv(x, modulo)
}

/// Computes `base ** exponent` mod `modulo` in `O(lg exponent)` time.
/// Returns `None` if the exponent is negative and `base` is not invertible mod `modulo`.
///
/// This function will panic if `modulo` is zero or negative.
pub fn checked_modpow<T: ModOps<T>>(base: T, exponent: T, modulo: T) -> Option<T> {
    assert!(modulo != T::zero());
    modpow(base, exponent, modulo)
}

/// Computes `x * y^-1 mod modulo`.
/// If `y^{-1} mod modulo` does not exist, the result is `None`.
/// If it exists, the result is returned.
///
/// This function will panic if `modulo` is zero or negative.
pub fn checked_moddiv<T: ModOps<T>>(x: T, y: T, modulo: T) -> Option<T> {
    assert!(modulo != T::zero());
    moddiv(x, y, modulo)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn gcd_returns_gcd() {
        assert_eq!(32, gcd::<u32>(128736, 72352));
    }

    #[test]
    fn lcm_returns_lcm() {
        assert_eq!(249318024, lcm::<u32>(234984, 12732));
    }

    #[test]
    fn gcd_u64_returns_gcd() {
        assert_eq!(6, gcd::<u64>(2763162631554, 1276921782234));
    }

    #[test]
    fn lcm_u64_returns_lcm() {
        assert_eq!(4264971179382324, lcm::<u64>(273652348, 62341452));
    }

    #[test]
    fn egcd_returns_gcd() {
        let a: i128 = 823327498201749212;
        let b: i128 = 734892783927949214;
        let (g, s, t) = egcd(a, b);
        let normal = gcd(a as u64, b as u64) as i128;
        assert_eq!(normal, g);
        assert_eq!(a * s + b * t, g);
    }

    #[test]
    fn modadd_returns_modadd() {
        assert_eq!(6i64, modadd(-3, -5, 7));
        assert_eq!(6u64, modadd(3, 10, 7));
        assert_eq!(
            2_147_483_643i32,
            modadd(-1_073_741_824, -1_073_741_827, 2_147_483_647)
        );
        assert_eq!(
            2_147_483_643i64,
            modadd(-1_073_741_824, -1_073_741_827, 2_147_483_647)
        );
        assert_eq!(4i32, modadd(1_073_741_824, 1_073_741_827, 2_147_483_647));
        assert_eq!(4i64, modadd(1_073_741_824, 1_073_741_827, 2_147_483_647));
        assert_eq!(
            2_147_483_645i32,
            modadd(-2_147_483_648, -2_147_483_648, 2_147_483_647)
        );
        assert_eq!(
            2_147_483_645i64,
            modadd(-2_147_483_648, -2_147_483_648, 2_147_483_647)
        );
        assert_eq!(8, modadd(5, 3, 0i64));
        assert_eq!(8, modadd(5, 3, 0u64));
    }

    #[test]
    fn modsub_returns_modsub() {
        assert_eq!(6i64, modsub(-3, 5, 7));
        assert_eq!(6u64, modsub(3, 4, 7));
        assert_eq!(
            2_147_483_643i32,
            modsub(-1_073_741_824, 1_073_741_827, 2_147_483_647)
        );
        assert_eq!(
            2_147_483_643i64,
            modsub(-1_073_741_824, 1_073_741_827, 2_147_483_647)
        );
        assert_eq!(4i32, modsub(1_073_741_824, -1_073_741_827, 2_147_483_647));
        assert_eq!(4i64, modsub(1_073_741_824, -1_073_741_827, 2_147_483_647));
        assert_eq!(4i32, modsub(-2_147_483_648, -5, 2_147_483_647));
        assert_eq!(4i64, modsub(-2_147_483_648, -5, 2_147_483_647));
        assert_eq!(2, modsub(5, 3, 0i64));
        assert_eq!(2, modsub(5, 3, 0u64));
    }

    #[test]
    fn modinv_returns_modinv() {
        assert_eq!(None, modinv(4i64, 16i64));
        assert_eq!(None, modinv(301i64, 7i64));
        assert_eq!(Some(4i64), modinv(3i64, 11i64));
        assert_eq!(Some(4u64), modinv(3u64, 11u64));
        let p = 0u64.wrapping_sub((1u64 << 32) - 1);
        assert_eq!(Some((p + 1) / 2), modinv(2u64, p));
        assert_eq!(Some(9999999966u64), modinv(9999999966u64, 9999999967u64));
        assert_eq!(Some(12297829382473034411u64), modinv(3, 0u64));
        assert_eq!(
            Some(102271515336455672821735593367980000139i128),
            modinv(19491, 0i128)
        );
        assert_eq!(
            Some(19491i128),
            modinv(102271515336455672821735593367980000139i128, 0i128)
        );
    }

    #[test]
    fn modpow_returns_modpow() {
        assert_eq!(Some(0i64), modpow(4i64, 4i64, 16i64));
        assert_eq!(None, modpow(4i64, -4i64, 16i64));
        assert_eq!(Some(1i64), modpow(2i64, 1_000_000_006i64, 1_000_000_007i64));
        assert_eq!(Some(1u64), modpow(2u64, 1_000_000_006u64, 1_000_000_007u64));
        assert_eq!(Some(0u64), modpow(2, 64, 0u64));
        let p = 0u64.wrapping_sub((1u64 << 32) - 1);
        assert_eq!(Some(1u64), modpow(2u64, p - 1, p));
        let p128 = 0u128.wrapping_sub(159);
        assert_eq!(Some(1u128), modpow(2u128, p128 - 1, p128));
    }

    #[test]
    fn moddiv_returns_moddiv() {
        assert_eq!(None, moddiv(7i64, 2i64, 24i64));
        assert_eq!(Some(14i64), moddiv(2i64, 7i64, 24i64));
        assert_eq!(Some(17361641481138401521u64), moddiv(1u64, 17u64, 0u64));
        assert_eq!(Some(3i64), moddiv(15i64, 5i64, 0i64));
    }
}
