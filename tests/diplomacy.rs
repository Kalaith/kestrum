//! Five P08 contracts exercise political outcomes through authoritative commands.

use kestrum::engine::diplomacy::peace_desired;

use kestrum::{
    data::{
        economy::{Habitation, TroopKind},
        world::{DiplomaticState, FactionId, MilitaryLayer, SiteId},
        GameData,
    },
    engine::{apply, diplomacy_view, preview, project, Actor, Command, MoveOrder},
    state::{
        diplomacy::{DefeatResolution, EndingKind},
        military::ArmyId,
        people::PersonStatus,
        Campaign, FactionStatus, StrategicCampaign,
    },
};
#[path = "support/diplomacy.rs"]
mod support;
use support::*;

#[test]
fn war_peace_offers_and_four_round_truces_share_one_public_rule_set() {
    let (data, mut campaign) = fixture();
    reject(
        &mut campaign,
        &data,
        Actor::Npc(FactionId(2)),
        Command::DeclareWar {
            faction: FactionId(1),
        },
    );
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::DeclareWar {
            faction: FactionId(2),
        },
    )
    .unwrap();
    reject(
        &mut campaign,
        &data,
        Actor::Player,
        Command::DeclareWar {
            faction: FactionId(2),
        },
    );
    let public = diplomacy_view(&campaign, &data, FactionId(1));
    assert!(!peace_desired(&campaign, &data, FactionId(2), FactionId(1)));
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::OfferPeace {
            faction: FactionId(2),
        },
    )
    .unwrap();
    assert_eq!(state(&campaign, 1, 2), DiplomaticState::War);
    reject(
        &mut campaign,
        &data,
        Actor::Player,
        Command::OfferPeace {
            faction: FactionId(2),
        },
    );
    finish(&mut campaign, &data);
    weak(&mut campaign, 2);
    let mut hidden = campaign.clone();
    for formation in hidden
        .formations
        .values_mut()
        .filter(|formation| formation.faction == FactionId(2))
    {
        formation.headcount = formation.capacity;
    }
    assert_eq!(
        diplomacy_view(&campaign, &data, FactionId(1)),
        diplomacy_view(&hidden, &data, FactionId(1)),
        "private acceptance totals never enter diplomacy projection"
    );
    assert!(public.factions[0].offer_blocked.is_none());
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::OfferPeace {
            faction: FactionId(2),
        },
    )
    .unwrap();
    assert_eq!(state(&campaign, 1, 2), DiplomaticState::Peace);
    for _ in 0..4 {
        reject(
            &mut campaign,
            &data,
            Actor::Player,
            Command::DeclareWar {
                faction: FactionId(2),
            },
        );
        finish(&mut campaign, &data);
    }
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::DeclareWar {
            faction: FactionId(2),
        },
    )
    .unwrap();
    assert_eq!(reload(&campaign, &data), campaign);
    assert_incoming_offer();
}

#[test]
fn peace_withdraws_foreign_occupants_and_rejects_any_missing_exit_atomically() {
    let (data, mut campaign) = fixture();
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::DeclareWar {
            faction: FactionId(2),
        },
    )
    .unwrap();
    weak(&mut campaign, 2);
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(2);
    campaign
        .set_site_control(&data, SiteId(5), Some(FactionId(2)), false)
        .unwrap();
    let before = campaign.clone();
    let error = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::OfferPeace {
            faction: FactionId(2),
        },
    )
    .unwrap_err();
    assert!(error.to_string().contains("Forces must withdraw first"));
    assert_eq!(campaign, before);
    campaign
        .set_site_control(&data, SiteId(5), None, false)
        .unwrap();
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::OfferPeace {
            faction: FactionId(2),
        },
    )
    .unwrap();
    assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(5));
    assert_eq!(state(&campaign, 1, 2), DiplomaticState::Peace);
    let formation = campaign.armies[&ArmyId(1)].formation_ids().next().unwrap();
    assert_eq!(
        campaign.formations[&formation].movement_spent,
        data.economy.formations[&campaign.formations[&formation].kind].movement_allowance
    );
    assert_eq!(reload(&campaign, &data), campaign);
    assert_siege_peace();
}

