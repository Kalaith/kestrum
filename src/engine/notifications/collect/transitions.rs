//! Accepted state boundaries catch public conditions with no individual fact.

use super::*;
use crate::data::world::{Facility, MilitaryLayer};
use crate::state::threat::ThreatStatus;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn collect(
    before: &StrategicCampaign,
    candidate: &mut StrategicCampaign,
    data: &GameData,
) -> Result<(), String> {
    control(before, candidate, data)?;
    territory_states(before, candidate, data)?;
    site_reclamation(before, candidate, data)?;
    local_threats(before, candidate, data)?;
    facilities(before, candidate, data)?;
    supply(before, candidate, data)?;
    economy(before, candidate, data)?;
    Ok(())
}

fn control(
    before: &StrategicCampaign,
    candidate: &mut StrategicCampaign,
    data: &GameData,
) -> Result<(), String> {
    let player = candidate.player;
    let ids = before
        .world
        .sites
        .iter()
        .chain(candidate.world.sites.iter())
        .map(|site| site.id)
        .collect::<BTreeSet<_>>();
    for id in ids {
        let old = before.world.site(id).and_then(|site| site.controller);
        let new = candidate.world.site(id).and_then(|site| site.controller);
        if old == Some(player) && new != Some(player) {
            let Some(place) = super::place(before, id) else {
                continue;
            };
            let draft = transition_draft(
                candidate,
                NotificationKind::ControlLost,
                NotificationEntity::Site(id),
                Some(NotificationSubjectSnapshot::Place(place.clone())),
                NotificationDetail::Place {
                    place,
                    before: Some("player".into()),
                    after: None,
                    cause: None,
                    forecast_round: None,
                    conditions: Vec::new(),
                },
            );
            append(candidate, data, draft)?;
        } else if new == Some(player) && old != Some(player) {
            let Some(place) = super::place(candidate, id) else {
                continue;
            };
            let draft = transition_draft(
                candidate,
                NotificationKind::ControlGained,
                NotificationEntity::Site(id),
                Some(NotificationSubjectSnapshot::Place(place.clone())),
                NotificationDetail::Place {
                    place,
                    before: None,
                    after: Some("player".into()),
                    cause: None,
                    forecast_round: None,
                    conditions: Vec::new(),
                },
            );
            append(candidate, data, draft)?;
        }
    }
    Ok(())
}

fn territory_states(
    before: &StrategicCampaign,
    candidate: &mut StrategicCampaign,
    data: &GameData,
) -> Result<(), String> {
    let player = candidate.player;
    let sites = before
        .world
        .sites
        .iter()
        .chain(candidate.world.sites.iter())
        .map(|site| site.id)
        .collect::<BTreeSet<_>>();
    for id in sites {
        let old_owner = before.world.site(id).and_then(|site| site.controller);
        let new_owner = candidate.world.site(id).and_then(|site| site.controller);
        if old_owner != Some(player) && new_owner != Some(player) {
            continue;
        }
        let old_contested = before.world.contested_sites.contains(&id);
        let new_contested = candidate.world.contested_sites.contains(&id);
        if old_contested != new_contested {
            let kind = if new_contested {
                NotificationKind::ContestEntered
            } else {
                NotificationKind::ContestCleared
            };
            let reference = if new_contested { &*candidate } else { before };
            let Some(place) = super::place(reference, id) else {
                continue;
            };
            let draft = transition_draft(
                candidate,
                kind,
                NotificationEntity::Site(id),
                Some(NotificationSubjectSnapshot::Place(place.clone())),
                NotificationDetail::Place {
                    place,
                    before: Some(if old_contested { "contested" } else { "clear" }.into()),
                    after: Some(if new_contested { "contested" } else { "clear" }.into()),
                    cause: None,
                    forecast_round: None,
                    conditions: Vec::new(),
                },
            );
            append(candidate, data, draft)?;
        }
        let old_occupation = before.world.occupation.get(&id).copied().unwrap_or(0);
        let new_occupation = candidate.world.occupation.get(&id).copied().unwrap_or(0);
        if (old_occupation == 0) != (new_occupation == 0) {
            let kind = if new_occupation > 0 {
                NotificationKind::OccupationStarted
            } else {
                NotificationKind::OccupationCleared
            };
            let reference = if new_occupation > 0 {
                &*candidate
            } else {
                before
            };
            let Some(place) = super::place(reference, id) else {
                continue;
            };
            let draft = transition_draft(
                candidate,
                kind,
                NotificationEntity::Site(id),
                Some(NotificationSubjectSnapshot::Place(place.clone())),
                NotificationDetail::Place {
                    place,
                    before: Some(format!("occupation_{}", old_occupation)),
                    after: Some(format!("occupation_{}", new_occupation)),
                    cause: None,
                    forecast_round: None,
                    conditions: Vec::new(),
                },
            );
            append(candidate, data, draft)?;
        }
    }
    Ok(())
}

