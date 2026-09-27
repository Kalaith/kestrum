use super::*;
use crate::state::construction::{ConstructionKind, ConstructionTarget};

pub(super) fn of(command: &Command) -> Option<AiIntent> {
    let (kind, targets) = match command {
        Command::EndTurn => (AiIntentKind::Pass, vec![]),
        Command::Move(order) => (
            AiIntentKind::Move,
            order
                .armies
                .iter()
                .map(|id| id.0)
                .chain(std::iter::once(0))
                .chain(order.path.iter().map(|id| id.0))
                .collect(),
        ),
        Command::Recruit { site, army, kind } => (
            AiIntentKind::Recruit,
            vec![site.0, army.map_or(0, |id| id.0), *kind as u32],
        ),
        Command::StartConstruction {
            target,
            kind,
            builder,
        } => {
            let (target_kind, id) = match target {
                ConstructionTarget::Site(id) => (0, id.0),
                ConstructionTarget::Route(id) => (1, id.0),
            };
            let (work, facility) = match kind {
                ConstructionKind::Outpost => (0, 0),
                ConstructionKind::Road => (1, 0),
                ConstructionKind::RoadRepair => (2, 0),
                ConstructionKind::Fort => (3, 0),
                ConstructionKind::Facility(f) => (4, *f as u32),
            };
            (
                AiIntentKind::Build,
                vec![target_kind, id, work, facility, builder.0],
            )
        }
        Command::ReassignBuilder { order, builder } => (
            AiIntentKind::Reassign,
            vec![(order.0 >> 32) as u32, order.0 as u32, builder.0],
        ),
        Command::SetFocus { site, focus } => (AiIntentKind::Focus, vec![site.0, *focus as u32]),
        Command::ClearThreat { armies, threat } => (
            AiIntentKind::Threat,
            std::iter::once(threat.0)
                .chain(armies.iter().map(|id| id.0))
                .collect(),
        ),
        Command::Siege(order) => (
            AiIntentKind::Siege,
            vec![
                order.site.0,
                order.action as u32,
                order.destination.map_or(0, |id| id.0),
            ]
            .into_iter()
            .chain(order.armies.iter().map(|id| id.0))
            .collect(),
        ),
        Command::OfferPeace { faction } => (AiIntentKind::Peace, vec![faction.0]),
        Command::DeclareWar { faction } => (AiIntentKind::War, vec![faction.0]),
        Command::RelocateHeadquarters { site } => (AiIntentKind::Headquarters, vec![site.0]),
        Command::Resettle { from, to } => (AiIntentKind::Resettle, vec![from.0, to.0]),
        Command::SetCommander { army, person } => (
            AiIntentKind::Progression,
            vec![0, army.0, person.map_or(0, |id| id.0)],
        ),
        Command::TrainPerson {
            person,
            class,
            site,
        } => (
            AiIntentKind::Progression,
            vec![1, person.0, *class as u32, site.0],
        ),
        Command::PracticeRiding { person, site } => {
            (AiIntentKind::Progression, vec![2, person.0, site.0])
        }
        Command::CancelPersonCourse { person } => (AiIntentKind::Progression, vec![4, person.0]),
        Command::StartMentorship {
            mentor,
            learner,
            discipline,
        } => (
            AiIntentKind::Progression,
            vec![6, mentor.0, learner.0, *discipline as u32],
        ),
        Command::EndMentorship { learner } => (AiIntentKind::Progression, vec![7, learner.0]),
        Command::AppointGovernor { person, site } => {
            (AiIntentKind::Progression, vec![8, person.0, site.0])
        }
        Command::SpecializeFormation {
            formation,
            specialization,
            site,
        } => (
            AiIntentKind::Progression,
            vec![3, formation.0, *specialization as u32, site.0],
        ),
        Command::CancelFormationCourse { formation } => {
            (AiIntentKind::Progression, vec![5, formation.0])
        }
        Command::FormHousehold {
            first,
            second,
            site,
        } => (
            AiIntentKind::Progression,
            vec![10, first.0, second.0, site.0],
        ),
        Command::EndHousehold { household } => (AiIntentKind::Progression, vec![16, household.0]),
        Command::SetHouseholdChildraising { household, enabled } => (
            AiIntentKind::Progression,
            vec![11, household.0, u32::from(*enabled)],
        ),
        Command::AdoptWard { guardian, site } => {
            (AiIntentKind::Progression, vec![12, guardian.0, site.0])
        }
        Command::AssignTrainee { person, site } => {
            (AiIntentKind::Progression, vec![13, person.0, site.0])
        }
        Command::EnterService { person, formation } => (
            AiIntentKind::Progression,
            vec![14, person.0, formation.map_or(0, |id| id.0)],
        ),
        Command::InviteApprentice { site } => (AiIntentKind::Progression, vec![9, site.0]),
        Command::DesignateSuccessor {
            predecessor,
            successor,
            category,
            link,
        } => (
            AiIntentKind::Progression,
            vec![
                15,
                predecessor.0,
                successor.0,
                *category as u32,
                *link as u32,
            ],
        ),
        _ => return None,
    };
    Some(AiIntent { kind, targets })
}
