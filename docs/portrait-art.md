# Kestrum portrait art

The user rejected the original vector proof and selected a cleaner anime direction on 2026-10-03. The current G02 exports are assembled from illustrated transparent components: confident linework, sculpted hair, expressive eyes and restrained cel shading. They establish the first production layer contract; they do not establish campaign-wide recognition or complete the larger release matrix.

## Sources and provenance

The [direction sheet](verification/portrait_direction.png) is an offline generated style concept, not four runtime portraits. Its exact prompt and provenance are in [concept.json](../assets/art-source/portraits/concept.json). The built-in image-generation tool does not expose its backend model; no image-model upgrade is claimed.

The [source manifest](../assets/art-source/portraits/illustrated/source-manifest.json) preserves every selected component's successful prompt sequence, source crop and fixed destination registration. Twelve G02 transparent source images supply two bald face masters, a neutral mouth, two eye/brow assemblies, two noses, front/rear cropped and coiled hair, and one neck/tunic assembly with explicit material channels. Fourteen additional G03 masters supply square and round faces, round eyes, an aquiline nose, and front/rear wavy, braided, topknot, long and undercut hair. The mouth is separate during authoring so it can be registered at y=354, then baked into each face export. The supporting assembly is split into matching neck and clothing layers. No new identity fields or runtime image generation are involved.

Generated placement is approximate: the exporter registers components deterministically on the shared canvas, and the assembled output is visually reviewed. Near-opaque source coverage at or above 240/255 is made opaque; lower antialiased edge coverage is retained. RGB stored under alpha zero is ignored. The original vector source remains in [PortraitArtSource.cs](../assets/art-source/portraits/PortraitArtSource.cs) as earlier authored work and the reference compositor/validation harness; its rejected path drawings are not the current exported portrait art.

## Export and review

From the project root:

```powershell
.\scripts\generate_portrait_art.ps1
.\scripts\generate_portrait_art.ps1 -ValidateOnly
```

The normal command delegates to [export_illustrated_portraits.ps1](../scripts/export_illustrated_portraits.ps1), whose [exporter](../assets/art-source/portraits/IllustratedPortraitExporter.cs) preflights all registered source components before writing runtime layers. It then validates actual exported composition and replaces the stable [contact sheet](verification/portrait_contact_sheet.png). Validation-only does not rewrite exports or the sheet. The [export manifest](../assets/art-source/portraits/illustrated/export-manifest.json) lists the 111 runtime paths and SHA-256 hashes. Only those runtime assets belong in the publisher; high-resolution masters and prompts are authoring inputs.

The current contact sheet uses the actual exported source/mask/ink planes and final illustrated supporting art. The superseded single-geometry study and its obsolete preview script were removed after this checkpoint was accepted.

## Frozen rig and material contract

All runtime layers are untrimmed 512x512 RGBA PNGs on `human_bust_front_v1`. The safe rectangle is `[40,24,472,496]`. Existing centerline anchors remain crown `[256,32]`, scalp center `[256,116]`, eye line `[256,236]`, nose bridge `[256,270]`, nose tip `[256,316]`, mouth `[256,354]`, chin `[256,408]`, neck `[256,444]` and shoulders `[256,482]`. Bald uses no hair layers.

Skin and hair luminance are decomposed into base/shadow/highlight ramp weights plus dark neutral ink. Plane alpha is solved from the final layer backwards so straight-alpha over reconstructs the reviewed mixture, including translucent edges. Sources carry grayscale value in red; masks provide white red/alpha coverage. Source alpha, mask red, mask alpha and tint alpha multiply. The original source's green iris pigment and red neck pigment are explicit authored material markers, not whole-image hue guesses. Eyes retain untinted whites, pupils, linework and highlights. Noses use skin-base tint at 70% authored coverage to keep their form restrained. Clothing uses the fixed neutral ramp.

Composition uses the fixed order: rear hair, shoulders, neck, face fills, eye whites/iris/ink, nose, face ink and mouth, front hair. Each shaded assembly uses base, shadow, highlight and ink, with face ink deferred as stated. The neutral clothing shadow/base/highlight RGB values are `[39,51,58]`, `[84,101,108]` and `[153,164,157]`. The source code and runtime renderer must agree on that order, alpha-over and tint math. Contact-sheet reduction uses exact area filtering in premultiplied-alpha space, followed by unpremultiplication; clamped byte conversion rounds halfway away from zero. Light and dark surfaces expose edge halos.

