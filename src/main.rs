use std::{thread, time::Duration};

use app::{Entity, World};
use rayon::prelude::*;

#[derive(Debug)]
struct Foo(i32);

#[derive(Debug)]
struct Bar(i32);

#[derive(Debug, Default)]
struct Entities {
    counter: u32,
}
impl Entities {
    pub fn alloc(&mut self) -> Entity {
        self.counter += 1;
        Entity::new(self.counter - 1, 0)
    }
}

fn main() {
    let mut world = World::default();
    for i in 0..10_000 {
        if rand::random::<bool>() {
            world.spawn().with(Foo(i)).with(Bar(i)).build();
        } else {
            world.spawn().with(Foo(i)).build();
        }
    }

    let ents = world.query().with::<Foo>().with::<Bar>().vec();

    let c = world.components();
    ents.par_iter().for_each(move |ent| {
        c.get_unchecked::<Foo>(ent.clone()).write().0 += 1;
        if let Some(bar) = c.get::<Bar>(ent.clone()) {
            bar.write().0 += 1;
        }
        thread::sleep(Duration::from_millis(20));
    });

    let c = world.components();
    ents.iter().for_each(move |ent| {
        let foo = c.get_unchecked::<Foo>(ent.clone()).read();
        let bar = c.get::<Bar>(ent.clone()).map(|v| v.read());
        println!("{:?}) {:?} {:?}", ent, foo, bar);
    });
}
