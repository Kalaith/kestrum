//! Consumed participation and retention rules, separate from combat arithmetic.

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProgressionRules {
    pub meaningful_opposition_permille: u32,
    pub meaningful_loss_permille: u32,
    pub outnumbered_permille: u32,
    pub battle_xp: u32,
    pub victory_xp: u32,
    pub outnumbered_xp: u32,
    pub round_xp_cap: u32,
    pub seasoned_xp: u32,
    pub veteran_xp: u32,
    pub ordinary_permille: u32,
    pub seasoned_permille: u32,
    pub veteran_permille: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryRules {
    pub recent_service_rounds: u32,
    pub detail_max_age_rounds: u32,
    pub detail_max_entries: usize,
    pub notable_max_age_rounds: u32,
    pub notable_max_entries: usize,
    pub knowledge_max_age_rounds: u32,
    pub knowledge_max_entries: usize,
    pub departed_max_age_rounds: u32,
    pub departed_max_entries: usize,
    pub page_size: usize,
}

impl ProgressionRules {
    pub fn validate(&self) -> Result<(), String> {
        if self.meaningful_opposition_permille > 1000
            || self.meaningful_loss_permille > 1000
            || !(1000..=10000).contains(&self.outnumbered_permille)
            || self.battle_xp == 0
            || self.round_xp_cap == 0
            || self.round_xp_cap > 100
            || self.battle_xp > self.round_xp_cap
            || self.victory_xp > self.round_xp_cap
            || self.outnumbered_xp > self.round_xp_cap
            || self.seasoned_xp == 0
            || self.veteran_xp <= self.seasoned_xp
            || self.ordinary_permille != 1000
            || self.seasoned_permille < self.ordinary_permille
            || self.veteran_permille < self.seasoned_permille
            || self.veteran_permille > 10000
        {
            return Err("progression.json: invalid significance, XP or veterancy rule".into());
        }
        Ok(())
    }
}

impl HistoryRules {
    pub fn validate(&self) -> Result<(), String> {
        if self.recent_service_rounds == 0
            || self.recent_service_rounds > 80
            || self.detail_max_age_rounds == 0
            || self.detail_max_age_rounds > self.notable_max_age_rounds
            || self.detail_max_entries == 0
            || self.detail_max_entries > 10000
            || self.notable_max_entries == 0
            || self.notable_max_entries > 12
            || self.notable_max_age_rounds > 80
            || self.knowledge_max_age_rounds == 0
            || self.knowledge_max_age_rounds > 80
            || self.knowledge_max_entries == 0
            || self.knowledge_max_entries > 10000
            || self.departed_max_age_rounds == 0
            || self.departed_max_age_rounds > 80
            || self.departed_max_entries == 0
            || self.departed_max_entries > 2000
            || self.page_size != 50
        {
            return Err("history_rules.json: invalid retention budget or page size".into());
        }
        Ok(())
    }
}
