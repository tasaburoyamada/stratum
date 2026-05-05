# Stratum Development Workflow

This document defines the formal development process for Stratum.

## 1. Branching Strategy
- **main**: Stable production-ready code. Only merged from `develop` via Pull Request / Release.
- **develop**: Integration branch for features. Base for all feature branches.
- **feature/<issue-id>-<name>**: Individual feature implementation.
- **bugfix/<issue-id>-<name>**: Individual bug fixes.

## 2. Issue Management
- Tasks must be tracked on the local Gitea instance: `http://localhost:3000/kubotad/stratum/issues`.
- Commits should reference issue IDs (e.g., `feat: implementation of PageIndex #5`).

## 3. Pull Request & Review
- All changes to `main` and `develop` must be made via PRs.
- Automatic verification (tests/lints) must pass before merging.

## 4. Release Notes
- `CHANGELOG.md` must be updated with every significant milestone or release.
