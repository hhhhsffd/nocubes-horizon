use nocubes_engine::*;
fn stone() -> Sample {
    Sample {
        state: 1,
        density: 1.,
        smooth: true,
        solid: true,
    }
}
#[test]
fn every_binary_cube_case_stays_finite_for_both_iso_meshers() {
    for mask in 0..256 {
        let world = |p: Pos| {
            if p.iter().all(|v| (0..=1).contains(v))
                && mask & (1 << (p[0] + 2 * p[1] + 4 * p[2])) != 0
            {
                stone()
            } else {
                Sample::AIR
            }
        };
        for mesher in [
            Mesher::SurfaceNets,
            Mesher::SurfaceNets2x,
            Mesher::MarchingCubes,
            Mesher::MarchingCubes2x,
        ] {
            let cfg = Config {
                mesher,
                ..Config::default()
            };
            let faces = generate(&world, [-2; 3], [3; 3], &cfg);
            assert!(
                faces
                    .iter()
                    .flat_map(|f| f.vertices)
                    .flatten()
                    .all(f32::is_finite),
                "{mesher:?} case {mask}"
            );
        }
    }
}
#[test]
fn legacy_roughness_uses_world_coordinates_across_rebased_sections() {
    let cfg = Config {
        mesher: Mesher::OldNoCubes,
        old_slopes: false,
        roughness: 0.5,
        ..Config::default()
    };
    let world = |p: Pos| if p[1] <= 0 { stone() } else { Sample::AIR };
    let global = generate(&world, [-2, -1, -2], [2, 1, 2], &cfg);
    let offset = [-16, 0, -16];
    let local = generate_at(
        &|p| world(std::array::from_fn(|i| p[i] + offset[i])),
        [14, -1, 14],
        [18, 1, 18],
        &cfg,
        offset,
    );
    assert_eq!(global.len(), local.len());
    for (a, b) in global.iter().zip(local) {
        for (a, b) in a.vertices.iter().zip(b.vertices) {
            for i in 0..3 {
                assert!((a[i] - b[i] - offset[i] as f32).abs() < 0.00001);
            }
        }
    }
}
#[test]
fn material_selection_and_light_fallback_follow_source_rules() {
    let face = Face {
        vertices: [[0., 1., 0.], [0., 1., 1.], [1., 1., 1.], [1., 1., 0.]],
        block: [99; 3],
        state: 1,
    };
    assert_eq!(material_position(&face, Mesher::SurfaceNets), [0; 3]);
    assert_eq!(material_position(&face, Mesher::OldNoCubes), [99; 3]);
    assert_eq!(
        vertex_light_position([0.5, 1., 0.5], [0., 1., 0.]),
        [0, 2, 0]
    );
    assert_eq!(
        vertex_light([0.5, 1., 0.5], [0., 1., 0.], &|p| if p == [0, 1, 0] {
            0x1234
        } else {
            0
        }),
        0x1234
    );
}
#[test]
fn original_legacy_mesher_discards_isolated_blocks() {
    let cfg = Config {
        mesher: Mesher::OldNoCubes,
        ..Config::default()
    };
    assert!(
        generate(
            &|p| if p == [0; 3] { stone() } else { Sample::AIR },
            [-1; 3],
            [2; 3],
            &cfg
        )
        .is_empty()
    );
}
#[test]
fn short_snow_layers_are_negative_density() {
    // ModUtil.mapSnowHeight: layers 1 through 8, not geometric height/8.
    let expected = [-1., -0.75, -0.5, -0.25, 0., 0.25, 0.5, 0.75];
    for (layer, want) in (1..=8).zip(expected) {
        assert_eq!(snow_density(layer), want);
    }
}
#[test]
fn input_bindings_and_permissions_match_creator_controls() {
    assert!(ui::binding_matches("Ctrl+KeyO", "KeyO", 2));
    assert!(!ui::binding_matches("KeyN", "KeyN", 1));
    assert!(ui::binding_matches("Shift+KeyN", "KeyN", 1));
    assert!(!ui::edit_authorized(1, false));
    assert!(ui::edit_authorized(2, false));
    assert!(ui::edit_authorized(0, true));
}
#[test]
fn exact_state_overrides_and_optional_defaults_work() {
    let props = std::collections::BTreeMap::from([("facing".into(), "north".into())]);
    let mut cfg = Config::default();
    cfg.blacklist.push("minecraft:stone".into());
    cfg.state_overrides
        .insert("minecraft:stone[facing=north]".into(), true);
    assert!(cfg.smoothable("minecraft:stone", &props));
    assert!(!cfg.smoothable("minecraft:stone", &Default::default()));
    assert!(cfg.smoothable("iceandfire:sapphire_ore", &Default::default()));
}

