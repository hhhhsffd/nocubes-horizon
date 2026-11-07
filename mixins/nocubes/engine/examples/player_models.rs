//! Validate the smoother against Horizon's locally prepared player models.
//! Does not copy or distribute Minecraft's exported model data.
use nocubes_engine::player::{SkinQuad, rounded_skin_box};
use serde_json::Value;

fn walk(name: &str, value: &Value, count: &mut usize, quads: &mut usize) {
    if matches!(
        name,
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
        let normals: Vec<[f32; 3]> = serde_json::from_value(value["normals"].clone()).unwrap();
        let vertices: Vec<[[f32; 5]; 4]> =
            serde_json::from_value(value["polygons"].clone()).unwrap();
        let faces: Vec<_> = normals
            .into_iter()
            .zip(vertices)
            .map(|(normal, vertices)| SkinQuad { normal, vertices })
            .collect();
        let output = rounded_skin_box(&faces)
            .unwrap_or_else(|| panic!("prepared {name} is not a supported box"));
        assert_eq!(output.len(), 54, "{name}");
        *count += 1;
        *quads += output.len();
    }
    if let Some(children) = value["children"].as_object() {
        for (name, part) in children {
            walk(name, part, count, quads);
        }
    }
}
fn main() {
    let file = std::env::args()
        .nth(1)
        .expect("path to locally prepared entity-models.json");
    let doc: Value = serde_json::from_slice(&std::fs::read(file).unwrap()).unwrap();
    let mut layers = Vec::new();
    for name in ["minecraft:player#main", "minecraft:player_slim#main"] {
        let mut parts = 0;
        let mut polygons = 0;
        walk(
            "root",
            doc.get(name).expect("prepared player layer"),
            &mut parts,
            &mut polygons,
        );
        assert_eq!(parts, 12, "{name}");
        layers.push(serde_json::json!({"layer":name,"rounded_parts":parts,"polygons":polygons}));
    }
    println!("{}", serde_json::json!({"passed":true,"layers":layers}));
}
