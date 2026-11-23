# macOS Packaging and Installation

## Scenario: GitHub Latest DMG Install

### 1. Scope / Trigger

Changing macOS release assets, stable latest download aliases, or user-facing
macOS installation instructions.

### 2. Signatures

- Apple Silicon alias: `EcoPaste-macos-arm64.dmg`
- Intel alias: `EcoPaste-macos-x64.dmg`
- Checksums: `<alias>.sha256`
- Stable base URL:
  `https://github.com/FeatherBirth/EcoPaste/releases/latest/download`

### 3. Contracts

- Both architectures are built on GitHub-hosted macOS runners with
  `aarch64-apple-darwin` and `x86_64-apple-darwin`.
- Release jobs copy the single generated DMG to the stable alias before upload;
  `latest/download` must not contain a version in the asset name.
- The publish job must verify both aliases exist before publishing `latest.json`.
- The updater manifest continues to use the original signed `.app.tar.gz`
  assets; stable DMG aliases are user installation assets only.
- macOS installation copies the `.app` bundle from the mounted DMG to
  `Applications`; the app bundle is not built on Windows.

### 4. Validation & Error Matrix

- Missing or multiple DMGs for one target -> release job fails.
- Missing arm64 or x64 stable alias -> publish job fails.
- DMG alias upload failure -> release job fails.
- Existing alias from a previous run -> upload with `--clobber`.
- Apple Developer ID signing or notarization absent -> keep the documented
  first-launch override guidance; do not claim notarization.

### 5. Good/Base/Bad Cases

- Good: `latest/download/EcoPaste-macos-arm64.dmg` installs on Apple Silicon and
  `latest/download/EcoPaste-macos-x64.dmg` installs on Intel.
- Base: GitHub Release also retains the versioned DMG and updater artifacts.
- Bad: documenting a versioned asset under `latest/download`.
- Bad: creating a DMG on a Windows runner or publishing an empty placeholder.

### 6. Tests Required

- Parse the release workflow YAML.
- Build both targets on a macOS runner and assert each target produces exactly
  one DMG.
- Assert both stable aliases and checksum sidecars are present in the draft
  release before publishing.
- Open each published DMG on a matching Mac and copy EcoPaste into
  `Applications`.
- Verify first launch, standard clipboard capture, and in-app updater behavior.

### 7. Wrong vs Correct

Wrong:

```text
https://github.com/FeatherBirth/EcoPaste/releases/latest/download/EcoPaste_1.6.0_aarch64.dmg
```

Correct:

```text
https://github.com/FeatherBirth/EcoPaste/releases/latest/download/EcoPaste-macos-arm64.dmg
```
