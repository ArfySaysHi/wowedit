use winit::event_loop::EventLoop;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new()?;
    let mut app = app::app::App::default(); // Appappappappapp, that's code ergonomics right there
    event_loop.run_app(&mut app)?;

    Ok(())
}
