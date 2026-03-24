---
title: "feat: CI/CD Redesign — PR Reporter, Security Audit, GoReleaser CD, Install Script"
type: feat
status: completed
date: 2026-03-25
origin: docs/brainstorms/2026-03-25-cicd-redesign-brainstorm.md
---

# feat: CI/CD Redesign

## Overview

Extend the existing CI/CD workflows with four additions:

1. **PR Test Reporter** — posts pass/fail summary + collapsible logs as a PR comment on every CI run
2. **Security Audit CI** — runs `cargo audit` on every PR to main, reports findings as a non-blocking PR comment
3. **GoReleaser CD** — replaces the manual packaging/release steps in `release.yml` with GoReleaser; adds auto-bump of `Cargo.toml` version from the tag before building
4. **Install Script** — GoReleaser-generated `install.sh` in repo root for one-command binary installation

---

## Problem Statement / Motivation

The existing workflows (`test.yml`, `release.yml`) run but provide no visibility into results on PRs, have no security scanning, require the developer to manually keep `Cargo.toml` version in sync with tags, and produce raw binaries with no standardized install path for end users.

---

## Proposed Solution

### Architecture Overview

```
PRs to main:
  test.yml  ──────────────────────── (existing jobs: fmt, clippy, unit-test, integration-test)
                │                                          │
                └──── report job (if: always()) ──────────┘
                         │
                         └── POST/UPDATE PR comment (summary table + <details> logs)

  security.yml ─── cargo audit ──── POST/UPDATE PR comment (non-blocking)

Developer release flow:
  1. bump Cargo.toml version locally
  2. commit → PR → merge to main
  3. git tag v1.0.0 && git push origin v1.0.0

Tag push (v*.*.*):
  release.yml ─── verify job ─── tag version == Cargo.toml? (fail fast if not)
                      │
                      └── build matrix (4 targets) ──────────────────────┐
                                                                          │
                      └── goreleaser job ──── .goreleaser.yml ─── collect binaries
                                                                        │
                                              │  Package .tar.gz per target
                                              │  Generate checksums.txt
                                              │  Create GitHub Release
                                              └── crates.io publish
```

---

## Technical Approach

### Phase 1: PR Test Reporter (modify `test.yml`)

**Changes to `.github/workflows/test.yml`:**

Add `pull-requests: write` to workflow-level permissions.

Add a `report` job at the end:

```yaml
report:
  name: PR Test Report
  needs: [fmt, clippy, unit-test, integration-test]
  runs-on: ubuntu-latest
  if: always() && github.event_name == 'pull_request'
  permissions:
    pull-requests: write
  steps:
    - uses: actions/github-script@v7
      with:
        script: |
          const jobs = {
            'fmt':              '${{ needs.fmt.result }}',
            'clippy':           '${{ needs.clippy.result }}',
            'unit-test':        '${{ needs.unit-test.result }}',
            'integration-test': '${{ needs.integration-test.result }}',
          };
          const icon = r => r === 'success' ? '✅' : r === 'skipped' ? '⏭️' : '❌';
          const rows = Object.entries(jobs)
            .map(([name, result]) => `| ${name} | ${icon(result)} ${result} |`)
            .join('\n');
          const body = [
            '<!-- ci-report -->',
            '## CI Results',
            '',
            '| Job | Status |',
            '|-----|--------|',
            rows,
          ].join('\n');

          // find-or-update existing comment
          const { data: comments } = await github.rest.issues.listComments({
            owner: context.repo.owner,
            repo: context.repo.repo,
            issue_number: context.issue.number,
          });
          const existing = comments.find(c => c.body.includes('<!-- ci-report -->'));
          if (existing) {
            await github.rest.issues.updateComment({
              owner: context.repo.owner,
              repo: context.repo.repo,
              comment_id: existing.id,
              body,
            });
          } else {
            await github.rest.issues.createComment({
              owner: context.repo.owner,
              repo: context.repo.repo,
              issue_number: context.issue.number,
              body,
            });
          }
```

**Note:** Full job logs are not accessible via the GitHub API from within `actions/github-script`. The comment shows pass/fail per job; developers click the linked job name in the Actions UI for full logs. This is the correct approach — trying to capture raw `cargo test` output requires step-level log upload as artifacts, which adds significant complexity for marginal benefit given the collapsible detail format.

**Permission note:** The workflow-level `permissions: contents: read` must be extended. Since different jobs need different permissions, use job-level `permissions` blocks rather than a single workflow-level block.

