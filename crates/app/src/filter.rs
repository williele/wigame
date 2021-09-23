use std::marker::PhantomData;

use util::{
    bit_set::BitSet,
    parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::{entity::Entity, Component, World};

pub trait Filter<'a> {
    type Item;
    fn bitset(world: &'a World) -> BitSet;
    fn bitset_op(bitset: &mut BitSet, world: &'a World);
    fn get_unchecked(world: &'a World, entity: Entity) -> Self::Item;
    fn get(world: &'a World, entity: Entity) -> Option<Self::Item>;
}

pub struct Lock<T>(PhantomData<T>);
impl<'a, T: Component> Filter<'a> for Lock<T> {
    type Item = &'a RwLock<T>;

    fn bitset(world: &'a World) -> BitSet {
        world.components().get_bitset::<T>().unwrap().clone()
    }
    fn bitset_op(bitset: &mut BitSet, world: &'a World) {
        bitset.intersect_with(world.components().get_bitset::<T>().unwrap());
    }
    fn get_unchecked(world: &'a World, entity: Entity) -> Self::Item {
        world.components().get::<T>(entity).unwrap()
    }
    fn get(world: &'a World, entity: Entity) -> Option<Self::Item> {
        world.components().get::<T>(entity)
    }
}

pub struct Read<T>(PhantomData<T>);
impl<'a, T: Component> Filter<'a> for Read<T> {
    type Item = RwLockReadGuard<'a, T>;

    fn bitset(world: &'a World) -> BitSet {
        world.components().get_bitset::<T>().unwrap().clone()
    }
    fn bitset_op(bitset: &mut BitSet, world: &'a World) {
        bitset.intersect_with(world.components().get_bitset::<T>().unwrap());
    }
    fn get_unchecked(world: &'a World, entity: Entity) -> Self::Item {
        world.components().get::<T>(entity).unwrap().read()
    }
    fn get(world: &'a World, entity: Entity) -> Option<Self::Item> {
        world.components().get::<T>(entity).map(|l| l.read())
    }
}

pub struct Write<T>(PhantomData<T>);
impl<'a, T: Component> Filter<'a> for Write<T> {
    type Item = RwLockWriteGuard<'a, T>;

    fn bitset(world: &'a World) -> BitSet {
        world.components().get_bitset::<T>().unwrap().clone()
    }
    fn bitset_op(bitset: &mut BitSet, world: &'a World) {
        bitset.intersect_with(world.components().get_bitset::<T>().unwrap());
    }
    fn get_unchecked(world: &'a World, entity: Entity) -> Self::Item {
        world.components().get::<T>(entity).unwrap().write()
    }
    fn get(world: &'a World, entity: Entity) -> Option<Self::Item> {
        world.components().get::<T>(entity).map(|l| l.write())
    }
}

pub struct Try<T>(PhantomData<T>);
impl<'a, T: Filter<'a>> Filter<'a> for Try<T> {
    type Item = Option<T::Item>;

    fn bitset(world: &'a World) -> BitSet {
        T::bitset(world)
    }
    fn bitset_op(bitset: &mut BitSet, world: &'a World) {
        bitset.union_with(&T::bitset(world));
    }
    fn get_unchecked(world: &'a World, entity: Entity) -> Self::Item {
        T::get(world, entity)
    }
    fn get(_world: &'a World, _entity: Entity) -> Option<Self::Item> {
        None
    }
}

pub struct Entities;
impl<'a> Filter<'a> for Entities {
    type Item = Entity;

    fn bitset(world: &'a World) -> BitSet {
        world.entities().get_bitset().clone()
    }
    fn bitset_op(_bitset: &mut BitSet, _world: &'a World) {}
    fn get_unchecked(_world: &'a World, entity: Entity) -> Self::Item {
        entity
    }
    fn get(_world: &'a World, entity: Entity) -> Option<Self::Item> {
        Some(entity)
    }
}

macro_rules! tuple_impl {
    ($($name: ident), *) => {
        impl<'a, $($name: Filter<'a>),*> Filter<'a> for ($($name,)*) {
            type Item = ($($name::Item,)*);

            fn bitset(world: &'a World) -> BitSet {
                let mut a = A::bitset(world);
                $($name::bitset_op(&mut a, world);)*
                a
            }
            fn bitset_op(bitset: &mut BitSet, world: &'a World) {
                $($name::bitset_op(bitset, world);)*
            }
            fn get_unchecked(world: &'a World, entity: Entity) -> Self::Item {
                ($($name::get_unchecked(world, entity),)*)
            }
            fn get(_world: &'a World, _entity: Entity) -> Option<Self::Item> {
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
