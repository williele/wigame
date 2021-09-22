use app::{Query, Scheduler, System, World};
use rayon::prelude::*;

#[derive(Debug)]
struct Foo(i32);

#[derive(Debug)]
struct Bar(i32);

struct WriteSystem;

impl System for WriteSystem {
    fn run(&mut self, components: &app::Components) {
        let ents = Query::empty(components).with::<Foo>().with::<Bar>().vec();
        ents.par_iter().for_each(|ent| {
            components.get_unchecked::<Foo>(ent.clone()).write().0 += 1;
            components.get_unchecked::<Bar>(ent.clone()).write().0 += 1;
        });
    }
}

struct ReadSystem;
impl System for ReadSystem {
    fn run(&mut self, components: &app::Components) {
        let ents = Query::empty(components).with::<Foo>().with::<Bar>().vec();
        ents.par_iter().for_each(|ent| {
            let _foo = components.get_unchecked::<Foo>(ent.clone()).read();
            let _bar = components.get_unchecked::<Bar>(ent.clone()).read();
        });
    }
}

fn main() {
    let mut world = World::default();
    for i in 0..1_000_000 {
        if rand::random::<bool>() {
            world.spawn().with(Foo(i)).with(Bar(i)).build();
        } else {
            world.spawn().with(Foo(i)).build();
        }
    }

    let mut scheduler = Scheduler::default();
    scheduler.add(WriteSystem).add(ReadSystem);

    scheduler.execute(world.components());
}