---

### Phase 2: Security Audit CI (new `security.yml`)

**New file: `.github/workflows/security.yml`**

```yaml
name: Security Audit

on:
  pull_request:
    branches: [main]

permissions:
  contents: read
  pull-requests: write

jobs:
  audit:
    runs-on: ubuntu-latest
    timeout-minutes: 10
    steps:
      - uses: actions/checkout@v6
        with:
          persist-credentials: false
      - name: Install cargo-audit
        uses: taiki-e/install-action@v2
        with:
          tool: cargo-audit
      - name: Run cargo audit
        id: audit
        run: |
          cargo audit --json > audit-results.json 2>&1 || true
          echo "exit_code=$?" >> $GITHUB_OUTPUT
      - name: Post audit results
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            let body;
            try {
              const raw = fs.readFileSync('audit-results.json', 'utf8');
              const report = JSON.parse(raw);
              const vulns = report.vulnerabilities?.list ?? [];
              if (vulns.length === 0) {
                body = '<!-- security-audit -->\n## Security Audit\n\n✅ No vulnerabilities found.';
              } else {
                const rows = vulns.map(v =>
                  `| ${v.advisory.id} | ${v.advisory.package.name} | ${v.advisory.cvss ?? 'N/A'} | ${v.advisory.title} |`
                ).join('\n');
                body = [
                  '<!-- security-audit -->',
                  '## Security Audit',
                  '',
                  `⚠️ Found ${vulns.length} vulnerabilit${vulns.length === 1 ? 'y' : 'ies'}`,
                  '',
                  '| Advisory | Crate | CVSS | Description |',
                  '|----------|-------|------|-------------|',
                  rows,
                  '',
                  '_This is informational only and does not block merging._',
                ].join('\n');
              }
            } catch {
              body = '<!-- security-audit -->\n## Security Audit\n\n⚠️ Audit failed to parse results.';
            }
            // find-or-update
            const { data: comments } = await github.rest.issues.listComments({
              owner: context.repo.owner, repo: context.repo.repo,
              issue_number: context.issue.number,
            });
            const existing = comments.find(c => c.body.includes('<!-- security-audit -->'));
            if (existing) {
              await github.rest.issues.updateComment({
                owner: context.repo.owner, repo: context.repo.repo,
                comment_id: existing.id, body,
              });
            } else {
              await github.rest.issues.createComment({
                owner: context.repo.owner, repo: context.repo.repo,
                issue_number: context.issue.number, body,
              });
            }
```

**Non-blocking by design** (see brainstorm: `docs/brainstorms/2026-03-25-cicd-redesign-brainstorm.md`): the audit step uses `|| true` so a finding never fails the job. The job is informational only.

---

### Phase 3: GoReleaser CD (rewrite `release.yml` + add `.goreleaser.yml`)

#### 3a. GoReleaser Config: `.goreleaser.yml`

GoReleaser's native build system is Go-centric. For Rust, use the `--skip=build` flag and pre-build binaries via the existing cargo matrix, then hand off to GoReleaser for packaging and release only.

```yaml
# .goreleaser.yml
version: 2

before:
  hooks: []

# We skip GoReleaser's build phase entirely.
# Binaries are pre-built by the GitHub Actions matrix and placed in dist/.
builds:
  - skip: true

archives:
  - id: default
    name_template: "solana-onchain-mcp-{{ .Os }}-{{ .Arch }}"
    format: tar.gz
    files:
      - none*  # only include the binary

checksum:
  name_template: "checksums.txt"
  algorithm: sha256

release:
  github:
    owner: widnyana
    name: solana-onchain-mcp
  name_template: "{{ .Tag }}"
  draft: false
  prerelease: auto

changelog:
  sort: asc
  filters:
    exclude:
      - "^docs:"
      - "^test:"
      - "^chore:"
```

**Implementation note on GoReleaser + Rust:** GoReleaser expects pre-built binaries to be placed in `dist/<target>/` following its naming convention when using `--skip=build`. The GitHub Actions matrix builds each target, renames the binary, and uploads as an artifact. A final job downloads all artifacts into the `dist/` layout GoReleaser expects, then runs `goreleaser release --skip=build`.

#### 3b. Release Workflow: `.github/workflows/release.yml` (rewrite)

**Developer release flow** (see brainstorm: auto-bump dropped):
1. Bump `Cargo.toml` version locally
2. Commit → PR → merge to main
3. `git tag v1.0.0 && git push origin v1.0.0`

