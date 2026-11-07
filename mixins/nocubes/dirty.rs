fn dirty(world: &mut crate::world::World, x: i32, y: i32, z: i32) {
    if !horizon_core::nocubes::config().render {
        return;
    }
    for cz in (z - 3).div_euclid(16)..=(z + 3).div_euclid(16) {
        for cx in (x - 3).div_euclid(16)..=(x + 3).div_euclid(16) {
            if let Some(chunk) = world.chunks.get(&(cx, cz)) {
                for by in [y - 3, y, y + 3] {
                    let sy = (by - chunk.min_y()).div_euclid(16);
                    if sy >= 0 && (sy as usize) < chunk.sections.len() {
                        world.dirty_sections.insert((cx, sy, cz));
                    }
                }
            }
        }
    }
}
