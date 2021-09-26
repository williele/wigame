use app::{App, AppExit, EventReader, Events};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use crate::{manager::WindowManager, CreateWindow, WindowLaunched, WindowResized};

pub fn window_runner(mut app: App) {
    let event_loop = EventLoop::new();

    let mut app_exit_event_reader = EventReader::<AppExit>::default();
    let mut create_window_event_reader = EventReader::<CreateWindow>::default();

    let mut active = true;

    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = ControlFlow::Poll;

        if app_exit_event_reader
            .iter(&app.resources.get::<Events<AppExit>>().unwrap())
            .next_back()
            .is_some()
        {
            *control_flow = ControlFlow::Exit;
        }

        match event {
            Event::WindowEvent { window_id, event } => {
                let resources = &mut app.resources;
                let manager = resources.get_mut::<WindowManager>().unwrap();
                if manager.get(&window_id).is_none() {
                    return;
                }

                match event {
                    WindowEvent::CloseRequested => {
                        //
                    }
                    WindowEvent::Resized(size) => resources
                        .get_mut::<Events<WindowResized>>()
                        .unwrap()
                        .send(WindowResized {
                            id: window_id,
                            width: size.width,
                            height: size.height,
                        }),
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => resources
                        .get_mut::<Events<WindowResized>>()
                        .unwrap()
                        .send(WindowResized {
                            id: window_id,
                            width: new_inner_size.width,
                            height: new_inner_size.height,
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
                {
                    let mut manager = app.resources.get_mut::<WindowManager>().unwrap();

                    let events = app.resources.get::<Events<CreateWindow>>().unwrap();
                    for event in create_window_event_reader.iter(&events) {
                        let id = manager.create(event_loop, event.descriptor.clone());
                        app.resources
                            .get_mut::<Events<WindowLaunched>>()
                            .unwrap()
                            .send(WindowLaunched { id })
                    }
                }

                if active {
                    app.update()
                }
            }
            _ => {}
        }
    });
}