```yaml
name: Release

on:
  push:
    tags:
      - "v*.*.*"

permissions:
  contents: read

jobs:
  # Step 1: Verify tag version matches Cargo.toml
  verify:
    runs-on: ubuntu-latest
    timeout-minutes: 5
    outputs:
      version: ${{ steps.extract.outputs.version }}
    steps:
      - uses: actions/checkout@v6
        with:
          persist-credentials: false
      - name: Extract version from tag
        id: extract
        run: echo "version=${GITHUB_REF#refs/tags/v}" >> $GITHUB_OUTPUT
      - name: Verify tag matches Cargo.toml
        run: |
          CARGO_VERSION=$(grep '^version = ' Cargo.toml | head -1 | cut -d '"' -f2)
          TAG_VERSION="${{ steps.extract.outputs.version }}"
          if [ "$CARGO_VERSION" != "$TAG_VERSION" ]; then
            echo "Version mismatch: Cargo.toml=${CARGO_VERSION}, tag=${TAG_VERSION}"
            echo "Bump Cargo.toml version and merge to main before tagging."
            exit 1
          fi

  # Step 2: Build binaries per target (matrix)
  build:
    needs: [verify]
    runs-on: ${{ matrix.runner }}
    timeout-minutes: 30
    strategy:
      fail-fast: false
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            runner: ubuntu-latest
            use_cross: false
          - target: aarch64-unknown-linux-gnu
            runner: ubuntu-latest
            use_cross: true
          - target: x86_64-apple-darwin
            runner: macos-13
            use_cross: false
          - target: aarch64-apple-darwin
            runner: macos-latest
            use_cross: false
    steps:
      - uses: actions/checkout@v6
        with:
          persist-credentials: false
      - uses: mozilla-actions/sccache-action@v0.0.9
        if: "!matrix.use_cross"
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - name: Install cross
        if: matrix.use_cross
        uses: taiki-e/install-action@v2
        with:
          tool: cross
      - name: Build release binary
        run: |
          if [ "${{ matrix.use_cross }}" = "true" ]; then
            cross build --release --target ${{ matrix.target }}
          else
            cargo build --release --target ${{ matrix.target }}
          fi
        env:
          OPENSSL_STATIC: 1
          RUSTC_WRAPPER: ${{ matrix.use_cross == true && '' || 'sccache' }}
          SCCACHE_GHA_ENABLED: ${{ matrix.use_cross == true && 'false' || 'true' }}
      - name: Strip binary
        if: "!matrix.use_cross"
        run: strip target/${{ matrix.target }}/release/solana-onchain-mcp
      - name: Upload binary artifact
        uses: actions/upload-artifact@v4
        with:
          name: binary-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/solana-onchain-mcp

  # Step 3: GoReleaser packages + creates GitHub Release
  goreleaser:
    needs: [verify, build]
    runs-on: ubuntu-latest
    timeout-minutes: 15
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@v6
        with:
          fetch-depth: 0
          persist-credentials: false
      - name: Download all binaries
        uses: actions/download-artifact@v4
        with:
          path: artifacts
          pattern: binary-*
          merge-multiple: false
      - name: Arrange binaries for GoReleaser
        run: |
          # GoReleaser --skip=build expects: dist/<project>_<os>_<arch>/<binary>
          mkdir -p dist
          declare -A targets=(
            ["binary-x86_64-unknown-linux-gnu"]="solana-onchain-mcp_linux_amd64_v1"
            ["binary-aarch64-unknown-linux-gnu"]="solana-onchain-mcp_linux_arm64"
            ["binary-x86_64-apple-darwin"]="solana-onchain-mcp_darwin_amd64_v1"
            ["binary-aarch64-apple-darwin"]="solana-onchain-mcp_darwin_arm64"
          )
          for artifact_dir in "${!targets[@]}"; do
            goreleaser_dir="dist/${targets[$artifact_dir]}"
            mkdir -p "$goreleaser_dir"
            cp "artifacts/${artifact_dir}/solana-onchain-mcp" "$goreleaser_dir/solana-onchain-mcp"
          done
      - uses: goreleaser/goreleaser-action@v6
        with:
          distribution: goreleaser
          version: latest
          args: release --skip=build --clean
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

  # Step 4: Publish to crates.io
  publish-crates:
    needs: [goreleaser]
    runs-on: ubuntu-latest
    timeout-minutes: 15
    environment: release
    continue-on-error: true
    steps:
      - uses: actions/checkout@v6
        with:
          persist-credentials: false
      - uses: mozilla-actions/sccache-action@v0.0.9
      - uses: dtolnay/rust-toolchain@stable
      - name: Publish to crates.io
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CRATESIO_TOKEN }}
          SCCACHE_GHA_ENABLED: "true"
          RUSTC_WRAPPER: "sccache"
        run: cargo publish
```

