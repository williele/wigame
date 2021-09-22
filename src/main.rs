use app::{Lock, Read, Scheduler, System, World};
use rayon::prelude::*;

#[derive(Debug)]
struct Foo(i32);
impl Foo {
    fn increment(&mut self) {
        self.0 += 1;
    }
}

#[derive(Debug)]
struct Bar(i32);

struct WriteSystem;

impl System for WriteSystem {
    fn run(&mut self, query: &app::QueryEntry) {
        query
            .filter::<Lock<Foo>>()
            .with::<Bar>()
            .vec()
            .par_iter()
            .for_each(|foo| foo.write().increment());
    }
}

struct ReadSystem;
impl System for ReadSystem {
    fn run(&mut self, query: &app::QueryEntry) {
        query
            .filter::<(Read<Foo>, Read<Bar>)>()
            .vec()
            .par_iter()
            .for_each(|_| {})
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
