use super::*;
use horizon_client::mobs::{MobType, Part, Polygon};
use std::sync::{Arc, Mutex, OnceLock, Weak};

type Models = std::collections::VecDeque<(Weak<Part>, Arc<Part>)>;
fn models() -> &'static Mutex<Models> {
    static MODELS: OnceLock<Mutex<Models>> = OnceLock::new();
    MODELS.get_or_init(Mutex::default)
}
fn rounded(base: &Arc<Part>) -> Arc<Part> {
    let mut cache = models().lock().unwrap_or_else(|e| e.into_inner());
    cache.retain(|(source, _)| source.strong_count() > 0);
    if let Some(index) = cache
        .iter()
        .position(|(source, _)| std::ptr::eq(source.as_ptr(), Arc::as_ptr(base)))
    {
        let entry = cache.remove(index).unwrap();
        let result = entry.1.clone();
        cache.push_back(entry);
        return result;
    }
    let mut result = (**base).clone();
    round_parts(&mut result);
    let result = Arc::new(result);
    if cache.len() >= 32 {
        cache.pop_front();
    }
    cache.push_back((Arc::downgrade(base), result.clone()));
    result
}
fn round_parts(part: &mut Part) {
    // Only the avatar's body/skin layers, not a cape, item, armour or arbitrary
    // custom attachment. Preserve poses, names, hierarchy and skin UVs.
    if matches!(
        part.name.as_str(),
        "head"
            | "body"
            | "right_arm"
            | "left_arm"
            | "right_leg"
            | "left_leg"
            | "hat"
            | "jacket"
            | "left_sleeve"
            | "right_sleeve"
            | "left_pants"
            | "right_pants"
    ) {
        let faces: Vec<_> = part
            .polygons
            .iter()
            .map(|p| horizon_core::nocubes::player::SkinQuad {
                normal: p.normal,
                vertices: p.vertices,
            })
            .collect();
        if let Some(faces) = horizon_core::nocubes::player::rounded_skin_box(&faces) {
            part.polygons = faces
                .into_iter()
                .map(|(source, q)| Polygon {
                    normal: q.normal,
                    vertices: q.vertices,
                    cull_back: part.polygons[source].cull_back,
                })
                .collect();
        }
    }
    for child in &mut part.children {
        round_parts(child);
    }
}
fn base_model(kind: &Arc<MobType>, slim: bool) -> &Arc<Part> {
    kind.layer(if slim { "player_slim#main" } else { "main" })
        .unwrap_or(&kind.model)
}
impl Game<'_> {
    pub(super) fn nocubes_player_model(
        &self,
        kind: &Arc<MobType>,
        slim: bool,
    ) -> Option<Arc<Part>> {
        if !self.native_on("nocubes")
            || !horizon_core::nocubes::config().player_model
            || &*kind.id != "minecraft:player"
        {
            return None;
        }
        // The caller still chooses each player's own downloaded/default skin.
        // A rounded outer layer takes precedence over the pixel-extrusion option.
        Some(rounded(base_model(kind, slim)))
    }
    pub(super) fn nocubes_player_arm(
        &self,
        left: bool,
        m: glam::Mat4,
        light: u32,
        out: &mut Vec<Vertex>,
    ) -> bool {
        if !self.native_on("nocubes") || !horizon_core::nocubes::config().player_model {
            return false;
        }
        let Some(player) = &self.player_model else {
            return false;
        };
        let model = rounded(base_model(&player.kind, player.slim));
        let name = if left { "left_arm" } else { "right_arm" };
        let Some(arm) = model.children.iter().find(|p| p.name == name) else {
            return false;
        };
        let sprites = &self.gui.atlas.sprites;
        let Some(r) = player
            .skin
            .as_deref()
            .and_then(|s| sprites.get(s))
            .or_else(|| sprites.get("entity/player/wide/steve"))
            .copied()
        else {
            return false;
        };
        let pose = |part: &Part| {
            let mut pose = part.pose;
            pose[3] = 0.;
            pose[4] = 0.;
            pose[5] = if part.name == arm.name {
                if left { -0.1 } else { 0.1 }
            } else {
                0.
            };
            pose
        };
        horizon_client::mobs::render_model(arm, m, &pose, [r.u0, r.v0, r.u1, r.v1], light, 0, out);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cached_models_preserve_pose_names_hierarchy_and_original_source() {
        let source = Arc::new(Part {
            name: "root".into(),
            pose: [1.; 9],
            polygons: Vec::new(),
            children: vec![Part {
                name: "cape".into(),
                pose: [2.; 9],
                polygons: Vec::new(),
                children: Vec::new(),
            }],
        });
        let first = rounded(&source);
        assert!(Arc::ptr_eq(&first, &rounded(&source)));
        assert!(!Arc::ptr_eq(&first, &source));
        assert_eq!(first.pose, source.pose);
        assert_eq!(first.children[0].pose, source.children[0].pose);
        assert_eq!(first.children[0].name, "cape");
        assert!(source.children[0].polygons.is_empty());
    }
}
