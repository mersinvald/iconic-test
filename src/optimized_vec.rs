use std::fmt::{self, Debug};
use std::ops::{Index, IndexMut};

#[derive(Clone)]
pub struct OptimizedVec<T> {
    first: usize,
    inner: Vec<T>
}

impl<T: Debug> Debug for OptimizedVec<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list()
            .entries(&self.inner[self.first..])
            .finish()
    }
}

impl<T: PartialEq> PartialEq for OptimizedVec<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner[self.first..]
            .eq(&other.inner[other.first..])
    }
}

impl<T> OptimizedVec<T> {
    pub fn new() -> Self {
        OptimizedVec {
            first: 0,
            inner: Vec::new()
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        OptimizedVec {
            first: 0,
            inner: Vec::with_capacity(cap)
        }
    }

    #[inline]
    pub fn push(&mut self, value: T) {
        self.inner.push(value)
    }

    #[inline]
    pub fn insert(&mut self, idx: usize, value: T) {
        // likely branch goes first
        if !self.inner.is_empty() {
            if idx == 0 && self.first != 0 {
                self.first -= 1;
                self.inner[self.first] = value;
            } else {
                self.inner.insert(idx, value);
            }
        } else {
            self.inner.insert(idx, value);
        }
    }

    #[inline]
    pub fn remove(&mut self, idx: usize) {
        assert!(self.len() != 0);

        // likely branch goes first
        if !self.inner.is_empty() {
            if idx == 0 {
                self.first += 1;
            } else {
                self.inner.remove(self.first + idx);
            }
        } else {
            self.inner.remove(idx);
        }
    }

    #[inline]
    pub fn get(&self, idx: usize) -> &T {
        let idx = idx + self.first;
        &self.inner[idx]
    }

    #[inline]
    pub fn get_mut(&mut self, idx: usize) -> &mut T {
        let idx = idx + self.first;
        &mut self.inner[idx]
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len() - self.first
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn binary_search_by_key<'a, B, F>(&'a self, b: &B, f: F) -> Result<usize, usize>
        where F: FnMut(&'a T) -> B,
              B: Ord
    {
        self.inner[self.first..].binary_search_by_key(b, f)
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.inner[self.first..].iter()
    }
}

impl<T> From<Vec<T>> for OptimizedVec<T> {
    fn from(inner: Vec<T>) -> Self {
        OptimizedVec {
            first: 0,
            inner
        }
    }
}

impl<T> Index<usize> for OptimizedVec<T> {
    type Output = T;
    #[inline]
    fn index(&self, idx: usize) -> &T {
        self.get(idx)
    }
}

impl<T> IndexMut<usize> for OptimizedVec<T> {
    #[inline]
    fn index_mut(&mut self, idx: usize) -> &mut T {
        self.get_mut(idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from() {
        let f = vec![1, 2, 3, 4, 5];
        let o = OptimizedVec::from(f.clone());
        assert_eq!(f.len(), o.len());
        assert_eq!(o.first, 0);
    }

    #[test]
    fn push_into_empty() {
        let mut o = OptimizedVec::new();
        assert_eq!(o.len(), 0);
        assert_eq!(o.first, 0);
        o.push(1);
        assert_eq!(o.len(), 1);
        assert_eq!(o.first, 0);
    }


    #[test]
    fn insert_into_empty() {
        let mut o = OptimizedVec::new();
        assert_eq!(o.len(), 0);
        assert_eq!(o.first, 0);
        o.insert(0, 1);
        assert_eq!(o.len(), 1);
        assert_eq!(o.first, 0);
    }

    #[test]
    fn insert_with_data() {
        let mut o = OptimizedVec::from(vec![1, 2, 3, 4, 5]);
        assert_eq!(o.len(), 5);
        assert_eq!(o.first, 0);
        o.insert(3, 3);
        assert_eq!(o.len(), 6);
        assert_eq!(o.first, 0);
        o.insert(0, 0);
        assert_eq!(o.len(), 7);
        assert_eq!(o.first, 0);
        o.insert(7, 7);
        assert_eq!(o.len(), 8);
        assert_eq!(o.first, 0);
    }

    #[test]
    fn insert_shifted_head() {
        let mut o = OptimizedVec::from(vec![1, 2, 3, 4, 5]);
        o.first = 2;
        assert_eq!(o.inner.len(), 5);
        assert_eq!(o.len(), 3);
        assert_eq!(o.first, 2);
        o.insert(3, 3);
        assert_eq!(o.inner.len(), 6);
        assert_eq!(o.len(), 4);
        assert_eq!(o.first, 2);
        o.insert(1, 1);
        assert_eq!(o.inner.len(), 7);
        assert_eq!(o.len(), 5);
        assert_eq!(o.first, 2);
        o.insert(0, 0);
        assert_eq!(o.inner.len(), 7);
        assert_eq!(o.len(), 6);
        assert_eq!(o.first, 1);
        o.insert(0, 0);
        assert_eq!(o.inner.len(), 7);
        assert_eq!(o.len(), 7);
        assert_eq!(o.first, 0);
    }

    #[test]
    #[should_panic]
    fn remove_empty() {
        let mut o = OptimizedVec::<i32>::new();
        o.remove(0)
    }

    #[test]
    fn remove_with_data() {
        let mut o = OptimizedVec::from(vec![1, 2, 3, 4, 5]);
        assert_eq!(o.inner.len(), 5);
        assert_eq!(o.len(), 5);
        assert_eq!(o.first, 0);
        o.remove(3);
        assert_eq!(o.inner.len(), 4);
        assert_eq!(o.len(), 4);
        assert_eq!(o.first, 0);
        o.remove(0);
        assert_eq!(o.inner.len(), 4);
        assert_eq!(o.len(), 3);
        assert_eq!(o.first, 1);
        o.remove(1);
        assert_eq!(o.inner.len(), 3);
        assert_eq!(o.len(), 2);
        assert_eq!(o.first, 1);
        o.remove(0);
        assert_eq!(o.inner.len(), 3);
        assert_eq!(o.len(), 1);
        assert_eq!(o.first, 2);

    }

    #[test]
    #[should_panic]
    fn remove_all_shifted() {
        let mut o = OptimizedVec::from(vec![1]);
        o.remove(0);
        assert_eq!(o.inner.len(), 1);
        assert_eq!(o.len(), 0);
        assert_eq!(o.first, 1);
        o.remove(0);
    }

    #[test]
    fn combined() {
        let mut o = OptimizedVec::new();
        o.push(1);
        assert_eq!(o.inner.len(), 1);
        assert_eq!(o.len(), 1);
        assert_eq!(o.first, 0);
        o.remove(0);
        assert_eq!(o.inner.len(), 1);
        assert_eq!(o.len(), 0);
        assert_eq!(o.first, 1);
        o.push(1);
        assert_eq!(o.inner.len(), 2);
        assert_eq!(o.len(), 1);
        assert_eq!(o.first, 1);
        o.insert(0, 0);
        assert_eq!(o.inner.len(), 2);
        assert_eq!(o.len(), 2);
        assert_eq!(o.first, 0);
    }


}
