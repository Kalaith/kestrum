use kestrum::{data::GameData, state::notifications::NotificationInbox, state::StrategicCampaign};
use serde_json::json;

#[test]
fn legacy_inbox_migration_defaults_only_when_the_whole_field_is_absent() {
    let data = GameData::load().unwrap();
    let campaign = StrategicCampaign::new(&data).unwrap();
    let mut legacy = serde_json::to_value(&campaign).unwrap();
    legacy.as_object_mut().unwrap().remove("notifications");
    let migrated: StrategicCampaign = serde_json::from_value(legacy).unwrap();
    assert_eq!(migrated.notifications, NotificationInbox::default());

    let mut partial = serde_json::to_value(&campaign).unwrap();
    partial["notifications"] = json!({"schema_version": 1});
    assert!(serde_json::from_value::<StrategicCampaign>(partial).is_err());
}