fn site_reclamation(
    before: &StrategicCampaign,
    candidate: &mut StrategicCampaign,
    data: &GameData,
) -> Result<(), String> {
    let player = candidate.player;
    let sites = before
        .world
        .sites
        .iter()
        .chain(candidate.world.sites.iter())
        .map(|site| site.id)
        .collect::<BTreeSet<_>>();
    for id in sites {
        if !before.site_is_ruined(id)
            || candidate.site_is_ruined(id)
            || candidate.world.site(id).and_then(|site| site.controller) != Some(player)
        {
            continue;
        }
        let Some(place) = super::place(candidate, id) else {
            continue;
        };
        let draft = transition_draft(
            candidate,
            NotificationKind::SiteReclaimed,
            NotificationEntity::Site(id),
            Some(NotificationSubjectSnapshot::Place(place.clone())),
            NotificationDetail::Place {
                place,
                before: Some("ruined".into()),
                after: Some("outpost".into()),
                cause: Some("reclaimed".into()),
                forecast_round: None,
                conditions: Vec::new(),
            },
        );
        append(candidate, data, draft)?;
    }
    Ok(())
}

fn local_threats(
    before: &StrategicCampaign,
    candidate: &mut StrategicCampaign,
    data: &GameData,
) -> Result<(), String> {
    let player = candidate.player;
    let visible_before = crate::engine::visible_threats(before, player)
        .into_iter()
        .map(|threat| threat.id)
        .collect::<BTreeSet<_>>();
    let visible_after = crate::engine::visible_threats(candidate, player)
        .into_iter()
        .map(|threat| threat.id)
        .collect::<BTreeSet<_>>();
    let ids = before
        .threats
        .keys()
        .chain(candidate.threats.keys())
        .copied()
        .collect::<BTreeSet<_>>();
    for id in ids {
        let started = visible_after.contains(&id)
            && !visible_before.contains(&id)
            && candidate
                .threats
                .get(&id)
                .is_some_and(|threat| threat.status == ThreatStatus::Active);
        let cleared = visible_before.contains(&id)
            && before
                .threats
                .get(&id)
                .is_some_and(|threat| threat.status == ThreatStatus::Active)
            && candidate
                .threats
                .get(&id)
                .is_none_or(|threat| threat.status != ThreatStatus::Active)
            && crate::engine::threats::threat_site_observed(
                candidate,
                player,
                candidate
                    .threats
                    .get(&id)
                    .or_else(|| before.threats.get(&id))
                    .expect("visible threat exists before or after")
                    .site,
            );
        let (kind, site, threat_name) = if started {
            let threat = candidate.threats.get(&id).expect("visible threat exists");
            (
                NotificationKind::LocalThreatStarted,
                threat.site,
                threat.name.clone(),
            )
        } else if cleared {
            let threat = before.threats.get(&id).expect("known threat exists");
            (
                NotificationKind::LocalThreatCleared,
                threat.site,
                threat.name.clone(),
            )
        } else {
            continue;
        };
        let place_campaign = if started { &*candidate } else { before };
        let Some(place) = super::place(place_campaign, site) else {
            continue;
        };
        let source = NotificationSourceId::Transition {
            accepted_sequence: candidate.accepted_sequence.max(1),
            kind,
            subject: NotificationEntity::Site(site),
            ordinal: id.0,
        };
        let round = candidate.completed_rounds;
        let sequence = candidate.accepted_sequence;
        append(
            candidate,
            data,
            NotificationDraft {
                source,
                kind,
                round,
                sequence,
                subject: Some(NotificationSubjectSnapshot::Place(place)),
                detail: NotificationDetail::Facts {
                    values: BTreeMap::from([("threat".into(), threat_name)]),
                },
                active: false,
            },
        )?;
    }
    Ok(())
}

