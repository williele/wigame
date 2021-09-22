use app::{Lock, QueryExc, Read, Scheduler, System, World};
use rayon::prelude::*;

#[derive(Debug)]
struct Foo(i32);

#[derive(Debug)]
struct Bar(i32);

struct WriteSystem;

fn process_foo(value: &mut Foo) {
    value.0 += 1;
}

impl System for WriteSystem {
    fn run(&mut self, components: &app::Components) {
        QueryExc::<Lock<Foo>>::of(components)
            .with::<Bar>()
            .vec()
            .par_iter()
            .for_each(|foo| process_foo(&mut foo.write()))
    }
}

struct ReadSystem;
impl System for ReadSystem {
    fn run(&mut self, components: &app::Components) {
        QueryExc::<(Read<Foo>, Read<Bar>)>::of(components)
            .vec()
            .par_iter()
            .for_each(|(foo, bar)| assert_eq!(foo.0 - bar.0, 1));
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
