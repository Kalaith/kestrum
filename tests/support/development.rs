//! Durable scenario fixtures and real seasonal command helpers.

use super::*;

pub(super) fn fixture() -> (GameData, StrategicCampaign) {
    let mut data = GameData::load().unwrap();
    data.threats.initial.clear();
    for site in &mut data.scenario.sites {
        site.tags.retain(|tag| *tag != SiteTag::Ruins);
        if [1, 5, 6, 8, 10, 11, 12].contains(&site.id.0) {
            site.controller = Some(FactionId(1));
        }
    }
    for relation in &mut data.scenario.relations {
        relation.state = DiplomaticState::Peace;
    }
    data.scenario
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(5))
        .unwrap()
        .habitation = Habitation::Village;
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    for faction in campaign.factions.values_mut() {
        faction.resources = Resources {
            gold: 100_000,
            wood: 100_000,
            stone: 100_000,
        };
    }
    (data, campaign)
}

pub(super) fn finish(campaign: &mut StrategicCampaign, data: &GameData) {
    if campaign.phase == CampaignPhase::PlayerTurn {
        apply(campaign, data, Actor::Player, Command::EndTurn).unwrap();
    }
    while matches!(campaign.phase, CampaignPhase::NpcTurn { .. }) {
        advance_npc(campaign, data).unwrap();
    }
}

pub(super) fn site_mut(
    campaign: &mut StrategicCampaign,
    id: u32,
) -> &mut kestrum::data::world::Site {
    campaign
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(id))
        .unwrap()
}

pub(super) fn reload(campaign: &StrategicCampaign, data: &GameData) -> StrategicCampaign {
    let raw = macroquad_toolkit::persistence::encode_slot(
        "strategic_v2",
        &Campaign::Strategic(Box::new(campaign.clone())),
        "2",
    )
    .unwrap();
    kestrum::state::persistence::load_legacy(&raw, data)
        .unwrap()
        .strategic()
        .unwrap()
        .clone()
}

pub(super) fn reject(campaign: &mut StrategicCampaign, data: &GameData, command: Command) {
    let before = campaign.clone();
    assert!(preview(campaign, data, Actor::Player, command.clone()).is_err());
    assert!(apply(campaign, data, Actor::Player, command).is_err());
    assert_eq!(*campaign, before);
}

pub(super) fn total(campaign: &StrategicCampaign) -> u64 {
    campaign
        .world
        .population
        .values()
        .map(|value| u64::from(*value))
        .sum()
}

pub(super) fn grow_capacity_cases() {
    for (geography, maximum) in [
        (Geography::Plains, Habitation::MajorCity),
        (Geography::River, Habitation::MajorCity),
        (Geography::Valley, Habitation::MajorCity),
        (Geography::Coast, Habitation::MajorCity),
        (Geography::Forest, Habitation::Town),
        (Geography::Hill, Habitation::Town),
        (Geography::Marsh, Habitation::Village),
        (Geography::Pass, Habitation::Village),
        (Geography::Island, Habitation::Village),
    ] {
        let (mut data, _) = fixture();
        let site = data
            .scenario
            .sites
            .iter_mut()
            .find(|site| site.id == SiteId(5))
            .unwrap();
        site.geography = geography;
        site.habitation = maximum;
        let mut campaign = StrategicCampaign::new(&data).unwrap();
        let cap = data.construction.population.minimum[&maximum] * 150 / 100;
        campaign.world.population.insert(SiteId(5), cap);
        campaign
            .world
            .development
            .get_mut(&SiteId(5))
            .unwrap()
            .pressure = 24;
        finish(&mut campaign, &data);
        let view = development_view(&campaign, &data, FactionId(1), SiteId(5)).unwrap();
        assert_eq!(
            (
                view.maximum_habitation,
                view.habitation,
                view.capacity,
                view.population
            ),
            (maximum, maximum, cap, cap)
        );
        assert_eq!(view.pressure, 24);
    }
}

pub(super) fn assert_income(
    campaign: &StrategicCampaign,
    data: &GameData,
    site: u32,
    expected: Resources,
) {
    let receipt = campaign.factions[&FactionId(1)]
        .last_economy
        .as_ref()
        .unwrap();
    let inputs = receipt.settlement_inputs.as_ref().unwrap();
    let entry = inputs
        .sites
        .iter()
        .find(|entry| entry.site == SiteId(site))
        .unwrap();
    assert_eq!(entry.income, expected);
    assert_eq!(reload(campaign, data), *campaign);
    let mut bad = campaign.clone();
    bad.factions
        .get_mut(&FactionId(1))
        .unwrap()
        .last_economy
        .as_mut()
        .unwrap()
        .settlement_inputs
        .as_mut()
        .unwrap()
        .sites[0]
        .damage_percent = 1;
    assert!(bad.validate(data).is_err());
}

