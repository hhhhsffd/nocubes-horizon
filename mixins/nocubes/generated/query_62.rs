fn adapter(level: &ParticleLevel, pos: [i32; 3]) -> core::ops::ControlFlow<bool> {
    core::ops::ControlFlow::Break(nocubes_queries::nocubes_query_62(level, pos))
}
