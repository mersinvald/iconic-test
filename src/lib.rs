pub mod optimized_vec;
use std::cmp::min;

pub type Price = i32;
pub type Size = u32;
pub type Meta = u128;

pub type Container<T> = optimized_vec::OptimizedVec<T>;

pub type StoreInner = Container<(Price, Container<(Size, Meta)>)>;
pub type SizeMetaList = Container<(Size, Meta)>;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Store {
    inner: StoreInner,
}

impl Store {
    pub fn new() -> Self {
        Store { inner: Container::new() }
    }

    // O(log(n))
    pub fn insert(&mut self, elem: (Price, Container<(Size, Meta)>)) {
        let idx = match self.find_price_idx(elem.0) {
            | Ok(idx)
            | Err(idx) => idx
        };
        self.inner.insert(idx, elem);
    }

    // O(log(n))
    pub fn append_size_and_meta_to_price(&mut self, price: Price, meta: (Size, Meta)) {
        let idx = match self.find_price_idx(price) {
            Ok(idx) => idx,
            Err(_) => panic!("price {} does not exist in Store", price),
        };
        self.inner[idx].1.push(meta);
    }

    // O(n * m)
    // n -- number of prices
    // m -- number of metadata chunks attached to each price
    pub fn split(&mut self, max_price: Price, mut requested_size: Size) -> Store {
        // Get an length of part of the prices array with prices < max_price
        let mut upper_bound = match self.find_price_idx(max_price) {
            Ok(idx) => {
                // if found, there might be several equal prices, need the last one
                self.inner[idx..].iter()
                    .enumerate()
                    .take_while(|(_, (price, _))| *price <= max_price)
                    .last()
                    .map(|(idx, _)| idx + 1)
                    .unwrap()
            },
            Err(idx) => min(idx, self.inner.len()),
        };

        let mut new = Container::with_capacity(self.inner.len());

        let mut idx = 0;
        while idx < upper_bound && requested_size != 0 {
            let price = self.inner[idx].0;
            let (drained_sizes, new_sizes, left_requested) = self.split_sizes(idx, requested_size);

            if !new_sizes.is_empty() {
                new.push((price, new_sizes));
            }

            requested_size = left_requested;

            if drained_sizes {
                self.inner.remove(idx);
                upper_bound -= 1;
            } else {
                idx += 1;
            }
        }

        Store::from(new)
    }

    /// Returns if sizes of this sizes list was drained, produced sizes list
    /// and how much more volume is left to move to the new vector
    fn split_sizes(&mut self, price_idx: usize, mut requested: u32) -> (bool, SizeMetaList, u32) {
        let (_, sizes) = &mut self.inner[price_idx];
        let mut new_sizes = Container::with_capacity(sizes.len());

        let mut idx = 0;
        while idx < sizes.len() && requested != 0 {
            let should_remove = {
                let (size, meta) = &mut sizes[idx];
                let new_size = min(requested, *size);
                *size -= new_size;
                requested -= new_size;
                new_sizes.push((new_size, *meta));
                *size == 0
            };

            if should_remove {
                sizes.remove(idx);
            } else {
                idx += 1;
            }
        }

        (sizes.is_empty(), new_sizes, requested)
    }

    // O(log(n))
    fn find_price_idx(&self, price: Price) -> Result<usize, usize> {
        self.inner.binary_search_by_key(&price, |elem| elem.0)
    }
}

impl From<StoreInner> for Store {
    fn from(inner: StoreInner) -> Self {
        Store {
            inner
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_split((orig_result, new_result): (Store, Store), (orig_expected, new_expected): (Store, Store)) {
        assert_eq!(orig_expected, orig_result);
        assert_eq!(new_expected, new_result);
    }

    fn make_initial() -> Store {
        Store::from(Container::from(vec![
            (5, Container::from(vec![(10, 0), (20, 0)])),
            (7, Container::from(vec![(10, 0), (20, 0)])),
        ]))
    }

    #[test]
    fn split_price_8_size_35() {
        let orig_expected = Container::from(vec![
            (7, vec![(5, 0), (20, 0)].into())
        ]).into();
        let new_expected = Container::from(vec![
            (5, vec![(10, 0), (20, 0)].into()),
            (7, vec![(5, 0)].into())
        ]).into();

        let mut orig = make_initial();
        let new = orig.split(8, 35);
        check_split((orig, new), (orig_expected, new_expected));
    }

    #[test]
    fn split_price_6_size_15() {
        let orig_expected = Container::from(vec![
            (5, vec![(15, 0)].into()),
            (7, vec![(10, 0), (20, 0)].into())
        ]).into();
        let new_expected = Container::from(vec![
            (5, vec![(10, 0), (5, 0)].into()),
        ]).into();

        let mut orig = make_initial();
        let new = orig.split(6, 15);
        check_split((orig, new), (orig_expected, new_expected));
    }

    #[test]
    fn split_price_8_size_15() {
        let orig_expected = Container::from(vec![
            (5, vec![(15, 0)].into()),
            (7, vec![(10, 0), (20, 0)].into())
        ]).into();
        let new_expected = Container::from(vec![
            (5, vec![(10, 0), (5, 0)].into()),
        ]).into();

        let mut orig = make_initial();
        let new = orig.split(8, 15);
        check_split((orig, new), (orig_expected, new_expected));
    }

}

