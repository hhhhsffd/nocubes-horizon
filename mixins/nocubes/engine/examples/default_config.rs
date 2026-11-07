fn main() {
    println!(
        "{}",
        serde_json::to_string_pretty(&nocubes_engine::Config::default())
            .expect("default configuration")
    );
}
