use super::Pos;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
};

/// Lazy contiguous storage for bounded mesher neighborhoods. Unusual sampling
/// outside the halo remains valid, including legacy displacement and large rays.
pub(super) struct GridCache<T: Copy> {
    min: Pos,
    size: [usize; 3],
    cells: Vec<Cell<Option<T>>>,
    outside: RefCell<HashMap<Pos, T>>,
}
impl<T: Copy> GridCache<T> {
    pub fn new(min: Pos, max: Pos, halo: i32) -> Self {
        let low = min.map(|v| v.saturating_sub(halo));
        let high = max.map(|v| v.saturating_add(halo));
        let mut size =
            std::array::from_fn(|i| (i64::from(high[i]) - i64::from(low[i])).max(0) as usize);
        let count = size.iter().try_fold(1usize, |a, b| a.checked_mul(*b));
        let count = match count {
            Some(n) if n <= 262_144 => n,
            _ => {
                size = [0; 3];
                0
            }
        };
        Self {
            min: low,
            size,
            cells: vec![Cell::new(None); count],
            outside: RefCell::new(HashMap::new()),
        }
    }
    pub fn get(&self, p: Pos, value: impl FnOnce() -> T) -> T {
        let q: [i64; 3] = std::array::from_fn(|i| i64::from(p[i]) - i64::from(self.min[i]));
        if (0..3).all(|i| q[i] >= 0 && q[i] < self.size[i] as i64) {
            let index =
                (q[2] as usize * self.size[1] + q[1] as usize) * self.size[0] + q[0] as usize;
            let cell = &self.cells[index];
            if let Some(v) = cell.get() {
                return v;
            }
            let v = value();
            cell.set(Some(v));
            v
        } else {
            *self.outside.borrow_mut().entry(p).or_insert_with(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cached_values_include_halo_and_unbounded_fallback() {
        let grid = GridCache::new([-2; 3], [3; 3], 2);
        for p in [[-4; 3], [4; 3], [0; 3], [99; 3]] {
            assert_eq!(grid.get(p, || p), p);
            assert_eq!(grid.get(p, || panic!("sample repeated")), p);
        }
    }
    #[test]
    fn excessive_or_inverted_bounds_use_sparse_storage() {
        for (min, max) in [([i32::MIN; 3], [i32::MAX; 3]), ([9; 3], [-9; 3])] {
            let grid = GridCache::new(min, max, 2);
            assert!(grid.cells.is_empty());
            assert_eq!(grid.get([0; 3], || 7), 7);
        }
    }
}
