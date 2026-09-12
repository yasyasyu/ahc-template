use rand::{Error, RngCore};
use std::time::Instant;

const DEFAULT_SEED: u64 = 88172645463393265;

pub struct TimeKeeper {
    start: Instant,
    limit_sec: f64,
}

impl TimeKeeper {
    pub fn new(limit_sec: f64) -> Self {
        Self {
            start: Instant::now(),
            limit_sec,
        }
    }

    pub fn is_over(&self) -> bool {
        self.is_over_elapsed(self.elapsed_sec())
    }

    pub fn progress(&self) -> f64 {
        self.progress_from_elapsed(self.elapsed_sec())
    }

    pub fn remaining_sec(&self) -> f64 {
        (self.limit_sec - self.elapsed_sec()).max(0.0)
    }

    pub fn multi_start_deadlines(&self, parts: usize) -> MultiStartSchedule<'_> {
        MultiStartSchedule {
            timer: self,
            remaining_parts: parts,
        }
    }

    #[inline(always)]
    pub fn elapsed_sec(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }

    #[inline(always)]
    pub fn is_over_elapsed(&self, elapsed_sec: f64) -> bool {
        elapsed_sec >= self.limit_sec
    }

    #[inline(always)]
    pub fn progress_from_elapsed(&self, elapsed_sec: f64) -> f64 {
        (elapsed_sec / self.limit_sec).clamp(0.0, 1.0)
    }
}

pub struct MultiStartSchedule<'a> {
    timer: &'a TimeKeeper,
    remaining_parts: usize,
}

impl Iterator for MultiStartSchedule<'_> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_parts == 0 {
            return None;
        }

        let now = self.timer.elapsed_sec();
        let deadline = now + self.timer.remaining_sec() / self.remaining_parts as f64;
        self.remaining_parts -= 1;
        Some(deadline)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TimeBudget {
    total_limit_sec: f64,
    reserve_limit_sec: f64,
}

impl TimeBudget {
    pub fn new(total_limit_sec: f64, reserve_limit_sec: f64) -> Self {
        let total_limit_sec = total_limit_sec.max(0.0);
        let reserve_limit_sec = reserve_limit_sec.clamp(0.0, total_limit_sec);
        Self {
            total_limit_sec,
            reserve_limit_sec,
        }
    }

    pub fn main_limit_sec(self) -> f64 {
        (self.total_limit_sec - self.reserve_limit_sec).max(0.0)
    }

    pub fn reserve_limit_sec(self) -> f64 {
        self.reserve_limit_sec
    }

    pub fn main_timer(self) -> TimeKeeper {
        TimeKeeper::new(self.main_limit_sec())
    }

    pub fn reserve_timer(self) -> TimeKeeper {
        TimeKeeper::new(self.reserve_limit_sec())
    }
}

#[derive(Clone, Debug)]
pub struct Track<T: Clone>(Vec<(usize, T)>);

impl<T: Clone> Track<T> {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn push(&mut self, prev: usize, value: T) -> usize {
        self.0.push((prev, value));
        self.0.len() - 1
    }

    pub fn restore(&self, mut index: usize) -> Vec<T> {
        let mut restored = vec![];
        while index != !0 {
            let (prev, value) = self.0[index].clone();
            restored.push(value);
            index = prev;
        }
        restored.reverse();
        restored
    }
}

pub struct XorShift(u64);
impl XorShift {
    pub fn new(seed: u64) -> Self {
        Self(if seed == 0 { DEFAULT_SEED } else { seed })
    }

    #[inline(always)]
    pub fn random_usize(&mut self, upper_bound: usize) -> usize {
        assert!(upper_bound > 0);
        ((self.next() as u128 * upper_bound as u128) >> 64) as usize
    }

    pub fn shuffle<T>(&mut self, values: &mut [T]) {
        for index in (1..values.len()).rev() {
            let swap_index = self.random_usize(index + 1);
            values.swap(index, swap_index);
        }
    }

    #[inline(always)]
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}
impl RngCore for XorShift {
    #[inline(always)]
    fn next_u32(&mut self) -> u32 {
        self.next() as u32
    }

    #[inline(always)]
    fn next_u64(&mut self) -> u64 {
        self.next()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        rand_core::impls::fill_bytes_via_next(self, dest);
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}
