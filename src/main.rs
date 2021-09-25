use app::{Query, World};

#[derive(Debug)]
struct Foo(i32);

#[derive(Debug)]
struct Bar(i32);

fn main() {
    let mut world = World::default();

    world.spawn().with(Foo(0)).with(Bar(10)).build();
    world.spawn().with(Foo(1)).build();
    world.spawn().with(Foo(2)).with(Bar(12)).build();
    world.flush();

    for (foo, bar) in Query::<(&mut Foo, &Bar)>::new().iter(&world) {
        foo.0 += bar.0;
    }

    for data in Query::<(&Foo, Option<&Bar>)>::new().iter(&world) {
        println!("{:?}", data);
    }
}
