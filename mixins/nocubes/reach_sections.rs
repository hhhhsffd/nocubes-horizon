fn reach_sections() -> core::ops::ControlFlow<io::Result<bool>> {
    if horizon_core::nocubes::config().collisions {
        core::ops::ControlFlow::Break(Ok(true))
    } else {
        core::ops::ControlFlow::Continue(())
    }
}
