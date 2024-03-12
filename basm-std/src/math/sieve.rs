use alloc::vec;
use alloc::vec::Vec;

/// A dynamically growing linear sieve.
/// We ensure amortized O(1) runtime by growing in a similar way as `Vec<T>`
/// and by using a linear sieve algorithm.
/// The growth strategy is to increase the upper bound by 50% each time.
pub struct LinearSieve {
    upto: usize,  // == smallest_prime_factor.len() - 1
    smallest_prime_factor: Vec<usize>,
    primes: Vec<usize>,
    mu: Vec<i8>,
    phi: Vec<usize>,
    //d: Vec<usize>,
    //s: Vec<usize>
}

impl LinearSieve {
    /// Creates a new instance of the dynamically growing linear sieve.
    pub fn new() -> Self {
        Self {
            upto: 1,
            smallest_prime_factor: vec![0, 1],
            primes: vec![],
            mu: vec![0, 1],
            phi: vec![0, 1],
            //d: vec![0, 1],
            //s: vec![0, 1]
        }
    }

    /// Returns true if and only if `x` is a prime number.
    pub fn is_prime(&mut self, x: usize) -> bool {
        x > 1 && self.smallest_prime_factor(x) == x
    }

    /// Returns the smallest prime factor of `x` (e.g. returns `3` for `x = 75`).
    /// Returns `x` back if `x <= 1`.
    pub fn smallest_prime_factor(&mut self, x: usize) -> usize {
        self.ensure_upto(x);
        self.smallest_prime_factor[x]
    }

    /// Returns the `n`-th prime in 1-based index (e.g. returns `2` for `n = 1`).
    /// TODO: Why don't make it 0-based and just remove the assert?
    pub fn nth_prime(&mut self, n: usize) -> usize {
        assert!(n >= 1);
        while self.primes.len() < n {
            self.ensure_upto(self.upto + 1);
        }
        self.primes[n - 1]
    }

    /// Mobius function.
    pub fn mu(&mut self, x: usize) -> i8 {
        assert!(x >= 1);
        if self.mu.len() <= x {
            let upto = Self::next_len(self.mu.len() - 1, x);
            self.ensure_upto(upto);
            self.mu.resize(upto + 1, 0);
            for i in 2..=upto {
                if self.is_prime(i) {
                    self.mu[i] = -1;
                }
                let lp = self.smallest_prime_factor(i);
                for &p in self.primes.iter() {
                    if i * p > upto || p > lp {
                        break;
                    }
                    self.mu[i * p] = if lp == p { 0 } else { -self.mu[i] };
                }
            }
        }
        self.mu[x]
    }

    /// Euler's totient function.
    pub fn phi(&mut self, x: usize) -> usize {
        assert!(x >= 1);
        if self.phi.len() <= x {
            let upto = Self::next_len(self.phi.len() - 1, x);
            self.ensure_upto(upto);
            self.phi.resize(upto + 1, 0);
            for i in 2..=upto {
                if self.is_prime(i) {
                    self.phi[i] = i - 1;
                }
                let lp = self.smallest_prime_factor(i);
                for &p in self.primes.iter() {
                    if i * p > upto || p > lp {
                        break;
                    }
                    self.phi[i * p] = if lp == p {
                        self.phi[i] * p
                    } else {
                        self.phi[i] * (p - 1)
                    };
                }
            }
        }
        self.phi[x]
    }

    /// Number of positive divisors of x.
    /// Note: This function can be slow. Performance optimization will be done later.
    pub fn d(&mut self, mut x: usize) -> usize {
        x = Self::sanitize(x);
        let mut ans = 1;
        while x > 1 {
            let lp = self.smallest_prime_factor(x);
            let mut lp_cnt = 0;
            while x % lp == 0 {
                x /= lp;
                lp_cnt += 1;
            }
            ans *= lp_cnt + 1;
        }
        ans
    }

    /// Sum of positive divisors of x.
    /// Note: This function can be slow. Performance optimization will be done later.
    pub fn s(&mut self, mut x: usize) -> usize {
        x = Self::sanitize(x);
        let mut ans = 1;
        while x > 1 {
            let lp = self.smallest_prime_factor(x);
            let mut lp_mul = 1;
            while x % lp == 0 {
                x /= lp;
                lp_mul = lp_mul * lp + 1;
            }
            ans *= lp_mul;
        }
        ans
    }

