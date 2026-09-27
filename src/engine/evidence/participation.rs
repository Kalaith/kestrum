//! Meaning and supported tags are calculated from immutable encounter snapshots.

use super::*;
use crate::{
    data::world::{AnchorExpression, FounderClass, MarkerLocation},
    state::{
        battle::{BattleArmyReport, BattleFormationReport, BattleOutcome, BattlePersonReport},
        people::{PersonAssignment, PersonCombatOutcome, PersonStatus},
    },
};

#[derive(Clone)]
pub(super) struct Participation {
    pub site: SiteId,
    pub opponent: EncounterOpponent,
    pub troop: TroopKind,
    pub enemy_types: BTreeSet<TroopKind>,
    pub meaningful: bool,
    pub tags: BTreeSet<EvidenceKind>,
    pub xp: u32,
    pub outnumbered: bool,
}

#[derive(Clone, Copy)]
pub(super) struct BattleContext<'a> {
    pub report: &'a BattleReport,
    pub own: &'a BattleSideReport,
    pub enemy: Opponent<'a>,
    pub attacking: bool,
}

#[derive(Clone, Copy)]
pub(super) enum Opponent<'a> {
    Faction(&'a BattleSideReport),
    Threat(&'a crate::state::battle::ThreatSideReport),
}

impl Opponent<'_> {
    pub(super) fn identity(self) -> EncounterOpponent {
        match self {
            Self::Faction(side) => EncounterOpponent::Faction(side.faction),
            Self::Threat(threat) => EncounterOpponent::Threat { threat: threat.id },
        }
    }

    fn power(self, data: &GameData) -> Result<u128, RuleError> {
        match self {
            Self::Faction(side) => power(side, data),
            Self::Threat(threat) => Ok(u128::from(threat.start) * u128::from(threat.attack)),
        }
    }

    fn armies(&self) -> &[BattleArmyReport] {
        match self {
            Self::Faction(side) => &side.armies,
            Self::Threat(_) => &[],
        }
    }
}

pub(super) fn classify(
    campaign: &StrategicCampaign,
    data: &GameData,
    context: BattleContext<'_>,
    army: &BattleArmyReport,
    formation: &BattleFormationReport,
) -> Result<Participation, RuleError> {
    let BattleContext {
        report,
        own,
        enemy,
        attacking,
    } = context;
    let rules = &data.progression;
    let own_power = power(own, data)?;
    let enemy_power = enemy.power(data)?;
    let outnumbered = enemy_power * 1000 >= own_power * u128::from(rules.outnumbered_permille);
    let victory = matches!(
        (attacking, report.outcome),
        (true, BattleOutcome::AttackerVictory) | (false, BattleOutcome::DefenderVictory)
    );
    let defeated = matches!(
        (attacking, report.outcome),
        (true, BattleOutcome::DefenderVictory) | (false, BattleOutcome::AttackerVictory)
    );
    let anchor = is_anchor(campaign, report.site);
    let captured = anchor
        && attacking
        && victory
        && report.control_before != Some(own.faction)
        && report.control_after == Some(own.faction);
    let defended = anchor
        && defended_place(report, attacking)
        && formation.end > 0
        && report.control_before == Some(own.faction)
        && report.control_after == Some(own.faction)
        && army.final_site == Some(report.site);
    let meaningful = enemy_power * 1000
        >= own_power * u128::from(rules.meaningful_opposition_permille)
        || (u128::from(formation.combat_losses) + u128::from(formation.encirclement_losses)) * 1000
            >= u128::from(formation.start) * u128::from(rules.meaningful_loss_permille)
        || captured
        || defended;
    let tags = encounter_tags(
        context,
        army,
        formation,
        [victory, defeated, captured, defended, outnumbered],
    );
    let xp = rules.battle_xp
        + u32::from(victory) * rules.victory_xp
        + u32::from(outnumbered && formation.end > 0) * rules.outnumbered_xp;
    Ok(Participation {
        site: report.site,
        opponent: enemy.identity(),
        troop: formation.kind,
        enemy_types: enemy
            .armies()
            .iter()
            .flat_map(|army| &army.formations)
            .map(|formation| formation.kind)
            .collect(),
        meaningful,
        tags,
        xp,
        outnumbered,
    })
}

fn defended_place(report: &BattleReport, attacking: bool) -> bool {
    use crate::state::battle::BattleContext;
    if !attacking {
        return matches!(
            report.outcome,
            BattleOutcome::DefenderVictory | BattleOutcome::Stalemate
        );
    }
    matches!(
        report.context,
        BattleContext::Sortie { .. } | BattleContext::Relief { .. }
    ) && report.outcome == BattleOutcome::AttackerVictory
}

