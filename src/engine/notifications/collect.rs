//! Receipt allocation, retention and shared safe snapshots for accepted events.

mod facts;
mod life;
mod transitions;

use crate::{
    data::{world::SiteId, GameData},
    engine::actions::ActionOutcome,
    state::{notifications::*, StrategicCampaign},
};

pub fn collect(
    before: &StrategicCampaign,
    candidate: &mut StrategicCampaign,
    data: &GameData,
    outcome: &ActionOutcome,
) -> Result<(), String> {
    if candidate.observer_mode || before.observer_mode {
        return Ok(());
    }
    if !candidate.notifications.baseline_complete {
        let mut baseline = before.clone();
        baseline_current_conditions(&mut baseline, data)?;
        candidate.notifications = baseline.notifications;
    }
    life::collect(before, candidate, data, outcome)?;
    facts::collect(before, candidate, data, outcome)?;
    transitions::collect(before, candidate, data)?;
    super::forecast::reconcile(candidate, data, false)?;
    trim(candidate, data);
    candidate.notifications.baseline_complete = true;
    Ok(())
}

/// Registers already-present conditions without claiming they were newly caused.
pub fn baseline_current_conditions(
    campaign: &mut StrategicCampaign,
    data: &GameData,
) -> Result<(), String> {
    if campaign.observer_mode {
        campaign.notifications.baseline_complete = true;
        return Ok(());
    }
    super::forecast::reconcile(campaign, data, true)?;
    trim(campaign, data);
    campaign.notifications.baseline_complete = true;
    Ok(())
}

pub(crate) fn collect_blocked_continuation(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    result: &crate::engine::MovementOutcome,
) -> Result<(), String> {
    if campaign.observer_mode {
        return Ok(());
    }
    let Some(stop) = &result.stop else {
        return Ok(());
    };
    let snapshots = result
        .armies
        .iter()
        .filter_map(|id| army_snapshot(campaign, *id))
        .collect::<Vec<_>>();
    let Some(army) = snapshots.first().cloned() else {
        return Ok(());
    };
    let destination = result
        .requested_destination
        .or(Some(stop.site))
        .or_else(|| result.path.last().copied())
        .and_then(|id| place(campaign, id));
    let kind = NotificationKind::MovementBlocked;
    let source = NotificationSourceId::Transition {
        accepted_sequence: campaign.accepted_sequence.max(1),
        kind,
        subject: NotificationEntity::Army(army.id),
        ordinal: 0,
    };
    let round = campaign.completed_rounds;
    let sequence = campaign.accepted_sequence;
    append(
        campaign,
        data,
        NotificationDraft {
            source,
            kind,
            round,
            sequence,
            subject: Some(NotificationSubjectSnapshot::Army(army)),
            detail: NotificationDetail::Movement {
                armies: snapshots,
                destination,
                cause: Some(format!("movement_{:?}", stop.reason).to_lowercase()),
            },
            active: false,
        },
    )?;
    trim(campaign, data);
    campaign
        .validate(data)
        .map_err(|error| format!("campaign after blocked movement: {error}"))?;
    Ok(())
}

pub(super) struct NotificationDraft {
    pub source: NotificationSourceId,
    pub kind: NotificationKind,
    pub round: u32,
    pub sequence: u64,
    pub subject: Option<NotificationSubjectSnapshot>,
    pub detail: NotificationDetail,
    pub active: bool,
}

pub(super) fn append(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    draft: NotificationDraft,
) -> Result<Option<NotificationId>, String> {
    let NotificationDraft {
        source,
        kind,
        round,
        sequence,
        subject,
        detail,
        active,
    } = draft;
    if campaign
        .notifications
        .receipts
        .iter()
        .any(|receipt| receipt.source == source)
    {
        return Ok(None);
    }
    let rule = data
        .notifications
        .kind(kind)
        .ok_or_else(|| format!("notification copy missing for {kind:?}"))?;
    let id = NotificationId(campaign.notifications.next_id);
    let next_id = campaign
        .notifications
        .next_id
        .checked_add(1)
        .ok_or_else(|| "campaign.notifications: receipt identifier exhausted".to_owned())?;
    let order = campaign.notifications.next_order;
    let next_order = order
        .checked_add(1)
        .ok_or_else(|| "campaign.notifications: receipt order exhausted".to_owned())?;
    campaign.notifications.receipts.push(NotificationReceipt {
        id,
        source,
        kind,
        priority: rule.default_priority,
        completed_rounds: round,
        occurred_sequence: sequence.max(1),
        occurred_order: order,
        subject,
        detail,
        is_read: false,
        is_dismissed: false,
        is_active: active,
        closed_round: None,
    });
    campaign.notifications.next_id = next_id;
    campaign.notifications.next_order = next_order;
    Ok(Some(id))
}