fn facilities(
    before: &StrategicCampaign,
    candidate: &mut StrategicCampaign,
    data: &GameData,
) -> Result<(), String> {
    let ids = before
        .world
        .sites
        .iter()
        .chain(candidate.world.sites.iter())
        .filter(|site| site.controller == Some(candidate.player))
        .map(|site| site.id)
        .collect::<BTreeSet<_>>();
    for id in ids {
        for (index, facility) in [
            Facility::TrainingGround,
            Facility::Stable,
            Facility::Infirmary,
            Facility::Workshop,
            Facility::Temple,
        ]
        .into_iter()
        .enumerate()
        {
            let was_functional = before
                .world
                .site(id)
                .is_some_and(|site| facility_functional(before, data, site.id, facility));
            let is_functional = candidate
                .world
                .site(id)
                .is_some_and(|site| facility_functional(candidate, data, site.id, facility));
            if was_functional == is_functional {
                continue;
            }
            let kind = NotificationKind::FacilityStatusChanged;
            let site = if is_functional { &*candidate } else { before };
            let Some(place) = super::place(site, id) else {
                continue;
            };
            let mut values = BTreeMap::new();
            values.insert(
                "facility".into(),
                format!("facility_{facility:?}").to_lowercase(),
            );
            values.insert(
                "before".into(),
                if was_functional {
                    "functional"
                } else {
                    "unavailable"
                }
                .into(),
            );
            values.insert(
                "after".into(),
                if is_functional {
                    "functional"
                } else {
                    "unavailable"
                }
                .into(),
            );
            let draft = NotificationDraft {
                source: NotificationSourceId::Transition {
                    accepted_sequence: candidate.accepted_sequence.max(1),
                    kind,
                    subject: NotificationEntity::Site(id),
                    ordinal: index as u32,
                },
                kind,
                round: candidate.completed_rounds,
                sequence: candidate.accepted_sequence,
                subject: Some(NotificationSubjectSnapshot::Place(place.clone())),
                detail: NotificationDetail::Facts { values },
                active: false,
            };
            append(candidate, data, draft)?;
        }
        let was_fort = before
            .world
            .site(id)
            .is_some_and(|site| fort_functional(before, data, site.id));
        let is_fort = candidate
            .world
            .site(id)
            .is_some_and(|site| fort_functional(candidate, data, site.id));
        if was_fort != is_fort {
            let kind = NotificationKind::FortStatusChanged;
            let site = if is_fort { &*candidate } else { before };
            let Some(place) = super::place(site, id) else {
                continue;
            };
            let mut values = BTreeMap::new();
            values.insert(
                "before".into(),
                if was_fort {
                    "functional"
                } else {
                    "unavailable"
                }
                .into(),
            );
            values.insert(
                "after".into(),
                if is_fort { "functional" } else { "unavailable" }.into(),
            );
            let draft = transition_draft(
                candidate,
                kind,
                NotificationEntity::Site(id),
                Some(NotificationSubjectSnapshot::Place(place.clone())),
                NotificationDetail::Facts { values },
            );
            append(candidate, data, draft)?;
        }
    }
    Ok(())
}

