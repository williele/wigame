use app::{App, AppExit, AppStage, EventReader, Events, ParRunnable, Resources, SystemBuilder};
use winit::{
    event::{self, Event},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
};

use crate::{
    manager::WindowManager, WindowCloseRequest, WindowClosed, WindowCreateRequest, WindowCreated,
    WindowKeyboardInput, WindowResized,
};

pub fn window_runner(mut app: App) {
    let event_loop = EventLoop::new();

    let mut app_exit_event_reader = EventReader::<AppExit>::default();
    let mut window_create_request_reader = EventReader::<WindowCreateRequest>::default();

    let mut active = true;

    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = ControlFlow::Poll;

        if app_exit_event_reader
            .iter(&app.resources.get::<Events<AppExit>>().unwrap())
            .next_back()
            .is_some()
        {
            *control_flow = ControlFlow::Exit;
            return;
        }

        match event {
            Event::WindowEvent { window_id, event } => {
                let manager = app.resources.get_mut::<WindowManager>().unwrap();
                if manager.get(&window_id).is_none() {
                    return;
                }

                match event {
                    event::WindowEvent::CloseRequested => app
                        .resources
                        .get_mut::<Events<WindowCloseRequest>>()
                        .unwrap()
                        .send(WindowCloseRequest { id: window_id }),
                    event::WindowEvent::Resized(size) => app
                        .resources
                        .get_mut::<Events<WindowResized>>()
                        .unwrap()
                        .send(WindowResized {
                            id: window_id,
                            width: size.width,
                            height: size.height,
                        }),
                    event::WindowEvent::ScaleFactorChanged { new_inner_size, .. } => app
                        .resources
                        .get_mut::<Events<WindowResized>>()
                        .unwrap()
                        .send(WindowResized {
                            id: window_id,
                            width: new_inner_size.width,
                            height: new_inner_size.height,
                        }),
                    event::WindowEvent::KeyboardInput { ref input, .. } => app
                        .resources
                        .get_mut::<Events<WindowKeyboardInput>>()
                        .unwrap()
                        .send(WindowKeyboardInput {
                            key_code: input.virtual_keycode,
                            scan_code: input.scancode,
                            state: input.state,
                        }),
                    _ => {}
                }
            }
            Event::Suspended => {
                active = false;
            }
            Event::Resumed => {
                active = true;
            }
            Event::MainEventsCleared => {
                handle_create_window_event(
                    &mut app.resources,
                    event_loop,
                    &mut window_create_request_reader,
                );

                if active {
                    app.update()
                }
            }
            _ => {}
        }
    });
}

fn handle_create_window_event(
    resources: &mut Resources,
    event_loop: &EventLoopWindowTarget<()>,
    event_reader: &mut EventReader<WindowCreateRequest>,
) {
    let mut manager = resources.get_mut::<WindowManager>().unwrap();
    let window_create_request_event = resources.get::<Events<WindowCreateRequest>>().unwrap();

    for event in event_reader.iter(&window_create_request_event) {
        let id = manager.create(event_loop, event.descriptor.clone());
        resources
            .get_mut::<Events<WindowCreated>>()
            .unwrap()
            .send(WindowCreated { id })
    }
}

pub(crate) fn handle_window_event_sys() -> impl ParRunnable {
    let mut window_close_request_reader = EventReader::<WindowCloseRequest>::default();

    SystemBuilder::new()
        .on_stage(AppStage::Begin)
        .write_resource::<WindowManager>()
        .write_resource::<Events<WindowCloseRequest>>()
        .write_resource::<Events<WindowClosed>>()
        .write_resource::<Events<AppExit>>()
        .build(
            move |_, _, (manager, close_request, closed_event, app_exit), _| {
                for event in window_close_request_reader.iter(&close_request) {
                    if manager.remove(&event.id).is_some() {
                        closed_event.send(WindowClosed { id: event.id });
                        if manager.len() <= 0 {
                            app_exit.send(AppExit);
                        }
                    }
                }
            },
        )
}
