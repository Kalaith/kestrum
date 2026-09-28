//! Per-campaign teaching receipts, separate from personal history and simulation.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TutorialStep {
    Headquarters,
    Movement,
    Region,
    WorldMap,
    Career,
    Household,
    FirstTurn,
    Records,
}

pub const TUTORIAL_STEPS: [TutorialStep; 8] = [
    TutorialStep::Headquarters,
    TutorialStep::Movement,
    TutorialStep::Region,
    TutorialStep::WorldMap,
    TutorialStep::Career,
    TutorialStep::Household,
    TutorialStep::FirstTurn,
    TutorialStep::Records,
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TutorialProgress {
    completed: BTreeSet<TutorialStep>,
    dismissed: bool,
}

impl Default for TutorialProgress {
    /// Migrated campaigns opt in from Help instead of interrupting established play.
    fn default() -> Self {
        Self {
            completed: BTreeSet::new(),
            dismissed: true,
        }
    }
}

impl TutorialProgress {
    pub fn new() -> Self {
        Self {
            dismissed: false,
            ..Self::default()
        }
    }

    pub fn current(&self) -> Option<TutorialStep> {
        (!self.dismissed)
            .then(|| {
                TUTORIAL_STEPS
                    .into_iter()
                    .find(|step| !self.completed(*step))
            })
            .flatten()
    }

    pub fn completed(&self, step: TutorialStep) -> bool {
        self.completed.contains(&step)
    }

    pub fn is_complete(&self) -> bool {
        self.completed.len() == TUTORIAL_STEPS.len()
    }

    /// Out-of-order visits count once; an accepted command can be observed again safely.
    pub fn record(&mut self, step: TutorialStep) {
        self.completed.insert(step);
        if self.is_complete() {
            self.dismiss();
        }
    }

    pub fn dismiss(&mut self) {
        self.dismissed = true;
    }

    pub fn reopen(&mut self) {
        if self.is_complete() {
            self.completed.clear();
        }
        self.dismissed = false;
    }
}
