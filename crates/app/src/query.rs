use std::marker::PhantomData;

use util::bit_set::BitSet;

use crate::{entity::Entity, Component, Components, Filter};

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

pub struct QueryEntry<'a> {
    components: &'a Components,
}

impl<'a> QueryEntry<'a> {
    pub(crate) fn new(components: &'a Components) -> Self {
        QueryEntry { components }
    }

    pub fn filter<F: Filter<'a>>(&self) -> QueryExc<'a, F> {
        QueryExc::<F>::of(self.components)
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

        let entry = QueryEntry::new(world.components());
        entry
            .filter::<(Lock<Foo>, Try<Read<Bar>>)>()
            .vec()
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
            .vec()
            .par_iter()
            .for_each(|foo| println!("{:?}", foo));
    }
}
