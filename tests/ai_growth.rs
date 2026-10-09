//! Public-planner coverage for funded army growth and recruitment recovery.

use kestrum::{
    data::{
        economy::{Habitation, Resources},
        generation::ProductionSetup,
        rules::Emblem,
        world::FactionId,
        GameData,
    },
    engine::{ai, apply, Actor, Command},
    state::{military::ArmyId, StrategicCampaign},
};

const OAK: FactionId = FactionId(2);
const RICH: Resources = Resources {
    gold: 61_000,
    wood: 61_000,
    stone: 61_000,
};

fn fixture() -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    assert_eq!(campaign.active_faction(), OAK);
    (data, campaign)
}

/// Growth past two armies needs more developed land than the authored Rosemarch map holds.
fn production_fixture() -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new_production(
        &data,
        &ProductionSetup {
            kingdom_name: "Rose".into(),
            emblem: Emblem::Rose,
            factions: 4,
            seed: 260_926,
        },
    )
    .unwrap();
    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    assert_eq!(campaign.active_faction(), OAK);
    (data, campaign)
}

fn next_oak_turn(campaign: &mut StrategicCampaign, data: &GameData) {
    let round = campaign.completed_rounds;
    for _ in 0..5 {
        if campaign.completed_rounds > round && campaign.active_faction() == OAK {
            return;
        }
        let active = campaign.active_faction();
        let actor = if active == campaign.player {
            Actor::Player
        } else {
            Actor::Npc(active)
        };
        apply(campaign, data, actor, Command::EndTurn).unwrap();
    }
    panic!("one cycle returns to Oak");
}

fn add_developed_holdings(campaign: &mut StrategicCampaign, count: usize) {
    let ruined: std::collections::BTreeSet<_> = campaign
        .world
        .sites
        .iter()
        .filter(|site| campaign.site_is_ruined(site.id))
        .map(|site| site.id)
        .collect();
    let headquarters: std::collections::BTreeSet<_> = campaign
        .factions
        .values()
        .map(|faction| faction.headquarters)
        .collect();
    let mut added = 0;
    for site in &mut campaign.world.sites {
        if added >= count || ruined.contains(&site.id) || headquarters.contains(&site.id) {
            continue;
        }
        site.controller = Some(OAK);
        site.habitation = Habitation::MajorCity;
        added += 1;
    }
    assert_eq!(added, count, "fixture needs enough non-ruined sites");
    campaign.reconcile_region_control();
}

fn create_minimum_army(campaign: &mut StrategicCampaign, data: &GameData) -> ArmyId {
    let site = campaign.factions[&OAK].headquarters;
    let mut army = None;
    for kind in data
        .ai
        .recruitment_order
        .iter()
        .take(data.ai.minimum_formations)
    {
        let result = apply(
            campaign,
            data,
            Actor::Npc(OAK),
            Command::Recruit {
                site,
                army,
                kind: *kind,
            },
        )
        .unwrap();
        army = Some(result.recruited.unwrap().army);
    }
    army.expect("minimum formation count is positive")
}

fn enrich_campaign(mut campaign: StrategicCampaign) -> StrategicCampaign {
    campaign.factions.get_mut(&OAK).unwrap().resources = RICH;
    campaign
}

fn has_new_army_recruit(decision: &kestrum::engine::ai::AiDecision) -> bool {
    matches!(&decision.command, Command::Recruit { army: None, .. })
}

#[test]
fn funded_developed_holdings_grow_the_public_planner_past_two_armies() {
    let (data, mut campaign) = production_fixture();
    add_developed_holdings(&mut campaign, 24);
    campaign = enrich_campaign(campaign);
    create_minimum_army(&mut campaign, &data);
    next_oak_turn(&mut campaign, &data);
    campaign.factions.get_mut(&OAK).unwrap().resources = RICH;

    let decision = ai::propose(&campaign, &data, OAK).unwrap();
    assert!(
        has_new_army_recruit(&decision),
        "24 developed holdings and a current income record should support a third army: {decision:?}"
    );
}

#[test]
fn treasury_and_elapsed_rounds_do_not_replace_recent_income() {
    let (data, mut campaign) = production_fixture();
    add_developed_holdings(&mut campaign, 24);
    campaign = enrich_campaign(campaign);
    create_minimum_army(&mut campaign, &data);
    next_oak_turn(&mut campaign, &data);
    campaign.factions.get_mut(&OAK).unwrap().resources = RICH;
    let faction = campaign.factions.get_mut(&OAK).unwrap();
    // A recovery receipt is dated by the economy receipt it followed.
    faction.last_economy = None;
    faction.last_recovery = None;

    let without_income = ai::propose(&campaign, &data, OAK).unwrap();
    assert!(
        !has_new_army_recruit(&without_income),
        "a treasury alone cannot raise the target"
    );

    let upkeep: i64 = campaign
        .formations
        .values()
        .filter(|formation| formation.faction == OAK && formation.headcount > 0)
        .map(|formation| data.economy.formations[&formation.kind].upkeep_gold)
        .sum();
    let faction = campaign.factions.get_mut(&OAK).unwrap();
    faction.last_economy = Some(kestrum::state::military::EconomyStatement {
        settlement_inputs: None,
        completed_rounds: campaign.completed_rounds,
        income: Resources {
            gold: upkeep - 1,
            wood: 0,
            stone: 0,
        },
        upkeep_due: upkeep,
        upkeep_paid: upkeep,
        shortfall: 0,
        closing: RICH,
    });
    let negative_net_income = ai::propose(&campaign, &data, OAK).unwrap();
    assert!(
        !has_new_army_recruit(&negative_net_income),
        "income below current upkeep must not underflow into maximum growth support"
    );
}

