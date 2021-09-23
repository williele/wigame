use std::{
    alloc::{handle_alloc_error, Layout},
    ptr::NonNull,
};

#[derive(Debug)]
pub struct BlobVec {
    layout: Layout,
    cap: usize,
    len: usize,
    ptr: NonNull<u8>,
    swap_ptr: NonNull<u8>,
    drop: unsafe fn(*mut u8),
}

impl BlobVec {
    pub fn new(layout: Layout, drop: unsafe fn(*mut u8), capacity: usize) -> BlobVec {
        if layout.size() == 0 {
            BlobVec {
                layout,
                cap: usize::MAX,
                len: 0,
                ptr: NonNull::dangling(),
                swap_ptr: NonNull::dangling(),
                drop,
            }
        } else {
            let swap_ptr = NonNull::new(unsafe { std::alloc::alloc(layout) })
                .unwrap_or_else(|| handle_alloc_error(layout));
            let mut vec = BlobVec {
                layout,
                cap: 0,
                len: 0,
                ptr: NonNull::dangling(),
                swap_ptr,
                drop,
            };
            vec.reserve_exact(capacity);
            vec
        }
    }

    #[inline]
    pub unsafe fn get_ptr(&self) -> NonNull<u8> {
        self.ptr
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> *mut u8 {
        self.get_ptr().as_ptr().add(index * self.layout.size())
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        let available_space = self.cap - self.len;
        if available_space < additional {
            self.grow_exact(additional - available_space);
        }
    }

    fn grow_exact(&mut self, increment: usize) {
        debug_assert!(self.layout.size() != 0);

        let new_capacity = self.cap + increment;
        let new_layout =
            Layout::from_size_align(self.layout.size() * new_capacity, self.layout.align())
                .expect("array layout should be valid");
        unsafe {
            let new_data = if self.cap == 0 {
                std::alloc::alloc(new_layout)
            } else {
                std::alloc::realloc(
                    self.get_ptr().as_ptr(),
                    Layout::from_size_align(self.layout.size() * self.cap, self.layout.align())
                        .expect("array layout should be valid"),
                    new_layout.size(),
                )
            };

            self.ptr = NonNull::new(new_data).unwrap_or_else(|| handle_alloc_error(new_layout));
        }
        self.cap = new_capacity;
    }

    #[inline]
    pub unsafe fn initialize_unchecked(&mut self, index: usize, value: *mut u8) {
        debug_assert!(index < self.len());
        let ptr = self.get_unchecked(index);
        std::ptr::copy_nonoverlapping(value, ptr, self.layout.size());
    }

    pub unsafe fn replace_unchecked(&mut self, index: usize, value: *mut u8) {
        debug_assert!(index < self.len());
        let ptr = self.get_unchecked(index);
        (self.drop)(ptr);
        std::ptr::copy_nonoverlapping(value, ptr, self.layout.size());
    }

    #[inline]
    pub unsafe fn push_uninit(&mut self) -> usize {
        self.reserve_exact(1);
        let index = self.len;
        self.len += 1;
        index
    }

    #[inline]
    pub unsafe fn swap_remove_and_forget_unchecked(&mut self, index: usize) -> *mut u8 {
        debug_assert!(index < self.len());
        let last = self.len - 1;
        let swap_scratch = self.swap_ptr.as_ptr();
        std::ptr::copy_nonoverlapping(self.get_unchecked(index), swap_scratch, self.layout.size());
        std::ptr::copy(
            self.get_unchecked(last),
            self.get_unchecked(index),
            self.layout.size(),
        );
        self.len -= 1;
        swap_scratch
    }

    #[inline]
    pub unsafe fn swap_remove_and_drop_unchecked(&mut self, index: usize) {
        debug_assert!(index < self.len());
        let value = self.swap_remove_and_forget_unchecked(index);
        (self.drop)(value)
    }

    pub fn clear(&mut self) {
        let len = self.len;
        self.len = 0;
        for i in 0..len {
            unsafe {
                let ptr = self.get_ptr().as_ptr().add(i * self.layout.size());
                (self.drop)(ptr);
            }
        }
    }
}

unsafe impl Send for BlobVec {}
unsafe impl Sync for BlobVec {}

impl Drop for BlobVec {
    fn drop(&mut self) {
        self.clear();
        let array_layout =
            Layout::from_size_align(self.layout.size() * self.cap, self.layout.align())
                .expect("array layout should be valid");
        if array_layout.size() > 0 {
            unsafe {
                std::alloc::dealloc(self.get_ptr().as_ptr(), array_layout);
                std::alloc::dealloc(self.swap_ptr.as_ptr(), self.layout);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Demo {
        a: i32,
    }

    unsafe fn drop_ptr<T>(ptr: *mut u8) {
        ptr.cast::<T>().drop_in_place();
    }

    unsafe fn push_value<T>(vec: &mut BlobVec, mut value: T) {
        let index = vec.push_uninit();
        vec.initialize_unchecked(index, (&mut value as *mut T).cast::<u8>());
        std::mem::forget(value);
    }

    #[test]
    fn bloc_vec() {
        let layout = Layout::new::<Demo>();
        let mut bv = BlobVec::new(layout, drop_ptr::<Demo>, 0);

        unsafe {
            push_value(&mut bv, Demo { a: 0 });
            push_value(&mut bv, Demo { a: 1 });
            push_value(&mut bv, Demo { a: 2 });

            let ptr = bv.get_unchecked(1);
            println!("{:?}", ptr.cast::<Demo>().as_ref())
        }
    }
}
