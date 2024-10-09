// Currently, no work has been done for preventing correctness issues from overflowing and
// performance problems. This should only be considered as a sort of skeleton code.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Modulo(pub u64, u128);

impl Modulo {
    // NOTE: Unlike `static_modint`, the arguments of each function may be larger than or equal to
    // `self.0`, the modulus.

    // TODO: Investigate the plausibility (performance, overflow) of Lemire's algorithm
    // https://snippets.kiwiyou.dev/cdiv
    // https://lemire.me/blog/2019/02/20/more-fun-with-fast-remainders-when-the-divisor-is-a-constant/

    /// Creates a new instance of `Modulo` with mod `m`.
    pub fn new(m: u64) -> Self {
        debug_assert!(m >= 2);
        Self(m, (!0u128 / m as u128).wrapping_add(1))
    }

    fn multop(a: u128, b: u64) -> u64 {
        let mut bottom = (a as u64 as u128) * b as u128;
        bottom >>= 64;
        let top = (a >> 64) * b as u128;
        ((bottom + top) >> 64) as u64
    }

    /// Returns `a % m`.
    pub fn rem(&self, a: u64) -> u64 {
        let low = self.1.wrapping_mul(a as u128);
        Self::multop(low, self.0)
    }

    /// Returns `floor(a / m)`.
    pub fn quot(&self, a: u64) -> u64 {
        Self::multop(self.1, a)
    }

    /// Checks if `a` is divisible by `m`.
    pub fn is_divisible(&self, a: u64) -> bool {
        self.1.wrapping_mul(a as u128) <= self.1.wrapping_sub(1)
    }

    /// Returns `(a + b) % m`.
    /// Caution: Make sure that `a < m` and `b < m` where `m == self.0`.
    pub fn add(&self, a: u64, b: u64) -> u64 {
        let m = self.0;
        let (val, carry) = a.overflowing_add(b);
        if val >= m || carry {
            val.wrapping_sub(m)
        } else {
            val
        }
    }

    /// Returns `(a - b) % m`.
    /// Caution: Make sure that `a < m` and `b < m` where `m == self.0`.
    pub fn sub(&self, a: u64, b: u64) -> u64 {
        let m = self.0;
        let nb = if b == 0 { 0 } else { m - b };
        self.add(a, nb)
    }

    /// Returns `a * b / m` and `a * b % m`.
    pub fn mulmod(&self, a: u64, b: u64) -> (u64, u64) {
        // https://snippets.kiwiyou.dev/mulmod
        let c = self.0;
        let (quot, rem);
        unsafe {
            core::arch::asm!(
                "mul {b}",
                "div {c}",
                inout("rax") a => quot,
                b = in(reg) b,
                c = in(reg) c,
                out("rdx") rem,
                options(nomem, pure)
            )
        };
        (quot, rem)
    }

    /// Returns `a * b % m`.
    pub fn mul(&self, a: u64, b: u64) -> u64 {
        self.mulmod(a, b).1
    }

    /// Returns `a / b` under modulo `m`.
    pub fn div(&self, a: u64, b: u64) -> u64 {
        let inv = super::egcd(b as i128, self.0 as i128).1;
        let inv = self.rem((self.0 as i128 + inv) as u64);
        self.mul(a, inv)
    }
}
