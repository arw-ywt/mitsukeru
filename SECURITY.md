# Security Policy

## Supported Versions

Only the latest minor release receives security updates.

| Version | Supported          |
| ------- | ------------------ |
| 0.2.x   | :white_check_mark: |
| < 0.2.0 | :x:                |

## Reporting a Vulnerability

**Please do NOT report security vulnerabilities through public GitHub issues, discussions, or pull requests.**

Instead, please report them privately via GitHub's **Private Vulnerability Reporting**:

1. Navigate to the [**Security Advisories**](https://github.com/arw-ywt/mitsukeru/security/advisories) page of this repository.
2. Click **"Report a vulnerability"** to open a private advisory draft.

### What to Include

To help us triage and reproduce the issue quickly, please include:

- A clear description of the vulnerability and its potential impact.
- Affected crate/package (`mitsukeru`, `mitsukeru-wasm`, or npm `mitsukeru`) and version.
- Minimal reproduction steps, payload JSON, or a Rust test case.
- Any relevant build target or feature flags (e.g., `no_std`, `--release`, `wasm32-unknown-unknown`).

### Response & Disclosure Process

- We aim to acknowledge new reports within **3 business days**.
- Once confirmed, we will collaborate on a fix in a temporary private fork and prepare a patched release before publishing a security advisory.
- We kindly ask that you follow coordinated disclosure practices and allow reasonable time for a fix to be released before disclosing details publicly.