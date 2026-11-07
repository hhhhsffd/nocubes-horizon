use super::*;
// Weak owners keep revisions independent across clients and world changes,
// without changing Horizon's constructor/layout contracts. Dead owners are
// removed on every check; only live LOD workers can occupy entries.
static REVISIONS: Mutex<Vec<(std::sync::Weak<Shared>, u64)>> = Mutex::new(Vec::new());
impl Lod {
    /// The server and client share settings, but each LOD owner must observe
    /// every revision independently. Never consume a client update globally.
    pub fn nocubes_sync(&mut self) {
        let revision = horizon_core::nocubes::revision();
        let changed = {
            let mut seen = REVISIONS.lock().unwrap_or_else(|e| e.into_inner());
            seen.retain(|(owner, _)| owner.strong_count() > 0);
            if let Some((_, previous)) = seen
                .iter_mut()
                .find(|(owner, _)| std::ptr::eq(owner.as_ptr(), Arc::as_ptr(&self.shared)))
            {
                let changed = *previous != revision;
                *previous = revision;
                changed
            } else {
                seen.push((Arc::downgrade(&self.shared), revision));
                true
            }
        };
        if changed {
            self.nocubes_remesh();
        }
    }
    /// Retain the sampled columns and quadtree; only rebuild visual meshes.
    pub fn nocubes_remesh(&mut self) {
        let keys: Vec<_> = self.active.iter().copied().collect();
        for key in keys {
            self.cancel_mesh(key);
            self.mark_mesh(key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_lod_owners_each_observe_a_shared_settings_revision() {
        let key = TileKey {
            x: 0,
            z: 0,
            level: 3,
        };
        let mut first = Lod::scheduler_for_test();
        let mut second = Lod::scheduler_for_test();
        for lod in [&mut first, &mut second] {
            lod.active.insert(key);
            lod.ready
                .insert(key, Arc::new(vec![Column::default(); 256]));
        }
        first.nocubes_sync();
        second.nocubes_sync();
        assert!(first.dirty.contains_key(&key));
        assert!(
            second.dirty.contains_key(&key),
            "another owner must not consume this owner's revision"
        );
        first.dirty.clear();
        first.nocubes_sync();
        assert!(
            first.dirty.is_empty(),
            "unchanged settings must not remesh every frame"
        );
    }
    #[test]
    fn visual_changes_remesh_active_tiles_without_discarding_reduced_columns() {
        let mut lod = Lod::scheduler_for_test();
        let key = TileKey {
            x: 0,
            z: 0,
            level: 3,
        };
        let columns = Arc::new(vec![Column::default(); 256]);
        lod.active.insert(key);
        lod.ready.insert(key, columns.clone());
        lod.nocubes_remesh();
        let first = lod.mesh_version[&key];
        assert!(lod.dirty.contains_key(&key));
        assert!(Arc::ptr_eq(&columns, &lod.ready[&key]));
        lod.nocubes_remesh();
        assert!(
            lod.mesh_version[&key] > first,
            "old in-flight results must be rejected"
        );
    }
}