fn encounter_tags(
    context: BattleContext<'_>,
    army: &BattleArmyReport,
    formation: &BattleFormationReport,
    flags: [bool; 5],
) -> BTreeSet<EvidenceKind> {
    let BattleContext {
        report, own, enemy, ..
    } = context;
    let [victory, defeated, captured, defended, outnumbered] = flags;
    let mut tags = [EvidenceKind::Battle, EvidenceKind::MeaningfulEncounter]
        .into_iter()
        .collect::<BTreeSet<_>>();
    for (condition, tag) in [
        (victory, EvidenceKind::Victory),
        (defeated, EvidenceKind::Defeat),
        (
            outnumbered && formation.end > 0,
            EvidenceKind::SurvivedOutnumbered,
        ),
        (captured, EvidenceKind::CapturedAnchor),
        (defended, EvidenceKind::DefendedAnchor),
        (
            army.final_site.is_some_and(|site| site != report.site),
            EvidenceKind::Retreated,
        ),
        (
            formation.kind == TroopKind::Medics && formation.end > 0 && friendly_casualties(own),
            EvidenceKind::TreatedWounded,
        ),
    ] {
        if condition {
            tags.insert(tag);
        }
    }
    if victory
        && enemy
            .armies()
            .iter()
            .any(|army| army.final_site.is_some_and(|site| site != report.site))
    {
        tags.insert(EvidenceKind::RetreatingEnemyVictory);
    }
    for person in army
        .people
        .iter()
        .filter(|person| person.starting_formation == formation.id)
    {
        add_personal_events(report, army, person, &mut tags);
    }
    if let Some(tag) = siege_tag(report, context.attacking) {
        tags.insert(tag);
    }
    if let Opponent::Threat(threat) = enemy {
        tags.insert(match threat.kind {
            crate::data::threats::ThreatKind::Bandits => EvidenceKind::EncounteredBandits,
            crate::data::threats::ThreatKind::Wildlife => EvidenceKind::EncounteredWildlife,
        });
        if victory {
            tags.insert(EvidenceKind::ClearedThreat);
        }
    }
    tags
}

fn siege_tag(report: &BattleReport, attacking: bool) -> Option<EvidenceKind> {
    use crate::state::battle::BattleContext;
    match (&report.context, attacking) {
        (BattleContext::Assault { .. }, true) => Some(EvidenceKind::AssaultedFort),
        (BattleContext::Assault { .. }, false) => Some(EvidenceKind::DefendedFort),
        (BattleContext::Sortie { .. }, true) => Some(EvidenceKind::Sortie),
        (BattleContext::Escape { .. }, true) => Some(EvidenceKind::EscapeAttempt),
        (BattleContext::Relief { .. }, true) => Some(EvidenceKind::Relief),
        _ => None,
    }
}

pub(super) fn personal_tags(
    report: &BattleReport,
    army: &BattleArmyReport,
    person: &BattlePersonReport,
    part: &mut Participation,
) {
    part.tags.remove(&EvidenceKind::SurvivedOutnumbered);
    let alive = !matches!(person.status, PersonStatus::Dead { .. });
    if part.outnumbered && alive {
        part.tags.insert(EvidenceKind::SurvivedOutnumbered);
    }
    part.tags.remove(&EvidenceKind::TreatedWounded);
    let own = if report
        .attacker
        .armies
        .iter()
        .any(|entry| entry.id == army.id)
    {
        &report.attacker
    } else {
        report
            .defender
            .faction_side()
            .expect("real participating defender")
    };
    if alive
        && person.starting_status == Some(PersonStatus::Fit)
        && friendly_casualties(own)
        && (person.class == FounderClass::Medic || part.troop == TroopKind::Medics)
    {
        part.tags.insert(EvidenceKind::TreatedWounded);
    }
    part.tags.remove(&EvidenceKind::Retreated);
    let destination = match person.assignment {
        PersonAssignment::Site { site } => Some(site),
        PersonAssignment::Formation { formation } => own
            .armies
            .iter()
            .find(|army| {
                army.formations
                    .iter()
                    .any(|member| member.id == formation && member.end > 0)
            })
            .and_then(|army| army.final_site),
        PersonAssignment::Dead => None,
    };
    if alive && destination.is_some_and(|site| site != report.site) {
        part.tags.insert(EvidenceKind::Retreated);
    }
    add_personal_events(report, army, person, &mut part.tags);
}

fn add_personal_events(
    report: &BattleReport,
    army: &BattleArmyReport,
    person: &BattlePersonReport,
    tags: &mut BTreeSet<EvidenceKind>,
) {
    for event in report
        .person_events
        .iter()
        .filter(|event| event.person == person.id)
    {
        match event.outcome {
            PersonCombatOutcome::Wounded {
                cause: crate::state::people::WoundCause::CommandCasualty,
                ..
            } => {
                tags.insert(EvidenceKind::CommanderWounded);
            }
            PersonCombatOutcome::AssumedCommand { .. } => {
                tags.insert(EvidenceKind::AssumedCommand);
            }
            _ => {}
        }
    }
    if tags.contains(&EvidenceKind::Victory)
        && army
            .commander
            .as_ref()
            .is_some_and(|commander| commander.id == person.id)
    {
        tags.insert(EvidenceKind::CommandedVictory);
    }
}

fn friendly_casualties(side: &BattleSideReport) -> bool {
    side.armies
        .iter()
        .flat_map(|army| &army.formations)
        .any(|formation| formation.combat_losses > 0)
}

fn power(side: &BattleSideReport, data: &GameData) -> Result<u128, RuleError> {
    side.armies
        .iter()
        .flat_map(|army| &army.formations)
        .try_fold(0_u128, |sum, formation| {
            sum.checked_add(
                u128::from(formation.start)
                    * u128::from(data.troops.formations[&formation.kind].attack),
            )
            .ok_or(RuleError::Overflow {
                field: "participation base power",
            })
        })
}

fn is_anchor(campaign: &StrategicCampaign, site: SiteId) -> bool {
    campaign.world.markers.iter().any(|marker|matches!(&marker.location,MarkerLocation::Region{anchors,..} if contains(anchors,site)))
}
fn contains(expression: &AnchorExpression, site: SiteId) -> bool {
    match expression {
        AnchorExpression::ControlledSite { site: id }
        | AnchorExpression::SuppliedEntrance { site: id } => *id == site,
        AnchorExpression::All { conditions } | AnchorExpression::Any { conditions } => {
            conditions.iter().any(|condition| contains(condition, site))
        }
    }
}
