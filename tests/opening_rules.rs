//! Opening-campaign rules: supply-dependent income, war restraint and the first
//! Apprentice.

use kestrum::{
    data::{
        economy::Habitation,
        generation::ProductionSetup,
        rules::Emblem,
        world::{DiplomaticState, FactionId},
        GameData,
    },
    engine::{ai, apply, diplomacy::peace_desired, Actor, Command},
    state::{
        battle::BattleOutcome, diplomacy::SiteLoss, people::PersonAssignment, threat::ThreatStatus,
        StrategicCampaign,
    },
};

const OAK: FactionId = FactionId(2);

fn production() -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let campaign = StrategicCampaign::new_production(
        &data,
        &ProductionSetup {
            kingdom_name: "Rose".into(),
            emblem: Emblem::Rose,
            factions: 4,
            seed: 260_926,
        },
    )
    .unwrap();
    (data, campaign)
}

fn finish_round(campaign: &mut StrategicCampaign, data: &GameData) {
    let round = campaign.completed_rounds;
    while campaign.completed_rounds == round {
        let actor = if campaign.active_faction() == campaign.player {
            Actor::Player
        } else {
            Actor::Npc(campaign.active_faction())
        };
        apply(campaign, data, actor, Command::EndTurn).unwrap();
    }
}

#[test]
fn holdings_cut_off_from_headquarters_pay_reduced_income() {
    let (data, mut campaign) = production();
    let player = campaign.player;
    let home = campaign.factions[&player].headquarters;
    // A village beyond every route from home is held but cannot be supplied.
    let distant = campaign
        .world
        .sites
        .iter()
        .filter(|site| site.controller.is_none() && campaign.active_threat(site.id).is_none())
        .map(|site| site.id)
        .find(|site| !campaign.world.adjacent_sites(home).contains(site))
        .unwrap();
    let entry = campaign
        .world
        .sites
        .iter_mut()
        .find(|entry| entry.id == distant)
        .unwrap();
    entry.habitation = Habitation::Village;
    campaign.world.population.insert(
        distant,
        data.construction.population.minimum[&Habitation::Village],
    );
    campaign
        .set_site_control(&data, distant, Some(player), false)
        .unwrap();
    assert!(!campaign.supplied_sites(player).contains(&distant));
    finish_round(&mut campaign, &data);
    let statement = campaign.factions[&player].last_economy.clone().unwrap();
    let entry = statement
        .settlement_inputs
        .unwrap()
        .sites
        .into_iter()
        .find(|entry| entry.site == distant)
        .unwrap();
    let base = data.economy.settlement_income[&Habitation::Village];
    assert_eq!(entry.supply_percent, data.economy.unsupplied_income_percent);
    assert_eq!(
        entry.income.gold,
        base.gold * i64::from(data.economy.unsupplied_income_percent) / 100
    );
    campaign.validate(&data).unwrap();
}

fn at_war(campaign: &mut StrategicCampaign, data: &GameData) {
    apply(
        campaign,
        data,
        Actor::Player,
        Command::DeclareWar { faction: OAK },
    )
    .unwrap();
}

#[test]
fn a_war_that_takes_no_ground_becomes_acceptable_to_end() {
    let (data, mut campaign) = production();
    at_war(&mut campaign, &data);
    for _ in 1..data.diplomacy.stalemate_rounds {
        finish_round(&mut campaign, &data);
    }
    assert!(
        !peace_desired(&campaign, &data, OAK, campaign.player),
        "an equal kingdom keeps fighting a young war"
    );
    finish_round(&mut campaign, &data);
    assert!(peace_desired(&campaign, &data, OAK, campaign.player));

    // A capture still inside the retained loss window keeps the war paying.
    let mut gaining = campaign.clone();
    let home = gaining.factions[&gaining.player].headquarters;
    gaining.diplomacy.losses.push(SiteLoss {
        faction: gaining.player,
        victor: OAK,
        site: home,
        completed_rounds: gaining.completed_rounds,
    });
    assert!(!peace_desired(&gaining, &data, OAK, gaining.player));

    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::OfferPeace { faction: OAK },
    )
    .unwrap();
    assert!(campaign
        .relations
        .iter()
        .any(|relation| relation.factions == [campaign.player, OAK]
            && relation.state == DiplomaticState::Peace));
}

