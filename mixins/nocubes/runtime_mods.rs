fn runtime_mods(mods: &mut Vec<NativeMod>) {
    if !mods.iter().any(|m| m.id == "nocubes") {
        mods.push(NativeMod {
            id: "nocubes".into(),
            title: "NoCubes 0.2.6".into(),
            description: "Smooth blocks with configurable rendering, collisions and block rules."
                .into(),
            ..NativeMod::default()
        });
    }
}
