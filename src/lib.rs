const DEFAULT_SHL_FIRST: bool = false;
const DEFAULT_SHL: u32 = 17;
const DEFAULT_SHR: u32 = 23;

/// A prng designed for autovectorized filling of buffers with random bits,
/// built from several lanes of individual modified 'xorshiftR+' prngs.
#[derive(Clone, Copy, Debug)]
pub struct XorshiftrWide<const LANES: usize = 16> {
    state: [[u64; LANES]; 2],
}
impl<const LANES: usize> XorshiftrWide<LANES> {
    /// Reseeds an existing prng instance, seeded with a source of randomness provided by the user.
    pub fn reseed(&mut self, mut provide_random_u64: impl FnMut() -> u64) {
        while {
            for arr in &mut self.state {
                for ptr in arr {
                    *ptr = provide_random_u64();
                }
            }
            // Check if any two states are the same
            // In the rare event that they are, we need to reseed
            let mut need_to_reseed = false;
            for i in 0..LANES {
                need_to_reseed |= self.state[0][i] == self.state[1][i];
            }
            for left in 0..LANES {
                for right in (left + 1)..LANES {
                    let top_same = self.state[0][left] == self.state[0][right];
                    let bottom_same = self.state[1][left] == self.state[1][right];
                    let either_same = top_same | bottom_same;
                    need_to_reseed |= either_same;
                }
            }
            need_to_reseed
        } {
            cold();
        }
    }
    /// Creates a new prng instance, seeded with a source of randomness provided by the user.
    pub fn new(provide_random_u64: impl FnMut() -> u64) -> Self {
        let mut new_state = Self {
            state: [[0; LANES]; 2],
        };
        new_state.reseed(provide_random_u64);
        new_state
    }
    /// Core function to fill a buffer with random data.
    /// This function is generic over the shift parameters to allow for
    /// different configurations of the algorithm.
    #[inline(never)]
    fn fill_core<const SHL_FIRST: bool, const SHL_BY: u32, const SHR_BY: u32>(
        &mut self,
        buffer: &mut [u64],
    ) {
        let mut exact_width_chunks = buffer.chunks_exact_mut(LANES);
        for chunk in exact_width_chunks.by_ref() {
            let chunk_as_array: &mut [u64; LANES] = unsafe { chunk.try_into().unwrap_unchecked() };
            for i in 0..LANES {
                let mut x = self.state[0][i];
                let y = self.state[1][i];
                self.state[0][i] = y;
                if const { SHL_FIRST } {
                    x ^= x << SHL_BY;
                    x ^= x >> SHR_BY;
                } else {
                    x ^= x >> SHR_BY;
                    x ^= x << SHL_BY;
                }
                self.state[1][i] = x.wrapping_add(y);
                chunk_as_array[i] = x;
            }
        }
        let tail = exact_width_chunks.into_remainder();
        if !tail.is_empty() {
            cold();
            let randomized_buf: [u64; LANES] = std::array::from_fn(|i| {
                let mut x = self.state[0][i];
                let y = self.state[1][i];
                self.state[0][i] = y;
                if const { SHL_FIRST } {
                    x ^= x << SHL_BY;
                    x ^= x >> SHR_BY;
                } else {
                    x ^= x >> SHR_BY;
                    x ^= x << SHL_BY;
                }
                self.state[1][i] = x.wrapping_add(y);
                x
            });
            let qty_to_copy = tail.len();
            let random_bits_to_copy = &randomized_buf[..qty_to_copy];
            tail.copy_from_slice(random_bits_to_copy);
        }
    }
    /// Fills a slice of u64 with random data.
    #[inline]
    pub fn fill_u64_buffer(&mut self, destination_buffer: &mut [u64]) {
        self.fill_core::<DEFAULT_SHL_FIRST, DEFAULT_SHL, DEFAULT_SHR>(destination_buffer);
    }
}

pub(crate) use private_utils::*;
pub(crate) mod private_utils {
    #[inline(always)]
    #[cold]
    pub(crate) fn cold() {}
}
