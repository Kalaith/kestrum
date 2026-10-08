//! Durable, observer-safe campaign event receipts and local delivery choices.

mod view;
pub use view::{
    NotificationGroup, NotificationProjection, NotificationSettingsCategory, NotificationTab,
    RenderedNotification,
};

use crate::{
    data::{
        economy::Habitation,
        portraits::AppearanceDescriptor,
        world::{FactionId, FounderClass, PersonClass, RouteId, SiteId},
    },
    state::{
        battle::{BattleId, BattleOutcome},
        campaign::FactId,
        construction::{ConstructionKind, ConstructionStatus, ConstructionTarget, OrderId},
        history::HistoryId,
        military::ArmyId,
        people::PersonId,
        relationships::LegacyCategory,
    },
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const NOTIFICATION_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationCategory {
    People,
    Places,
    Security,
    Orders,
    Military,
    EconomyDiplomacy,
    Remembrance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationPriority {
    Urgent,
    Warning,
    Information,
    History,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationDelivery {
    TopBarAndHistory,
    HistoryOnly,
    Off,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationKind {
    #[serde(alias = "new_hero")]
    PersonEmerged,
    PersonClassCompleted,
    PersonRecognized,
    PersonArrived,
    PersonRetired,
    PersonDied,
    CareerOpportunity,
    Succession,
    VacantCommand,
    GrowthApproaching,
    DeclineRisk,
    RuinRisk,
    HabitationChanged,
    SiteRuined,
    SiteReclaimed,
    CapitalRelocated,
    HeadquartersRelocated,
    FortStatusChanged,
    FacilityStatusChanged,
    ControlGained,
    ControlLost,
    ContestEntered,
    ContestCleared,
    LocalThreatStarted,
    LocalThreatCleared,
    OccupationStarted,
    OccupationCleared,
    SiegeStarted,
    SiegeLifted,
    MovementArrived,
    MovementBlocked,
    SupplyLost,
    SupplyRecovered,
    ArmyRetreated,
    ArmyDestroyed,
    BattleResolved,
    ConstructionCompleted,
    ConstructionExpected,
    ConstructionBlocked,
    ConstructionResumed,
    ConstructionLost,
    UnpaidUpkeepBegan,
    UnpaidUpkeepCleared,
    DiplomacyOutcome,
    Remembrance,
    LegacyTransfer,
}

pub type NotificationType = NotificationKind;

impl NotificationKind {
    pub const ALL: [Self; 46] = [
        Self::PersonEmerged,
        Self::PersonClassCompleted,
        Self::PersonRecognized,
        Self::PersonArrived,
        Self::PersonRetired,
        Self::PersonDied,
        Self::CareerOpportunity,
        Self::Succession,
        Self::VacantCommand,
        Self::GrowthApproaching,
        Self::DeclineRisk,
        Self::RuinRisk,
        Self::HabitationChanged,
        Self::SiteRuined,
        Self::SiteReclaimed,
        Self::CapitalRelocated,
        Self::HeadquartersRelocated,
        Self::FortStatusChanged,
        Self::FacilityStatusChanged,
        Self::ControlGained,
        Self::ControlLost,
        Self::ContestEntered,
        Self::ContestCleared,
        Self::LocalThreatStarted,
        Self::LocalThreatCleared,
        Self::OccupationStarted,
        Self::OccupationCleared,
        Self::SiegeStarted,
        Self::SiegeLifted,
        Self::MovementArrived,
        Self::MovementBlocked,
        Self::SupplyLost,
        Self::SupplyRecovered,
        Self::ArmyRetreated,
        Self::ArmyDestroyed,
        Self::BattleResolved,
        Self::ConstructionCompleted,
        Self::ConstructionExpected,
        Self::ConstructionBlocked,
        Self::ConstructionResumed,
        Self::ConstructionLost,
        Self::UnpaidUpkeepBegan,
        Self::UnpaidUpkeepCleared,
        Self::DiplomacyOutcome,
        Self::Remembrance,
        Self::LegacyTransfer,
    ];

    pub fn category(self) -> NotificationCategory {
        use NotificationCategory as Category;
        match self {
            Self::PersonEmerged
            | Self::PersonClassCompleted
            | Self::PersonRecognized
            | Self::PersonArrived
            | Self::PersonRetired
            | Self::PersonDied
            | Self::CareerOpportunity
            | Self::Succession
            | Self::VacantCommand => Category::People,
            Self::GrowthApproaching
            | Self::DeclineRisk
            | Self::RuinRisk
            | Self::HabitationChanged
            | Self::SiteRuined
            | Self::SiteReclaimed
            | Self::CapitalRelocated
            | Self::HeadquartersRelocated
            | Self::FortStatusChanged
            | Self::FacilityStatusChanged => Category::Places,
            Self::ControlGained
            | Self::ControlLost
            | Self::ContestEntered
            | Self::ContestCleared
            | Self::LocalThreatStarted
            | Self::LocalThreatCleared
            | Self::OccupationStarted
            | Self::OccupationCleared
            | Self::SiegeStarted
            | Self::SiegeLifted => Category::Security,
            Self::MovementArrived
            | Self::MovementBlocked
            | Self::SupplyLost
            | Self::SupplyRecovered
            | Self::ConstructionCompleted
            | Self::ConstructionExpected
            | Self::ConstructionBlocked
            | Self::ConstructionResumed
            | Self::ConstructionLost => Category::Orders,
            Self::ArmyRetreated | Self::ArmyDestroyed | Self::BattleResolved => Category::Military,
            Self::UnpaidUpkeepBegan | Self::UnpaidUpkeepCleared | Self::DiplomacyOutcome => {
                Category::EconomyDiplomacy
            }
            Self::Remembrance | Self::LegacyTransfer => Category::Remembrance,
        }
    }

    pub fn is_groupable(self) -> bool {
        !matches!(self, Self::PersonEmerged | Self::ControlLost)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NotificationId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "identity",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum NotificationEntity {
    Person(PersonId),
    Site(SiteId),
    Army(ArmyId),
    Route(RouteId),
    Construction(OrderId),
    Battle(BattleId),
    Faction(FactionId),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case", deny_unknown_fields)]
pub enum NotificationSourceId {
    Fact {
        id: FactId,
    },
    History {
        id: HistoryId,
    },
    Transition {
        accepted_sequence: u64,
        kind: NotificationKind,
        subject: NotificationEntity,
        ordinal: u32,
    },
    Warning {
        subject: NotificationEntity,
        kind: NotificationKind,
        episode: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PersonNotificationSnapshot {
    pub id: PersonId,
    pub faction: FactionId,
    pub name: String,
    #[serde(default)]
    pub age_years: Option<u32>,
    pub class: FounderClass,
    pub site: Option<PlaceNotificationSnapshot>,
    pub army: Option<ArmyNotificationSnapshot>,
    pub appearance: Option<AppearanceDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlaceNotificationSnapshot {
    pub id: SiteId,
    pub name: String,
    pub controller: Option<FactionId>,
    pub habitation: Habitation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArmyNotificationSnapshot {
    pub id: ArmyId,
    pub faction: FactionId,
    pub name: String,
    pub site: Option<PlaceNotificationSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouteNotificationSnapshot {
    pub id: RouteId,
    pub from: PlaceNotificationSnapshot,
    pub to: PlaceNotificationSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConstructionNotificationSnapshot {
    pub id: OrderId,
    pub owner: FactionId,
    pub kind: ConstructionKind,
    pub target: ConstructionTarget,
    pub progress: u32,
    pub required_steps: u32,
    pub status: ConstructionStatus,
    pub site: Option<PlaceNotificationSnapshot>,
    pub route: Option<RouteNotificationSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattleNotificationSnapshot {
    pub id: BattleId,
    pub completed_rounds: u32,
    pub outcome: BattleOutcome,
    pub site: PlaceNotificationSnapshot,
    pub own_armies: Vec<ArmyNotificationSnapshot>,
    pub retreated: Vec<ArmyNotificationSnapshot>,
    pub destroyed_armies: Vec<ArmyNotificationSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "snapshot",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum NotificationSubjectSnapshot {
    Person(Box<PersonNotificationSnapshot>),
    Place(PlaceNotificationSnapshot),
    Army(ArmyNotificationSnapshot),
    Route(RouteNotificationSnapshot),
    Construction(ConstructionNotificationSnapshot),
    Battle(BattleNotificationSnapshot),
    Faction { id: FactionId, name: String },
    History { id: HistoryId, label: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum NotificationDetail {
    Person {
        person: Box<PersonNotificationSnapshot>,
        reason_key: Option<String>,
        reason: Option<String>,
        opportunities: Vec<PersonClass>,
    },
    Place {
        place: PlaceNotificationSnapshot,
        before: Option<String>,
        after: Option<String>,
        cause: Option<String>,
        forecast_round: Option<u32>,
        conditions: Vec<String>,
    },
    Work {
        work: ConstructionNotificationSnapshot,
        before: Option<ConstructionStatus>,
        cause: Option<String>,
        forecast_round: Option<u32>,
    },
    Movement {
        armies: Vec<ArmyNotificationSnapshot>,
        destination: Option<PlaceNotificationSnapshot>,
        cause: Option<String>,
    },
    Battle {
        battle: BattleNotificationSnapshot,
    },
    Diplomacy {
        faction: Option<(FactionId, String)>,
        outcome_key: String,
        details: BTreeMap<String, String>,
    },
    Remembrance {
        subject: NotificationSubjectSnapshot,
        years: Option<u32>,
        category: Option<LegacyCategory>,
    },
    Facts {
        values: BTreeMap<String, String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NotificationReceipt {
    pub id: NotificationId,
    pub source: NotificationSourceId,
    pub kind: NotificationKind,
    pub priority: NotificationPriority,
    pub completed_rounds: u32,
    pub occurred_sequence: u64,
    pub occurred_order: u64,
    pub subject: Option<NotificationSubjectSnapshot>,
    pub detail: NotificationDetail,
    pub is_read: bool,
    pub is_dismissed: bool,
    pub is_active: bool,
    pub closed_round: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "identity",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum WarningSubject {
    Site(SiteId),
    Construction(OrderId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WarningKey {
    pub subject: WarningSubject,
    pub kind: NotificationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WarningEpisode {
    pub key: WarningKey,
    pub next_episode: u32,
    pub active_receipt: Option<NotificationId>,
    /// A baseline can be active before the campaign has a deliverable receipt.
    pub is_present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NotificationInbox {
    pub schema_version: u32,
    pub next_id: u64,
    pub next_order: u64,
    pub receipts: Vec<NotificationReceipt>,
    pub warning_episodes: Vec<WarningEpisode>,
    pub pruned_count: u64,
    pub pruned_unread_count: u64,
    pub hidden_after_id: BTreeMap<NotificationKind, u64>,
    pub top_bar_after_id: BTreeMap<NotificationKind, u64>,
    pub preference_snapshot: NotificationPreferences,
    pub baseline_complete: bool,
}

impl Default for NotificationInbox {
    fn default() -> Self {
        Self {
            schema_version: NOTIFICATION_SCHEMA_VERSION,
            next_id: 1,
            next_order: 1,
            receipts: Vec::new(),
            warning_episodes: Vec::new(),
            pruned_count: 0,
            pruned_unread_count: 0,
            hidden_after_id: BTreeMap::new(),
            top_bar_after_id: BTreeMap::new(),
            preference_snapshot: NotificationPreferences::default(),
            baseline_complete: false,
        }
    }
}

impl NotificationInbox {
    pub fn receipt(&self, id: NotificationId) -> Option<&NotificationReceipt> {
        self.receipts.iter().find(|receipt| receipt.id == id)
    }

    pub fn receipt_mut(&mut self, id: NotificationId) -> Option<&mut NotificationReceipt> {
        self.receipts.iter_mut().find(|receipt| receipt.id == id)
    }

    pub fn validate(
        &self,
        rules: &crate::data::notifications::NotificationRules,
    ) -> Result<(), String> {
        if self.schema_version != NOTIFICATION_SCHEMA_VERSION {
            return Err("campaign.notifications.schema_version: unsupported inbox schema".into());
        }
        let max_id = self
            .receipts
            .iter()
            .map(|receipt| receipt.id.0)
            .max()
            .unwrap_or(0);
        let max_order = self
            .receipts
            .iter()
            .map(|receipt| receipt.occurred_order)
            .max()
            .unwrap_or(0);
        let ids: BTreeSet<_> = self.receipts.iter().map(|receipt| receipt.id).collect();
        let orders: BTreeSet<_> = self
            .receipts
            .iter()
            .map(|receipt| receipt.occurred_order)
            .collect();
        let sources: BTreeSet<_> = self
            .receipts
            .iter()
            .map(|receipt| &receipt.source)
            .collect();
        let max_active = self
            .receipts
            .iter()
            .filter(|receipt| receipt.is_active)
            .count();
        if self.next_id == 0
            || self.next_id <= max_id
            || self.next_order == 0
            || self.next_order <= max_order
            || ids.len() != self.receipts.len()
            || orders.len() != self.receipts.len()
            || sources.len() != self.receipts.len()
            || max_active > rules.max_active_warnings
            || self.receipts.len() > rules.max_receipts.saturating_add(rules.max_active_warnings)
        {
            return Err(
                "campaign.notifications: invalid counter, duplicate identity or retention bound"
                    .into(),
            );
        }
        for receipt in &self.receipts {
            if receipt.occurred_order == 0
                || receipt.id.0 == 0
                || receipt.occurred_sequence == 0
                || (receipt.is_active && receipt.closed_round.is_some())
                || (receipt.is_active && !receipt.kind.is_forecast())
                || (!receipt.is_active
                    && receipt.kind.is_forecast()
                    && receipt.closed_round.is_none())
                || (!receipt.kind.is_forecast() && receipt.closed_round.is_some())
                || matches!(
                    &receipt.source,
                    NotificationSourceId::Transition { kind, .. }
                        | NotificationSourceId::Warning { kind, .. }
                        if *kind != receipt.kind
                )
                || (receipt.kind.is_forecast()
                    && !matches!(&receipt.source, NotificationSourceId::Warning { .. }))
            {
                return Err(
                    "campaign.notifications.receipts: invalid lifecycle or source order".into(),
                );
            }
        }
        let mut warning_keys = BTreeSet::new();
        for episode in &self.warning_episodes {
            let key = episode.key;
            if episode.next_episode == 0
                || !key.kind.is_forecast()
                || !warning_keys.insert(key)
                || (episode.active_receipt.is_some() && !episode.is_present)
                || episode.active_receipt.is_some_and(|id| {
                    !ids.contains(&id)
                        || self
                            .receipts
                            .iter()
                            .filter(|receipt| receipt.is_active && receipt.id == id)
                            .count()
                            != 1
                        || self.receipt(id).is_none_or(|receipt| {
                            !receipt.is_active
                                || receipt.kind != key.kind
                                || receipt.source
                                    != (NotificationSourceId::Warning {
                                        subject: match key.subject {
                                            WarningSubject::Site(id) => {
                                                NotificationEntity::Site(id)
                                            }
                                            WarningSubject::Construction(id) => {
                                                NotificationEntity::Construction(id)
                                            }
                                        },
                                        kind: key.kind,
                                        episode: episode.next_episode.saturating_sub(1),
                                    })
                        })
                })
            {
                return Err(
                    "campaign.notifications.warning_episodes: invalid active episode".into(),
                );
            }
        }
        for receipt in self.receipts.iter().filter(|receipt| receipt.is_active) {
            let NotificationSourceId::Warning {
                subject,
                kind,
                episode,
            } = &receipt.source
            else {
                return Err(
                    "campaign.notifications.receipts: active warning has no episode source".into(),
                );
            };
            let warning_subject = match subject {
                NotificationEntity::Site(id) => WarningSubject::Site(*id),
                NotificationEntity::Construction(id) => WarningSubject::Construction(*id),
                _ => {
                    return Err("campaign.notifications.receipts: invalid warning subject".into());
                }
            };
            let key = WarningKey {
                subject: warning_subject,
                kind: *kind,
            };
            if self
                .warning_episodes
                .iter()
                .filter(|entry| entry.key == key && entry.active_receipt == Some(receipt.id))
                .count()
                != 1
                || self
                    .warning_episodes
                    .iter()
                    .find(|entry| entry.key == key)
                    .is_none_or(|entry| entry.next_episode != episode.saturating_add(1))
            {
                return Err(
                    "campaign.notifications.receipts: active warning episode is not reverse-linked"
                        .into(),
                );
            }
        }
        if self.hidden_after_id.values().any(|id| *id >= self.next_id)
            || self.top_bar_after_id.values().any(|id| *id >= self.next_id)
        {
            return Err("campaign.notifications: invalid delivery cutoff".into());
        }
        Ok(())
    }

    pub fn mark_read(&mut self, id: NotificationId) -> bool {
        let Some(receipt) = self.receipt_mut(id) else {
            return false;
        };
        let changed = !receipt.is_read;
        receipt.is_read = true;
        changed
    }

    pub fn mark_all_read(&mut self) -> bool {
        let mut changed = false;
        for receipt in &mut self.receipts {
            if !receipt.is_dismissed && !receipt.is_read {
                receipt.is_read = true;
                changed = true;
            }
        }
        changed
    }

    pub fn dismiss(&mut self, id: NotificationId) -> bool {
        let Some(receipt) = self.receipt_mut(id) else {
            return false;
        };
        let changed = !receipt.is_dismissed;
        receipt.is_dismissed = true;
        changed
    }

    pub fn dismiss_read(&mut self) -> bool {
        let mut changed = false;
        for receipt in &mut self.receipts {
            if receipt.is_read
                && !receipt.is_dismissed
                && !receipt.is_active
                && receipt.priority == NotificationPriority::Information
            {
                receipt.is_dismissed = true;
                changed = true;
            }
        }
        changed
    }

    pub fn apply_preference_change(
        &mut self,
        previous: &NotificationPreferences,
        next: &NotificationPreferences,
        rules: &crate::data::notifications::NotificationRules,
    ) {
        let last_id = self.next_id.saturating_sub(1);
        for kind in NotificationKind::ALL {
            let before = previous.choice(kind, rules);
            let after = next.choice(kind, rules);
            if before != after
                && (before == NotificationDelivery::Off || after == NotificationDelivery::Off)
            {
                self.hidden_after_id.insert(kind, last_id);
            }
            if before != NotificationDelivery::TopBarAndHistory
                && after == NotificationDelivery::TopBarAndHistory
            {
                self.top_bar_after_id.insert(kind, last_id);
            }
        }
        self.preference_snapshot = next.clone();
    }

    /// Applies locally stored choices when a saved campaign is opened. Cutoffs
    /// belong to the campaign so a different local preference cannot replay a
    /// previously suppressed backlog into the compact rail.
    pub fn sync_preferences(
        &mut self,
        next: &NotificationPreferences,
        rules: &crate::data::notifications::NotificationRules,
    ) {
        let previous = self.preference_snapshot.clone();
        self.apply_preference_change(&previous, next, rules);
    }
}

impl NotificationKind {
    pub fn is_forecast(self) -> bool {
        matches!(
            self,
            Self::GrowthApproaching
                | Self::DeclineRisk
                | Self::RuinRisk
                | Self::ConstructionExpected
        )
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NotificationPreferences {
    pub choices: BTreeMap<NotificationKind, NotificationDelivery>,
}

impl NotificationPreferences {
    pub fn choice(
        &self,
        kind: NotificationKind,
        rules: &crate::data::notifications::NotificationRules,
    ) -> NotificationDelivery {
        self.choices
            .get(&kind)
            .copied()
            .or_else(|| rules.kind(kind).map(|rule| rule.default_delivery))
            .unwrap_or(NotificationDelivery::Off)
    }

    pub fn all_off(&mut self) {
        self.choices = NotificationKind::ALL
            .into_iter()
            .map(|kind| (kind, NotificationDelivery::Off))
            .collect();
    }

    pub fn restore_defaults(&mut self) {
        self.choices.clear();
    }
}
