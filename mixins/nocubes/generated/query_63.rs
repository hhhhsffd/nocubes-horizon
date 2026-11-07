fn adapter(level: &ParticleLevel, a: [f64; 3], b: [f64; 3]) -> core::ops::ControlFlow<bool> {
    core::ops::ControlFlow::Break(nocubes_queries::nocubes_query_63(level, a, b))
}