    /// Returns the positive divisors of x, in ascending order.
    pub fn divisors(&mut self, mut x: usize) -> Vec<usize> {
        x = Self::sanitize(x);
        if x == 1 {
            vec![1]
        } else {
            let lp = self.smallest_prime_factor(x);
            let mut lp_cnt = 0;
            while x % lp == 0 {
                x /= lp;
                lp_cnt += 1;
            }
            let part = self.divisors(x);
            let mut out = vec![];
            for d in part {
                let mut d_times_lp_pow = d;
                for _ in 0..=lp_cnt {
                    out.push(d_times_lp_pow);
                    d_times_lp_pow *= lp;
                }
            }
            out.sort_unstable();
            out
        }
    }

    /// Ensures that every integer in [1, x] is precomputed.
    fn ensure_upto(&mut self, x: usize) {
        if x > self.upto {
            // (prev_upto, x] are the numbers which needs computation.
            // We already know everything about [1, prev_upto].
            let prev_upto = self.upto;
            // next_len below returns max(self.upto*3/2, x). This loop is executed only when
            // x > self.upto, so the new self.upto is always self.upto*3/2. This ensures
            // amortization, but the implementation for now is quite weird...
            self.upto = Self::next_len(self.upto, x);
            self.smallest_prime_factor.resize(self.upto + 1, 0);
            // self.primes.clear(); - We don't need to clear self.primes at all!
            for i in 2..=self.upto {
                if self.smallest_prime_factor[i] == 0 || self.smallest_prime_factor[i] == i {
                    if i > prev_upto {
                        self.primes.push(i);
                    }
                    self.smallest_prime_factor[i] = i;
                }
                let look_from = self
                    .primes
                    .partition_point(|&p| i.checked_mul(p).is_some_and(|v| v <= prev_upto));
                for &p in self.primes.iter().skip(look_from) {
                    if i * p > self.upto || p > self.smallest_prime_factor[i] {
                        break;
                    }
                    self.smallest_prime_factor[i * p] = p;
                }
            }
        }
    }

    /// Ensure that the value of `x` makes sense.
    /// All it does is just panicing if `x == 0`.
    ///
    /// # Why is sanitization here?
    /// The original implementation mainly used a sigend integer, so we had to make it to an
    /// absolute value before any calculation.
    /// Yes, I know that this explanation isn't sufficient enough, but it was there...
    ///
    /// # Panic
    /// Panics if `x = 0`.
    fn sanitize(x: usize) -> usize {
        // As x is usize, x < 0 is always false.
        // let out = if x < 0 { -x } else { x };
        let out = x;
        assert!(x > 0);
        out
    }

    fn next_len(cur_upto: usize, x: usize) -> usize {
        let out = cur_upto + (cur_upto >> 1) + 1;
        if x > out {
            x
        } else {
            out
        }
    }
}

impl Default for LinearSieve {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_is_prime() {
        let mut ls = LinearSieve::new();
        assert_eq!(false, ls.is_prime(0));
        assert_eq!(false, ls.is_prime(1));
        assert_eq!(true, ls.is_prime(2));
        assert_eq!(true, ls.is_prime(3));
        assert_eq!(false, ls.is_prime(20));
        assert_eq!(true, ls.is_prime(19));
    }

    #[test]
    fn check_mu() {
        let mut ls = LinearSieve::new();
        assert_eq!(1, ls.mu(1));
        assert_eq!(-1, ls.mu(2));
        assert_eq!(-1, ls.mu(3));
        assert_eq!(0, ls.mu(4));
        assert_eq!(1, ls.mu(6));
        assert_eq!(1, ls.mu(41879491));
        assert_eq!(-1, ls.mu(30));
        assert_eq!(0, ls.mu(20443543));
    }

    #[test]
    fn check_phi() {
        let mut ls = LinearSieve::new();
        assert_eq!(1, ls.phi(1));
        assert_eq!(1, ls.phi(2));
        assert_eq!(2, ls.phi(3));
        assert_eq!(12, ls.phi(36));
        assert_eq!(72, ls.phi(91));
        assert_eq!(1648512, ls.phi(5986008));
        assert_eq!(40, ls.phi(100));
        assert_eq!(115200, ls.phi(442800));
    }

    #[test]
    fn check_d_and_s() {
        let mut ls = LinearSieve::new();
        assert_eq!(9, ls.d(100));
        assert_eq!(217, ls.s(100));
    }

    #[test]
    fn check_divisors() {
        let mut ls = LinearSieve::new();
        assert_eq!(vec![1], ls.divisors(1));
        assert_eq!(vec![1, 2, 3, 4, 6, 12], ls.divisors(12));
        assert_eq!(vec![1, 2, 4, 5, 10, 20, 25, 50, 100], ls.divisors(100));
    }
}
