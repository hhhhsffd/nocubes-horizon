use nocubes_engine::{Config, kinds::StateKind, ui};

#[test]
fn default_controls_are_unbound_and_legacy_controls_do_not_reactivate() {
    let default = Config::default();
    assert!(default.block_toggle_key.is_empty());
    let old: Config = serde_json::from_value(serde_json::json!({
        "info_message": true, "smooth_key": "KeyN", "state_key": "Shift+KeyN",
        "visuals_key": "Ctrl+KeyV", "config_key": "Ctrl+KeyO"
    }))
    .unwrap();
    assert_eq!(old, default);
    let saved = serde_json::to_value(old).unwrap();
    for key in [
        "info_message",
        "smooth_key",
        "state_key",
        "visuals_key",
        "config_key",
    ] {
        assert!(saved.get(key).is_none());
    }
    assert_eq!(saved["block_toggle_key"], "");
    assert!(serde_json::from_value::<Config>(serde_json::json!({"typo": true})).is_err());
}

#[test]
fn captured_binding_round_trips_and_empty_never_matches() {
    let binding = ui::binding_chord("KeyJ", 3).unwrap();
    assert!(ui::binding_matches(&binding, "KeyJ", 3));
    assert!(!ui::binding_matches(&binding, "KeyJ", 0));
    assert!(!ui::binding_matches(&binding, "KeyK", 3));
    assert!(!ui::binding_matches("", "KeyJ", 0));
    assert!(!ui::binding_matches("", "", 0));
    for key in ["ShiftLeft", "ControlRight", "AltLeft", "SuperRight"] {
        assert!(ui::binding_chord(key, 15).is_none());
    }
    let cfg = Config {
        block_toggle_key: binding,
        ..Config::default()
    };
    let saved = serde_json::to_vec(&cfg).unwrap();
    assert_eq!(serde_json::from_slice::<Config>(&saved).unwrap(), cfg);
    assert!(!ui::server_field("block_toggle_key"));
}

#[test]
fn full_blocks_default_to_smooth_but_decorations_and_partial_shapes_do_not() {
    let cfg = Config::default();
    for name in [
        "minecraft:stone_bricks",
        "minecraft:oak_planks",
        "minecraft:glass",
        "mod:machine",
    ] {
        let kind = StateKind {
            full_block: true,
            ..Default::default()
        };
        assert!(cfg.smoothable_with_default(name, |_| None, kind.default_smooth(false)));
    }
    for kind in [
        StateKind::default(), // Slabs, stairs, fences, torches, rails, etc.
        StateKind {
            air: true,
            ..Default::default()
        },
        StateKind {
            plant: true,
            full_block: true,
            ..Default::default()
        },
        StateKind {
            vine: true,
            ..Default::default()
        },
        StateKind {
            source_liquid_block: true,
            ..Default::default()
        },
    ] {
        assert!(!kind.default_smooth(false));
    }
    assert!(
        StateKind {
            snow_layer: true,
            ..Default::default()
        }
        .default_smooth(true)
    );
}

#[test]
fn toggled_block_rules_override_new_defaults_and_do_not_affect_other_types() {
    let mut cfg = Config::default();
    cfg.blacklist.push("minecraft:oak_planks".into());
    assert!(!cfg.smoothable_with_default("minecraft:oak_planks", |_| None, true));
    assert!(cfg.smoothable_with_default("minecraft:stone_bricks", |_| None, true));
    cfg.state_overrides
        .insert("oak_planks[axis=y]".into(), true);
    assert!(cfg.smoothable_with_default("minecraft:oak_planks", |_| Some("y"), true));
    cfg.use_defaults = false;
    assert!(!cfg.smoothable_with_default("minecraft:stone_bricks", |_| None, true));
    cfg.whitelist.push("minecraft:stone_bricks".into());
    assert!(cfg.smoothable_with_default("minecraft:stone_bricks", |_| None, true));
}