#[test]
fn losing_a_capital_or_one_kind_of_power_does_not_equal_defeat() {
    let (data, mut campaign) = conquest_fixture();
    let army = campaign.armies[&ArmyId(2)].clone();
    campaign.armies.get_mut(&ArmyId(2)).unwrap().site = SiteId(7);
    travel(&mut campaign, &data, &[1, 5, 2]);
    assert!(campaign.is_independent(FactionId(2)));
    assert!(!campaign.defeat_eligible(FactionId(2)));
    assert_eq!(campaign.factions[&FactionId(2)].capital, SiteId(2));
    assert!(campaign
        .diplomacy
        .losses
        .iter()
        .any(|loss| loss.faction == FactionId(2)
            && loss.victor == FactionId(1)
            && loss.site == SiteId(2)));
    assert!(peace_desired(&campaign, &data, FactionId(2), FactionId(1)));
    assert_eq!(campaign.armies[&ArmyId(2)].slots, army.slots);
    let (_, pending) = pending_defeat();
    assert!(pending.defeat_eligible(FactionId(2)));
    assert!(!pending.is_independent(FactionId(2)));
}

#[test]
fn annex_and_submission_are_final_and_displaced_survivors_never_join_the_victor() {
    let (data, pending) = pending_defeat();
    for resolution in [DefeatResolution::Annex, DefeatResolution::Submission] {
        let mut campaign = reload(&pending, &data);
        let own = campaign
            .formations
            .iter()
            .filter(|(_, formation)| formation.faction == FactionId(1))
            .count();
        apply(
            &mut campaign,
            &data,
            Actor::Player,
            Command::ResolveDefeat {
                faction: FactionId(2),
                resolution,
            },
        )
        .unwrap();
        assert_eq!(
            campaign.factions[&FactionId(2)].status,
            if resolution == DefeatResolution::Annex {
                FactionStatus::Eliminated
            } else {
                FactionStatus::Vassal {
                    sovereign: FactionId(1),
                }
            }
        );
        assert!(campaign
            .people
            .values()
            .filter(|person| person.faction == FactionId(2) && person.is_alive())
            .all(|person| matches!(person.status, PersonStatus::Displaced { .. })));
        assert_eq!(
            campaign
                .formations
                .values()
                .filter(|formation| formation.faction == FactionId(1))
                .count(),
            own
        );
        let gold = campaign.factions[&FactionId(2)].resources.gold;
        finish(&mut campaign, &data);
        assert_eq!(campaign.factions[&FactionId(2)].resources.gold, gold);
        assert!(!campaign.round_order.contains(&FactionId(2)));
        reject(
            &mut campaign,
            &data,
            Actor::Player,
            Command::ResolveDefeat {
                faction: FactionId(2),
                resolution,
            },
        );
        assert_eq!(reload(&campaign, &data), campaign);
    }
    assert_inherited_vassal();
}

#[test]
fn real_conquest_reaches_victory_and_mutual_last_army_loss_is_player_defeat() {
    let (data, mut campaign) = conquest_fixture();
    for path in [
        &[1, 5, 2][..],
        &[2, 5, 6, 8, 9, 10, 11, 3][..],
        &[3, 11, 4][..],
    ] {
        travel(&mut campaign, &data, path);
        let defeated = campaign.diplomacy.pending_defeats.first().unwrap().faction;
        apply(
            &mut campaign,
            &data,
            Actor::Player,
            Command::ResolveDefeat {
                faction: defeated,
                resolution: DefeatResolution::Annex,
            },
        )
        .unwrap();
    }
    assert_eq!(
        campaign.diplomacy.ending.as_ref().unwrap().kind,
        EndingKind::Victory
    );
    for relation in campaign.relations.iter().filter(|relation| {
        relation.state == DiplomaticState::War
            && relation
                .factions
                .iter()
                .any(|id| !campaign.is_independent(*id))
    }) {
        assert!(campaign
            .diplomacy
            .pair(relation.factions[0], relation.factions[1])
            .is_some_and(|pair| pair.war_ended_round.is_some()));
    }
    assert!(!project(&campaign, campaign.player)
        .unwrap()
        .era_label
        .starts_with("War ·"));
    let before = campaign.clone();
    reject(&mut campaign, &data, Actor::Player, Command::EndTurn);
    assert_eq!(reload(&campaign, &data), before);
    assert_mutual_defeat();
}