#[test]
fn collision_shapes_keep_fractional_coordinates_far_from_world_origin() {
    let world = |p: Pos| if p[1] <= 0 { stone() } else { Sample::AIR };
    for mesher in [
        Mesher::SurfaceNets,
        Mesher::SurfaceNets2x,
        Mesher::MarchingCubes,
        Mesher::MarchingCubes2x,
        Mesher::Wulferis,
    ] {
        let cfg = Config {
            mesher,
            ..Config::default()
        };
        assert_eq!(
            collision_boxes(&world, [0; 3], &cfg),
            collision_boxes(&world, [20_000_000, 0, -20_000_000], &cfg),
            "{mesher:?}"
        );
    }
    let cfg = Config {
        mesher: Mesher::OldNoCubes,
        old_slopes: false,
        ..Config::default()
    };
    let p = [20_000_000, 0, -20_000_000];
    assert_eq!(
        collision_boxes(&world, p, &cfg),
        collision_boxes_at(&world, [0; 3], &cfg, p)
    );
}

#[test]
fn cube_and_chamfer_topology_matches_upstream_faces() {
    let isolated = |p| if p == [0; 3] { stone() } else { Sample::AIR };
    let pair = |p: Pos| {
        if p[1] == 0 && p[2] == 0 && (0..2).contains(&p[0]) {
            stone()
        } else {
            Sample::AIR
        }
    };
    for (mesher, expected) in [
        (Mesher::CullingCubic, 6),
        (Mesher::StupidCubic, 6),
        (Mesher::CullingChamfer, 18),
    ] {
        let cfg = Config {
            mesher,
            ..Config::default()
        };
        assert_eq!(
            generate(&isolated, [0; 3], [1; 3], &cfg).len(),
            expected,
            "{mesher:?}"
        );
    }
    for (mesher, expected) in [(Mesher::CullingCubic, 10), (Mesher::StupidCubic, 12)] {
        let cfg = Config {
            mesher,
            ..Config::default()
        };
        assert_eq!(
            generate(&pair, [0; 3], [2, 1, 1], &cfg).len(),
            expected,
            "{mesher:?}"
        );
    }
}

#[test]
fn legacy_fluid_solidity_only_excludes_source_liquid_blocks() {
    use nocubes_engine::kinds::StateKind;
    let source = StateKind {
        source_liquid_block: true,
        ..Default::default()
    };
    assert!(!source.legacy_solid(true));
    assert!(source.legacy_solid(false));
    // Both flowing liquid blocks and waterlogged full blocks remain solid for
    // this legacy slope predicate, despite containing fluid.
    let flowing = StateKind::default();
    let waterlogged = StateKind::default();
    assert!(flowing.legacy_solid(true));
    assert!(waterlogged.legacy_solid(true));
    assert!(
        !StateKind {
            snow_layer: true,
            ..Default::default()
        }
        .legacy_solid(false)
    );
    assert!(
        !StateKind {
            plant: true,
            ..Default::default()
        }
        .legacy_solid(false)
    );
}
#[test]
fn wulferis_other_render_pass_is_soft_even_for_nonreplaceable_blocks() {
    use nocubes_engine::kinds::StateKind;
    let glass = StateKind::default();
    assert!(glass.wulferis_barrier(false));
    assert!(!glass.wulferis_barrier(true));
    assert!(
        StateKind {
            vine: true,
            ..Default::default()
        }
        .wulferis_barrier(true)
    );
}
#[test]
fn legacy_faces_are_not_culled_against_partial_snow_density() {
    let world = |p| {
        if p == [0; 3] {
            stone()
        } else if p == [0, 1, 0] {
            Sample {
                state: 2,
                density: snow_density(2),
                smooth: true,
                solid: false,
            }
        } else {
            Sample::AIR
        }
    };
    let cfg = Config {
        mesher: Mesher::OldNoCubes,
        roughness: 0.,
        ..Config::default()
    };
    assert_eq!(generate(&world, [0; 3], [1; 3], &cfg).len(), 6);
}