`scripts/export_illustrated_portraits.ps1 -ParityOnly` emits durable 128px and 256px straight-alpha reference composites for the fixed oval/upturned/lidded/coiled/ochre/flax/hazel v1 descriptor. [Parity provenance](../assets/art-source/portraits/illustrated/parity_v1.json) records its exact signature, frozen source-manifest hash, composition order and rounding policy. The delegated run passed, producing [128px](../assets/art-source/portraits/illustrated/parity_v1_128.png) and [256px](../assets/art-source/portraits/illustrated/parity_v1_256.png) PNGs. Their SHA-256 hashes are `B51D00EFE4E553B6006E33CAD0CC088D052A87F5C8465C8DC7D16F4DBA8D9C46` and `3D2A9272887073B9ED259D6ECEB470D1C2D9811A8D9A08B70A36B8C791471883`. These are authoring fixtures, not shipping portrait assets. `-ReviewOnly` refreshes review sheets from existing runtime layers without exporting new ones.

| Art | Paths under `assets/portraits/` | Count |
| --- | --- | ---: |
| Faces | `faces/{face}/{base,shadow,highlight}_{source,mask}.png`, `ink.png` | 14 |
| Nose fits | `noses/{face}/{nose}_{source,mask}.png` | 8 |
| Eye fits | `eyes/{face}/{eyes}_{whites,iris_source,iris_mask,ink}.png` | 16 |
| Hair fits | `hair/{face}/{hair}/{rear,front}_{base,shadow,highlight}_{source,mask}.png`, `{rear,front}_ink.png` | 56 |
| Supporting art | `supporting/{neck,shoulders}/{base,shadow,highlight}_{source,mask}.png`, `ink.png` | 14 |
| Neutral fallbacks | `supporting/{adult_fallback,child_silhouette,unknown_silhouette}.png` | 3 |

The two faces are `face_oval` and `face_tapered`; noses are `nose_straight` and `nose_upturned`; eyes are `eyes_open` and `eyes_lidded`. Hair includes `hair_bald`, `hair_cropped` and `hair_coiled`. The fallback drawings remain simple, identity-neutral authored artwork. Existing catalog IDs, visual keys, palette RGBs, legacy migration and reservation signatures are unchanged.

## G02 verification and limits

The delegated export check exited successfully and fully raster-composited all 336 legal palette combinations across all 24 geometry tuples. All 111 runtime PNGs decode as nonempty aligned 512x512 RGBA, all 44 source/mask pairs align, and composed pixels remain inside the safe rectangle. The stable 880x1814 contact sheet has SHA-256 `F229E7D013FE52D09CB53F9FE8B578893CA69146B0A02B69D9BDFD6FB5D15439`.

Art review found clear cropped/coiled silhouettes and readable eyes at 64/128px. The tapered jaw differs from the oval, but that distinction is subtle at 40px; nose changes are not a thumbnail uniqueness basis. Umber skin with ebony hair is subdued against charcoal backgrounds. These are observed visual limits, not proof of human recognition.

The G02 checkpoint is accepted and its 111 exported bytes and metadata remain frozen. The exporter verifies their SHA-256 hashes before and after every expanded export and skips writing those paths. Later production appends new features and per-face fits rather than repainting established identities.

## G03 review in progress

The complete authored source set is ready for the four-face/three-eye/three-nose/eight-hair release matrix. The append-only exporter is designed to produce 509 runtime PNGs with 210 source/mask pairs. The review harness uses the actual revision-2 catalog, enumerates 37,152 legal descriptors and rasterizes 864 baseline/extreme composites across all 288 geometry tuples. It also checks that front hair retains at least 45% of each iris's authored coverage. Those counts describe the intended checks until the delegated export run passes; they are not a claim that every legal descriptor has been rasterized.

Final evidence will include the representative 40/64/128px contact sheet, every geometry at 64px, and all six skin/hair palette combinations on light and dark surfaces. Runtime UI/render parity and a human recognition trial remain separate acceptance work.