#[test]
fn inaccessible_understrength_army_does_not_block_a_supplied_reliever() {
    let (data, mut campaign) = fixture();
    campaign.factions.get_mut(&OAK).unwrap().resources = RICH;
    let army_id = campaign
        .armies
        .values()
        .find(|army| army.faction == OAK)
        .unwrap()
        .id;
    let formations: Vec<_> = campaign.armies[&army_id].formation_ids().collect();
    for formation in formations.iter().skip(1) {
        apply(
            &mut campaign,
            &data,
            Actor::Npc(OAK),
            Command::Disband {
                formation: *formation,
            },
        )
        .unwrap();
    }
    let isolated = campaign
        .world
        .sites
        .iter()
        .find(|site| {
            site.id != campaign.factions[&OAK].headquarters
                && !campaign.site_is_ruined(site.id)
                && !campaign.world.contested_sites.contains(&site.id)
        })
        .unwrap()
        .id;
    let site = campaign
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == isolated)
        .unwrap();
    site.controller = Some(OAK);
    site.habitation = Habitation::MajorCity;
    campaign.world.contested_sites.insert(isolated);
    campaign.armies.get_mut(&army_id).unwrap().site = isolated;
    for formation in campaign
        .formations
        .values_mut()
        .filter(|formation| formation.faction == OAK)
    {
        formation.movement_spent = formation.movement_allowance(&data);
    }
    campaign.reconcile_region_control();

    let first = ai::propose(&campaign, &data, OAK).unwrap();
    assert!(
        has_new_army_recruit(&first),
        "the supplied headquarters should recruit despite an isolated one-formation army: {first:?}"
    );
    let created = apply(&mut campaign, &data, Actor::Npc(OAK), first.command)
        .unwrap()
        .recruited
        .unwrap()
        .army;
    let next = ai::propose(&campaign, &data, OAK).unwrap();
    assert!(
        matches!(next.command, Command::Recruit { army: Some(army), .. } if army == created),
        "the new local army is replenished before another is opened: {next:?}"
    );
}

#[test]
fn funded_supplied_core_can_raise_a_relief_army_for_isolated_forces() {
    let (data, mut campaign) = fixture();
    add_developed_holdings(&mut campaign, 7);
    campaign.factions.get_mut(&OAK).unwrap().resources = RICH;
    create_minimum_army(&mut campaign, &data);
    next_oak_turn(&mut campaign, &data);
    campaign.factions.get_mut(&OAK).unwrap().resources = RICH;

    let headquarters = campaign.factions[&OAK].headquarters;
    let isolated_sites: Vec<_> = campaign
        .world
        .sites
        .iter()
        .filter(|site| {
            site.id != headquarters
                && !campaign.site_is_ruined(site.id)
                && !campaign.world.contested_sites.contains(&site.id)
        })
        .take(2)
        .map(|site| site.id)
        .collect();
    assert_eq!(isolated_sites.len(), 2);
    let armies: Vec<_> = campaign
        .armies
        .values()
        .filter(|army| army.faction == OAK)
        .map(|army| army.id)
        .collect();
    assert_eq!(armies.len(), 2);
    for (army, site) in armies.iter().zip(isolated_sites) {
        let location = campaign
            .world
            .sites
            .iter_mut()
            .find(|entry| entry.id == site)
            .unwrap();
        location.controller = Some(OAK);
        location.habitation = Habitation::MajorCity;
        campaign.world.contested_sites.insert(site);
        campaign.armies.get_mut(army).unwrap().site = site;
    }
    for formation in campaign
        .formations
        .values_mut()
        .filter(|formation| formation.faction == OAK)
    {
        formation.movement_spent = formation.movement_allowance(&data);
    }
    campaign.reconcile_region_control();

    let decision = ai::propose(&campaign, &data, OAK).unwrap();
    assert!(
        has_new_army_recruit(&decision),
        "a funded supplied core should add one relief force when all armies are isolated: {decision:?}"
    );
}

