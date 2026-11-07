use std::{
    fs,
    io::{self, Read, Write},
    path::Path,
};
fn tree(root: &Path, path: &Path, zip: &mut zip::ZipWriter<fs::File>) -> io::Result<()> {
    let mut entries = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            tree(root, &path, zip)?;
        } else {
            let relative = path
                .strip_prefix(root)
                .unwrap()
                .to_str()
                .unwrap()
                .replace('\\', "/");
            if relative.ends_with("Cargo.lock") {
                continue;
            }
            zip.start_file(
                relative,
                zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated)
                    .unix_permissions(0o644),
            )?;
            zip.write_all(&fs::read(path)?)?;
        }
    }
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("horizon-native.json"))?)?;
    let id = metadata["id"].as_str().ok_or("missing native mod id")?;
    let version = metadata["version"]
        .as_str()
        .ok_or("missing native mod version")?;
    let destination = root.join("dist").join(format!("{id}-{version}.hnmod"));
    fs::create_dir_all(destination.parent().unwrap())?;
    let mut zip = zip::ZipWriter::new(fs::File::create(&destination)?);
    zip.start_file(
        "horizon-native.json",
        zip::write::SimpleFileOptions::default(),
    )?;
    zip.write_all(&fs::read(root.join("horizon-native.json"))?)?;
    tree(root, &root.join("mixins"), &mut zip)?;
    zip.finish()?;
    let mut archive = zip::ZipArchive::new(fs::File::open(&destination)?)?;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let name = entry.name().to_owned();
        let mut packed = Vec::new();
        entry.read_to_end(&mut packed)?;
        if packed != fs::read(root.join(&name))? {
            return Err(format!("archive source differs from workspace: {name}").into());
        }
    }
    println!("verified {} files match the workspace", archive.len());
    println!("{}", destination.display());
    Ok(())
}
