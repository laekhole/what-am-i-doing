# Code signing policy

Status (2026-09-14): **preparing an application; not submitted or approved**. No waid release currently has SignPath Foundation Authenticode signing. The published v0.2.0 release has a detached Sigstore signature. See [download verification](docs/USER_GUIDE.md#download-and-verify-a-release).

## Proposed policy

The proposed source author, reviewer and release-signing approver is [laekhole](https://github.com/laekhole), the maintainer. Confirm account permissions and MFA in GitHub and SignPath before enabling signing. Each production signing request will require the maintainer's review and manual approval.

Only waid Windows binaries built from this public repository on GitHub-hosted runners will be submitted. Product metadata must identify waid and match the release version. Third-party dependencies and assets keep their own licenses; their eligibility and notices still need review before submission.

waid collects coding-agent logs locally and has no telemetry or automatic upload of session data. User-requested links, integrations, and custom HTML templates have the limits described in the [privacy and local operation notice](docs/USER_GUIDE.md#privacy-and-local-operation).

After approval and a verified production-signed release, add this attribution to the policy and release download page: Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by [SignPath Foundation](https://signpath.org). Until then, this is a proposed provider, not an endorsement.

The [Foundation's published conditions](https://signpath.org/terms.html) include project-reputation review. waid is an early public project; meeting technical prerequisites does not establish acceptance.

## Application draft

Submit through the [official application page](https://signpath.org/apply.html). The following is draft project information; the maintainer must supply their own contact details in the form.

**Project:** waid — what am I doing?

**Repository / homepage:** https://github.com/laekhole/what-am-i-doing

**Existing Windows release:** https://github.com/laekhole/what-am-i-doing/releases/tag/v0.2.0

**Privacy notice:** https://github.com/laekhole/what-am-i-doing#privacy-and-local-operation

**Maintainer type / build system:** Individual maintainer(s) / GitHub Actions

**One-line summary:** A local desktop overview of coding-agent sessions, showing their latest tasks and observed status.

**License:** MIT for waid source code; see [LICENSE](LICENSE). Bundled Pretendard fonts use the [SIL Open Font License](desktop/assets/fonts/LICENSE.txt).

**Description:** waid is a native desktop dashboard for coding-agent sessions. It reads existing local logs and shows the latest task, project, agent/model, and observed status. It does not send prompts to agents or upload session data. Claude Code and Codex are the priority sources. Windows is the current release platform; macOS development is ongoing.

**Signing request:** We would like to apply for SignPath Foundation signing for our standalone Windows x64 EXE. The existing GitHub Actions release workflow tests and builds the application on a GitHub-hosted Windows runner and publishes a portable EXE with checksums and a detached Sigstore signature. We want to add Windows Authenticode signing and timestamping to address unsigned-application trust checks. waid is an early public project without established community adoption yet. Please advise whether it can be considered at this stage and what further evidence is needed. The proposed maintainer and signing approver is the repository owner, laekhole.

**Reputation:** waid is an early-stage project. Its public repository was created on September 5, 2026, and a standalone Windows v0.2.0 release is available. The repository includes documented behavior, validation records, and GitHub Actions build workflows. We do not yet have independent reviews or evidence of broad adoption. We understand that this may be insufficient for Foundation certificate approval and would appreciate guidance on the additional evidence required.

The form also asks for the applicant's first/last name, email and discovery source. If this conversation was the first introduction, use **AI / LLM tools** for the discovery source. Complete the required terms/privacy consent and reCAPTCHA in the form; marketing consent is optional. Do not put personal contact information in this repository.

## Before submission

- Resolve the embedded OpenAI, Claude and Orca marks: their [provenance record](desktop/assets/README.md) does not establish OSI licensing, and waid's MIT license explicitly does not relicense them. Obtain acceptable evidence or replace them with the existing generic symbol and agent names. Confirm the license scope of waid's own artwork too.
- The current source embeds waid's MIT license, [third-party notices](THIRD_PARTY_NOTICES.txt), and the font license in the standalone EXE. Review the notice snapshot when dependencies or the Rust toolchain change, and verify the next release's `--licenses` output. The published v0.2.0 EXE still has only the earlier font notice. The [eligibility review](VALIDATION.md#signpath-기본-여섯-조건-대조--2026-09-14) records the original gap.
- Publish the reviewed license, policy, removal instructions and product metadata changes to the public repository; local edits are not visible to reviewers.
- Confirm maintainer MFA, repository control, component licenses, and the form's current terms. Fill personal contact fields directly in the provider's form.
- Add the public URL of this policy to the application and release download page. Do not claim approval or signed releases before they exist.

## Release integration after approval

The Windows build embeds product and version metadata from Cargo. Check an already-built EXE without running it:

```powershell
./desktop/test-version-info.ps1 -Executable ./desktop/target/release/waid-desktop.exe
```

The current release workflow remains unchanged until an approved project and signing policy are available. Follow the [official GitHub integration](https://docs.signpath.io/trusted-build-systems/github): upload the unsigned EXE as a workflow artifact, submit its artifact ID, await manual approval, and download the signed result.

Validate its Authenticode trust chain, expected publisher, RSA certificate, timestamp and waid version metadata before packaging. Run the existing [package script](desktop/package.ps1) against the signed `waid-desktop.exe` in a fresh output directory; this makes SHA256SUMS describe the signed bytes. Apply and verify the existing Sigstore signature to that final EXE, then publish. A signing failure must stop a signed release rather than publish an unsigned substitute.

Finally, download the release on a Windows machine with Smart App Control enabled and verify startup, its internal collector, and session return. Initial SmartScreen reputation warnings may still occur even with a valid signature; see [Microsoft's explanation](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation).
