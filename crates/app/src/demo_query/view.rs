use std::marker::PhantomData;

use util::bit_set::BitSet;

use crate::{Component, Entity, World};

pub trait IntoView {
    type View: for<'a> View<'a> + 'static;
}

pub trait View<'a>: Sized {
    type Item: Send + Sync + 'a;

    fn filter(bitset: &mut BitSet, world: &World);
    fn fetch(entity: Entity, world: &World) -> Self::Item;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Read<T>(PhantomData<T>);

unsafe impl<T> Send for Read<T> {}
unsafe impl<T> Sync for Read<T> {}

impl<T: Component> IntoView for Read<T> {
    type View = Self;
}

impl<'a, T: Component> View<'a> for Read<T> {
    type Item = &'a T;

    fn filter(bitset: &mut BitSet, world: &World) {
        bitset.intersect_with(world.components().get_bitset::<T>().unwrap());
    }

    fn fetch(entity: Entity, world: &World) -> Self::Item {
        unsafe {
            world
                .components()
                .get::<T>(entity)
                .unwrap()
                .data_ptr()
                .as_ref()
                .unwrap()
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Write<T>(PhantomData<T>);

unsafe impl<T> Send for Write<T> {}
unsafe impl<T> Sync for Write<T> {}

impl<T: Component> IntoView for Write<T> {
    type View = Self;
}

impl<'a, T: Component> View<'a> for Write<T> {
    type Item = &'a mut T;

    fn filter(bitset: &mut BitSet, world: &World) {
        bitset.intersect_with(world.components().get_bitset::<T>().unwrap());
    }

    fn fetch(entity: Entity, world: &World) -> Self::Item {
        unsafe {
            world
                .components()
                .get::<T>(entity)
                .unwrap()
                .data_ptr()
                .as_mut()
                .unwrap()
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TryRead<T>(PhantomData<T>);

unsafe impl<T> Send for TryRead<T> {}
unsafe impl<T> Sync for TryRead<T> {}

impl<T: Component> IntoView for TryRead<T> {
    type View = Self;
}

impl<'a, T: Component> View<'a> for TryRead<T> {
    type Item = Option<&'a T>;

    fn filter(bitset: &mut BitSet, world: &World) {
        bitset.union_with(world.components().get_bitset::<T>().unwrap());
    }

    fn fetch(entity: Entity, world: &World) -> Self::Item {
        unsafe {
            world
                .components()
                .get::<T>(entity)
                .and_then(|l| l.data_ptr().as_ref())
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TryWrite<T>(PhantomData<T>);

unsafe impl<T> Send for TryWrite<T> {}
unsafe impl<T> Sync for TryWrite<T> {}

impl<T: Component> IntoView for TryWrite<T> {
    type View = Self;
}

impl<'a, T: Component> View<'a> for TryWrite<T> {
    type Item = Option<&'a mut T>;

    fn filter(bitset: &mut BitSet, world: &World) {
        bitset.union_with(world.components().get_bitset::<T>().unwrap());
    }

    fn fetch(entity: Entity, world: &World) -> Self::Item {
        unsafe {
            world
                .components()
                .get::<T>(entity)
                .and_then(|l| l.data_ptr().as_mut())
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

    fn filter(_bitset: &mut BitSet, _world: &World) {}
    fn fetch(entity: Entity, _world: &World) -> Self::Item {
        entity
    }
}

macro_rules! view_tuple {
    ($($name: ident), *) => {
        impl<'a, $($name: View<'a> + 'a),*> View<'a> for ($($name,)*) {
            type Item = ($($name::Item,)*);

            fn filter(bitset: &mut BitSet, world: &World) {
                $($name::filter(bitset, world);)*
            }
            fn fetch(entity: Entity, world: &World) -> Self::Item {
                ($($name::fetch(entity, world),)*)
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