//! Shared semantic checks; parsing and source loading remain toolkit-owned.

use std::{collections::BTreeSet, fmt::Debug};

pub(super) fn require(source: &str, field: &str, valid: bool, reason: &str) -> Result<(), String> {
    if valid {
        Ok(())
    } else {
        Err(format!("{source}: {field}: {reason}"))
    }
}

pub(super) fn unique<T: Ord + Debug>(
    source: &str,
    field: &str,
    values: impl IntoIterator<Item = T>,
) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for value in values {
        if seen.contains(&value) {
            return Err(format!("{source}: {field}: duplicate {value:?}"));
        }
        seen.insert(value);
    }
    Ok(())
}
