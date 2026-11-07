fn runtime_settings(mod_id: &str, buttons: &mut Vec<String>) {
    if mod_id == "nocubes" {
        // Make Horizon expose the Settings entry; our screen hooks draw and
        // handle the settings page itself, including on the title screen.
        buttons.push(horizon_core::nocubes::ui::text("nocubes.ui.title"));
        *horizon_core::nocubes::ui::screen()
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Default::default();
    }
}
