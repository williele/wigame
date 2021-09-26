use std::marker::PhantomData;

use util::bit_set::BitSet;

use crate::{Component, Components, Entity};

pub trait IntoView {
    type View: for<'a> View<'a> + 'static;
}

pub trait View<'a>: Sized {
    type Item: Send + Sync + 'a;

    fn filter(bitset: &mut BitSet, components: &Components);
    fn fetch(entity: Entity, components: &Components) -> Self::Item;
}

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
impl<'a, T: Component> IntoView for &'a T {
    type View = Read<T>;
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
impl<'a, T: Component> IntoView for &'a mut T {
    type View = Write<T>;
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
impl<'a, T: Component> IntoView for Option<&'a T> {
    type View = TryRead<T>;
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
impl<'a, T: Component> IntoView for Option<&'a mut T> {
    type View = TryWrite<T>;
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

#[derive(Debug, Clone, Copy, Default)]
pub struct Entities;

unsafe impl Send for Entities {}
unsafe impl Sync for Entities {}

impl IntoView for Entities {
    type View = Self;
}

impl<'a> View<'a> for Entities {
    type Item = Entity;

    fn filter(_bitset: &mut BitSet, _components: &Components) {}
    fn fetch(entity: Entity, _components: &Components) -> Self::Item {
        entity
    }
}

macro_rules! view_tuple {
    ($($name: ident), *) => {
        impl<'a, $($name: View<'a> + 'a),*> View<'a> for ($($name,)*) {
            type Item = ($($name::Item,)*);

            fn filter(bitset: &mut BitSet, components: &Components) {
                $($name::filter(bitset, components);)*
            }
            fn fetch(entity: Entity, components: &Components) -> Self::Item {
                ($($name::fetch(entity, components),)*)
            }
        }

        impl<$($name: IntoView),*> IntoView for ($($name,)*) {
            type View = ($($name::View,)*);
        }
    };
}

macro_rules! impl_view_tuple {
    ($head_ty:ident) => {
        view_tuple!($head_ty);
    };
    ($head_ty:ident, $( $tail_ty:ident ),*) => (
        view_tuple!($head_ty, $( $tail_ty ),*);
        impl_view_tuple!($( $tail_ty ),*);
    );
}

impl_view_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);
