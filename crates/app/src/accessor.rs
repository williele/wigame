use std::marker::PhantomData;

use util::bit_set::BitSet;

use crate::{Component, Components, Entity, IntoView, View};

// Read
#[derive(Debug, Clone, Copy)]
pub struct Read<T>(PhantomData<*const T>);
impl<T> Default for Read<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

unsafe impl<T> Send for Read<T> {}
unsafe impl<T> Sync for Read<T> {}

impl<T: Component> IntoView for Read<T> {
    type View = Self;
}
impl<'a, T: Component> View<'a> for Read<T> {
    type Item = &'a T;

    fn filter(bitset: &mut BitSet, components: &Components) {
        bitset.intersect_with(components.get_bitset::<T>().unwrap_or(&BitSet::new()));
    }

    fn fetch(entity: Entity, components: &Components) -> Self::Item {
        unsafe {
            components
                .get_ptr::<T>(entity)
                .and_then(|ptr| ptr.cast::<T>().as_ref())
                .expect("failed to cast ReadView")
        }
    }
}

// Write
#[derive(Debug, Clone, Copy)]
pub struct Write<T>(PhantomData<*const T>);
impl<T> Default for Write<T> {
    fn default() -> Self {
        Write(PhantomData)
    }
}

unsafe impl<T> Send for Write<T> {}
unsafe impl<T> Sync for Write<T> {}

impl<T: Component> IntoView for Write<T> {
    type View = Self;
}
impl<'a, T: Component> View<'a> for Write<T> {
    type Item = &'a mut T;

    fn filter(bitset: &mut BitSet, components: &Components) {
        bitset.intersect_with(components.get_bitset::<T>().unwrap_or(&BitSet::new()));
    }

    fn fetch(entity: Entity, components: &Components) -> Self::Item {
        unsafe {
            components
                .get_ptr::<T>(entity)
                .and_then(|ptr| ptr.cast::<T>().as_mut())
                .expect("failed to cast WriteView")
        }
    }
}

// TryRead

#[derive(Debug, Clone, Copy)]
pub struct TryRead<T>(PhantomData<*const T>);
impl<T> Default for TryRead<T> {
    fn default() -> Self {
        TryRead(PhantomData)
    }
}

unsafe impl<T> Send for TryRead<T> {}
unsafe impl<T> Sync for TryRead<T> {}

impl<T: Component> IntoView for TryRead<T> {
    type View = Self;
}
impl<'a, T: Component> View<'a> for TryRead<T> {
    type Item = Option<&'a T>;

    fn filter(bitset: &mut BitSet, components: &Components) {
        bitset.union_with(components.get_bitset::<T>().unwrap_or(&BitSet::new()));
    }

    fn fetch(entity: Entity, components: &Components) -> Self::Item {
        unsafe {
            components
                .get_ptr::<T>(entity)
                .and_then(|ptr| ptr.cast::<T>().as_ref())
        }
    }
}

// TryWrite
#[derive(Debug, Clone, Copy)]
pub struct TryWrite<T>(PhantomData<*const T>);
impl<T> Default for TryWrite<T> {
    fn default() -> Self {
        TryWrite(PhantomData)
    }
}

unsafe impl<T> Send for TryWrite<T> {}
unsafe impl<T> Sync for TryWrite<T> {}

impl<T: Component> IntoView for TryWrite<T> {
    type View = Self;
}
impl<'a, T: Component> View<'a> for TryWrite<T> {
    type Item = Option<&'a mut T>;

    fn filter(bitset: &mut BitSet, components: &Components) {
        bitset.union_with(components.get_bitset::<T>().unwrap_or(&BitSet::new()));
    }

    fn fetch(entity: Entity, components: &Components) -> Self::Item {
        unsafe {
            components
                .get_ptr::<T>(entity)
                .and_then(|ptr| ptr.cast::<T>().as_mut())
        }
    }
}
