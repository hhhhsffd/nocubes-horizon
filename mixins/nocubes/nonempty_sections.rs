fn nonempty_sections() -> core::ops::ControlFlow<io::Result<bool>> {
    if horizon_core::nocubes::config().collisions {
        core::ops::ControlFlow::Break(Ok(false))
    } else {
        core::ops::ControlFlow::Continue(())
    }
}
