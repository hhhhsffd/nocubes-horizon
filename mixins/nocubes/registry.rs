use super::*;
/// Install one registry snapshot for rendering, picking and physical queries.
pub fn install(facts: &BlockFacts, assets: &Assets) {
    let registry = assets.registry;
    let states: Vec<_> = facts
        .states
        .iter()
        .enumerate()
        .map(|(id, f)| {
            let plant = f.has_class("BushBlock") || f.has_class("VegetationBlock");
            horizon_core::nocubes::kinds::StateKind {
                full_block: f.full_collision && f.render_shape == RenderShape::Model,
                air: matches!(
                    f.name.as_ref(),
                    "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
                ),
                plant,
                short_plant: plant
                    && !f.has_class("DoublePlantBlock")
                    && !f.has_class("CropBlock")
                    && !f.has_class("StemBlock"),
                snow_layer: registry
                    .state(horizon_world::registry::BlockStateId(id as u32))
                    .is_some_and(|s| s.properties.contains_key("layers")),
                source_liquid_block: f.has_class("LiquidBlock")
                    && f.fluid_level == 8
                    && !f.fluid_falling,
                replaceable: f.replaceable,
                vine: f.has_class("VineBlock"),
            }
        })
        .collect();
    horizon_core::nocubes::kinds::install_resolver(states.len(), move |cfg, id| {
        use horizon_core::nocubes::{Mesher, Sample, snow_density};
        let Some(s) = registry.state(horizon_world::registry::BlockStateId(id)) else {
            return Sample::AIR;
        };
        let kind = states.get(id as usize).copied().unwrap_or_default();
        let property = |k: &str| s.properties.get(k).map(String::as_str);
        let default_smooth = kind.default_smooth(cfg.legacy_default(&s.name, property));
        let smooth = cfg.smoothable_with_default(&s.name, property, default_smooth);
        let mut density = s
            .properties
            .get("layers")
            .and_then(|s| s.parse::<u8>().ok())
            .map_or(1., snow_density);
        if cfg.mesher == Mesher::Wulferis && s.name != "minecraft:snow" {
            density = 1.;
        }
        Sample {
            state: id,
            smooth,
            density: if smooth { density } else { -1. },
            solid: if cfg.mesher == Mesher::Wulferis {
                kind.wulferis_barrier(smooth)
            } else {
                kind.legacy_solid(cfg.old_in_fluids)
            },
        }
    });
    let mut kinds = std::collections::HashMap::new();
    for s in &facts.states {
        let plant = s.has_class("BushBlock") || s.has_class("VegetationBlock");
        kinds
            .entry(s.name.to_string())
            .or_insert(horizon_core::nocubes::kinds::BlockKind {
                plant,
                short_plant: plant
                    && !s.has_class("DoublePlantBlock")
                    && !s.has_class("CropBlock")
                    && !s.has_class("StemBlock"),
                replaceable: s.replaceable,
                snow: s.has_class("SnowLayerBlock") || s.name.as_ref() == "minecraft:snow",
                vine: s.has_class("VineBlock"),
                single_sided: s.has_class("BeaconBeamBlock")
                    || s.has_class("NetherPortalBlock")
                    || s.has_class("EndPortalBlock")
                    || s.has_class("SnowLayerBlock"),
            });
    }
    horizon_core::nocubes::kinds::install(kinds);
}