#[test]
fn rivals_neither_open_wars_on_stronger_neighbors_nor_stall_behind_them() {
    let (data, mut campaign) = production();
    let player = campaign.player;
    let home = campaign.factions[&OAK].headquarters;
    // Make Rose and Oak neighbors with no unclaimed land left near Oak.
    for site in campaign.world.sites.iter_mut() {
        if site.controller.is_none() {
            site.controller = Some(player);
        }
    }
    campaign.reconcile_region_control();
    for _ in 0..data.ai.war_peace_rounds {
        finish_round(&mut campaign, &data);
    }
    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    assert_eq!(campaign.active_faction(), OAK);
    campaign.factions.get_mut(&OAK).unwrap().resources.gold = 5_000;
    // Two full supplied Oak armies at home meet every ordinary war prerequisite.
    let first = campaign
        .armies
        .values()
        .find(|army| army.faction == OAK)
        .unwrap()
        .id;
    let mut second = None;
    for army in [Some(first), None] {
        let mut target = army;
        for _ in 0..6 {
            if target.is_some_and(|id| campaign.armies[&id].first_empty_slot().is_none()) {
                break;
            }
            let recruited = apply(
                &mut campaign,
                &data,
                Actor::Npc(OAK),
                Command::Recruit {
                    site: home,
                    army: target,
                    kind: kestrum::data::economy::TroopKind::Warriors,
                },
            )
            .unwrap()
            .recruited
            .unwrap();
            target = Some(recruited.army);
        }
        if army.is_none() {
            second = target;
        }
    }
    let second = second.unwrap();
    for army in [first, second] {
        assert!(campaign.armies[&army].first_empty_slot().is_none());
    }
    let commands = oak_phase(&mut campaign.clone(), &data);
    assert!(
        commands.contains(&Command::DeclareWar { faction: player }),
        "an equal or stronger rival may open the war: {commands:?}"
    );

    let mut outmatched = campaign;
    for formation in outmatched
        .formations
        .values_mut()
        .filter(|formation| formation.faction == OAK)
    {
        formation.headcount = formation.capacity / 10;
    }
    // Parity growth stays bounded by sustainable income, so fund it explicitly.
    let upkeep: i64 = outmatched
        .formations
        .values()
        .filter(|formation| formation.faction == OAK && formation.headcount > 0)
        .map(|formation| data.economy.formations[&formation.kind].upkeep_gold)
        .sum();
    let round = outmatched.completed_rounds;
    let faction = outmatched.factions.get_mut(&OAK).unwrap();
    faction.last_recovery = None;
    faction.last_economy = Some(kestrum::state::military::EconomyStatement {
        settlement_inputs: None,
        completed_rounds: round,
        income: kestrum::data::economy::Resources {
            gold: upkeep * 4,
            wood: 0,
            stone: 0,
        },
        upkeep_due: upkeep,
        upkeep_paid: upkeep,
        shortfall: 0,
        closing: faction.resources,
    });
    outmatched.validate(&data).unwrap();
    let commands = oak_phase(&mut outmatched, &data);
    let raised = commands
        .iter()
        .position(|command| matches!(command, Command::Recruit { army: None, .. }));
    let declared = commands
        .iter()
        .position(|command| *command == Command::DeclareWar { faction: player });
    assert!(
        raised.is_some_and(|raised| declared.is_none_or(|declared| raised < declared)),
        "an outmatched rival builds a new army before any war: {commands:?}"
    );
}

/// Apply Oak's own planner decisions until it ends its turn.
fn oak_phase(campaign: &mut StrategicCampaign, data: &GameData) -> Vec<Command> {
    let mut commands = Vec::new();
    while campaign.active_faction() == OAK && commands.len() < 80 {
        let decision = ai::propose(campaign, data, OAK).unwrap();
        commands.push(decision.command.clone());
        if decision.command == Command::EndTurn {
            break;
        }
        apply(campaign, data, Actor::Npc(OAK), decision.command).unwrap();
    }
    commands
}

#[test]
fn a_kingdom_without_its_seat_or_armies_can_refound_headquarters() {
    let (data, mut campaign) = production();
    let player = campaign.player;
    let home = campaign.factions[&player].headquarters;
    // A settled holding away from home, with no army and no Training Ground.
    let refuge = campaign
        .world
        .sites
        .iter()
        .filter(|site| site.controller.is_none() && campaign.active_threat(site.id).is_none())
        .map(|site| site.id)
        .find(|site| !campaign.world.adjacent_sites(home).contains(site))
        .unwrap();
    let entry = campaign
        .world
        .sites
        .iter_mut()
        .find(|entry| entry.id == refuge)
        .unwrap();
    entry.habitation = data.development.administration.headquarters_minimum;
    let minimum = data.construction.population.minimum[&entry.habitation];
    campaign.world.population.insert(refuge, minimum);
    campaign
        .set_site_control(&data, refuge, Some(player), false)
        .unwrap();
    let command = Command::RelocateHeadquarters { site: refuge };
    assert!(
        kestrum::engine::preview(&campaign, &data, Actor::Player, command.clone()).is_err(),
        "a kingdom holding its seat still needs an army and Training Ground"
    );

    campaign
        .set_site_control(&data, home, Some(OAK), false)
        .unwrap();
    let armies: Vec<_> = campaign
        .armies
        .values()
        .filter(|army| army.faction == player)
        .map(|army| army.id)
        .collect();
    for army in armies {
        let formations: Vec<_> = campaign.armies[&army].formation_ids().collect();
        for formation in formations {
            apply(
                &mut campaign,
                &data,
                Actor::Player,
                Command::Disband { formation },
            )
            .unwrap();
        }
    }
    assert!(!campaign.armies.values().any(|army| army.faction == player));
    apply(&mut campaign, &data, Actor::Player, command).unwrap();
    assert_eq!(campaign.factions[&player].headquarters, refuge);
    assert!(campaign.supplied_sites(player).contains(&refuge));
}

