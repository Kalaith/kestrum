//! A biography built from actual local service orders in the isolated capture campaign.
use super::*;

impl Game {
    pub(super) fn capture_life_history(&mut self) {
        self.capture_campaign();
        let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign else {
            return;
        };
        let teacher = campaign.people.get_mut(&PersonId(1)).expect("teacher");
        teacher.class = PersonClass::Infantry;
        teacher.birth_round = -104;
        for _ in 0..4 {
            local_season(campaign, &self.data);
        }
        let learner = engine::apply(
            campaign,
            &self.data,
            Actor::Player,
            Command::InviteApprentice { site: SiteId(1) },
        )
        .expect("invite learner")
        .new_people[0];
        engine::apply(
            campaign,
            &self.data,
            Actor::Player,
            Command::StartMentorship {
                mentor: PersonId(1),
                learner,
                discipline: kestrum::data::progression::TrainingDiscipline::Infantry,
            },
        )
        .expect("begin lessons");
        for _ in 0..4 {
            local_season(campaign, &self.data);
        }
        engine::apply(
            campaign,
            &self.data,
            Actor::Player,
            Command::TrainPerson {
                person: learner,
                class: PersonClass::Infantry,
                site: SiteId(1),
            },
        )
        .expect("begin course");
        for _ in 0..2 {
            local_season(campaign, &self.data);
        }
        engine::apply(
            campaign,
            &self.data,
            Actor::Player,
            Command::RetirePerson {
                person: learner,
                site: SiteId(1),
            },
        )
        .expect("retire local veteran");
        self.state.overlay = Overlay::Menu;
        self.open_history(HistorySubject::Person(learner));
        self.history.mode = ui::HistoryMode::Events;
    }
}

fn local_season(campaign: &mut StrategicCampaign, data: &GameData) {
    let season = campaign.completed_rounds;
    while campaign.completed_rounds == season {
        let actor = if campaign.active_faction() == campaign.player {
            Actor::Player
        } else {
            Actor::Npc(campaign.active_faction())
        };
        engine::apply(campaign, data, actor, Command::EndTurn).expect("capture season");
    }
}