pub(super) fn assert_builder_population_survives_snapshot() {
    let (data, mut campaign) = fixture();
    site_mut(&mut campaign, 5).habitation = Habitation::Camp;
    campaign.world.population.insert(SiteId(5), 20);
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(5);
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartConstruction {
            target: ConstructionTarget::Site(SiteId(5)),
            kind: ConstructionKind::Outpost,
            builder: ArmyId(1),
        },
    )
    .unwrap();
    for _ in 0..2 {
        finish(&mut campaign, &data);
    }
    let destination = campaign.world.population[&SiteId(5)];
    let donor = campaign.world.population[&SiteId(1)];
    finish(&mut campaign, &data);
    assert_eq!(
        campaign.world.population[&SiteId(5)],
        destination + 50,
        "new Outpost keeps settlers and gets no retroactive growth"
    );
    assert_eq!(
        campaign.world.population[&SiteId(1)],
        donor - 50 + 2,
        "donor grows after real construction deduction"
    );
    assert_eq!(
        campaign.world.site(SiteId(5)).unwrap().habitation,
        Habitation::Outpost
    );
}

pub(super) fn assert_shared_arrival_budget() {
    let (data, mut campaign) = fixture();
    for id in [1, 5, 6, 8, 10, 11, 12] {
        site_mut(&mut campaign, id).habitation = Habitation::Village;
        let capacity = development_view(&campaign, &data, FactionId(1), SiteId(id))
            .unwrap()
            .capacity;
        campaign.world.population.insert(SiteId(id), capacity);
    }
    campaign.world.population.insert(SiteId(1), 2600);
    for id in [5, 6] {
        campaign.world.site_damage.insert(SiteId(id), 80);
    }
    let before = total(&campaign);
    finish(&mut campaign, &data);
    assert_eq!(total(&campaign), before + 32);
    assert_eq!(campaign.world.population[&SiteId(1)], 2700);
    assert_eq!(
        campaign.world.population[&SiteId(5)],
        2626,
        "lower source ID fills the single74-person remaining budget"
    );
    assert_eq!(
        campaign.world.population[&SiteId(6)],
        2700,
        "arrivals never create new destination capacity"
    );
    assert_eq!(campaign.world.development[&SiteId(5)].displaced, 61);
    assert_eq!(campaign.world.development[&SiteId(6)].displaced, 135);
}

pub(super) fn assert_threat_blocks_migration() {
    let (mut data, _) = fixture();
    data.scenario
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(6))
        .unwrap()
        .controller = None;
    data.threats
        .initial
        .push(kestrum::data::threats::InitialThreat {
            site: SiteId(6),
            kind: kestrum::data::threats::ThreatKind::Wildlife,
        });
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    campaign
        .set_site_control(&data, SiteId(6), Some(FactionId(1)), false)
        .unwrap();
    campaign
        .world
        .development
        .get_mut(&SiteId(5))
        .unwrap()
        .displaced = 10;
    site_mut(&mut campaign, 8).habitation = Habitation::Village;
    reject(
        &mut campaign,
        &data,
        Command::Resettle {
            from: SiteId(5),
            to: SiteId(8),
        },
    );
    campaign
        .world
        .development
        .get_mut(&SiteId(6))
        .unwrap()
        .displaced = 1;
    campaign.world.population.insert(SiteId(6), 1);
    reject(
        &mut campaign,
        &data,
        Command::Resettle {
            from: SiteId(6),
            to: SiteId(5),
        },
    );
}

pub(super) fn assert_remote_forecast_privacy() {
    let (data, mut campaign) = fixture();
    campaign
        .relations
        .iter_mut()
        .find(|relation| relation.factions == [FactionId(1), FactionId(2)])
        .unwrap()
        .state = DiplomaticState::War;
    let original = development_view(&campaign, &data, FactionId(1), SiteId(10)).unwrap();
    assert_eq!(original.safe, None);
    assert_eq!(original.contribution, None);
    assert!(original.causes.is_empty());
    campaign.armies.get_mut(&ArmyId(2)).unwrap().site = SiteId(11);
    assert_eq!(
        development_view(&campaign, &data, FactionId(1), SiteId(10)).unwrap(),
        original
    );
}

pub(super) fn assert_npc_administration() {
    let (data, mut campaign) = fixture();
    campaign
        .set_site_control(&data, SiteId(5), Some(FactionId(2)), false)
        .unwrap();
    site_mut(&mut campaign, 5).habitation = Habitation::Town;
    site_mut(&mut campaign, 5)
        .facilities
        .push(Facility::TrainingGround);
    campaign.armies.get_mut(&ArmyId(2)).unwrap().site = SiteId(5);
    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    assert_eq!(campaign.active_faction(), FactionId(2));
    for command in [
        Command::RenameSite {
            site: SiteId(5),
            name: "Eastern Haven".into(),
        },
        Command::MoveCapital { site: SiteId(5) },
        Command::RelocateHeadquarters { site: SiteId(5) },
    ] {
        apply(&mut campaign, &data, Actor::Npc(FactionId(2)), command).unwrap();
    }
    assert_eq!(
        (
            campaign.factions[&FactionId(2)].capital,
            campaign.factions[&FactionId(2)].headquarters
        ),
        (SiteId(5), SiteId(5))
    );
    assert_eq!(reload(&campaign, &data), campaign);
}
