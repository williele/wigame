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

#[cfg(test)]
mod tests {
    use crate::World;

    use super::super::view::{Entities, Read, TryRead, Write};
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