#[test]
fn raids_spare_small_kingdoms_and_seize_exposed_holdings_of_sprawling_ones() {
    let (data, mut campaign) = production();
    let player = campaign.player;
    let rules = data.threats.raids;
    let raids = |campaign: &StrategicCampaign| {
        campaign
            .threats
            .values()
            .filter(|threat| threat.raid.is_some())
            .count()
    };
    // Grant the player enough settled holdings for exactly two raids.
    let wanted = rules.settlements_per_raid as usize * 2;
    let grants: Vec<_> = campaign
        .world
        .sites
        .iter()
        .filter(|site| site.controller.is_none() && campaign.active_threat(site.id).is_none())
        .map(|site| site.id)
        .take(wanted)
        .collect();
    assert_eq!(grants.len(), wanted);
    for site in grants {
        let entry = campaign
            .world
            .sites
            .iter_mut()
            .find(|entry| entry.id == site)
            .unwrap();
        entry.habitation = Habitation::Village;
        campaign.world.population.insert(
            site,
            data.construction.population.minimum[&Habitation::Village],
        );
        campaign
            .set_site_control(&data, site, Some(player), false)
            .unwrap();
    }
    campaign.validate(&data).unwrap();
    while campaign.completed_rounds < rules.first_round {
        assert_eq!(raids(&campaign), 0, "no raid before the first raid round");
        finish_round(&mut campaign, &data);
    }
    let raised: Vec<_> = campaign
        .threats
        .values()
        .filter(|threat| threat.raid == Some(rules.first_round))
        .collect();
    let owners: Vec<_> = raised
        .iter()
        .map(|threat| campaign.world.site(threat.site).unwrap().controller)
        .collect();
    assert_eq!(
        owners
            .iter()
            .filter(|owner| **owner == Some(player))
            .count(),
        2,
        "one raid per {} settlements: {raised:?}",
        rules.settlements_per_raid
    );
    assert!(
        owners.iter().all(|owner| *owner == Some(player)),
        "rivals with a single settlement are spared: {owners:?}"
    );
    for threat in &raised {
        let home = campaign.factions[&player].headquarters;
        assert_ne!(threat.site, home);
        assert!(!campaign
            .armies
            .values()
            .any(|army| army.site == threat.site));
    }
    campaign.validate(&data).unwrap();
    let saved: StrategicCampaign =
        serde_json::from_str(&serde_json::to_string(&campaign).unwrap()).unwrap();
    assert_eq!(saved, campaign);
}

fn clear_home_bandits(data: &GameData, enabled: bool) -> StrategicCampaign {
    let (mut data, mut campaign) = (data.clone(), production().1);
    data.progression.emergence.first_threat_victory = enabled;
    let player = campaign.player;
    let home = campaign.factions[&player].headquarters;
    let army = campaign
        .armies
        .values()
        .find(|army| army.faction == player)
        .unwrap()
        .id;
    let threat = campaign
        .threats
        .values()
        .find(|threat| {
            threat.status == ThreatStatus::Active
                && campaign.world.adjacent_sites(home).contains(&threat.site)
        })
        .expect("the production opening places a threat beside home")
        .id;
    let outcome = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::ClearThreat {
            armies: vec![army],
            threat,
        },
    )
    .unwrap();
    let outcome = if outcome.battle_pending {
        apply(
            &mut campaign,
            &data,
            Actor::Player,
            Command::StartPendingBattle,
        )
        .unwrap()
    } else {
        outcome
    };
    assert_eq!(
        campaign.battles[&outcome.battle.unwrap()].outcome,
        BattleOutcome::AttackerVictory
    );
    finish_round(&mut campaign, &data);
    campaign
}

#[test]
fn the_first_victory_over_local_bandits_raises_one_apprentice() {
    let data = GameData::load().unwrap();
    let emerged = |campaign: &StrategicCampaign| {
        campaign
            .people
            .values()
            .filter(|person| person.faction == campaign.player && person.career.emergence.is_some())
            .count()
    };
    let without = clear_home_bandits(&data, false);
    assert_eq!(
        emerged(&without),
        0,
        "bandits are not meaningful opposition"
    );
    let with = clear_home_bandits(&data, true);
    assert_eq!(emerged(&with), 1, "one Apprentice, not one per vacant slot");
    let apprentice = with
        .people
        .values()
        .find(|person| person.faction == with.player && person.career.emergence.is_some())
        .unwrap();
    assert!(matches!(
        apprentice.assignment,
        PersonAssignment::Formation { .. }
    ));
    let xp = |campaign: &StrategicCampaign| {
        campaign
            .formations
            .values()
            .filter(|formation| formation.faction == campaign.player)
            .map(|formation| formation.service.xp)
            .sum::<u32>()
    };
    assert_eq!(
        xp(&with),
        xp(&without),
        "the fight still earns no service XP"
    );
}