**Key design decisions carried from brainstorm:**
- No auto-bump — developer is responsible for bumping `Cargo.toml` before tagging; `verify` job enforces this with a fast-fail check (see brainstorm: auto-bump dropped)
- No PAT needed — standard `GITHUB_TOKEN` is sufficient since no commits/force-pushes are made by the workflow
- `fail-fast: false` on build matrix so all targets attempt to build even if one fails
- `goreleaser` uses `--skip=build` — GoReleaser only handles packaging and release, not Rust compilation
- `publish-crates` has `continue-on-error: true` — GitHub Release proceeds even if crates.io publish fails

---

### Phase 4: Install Script (`install.sh`)

The install script is generated via GoReleaser's installer template or written as a standalone script derived from GoReleaser's asset naming convention. Since GoReleaser's `godownloader` tool is deprecated, write a maintainable bash script that follows the same pattern.

**File: `install.sh`** (repo root)

The script must:

1. Detect OS: `uname -s` → `Linux` or `Darwin`
2. Detect arch: `uname -m` → `x86_64` → `amd64`, `arm64`/`aarch64` → `arm64`
3. Construct the asset name matching GoReleaser's archive naming: `solana-onchain-mcp-<Os>-<Arch>.tar.gz`
4. Fetch the latest release tag from the GitHub API
5. Download the asset
6. Verify checksum against `checksums.txt` from the release
7. Extract, move binary to `${INSTALL_DIR:-$HOME/.local/bin}`
8. Create `INSTALL_DIR` if it doesn't exist
9. Clean up: remove downloaded archive and extracted directory

**Script structure:**

```bash
#!/usr/bin/env bash
set -euo pipefail

REPO="widnyana/solana-onchain-mcp"
BINARY_NAME="solana-onchain-mcp"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# detect_os, detect_arch, fetch_latest_tag, download_and_verify, install, cleanup functions
```

Dependencies: `curl`, `tar`, `sha256sum` (Linux) or `shasum -a 256` (macOS).

---

## Acceptance Criteria

### Feature 1: PR Test Reporter

- [x] `test.yml` gains a `report` job with `if: always()`
- [x] Report job only runs on `pull_request` events (not on push to main)
- [x] PR comment is created on first run, updated (not duplicated) on subsequent runs
- [x] Comment uses HTML marker `<!-- ci-report -->` for find-or-update
- [x] Comment shows pass/fail for: fmt, clippy, unit-test, integration-test
- [x] Workflow-level permissions do not regress (no over-broad permissions)

### Feature 2: Security Audit

- [x] New `security.yml` workflow file exists
- [x] Triggers only on `pull_request` targeting `main`
- [x] Runs `cargo audit` on `Cargo.lock`
- [x] Posts/updates a PR comment with `<!-- security-audit -->` marker
- [x] Job never fails the workflow on vulnerability findings (non-blocking)
- [x] Comment clearly states "informational only, does not block merging"

### Feature 3: GoReleaser CD

- [x] `.goreleaser.yml` exists and is valid (`goreleaser check`)
- [x] `release.yml` triggers on `v*.*.*` tag pushes
- [x] `verify` job: fails fast if tag version does not match `Cargo.toml` version
- [x] Build matrix: all 4 targets produce binaries (`fail-fast: false`)
- [x] GoReleaser: creates GitHub Release with 4 `.tar.gz` archives + `checksums.txt`
- [x] Archive naming: `solana-onchain-mcp-<os>-<arch>.tar.gz`
- [x] crates.io publish runs post-release with `continue-on-error: true`
- [x] No PAT secret required — standard `GITHUB_TOKEN` is sufficient
- [x] Developer workflow is documented: bump Cargo.toml → commit → PR → merge → tag

### Feature 4: Install Script

- [x] `install.sh` exists in repo root
- [x] Detects Linux/macOS correctly
- [x] Detects x86_64 and arm64 correctly
- [x] Downloads from correct GitHub Release asset URL
- [x] Verifies checksum before installing
- [x] Installs to `$HOME/.local/bin` by default
- [x] `INSTALL_DIR` env var overrides install location
- [x] Creates `INSTALL_DIR` if it doesn't exist
- [x] Cleans up archive and extracted files after install
- [x] Script is executable (`chmod +x`)

