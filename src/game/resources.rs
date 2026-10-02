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
    for text in data
        .game_text
        .text
        .values()
        .chain(data.presentation.map.text.values())
        .chain([
            &data.game_text.title,
            &data.game_text.subtitle,
            &data.game_text.edition,
        ])
        .chain(data.game_text.seasons.iter())
        .chain(data.game_text.geography.values())
    {
        characters.extend(text.chars());
    }
    characters.sort_unstable();
    characters.dedup();
    let common_text: String = characters.into_iter().collect();
    for (key, sizes) in [
        ("cinzel", &[15, 17, 18, 19, 20, 21, 24, 28, 30][..]),
        ("body", &[14, 16, 17, 18, 19, 20, 21, 23, 25][..]),
    ] {
        let font = assets
            .get_font(key)
            .ok_or_else(|| format!("Loaded UI font {key} is unavailable"))?;
        let mut samples: Vec<_> = sizes
            .iter()
            .map(|size| (*size, common_text.as_str()))
            .collect();
        if key == "cinzel" {
            samples.push((88, data.game_text.title.as_str()));
            samples.extend(data.presentation.geography.iter().filter_map(|label| {
                data.game_text
                    .geography
                    .get(&label.id)
                    .map(|name| (label.size as u16, name.as_str()))
            }));
        }
        macroquad_toolkit::ui::prepare_font_text(font, &samples);
    }
    next_frame().await;
    Ok(assets)
}
