use std::marker::PhantomData;

use util::bit_set::BitSet;

use crate::{entity::Entity, Component, Filter, World};

pub struct QueryExc<'a, F: Filter<'a>> {
    bitset: BitSet,
    world: &'a World,
    _marker: PhantomData<F>,
}

impl<'a, F: Filter<'a>> QueryExc<'a, F> {
    fn of(world: &'a World) -> Self {
        let mut bitset = world.entities().get_bitset().clone();
        F::bitset_op(&mut bitset, world);
        QueryExc {
            bitset,
            world,
            _marker: Default::default(),
        }
    }

    pub fn with<T: Component>(&mut self) -> &mut Self {
        self.bitset
            .intersect_with(self.world.components().get_bitset::<T>().unwrap());
        self
    }

    pub fn without<T: Component>(&mut self) -> &mut Self {
        self.bitset
            .symmetric_difference_with(self.world.components().get_bitset::<T>().unwrap());
        self
    }

    pub fn entity(&self, entity: Entity) -> Option<F::Item> {
        (self.world.entities().is_live(entity) && self.bitset.contains(entity.id() as usize))
            .then(|| F::get_unchecked(self.world, entity))
    }

    pub fn entities(&self, entities: &[Entity]) -> Vec<F::Item> {
        entities
            .into_iter()
            .filter(|&&entity| {
                self.world.entities().is_live(entity) && self.bitset.contains(entity.id() as usize)
            })
            .map(|&entity| F::get_unchecked(self.world, entity))
            .collect()
    }

    pub fn all(&self) -> Vec<F::Item> {
        self.bitset
            .into_iter()
            .filter_map(|id| self.world.entities().get_entity(id as u32))
            .map(|entity| F::get_unchecked(self.world, entity))
            .collect()
    }
}

pub struct QueryEntry<'a> {
    world: &'a World,
}

impl<'a> QueryEntry<'a> {
    pub(crate) fn new(world: &'a World) -> Self {
        QueryEntry { world }
    }

    pub fn filter<F: Filter<'a>>(&self) -> QueryExc<'a, F> {
        QueryExc::<F>::of(self.world)
    }
}

#[cfg(test)]
mod tests {
    use util::rayon::prelude::*;

    use crate::{Lock, Read, Try, World};

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

        let entry = QueryEntry::new(&world);
        entry
            .filter::<(Lock<Foo>, Try<Read<Bar>>)>()
            .all()
            .par_iter()
            .for_each(|(foo, bar)| {
                if let Some(bar) = bar {
                    foo.write().0 += bar.0;
                } else {
                    foo.write().0 -= 1;
                }
            });

        entry
            .filter::<Read<Foo>>()
            .all()
            .par_iter()
            .for_each(|foo| println!("{:?}", foo));
    }
}
