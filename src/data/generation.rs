//! Deterministic instantiation of production campaigns from authored geography.

use super::{
    economy::{Habitation, TroopKind},
    rules::Emblem,
    threats::{InitialThreat, ThreatKind},
    world::{
        DiplomaticState, FactionId, FactionSetup, FounderClass, FounderGrant, Relation,
        ResourceGrant, Scenario, ScenarioKind, SiteId, WorldLayout,
    },
    GameData,
};
use macroquad_toolkit::rng::SeededRng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionSetup {
    pub kingdom_name: String,
    pub emblem: Emblem,
    pub factions: usize,
    pub seed: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedProduction {
    pub scenario: Scenario,
    pub initial_threats: Vec<InitialThreat>,
}

impl ProductionSetup {
    pub fn validate(&self, data: &GameData) -> Result<(), String> {
        if !data.rules.valid_kingdom_name(&self.kingdom_name) {
            return Err("Kingdom name must be trimmed, nonempty and at most 32 characters.".into());
        }
        if !(data.rules.min_factions..=data.rules.max_factions).contains(&self.factions) {
            return Err(format!(
                "Choose between {} and {} total factions.",
                data.rules.min_factions, data.rules.max_factions
            ));
        }
        if !data
            .rules
            .emblems
            .iter()
            .any(|entry| entry.id == self.emblem)
        {
            return Err("Choose one of the eight botanical emblems.".into());
        }
        Ok(())
    }
}

impl WorldLayout {
    pub fn generate(
        &self,
        data: &GameData,
        setup: &ProductionSetup,
    ) -> Result<GeneratedProduction, String> {
        setup.validate(data)?;
        self.validate(&data.rules, &data.economy)?;
        let mut rng = SeededRng::new(setup.seed);
        let mut candidates = self.headquarters_candidates.clone();
        shuffle(&mut rng, &mut candidates);
        candidates.truncate(setup.factions);

        let mut sites = self.sites.clone();
        let candidate_set: std::collections::BTreeSet<_> = candidates.iter().copied().collect();
        for site in &mut sites {
            if candidate_set.contains(&site.id) {
                site.habitation = Habitation::Village;
            } else {
                site.habitation = generated_habitation(&mut rng);
            }
        }

        let mut emblems: Vec<_> = data
            .rules
            .emblems
            .iter()
            .map(|definition| (definition.id, definition.name.clone()))
            .filter(|(emblem, _)| *emblem != setup.emblem)
            .collect();
        shuffle(&mut rng, &mut emblems);
        let mut names = name_pairs(data);
        names.retain(|name| name != &setup.kingdom_name);
        shuffle(&mut rng, &mut names);
        if names.len() < setup.factions.saturating_sub(1) + setup.factions {
            return Err("human_names.json does not contain enough unique production names".into());
        }

        let mut factions = Vec::with_capacity(setup.factions);
        factions.push(founder_setup(
            FactionId(1),
            setup.kingdom_name.clone(),
            setup.emblem,
            candidates[0],
            names.remove(0),
            data,
        ));
        for (index, headquarters) in candidates.iter().copied().enumerate().skip(1) {
            let (emblem, _) = emblems
                .get(index - 1)
                .cloned()
                .ok_or("not enough distinct botanical emblems")?;
            let name = names.remove(0);
            factions.push(founder_setup(
                FactionId(index as u32 + 1),
                name,
                emblem,
                headquarters,
                names.remove(0),
                data,
            ));
        }
        for faction in &factions {
            let hq = sites
                .iter_mut()
                .find(|site| site.id == faction.headquarters)
                .ok_or("generated headquarters is missing from the layout")?;
            hq.controller = Some(faction.id);
            hq.habitation = Habitation::Village;
            hq.facilities.push(super::world::Facility::TrainingGround);
        }

        let mut relations = Vec::new();
        for left in 1..=setup.factions {
            for right in (left + 1)..=setup.factions {
                relations.push(Relation {
                    factions: [FactionId(left as u32), FactionId(right as u32)],
                    state: DiplomaticState::Peace,
                });
            }
        }
        let scenario = Scenario {
            schema_version: self.schema_version,
            content_version: self.content_version,
            kind: ScenarioKind::Production,
            name: "Kestrum Production Campaign".into(),
            seed: setup.seed,
            difficulty: data.rules.difficulty,
            player: FactionId(1),
            markers: self.markers.clone(),
            sites,
            routes: self.routes.clone(),
            factions,
            relations,
        };
        scenario.validate_generated(&data.rules, &data.economy)?;
        let initial_threats = production_threats(&scenario, &mut rng)?;
        data.threats
            .validate_initials(&scenario, &initial_threats)?;
        Ok(GeneratedProduction {
            scenario,
            initial_threats,
        })
    }
}

fn founder_setup(
    id: FactionId,
    name: String,
    emblem: Emblem,
    hq: SiteId,
    founder_name: String,
    data: &GameData,
) -> FactionSetup {
    let emblem_name = &data
        .rules
        .emblems
        .iter()
        .find(|entry| entry.id == emblem)
        .unwrap()
        .name;
    FactionSetup {
        id,
        name,
        army_name: format!("{emblem_name} Host"),
        emblem,
        headquarters: hq,
        capital: hq,
        resources: ResourceGrant::EconomyStartingResources,
        starting_formations: vec![TroopKind::Warriors, TroopKind::Spearmen, TroopKind::Archers],
        founder: FounderGrant {
            name: founder_name,
            age_years: 24,
            class: FounderClass::Officer,
            attached_to: TroopKind::Warriors,
            commander: true,
        },
    }
}

fn name_pairs(data: &GameData) -> Vec<String> {
    data.human_names
        .given_names
        .iter()
        .flat_map(|given| {
            data.human_names
                .family_names
                .iter()
                .map(move |family| format!("{given} {family}"))
        })
        .collect()
}

fn production_threats(
    scenario: &Scenario,
    rng: &mut SeededRng,
) -> Result<Vec<InitialThreat>, String> {
    let mut threats = Vec::with_capacity(scenario.factions.len() * 2);
    for faction in &scenario.factions {
        let mut neighbors: Vec<_> = scenario
            .routes
            .iter()
            .filter_map(|route| route.other_endpoint(faction.headquarters))
            .filter(|site| {
                scenario
                    .site(*site)
                    .is_some_and(|site| site.controller.is_none())
            })
            .collect();
        neighbors.sort();
        shuffle(rng, &mut neighbors);
        if neighbors.len() < 2 {
            return Err(format!(
                "headquarters {:?} has fewer than two threat sites",
                faction.headquarters
            ));
        }
        // Keep one first journey open inside the home region. Its other nearby
        // threat waits one route farther out, rather than sealing both exits.
        let home = scenario.site(faction.headquarters).unwrap();
        let safe = neighbors
            .iter()
            .copied()
            .filter(|id| {
                scenario
                    .site(*id)
                    .is_some_and(|site| site.marker == home.marker)
            })
            .min_by_key(|id| {
                (
                    scenario
                        .routes
                        .iter()
                        .find(|route| route.other_endpoint(home.id) == Some(*id))
                        .map(|route| route.terrain_cost)
                        .unwrap_or(u32::MAX),
                    *id,
                )
            });
        if let Some(safe) = safe {
            let mut farther: Vec<_> = scenario
                .routes
                .iter()
                .flat_map(|route| {
                    neighbors
                        .iter()
                        .filter_map(|site| route.other_endpoint(*site))
                })
                .filter(|id| {
                    !neighbors.contains(id)
                        && scenario
                            .site(*id)
                            .is_some_and(|site| site.controller.is_none())
                })
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect();
            shuffle(rng, &mut farther);
            let next = farther
                .first()
                .copied()
                .ok_or("regional start needs a threat within two routes")?;
            neighbors.retain(|id| *id != safe);
            neighbors.truncate(1);
            neighbors.push(next);
        }
        threats.push(InitialThreat {
            site: neighbors[0],
            kind: ThreatKind::Bandits,
        });
        threats.push(InitialThreat {
            site: neighbors[1],
            kind: ThreatKind::Wildlife,
        });
    }
    Ok(threats)
}

fn generated_habitation(rng: &mut SeededRng) -> Habitation {
    match rng.below(100) {
        0..=39 => Habitation::Unsettled,
        40..=49 => Habitation::Camp,
        50..=59 => Habitation::Outpost,
        60..=77 => Habitation::Hamlet,
        78..=96 => Habitation::Village,
        _ => Habitation::Town,
    }
}

fn shuffle<T>(rng: &mut SeededRng, values: &mut [T]) {
    for index in (1..values.len()).rev() {
        values.swap(index, rng.below(index + 1));
    }
}
