use super::*;
use crate::collisions::nocubes_collision::RegistryExt;
impl<'w> SceneCollisions<'w, '_> {
    pub(super) fn nocubes_query_59(
        &mut self,
        block: CollisionBlock,
        p: BlockPos,
        context: &CollisionContext<'_>,
    ) -> io::Result<VoxelShape> {
        self.binding
            .nocubes_collision_shape_with_context(self, block.state, p, context)
            .map_err(io::Error::other)
    }
}
impl SceneCollisions<'_, '_> {
    pub(super) fn nocubes_query_60(
        &mut self,
        state: BlockStateId,
        p: [i32; 3],
    ) -> io::Result<VoxelShape> {
        let binding = self.binding;
        binding
            .nocubes_collision_shape(self, state, p)
            .map_err(io::Error::other)
    }
}
impl LookupCollisions<'_, '_> {
    pub(super) fn nocubes_query_61(
        &mut self,
        block: CollisionBlock,
        p: BlockPos,
        context: &CollisionContext<'_>,
    ) -> io::Result<VoxelShape> {
        self.binding
            .nocubes_collision_shape_with_context(self, block.state, p, context)
            .map_err(io::Error::other)
    }
}
