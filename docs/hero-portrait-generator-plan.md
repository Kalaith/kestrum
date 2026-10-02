# Hero portrait generator

Planned on 2026-10-02. **Design only; no implementation or portrait assets exist.**
This is the first graphics workstream alongside the
[map roadmap](map-playability-plan.md#delivery-sequence). M02 remains next;
G01-G04 below can be scheduled independently after M01A and do not replace or
block M02-M05. The [documentation index](README.md) owns navigation, and this
document owns portrait scope, defaults and outstanding work.

## Outcome and scope

Give each named hero a recognizable face that remains theirs through saves,
promotion, transfer, retirement and remembered history. Compose authored parts
with randomized hair style/color, eye style/color, face shape and nose shape;
do not cycle through ten complete portraits or generate images with AI at runtime.
The same person uses the same appearance wherever their identity is visible.

Start with the existing human roster. There is no species, sex or appearance
field in [Person](../src/state/people.rs); the ordinary classes in
[world data](../src/data/world.rs) are Recruit, Infantry, Archer, Scout, Cavalry,
Medic and Officer. A portrait style must not invent ancestry statistics, fantasy
species, gender rules, classes or powers. Family relationships already exist,
but hereditary facial genetics are outside this feature. No personal-photo
collection, web asset scraping or general character-creation system is needed.

Allocate appearance for tracked people, including founders, emergent recruits,
children, wards and trainees, rather than waiting for formal Hero recognition.
Abstract troops remain abstract. Recognition can change a frame/title; it must
not replace the face. Initially use an age-neutral human style for service-age
people; dependents below the existing service-entry age can share a clearly
generic child silhouette while retaining their reserved descriptor. Youth art
and age-linked changes are a later slice, not prerequisites for hero portraits.

## Existing seams and constraints

| Current source | Consequence for the plan |
| --- | --- |
| [PersonId and Person](../src/state/people.rs), [campaign state](../src/state/campaign.rs) | Stable monotonic person IDs own appearance; names, class, formation, faction, array order and roster position are unsuitable keys. |
| [Course completion](../src/engine/progression/courses.rs), [succession commands](../src/engine/succession/commands.rs) | Class/role changes retain the person. Cover every creation path through a common allocation service. A successor is a different person, never a copied portrait. |
| [Save compatibility](../src/state/campaign/compatibility.rs) | Strategic saves currently use version 2 with explicit additive migrations and strict field validation. Extend that actual path for catalogue and legacy imports. |
| [Campaign setup](../src/game/campaign.rs), [save requests](../src/state/persistence/requests.rs) | Storage `campaign_id` can be reassigned after construction/import; it cannot be the appearance seed. |
| [Knowledge queries](../src/engine/knowledge.rs), [encounter snapshots](../src/state/knowledge.rs), [battle reports](../src/state/battle.rs) | Own people are current; encountered enemies are dated snapshots. Portraits must follow the same observation boundary. |
| [Departed-person retention](../src/engine/history/departed.rs), [history rules](../assets/data/history_rules.json) | Full dead-person records and old knowledge can be pruned. Duplicate reservations must survive separately from narrative/person retention. |
| [Resources](../src/game/resources.rs), [asset registry](../asset_registry.json) | Current art is the atlas and fonts; no modular heads, masks or portrait cache are available. Asset production is real work. |

## Persistent identity and deterministic allocation

### Descriptor and versions

Store a proposed `AppearanceDescriptor` on `Person`, plus a compact
campaign-owned appearance registry. These names describe future schema, not
fields already implemented. Save selected values explicitly:

| Field group | Planned contract |
| --- | --- |
| `schema_version` | Descriptor structure version; migrate deliberately, reject unsupported future structures through save recovery. |
| `catalog_revision`, `rig_id` | Immutable catalog/anchor contracts, initially one human head rig. Resolve old revisions after art expands. |
| `face_id`, `nose_id`, `eyes_id`, `hair_id` | Stable semantic feature IDs, never positions in a list. Catalog maps each to compatible layer assets and rig adapters. |
| `skin_palette_id`, `hair_palette_id`, `eye_palette_id` | Immutable palette entries including shade ramps. Bald hair has a canonical `none` color, not six invisible variants. |
| `allocation_revision`, `signature` | Versioned allocation policy and canonical visual signature; validate the signature against the stored choices. A seed alone is insufficient. |

Persist a campaign `appearance_salt`, allocation/catalog revision for future
people, and reservation records. Derive the salt once from saved campaign
`seed` plus a fixed portrait namespace; do not use mutable storage identity.
Default: identical campaign seeds and accepted person-creation sequences yield
identical appearances, including across native and WASM. No global uniqueness
between unrelated campaigns is promised.

Use a specified integer hash/PRNG algorithm, stable ID ordering and a dedicated
cosmetic namespace. Never consume the existing generation, combat, development
or people RNG streams. Sort eligible feature IDs before sampling; pin golden
allocation fixtures. Random choices obey the compatibility matrix rather than
retrying arbitrary broken combinations. Catalog weights are immutable within
an allocation revision and must not encode combat ability, virtue or faction.

Allocate when the person is added to an accepted campaign transaction. Reserve
the signature in the same candidate state; rejected actions, previews, redraws
and failed saves cannot consume another appearance. Batch creation/migration
uses ascending `PersonId`. UI opening order, cache misses and observer switching
must never generate or reassign identity. Import, renaming, promotion, recognition,
family membership, succession and equipment changes preserve existing descriptors.

### Migration and missing assets

- Extend the strict compatibility decoder with an explicit appearance revision.
  Only wholly absent legacy appearance state triggers initialization; partial or
  malformed new fields remain errors, not permission to reroll.
- Freeze the legacy migration's allocator and catalog revision permanently.
  The current decoder has no `GameData` argument: supply a small embedded,
  versioned migration catalog through toolkit JSON loading and the same semantic
  validator, independent of textures. Do not use the newest art catalog or hidden
  global state. Reopening an untouched old save after expansion must assign the
  same features. Keep both catalogue and legacy-import decoding covered.
- For an old save, gather the union of retained people and IDs in retained
  encounter/completed-battle snapshots and `pending_battle.report` person
  snapshots. Assign once per ID in stable order, then copy that descriptor
  consistently to every retained occurrence. Update
  [knowledge validation](../src/state/knowledge/validation.rs) with the snapshot
  migration so reports and encounters agree. Do not recreate already-pruned
  people or invent sightings/history. The reservation guarantee starts at
  migration for recoverable identities; lost historical faces cannot be recovered.
- Persist the migrated descriptor/registry in the next successful save/checkpoint
  through existing recovery. Retrying the original legacy save gives the same
  assignments. Do not overwrite named saves solely because they were opened.
- A valid descriptor with an unavailable PNG uses the fallback and retains its
  exact IDs. Unknown catalog/feature versions must preserve opaque appearance
  data losslessly if supported, or report an explicit compatibility error; never
  silently drop fields or reinterpret them as a fresh person.
- Save descriptors and reservation metadata, not textures or PNG bytes. Keep
  snapshot appearance sufficient to render after the live person is pruned.
  Pending battle, report, knowledge and future notification snapshots must not
  resolve hidden live appearance to fill missing historical information.

### Exact and perceptual duplicate policy

**Recommended default:** avoid exact duplicates across all factions in one
campaign while any compatible unused signature remains. Keep deceased portraits
reserved across generations, independently of history pruning. A canonical
signature represents visible parts and colors; omit unused channels and UI
frames. Different IDs or revisions for visually equivalent parts must resolve
to the same visual-equivalence key. A hash accelerates lookup but full keys or
collision-checked indices establish equality.

Perceptual distinction needs a separate rule. Start with catalog-authored groups
for face silhouette, hair silhouette, hair light/dark family and skin value
family. Prefer a candidate differing in at least two of these four groups from
every living person's portrait, including at least one silhouette difference;
eye/nose details are secondary tie-breakers,
not proof of thumbnail uniqueness. Different skin/eye colors alone must not
compensate for an overwhelmingly repeated silhouette. Apply the same policy to
all factions; no enemy/public allocation split. These are tunable art-review
defaults, not a claim that a distance formula predicts human recognition.

Bound allocation work and make exhaustion explicit:

1. Try up to 256 deterministically ordered legal candidates with exact exclusion
   and the preferred similarity threshold. Use an indexed living-roster feature
   summary rather than comparing against all historical portraits each time.
2. If that fails, relax the near-duplicate threshold and traverse the finite
   legal signature space deterministically, choosing an unused exact signature.
   A failed random retry budget alone must never authorize exact reuse. Keep the
   legal-space index/availability structure bounded and profile worst-case cost.
3. Only after the applicable exact space is proven exhausted, prefer a signature
   used only by deceased people, choosing its oldest last-use record with stable
   tie-breaking. If every compatible signature is in living use, choose the
   least-used signature deterministically and preserve names/roles as distinction.
   Record fallback usage for QA, not as a blocking gameplay dialog.

The registry retains each used signature and compact first/last-use metadata
and reuse count; full biographies are unnecessary. Its size is bounded by the
finite released signature space, not by seasons elapsed. Keep a separate
derived living-use index, rebuildable on load. Do not release reservations just
because a person or story was pruned. Exact reuse after saturation is an explicit
limit to the lifetime reservation promise; there is no infinite-uniqueness claim.
Future catalog revisions should make new compatible candidates available before
reuse, without changing any established descriptor. Catalog expansion and the
saved allocator-revision transition must be explicit and tested.

## Art direction and minimum asset matrix

Recommended style: stylized illustrated human busts with restrained colors,
clear outer contours and consistent front/three-quarter orientation, lighting,
eye line, stroke weight and shading. Choose one orientation for the first rig;
do not randomly mirror asymmetric layers. Art review settles the final treatment
against Kestrum's atlas and UI. Hair length/texture and face silhouettes should
provide variation across the palette; no cosmetic choice implies a statistic.

Author one shared 512x512 transparent RGBA canvas for every layer, with matching
origin and untrimmed bounds. Keep the face within the central safe area and hair
away from frame edges; document the exact safe rectangle in the catalog after
the proof sheet. Specify scalp/hairline, eye pair/brows, nose bridge/tip, mouth,
ears and neck anchors in pixels. Per-face adapters supply tested offsets and
occlusion masks; arbitrary runtime stretch is not a compatibility solution.

| Minimum release component | Proposed authored set and QA requirement |
| --- | --- |
| Face shape | 4 distinct jaw/cheek silhouettes with ears/neck, face masks and anchors. |
| Nose shape | 3 shapes with compatible placement/shading for all 4 faces: up to 12 fitted variants. |
| Eye style | 3 paired eye/brow styles with iris masks, fitted to all 4 faces: up to 12 variants. |
| Hair style | 8 silhouette IDs including bald; 7 front/rear hair pairs fitted to each face: up to 28 paired fits. Shared fits are allowed only when visually verified. |
| Color | 6 skin ramps, 6 hair ramps, 4 iris colors; authored base/shadow/highlight relationships, not arbitrary HSV noise. |
| Supporting parts | Neutral mouth per face, one simple shoulder/clothing base, clipping masks and neutral adult/child/unknown fallbacks. Clothes and frames do not multiply identity counts. |

The intended compatibility coverage is 4 x 3 x 3 x 8 = **288 geometry tuples**.
With every tuple valid, the naive color product is
4 x 3 x 3 x 8 x 6 x 4 x 6 = **41,472 combinations**. Normalizing the bald style's
unused hair color gives 4 x 3 x 3 x (1 + 7 x 6) x 4 x 6 = **37,152 nominal
distinct descriptors**. These are illustrative upper bounds, not available
assets or guaranteed recognizable faces. Excluded combinations, visually
equivalent colors and small display sizes reduce useful diversity. All 288
geometry tuples can be produced from a much smaller authored parts library;
they are not 288 individually painted heads.

Layer order is fixed by the rig: rear hair, shoulders/neck, face/ears, eye/brow
assembly, nose, mouth, front hair, optional future cosmetic overlay; UI frame and
status badges draw outside the identity composite. Per-face masks handle eye
sockets, scalp boundaries and hair that passes behind ears or shoulders. Front
bangs can occlude a brow deliberately but must not accidentally erase both eyes.
Split grayscale fill/mask channels for skin, hair and iris from untinted ink,
highlights and eye whites. Clip tint to the matching mask; never recolor by a
whole-image hue guess. Export straight-alpha PNGs with clean transparent edges;
define alpha-over and tint math once, test edge halos on light and dark surfaces.

Validate catalog IDs, adapter references, palette values, dimensions, anchor
bounds, mask alignment, alpha, allowed tuples and layer order before enabling
allocation. Reject invalid content with a source-labeled error; use the existing
UI fallback if rendering assets cannot load. Keep source artwork, exported PNGs,
catalog/license/provenance records as intentional assets. Actual filenames and
asset counts follow approved production exports, not invented placeholders.

## Compositor, caching and UI integration

Layered authored assets fit the existing Macroquad 2D texture pipeline. Kestrum
already uses toolkit `AssetManager` and `draw_texture_ex`; no engine replacement
or runtime image service is needed. The toolkit has raster helpers and
`SpriteVariationCache`, but the latter selects seeded HSV recolors and caches by
sprite/seed; it is not a modular face compositor or a bounded generational cache.

At implementation time, first check shared toolkit capabilities again. If still
absent, add reusable CPU alpha-over/mask-tint composition there, using Macroquad
images and one-time texture upload. Toolkit `AssetManager` currently owns GPU
textures; add decoded-image access for CPU composition without uploading every
source layer or reading it back from the GPU. Kestrum owns the human rig,
catalog validation, feature allocation, observer projections and UI placement.
Load the proposed
`assets/data/portrait_catalog.json` through toolkit JSON loading, register actual
exported assets in `asset_registry.json`, and verify native/WebGL publishing
includes them. Do not add a second generic project-local loader/compositor.

Compose lazily from an already assigned, observer-authorized descriptor. Cache
by descriptor content/revisions plus size and render revision, never by mutable
roster index. Proposed tiers: 128px composite for 40/64px thumbnails and 256px
for 128px detail. Use a bounded LRU, initially 128 small and 32 detail entries
(about 16 MiB RGBA texture payload before source images/driver overhead); measure
total CPU/GPU memory and decoding cost. Eviction, save/load, resizing or switching
campaigns can rebuild textures without changing appearance. Limit uploads per
frame, draw fallbacks while pending, and release old campaign/observer caches.
Never precompose every possible face or retain every generation's GPU textures.

Bound source pixels separately: one 512px RGBA plane is 1 MiB, and 28 paired hair
fits alone can occupy 56 MiB before masks. Start with a 32 MiB decoded-source LRU
budget plus one bounded composition job; retain compressed assets and load only
needed variants. Measure peak memory including decoding buffers, masks and GPU
overhead before release. If necessary ship validated 256px runtime layer exports
alongside 512px masters. Use one alpha-aware downsampling/filter policy for both
cache tiers, preserve clear ink at 40/64px, and test halos after reduction.

### Screen brief and actual sizes

| Question | Planned behavior |
| --- | --- |
| Current decision | Recognize the person being inspected or assigned, then choose an existing career, transfer or succession action. |
| Dominant focus | The roster/career decision in its sheet; on the strategic screen the map remains dominant. |
| Primary action | Existing labelled actions remain visible and retain their costs and touch targets. |
| Supporting information | Portrait, full name when inspected, role/class, status and the existing evidence. |
| Deferred information | Large art, biography and historical detail stay in deliberate inspection; no permanent hero gallery beside the map. |
| Layout and camera | Sole 1920x1080 logical canvas; existing 1280x720 management content is centered within it. Recompose the affected header/row, do not expand all sheets. |
| Input and feedback | Decorative portraits add no hidden gesture. If a portrait opens a person view later, pair it with a visible labelled tap control and preserve Back/Close. |

- [People rows](../src/ui/army/people.rs) have an 81px stride: start with a 64x64
  portrait and reflow text beside it without shrinking the 48px actions.
- [Career detail](../src/ui/army/progression/person.rs) puts identity above
  options starting around y=298 in sheet coordinates. Fit a 128x128 portrait by
  recomposing that header; never overlay the action list with a large image.
- [Formation slots](../src/ui/army/formation_slots.rs) have 52px rows: a 40x40
  portrait is optional only if name, troop type, headcount and selection survive.
  Retain symbols instead when that row cannot support it. Tiny map force marks
  remain symbols; portraits belong in a selected, identified commander's details.
- Subsequent contexts include households, known-person history, battle leaders
  and the planned [notification details](notification-plan.md). Use the same
  descriptor and crop; notification rail icons need not become tiny faces. Add
  a context only where an existing identity is permitted and useful.

Names, class/status text and selection remain authoritative. While assets are
absent, decoding fails or a cache entry is pending, keep the existing labelled
UI with a neutral silhouette/emblem; do not substitute another hero's portrait.
Unknown identity uses a universal silhouette with no descriptor-derived colors,
file key, tooltip or clickable secret person. Portrait caches are never a
knowledge query. Player and rival people use the same generator; only the
projection differs. Encountered enemy portraits come from the saved encounter,
including its date, not a live enemy lookup. Observer mode uses its separate
authorized session and must not leak appearances into the restored human game.

## Outstanding work (TODO)

Only unfinished portrait delivery belongs here. Commit each useful validated
slice; remove completed TODO entries and link their scoped evidence rather than
duplicating the map milestone ledger. All four packages are currently unstarted.

- [ ] **G01 - Identity and compatibility.** Add descriptor/catalog schema,
  immutable feature IDs, independent deterministic allocator and reservation
  index; audit every person-creation path. Migrate retained people and observed
  snapshots coherently through actual v2 decoding. Exit: old/new save fixtures,
  promotion/retention/visibility and deterministic allocation tests pass with
  all four gameplay RNG streams unchanged. Labelled fallback remains usable.
- [ ] **G02 - Art proof and first visible slice.** Produce a proof set with
  2 faces, 2 noses, 2 eye styles and 3 hair silhouettes including bald, plus
  3 skin/3 hair/2 iris colors and required masks/adapters. Establish the frozen
  rig and palette contract with a 40/64/128px contact sheet. Add any missing
  shared compositor support, bounded game cache and People/Career integration.
  Exit: a founder and an emergent hero retain visibly distinct faces after
  promotion and reload; one incomplete/missing-asset case shows the fallback.
  This is a proof set, not acceptance of campaign-wide perceptual diversity.
- [ ] **G03 - Release art and known-identity contexts.** Expand to the minimum
  release matrix above, validate every allowed geometry tuple and palette
  extremes, include real exports in the publisher, and connect eligible
  formation/household/history/battle contexts without adding permanent panels.
  Connect notification details when that M02 feature exists; it is not a
  prerequisite for People/Career portraits. Exit: no missing adapters or assets,
  no hidden-enemy leaks, and every integrated view uses the same identity.
- [ ] **G04 - Generational and readability acceptance.** Exercise old/new
  campaigns, collision-heavy tiny catalogs, saturation, history pruning and
  catalog expansion. Review dense rosters, long names, deceased memories and
  unknown enemies at actual display sizes. Record cache/registry costs and
  native/WebGL checks. Exit: explicit duplicate/reuse behavior, measured visual
  findings and remaining limits; do not claim uniqueness from the product count.

### Verification contract

For implementation, strongly target five cohesive integration cases in `tests/`:

1. Deterministic creation/migration across creation paths and native/WASM fixtures;
   all gameplay RNG states and simulation outcomes unchanged by cosmetics.
2. Save/load/import, promotion/transfer/recognition/succession and old snapshot
   retention preserve IDs/appearance; malformed/future data follows recovery.
3. Exact collisions, near-duplicate preference, bounded retries, tiny finite-space
   exhaustion and deceased reservations survive history pruning without looping.
4. Catalog additions/reordering and immutable old assets preserve earlier faces;
   invalid adapters/masks are rejected and absent PNGs use a nonmutating fallback.
5. Observer-safe portrait projections preserve snapshots and hide unknown
   identities, including after cache eviction/reconstruction and observer return.

Use table-driven variants; these are behavior tests, not five tests per helper.
Shared compositor alpha/mask/downsampling fixtures belong to the toolkit's own
feature tests; do not bundle unrelated pixel math into the projection case.
Review the actual exported assets as well: automate the geometry matrix and
palette-edge coverage, then inspect a representative stable contact sheet with
similar faces adjacent. Ask a human to match repeated heroes across shuffled
40/64/128px views, identify confusing pairs and judge silhouette/color consistency
in dense rosters. Report sample size, confusion and revisions; human readability
remains unverified until someone performs it. Large combinatorial counts and
an automated walkthrough do not establish recognition.

Follow [delivery rules](12-delivery-and-validation.md) for future runtime work:
actual checkout, shared launcher, formatting, Clippy, source-size gate, affected
tests and no-parameter publishing. Keep verification headless per
[project instructions](../PROJECT_AGENTS.md); no native/browser fullscreen
takeover. Use existing capture tooling with supported scenes and confirm exit.
Store/replace durable captures directly in `docs/verification/`, including a
stable portrait contact sheet when produced. Check actual 1920x1080 native and
browser canvas, dense states and visible tap equivalents. Smaller hosts only
scale/letterbox under the current spec; do not reinstate 720p acceptance. Preserve
the physical-touch waiver and report human/physical-touch gaps separately.

This planning change runs only documentation link/anchor, preservation,
whitespace and diff checks; it does not run game builds, tests, captures or publish.

## Defaults, decisions and later expansion

The layered human rig, persisted choices, campaign-wide reservations, finite
fallback, proposed sizes and minimum matrix are recommended implementation
defaults. They permit G01 without another approval gate. Before full art
production, settle the exact illustration treatment from G02's proof and review
whether the proposed similarity threshold offers enough visible variety. A
preference to recycle deceased faces earlier or change delivery priority should
be an explicit product decision, not an unnoticed allocator shortcut.

For backward-compatible expansion, append stable IDs and palette ramps, preserve
old exported bytes/anchors/compatibility tables, and version any changed pixels
or composition rules. New weights and catalog revisions affect new assignments
only through the saved allocation transition. Keep historical render support;
do not edit a shared color ID in place or reuse a retired feature ID for new art.

Optional later cosmetic work may add age bands, gray hair, wrinkles or earned
scars. Keep base face/nose/eye geometry and feature IDs; persist a versioned
cosmetic state rather than rerolling on birthdays or loading. Historical enemy
snapshots keep the last observed age/appearance. Family resemblance, elaborate
equipment and additional species need separate scoped decisions and compatible
art; none are hidden prerequisites of this portrait generator.
