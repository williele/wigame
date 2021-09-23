use std::alloc::Layout;

use bit_set::BitSet;

use crate::{
    blob_vec::BlobVec,
    sparse_set::{SparseArray, SparseIndex},
};

#[derive(Debug)]
pub struct BlobSparseSet<I> {
    dense: BlobVec,
    incides: Vec<I>,
    sparse: SparseArray<I, usize>,
}

impl<I: SparseIndex> BlobSparseSet<I> {
    unsafe fn drop_ptr<T>(ptr: *mut u8) {
        ptr.cast::<T>().drop_in_place()
    }

    pub fn of<T: 'static>(capacity: usize) -> Self {
        let layout = Layout::new::<T>();
        BlobSparseSet {
            dense: BlobVec::new(layout, Self::drop_ptr::<T>, capacity),
            incides: Vec::with_capacity(capacity),
            sparse: SparseArray::with_capacity(capacity),
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

    pub fn get_ptr(&self, index: I) -> Option<*mut u8> {
        self.sparse
            .get(index)
            .map(|dense_index| unsafe { self.dense.get_unchecked(*dense_index) })
    }

    pub fn get<T>(&self, index: I) -> Option<&T> {
        self.get_ptr(index)
            .and_then(|ptr| unsafe { ptr.cast::<T>().as_ref() })
    }

    pub fn get_mut<T>(&self, index: I) -> Option<&mut T> {
        self.get_ptr(index)
            .and_then(|ptr| unsafe { ptr.cast::<T>().as_mut() })
    }

    pub unsafe fn insert(&mut self, index: I, value: *mut u8) {
        if let Some(&dense_index) = self.sparse.get(index.clone()) {
            self.dense.replace_unchecked(dense_index, value);
        } else {
            let dense_index = self.dense.push_uninit();
            self.sparse.insert(index.clone(), dense_index);
            self.incides.push(index);
            self.dense.initialize_unchecked(dense_index, value);
        }
    }

    pub unsafe fn insert_type<T>(&mut self, index: I, mut value: T) {
        let ptr = (&mut value as *mut T).cast::<u8>();
        self.insert(index, ptr);
        std::mem::forget(value)
    }

    pub fn remove_and_forget(&mut self, index: I) -> Option<*mut u8> {
        self.sparse.remove(index).map(|dense_index| {
            let is_last = dense_index == self.dense.len() - 1;
            let value = unsafe { self.dense.swap_remove_and_forget_unchecked(dense_index) };
            self.incides.swap_remove(dense_index);
            if !is_last {
                let swapped_index = self.incides[dense_index].clone();
                *self.sparse.get_mut(swapped_index).unwrap() = dense_index;
            }
            value
        })
    }

    pub fn remove(&mut self, index: I) -> bool {
        if let Some(dense_index) = self.sparse.remove(index) {
            let is_last = dense_index == self.dense.len() - 1;
            unsafe { self.dense.swap_remove_and_drop_unchecked(dense_index) };
            self.incides.swap_remove(dense_index);
            if !is_last {
                let swapped_index = self.incides[dense_index].clone();
                *self.sparse.get_mut(swapped_index).unwrap() = dense_index;
            }
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use parking_lot::RwLock;

    use super::*;

    #[derive(Debug)]
    struct Demo {
        a: i32,
    }

    impl SparseIndex for u32 {
        fn get_sparse_index(&self) -> usize {
            *self as usize
        }
    }

    #[test]
    fn bloc_vec() {
        let mut bs = BlobSparseSet::<u32>::of::<RwLock<Demo>>(0);

        unsafe {
            bs.insert_type(0, RwLock::new(Demo { a: 0 }));
            bs.insert_type(1, RwLock::new(Demo { a: 1 }));
            bs.insert_type(2, RwLock::new(Demo { a: 2 }));

            println!("{:?}", bs.get::<RwLock<Demo>>(0));
            println!("{:?}", bs.get::<RwLock<Demo>>(1));
            println!("{:?}", bs.get::<RwLock<Demo>>(2));
        }
    }
}
