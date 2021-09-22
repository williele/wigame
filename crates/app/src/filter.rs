use std::marker::PhantomData;

use util::{
    bit_set::BitSet,
    parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{Component, Components, Entity};

pub trait Filter<'a> {
    type Item;
    fn bitset(components: &'a Components) -> BitSet;
    fn bitset_op(bitset: &mut BitSet, components: &'a Components);
    fn get_unchecked(components: &'a Components, entity: Entity) -> Self::Item;
    fn get(components: &'a Components, entity: Entity) -> Option<Self::Item>;
}

pub struct Lock<T>(PhantomData<T>);
impl<'a, T: Component> Filter<'a> for Lock<T> {
    type Item = &'a RwLock<T>;

    fn bitset(components: &'a Components) -> BitSet {
        components.get_bitset::<T>().unwrap().clone()
    }
    fn bitset_op(bitset: &mut BitSet, components: &'a Components) {
        bitset.intersect_with(components.get_bitset::<T>().unwrap());
    }
    fn get_unchecked(components: &'a Components, entity: Entity) -> Self::Item {
        components.get_unchecked::<T>(entity)
    }
    fn get(components: &'a Components, entity: Entity) -> Option<Self::Item> {
        components.get::<T>(entity)
    }
}

pub struct Read<T>(PhantomData<T>);
impl<'a, T: Component> Filter<'a> for Read<T> {
    type Item = RwLockReadGuard<'a, T>;

    fn bitset(components: &'a Components) -> BitSet {
        components.get_bitset::<T>().unwrap().clone()
    }
    fn bitset_op(bitset: &mut BitSet, components: &'a Components) {
        bitset.intersect_with(components.get_bitset::<T>().unwrap());
    }
    fn get_unchecked(components: &'a Components, entity: Entity) -> Self::Item {
        components.get_unchecked::<T>(entity).read()
    }
    fn get(components: &'a Components, entity: Entity) -> Option<Self::Item> {
        components.get::<T>(entity).map(|l| l.read())
    }
}

pub struct Write<T>(PhantomData<T>);
impl<'a, T: Component> Filter<'a> for Write<T> {
    type Item = RwLockWriteGuard<'a, T>;

    fn bitset(components: &'a Components) -> BitSet {
        components.get_bitset::<T>().unwrap().clone()
    }
    fn bitset_op(bitset: &mut BitSet, components: &'a Components) {
        bitset.intersect_with(components.get_bitset::<T>().unwrap());
    }
    fn get_unchecked(components: &'a Components, entity: Entity) -> Self::Item {
        components.get_unchecked::<T>(entity).write()
    }
    fn get(components: &'a Components, entity: Entity) -> Option<Self::Item> {
        components.get::<T>(entity).map(|l| l.write())
    }
}

pub struct Try<T>(PhantomData<T>);
impl<'a, T: Filter<'a>> Filter<'a> for Try<T> {
    type Item = Option<T::Item>;

    fn bitset(components: &'a Components) -> BitSet {
        T::bitset(components)
    }
    fn bitset_op(bitset: &mut BitSet, components: &'a Components) {
        bitset.union_with(&T::bitset(components));
    }
    fn get_unchecked(components: &'a Components, entity: Entity) -> Self::Item {
        T::get(components, entity)
    }
    fn get(_components: &'a Components, _entity: Entity) -> Option<Self::Item> {
        None
    }
}

macro_rules! tuple_impl {
    ($($name: ident), *) => {
        impl<'a, $($name: Filter<'a>),*> Filter<'a> for ($($name,)*) {
            type Item = ($($name::Item,)*);

            fn bitset(components: &'a Components) -> BitSet {
                let mut a = A::bitset(components);
                $($name::bitset_op(&mut a, components);)*
                a
            }
            fn bitset_op(_bitset: &mut BitSet, _components: &Components) {}
            fn get_unchecked(components: &'a Components, entity: Entity) -> Self::Item {
                ($($name::get_unchecked(components, entity),)*)
            }
            fn get(_components: &'a Components, _entity: Entity) -> Option<Self::Item> {
                None
            }
        }
    };
}
tuple_impl!(A, B);
tuple_impl!(A, B, C);
tuple_impl!(A, B, C, D);
tuple_impl!(A, B, C, D, E);
tuple_impl!(A, B, C, D, E, F);
tuple_impl!(A, B, C, D, E, F, G);
tuple_impl!(A, B, C, D, E, F, G, H);
tuple_impl!(A, B, C, D, E, F, G, H, I);
tuple_impl!(A, B, C, D, E, F, G, H, I, J);
tuple_impl!(A, B, C, D, E, F, G, H, I, J, K);
tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L);
tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M);
tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);

pub struct QueryExc<'a, F: Filter<'a>> {
    bitset: BitSet,
    components: &'a Components,
    _marker: PhantomData<F>,
}

impl<'a, F: Filter<'a>> QueryExc<'a, F> {
    pub fn of(components: &'a Components) -> Self {
        let bitset = F::bitset(components);
        QueryExc {
            bitset,
            components,
            _marker: Default::default(),
        }
    }

    pub fn with<T: Component>(&mut self) -> &mut Self {
        self.bitset
            .intersect_with(self.components.get_bitset::<T>().unwrap());
        self
    }

    pub fn without<T: Component>(&mut self) -> &mut Self {
        self.bitset
            .symmetric_difference_with(self.components.get_bitset::<T>().unwrap());
        self
    }

    pub fn vec(&self) -> Vec<F::Item> {
        self.bitset
            .into_iter()
            .map(|id| Entity::new(id as u32, 0))
            .map(|ent| F::get_unchecked(self.components, ent))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use util::rayon::prelude::*;

    use crate::World;

    use super::*;

    #[derive(Debug)]
    struct Foo(i32);
    #[derive(Debug)]
    struct Bar(i32);
    #[derive(Debug)]
    struct Baz(i32);

    #[test]
    fn query() {
        let mut world = World::default();
        world.spawn().with(Foo(0)).with(Bar(1)).build();
        world.spawn().with(Foo(0)).build();
        world
            .spawn()
            .with(Foo(0))
            .with(Bar(2))
            .with(Baz(10))
            .build();

        QueryExc::<(Read<Foo>, Try<Read<Bar>>)>::of(world.components())
            .with::<Baz>()
            .vec()
            .par_iter()
            .for_each(|data| println!("{:?}", data));
    }
}
