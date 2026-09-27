//! Load and prepare both UI font atlases before drawing the first frame.

use super::*;

pub(super) async fn load(data: &GameData) -> Result<AssetManager, String> {
    let mut assets = AssetManager::new();
    assets
        .load_texture_with_filter("atlas", &data.presentation.map_path, FilterMode::Linear)
        .await?;
    assets
        .load_font("cinzel", &data.presentation.font_path)
        .await?;
    assets
        .load_font("body", &data.presentation.body_font_path)
        .await?;
    // Prepare all fixed UI sizes together before the first visible frame.
    // Runtime names and messages are prepared separately before UI drawing.
    let mut characters: Vec<char> = (b' '..=b'~').map(char::from).collect();
    characters.extend("…—–×·→".chars());
    for text in data.presentation.text.values().chain([
        &data.presentation.title,
        &data.presentation.subtitle,
        &data.presentation.edition,
    ]) {
        characters.extend(text.chars());
    }
    characters.sort_unstable();
    characters.dedup();
    let common_text: String = characters.into_iter().collect();
    for (key, sizes) in [
        ("cinzel", &[15, 17, 18, 19, 20, 21, 24, 28, 30][..]),
        ("body", &[16, 18, 19, 20, 21, 23, 25][..]),
    ] {
        let font = assets
            .get_font(key)
            .ok_or_else(|| format!("Loaded UI font {key} is unavailable"))?;
        let mut samples: Vec<_> = sizes
            .iter()
            .map(|size| (*size, common_text.as_str()))
            .collect();
        if key == "cinzel" {
            samples.push((76, &data.presentation.title));
            samples.extend(
                data.presentation
                    .geography
                    .iter()
                    .map(|label| (label.size as u16, label.name.as_str())),
            );
        }
        macroquad_toolkit::ui::prepare_font_text(font, &samples);
    }
    next_frame().await;
    Ok(assets)
}
