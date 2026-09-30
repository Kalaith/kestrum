//! Compact campaign fixture for doctrine and rival-preparation regressions.

use super::*;

pub(super) fn fixture(first: u32, second: u32) -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    campaign
        .armies
        .retain(|id, _| [ArmyId(1), ArmyId(3)].contains(id));
    campaign
        .formations
        .retain(|id, _| [FormationId(1), FormationId(7)].contains(id));
    campaign.people.clear();
    campaign.legacy_items.clear();
    for (army_id, formation_id, site, count) in [(1, 1, 8, first), (3, 7, 10, second)] {
        let army = campaign.armies.get_mut(&ArmyId(army_id)).unwrap();
        army.site = SiteId(site);
        army.commander = None;
        army.slots = [
            Some(FormationId(formation_id)),
            None,
            None,
            None,
            None,
            None,
        ];
        campaign
            .formations
            .get_mut(&FormationId(formation_id))
            .unwrap()
            .headcount = count;
    }
    campaign.validate(&data).unwrap();
    (data, campaign)
}

pub(super) fn order(armies: &[u32], path: &[u32]) -> Command {
    Command::Move(MoveOrder {
        armies: armies.iter().copied().map(ArmyId).collect(),
        path: path.iter().copied().map(SiteId).collect(),
    })
}

pub(super) fn kind(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    id: u32,
    kind: TroopKind,
    count: u32,
) {
    let formation = campaign.formations.get_mut(&FormationId(id)).unwrap();
    formation.kind = kind;
    formation.capacity = data.economy.formations[&kind].capacity;
    formation.headcount = count;
}
