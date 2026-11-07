fn mesh(
    mesher: &mut crate::mesher::Mesher<'_>,
    region: &[[Option<&crate::chunk::Chunk>; 3]; 3],
    sy: i32,
    out: &mut crate::mesher::SectionMesh,
    scratch: &mut crate::mesher::MeshScratch,
) -> core::ops::ControlFlow<SectionMesh> {
    core::ops::ControlFlow::Break(mesher.nocubes_mesh(region, sy, std::mem::take(out), scratch))
}
