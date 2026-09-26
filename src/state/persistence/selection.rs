//! Continue selection acknowledges committed saves independently of simulation state.

use super::*;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ContinueRecord {
    selection: Option<ContinueSelection>,
    acknowledged_order: u64,
}

impl SaveLibrary {
    pub(super) fn remember(
        &mut self,
        store: &mut impl IndexedSaveStore,
        selection: Option<ContinueSelection>,
    ) -> Result<(), String> {
        let acknowledged_order = self.latest_order().max(self.acknowledged_order);
        let raw = serde_json::to_string(&ContinueRecord {
            selection,
            acknowledged_order,
        })
        .map_err(|error| error.to_string())?;
        store.write(CONTINUE_KEY, &raw)?;
        self.last_successful = selection;
        self.acknowledged_order = acknowledged_order;
        Ok(())
    }

    pub(super) fn refresh_selection(
        &mut self,
        store: &mut impl IndexedSaveStore,
        recovered: &[u64],
    ) {
        let result = self.read_selection(store, recovered);
        let mut repair = match result {
            Ok(repair) => repair,
            Err(error) => {
                self.warnings.push(error);
                return;
            }
        };
        if let Some(ContinueSelection::Indexed(id)) = self.last_successful {
            if !self.entries().iter().any(|entry| entry.id == id) {
                self.last_successful = self.latest_entry();
                repair = true;
            }
        }
        // A payload/index can commit even when journal cleanup or the following
        // Continue write fails. A later deliberate load acknowledges every known
        // commit, so recovering storage never undoes that older-save selection.
        if self.latest_order() > self.acknowledged_order {
            self.last_successful = self.latest_entry();
            repair = true;
        }
        if repair {
            if let Err(error) = self.remember(store, self.last_successful) {
                self.warnings.push(format!(
                    "Saved campaigns are available, but Continue could not be updated: {error}"
                ));
            }
        }
    }

    fn read_selection(
        &mut self,
        store: &impl IndexedSaveStore,
        recovered: &[u64],
    ) -> Result<bool, String> {
        let Some(raw) = store.read(CONTINUE_KEY)? else {
            self.last_successful = self.latest_entry();
            self.acknowledged_order = 0;
            return Ok(self.last_successful.is_some());
        };
        let value: serde_json::Value = serde_json::from_str(&raw)
            .map_err(|error| format!("Continue selection is invalid: {error}"))?;
        if value.get("selection").is_some() {
            let record: ContinueRecord = serde_json::from_value(value)
                .map_err(|error| format!("Continue selection is invalid: {error}"))?;
            self.last_successful = record.selection;
            self.acknowledged_order = record.acknowledged_order;
            return Ok(false);
        }
        // Earlier K03 development saves used only an optional selection. Retain
        // their choice, while distinguishing newly recovered journal payloads.
        self.last_successful = serde_json::from_value(value)
            .map_err(|error| format!("Continue selection is invalid: {error}"))?;
        self.acknowledged_order = self
            .entries()
            .iter()
            .filter(|entry| {
                !recovered.contains(&entry.id)
                    || self.last_successful == Some(ContinueSelection::Indexed(entry.id))
            })
            .map(|entry| entry.metadata.success_order)
            .max()
            .unwrap_or(0);
        Ok(true)
    }

    fn latest_order(&self) -> u64 {
        self.entries()
            .iter()
            .map(|entry| entry.metadata.success_order)
            .max()
            .unwrap_or(0)
    }

    pub(super) fn latest_entry(&self) -> Option<ContinueSelection> {
        self.entries()
            .iter()
            .max_by_key(|entry| entry.metadata.success_order)
            .map(|entry| ContinueSelection::Indexed(entry.id))
    }
}
