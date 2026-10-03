//! Durable synthetic save fixtures let browser checks use the ordinary import UI.

use super::*;

impl Game {
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn write_capture_import(&self, filename: &str) {
        assert!(self.capture && macroquad_toolkit::capture::capture_requested("KESTRUM"));
        let campaign = self
            .state
            .campaign
            .as_ref()
            .expect("capture campaign exists");
        campaign
            .validate(&self.data)
            .expect("exported capture campaign validates");
        let raw =
            macroquad_toolkit::persistence::encode_slot(campaign::STRATEGIC_SLOT, campaign, "2")
                .expect("capture slot encoding");
        let restored = kestrum::state::persistence::load_legacy(&raw, &self.data)
            .expect("capture fixture passes the ordinary import decoder");
        assert_eq!(&restored, campaign, "capture import preserves the campaign");
        std::fs::write(
            std::path::Path::new("docs/verification").join(filename),
            raw,
        )
        .expect("write the durable browser verification fixture");
    }
}
