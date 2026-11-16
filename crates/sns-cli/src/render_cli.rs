use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
pub fn execute(source: &Path, args: &[String]) -> Result<u8, String> {
    let mut output = None;
    let mut diagram = false;
    let mut options = sns_render::PngOptions::default();
    let mut seen = BTreeSet::new();
    let mut pairs = args.chunks_exact(2);
    for pair in &mut pairs {
        if !seen.insert(&pair[0]) {
            return Err(format!("duplicate option: {}", pair[0]));
        }
        match pair[0].as_str() {
            "--output" => output = Some(PathBuf::from(&pair[1])),
            "--view" => {
                diagram = match pair[1].as_str() {
                    "icons" => false,
                    "diagram" => true,
                    _ => return Err("--view must be icons or diagram".into()),
                }
            }
            "--size" => options.longest_side = pair[1].parse().map_err(|_| "invalid --size")?,
            "--background" => {
                options.transparent = match pair[1].as_str() {
                    "transparent" => true,
                    "canvas" => false,
                    _ => return Err("--background must be transparent or canvas".into()),
                }
            }
            _ => return Err(format!("unknown option: {}", pair[0])),
        }
    }
    if !pairs.remainder().is_empty() {
        return Err("missing render option value".into());
    }
    let output = output.ok_or("missing --output")?;
    let resolved = if output.exists() {
        output.canonicalize().map_err(|e| e.to_string())?
    } else {
        output
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."))
            .canonicalize()
            .map_err(|e| e.to_string())?
            .join(output.file_name().ok_or("invalid output path")?)
    };
    if source.canonicalize().map_err(|e| e.to_string())? == resolved {
        return Err("render.source_path_conflict".into());
    }
    let project = sns_project::load_project(source).map_err(|e| e.to_string())?;
    let scene = sns_render::Scene::build(&project, diagram).map_err(|e| e.to_string())?;
    let image = sns_render::render_png(&scene, options).map_err(|e| e.to_string())?;
    sns_project::atomic_write_runtime(&output, &image.bytes).map_err(|e| e.to_string())?;
    println!(
        "{}",
        serde_json::json!({"width":image.width,"height":image.height,"bytes":image.bytes.len(),"devices":scene.counts().devices,"links":scene.counts().links,"annotations":scene.counts().annotations,"clipped_texts":scene.counts().clipped_texts,"view":if diagram{"diagram"}else{"icons"},"transparent":options.transparent})
    );
    Ok(0)
}
