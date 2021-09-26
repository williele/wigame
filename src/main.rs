use app::{App, AppStage, ParRunnable, Query, SystemBuilder};

#[derive(Debug)]
struct Foo(i32);

#[derive(Debug)]
struct Bar(i32);

#[derive(Debug)]
struct Baz;

#[derive(Debug)]
struct DemoResource {
    message: String,
}

fn foo_sys() -> impl ParRunnable {
    let a = SystemBuilder::new()
        .on_stage(AppStage::Startup)
        .write_resource::<DemoResource>()
        .build(|world, cmd, demo_resource, _| {
            demo_resource.message = "another message".to_string();

            cmd.spawn(world).add(Foo(0)).add(Bar(0));
            cmd.spawn(world).add(Foo(1));
            cmd.spawn(world).add(Foo(2)).add(Bar(2));
        });
    a
}

fn bar_sys() -> impl ParRunnable {
    SystemBuilder::new()
        .with_query(Query::<(&Foo, Option<&Bar>)>::new())
        .read_resource::<DemoResource>()
        .build(|world, _, demo_resource, query| {
            println!("{:?}", demo_resource);

            query
                .iter(world)
                .into_iter()
                .for_each(|data| println!("{:?}", data))
        })
}

fn main() {
    App::new()
        .add_resource(DemoResource {
            message: "this is awesome".to_string(),
        })
        .add_system(foo_sys())
        .add_system(bar_sys())
        .run();
}
