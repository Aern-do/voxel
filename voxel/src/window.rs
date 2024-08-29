use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::ActiveEventLoop,
    window::WindowId,
};

#[derive(Debug, Clone)]
pub struct Window<A, F> {
    application: Option<A>,
    initalizer: F,
}

impl<A, F> Window<A, F> {
    pub fn new(initalizer: F) -> Self {
        Self {
            application: None,
            initalizer,
        }
    }
}

impl<A: ApplicationHandler, F: Fn(&ActiveEventLoop) -> A> ApplicationHandler for Window<A, F> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.application.is_some() {
            return;
        }

        self.application = Some((self.initalizer)(event_loop))
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(application) = &mut self.application {
            application.window_event(event_loop, window_id, event)
        }
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let Some(application) = &mut self.application {
            application.device_event(event_loop, device_id, event)
        }
    }
}
