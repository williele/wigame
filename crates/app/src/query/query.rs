use std::marker::PhantomData;

use crate::World;

use super::{IntoView, View};

#[derive(Debug, Default)]
pub struct Query<V: IntoView> {
    _view: PhantomData<V>,
}

impl<V: IntoView> Query<V> {
    pub fn new() -> Self {
        Query {
            _view: Default::default(),
        }
    }

    pub fn iter<'a>(&'a self, world: &'a World) -> Vec<<V::View as View>::Item> {
        let mut bitset = world.entity_allocator().get_bitset().clone();
        <V::View as View>::filter(&mut bitset, world.components());
        bitset
            .into_iter()
            .filter_map(|id| world.entity_allocator().get_entity(id as u32))
            .map(|entity| <V::View as View>::fetch(entity, world.components()))
            .collect()
    }
}

pub trait QuerySet: Send + Sync {}

macro_rules! impl_queryset_tuple {
    ($($name: ident),*) => {
        impl<$($name,)*> QuerySet for ($($name,)*)
        where
            $($name: QuerySet,)*
        {}
    };
}

macro_rules! queryset_tuple {
    ($head_ty:ident) => {
        impl_queryset_tuple!($head_ty);
    };
    ($head_ty:ident, $( $tail_ty:ident ),*) => (
        impl_queryset_tuple!($head_ty, $($tail_ty),*);
        queryset_tuple!($($tail_ty),*);
    );
}

queryset_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);

impl QuerySet for () {}
impl<V> QuerySet for Query<V> where V: IntoView + Send + Sync {}

#[cfg(test)]
mod tests {
    use crate::{Read, TryRead, World, Write};

    use super::super::view::Entities;
    use super::Query;

    #[derive(Debug)]
    struct Foo(i32);

    #[derive(Debug)]
    struct Bar(i32);

    #[test]
    fn query() {
        let mut world = World::default();
        world.spawn().add(Foo(0)).add(Bar(0));
        world.spawn().add(Foo(1));
        world.spawn().add(Foo(2)).add(Bar(2));

        for foo in Query::<Write<Foo>>::new().iter(&world) {
            foo.0 += 1;
        }

        for (ent, foo, bar) in Query::<(Entities, Read<Foo>, TryRead<Bar>)>::new().iter(&world) {
            println!("{:?} {:?} {:?}", ent, foo, bar);
        }
    }
}