pub(super) fn transition_source(
    _campaign: &StrategicCampaign,
    kind: NotificationKind,
    subject: NotificationEntity,
) -> NotificationSourceId {
    NotificationSourceId::Transition {
        accepted_sequence: _campaign.accepted_sequence.max(1),
        kind,
        subject,
        ordinal: 0,
    }
}

pub(super) fn transition_draft(
    campaign: &StrategicCampaign,
    kind: NotificationKind,
    entity: NotificationEntity,
    subject: Option<NotificationSubjectSnapshot>,
    detail: NotificationDetail,
) -> NotificationDraft {
    NotificationDraft {
        source: transition_source(campaign, kind, entity),
        kind,
        round: campaign.completed_rounds,
        sequence: campaign.accepted_sequence,
        subject,
        detail,
        active: false,
    }
}

pub(super) fn set_warning(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    key: WarningKey,
    present: bool,
    detail: NotificationDetail,
    baseline: bool,
) -> Result<(), String> {
    let index = campaign
        .notifications
        .warning_episodes
        .iter()
        .position(|episode| episode.key == key);
    let index = index.unwrap_or_else(|| {
        campaign
            .notifications
            .warning_episodes
            .push(WarningEpisode {
                key,
                next_episode: 1,
                active_receipt: None,
                is_present: false,
            });
        campaign.notifications.warning_episodes.len() - 1
    });
    let was_present = campaign.notifications.warning_episodes[index].is_present;
    let active_receipt = campaign.notifications.warning_episodes[index].active_receipt;
    if present {
        if !was_present && !baseline {
            let episode = campaign.notifications.warning_episodes[index].next_episode;
            let subject = warning_entity(key.subject);
            let round = campaign.completed_rounds;
            let sequence = campaign.accepted_sequence;
            let snapshot = subject_snapshot_for_entity(campaign, subject.clone());
            if episode == u32::MAX
                || campaign
                    .notifications
                    .receipts
                    .iter()
                    .filter(|receipt| receipt.is_active)
                    .count()
                    >= data.notifications.max_active_warnings
            {
                omit_warning(campaign);
                campaign.notifications.warning_episodes[index].next_episode =
                    episode.saturating_add(1);
            } else {
                let id = append(
                    campaign,
                    data,
                    NotificationDraft {
                        source: NotificationSourceId::Warning {
                            subject: subject.clone(),
                            kind: key.kind,
                            episode,
                        },
                        kind: key.kind,
                        round,
                        sequence,
                        subject: snapshot,
                        detail: detail.clone(),
                        active: true,
                    },
                )?;
                if let Some(id) = id {
                    let episode_state = &mut campaign.notifications.warning_episodes[index];
                    episode_state.active_receipt = Some(id);
                    episode_state.next_episode = episode.saturating_add(1);
                }
            }
        } else if let Some(id) = active_receipt {
            if let Some(receipt) = campaign.notifications.receipt_mut(id) {
                receipt.detail = detail;
            }
        }
        campaign.notifications.warning_episodes[index].is_present = true;
    } else {
        if let Some(id) = active_receipt {
            if let Some(receipt) = campaign.notifications.receipt_mut(id) {
                receipt.is_active = false;
                receipt.closed_round = Some(campaign.completed_rounds);
            }
        }
        let episode = &mut campaign.notifications.warning_episodes[index];
        episode.active_receipt = None;
        episode.is_present = false;
    }
    Ok(())
}

fn omit_warning(campaign: &mut StrategicCampaign) {
    campaign.notifications.pruned_count = campaign.notifications.pruned_count.saturating_add(1);
    campaign.notifications.pruned_unread_count =
        campaign.notifications.pruned_unread_count.saturating_add(1);
}

fn warning_entity(subject: WarningSubject) -> NotificationEntity {
    match subject {
        WarningSubject::Site(id) => NotificationEntity::Site(id),
        WarningSubject::Construction(id) => NotificationEntity::Construction(id),
    }
}

fn subject_snapshot_for_entity(
    campaign: &StrategicCampaign,
    subject: NotificationEntity,
) -> Option<NotificationSubjectSnapshot> {
    match subject {
        NotificationEntity::Site(id) => place(campaign, id).map(NotificationSubjectSnapshot::Place),
        NotificationEntity::Construction(id) => campaign
            .construction
            .get(&id)
            .filter(|work| work.owner == campaign.player)
            .map(|work| NotificationSubjectSnapshot::Construction(construction(campaign, work))),
        _ => None,
    }
}

