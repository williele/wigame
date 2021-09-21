use std::marker::PhantomData;

use bit_set::BitSet;

pub trait SparseIndex: Clone {
    fn get_sparse_index(&self) -> usize;
}

#[derive(Debug)]
pub struct SparseArray<I, V = I> {
    values: Vec<Option<V>>,
    bitset: BitSet,
    _phamtom: PhantomData<I>,
}

impl<I: SparseIndex, V> Default for SparseArray<I, V> {
    fn default() -> Self {
        SparseArray {
            values: Vec::new(),
            bitset: BitSet::new(),
            _phamtom: Default::default(),
        }
    }
}

impl<I: SparseIndex, V> SparseArray<I, V> {
    pub fn with_capacity(capacity: usize) -> Self {
        SparseArray {
            values: Vec::with_capacity(capacity),
            bitset: BitSet::with_capacity(capacity),
            _phamtom: Default::default(),
        }
    }

    #[inline]
    pub fn bitset(&self) -> &BitSet {
        &self.bitset
    }

    #[inline]
    pub fn has(&self, index: I) -> bool {
        let index = index.get_sparse_index();
        self.bitset.contains(index)
    }

    #[inline]
    pub fn get(&self, index: I) -> Option<&V> {
        let index = index.get_sparse_index();
        self.values.get(index).map(|v| v.as_ref()).unwrap_or(None)
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, index: I) -> &V {
        let index = index.get_sparse_index();
        self.values.get_unchecked(index).as_ref().unwrap()
    }

    #[inline]
    pub fn get_mut(&mut self, index: I) -> Option<&mut V> {
        let index = index.get_sparse_index();
        self.values
            .get_mut(index)
            .map(|v| v.as_mut())
            .unwrap_or(None)
    }

    #[inline]
    pub fn insert(&mut self, index: I, value: V) {
        let index = index.get_sparse_index();
        if self.values.len() <= index {
            self.values.resize_with(index + 1, || None);
        }
        self.values[index] = Some(value);
        self.bitset.insert(index);
    }

    #[inline]
    pub fn remove(&mut self, index: I) -> Option<V> {
        let index = index.get_sparse_index();
        self.bitset.remove(index);
        self.values.get_mut(index).and_then(|value| value.take())
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }
}

#[derive(Debug)]
pub struct SparseSet<I, V = I> {
    dense: Vec<V>,
    indices: Vec<I>,
    sparse: SparseArray<I, usize>,
}

impl<I: SparseIndex, V> Default for SparseSet<I, V> {
    fn default() -> Self {
        SparseSet {
            dense: Vec::new(),
            indices: Vec::new(),
            sparse: SparseArray::default(),
        }
    }
}

impl<I: SparseIndex, V> SparseSet<I, V> {
    pub fn with_capacity(capacity: usize) -> Self {
        SparseSet {
            dense: Vec::with_capacity(capacity),
            indices: Vec::with_capacity(capacity),
            sparse: Default::default(),
        }
    }

    #[inline]
    pub fn bitset(&self) -> &BitSet {
        self.sparse.bitset()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.dense.capacity()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.dense.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.dense.len() == 0
    }

    #[inline]
    pub fn has(&self, index: I) -> bool {
        self.sparse.has(index)
    }

    pub fn get(&self, index: I) -> Option<&V> {
        self.sparse
            .get(index)
            .map(|dense_index| unsafe { self.dense.get_unchecked(*dense_index) })
    }

    pub fn get_mut(&mut self, index: I) -> Option<&mut V> {
        let dense = &mut self.dense;
        self.sparse
            .get(index)
            .map(move |dense_index| unsafe { dense.get_unchecked_mut(*dense_index) })
    }

    pub fn get_unchecked(&self, index: I) -> &V {
        unsafe {
            let dense_index = self.sparse.get_unchecked(index);
            self.dense.get_unchecked(*dense_index)
        }
    }

    pub fn get_unchecked_mut(&mut self, index: I) -> &mut V {
        unsafe {
            let dense_index = self.sparse.get_unchecked(index);
            self.dense.get_unchecked_mut(*dense_index)
        }
    }

    pub fn insert(&mut self, index: I, value: V) {
        if let Some(&dense_index) = self.sparse.get(index.clone()) {
            unsafe { *self.dense.get_unchecked_mut(dense_index) = value }
        } else {
            self.sparse.insert(index.clone(), self.dense.len());
            self.indices.push(index);
            self.dense.push(value);
        }
    }
    pub fn remove(&mut self, index: I) -> Option<V> {
        self.sparse.remove(index).map(|dense_index| {
            let is_last = dense_index == self.dense.len() - 1;
            let value = self.dense.swap_remove(dense_index);
            self.indices.swap_remove(dense_index);
            if !is_last {
                let swapped_index = self.indices[dense_index].clone();
                *self.sparse.get_mut(swapped_index).unwrap() = dense_index;
            }
            value
        })
    }
}
