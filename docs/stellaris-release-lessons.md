# Early Stellaris lessons for Kestrum

Reviewed 2026-10-02. **Documentation review; no gameplay changes.**

Kestrum's next quality goal is a campaign whose decisions remain understandable
and worthwhile after the opening. Implemented systems give it a foundation;
their presence alone does not establish that experience. This review connects
an early Stellaris critique to Kestrum's existing plans and evidence. The
[map plan](map-playability-plan.md) remains the delivery authority, with M02
next. The recommendations below are analysis, not newly implemented behavior
or a separate feature queue.

## What the article contributes

Hentzau's [Thoughts: Stellaris, The Scientific Gamer, 16 May 2016](https://scientificgamer.com/thoughts-stellaris/)
praises discovery and events with lasting consequences. The criticism is that
this opening loses momentum as exploration ends, while opaque research and
combat, restrictive war outcomes, unreliable allies and cumbersome information
retrieval weaken later play. One reported war becomes impossible to finish when
its objectives change ownership. The useful distinction is between promising
systems and a coherent, usable campaign.

This is one reviewer's launch-era account, not a description of current
Stellaris or proof that Kestrum has the same defects. The applications below
are our own assessment of Kestrum's documentation and source.

## Decisions after the opening

**Existing foundation:** Kestrum already models growth, damage, recovery,
careers, aging, mentorship and succession. Its
[experience goal](01-vision-and-experience.md#intended-player-experience) asks
players to remember a person, a place and a strategic reversal. These provide
reasons to care about familiar territory after discovering it.

**Risk to assess:** recurring state changes may still leave the player mostly
advancing seasons. A biography or anniversary is valuable context, but it does
not automatically create a useful choice. More alerts would not by themselves
solve that problem.

**Recommendation:** use M05's established realm to ask what the player is
working toward, which competing use of resources they rejected, and how a
recent change alters that goal. Include a period of consolidation: repairing
damage, restoring supply, developing a holding or preparing a successor. Those
are meaningful intermediate aims under the current rules; they are not claims
that peaceful victory conditions exist. Treat deliberate waiting as valid when
the player can explain its expected payoff and what would change the plan.

Keep the roughly 20–50-year campaign target separate from evidence. The
[recorded production-victory failure](verification/border-visuals.md#validation)
at 240 rounds raises a pacing/completion question; it does not establish that
every campaign stalls or identify the cause. The existing deferred balance scope
stays in place. Record sustained campaign quality as unverified until observed.

## Preparation that players can learn from

**Existing foundation:** [combat](05-battles-and-sieges.md) has preparation,
formation roles, tactics, deterministic receipts, playback and persistent
consequences. The player already controls more than a single strength number.

**Risk to assess:** a correct resolver can still leave the player unable to
connect preparation to the result. The
[battle review](verification/battle-review.md#remaining-presentation-findings)
already records presentation limits, including dense groups and tactic controls.
Those remain scoped findings; this review does not declare combat generally
broken or reopen B01–B07.

**Recommendation:** in a representative encounter, ask the player to name a
formation's intended role, identify one visible action or reaction that mattered,
then explain one change they would make next time. Use the recorded sequence
and known terrain, morale and casualties as evidence. M02 should connect the
aftermath to the affected force and place. A richer combat interface, if needed,
belongs to a separately scoped follow-up after the gap is observed. Do not add
manual tactical control merely to make automatic combat understandable.

## Planning with uncertainty

**Existing foundation:** careers depend on relevant service, while movement,
supply, construction and diplomacy have defined eligibility rules. Unknown
enemy details are deliberately restricted. Seeded outcomes and save continuity
protect consistency; they do not by themselves tell a player how to plan.

**Recommendation:** distinguish a known prerequisite, a conditional forecast
and an unknown enemy fact. For a selected order or career, the player should
find the missing requirement and a legal step toward it without consulting
source code. Present relevant costs, blocked reasons and existing evidence near
the action. Do not guarantee an emergence roll, a safe route or a future income
receipt merely because current conditions look favorable.

Apply this to M02 inspectors and M04 contextual teaching using the existing
[career rules](06-character-development.md) and
[army rules](04-armies-and-logistics.md). The
[notification plan](notification-plan.md#advance-warnings) already distinguishes
conditional warnings from actual outcomes; preserve that distinction.

## Commitments that survive changed circumstances

**Existing foundation:** Kestrum has saved routes, access checks, route
cancellation, siege exits and explicit diplomatic decisions. Its diplomacy is
narrower than an alliance or federation system; those larger systems remain
deferred under the [decision register](13-decisions-and-open-questions.md).

**Recommendation:** document recovery as part of each continuing order, not
only its successful path. If a border closes, a destination changes control, a
participant disappears or peace invalidates a siege, the player needs the
current reason and the legal next action. Cancellation, replacement, a resolved
outcome or an explained restriction should follow the authoritative rules.
Recovery need not be free, and a genuine defeat is a valid terminal result.

M02 acceptance should exercise an interruption and reload it, showing that the
same commitment cannot charge twice, repeat a reward or leave the campaign in
an unexplained waiting state. Check the existing behavior before proposing a
new rule. AI legality and bounded execution also need to remain distinct from
whether a rival makes strategically convincing choices.

## Information that stays useful after dismissal

**Existing foundation:** current Attention shows known conditions;
[Records and histories](09-history-and-content.md) retain bounded past facts.
M02's [notification plan](notification-plan.md) adds event receipts, compact
details, subject navigation and delivery preferences. It is planned behavior.

**Recommendation:** keep these three purposes clear: what needs attention now,
what just changed, and what history is still retained. Dismissing a receipt
must not silently resolve its underlying condition. Muting a message type must
not conceal a required decision. Retrieving an event should lead to its eligible
subject without issuing an order or exposing hidden state.

Use the existing history limits. A bounded Recent view is not a promise of a
permanent universal archive. If a person has departed or detail has expired,
explain what remains known and keep current gameplay facts valid. M02 should
verify a dismissed event after reload, a changed or missing subject, and a dense
season with several outcomes. Record unnecessary navigation and lost context;
avoid prescribing a universal click count before observing the task.

## Depth from familiar places

**Existing foundation:** ownership, secure supply, local development and people
can give the same place different significance over time. M03 already plans
distinct regional networks and M04 teaches an action through its consequence.

**Recommendation:** make regional variety change a choice. A different name,
background or route length is insufficient if the same order is always best.
Review whether a player can explain why one approach, supply connection or
holding matters, and whether rebuilding or a change of commander changes their
priorities there. This uses the existing human roster and world scope. Additional
races, rare crises, more locations and larger event libraries remain deferred;
they are not substitutes for assessing the current campaign.

## Applying the review

The [M05 campaign review cases](map-playability-plan.md#campaign-review-cases)
translate these recommendations into observable scenarios. M02 carries the
immediate work on orders, explanations and retrievable consequences; M03 and
M04 retain their geographic and opening-loop responsibilities. M05 reports
what players could actually understand, including remaining combat, pacing and
continuity gaps. It does not silently expand implementation scope.

The [delivery chapter](12-delivery-and-validation.md#playability-review)
separates engineering evidence from player comprehension. This review used
documentation and source inspection, not a new campaign playtest. Earlier
captures, published builds and test counts retain their original scope. The
documentation update requires link, reference, consistency and diff checks;
it does not require publishing an unchanged game.