pub(super) fn person(
    campaign: &StrategicCampaign,
    id: crate::state::people::PersonId,
) -> Option<PersonNotificationSnapshot> {
    let person = campaign.people.get(&id)?;
    if person.faction != campaign.player {
        return None;
    }
    let site_id = crate::engine::person_site(campaign, id);
    let army_id = match person.assignment {
        crate::state::people::PersonAssignment::Formation { formation } => campaign
            .armies
            .values()
            .find(|army| {
                army.faction == campaign.player && army.formation_ids().any(|id| id == formation)
            })
            .map(|army| army.id),
        _ => None,
    };
    Some(PersonNotificationSnapshot {
        id,
        faction: person.faction,
        name: person.name.clone(),
        age_years: Some(person.age_years(campaign.completed_rounds)),
        class: person.class,
        site: site_id.and_then(|site| place(campaign, site)),
        army: army_id.and_then(|army| army_snapshot(campaign, army)),
        appearance: Some(person.appearance.clone()),
    })
}

pub(super) fn place(campaign: &StrategicCampaign, id: SiteId) -> Option<PlaceNotificationSnapshot> {
    let site = campaign.world.site(id)?;
    Some(PlaceNotificationSnapshot {
        id,
        name: site.name.clone(),
        controller: (site.controller == Some(campaign.player)).then_some(campaign.player),
        habitation: site.habitation,
    })
}

pub(super) fn army_snapshot(
    campaign: &StrategicCampaign,
    id: crate::state::military::ArmyId,
) -> Option<ArmyNotificationSnapshot> {
    let army = campaign.armies.get(&id)?;
    if army.faction != campaign.player {
        return None;
    }
    Some(ArmyNotificationSnapshot {
        id,
        faction: campaign.player,
        name: army.name.clone(),
        site: place(campaign, army.site),
    })
}

pub(super) fn construction(
    campaign: &StrategicCampaign,
    work: &crate::state::construction::ConstructionOrder,
) -> ConstructionNotificationSnapshot {
    let (site, route) = match work.target {
        crate::state::construction::ConstructionTarget::Site(id) => (place(campaign, id), None),
        crate::state::construction::ConstructionTarget::Route(id) => {
            let route = campaign.world.route(id).and_then(|route| {
                Some(RouteNotificationSnapshot {
                    id,
                    from: place(campaign, route.from)?,
                    to: place(campaign, route.to)?,
                })
            });
            (None, route)
        }
    };
    ConstructionNotificationSnapshot {
        id: work.id,
        owner: work.owner,
        kind: work.kind,
        target: work.target,
        progress: work.progress,
        required_steps: work.required_steps,
        status: work.status,
        site,
        route,
    }
}

fn trim(campaign: &mut StrategicCampaign, data: &GameData) {
    let rules = &data.notifications;
    let max_age = rules.max_completed_seasons;
    let receipts = std::mem::take(&mut campaign.notifications.receipts);
    let mut retained = Vec::with_capacity(receipts.len());
    for receipt in receipts {
        if receipt.is_active
            || campaign
                .completed_rounds
                .saturating_sub(receipt.completed_rounds)
                < max_age
        {
            retained.push(receipt);
        } else {
            prune_count(campaign, &receipt);
        }
    }
    campaign.notifications.receipts = retained;
    while campaign
        .notifications
        .receipts
        .iter()
        .filter(|receipt| !receipt.is_active)
        .count()
        > rules.max_receipts
    {
        let Some(index) = campaign
            .notifications
            .receipts
            .iter()
            .position(|receipt| !receipt.is_active && receipt.is_dismissed)
            .or_else(|| {
                campaign
                    .notifications
                    .receipts
                    .iter()
                    .position(|receipt| !receipt.is_active && receipt.is_read)
            })
            .or_else(|| {
                campaign
                    .notifications
                    .receipts
                    .iter()
                    .position(|receipt| !receipt.is_active)
            })
        else {
            break;
        };
        let receipt = campaign.notifications.receipts.remove(index);
        prune_count(campaign, &receipt);
    }
    campaign.notifications.warning_episodes.retain(|episode| {
        matches!(episode.key.subject, WarningSubject::Site(_))
            || matches!(
                episode.key.subject,
                WarningSubject::Construction(id)
                    if campaign.construction.get(&id).is_some_and(|order| order.is_open())
            )
            || episode.is_present
            || episode.active_receipt.is_some()
            || campaign.notifications.receipts.iter().any(|receipt| {
                matches!(
                    &receipt.source,
                    NotificationSourceId::Warning { subject, kind, .. }
                        if *kind == episode.key.kind
                            && *subject == warning_entity(episode.key.subject)
                )
            })
    });
}

fn prune_count(campaign: &mut StrategicCampaign, receipt: &NotificationReceipt) {
    campaign.notifications.pruned_count = campaign.notifications.pruned_count.saturating_add(1);
    if !receipt.is_read && !receipt.is_dismissed {
        campaign.notifications.pruned_unread_count =
            campaign.notifications.pruned_unread_count.saturating_add(1);
    }
}
