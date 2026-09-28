//! Family reviews use the same phase and semantic guards as accepted commands.
use super::commands::helpers::{active_household_for, owned_person};
use crate::{
    data::{
        world::{FactionId, SiteId},
        GameData,
    },
    engine::{actions::validate_command, Actor, Command, RuleError},
    state::{
        people::PersonId,
        relationships::{LegacyCategory, SuccessorLink},
        StrategicCampaign,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HouseholdAction {
    Form,
    Adopt,
    Children,
    End,
    Train,
    Service,
    Invite,
    Designate,
}

#[derive(Debug, Clone, Copy)]
pub struct HouseholdSelection {
    pub site: SiteId,
    pub first: Option<PersonId>,
    pub second: Option<PersonId>,
    pub category: LegacyCategory,
    pub link: SuccessorLink,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HouseholdOption {
    pub command: Option<Command>,
    pub blocked: Option<String>,
}

pub fn household_option(
    campaign: &StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    action: HouseholdAction,
    selection: HouseholdSelection,
) -> HouseholdOption {
    let command = match command(campaign, owner, action, selection) {
        Ok(command) => command,
        Err(error) => {
            return HouseholdOption {
                command: None,
                blocked: Some(error.to_string()),
            }
        }
    };
    let actor = if owner == campaign.player {
        Actor::Player
    } else {
        Actor::Npc(owner)
    };
    let blocked = validate_command(campaign, actor, &command)
        .and_then(|()| super::validate(campaign, data, owner, &command))
        .err()
        .map(|error| error.to_string());
    HouseholdOption {
        command: Some(command),
        blocked,
    }
}

fn command(
    campaign: &StrategicCampaign,
    owner: FactionId,
    action: HouseholdAction,
    selected: HouseholdSelection,
) -> Result<Command, RuleError> {
    if !campaign.factions.contains_key(&owner) {
        return Err(RuleError::UnknownActor);
    }
    for person in [selected.first, selected.second].into_iter().flatten() {
        owned_person(campaign, owner, person)?;
    }
    let first = || {
        selected
            .first
            .ok_or_else(|| missing("Choose the first person before reviewing this action."))
    };
    let second = || {
        selected.second.ok_or_else(|| {
            missing("Choose a second person or successor before reviewing this action.")
        })
    };
    let site = selected.site;
    Ok(match action {
        HouseholdAction::Form => Command::FormHousehold {
            first: first()?,
            second: second()?,
            site,
        },
        HouseholdAction::Adopt => Command::AdoptWard {
            guardian: first()?,
            site,
        },
        HouseholdAction::Train => Command::AssignTrainee {
            person: first()?,
            site,
        },
        HouseholdAction::Service => Command::EnterService {
            person: first()?,
            formation: None,
        },
        HouseholdAction::Invite => Command::InviteApprentice { site },
        HouseholdAction::Designate => Command::DesignateSuccessor {
            predecessor: first()?,
            successor: second()?,
            category: selected.category,
            link: selected.link,
        },
        HouseholdAction::Children | HouseholdAction::End => {
            let household = active_household_for(campaign, first()?)
                .ok_or_else(|| missing("The first person needs an active household."))?;
            if action == HouseholdAction::End {
                Command::EndHousehold {
                    household: household.id,
                }
            } else {
                Command::SetHouseholdChildraising {
                    household: household.id,
                    enabled: !household.raising_children,
                }
            }
        }
    })
}

fn missing(message: &str) -> RuleError {
    RuleError::Progression(message.into())
}
