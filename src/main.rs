use app::{App, AppStage, ParRunnable, Query, SystemBuilder};

#[derive(Debug)]
struct Foo(i32);

#[derive(Debug)]
struct Bar(i32);

#[derive(Debug)]
struct Baz;

fn foo_sys() -> impl ParRunnable {
    SystemBuilder::new()
        .on_stage(AppStage::Startup)
        .build(|world, cmd, _| {
            cmd.spawn(world).add(Foo(0)).add(Bar(0));
            cmd.spawn(world).add(Foo(1));
            cmd.spawn(world).add(Foo(2)).add(Bar(2));
        })
}

fn bar_sys() -> impl ParRunnable {
    let query = Query::<(&Foo, Option<&Bar>)>::new();

    SystemBuilder::new()
        .with_query(query)
        .build(|world, _, query| {
            query
                .iter(world)
                .into_iter()
                .for_each(|data| println!("{:?}", data))
        })
}

fn main() {
    App::new().add_system(foo_sys()).add_system(bar_sys()).run();
}
