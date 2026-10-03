//! Authored notification copy, defaults, and bounded retention settings.

use crate::state::notifications::{
    NotificationCategory, NotificationDelivery, NotificationKind, NotificationPriority,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const SOURCE: &str = "assets/data/notifications.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NotificationRules {
    pub schema_version: u32,
    pub max_receipts: usize,
    pub max_completed_seasons: u32,
    pub max_active_warnings: usize,
    pub max_rail_items: usize,
    pub categories: BTreeMap<NotificationCategory, String>,
    pub kinds: BTreeMap<NotificationKind, NotificationKindRule>,
    pub terms: BTreeMap<String, String>,
    pub first_use_help: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NotificationKindRule {
    pub category: NotificationCategory,
    pub default_priority: NotificationPriority,
    pub default_delivery: NotificationDelivery,
    pub group: bool,
    pub label: String,
    pub title: String,
    pub body: String,
}

impl NotificationRules {
    pub fn kind(&self, kind: NotificationKind) -> Option<&NotificationKindRule> {
        self.kinds.get(&kind)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err(format!("{SOURCE}: schema_version must be 1"));
        }
        if !(20..=2000).contains(&self.max_receipts)
            || !(1..=40).contains(&self.max_completed_seasons)
            || !(1..=512).contains(&self.max_active_warnings)
            || !(1..=8).contains(&self.max_rail_items)
            || self.first_use_help.trim().is_empty()
        {
            return Err(format!(
                "{SOURCE}: invalid retention, rail or help settings"
            ));
        }
        let categories: BTreeSet<_> = [
            NotificationCategory::People,
            NotificationCategory::Places,
            NotificationCategory::Security,
            NotificationCategory::Orders,
            NotificationCategory::Military,
            NotificationCategory::EconomyDiplomacy,
            NotificationCategory::Remembrance,
        ]
        .into_iter()
        .collect();
        if self.categories.keys().copied().collect::<BTreeSet<_>>() != categories
            || self
                .categories
                .values()
                .any(|label| label.trim().is_empty() || label.chars().count() > 80)
        {
            return Err(format!(
                "{SOURCE}: category labels must cover every category"
            ));
        }
        let expected: BTreeSet<_> = NotificationKind::ALL.into_iter().collect();
        if self.kinds.keys().copied().collect::<BTreeSet<_>>() != expected {
            return Err(format!(
                "{SOURCE}: notification kinds must cover the complete schema"
            ));
        }
        for (&kind, rule) in &self.kinds {
            if rule.category != kind.category()
                || (rule.group && !kind.is_groupable())
                || rule.label.trim().is_empty()
                || rule.label.chars().count() > 80
                || rule.title.trim().is_empty()
                || rule.title.chars().count() > 160
                || rule.body.trim().is_empty()
                || rule.body.chars().count() > 1200
                || !valid_template(&rule.title)
                || !valid_template(&rule.body)
            {
                return Err(format!(
                    "{SOURCE}: invalid copy, category or grouping for {kind:?}"
                ));
            }
        }
        if self.terms.len() < 20
            || self.terms.iter().any(|(key, label)| {
                key.trim().is_empty()
                    || key.chars().count() > 64
                    || label.trim().is_empty()
                    || label.chars().count() > 120
            })
        {
            return Err(format!("{SOURCE}: invalid authored event terms"));
        }
        Ok(())
    }
}

fn valid_template(template: &str) -> bool {
    let mut opening = false;
    let mut key = String::new();
    for character in template.chars() {
        match character {
            '{' if !opening => {
                opening = true;
                key.clear();
            }
            '}' if opening => {
                if key.is_empty()
                    || key.len() > 32
                    || !key
                        .chars()
                        .all(|character| character.is_ascii_alphanumeric() || character == '_')
                {
                    return false;
                }
                opening = false;
            }
            '{' | '}' => return false,
            _ if opening => key.push(character),
            _ => {}
        }
    }
    !opening
}
