# Licensing scope and review notes

Date: 2026-11-05. The maintainer has selected **GNU General Public License version 3 only**, SPDX **`GPL-3.0-only`**, for OwlenShell's project-authored code and documentation. The [root LICENSE](../LICENSE), root Cargo package metadata and [README](../README.md#license-and-acknowledgements) record that choice. This is not an “or later” project grant.

## License text

`LICENSE` is the complete, unmodified text downloaded from [GNU's official GPLv3 text](https://www.gnu.org/licenses/gpl-3.0.txt). Two independent HTTPS downloads were byte-identical: 35,149 bytes, SHA-256:

```text
3972dc9744f6499f0f9b2dbf76696f2ae7ad8af9b23dde66d6af86c9dfb36986
```

The FSF copyright and license-document notices remain intact. The generic application instructions at the end of that official text do not change OwlenShell's explicit `GPL-3.0-only` choice. No new contributor copyright notice has been invented or added.

## Ownership and scope checked

The pre-change canonical baseline is `673d65b1bd0b949f6de8a8ba13cfb88338a63d93`. Inspection covered its 95 tracked files, package metadata, author/committer history, existing attribution and the curated evidence provenance.

The practical basis for applying the project license is the maintainer's explicit authorization, consistent maintainer attribution and no identified foreign-owned copied material in that inspected tree. The history uses three maintainer-related author spellings. GitHub associates the [initial source commit](https://github.com/VikingOwl91/owlenshell/commit/a69b23943cbe2f30d063046ef7167264b7338631) with `VikingOwl91`; the other author spellings use that account's numeric GitHub identity. Commit messages contain no additional co-author or import notices. Git attribution supports this review but is not proof of exclusive copyright ownership.

No existing project license, separate copyright notice, vendored dependency, font, screenshot, artwork or binary asset was found in the tracked baseline. The renderer snapshot contains project narrative/decision documents and source identities, not the excluded third-party assets or experimental source trees. No concrete ownership conflict was identified. This is a bounded engineering review, not a formal legal opinion or exhaustive provenance audit. A newly identified rightsholder, copied source or separately licensed component must be resolved before treating it as project-owned material.

The project license applies to material for which the project has licensing rights. Dependencies, separately licensed third-party material, attributed quotations and referenced external projects retain their own terms. Existing and future notices must be preserved; the root license must not be used to overwrite them. Referencing a file or its digest in the evidence catalogue does not relicense or distribute that omitted file.

## Dependencies and attribution

The repository has one Cargo package. Its `license` field is `GPL-3.0-only`; dependency versions, features and `Cargo.lock` are unchanged by this task.

The declared license metadata of all 560 registry packages in the lockfile was inspected from cached package manifests, with the remaining cached `libredox` 0.1.21 archive read directly and checked against its lockfile checksum. Package declarations provide available licensing options, not a file-by-file compliance guarantee. Optional and target-specific lockfile entries are not all part of the Linux runtime.

The pinned Slint 1.17.1 packages declare:

```text
GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0
```

The GPLv3 option is consistent with this project's selected license. Slint's other options are alternatives, not permissions being granted by OwlenShell. Upstream assets and other separately licensed files can have additional notices; no Slint dependency files or notices are changed here. See [Slint's licensing information](https://slint.dev/get-started). No incompatible mandatory package-level licensing condition was identified in this review. A binary/package release will still need its own corresponding-source, third-party-notice and asset/font review for the exact distributed artifacts.

Slint and Smithay Client Toolkit are dependencies, not project-owned code. GPUi Shell informed README organization and the service-integration vision. This task copied no GPUi Shell source, prose, artwork or screenshots and makes no claim about permission for future code reuse; that requires an independent review of its applicable license.

## Historical evidence and README evolution

[ADR-0006](ADR/0006-renderer-selection.md) remains Accepted. Gate P remains unpassed, and all historical failures, NO-GOs and exploratory classifications are preserved. This licensing/presentation change does not alter the renderer decision, acceptance scope, source reports or independent reviews.

The [curated provenance manifest](bakeoffs/renderer/provenance.json) describes snapshot commit `58877afe4994a3a40522fc40a96e0df7c3c8d21f`, including its original root README. This task intentionally replaces that public entry point. It does **not** update the historical manifest or claim that the current README still matches the snapshot's old README hash. The [snapshot README remains available at its original commit](https://github.com/VikingOwl91/owlenshell/blob/58877afe4994a3a40522fc40a96e0df7c3c8d21f/README.md). Original evidence documents, hashes and conclusions are unchanged.

## Validation scope

README build instructions are checked on Linux x86_64 with Rust/Cargo 1.99.1 and the checked-in lockfile. Validation uses an isolated build-output directory, two build jobs, development debug information disabled and an already populated offline Cargo cache. The inherited `sccache` wrapper failed with `Operation not permitted`; it was disabled only for the validation commands. Display and session-bus environment variables are unset for those commands. No compositor, shell application or desktop service is launched.

This is build/automated-test and static documentation validation, not a new functional acceptance run, benchmark, packaging recipe, minimum-supported-toolchain claim or distribution/hardware certification. The exact command outcomes and independent review are reported in the pull request. The license text, metadata, links, privacy and scoped diff are checked before publication. No base implementation or production migration is included.