---

## System-Wide Impact

### Interaction Graph

- `test.yml` gains a new `report` job dependent on all existing jobs. Adding `pull-requests: write` at job level does not affect the workflow-level `contents: read` — no regression.
- `release.yml` rewrite: the `bump-version` job force-pushes to `main` and re-tags. If branch protection requires PR for main, the PAT must be configured to bypass protection rules, OR the bump commit is pushed directly using admin bypass. This is a hard dependency.
- GoReleaser reads `GITHUB_TOKEN` for release creation. The `goreleaser-action@v6` handles this automatically.

### Error & Failure Propagation

| Scenario | Behavior |
|----------|----------|
| `verify` fails (tag version != Cargo.toml) | All downstream jobs skip — no release created. Developer must bump Cargo.toml, merge, re-tag |
| One build target fails | Other targets continue (`fail-fast: false`) — GoReleaser will fail if a target binary is missing |
| `goreleaser` fails | `publish-crates` is skipped |
| `cargo audit` finds vulnerabilities | PR comment posted, job exits 0, PR not blocked |
| PR comment API call fails | `actions/github-script` throws, job fails — but this does not block merge since `report` and `audit` are not required status checks |

### State Lifecycle Risks

- **Tag/Cargo.toml mismatch**: developer tags without bumping Cargo.toml first. Mitigated by the `verify` job failing fast and providing a clear error message with instructions.
- **Deleted and re-pushed tag**: developer deletes a tag and re-pushes it. The `verify` job will pass/fail based on the current Cargo.toml state at checkout — no special handling needed.

---

## Dependencies & Prerequisites

| Dependency | Details |
|------------|---------|
| `CRATESIO_TOKEN` GitHub secret | Already exists per existing `release.yml` |
| `goreleaser-action@v6` | Public GitHub Action, no additional secrets needed |
| `taiki-e/install-action@v2` | Already used for `cross` in existing `release.yml` |
| `actions/github-script@v7` | First-party GitHub Action |
| GoReleaser binary naming | Install script must match `.goreleaser.yml` archive `name_template` exactly |

---

## Files to Create / Modify

| File | Action | Notes |
|------|--------|-------|
| `.github/workflows/test.yml` | Modify | Add `report` job + job-level permissions |
| `.github/workflows/security.yml` | Create | New security audit workflow |
| `.github/workflows/release.yml` | Rewrite | Replace with bump + build matrix + goreleaser + crates jobs |
| `.goreleaser.yml` | Create | GoReleaser config with `skip: true` for builds |
| `install.sh` | Create | Bash install script |

---

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Developer forgets to bump Cargo.toml before tagging | Medium | `verify` job fails fast with a clear message; no damage done |
| GoReleaser `--skip=build` artifact layout wrong | Medium | Test locally with `goreleaser release --skip=build --snapshot` |
| `cargo audit` JSON format changes between versions | Low | Pin `cargo-audit` version in `taiki-e/install-action` |
| `actions/checkout@v6` — currently the repo uses v6 but latest is v4 | Note | `v6` appears to be a typo/future version in the existing files; keep consistent with what's already there |
| Force-push to main rejected by branch protection | Medium | PAT must have bypass rights, or use GitHub environment protection instead |

---

## Sources & References

### Origin

- **Brainstorm document:** [docs/brainstorms/2026-03-25-cicd-redesign-brainstorm.md](../brainstorms/2026-03-25-cicd-redesign-brainstorm.md)

Key decisions carried forward:
- Extend existing workflows, do not replace
- GoReleaser owns full CD pipeline (packaging, checksums, release); GitHub Actions matrix handles Rust compilation
- Security audit is non-blocking reporter (dependabot model)
- Auto-bump: tag is single source of truth for version; workflow bumps Cargo.toml and force-re-tags

### Internal References

- Existing CI: `.github/workflows/test.yml`
- Existing Release: `.github/workflows/release.yml`
- Cargo config (sccache wrapper): `.cargo/config.toml:3`
- Cross-compilation config: `Cross.toml`
- Previous CI/CD plan (superseded): `docs/plans/2026-02-26-feat-cicd-pipeline-plan.md`
- Performance optimization brainstorm: `docs/brainstorms/2026-02-27-cicd-performance-optimization-brainstorm.md`
