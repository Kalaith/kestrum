//! Create a separate native review save through ordinary campaign and catalogue APIs.

#[path = "prepare_midgame/campaign.rs"]
mod campaign;

use kestrum::{
    data::GameData,
    state::{
        persistence::{SaveLibrary, SAVE_NAMESPACE},
        Campaign,
    },
};
use macroquad_toolkit::persistence::{get_app_data_path, IndexedKeyStore, WriterStatus};

fn main() -> Result<(), String> {
    let data = GameData::load()?;
    let mut strategic = campaign::generate(&data)?;
    let mut store = IndexedKeyStore::new("kestrum", SAVE_NAMESPACE)?;
    if store.poll_writer()? != WriterStatus::Ready {
        return Err("Close the running native game before creating the review save.".into());
    }
    let mut library = SaveLibrary::open(&mut store, &data)?;
    strategic.campaign_id = library.allocate_campaign_id(&mut store)?;
    let snapshot = Campaign::Strategic(Box::new(strategic));
    let prepared = library.prepare_manual(
        &mut store,
        &data,
        &snapshot,
        "Briarhold - Midgame Battle Review",
        None,
    )?;
    let receipt = library.write(&mut store, &data, &prepared)?;
    if !receipt.warnings.is_empty() {
        return Err(receipt.warnings.join(" "));
    }
    let restored = library.load(&mut store, &data, receipt.id)?;
    if restored != snapshot {
        return Err("The review save did not reload identically.".into());
    }
    let saved = restored.strategic().ok_or("Missing strategic save")?;
    println!(
        "Saved #{}: {} completed rounds, Year {}, {} retained battles, pending battle={}",
        receipt.id,
        saved.completed_rounds,
        saved.year(data.presentation.start_year),
        saved.battles.len(),
        saved.pending_battle.is_some(),
    );
    let view =
        kestrum::engine::project_map(saved, saved.player).map_err(|error| error.to_string())?;
    let settlements = saved
        .world
        .sites
        .iter()
        .filter(|site| {
            site.controller == Some(saved.player)
                && site.habitation >= kestrum::data::economy::Habitation::Outpost
        })
        .count();
    let people = saved
        .people
        .values()
        .filter(|person| person.faction == saved.player && person.is_alive())
        .count();
    let armies = saved
        .armies
        .values()
        .filter(|army| army.faction == saved.player)
        .count();
    println!("Revealed {}/{} markers and {}/{} sites; {settlements} settlements, {armies} armies, {people} living named people, {} known factions",
        view.world.markers.len(), saved.world.markers.len(), view.world.sites.len(), saved.world.sites.len(), view.factions.len());
    for person in saved
        .people
        .values()
        .filter(|person| person.faction == saved.player && person.is_alive())
    {
        println!(
            "{}: {:?}, age {}",
            person.name,
            person.class,
            person.age_years(saved.completed_rounds)
        );
    }
    let borders: std::collections::BTreeSet<_> = saved
        .world
        .sites
        .iter()
        .filter(|site| site.controller == Some(saved.player))
        .flat_map(|site| saved.world.adjacent_sites(site.id))
        .filter_map(|id| saved.world.site(id)?.controller)
        .filter(|faction| *faction != saved.player && saved.is_independent(*faction))
        .collect();
    println!(
        "Bordering kingdoms: {}",
        borders
            .iter()
            .map(|faction| saved.factions[faction].name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    if let Some(path) = get_app_data_path("kestrum", "") {
        println!("Native save catalogue: {}", path.display());
    }
    println!("cargo run resumes this save. Use cargo run -- --title for the title screen.");
    Ok(())
}
