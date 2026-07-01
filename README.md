<p align="center">
  <img src="app-logo.png" alt="Polyflo" width="120" />
</p>

<h1 align="center">Polyflo</h1>

<p align="center">
  Minimal push-to-talk dictation for Windows and macOS.<br />
  Hold a hotkey, speak, release — text appears wherever your cursor is focused.
</p>

---

## What is Polyflo?

Polyflo is a lightweight desktop dictation app built with [Tauri](https://v2.tauri.app/) (Rust backend) and powered by [Sarvam AI Saaras v3](https://docs.sarvam.ai/api-reference-docs/getting-started/models/saaras) speech-to-text. It runs in your system tray and works in any application that accepts typed text.

### Dictation modes

| Mode | What happens |
|------|--------------|
| **Transcribe** | Speech → auto-detect language → paste as spoken |
| **Translate** | Speech → transcribe → translate to English → paste |

Saaras v3 supports [23 languages](https://docs.sarvam.ai/api-reference-docs/getting-started/models) (22 Indian languages + English) with automatic language detection and code-mixed speech handling — a strong fit if you dictate in Hindi, Tamil, Bengali, or mix languages mid-sentence.

---

## Install

### Option A — Download a release (recommended)

1. Go to **[Releases](https://github.com/Crisiswastaken/vox/releases)** on GitHub.
2. Download the installer for your platform:
   - **Windows:** `Polyflo_*_x64-setup.exe` (NSIS installer)
   - **macOS (Apple Silicon):** `Polyflo_*_aarch64.dmg`
   - **macOS (Intel):** `Polyflo_*_x64.dmg`
3. Run the installer (Windows) or open the `.dmg` and drag Polyflo to Applications (macOS).
4. Launch Polyflo — it appears in your **system tray**.
5. Right-click the tray icon → **Open Settings** and enter your [Sarvam API key](https://dashboard.sarvam.ai/).

> **First launch:** If no API key is configured, the Settings window opens automatically.

### Option B — Build from source

**Prerequisites**

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/tools/install) 1.77+
- [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS

**Windows**

- WebView2 (pre-installed on Windows 10/11)
- Microphone access
- Build with **PowerShell or CMD** — not Git Bash (MSVC `link.exe` conflict)

**macOS**

- Xcode Command Line Tools
- **Accessibility** permission for auto-paste (System Settings → Privacy & Security → Accessibility)

```bash
git clone https://github.com/Crisiswastaken/vox.git
cd vox
npm install
```

Optional dev fallback: copy `.env.example` to `.env` and set `SARVAM_API_KEY`. Production builds store the key in the OS credential manager via Settings.

```bash
# Development
npm run tauri:dev

# Production build (PowerShell/CMD on Windows)
npm run tauri:build
```

Installers are written to `src-tauri/target/release/bundle/`.

### Publishing a GitHub release

Tag a version to trigger the release workflow:

```bash
git tag v1.0.0
git push origin v1.0.0
```

GitHub Actions builds Windows and macOS installers and creates a **draft** release you can review and publish.

---

## How to use

1. **Tray icon** — Polyflo lives in the system tray. Right-click for **Open Settings** or **Quit**.
2. **Hotkey** — Hold the default shortcut to dictate:
   - Windows: `Ctrl+Shift+Space`
   - macOS: `Cmd+Shift+Space`
   - Change it anytime in Settings.
3. **Speak** — A small overlay shows listening/processing state.
4. **Release** — Transcript is pasted into the focused field (or copied to clipboard with a notification if paste is unavailable).
5. **History** — Recent transcripts appear in Settings → Clipboard.

### API key

Your Sarvam API key is stored in the OS credential manager (service: `polyflo`, user: `sarvam`). It never leaves your machine except when sent to Sarvam's API for transcription. Never commit real keys to git.

Sarvam offers [free trial credits](https://www.sarvam.ai/voice-to-text) with pay-as-you-go pricing at roughly [₹30/hour of audio](https://docs.sarvam.ai/api-reference-docs/pricing) for speech-to-text — billed per second, rounded up.

---

## Polyflo vs Wispr Flow

[Wispr Flow](https://wisprflow.ai/) is a polished, subscription-based AI dictation product. Polyflo targets a different trade-off: minimal, open, and pay-per-use via your own Sarvam key. Below is an honest comparison based on each product's public documentation and pricing pages as of 2026.

| | **Polyflo** | **Wispr Flow** |
|---|-------------|----------------|
| **Pricing** | Pay-as-you-go via [Sarvam API](https://docs.sarvam.ai/api-reference-docs/pricing) (~₹30/hr STT); [free trial credits](https://www.sarvam.ai/voice-to-text) | [Free tier](https://wisprflow.ai/pricing): 2,000 words/week desktop; Pro [$12–15/mo](https://wisprflow.ai/pricing) unlimited |
| **Account** | Sarvam API key only | Wispr account required |
| **Open source** | Yes — build and audit yourself | Closed source |
| **Platforms** | Windows, macOS (desktop) | [Mac, Windows, iOS, Android](https://wisprflow.ai/) |
| **Languages** | [23 languages](https://docs.sarvam.ai/api-reference-docs/getting-started/models) (22 Indian + English), code-mixing | [100+ languages](https://wisprflow.ai/pricing) |
| **Indian languages** | Core strength — Saaras v3 built for Indic speech | Supported, not Indic-specialized |
| **Word limits** | None (limited by API usage/cost) | [2,000 words/week](https://wisprflow.ai/pricing) on free desktop tier |
| **AI editing** | Raw transcription + optional English translation | [AI auto-edits](https://wisprflow.ai/), filler removal, Command Mode (Pro) |
| **Snippets / dictionary** | Clipboard history only | [Custom dictionary & snippets](https://wisprflow.ai/pricing), synced across devices |
| **Privacy** | Your key → Sarvam only; lightweight local app | Cloud processing; [Privacy Mode on all plans](https://wisprflow.ai/pricing); HIPAA-ready |
| **App size** | Small Tauri binary (~few MB + WebView) | Larger native + cloud stack |
| **Offline** | No (requires Sarvam API) | No (cloud-only) |
| **Best for** | Developers, Indic-language users, cost-conscious heavy dictation, self-hosted control | All-in-one polished UX, mobile, AI formatting, teams/enterprise compliance |

**When Polyflo is the better fit**

- You dictate primarily in **Indian languages** or **code-mixed** speech (Hindi + English, etc.).
- You want **no monthly subscription** and prefer transparent per-minute API billing.
- You dictate **more than ~2,000 words/week** and don't want a $12–15/mo Pro plan.
- You want an **open, auditable** desktop app you can build yourself.
- You need a **minimal tray app** without account signup beyond an API key.

**When Wispr Flow is the better fit**

- You want **AI cleanup** (filler removal, formatting, tone adjustment) and **Command Mode** voice editing.
- You need **iOS/Android** or **cross-device sync** (dictionary, snippets, history).
- You prefer a **turnkey subscription** with no API key management.
- You need **enterprise compliance** (SOC 2 Type II, SSO/SAML) out of the box.

Sources: [Wispr Flow](https://wisprflow.ai/), [Wispr Flow pricing](https://wisprflow.ai/pricing), [Wispr Flow docs](https://docs.wisprflow.ai/articles/2772472373-what-is-flow), [Sarvam Saaras v3](https://docs.sarvam.ai/api-reference-docs/getting-started/models/saaras), [Sarvam pricing](https://docs.sarvam.ai/api-reference-docs/pricing), [Sarvam voice-to-text](https://www.sarvam.ai/voice-to-text).

---

## Architecture

- **Tauri v2** + Rust backend
- **cpal** microphone capture → 16 kHz mono PCM → Sarvam WebSocket STT (`saaras:v3`)
- **Sarvam Translate API** for English mode
- **enigo** paste simulation with clipboard + notification fallback
- Circular always-on-top overlay with live audio waveform

---

## Development scripts

Windows-only helpers for local dev environment setup live in [`scripts/`](scripts/):

- `install-windows-sdk.ps1` — Windows SDK setup
- `setup-msvc.ps1` — MSVC toolchain setup
- `run-dev.ps1` — convenience dev launcher


