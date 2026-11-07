fn adapter(
    receiver: &mut Game<'_>,
    camera: glam::DVec3,
    alpha: f64,
    fov_degrees: f32,
    out: &mut Vec<Vertex>,
) -> core::ops::ControlFlow<(
    Vec<Vertex>,
    Arc<[StaticSectionSnapshot]>,
    Vec<[i32; 3]>,
    StaticCacheFrameStats,
)> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_33(
        camera,
        alpha,
        fov_degrees,
        std::mem::take(out),
    ))
}
