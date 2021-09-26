use app::{App, AppStage, EventReader, Events, ParRunnable, Query, SystemBuilder};

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

struct AppExitEvent;

fn foo_sys() -> impl ParRunnable {
    let a = SystemBuilder::new()
        .on_stage(AppStage::Startup)
        .write_resource::<DemoResource>()
        .write_resource::<Events<AppExitEvent>>()
        .build(|world, cmd, (demo_resource, app_exit_events), _| {
            demo_resource.message = "another message".to_string();
            app_exit_events.send(AppExitEvent);

            cmd.spawn(world).add(Foo(0)).add(Bar(0));
            cmd.spawn(world).add(Foo(1));
            cmd.spawn(world).add(Foo(2)).add(Bar(2));
        });
    a
}

fn bar_sys() -> impl ParRunnable {
    let mut app_exit_reader = EventReader::<AppExitEvent>::default();

    SystemBuilder::new()
        .with_query(Query::<(&Foo, Option<&Bar>)>::new())
        .read_resource::<DemoResource>()
        .read_resource::<Events<AppExitEvent>>()
        .build(move |world, _, (demo_resource, app_exit_events), query| {
            println!("{:?}", demo_resource);
            for _ in app_exit_reader.iter(app_exit_events) {
                println!("ask for exit");
            }

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
        .add_event::<AppExitEvent>()
        .add_system(foo_sys())
        .add_system(bar_sys())
        .run();
}
