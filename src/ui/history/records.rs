//! Touch discovery of public places, own armies, and legitimately known people.

use super::*;

pub(super) fn draw(ctx: &Context<'_>) -> Option<UiAction> {
    body(
        ctx,
        &ctx.text("records_help"),
        vec2(112.0, 125.0),
        18.0,
        MUTED,
    );
    for (index, (category, key)) in [
        (RecordCategory::People, "army_people"),
        (RecordCategory::Places, "history_places"),
        (RecordCategory::Armies, "own_armies"),
        (RecordCategory::Items, "legacy_item"),
    ]
    .into_iter()
    .enumerate()
    {
        if button(
            ctx,
            Rect::new(112.0 + index as f32 * 216.0, 151.0, 200.0, 48.0),
            &ctx.text(key),
            true,
            ctx.history.category == category,
        ) {
            return Some(UiAction::SetRecordCategory(category));
        }
    }
    if button(
        ctx,
        Rect::new(964.0, 151.0, 204.0, 48.0),
        &ctx.text("battle_reports"),
        true,
        false,
    ) {
        return Some(UiAction::OpenBattleReports);
    }
    let rows = rows(ctx);
    let people = ctx.history.category == RecordCategory::People;
    let total = if people {
        ctx.history
            .known_people
            .as_ref()
            .map_or(0, |page| page.total)
    } else {
        rows.len()
    };
    let offset = if people {
        ctx.history.records_page % 10 * 5
    } else {
        ctx.history.records_page * 5
    };
    if let Some(action) = draw_rows(ctx, &rows, offset) {
        return Some(action);
    }
    if people
        && button(
            ctx,
            Rect::new(294.0, 626.0, 150.0, 48.0),
            &ctx.text("history_search"),
            true,
            false,
        )
    {
        return Some(UiAction::SetHistoryMode(HistoryMode::Search));
    }
    pagination(ctx, ctx.history.records_page, total, false)
}

pub(super) fn rows(ctx: &Context<'_>) -> Vec<HistoryRow> {
    let Some(campaign) = ctx.campaign_view else {
        return Vec::new();
    };
    match ctx.history.category {
        RecordCategory::People => ctx
            .history
            .known_people
            .as_ref()
            .map(|page| {
                page.people
                    .iter()
                    .map(|person| {
                        let detail = match person {
                            PersonKnowledge::CurrentOwn(_) => ctx.text("history_own_person"),
                            PersonKnowledge::LastEncountered { snapshot, .. } => format!(
                                "{}: {} · {}",
                                ctx.text("last_encountered"),
                                date(ctx, snapshot.completed_rounds),
                                snapshot.site_name
                            ),
                        };
                        let row = HistoryRow::new(person.name().to_owned(), detail).link(
                            UiAction::OpenHistory(HistorySubject::Person(person.id())),
                            "history_open",
                        );
                        let portrait = match person {
                            PersonKnowledge::CurrentOwn(record) => {
                                let age_round = match record.status {
                                    kestrum::state::people::PersonStatus::Dead {
                                        completed_rounds,
                                        ..
                                    } => completed_rounds,
                                    _ => campaign.completed_rounds,
                                };
                                super::HistoryPortrait::CurrentOwn {
                                    appearance: record.appearance.clone(),
                                    age_years: record.age_years(age_round),
                                }
                            }
                            PersonKnowledge::LastEncountered { snapshot, .. } => {
                                super::HistoryPortrait::AdultSnapshot(snapshot.appearance.clone())
                            }
                        };
                        row.portrait(portrait)
                    })
                    .collect()
            })
            .unwrap_or_default(),
        RecordCategory::Places => campaign
            .world
            .sites
            .iter()
            .map(|site| {
                let owner = site
                    .controller
                    .and_then(|id| campaign.factions.iter().find(|f| f.id == id))
                    .map(|f| f.name.clone())
                    .unwrap_or_else(|| ctx.text("uncontrolled"));
                HistoryRow::new(
                    site.name.clone(),
                    format!("{}: {owner}", ctx.text("local_control")),
                )
                .link(
                    UiAction::OpenHistory(HistorySubject::Site(site.id)),
                    "history_open",
                )
            })
            .collect(),
        RecordCategory::Armies => campaign
            .armies
            .iter()
            .map(|army| {
                let site = campaign
                    .world
                    .site(army.site)
                    .map(|s| s.name.as_str())
                    .unwrap_or_default();
                HistoryRow::new(army.name.clone(), site.to_owned()).link(
                    UiAction::OpenHistory(HistorySubject::Army(army.id)),
                    "history_open",
                )
            })
            .collect(),
        RecordCategory::Items => campaign
            .legacy_items
            .iter()
            .map(|item| {
                let custody = match item.custody {
                    kestrum::state::legacy::LegacyItemCustody::Person(id) => campaign
                        .people
                        .iter()
                        .find(|person| person.id == id)
                        .map(|person| person.name.clone())
                        .unwrap_or_else(|| ctx.text("history_unknown_custodian")),
                    kestrum::state::legacy::LegacyItemCustody::SiteEstate(site) => campaign
                        .world
                        .site(site)
                        .map(|place| format!("{} · {}", ctx.text("legacy_estate"), place.name))
                        .unwrap_or_else(|| ctx.text("legacy_estate")),
                };
                HistoryRow::new(item.name.clone(), custody).link(
                    UiAction::OpenHistory(HistorySubject::Item(item.id)),
                    "history_open",
                )
            })
            .collect(),
    }
}
