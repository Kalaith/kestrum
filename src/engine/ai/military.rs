//! Attacks, retreats and strength estimates from observed forces.
use super::*;
use crate::{
    engine::{MoveOrder, SiegeRole},
    state::{
        military::{Army, ArmyId},
        people::PersonAssignment,
        siege::{SiegeAction, SiegeOrder},
    },
};
use std::collections::{BTreeMap, BTreeSet};

impl Planner<'_> {
    pub(super) fn threatened(&self, site: SiteId) -> bool {
        self.view.hostile_presence.contains(&site)
            || self
                .view
                .world
                .adjacent_sites(site)
                .into_iter()
                .any(|id| self.view.hostile_presence.contains(&id))
    }
    pub(super) fn threatened_headquarters(&self) -> bool {
        self.threatened(self.campaign.factions[&self.owner].headquarters)
    }

    fn at_war_anywhere(&self) -> bool {
        self.campaign.relations.iter().any(|relation| {
            relation.factions.contains(&self.owner)
                && relation.state == crate::data::world::DiplomaticState::War
        })
    }

    pub(super) fn defend(&self) -> Option<AiDecision> {
        let headquarters = self.campaign.factions[&self.owner].headquarters;
        let anchors = self.anchors();
        let mut targets: Vec<_> = self
            .view
            .world
            .sites
            .iter()
            .filter(|site| {
                site.controller == Some(self.owner)
                    && (site.id == headquarters || anchors.contains(&site.id))
                    && self.threatened(site.id)
            })
            .map(|site| site.id)
            .collect();
        targets.sort_by_key(|site| (*site != headquarters, *site));
        targets.into_iter().find_map(|site| {
            // A tolerable force blocking the HQ approach can be driven off.
            // Waiting at an unreachable defensive objective would waste the phase.
            std::iter::once(site)
                .chain(self.view.world.adjacent_sites(site))
                .filter(|enemy| self.view.hostile_presence.contains(enemy))
                .find_map(|enemy| self.attack_at(enemy))
                .map(|mut decision| {
                    decision.objective = Some(self.objective(site, AiObjectiveKind::Defend));
                    decision
                })
                .or_else(|| self.toward(site, AiObjectiveKind::Defend, true))
        })
    }

    pub(super) fn weak(&self, army: &Army) -> bool {
        let (heads, capacity) = army
            .formation_ids()
            .filter_map(|id| {
                self.view
                    .formations
                    .iter()
                    .find(|formation| formation.id == id)
            })
            .fold((0_u64, 0_u64), |(heads, capacity), f| {
                (
                    heads + u64::from(f.headcount),
                    capacity + u64::from(f.capacity),
                )
            });
        heads * 100 < capacity * u64::from(self.data.ai.retreat_capacity_percent)
    }

    pub(super) fn retreat(&self) -> Option<AiDecision> {
        for siege in &self.view.sieges {
            let view = crate::engine::siege_view(self.campaign, self.data, self.owner, siege.site)?;
            if !view.own_armies.iter().any(|id| {
                self.view
                    .armies
                    .iter()
                    .find(|army| army.id == *id)
                    .is_some_and(|army| self.weak(army))
            }) {
                continue;
            }
            let action = if view.role == SiegeRole::Defender {
                SiegeAction::Escape
            } else {
                SiegeAction::Withdraw
            };
            let Some(option) = view
                .actions
                .iter()
                .find(|option| option.action == action && option.blocked.is_none())
            else {
                continue;
            };
            let mut destinations = option.destinations.clone();
            destinations.sort_by_key(|site| (!self.view.supplied_sites.contains(site), *site));
            for destination in destinations {
                let command = Command::Siege(SiegeOrder {
                    site: view.site,
                    action,
                    armies: view.own_armies.clone(),
                    destination: Some(destination),
                });
                if let Some(decision) = self.choose(command, self.objective.clone()) {
                    return Some(decision);
                }
            }
        }
        let headquarters = self.campaign.factions[&self.owner].headquarters;
        let anchors = self.anchors();
        for army in self.view.armies.iter().filter(|army| {
            !self.view.supplied_armies.contains(&army.id)
                && self.moving(army, true)
                && !self.is_last_wartime_garrison(army, headquarters, &anchors)
                && !self
                    .view
                    .construction
                    .iter()
                    .any(|order| order.is_open() && order.builder == Some(army.id))
        }) {
            let mut routes: Vec<_> = self
                .view
                .supplied_sites
                .iter()
                .filter_map(|site| {
                    self.path(army.site, *site, false)
                        .map(|(cost, path)| (cost, *site, path))
                })
                .collect();
            routes.sort();
            for (_, _, path) in routes.into_iter().filter(|(_, _, path)| path.len() > 1) {
                if let Some(decision) = self.choose(
                    Command::Move(MoveOrder {
                        armies: vec![army.id],
                        path: path[..2].to_vec(),
                    }),
                    self.objective.clone(),
                ) {
                    return Some(decision);
                }
            }
        }
        None
    }

    fn is_last_wartime_garrison(
        &self,
        army: &Army,
        headquarters: SiteId,
        anchors: &BTreeSet<SiteId>,
    ) -> bool {
        let is_important = army.site == headquarters
            || anchors.contains(&army.site)
            || self.border_post(army.site);
        self.at_war_anywhere()
            && is_important
            && !self
                .view
                .armies
                .iter()
                .any(|other| other.id != army.id && other.site == army.site)
    }

    pub(super) fn clear_threat(&self) -> Option<AiDecision> {
        let mut targets = self
            .view
            .threats
            .iter()
            .filter_map(|threat| {
                let approaches = self.threat_approaches(threat.site);
                approaches.first().map(|(cost, _, _)| {
                    let apprentice_can_approach = approaches
                        .iter()
                        .any(|(_, army, _)| self.army_has_pending_hero_service(*army));
                    (
                        !apprentice_can_approach,
                        *cost,
                        self.strategic_priority(threat.site),
                        threat.site,
                    )
                })
            })
            .collect::<Vec<_>>();
        targets.sort();
        targets
            .into_iter()
            .find_map(|(_, _, _, site)| self.clear_threat_at(site))
    }
    pub(super) fn clear_threat_at(&self, site: SiteId) -> Option<AiDecision> {
        let threat = self
            .view
            .threats
            .iter()
            .find(|threat| threat.site == site)?;
        let mut groups: BTreeMap<SiteId, Vec<ArmyId>> = BTreeMap::new();
        for army in self.view.armies.iter().filter(|army| {
            self.moving(army, false) && self.view.world.connected_route(army.site, site).is_some()
        }) {
            groups.entry(army.site).or_default().push(army.id);
        }
        self.order_contact_groups(groups)
            .into_iter()
            .find_map(|(_, armies)| {
                self.choose(
                    Command::ClearThreat {
                        armies,
                        threat: threat.id,
                    },
                    Some(self.objective(site, AiObjectiveKind::Threat)),
                )
            })
            .or_else(|| {
                self.threat_approaches(site)
                    .into_iter()
                    .find_map(|(_, army, path)| {
                        (path.len() > 1)
                            .then(|| {
                                self.choose(
                                    Command::Move(MoveOrder {
                                        armies: vec![army],
                                        path: path[..2].to_vec(),
                                    }),
                                    Some(self.objective(site, AiObjectiveKind::Threat)),
                                )
                            })
                            .flatten()
                    })
            })
    }

    pub(super) fn attack(&self) -> Option<AiDecision> {
        if !self.at_war_anywhere() {
            return None;
        }
        for siege in &self.view.sieges {
            if ![siege.defender, siege.besieger]
                .into_iter()
                .flatten()
                .any(|faction| self.at_war(faction))
            {
                continue;
            }
            let view = crate::engine::siege_view(self.campaign, self.data, self.owner, siege.site)?;
            if self.advantage(&view.own_armies, view.site) {
                let action = if view.role == SiegeRole::Defender {
                    SiegeAction::Sortie
                } else {
                    SiegeAction::Assault
                };
                if let Some(decision) = self.choose(
                    Command::Siege(SiegeOrder {
                        site: view.site,
                        action,
                        armies: view.own_armies,
                        destination: None,
                    }),
                    Some(self.objective(view.site, AiObjectiveKind::Attack)),
                ) {
                    return Some(decision);
                }
            }
        }
        let observed_empty = self
            .view
            .world
            .sites
            .iter()
            .filter(|site| {
                site.controller.is_some_and(|owner| self.at_war(owner))
                    && self.view.armies.iter().any(|army| {
                        self.view
                            .world
                            .connected_route(army.site, site.id)
                            .is_some()
                    })
                    && !self
                        .view
                        .threats
                        .iter()
                        .any(|threat| threat.site == site.id)
            })
            .map(|site| site.id);
        let mut targets = self.targets(
            self.view
                .hostile_presence
                .iter()
                .copied()
                .chain(observed_empty),
        );
        let mut ranked = targets.drain(..).enumerate().collect::<Vec<_>>();
        ranked.sort_by_key(|(rank, site)| (!self.has_hero_battle_opportunity(*site), *rank));
        ranked
            .into_iter()
            .map(|(_, site)| site)
            .find_map(|site| self.attack_at(site))
    }

    pub(super) fn recover_lost_territory(&self) -> Option<AiDecision> {
        if !self.at_war_anywhere() {
            return None;
        }
        let mut latest: BTreeMap<SiteId, &crate::state::battle::BattleReport> = BTreeMap::new();
        for report in &self.view.battles {
            if latest.get(&report.site).is_none_or(|old| {
                (report.completed_rounds, report.sequence, report.id)
                    > (old.completed_rounds, old.sequence, old.id)
            }) {
                latest.insert(report.site, report);
            }
        }
        let targets: BTreeSet<_> = latest
            .into_values()
            .filter_map(|report| {
                let controller = report.control_after?;
                (report.control_before == Some(self.owner)
                    && self.at_war(controller)
                    && self
                        .view
                        .world
                        .site(report.site)
                        .is_some_and(|site| site.controller == Some(controller))
                    && self
                        .campaign
                        .completed_rounds
                        .saturating_sub(report.completed_rounds)
                        <= self.data.ai.objective_rounds)
                    .then_some(report.site)
            })
            .collect();
        self.targets(targets.into_iter())
            .into_iter()
            .find_map(|site| {
                self.attack_at(site)
                    .or_else(|| self.toward(site, AiObjectiveKind::Attack, false))
            })
    }

    pub(super) fn pursue_attack(&self) -> Option<AiDecision> {
        let objective = self.objective.as_ref()?;
        if objective.kind != AiObjectiveKind::Attack {
            return None;
        }
        self.attack_at(objective.site)
            .or_else(|| self.toward(objective.site, AiObjectiveKind::Attack, false))
    }

    pub(super) fn attack_at(&self, site: SiteId) -> Option<AiDecision> {
        if !self
            .view
            .world
            .site(site)?
            .controller
            .is_some_and(|owner| self.at_war(owner))
        {
            return None;
        }
        let mut groups: BTreeMap<SiteId, Vec<ArmyId>> = BTreeMap::new();
        for army in self.view.armies.iter().filter(|army| {
            self.moving(army, false) && self.view.world.connected_route(army.site, site).is_some()
        }) {
            groups.entry(army.site).or_default().push(army.id);
        }
        self.order_contact_groups(groups)
            .into_iter()
            .find_map(|(origin, armies)| {
                (!self.view.hostile_presence.contains(&site) || self.advantage(&armies, site))
                    .then(|| {
                        self.choose(
                            Command::Move(MoveOrder {
                                armies,
                                path: vec![origin, site],
                            }),
                            Some(self.objective(site, AiObjectiveKind::Attack)),
                        )
                    })
                    .flatten()
            })
    }

    pub(super) fn army_has_pending_hero_service(&self, army_id: ArmyId) -> bool {
        let Some(army) = self.view.armies.iter().find(|army| army.id == army_id) else {
            return false;
        };
        self.view.people.iter().any(|person| {
            person.faction == self.owner
                && !person.career.retired
                && person.is_fit_for_field(
                    self.campaign.completed_rounds,
                    self.data.rules.leadership.field_min_age_years,
                )
                && person.career.emergence.is_some()
                && person.career.recognition.is_none()
                && person.career.hero_service_progress
                    < self.data.progression.recognition.personal_engagements
                && matches!(person.assignment, PersonAssignment::Formation { formation }
                    if army.formation_ids().any(|member| member == formation))
        })
    }

    fn has_hero_battle_opportunity(&self, site: SiteId) -> bool {
        self.view.hostile_presence.contains(&site)
            && self.view.armies.iter().any(|army| {
                self.moving(army, false)
                    && self.army_has_pending_hero_service(army.id)
                    && self.view.world.connected_route(army.site, site).is_some()
            })
    }

    fn order_contact_groups(
        &self,
        groups: BTreeMap<SiteId, Vec<ArmyId>>,
    ) -> Vec<(SiteId, Vec<ArmyId>)> {
        let mut ordered = groups.into_iter().collect::<Vec<_>>();
        ordered.sort_by_key(|(origin, armies)| {
            (
                !armies
                    .iter()
                    .any(|army| self.army_has_pending_hero_service(*army)),
                *origin,
            )
        });
        ordered
    }

    pub(super) fn advantage(&self, armies: &[ArmyId], site: SiteId) -> bool {
        let own: u128 = armies
            .iter()
            .filter_map(|id| self.view.armies.iter().find(|army| army.id == *id))
            .map(|army| {
                let leadership = u128::from(
                    self.campaign
                        .army_leadership_permille(army.id, self.data)
                        .unwrap_or(0),
                );
                army.formation_ids()
                    .filter_map(|id| {
                        self.view
                            .formations
                            .iter()
                            .find(|formation| formation.id == id)
                    })
                    .map(|formation| {
                        u128::from(formation.headcount)
                            * u128::from(self.data.troops.formations[&formation.kind].attack)
                            * leadership
                            * u128::from(formation.service.tier.permille(&self.data.progression))
                    })
                    .sum::<u128>()
            })
            .sum();
        let mut observations = BTreeMap::new();
        for report in &self.view.battles {
            for side in report
                .faction_sides()
                .filter(|side| self.at_war(side.faction))
            {
                for army in &side.armies {
                    let key = (report.completed_rounds, report.sequence, report.id);
                    if observations.get(&army.id).is_none_or(|(old, _)| key > *old) {
                        observations.insert(army.id, (key, army));
                    }
                }
            }
        }
        let enemy: u128 = observations
            .values()
            .filter(|(_, army)| army.final_site == Some(site))
            .map(|(_, army)| {
                army.formations
                    .iter()
                    .map(|f| {
                        u128::from(f.end)
                            * u128::from(self.data.troops.formations[&f.kind].attack)
                            * u128::from(army.leadership_permille)
                            * u128::from(f.veterancy_permille)
                    })
                    .sum::<u128>()
            })
            .sum();
        // Presence supplies no headcount. Risk a first encounter only when our
        // known force clears an authored estimate (two full warrior formations)
        // by the same safety margin used for reported enemies. Never inspect the
        // live enemy roster; subsequent reports replace this prior.
        let estimate = if enemy == 0 {
            u128::from(self.data.ai.unknown_enemy_power) * 1_000 * 1_000
        } else {
            enemy
        };
        own * 100 > estimate * u128::from(self.data.ai.attack_advantage_percent)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{ObservedRoutes, Planner};
    use crate::{
        data::{
            economy::Resources,
            world::{DiplomaticState, FactionId},
            GameData,
        },
        engine::{apply, Actor, Command},
        state::{
            people::{EmergenceRecord, PersonAssignment, PersonCareer, PersonId},
            threat::ThreatStatus,
            StrategicCampaign,
        },
    };
    use std::collections::BTreeSet;

    const NPC: FactionId = FactionId(2);

    #[test]
    fn attack_prefers_an_existing_hostile_battle_for_an_apprentice_army() {
        let mut data = GameData::load().unwrap();
        data.ai.unknown_enemy_power = 1;
        let mut campaign = StrategicCampaign::new(&data).unwrap();
        apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
        let round = campaign.completed_rounds;
        for threat in campaign.threats.values_mut() {
            threat.headcount = 0;
            threat.status = ThreatStatus::Cleared {
                round,
                by: NPC,
                payout: Resources {
                    gold: 0,
                    wood: 0,
                    stone: 0,
                },
            };
        }

        let target = campaign
            .world
            .sites
            .iter()
            .map(|site| site.id)
            .find(|site| campaign.world.adjacent_sites(*site).len() >= 2)
            .unwrap();
        let origins = campaign
            .world
            .adjacent_sites(target)
            .into_iter()
            .take(2)
            .collect::<Vec<_>>();
        let npc_army = campaign
            .armies
            .values()
            .find(|army| army.faction == NPC)
            .unwrap()
            .id;
        let original = campaign.armies[&npc_army].clone();
        let empty_formations = original
            .formation_ids()
            .filter(|formation| {
                !campaign.people.values().any(|person| {
                    matches!(person.assignment, PersonAssignment::Formation { formation: assigned }
                        if assigned == *formation)
                })
            })
            .collect::<Vec<_>>();
        let apprentice_formation = empty_formations[0];
        let baseline_formation = empty_formations[1];
        let baseline_army = apply(
            &mut campaign,
            &data,
            Actor::Npc(NPC),
            Command::SplitArmy {
                formation: baseline_formation,
            },
        )
        .unwrap()
        .split_army
        .unwrap();
        campaign.armies.get_mut(&npc_army).unwrap().site = origins[1];
        campaign.armies.get_mut(&baseline_army).unwrap().site = origins[0];

        let commander = original.commander.expect("the NPC army has its founder");
        let mut apprentice = campaign.people[&commander].clone();
        let apprentice_id = campaign.next_ids.person;
        campaign.next_ids.person = PersonId(apprentice_id.0.checked_add(1).unwrap());
        apprentice.appearance = crate::engine::portraits::allocate_for_person(
            &mut campaign,
            &data.portraits,
            apprentice_id,
        )
        .unwrap();
        apprentice.id = apprentice_id;
        apprentice.name = format!("{} Apprentice", apprentice.name);
        apprentice.class = crate::data::world::PersonClass::Recruit;
        apprentice.assignment = PersonAssignment::Formation {
            formation: apprentice_formation,
        };
        apprentice.birth_round = -80;
        apprentice.service_start_round = round;
        apprentice.movement_spent = 0;
        apprentice.career = PersonCareer::default();
        apprentice.career.emergence = Some(EmergenceRecord {
            completed_rounds: round,
            source_formation: apprentice_formation,
            source_troop: campaign.formations[&apprentice_formation].kind,
            site: origins[1],
            distinguishing_deed: None,
        });
        apprentice.evidence = Default::default();
        campaign.people.insert(apprentice_id, apprentice);

        let enemy = campaign
            .armies
            .values()
            .find(|army| army.faction != NPC)
            .unwrap()
            .clone();
        let other_factions = campaign
            .factions
            .keys()
            .copied()
            .filter(|other| *other != NPC)
            .collect::<Vec<_>>();
        for other in other_factions {
            set_relation(
                &mut campaign,
                other,
                if other == enemy.faction {
                    DiplomaticState::War
                } else {
                    DiplomaticState::Peace
                },
            );
        }
        for army in campaign
            .armies
            .values_mut()
            .filter(|army| army.faction == enemy.faction)
        {
            army.site = target;
        }
        for formation in campaign
            .formations
            .values_mut()
            .filter(|formation| formation.faction == enemy.faction)
        {
            formation.headcount = 1;
        }
        for site in &origins {
            campaign
                .set_site_control(&data, *site, Some(NPC), false)
                .unwrap();
        }
        campaign
            .set_site_control(&data, target, Some(enemy.faction), false)
            .unwrap();
        campaign.validate(&data).unwrap();

        let battle_planner = planner(&campaign, &data);
        assert!(battle_planner.at_war(enemy.faction));
        assert!(battle_planner.army_has_pending_hero_service(npc_army));
        assert!(
            battle_planner.advantage(&[npc_army], target),
            "the Apprentice's full army clears the observed-force estimate"
        );
        assert!(
            battle_planner.has_hero_battle_opportunity(target),
            "the Apprentice's army can reach the observed enemy"
        );
        let preferred = battle_planner.attack_at(target).unwrap();
        assert!(
            matches!(&preferred.command, Command::Move(order)
                if order.armies == [npc_army] && order.path == [origins[1], target]),
            "an existing reachable battle should favor the Apprentice's army: {preferred:?}"
        );

        campaign
            .people
            .get_mut(&apprentice_id)
            .unwrap()
            .career
            .emergence = None;
        campaign.validate(&data).unwrap();
        let ordinary = planner(&campaign, &data).attack_at(target).unwrap();
        assert!(
            matches!(&ordinary.command, Command::Move(order)
                if order.armies == [baseline_army] && order.path == [origins[0], target]),
            "without an Apprentice, existing target ordering should be retained: {ordinary:?}"
        );
    }

    fn planner<'a>(campaign: &'a StrategicCampaign, data: &'a GameData) -> Planner<'a> {
        let view = super::super::project(campaign, NPC).unwrap();
        let enemies = campaign
            .relations
            .iter()
            .filter(|relation| {
                relation.factions.contains(&NPC) && relation.state == DiplomaticState::War
            })
            .flat_map(|relation| relation.factions)
            .filter(|faction| *faction != NPC)
            .collect::<BTreeSet<_>>();
        let routes = ObservedRoutes::new(&view, data, &enemies);
        Planner {
            campaign,
            data,
            owner: NPC,
            view,
            objective: None,
            routes,
            rejected_candidates: Default::default(),
            include_diagnostics: false,
            candidate_trace: Default::default(),
        }
    }

    fn set_relation(campaign: &mut StrategicCampaign, other: FactionId, state: DiplomaticState) {
        let factions = if NPC.0 < other.0 {
            [NPC, other]
        } else {
            [other, NPC]
        };
        let round = campaign.completed_rounds;
        campaign
            .relations
            .iter_mut()
            .find(|relation| relation.factions == factions)
            .unwrap()
            .state = state;
        let pair = campaign
            .diplomacy
            .pairs
            .iter_mut()
            .find(|pair| pair.factions == factions)
            .unwrap();
        pair.peace_since = (state == DiplomaticState::Peace).then_some(round);
        pair.truce_until = None;
    }
}