#[test]
fn army_cap_deficit_and_full_complement_reserve_bound_growth() {
    let (mut data, mut campaign) = production_fixture();
    let mut invalid = data.ai.clone();
    invalid.maximum_armies = 1;
    assert!(
        invalid.validate().is_err(),
        "army cap cannot be below the base target"
    );
    add_developed_holdings(&mut campaign, 24);
    campaign.factions.get_mut(&OAK).unwrap().resources = RICH;
    create_minimum_army(&mut campaign, &data);
    next_oak_turn(&mut campaign, &data);
    campaign.factions.get_mut(&OAK).unwrap().resources = RICH;
    data.ai.maximum_armies = 3;
    let third = ai::propose(&campaign, &data, OAK).unwrap();
    assert!(
        has_new_army_recruit(&third),
        "the cap still permits growth up to three"
    );
    let third_id = apply(&mut campaign, &data, Actor::Npc(OAK), third.command)
        .unwrap()
        .recruited
        .unwrap()
        .army;
    for kind in data
        .ai
        .recruitment_order
        .iter()
        .take(data.ai.minimum_formations)
        .skip(1)
    {
        let site = campaign.armies[&third_id].site;
        apply(
            &mut campaign,
            &data,
            Actor::Npc(OAK),
            Command::Recruit {
                site,
                army: Some(third_id),
                kind: *kind,
            },
        )
        .unwrap();
    }
    assert!(
        !has_new_army_recruit(&ai::propose(&campaign, &data, OAK).unwrap()),
        "a high-growth holding count cannot exceed the authored army cap"
    );

    data.ai.maximum_armies = 8;
    let mut deficit = campaign.clone();
    let faction = deficit.factions.get_mut(&OAK).unwrap();
    faction.deficit = true;
    let statement = faction.last_economy.as_mut().unwrap();
    statement.upkeep_paid = statement.upkeep_due - 1;
    statement.shortfall = 1;
    assert!(
        !has_new_army_recruit(&ai::propose(&deficit, &data, OAK).unwrap()),
        "a deficit blocks ordinary army growth"
    );

    let mut thin_reserve = campaign;
    let current_upkeep: i64 = thin_reserve
        .formations
        .values()
        .filter(|formation| formation.faction == OAK && formation.headcount > 0)
        .map(|formation| data.economy.formations[&formation.kind].upkeep_gold)
        .sum();
    let complement_upkeep: i64 = data
        .ai
        .recruitment_order
        .iter()
        .take(data.ai.minimum_formations)
        .map(|kind| data.economy.formations[kind].upkeep_gold)
        .sum();
    let complement_gold: i64 = data
        .ai
        .recruitment_order
        .iter()
        .take(data.ai.minimum_formations)
        .map(|kind| data.economy.formations[kind].recruit_cost.gold)
        .sum();
    thin_reserve.factions.get_mut(&OAK).unwrap().resources = Resources {
        gold: current_upkeep * i64::from(data.ai.reserve_upkeep_rounds)
            + complement_upkeep * i64::from(data.ai.reserve_upkeep_rounds)
            + complement_gold
            - 1,
        wood: RICH.wood,
        stone: RICH.stone,
    };
    assert!(
        !has_new_army_recruit(&ai::propose(&thin_reserve, &data, OAK).unwrap()),
        "one gold below the full-complement reserve cannot open a partially funded army"
    );
}

#[test]
fn a_stranded_army_opens_a_relief_slot_above_the_holdings_target() {
    let (mut data, mut campaign) = production_fixture();
    add_developed_holdings(&mut campaign, 24);
    campaign = enrich_campaign(campaign);
    create_minimum_army(&mut campaign, &data);
    create_minimum_army(&mut campaign, &data);
    next_oak_turn(&mut campaign, &data);
    campaign.factions.get_mut(&OAK).unwrap().resources = RICH;
    data.ai.maximum_armies = 8;
    // Let the planner recruit until the realm reaches its ordinary target.
    for _ in 0..40 {
        let decision = ai::propose(&campaign, &data, OAK).unwrap();
        if !matches!(decision.command, Command::Recruit { .. }) {
            break;
        }
        apply(&mut campaign, &data, Actor::Npc(OAK), decision.command).unwrap();
        campaign.factions.get_mut(&OAK).unwrap().resources = RICH;
    }
    assert!(
        !has_new_army_recruit(&ai::propose(&campaign, &data, OAK).unwrap()),
        "a fully supplied realm at its target raises nothing more"
    );
    let armies: Vec<_> = campaign
        .armies
        .values()
        .filter(|army| army.faction == OAK)
        .map(|army| army.id)
        .collect();
    assert!(
        armies.len() > data.ai.target_armies,
        "the holdings target exceeds the base"
    );

    // Strand one army on a contested holding away from home.
    let headquarters = campaign.factions[&OAK].headquarters;
    let isolated = campaign
        .world
        .sites
        .iter()
        .find(|site| {
            site.controller == Some(OAK)
                && site.id != headquarters
                && !campaign.world.contested_sites.contains(&site.id)
        })
        .unwrap()
        .id;
    campaign.world.contested_sites.insert(isolated);
    campaign
        .armies
        .get_mut(armies.last().unwrap())
        .unwrap()
        .site = isolated;
    for formation in campaign
        .formations
        .values_mut()
        .filter(|formation| formation.faction == OAK)
    {
        formation.movement_spent = formation.movement_allowance(&data);
    }
    campaign.reconcile_region_control();
    let decision = ai::propose(&campaign, &data, OAK).unwrap();
    assert!(
        has_new_army_recruit(&decision),
        "the stranded army leaves room for a supplied relief force: {decision:?}"
    );
}
