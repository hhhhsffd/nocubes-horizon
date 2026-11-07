fn no_shortcut() -> core::ops::ControlFlow<bool> {
    if horizon_core::nocubes::config().collisions {
        core::ops::ControlFlow::Break(false)
    } else {
        core::ops::ControlFlow::Continue(())
    }
}
