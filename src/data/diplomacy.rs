//! P08 negotiation and truce values; no diplomacy strength is projected to rivals.

use serde::{Deserialize, Serialize};

pub const SOURCE: &str = "assets/data/diplomacy.json";
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiplomacyRules {
    pub schema_version: u32,
    pub truce_rounds: u32,
    pub recent_loss_rounds: u32,
    pub peace_strength_numerator: u32,
    pub peace_strength_denominator: u32,
    /// A war this old that has taken no recent site from the opponent is a stalemate.
    pub stalemate_rounds: u32,
}
impl DiplomacyRules {
    pub fn validate_economy(&self, economy: &super::economy::Economy) -> Result<(), String> {
        let mut common = 1_u128;
        for formation in economy.formations.values() {
            if formation.capacity == 0 {
                return Err(format!("{SOURCE}: zero formation capacity"));
            }
            let capacity = u128::from(formation.capacity);
            let mut a = common;
            let mut b = capacity;
            while b != 0 {
                let next = a % b;
                a = b;
                b = next;
            }
            common = (common / a)
                .checked_mul(capacity)
                .ok_or_else(|| format!("{SOURCE}: capacity fraction overflow"))?;
        }
        if common
            > u128::MAX / u128::from(u32::MAX) / u128::from(self.peace_strength_denominator.max(1))
        {
            return Err(format!(
                "{SOURCE}: normalized formation fractions exceed supported arithmetic"
            ));
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1
            || !(1..=100).contains(&self.truce_rounds)
            || !(1..=100).contains(&self.recent_loss_rounds)
            || !(1..=400).contains(&self.stalemate_rounds)
            || self.peace_strength_numerator == 0
            || self.peace_strength_denominator == 0
            || self.peace_strength_numerator > self.peace_strength_denominator
            || self.peace_strength_denominator > 1000
        {
            return Err(format!(
                "{SOURCE}: invalid schema, duration or strength fraction"
            ));
        }
        Ok(())
    }
}
