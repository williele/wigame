use util::bit_set::BitSet;

use crate::{
    accessor::{Read, TryRead, TryWrite, Write},
    Component, Components, Entity,
};

pub trait IntoView {
    type View: for<'a> View<'a> + 'static;
}

pub trait View<'a>: Sized {
    type Item: Send + Sync + 'a;

    fn filter(bitset: &mut BitSet, components: &Components);
    fn fetch(entity: Entity, components: &Components) -> Self::Item;
}

impl<'a, T: Component> IntoView for &'a T {
    type View = Read<T>;
}

impl<'a, T: Component> IntoView for &'a mut T {
    type View = Write<T>;
}

impl<'a, T: Component> IntoView for Option<&'a T> {
    type View = TryRead<T>;
}

impl<'a, T: Component> IntoView for Option<&'a mut T> {
    type View = TryWrite<T>;
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