fn facility_functional(
    campaign: &StrategicCampaign,
    data: &GameData,
    site: crate::data::world::SiteId,
    facility: Facility,
) -> bool {
    campaign.world.site(site).is_some_and(|site| {
        site.controller == Some(campaign.player)
            && site.facilities.contains(&facility)
            && !campaign.site_is_ruined(site.id)
            && campaign.world.structural_damage(site.id) < data.economy.facility_failure_damage
    })
}

fn fort_functional(
    campaign: &StrategicCampaign,
    data: &GameData,
    id: crate::data::world::SiteId,
) -> bool {
    campaign.world.site(id).is_some_and(|site| {
        site.controller == Some(campaign.player)
            && site.military == MilitaryLayer::Fort
            && campaign.world.fort_damage.get(&id).copied().unwrap_or(0)
                < data.development.conditions.functional_fort_damage
            && !campaign.world.contested_sites.contains(&id)
    })
}

fn supply(
    before: &StrategicCampaign,
    candidate: &mut StrategicCampaign,
    data: &GameData,
) -> Result<(), String> {
    let ids = before
        .armies
        .values()
        .chain(candidate.armies.values())
        .filter(|army| army.faction == candidate.player)
        .map(|army| army.id)
        .collect::<BTreeSet<_>>();
    for id in ids {
        let (Some(old_army), Some(new_army)) = (before.armies.get(&id), candidate.armies.get(&id))
        else {
            continue;
        };
        if old_army.faction != candidate.player || new_army.faction != candidate.player {
            continue;
        }
        let was = before.army_is_supplied(id);
        let is = candidate.army_is_supplied(id);
        if was == is {
            continue;
        }
        let kind = if is {
            NotificationKind::SupplyRecovered
        } else {
            NotificationKind::SupplyLost
        };
        let Some(army) =
            super::army_snapshot(candidate, id).or_else(|| super::army_snapshot(before, id))
        else {
            continue;
        };
        let draft = transition_draft(
            candidate,
            kind,
            NotificationEntity::Army(id),
            Some(NotificationSubjectSnapshot::Army(army.clone())),
            NotificationDetail::Movement {
                armies: vec![army.clone()],
                destination: army.site,
                cause: Some(
                    if is {
                        "supply_recovered"
                    } else {
                        "supply_lost"
                    }
                    .into(),
                ),
            },
        );
        append(candidate, data, draft)?;
    }
    Ok(())
}

fn economy(
    before: &StrategicCampaign,
    candidate: &mut StrategicCampaign,
    data: &GameData,
) -> Result<(), String> {
    let old = before
        .factions
        .get(&candidate.player)
        .is_some_and(|faction| faction.deficit);
    let new = candidate
        .factions
        .get(&candidate.player)
        .is_some_and(|faction| faction.deficit);
    if old == new {
        return Ok(());
    }
    let kind = if new {
        NotificationKind::UnpaidUpkeepBegan
    } else {
        NotificationKind::UnpaidUpkeepCleared
    };
    let shortfall = candidate
        .factions
        .get(&candidate.player)
        .and_then(|faction| faction.last_economy.as_ref())
        .map_or(0, |statement| statement.shortfall);
    let values = BTreeMap::from([
        ("shortfall".into(), shortfall.to_string()),
        (
            "event".into(),
            if new {
                "upkeep_shortfall"
            } else {
                "upkeep_cleared"
            }
            .into(),
        ),
    ]);
    let player = candidate.player;
    let subject = Some(NotificationSubjectSnapshot::Faction {
        id: player,
        name: candidate.factions[&player].name.clone(),
    });
    let draft = transition_draft(
        candidate,
        kind,
        NotificationEntity::Faction(player),
        subject,
        NotificationDetail::Facts { values },
    );
    append(candidate, data, draft)?;
    Ok(())
}
