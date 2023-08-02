pub struct ChunksExact<const N: usize, I> {
    iter: I,
}

impl<const N: usize, I, T> Iterator for ChunksExact<N, I>
where
    I: Iterator<Item = T>,
{
    type Item = [T; N];

    fn next(&mut self) -> Option<Self::Item> {
        assert_ne!(N, 0);

        let mut vec: Vec<T> = Vec::with_capacity(N);
        for _ in 0..N {
            match self.iter.next() {
                Some(item) => vec.push(item),
                None => return None,
            }
        }
        let ary: [T; N] = vec.try_into().unwrap_or_else(|v: Vec<T>| {
            panic!("Expected a Vec of length {} but it was {}", N, v.len())
        });
        Some(ary)
    }
}

pub(crate) trait ChunksExactIterator: Sized {
    /// Returns an iterator over `N` elements of the iterator at a time.
    ///
    /// This is a custom implementation of [`Iterator::array_chunks`] for stable
    /// Rust 1.66, but for simplicity without
    /// [`.into_remainder()`][std::iter::adapters::ArrayChunks].
    ///
    /// TODO: Remove after `iter_array_chunks` is stabilized.
    fn chunks_exact<const N: usize>(self) -> ChunksExact<N, Self> {
        assert!(N != 0, "chunk size must be non-zero");

        ChunksExact { iter: self }
    }
}

impl<I> ChunksExactIterator for I where I: Iterator {}

/// Point-free value inspection and modification.
// This is inspired by https://github.com/myrrlyn/tap/.
pub(crate) trait Tap: Sized {
    /// Mutable access to a value.
    #[inline(always)]
    fn tap_mut<F: FnOnce(&mut Self)>(mut self, f: F) -> Self {
        f(&mut self);
        self
    }

    /// Conditional mutable access to a value.
    #[inline(always)]
    fn tap_mut_if<F: FnOnce(&mut Self)>(mut self, cond: bool, f: F) -> Self {
        if cond {
            f(&mut self);
        }
        self
    }

    /// Conditionally pipes by a value and returns self.
    #[inline(always)]
    fn pipe_if<F: FnOnce(Self) -> Self>(self, cond: bool, f: F) -> Self {
        if cond {
            f(self)
        } else {
            self
        }
    }
}

impl<T> Tap for T where T: Sized {}

#[cfg(test)]
#[path = "std_ext.test.rs"]
mod test;
