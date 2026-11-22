# DocuFlow — Product Requirements & Technical Foundation

**Version:** 3.0
**Date:** 20 November 2026
**Platforms:** Windows (x64) and macOS (Apple Silicon + Intel)
**Status:** Draft for review — consolidates *MVP Product Requirements & Tech Stack v2* and *Final Technology Stack Recommendation*, with revised processing libraries, performance guidelines and licensing rules.

---

## 1. Summary

DocuFlow is a privacy-first desktop utility for common image and PDF tasks, especially when websites and portals impose strict file-size or dimension limits.

> **Your files. Your device. Your privacy.**

**Final stack:** **Tauri 2 + React + TypeScript + Vite + Rust**, with a processing layer built from permissively licensed libraries:

- **Images:** `image` + `fast_image_resize` + `mozjpeg` + `webp` + `oxipng`
- **PDF:** `lopdf` (merge, split, reorder and remove pages, compress, image-to-PDF) + PDFium via `pdfium-render` (previews)
- **Local history:** SQLite via `rusqlite`

**What changed from v2:**

| Area | v2 | v3 | Reason |
|---|---|---|---|
| Image processing | libvips via Rust bindings | Pure-Rust / statically linked crates | libvips is LGPL, its Rust bindings are thinly maintained, and it ships many DLLs on Windows |
| PDF merge/split | qpdf | `lopdf` | No C++ library to build on two platforms; pure Rust, MIT |
| PDF compression | qpdf (structural only) | `lopdf` + custom image re-encoding | qpdf can't downsample images, so Medium/High presets had no engine |
| Image-to-PDF | Not specified | `lopdf` | Embeds JPEGs as-is, without re-encoding |
| PDF page reordering and removal | Pending confirmation | **In MVP** — one "Organize pages" tool using `lopdf` + PDFium thumbnails + dnd-kit | Fully covered by the existing stack; no new dependencies |
| E2E testing | Playwright | Playwright (mocked IPC) + `tauri-driver` (Windows) + macOS manual smoke test | No WebDriver support for WKWebView on macOS |
| Licensing | Reviewed manually | Allowlist enforced in CI (`cargo-deny`, `license-checker`) | Stops copyleft dependencies at build time |

---

## 2. Product principles

- Installable desktop app for Windows and macOS.
- Core image and PDF operations work **offline** after installation.
- Files are processed **locally**; normal workflows never upload documents.
- No account or cloud workspace in the MVP.
- **No telemetry** by default; no analytics SDKs.
- **No automatic update checks or auto-update** in the MVP.
- Small footprint, fast startup and low memory use are explicit engineering goals.
- Originals are **never overwritten** automatically.

---

## 3. MVP scope

### 3.1 Image Toolkit

| Feature | MVP behaviour |
|---|---|
| Compress image to target size | User enters a maximum size (e.g. under 2 MB); DocuFlow finds the highest acceptable quality that fits, where feasible. Shows before/after preview and actual sizes. |
| Resize by exact pixels | Target width and height, with a preserve-aspect-ratio option. |
| Resize by percentage | Shrink or enlarge by a percentage. |
| Image-to-PDF | Convert one or more JPG/PNG images into one PDF, one image per page by default. |
| Before/after preview | Original and output preview, dimensions, format and file size. |

**Critical UX rule:** dimensions and file size are separate settings. Resizing changes pixel dimensions; compression changes encoding quality and file size. The interface must explain this.

### 3.2 PDF Toolkit

| Feature | MVP behaviour |
|---|---|
| Merge PDFs | Combine multiple PDFs in a chosen order. |
| Split PDFs | Export individual pages or user-defined page ranges. |
| Organize PDF pages | Open one PDF, reorder pages by dragging thumbnails and/or remove unwanted pages, then save as a new PDF. |
| Compress PDFs | Low, Medium and High presets. No "compress to exactly X MB" in the MVP. |
| PDF preview | Render page thumbnails where required. |

### 3.3 Local recent history

- Stores job details and output locations only.
- Never stores thumbnails, document contents or copies of files.
- Visible **Clear history** action, which removes job records only and never deletes documents.
- Files that have been moved or deleted show as unavailable.
- Suggested fields: operation type, date/time, source filenames, output filenames and paths, status, result sizes.

### 3.4 Visible proof of privacy

- A short "How your files stay local" screen.
- States that processing is on-device and nothing is uploaded.
- Documents any installer-time network activity (the WebView2 bootstrapper on Windows).
- Uses narrow Tauri capabilities (see §9).

---

## 4. Functional requirements

### 4.1 Target-size image compression

**User story:** As a user, I want to enter a maximum file size and have DocuFlow produce the best-looking image that fits, without uploading my photo.

1. Import through the file picker or drag-and-drop.
2. Show the original preview, dimensions, format and size.
3. Accept a maximum output size.
4. Let the user choose or confirm the output format (JPEG / WebP / PNG).
5. Search encoder settings for the highest quality at or below the target (algorithm in §7.2).
6. Show the before/after preview and actual output size.
7. Let the user choose the output path; never overwrite the original by default.
8. If the target can't be reached above the quality floor (JPEG/WebP quality 40, §6.4), explain why and offer alternatives (resize, change format, accept a larger file).
9. Work fully offline.

PNG is lossless. Reaching a small target may need palette reduction, conversion to a lossy format or resizing. Never silently go below the agreed quality floor.

### 4.2 Image resizing

- Exact pixels or a percentage, with a preserve-aspect-ratio option.
- Show the original and resulting dimensions and a preview before saving.
- Respect EXIF orientation.
- Strip metadata by default, including GPS location.

### 4.3 Image-to-PDF

- Import one or more JPG/PNG images and reorder them (drag-and-drop).
- One image per page; each page is sized to match its image (no A4/Letter, margin or orientation controls in the MVP, §6.4).
- JPEGs are embedded **without re-encoding** (no quality loss). PNGs are embedded with Flate compression.
- Choose the output name and destination; source images are untouched.

### 4.4 Merge PDFs

- Select two or more PDFs; show filename and page count.
- Reorder inputs; merge in the order shown.
- Save as a new file.
- Show clear errors for invalid or unsupported files. Password-protected PDFs are rejected in the MVP (§6.4).

### 4.5 Split PDFs

- One PDF per page, or custom page ranges.
- Validate ranges and explain invalid or overlapping input.
- Summarize the outputs before processing.
- Choose the output folder and a filename pattern.

### 4.6 PDF compression presets

| Preset | Intent | Technique |
|---|---|---|
| Low | Keep fidelity | Structural only: Flate-compress streams, remove unused objects. No image changes. |
| Medium | Balanced | Structural + downsample images above ~150 DPI and re-encode them as JPEG (quality ~75). |
| High | Smallest size | Structural + downsample images above ~100 DPI and re-encode them as JPEG (quality ~60), with a quality warning. |

DPI and quality values are starting points to tune during the proof of concept.

- Show the original size, the result size and the percentage change.
- **If the output isn't smaller, say so and keep the original.** Don't write a larger file.
- Images using JBIG2, CCITT or JPEG2000 encoding are left unchanged.
- Always save to a new file.

### 4.7 Organize PDF pages (reorder and remove)

**User stories:**
- As a user, I want to open a PDF and rearrange its pages, so the document is in the right order without re-scanning or uploading it to a website.
- As a user, I want to remove unwanted pages (blank scans, cover sheets, pages with private information), so I only share what's needed.

**Reordering:**

1. Select one PDF through the file picker or drag-and-drop.
2. Show a grid of page thumbnails with page numbers (rendered lazily by PDFium; only visible pages are rendered).
3. Reorder pages by **drag-and-drop** (dnd-kit), including:
   - moving several selected pages at once (Shift/Ctrl/Cmd-click to select)
   - **Move to start** / **Move to end** actions for the selected pages
   - **keyboard reordering** for accessibility (dnd-kit keyboard sensor)
4. Show each page's original page number on its thumbnail, so the user can see what moved.
5. Provide **Reset to original order** and **Undo/Redo** (state is a simple array of page indices on the frontend).
6. Choose the output filename and destination; save as a **new** PDF. The original is never modified.
7. Show clear errors for invalid, malformed or password-protected files.
8. Large documents (200+ pages) must stay responsive: virtualized grid (TanStack Virtual), thumbnails at display resolution.

**Removal:**

9. Remove pages with a **Remove** button on each thumbnail, or select several pages and press **Delete/Backspace** or the toolbar **Remove selected** action.
10. Removed pages disappear from the grid immediately; **Undo** (and **Reset**) restores them. Nothing is written until the user saves.
11. At least one page must remain; disable removal of the last page and explain why.
12. Before saving, show a summary such as **"12 pages → 9 pages (3 removed)"**.
13. **Removed pages are fully gone from the output file.** Their content, images and fonts are deleted, not just hidden, so removing a page with private information is safe (see §8.1).
14. If a removed page contained form fields, bookmarks or links, tell the user that those entries were removed as well.

**Out of scope for this feature (MVP):** rotating or inserting pages, and extracting removed pages to a separate file (Split already covers extraction). The same UI and engine can support them later.

---

## 5. Technology stack

### 5.1 Final stack and licenses

All licenses below are permissive: MIT, Apache-2.0, BSD, Zlib, IJG or public domain.

| Layer | Library | License | Purpose |
|---|---|---|---|
| Desktop shell | **Tauri 2** + `tauri-plugin-dialog`, `tauri-plugin-fs` | MIT / Apache-2.0 | Native window, system WebView, IPC, scoped file access |
| UI framework | **React** | MIT | Components |
| Language | **TypeScript** | Apache-2.0 | Type safety |
| Build | **Vite** | MIT | Dev server and bundling |
| UI components | **Radix UI** or **shadcn/ui**, **Tailwind CSS** | MIT | Accessible primitives and styling |
| Drag-to-reorder | **dnd-kit** | MIT | Ordering images, PDFs and PDF pages (Organize pages grid) |
| Virtual lists | **TanStack Virtual** | MIT | Long page and thumbnail lists |
| State | **Zustand** | MIT | Lightweight app state |
| Backend language | **Rust** | MIT / Apache-2.0 | Native processing |
| Concurrency | **rayon**, **tokio** (bundled with Tauri) | MIT / Apache-2.0 | Parallel CPU work, async tasks |
| Image decode/encode | **`image`** | MIT / Apache-2.0 | Core image I/O (uses `zune-jpeg` for JPEG decoding) |
| Resize | **`fast_image_resize`** | MIT / Apache-2.0 | SIMD resizing (AVX2/SSE4.1 on x64, NEON on ARM) |
| JPEG encode | **`mozjpeg`** | IJG / BSD-3-Clause / Zlib | Smaller JPEGs at the same visual quality |
| WebP encode | **`webp`** (libwebp) | MIT/Apache-2.0 + BSD-3-Clause | Lossy and lossless WebP |
| PNG optimize | **`oxipng`** | MIT | Lossless PNG size reduction |
| PNG palette reduction | **`color_quant`** or **`exoquant`** | MIT | Lossy PNG reduction without GPL code |
| EXIF | **`kamadak-exif`** | BSD-2-Clause | Read orientation |
| PDF manipulation | **`lopdf`** | MIT | Merge, split, reorder/remove pages, compress, image-to-PDF |
| PDF rendering | **`pdfium-render`** + PDFium binaries from `bblanchon/pdfium-binaries` | MIT/Apache-2.0 + BSD-3-Clause/Apache-2.0 | Thumbnails and previews |
| Local history | **`rusqlite`** (`bundled` feature) | MIT + SQLite public domain | Job metadata |
| Serialization | **`serde`**, **`serde_json`** | MIT / Apache-2.0 | IPC payloads |
| IPC type generation | **`tauri-specta`** + **`specta`** | MIT | Generates TypeScript bindings from Rust commands (AD-2) |
| Errors | **`thiserror`** | MIT / Apache-2.0 | Typed errors with stable codes (AD-3) |
| History migrations | **`rusqlite_migration`** | Apache-2.0 | Versioned SQLite schema (AD-8). Use `rusqlite` ≥ 0.41: older bundled SQLite is affected by RUSTSEC-2026-0459 |
| Logging | **`tracing`** + **`tracing-subscriber`** + **`tracing-appender`** | MIT | Local, redacted, rotating log file (AD-9) |
| UI text | **`i18next`** + **`react-i18next`** | MIT | All UI strings in one place (AD-10) |
| Frontend tests | **Vitest** + **React Testing Library** | MIT | Unit and component tests |
| Backend tests | **`cargo test`** | — | Unit and integration tests |
| E2E tests | **Playwright**, **`tauri-driver`** + **WebdriverIO** | Apache-2.0, MIT/Apache-2.0, MIT | UI flows and full-app tests |
| CI/CD | **GitHub Actions** | Free for public repos | Windows and macOS builds |
| License checks | **`cargo-deny`**, **`cargo-about`**, **`license-checker`** | MIT / Apache-2.0, MIT / Apache-2.0, BSD-3-Clause | License enforcement and notices |

**Alternative for previews:** **PDF.js** (Apache-2.0) renders inside the WebView and removes PDFium completely, so no native PDF library ships. PDFium is faster on large documents, so it's the default; switch only if packaging PDFium becomes a problem.

### 5.2 Libraries to avoid

| Library | License | Problem |
|---|---|---|
| libvips | LGPL-2.1 | Must be dynamically linked, plus compliance work and large DLLs on Windows |
| libimagequant / `imagequant` | GPL-3.0 or commercial | Would force DocuFlow to be GPL |
| MuPDF | AGPL-3.0 or commercial | Copyleft that also covers network use |
| Ghostscript | AGPL-3.0 or commercial | Same |
| libheif / x265 | LGPL / GPL, plus HEVC patents | HEIC stays out of the MVP |
| `sharp` (Node) | Apache-2.0, but needs Node + libvips | Would need a Node runtime in Tauri and erode the footprint advantage |

### 5.3 Why Tauri over the alternatives

| Criterion | Tauri 2 + Rust | Electron + Node | Avalonia + C# |
|---|---|---|---|
| Runtime | System WebView (WebView2 / WKWebView) | Bundled Chromium + Node | Native .NET backend |
| Installer footprint | Smallest potential | Larger | Moderate |
| Memory baseline | Lower | Higher | Benchmark required |
| React/TS reuse | Yes | Yes | No |
| Permission model | Scoped capabilities | Manual hardening | Manual |
| Learning curve | Rust | Lowest | C# + XAML |

**Ranking:** (1) Tauri, best overall fit; (2) Electron, the fallback if Rust becomes a major obstacle; (3) Avalonia, only for a C#-centric team.

The shell choice is final. The processing libraries are locked in after the proof of concept (§14).

---

## 6. Architecture

```text
┌──────────────────────────────────────────────────┐
│                Presentation layer                │
│     React · TypeScript · Vite · Tailwind/Radix   │
│  Image tools · PDF tools · History · Privacy     │
└───────────────┬───────────────────▲──────────────┘
                │ invoke()          │ Channel<Progress>
                │                   │ asset:// (previews)
┌───────────────▼───────────────────┴──────────────┐
│               Application layer (Rust)           │
│  Commands · Validation · Job queue · Cancellation│
│  Temp-file management · Error mapping            │
└───────┬──────────────────┬──────────────┬────────┘
        │                  │              │
┌───────▼───────┐  ┌───────▼───────┐  ┌───▼──────────┐
│ Image engine  │  │  PDF engine   │  │ PDF renderer │
│ image         │  │  lopdf        │  │ pdfium-render│
│ fast_image_   │  │  Merge        │  │ (single      │
│   resize      │  │  Split        │  │  dedicated   │
│ mozjpeg/webp  │  │  Compress     │  │  thread)     │
│ oxipng        │  │  Image→PDF    │  │  Thumbnails  │
│               │  │  Organize     │  │              │
└───────────────┘  └───────────────┘  └──────────────┘
┌──────────────────────────────────────────────────┐
│        Local storage — rusqlite (SQLite)         │
│        Job metadata · Output locations           │
└──────────────────────────────────────────────────┘
```

### 6.1 UI independence

React calls simple commands and never knows which library does the work:

```typescript
const result = await compressImage({
  inputPath: "/path/photo.jpg",
  targetSizeBytes: 2 * 1024 * 1024,
  format: "jpeg",
  onProgress: (p) => setProgress(p.percent),
});
```

Each engine sits behind a Rust trait, so a library can be swapped (for example, PDFium for PDF.js, or `lopdf` for qpdf) without UI changes.

### 6.2 Project structure

The repository is a **Cargo workspace** (decision AD-1). All processing lives in `docuflow-core`, which has no Tauri dependency.

```text
docuflow/
├── Cargo.toml                    # Cargo workspace root
├── crates/
│   └── docuflow-core/            # Processing library (no Tauri dependency)
│       ├── src/
│       │   ├── images/           # Decode, resize, encode, target-size search
│       │   ├── pdf/              # Merge, split, organize, compress, image-to-PDF
│       │   ├── render/           # PDFium wrapper (single-thread use)
│       │   ├── job.rs            # Progress callback + cancel token types
│       │   ├── limits.rs         # Input limits (AD-7)
│       │   └── error.rs          # CoreError enum with stable codes (AD-3)
│       ├── benches/              # Benchmarks run without the app
│       └── tests/                # Integration tests
├── src/                          # React frontend
│   ├── components/               # Shared UI
│   ├── features/
│   │   ├── image-tools/
│   │   ├── pdf-tools/
│   │   ├── history/
│   │   └── privacy/
│   ├── bindings/                 # Generated by tauri-specta (AD-2); do not edit
│   ├── services/                 # Thin wrappers over the generated bindings
│   ├── i18n/                     # All UI text (AD-10)
│   └── lib/
├── src-tauri/                    # Thin app layer
│   ├── capabilities/             # Tauri permission files
│   ├── src/
│   │   ├── commands/             # IPC commands calling docuflow-core
│   │   ├── jobs/                 # Job queue, progress channels, cancellation (AD-4)
│   │   ├── access/               # Allowed-path registry (AD-6)
│   │   ├── output/               # Temp-then-rename writer, name conflicts (AD-5)
│   │   ├── storage/              # SQLite history + migrations (AD-8)
│   │   └── logging/              # tracing setup with path redaction (AD-9)
│   ├── resources/pdfium/         # Per-platform PDFium binaries
│   ├── Cargo.toml
│   └── tauri.conf.json
├── tests/
│   ├── fixtures/                 # Shared test image/PDF corpus
│   └── e2e/
├── deny.toml                     # cargo-deny license policy
└── .github/workflows/
```

### 6.3 Architecture decisions

These decisions are **accepted** for implementation. Revisit one only if the proof of concept shows a concrete problem with it.

| ID | Decision | Details | Why |
|---|---|---|---|
| **AD-1** | Cargo workspace with a separate core crate | `docuflow-core` holds all image/PDF processing and rendering and has no Tauri dependency. `src-tauri` is a thin layer of commands, jobs, access control, storage and logging. | The core can be tested and benchmarked without the app and reused for future mobile or CLI targets. Splitting it later would be a large refactor. |
| **AD-2** | Generated IPC types | Use `tauri-specta` to generate TypeScript bindings from Rust command signatures and types into `src/bindings/`. CI fails if the generated file is out of date. | React and Rust types can't drift apart. `tauri-specta` for Tauri 2 is still a release candidate: **pin an exact version**. |
| **AD-3** | One error model with stable codes | `docuflow-core` defines a `CoreError` enum (`thiserror`). Each variant serializes as `{ code, details }` with a stable `code` (e.g. `PdfEncrypted`, `PdfMalformed`, `TargetUnreachable`, `FileTooLarge`, `ImageTooLarge`, `UnsupportedFormat`, `PathNotAllowed`, `Cancelled`, `Io`). React maps each code to a translated, user-facing message. | Every feature depends on it; changing it later touches everything. Stable codes also keep error messages free of file contents. |
| **AD-4** | Job model | Every operation is a job: `start*` returns a `jobId`; progress streams over a Tauri `Channel` (`{ jobId, percent, stage }`); `cancelJob(jobId)` sets a cancel token that `docuflow-core` checks between steps. **One heavy job at a time**; others queue (FIFO). Thumbnail rendering runs separately on the PDFium thread. | Progress, cancellation and future batch processing all build on this. One job at a time keeps peak memory predictable. |
| **AD-5** | Safe output writing | Write to a temp file **in the destination folder** (`.docuflow-<random>.tmp`), flush and sync it, then rename it to the final name. On failure or cancellation, delete the temp file. If the name exists, add ` (1)`, ` (2)`, …, so nothing is ever overwritten. Default names: `<name>-compressed`, `<name>-resized`, `<name>-merged`, `<name>-organized`, `<name>-page-<n>`. | A crash or cancel never leaves a half-written file, and originals are never overwritten. A temp file in the same folder makes the rename reliable on both OSes. |
| **AD-6** | File access security | The frontend gets **no** `fs` or `dialog` permission (`core:default` only). File and folder pickers are opened **from Rust** (`pick_input_files`, `pick_output_folder`), and dropped files are captured by a Rust window event, so every path enters through Rust. Rust keeps an **allowed-path registry** for the session: files the user picked through the dialog or dropped onto the window, and folders chosen as output destinations. Every command checks its paths against the registry (after resolving them to canonical absolute paths) and returns `PathNotAllowed` otherwise. The asset protocol scope is limited to the app's temp/preview folder. | Tauri capabilities don't restrict the app's own Rust code. The registry is what actually limits DocuFlow to user-selected files. |
| **AD-7** | Input limits | Defaults (tune in the proof of concept): images up to **100 MP** and **200 MB**; PDFs up to **500 MB** and **2,000 pages**. Enforced in `docuflow-core/limits.rs` and through `image`'s `Limits` before decoding. Exceeding a limit returns `ImageTooLarge`/`FileTooLarge` with a clear message. | Protects against crafted "decompression bomb" files and out-of-memory crashes. |
| **AD-8** | Versioned history schema | `rusqlite_migration` with numbered migrations from day one. Schema v1: `jobs(id, operation, created_at, status, source_names, output_paths, input_bytes, output_bytes)`. **Clear history** runs `DELETE FROM jobs` followed by `VACUUM`, so cleared records are not left in the database file. | Adding versioning after users already have a database is messy. `VACUUM` keeps the privacy promise of clearing. |
| **AD-9** | Privacy-safe logging | `tracing` + `tracing-appender` writes to a local log file in the app's log folder, rotated daily with the last 7 files kept (`tracing-appender` has no size-based rotation, so there is no hard 5 MB cap; the default level is `info`, overridable with `DOCUFLOW_LOG`). A redaction layer replaces file paths and names with placeholders (`<file:1>`); no document content is ever logged. Logs never leave the device; there's an "Open log folder" action for support. | The privacy promise covers logs too, so the rule exists before anyone writes a log line. |
| **AD-10** | Text in one place | English only for the MVP, with all UI text in `src/i18n/` using `i18next` + `react-i18next`. No hard-coded UI strings in components. | Adding languages later means adding a file, not editing every component. |

### 6.4 Product decisions resolved for implementation

| Topic | Decision |
|---|---|
| Quality floor for target-size compression | JPEG/WebP quality **40**. Below that, stop and offer: resize, switch format, or accept a larger file. |
| Image-to-PDF page sizing | Each page is **sized to match its image** (fit-to-page). No A4/Letter, margin or orientation controls in the MVP. |
| Password-protected PDFs | **Rejected in the MVP** with a clear `PdfEncrypted` message, in all PDF tools. Exception: PDFs that open without a password (owner-password only) are processed if `lopdf` can load them with an empty password; otherwise they're rejected too. |
| Minimum OS versions | **Windows 10 (version 1809+) and Windows 11, x64**; **macOS 11 Big Sur+**, Apple Silicon and Intel. Set `bundle.macOS.minimumSystemVersion = "11.0"` in `tauri.conf.json`. |
| File paths in history | **Stored locally**, needed to open output locations and detect moved or deleted files. Never logged or sent anywhere, and removed by **Clear history**. The privacy screen explains this. |

---

## 7. Image pipeline

### 7.1 Processing flow

1. Decode with `image`; read orientation with `kamadak-exif` and apply it.
2. Convert to an 8-bit RGB/RGBA buffer.
3. Resize with `fast_image_resize` (Lanczos3 for downscaling) if requested.
4. Encode:
   - JPEG with `mozjpeg` (progressive, optimized Huffman tables)
   - WebP with `webp`
   - PNG with `image`, then `oxipng`; palette reduction with `color_quant`/`exoquant` when a lossy PNG is allowed
5. Write to the user-chosen path. Metadata is dropped, because re-encoding doesn't copy EXIF/GPS.

### 7.2 Target-size algorithm

1. Normalize orientation; keep dimensions fixed unless the user asked for a resize.
2. **For large images (over ~8 MP):** estimate quality on a downscaled copy to narrow the search range.
3. **Binary search** quality in the range `[quality_floor, 95]`, encoding **in memory** (about 6–7 encodes; no temp files).
4. Optionally encode candidates in parallel with `rayon`.
5. Pick the highest quality whose size is ≤ the target.
6. If even `quality_floor` is too large, stop and offer: resize, switch to WebP, or accept a larger file.
7. Show the actual size and a preview.

Don't treat PNG "quality" as equivalent to JPEG quality. A PNG target uses lossless optimization first, then palette reduction (if allowed), then suggests a format change.

---

## 8. PDF pipeline

| Operation | Implementation |
|---|---|
| Merge | `lopdf`: load each document, renumber objects, copy page trees in order, rebuild the catalog |
| Split | `lopdf`: copy selected pages and the objects they reference into new documents |
| Organize pages (reorder + remove) | `lopdf`: rewrite the page tree with the kept pages in the new order, then delete everything only removed pages used (details in §8.1) |
| Compress — structural | `lopdf`: remove unused objects, Flate-compress uncompressed streams, use object streams |
| Compress — images | Find image XObjects (DCTDecode/FlateDecode), decode, downsample with `fast_image_resize`, re-encode with `mozjpeg`, replace the stream **only if the new one is smaller** |
| Image-to-PDF | `lopdf`: JPEG as a DCTDecode stream (no re-encode); PNG as Flate with SMask for alpha; page sized to fit the image |
| Preview | `pdfium-render`: render visible pages only, at display resolution, on a dedicated thread |

**Notes:**
- `lopdf` loads whole documents into memory. Benchmark "merge 100 PDFs" and large scanned PDFs during the proof of concept.
- PDFium **isn't thread-safe**. All PDFium calls go through one worker thread with a job queue.
- Load PDFium dynamically at runtime, so the app starts quickly and only pays the cost when PDF features are used.
- Encrypted PDFs: rejected with `PdfEncrypted`, unless they open with an empty password (§6.4).

### 8.1 Organize pages (reorder and remove)

Reordering and removal share **one command**. The frontend sends the list of pages to keep, in the order to keep them; any page not in the list is removed.

**Frontend → backend contract:**

```typescript
await organizePdfPages({
  inputPath: "/path/report.pdf",
  outputPath: "/path/report-organized.pdf",
  pages: [2, 0, 3, 4], // zero-based original indices, in the new order; page 1 is removed
});
```

**Backend steps (`lopdf`):**

1. Load the document and get the page object IDs in their original order (`get_pages()`).
2. **Validate** `pages`: not empty, every index within range, **no duplicates**. Reject anything else.
3. **Push inherited attributes down** (`Resources`, `MediaBox`, `CropBox`, `Rotate`) from intermediate `Pages` nodes onto each kept page. Many PDFs store these on parent nodes, and flattening the tree without this step breaks page size or fonts.
4. Rebuild the root `Pages` node as a flat list: `Kids` = kept page IDs in the new order, `Count` = number of kept pages, and set each page's `Parent` to the root.
5. **If any pages were removed, clean up references to them:**
   - remove bookmark (outline) entries and named destinations that point to removed pages, and fix the outline `Count`/`First`/`Last`/`Prev`/`Next` links
   - remove link annotations on kept pages that jump to removed pages
   - remove form fields (`/AcroForm /Fields`) whose widgets are on removed pages
   - remove structure-tree (tagged PDF) entries for removed pages, or drop `/StructTreeRoot` for the MVP if cleanup is too complex
6. **Delete unreferenced objects** (`prune_objects()`), so content streams, images and fonts used only by removed pages are physically gone. This also removes the old intermediate `Pages` nodes.
7. Compress and save to the new path. **Write a fresh file, not an incremental update**, because incremental saves keep the old objects inside the file.

**What is preserved:**
- Kept pages' content, fonts, images and annotations are untouched, so there is **no quality loss** and it's fast.
- Bookmarks and internal links point to page objects rather than page numbers, so links to kept pages still go to the right page after reordering.

**Privacy check for removal:** add a test that removes a page containing a unique marker string or image, then confirms that marker no longer appears anywhere in the output bytes.

**Known caveat:** `/PageLabels` (custom page numbering such as "i, ii, iii, 1, 2…") is based on position and may no longer match. For the MVP, remove `/PageLabels` from organized output so viewers show plain 1…N numbering, and note this in the UI help text.

---

## 9. Performance guidelines

### 9.1 Backend (Rust)

- **Never block the main thread.** Run each job with `tauri::async_runtime::spawn_blocking`; use `rayon` for parallel CPU work.
- **Don't send file bytes over IPC.** Write outputs and previews to a temp folder and load them in React with `convertFileSrc` (asset protocol).
- **Stream progress** with Tauri `Channel`s; support **cancellation** through a shared cancel flag checked between steps.
- **Thumbnails at display size.** Never decode or render at full resolution just for a preview.
- **Avoid repeated decode/encode cycles.** Decode once and reuse the buffer across target-size candidates.
- **Limit memory:** process one large image at a time; queue batch jobs rather than running them all at once.
- **Clean up temp files** after success, failure or cancellation.

### 9.2 Frontend (React)

- Lazy-load feature screens with `React.lazy` for faster startup.
- Virtualize long thumbnail and page lists with TanStack Virtual.
- Keep heavy state out of React; store only references (paths, IDs) and display metadata.
- Test in **both** WebView2 and WKWebView, because CSS, fonts and drag-and-drop differ slightly.

### 9.3 Release build profile

`src-tauri/Cargo.toml`:

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

### 9.4 Platform builds

| Platform | Target | Notes |
|---|---|---|
| macOS | `universal-apple-darwin` (Apple Silicon + Intel) | `tauri build --target universal-apple-darwin`; needs both Rust targets installed |
| Windows | `x86_64-pc-windows-msvc` | ARM64 optional later |
| Windows WebView2 | `downloadBootstrapper` (default) | Smallest installer; needs internet if WebView2 is missing. Offer an `offlineInstaller` build (~+127 MB) as a separate download if needed |
| CI | Install **NASM** on Windows and macOS runners | Without it, `mozjpeg` builds without SIMD and runs slower |

### 9.5 Metrics to measure on both platforms

| Metric | Measurement |
|---|---|
| Cold startup | Launch to usable screen |
| Idle memory | Settled resident memory |
| Peak memory | Large image/PDF operations |
| Installer size | Downloadable package |
| Installed size | On-disk footprint |
| Processing time | Fixed test corpus and machine specs |
| UI responsiveness | Navigation, scrolling and cancel during processing |

Don't promise an installer size (e.g. "under 10 MB") until release artifacts, including PDFium, have been measured.

---

## 10. Privacy and security

1. No account or cloud workspace.
2. No document uploads in normal workflows.
3. No telemetry, analytics or automatic update checks in the MVP.
4. **Narrow Tauri capabilities:** the frontend has `core:default` only; **no `fs`, `dialog`**, shell or HTTP permissions (pickers run in Rust). The asset protocol is limited to the app's temp/preview folder (AD-6).
5. Access only user-selected files and user-chosen output locations, enforced by the Rust allowed-path registry (AD-6).
6. Store minimal history metadata; clearing history never deletes files and runs `VACUUM` (AD-8).
7. Never put document contents, thumbnails, filenames or paths in logs; logs are redacted and stay on the device (AD-9).8. Strip GPS and other metadata from exported images by default.
9. Clean up temp files on every exit path.
10. Fuzz and test malformed, truncated and unusually large files; a bad file must produce an error, never a crash.
11. Document installer-time network activity (WebView2 bootstrapper).
12. Enforce input size limits before decoding (AD-7).
13. Write outputs with temp-then-rename; never overwrite existing files (AD-5).

---

## 11. Testing strategy

| Level | Tooling | Scope |
|---|---|---|
| Rust unit | `cargo test` | Target-size search, range parsing, preset logic, organize-pages validation (empty, out-of-range, duplicates), allowed-path checks, output name conflicts, log redaction, input limits |
| Rust integration | `cargo test` + `tests/fixtures/` corpus | Real images and PDFs: text, vector, scanned, encrypted, malformed; removed-page content is absent from the output bytes |
| Frontend unit/component | Vitest + React Testing Library | Forms, validation, state |
| UI flows | Playwright against the Vite dev server, with Tauri IPC mocked via `@tauri-apps/api/mocks` | Full user journeys on both OSes in CI |
| Full-app E2E | `tauri-driver` + WebdriverIO (**Windows only**) | Real app with real IPC |
| macOS full-app | Manual smoke checklist per release | WKWebView has no WebDriver support |
| Performance | Benchmark scripts on fixed hardware | Metrics in §9.5 |

---

## 12. Licensing compliance

Permissive licenses still require **including their notices**. Enforce this automatically.

### 12.1 `cargo-deny` policy (`deny.toml`)

```toml
[licenses]
version = 2
allow = [
  "MIT",
  "Apache-2.0",
  "Apache-2.0 WITH LLVM-exception",
  "BSD-2-Clause",
  "BSD-3-Clause",
  "ISC",
  "Zlib",
  "IJG",
  "Unicode-3.0",
  "BSL-1.0",
  "CC0-1.0",
  "MPL-2.0",
]
confidence-threshold = 0.9

[bans]
multiple-versions = "warn"
```

`MPL-2.0` is allowed because some of Tauri's transitive dependencies use it. It is file-level copyleft and fine when those files are unmodified. Any GPL, LGPL or AGPL dependency fails the build.

### 12.2 Notices

- **Rust:** `cargo about generate` produces the third-party notices file.
- **npm:** `npx license-checker --production --onlyAllow "MIT;Apache-2.0;BSD-2-Clause;BSD-3-Clause;ISC;0BSD"` in CI.
- **PDFium:** ship its notices plus those for its bundled components: FreeType (under its FTL option), libjpeg-turbo, zlib, OpenJPEG and lcms2.
- **mozjpeg / libwebp:** include IJG, BSD-3 and Zlib notices (covered by `cargo-about`).
- Show the notices in an **About → Open-source licenses** screen and include them in the installers.

This is technical planning, not legal advice. Review exact versions before a public release.

---

## 13. CI/CD and distribution

**GitHub Actions matrix:** `windows-latest` (x64) and `macos-latest` (universal).

Pipeline steps:
1. Install Rust stable + targets (`aarch64-apple-darwin`, `x86_64-apple-darwin` on macOS), Node LTS, NASM.
2. Lint and format: `cargo fmt --check`, `cargo clippy -D warnings`, `eslint`, `tsc --noEmit`; regenerate `tauri-specta` bindings and fail if `src/bindings/` changed (AD-2).
3. License gate: `cargo deny check licenses` and `license-checker`.
4. Tests: `cargo test`, `vitest run`, Playwright (mocked IPC); `tauri-driver` E2E on Windows.
5. Build: `tauri build` and record installer and installed sizes as artifacts.
6. Dependency security: `cargo deny check advisories`, `npm audit`, Dependabot.

**Distribution:**
- Windows: NSIS or MSI installer; sign with a code-signing certificate before public release.
- macOS: `.dmg`; sign and **notarize** (Apple Developer Program, $99/yr).
- Include the privacy explanation and third-party notices in both.
- **No auto-update or update checks in the MVP.** The architecture allows adding `tauri-plugin-updater` later.

---

## 14. Proof of concept (before full implementation)

### A. Desktop shell
- Tauri + React + TypeScript app building on Windows and macOS (universal).
- Measure startup, idle memory, installer size and installed size.
- Test the file picker, drag-and-drop, scoped writes and offline launch.

### B. Image pipeline
- Resize JPEG/PNG by pixels and percentage with `fast_image_resize`.
- Target-size compression for JPEG (`mozjpeg`) and WebP with binary search.
- EXIF orientation and metadata stripping.

### C. PDF pipeline
- Merge, split, organize pages (reorder + remove) and image-to-PDF with `lopdf`.
- Organize test PDFs with nested page trees and inherited attributes, bookmarks, links, form fields and tagged structure; confirm the output in Acrobat Reader, macOS Preview and a browser viewer.
- Confirm that removed pages' content is physically gone from the output file.
- Compression presets on text, vector and scanned PDFs.
- Previews with `pdfium-render` on a dedicated thread.
- Encrypted and malformed file handling.

### Benchmark scenarios

| Test | What we measure |
|---|---|
| Compress a 12 MP photo to under 2 MB | Quality, speed, memory |
| Resize a 50 MB image | Peak memory, UI responsiveness |
| Merge 100 PDFs | Time, peak memory (`lopdf` loads in memory), output integrity |
| Compress a scanned PDF (Medium/High) | Size reduction, readability |
| Preview a 200-page PDF | Time to first thumbnail, scroll smoothness, memory |
| Reorder pages and remove 20 pages in a 200-page PDF | Drag smoothness in the thumbnail grid, save time, output size drops accordingly, output integrity |

### Exit criteria and fallbacks

Lock in the libraries after reviewing size, startup, memory, speed, fidelity, licensing and packaging on both platforms.

| If… | Then… |
|---|---|
| `lopdf` memory is too high for large merges | Evaluate qpdf (Apache-2.0) behind the same trait |
| PDFium packaging is painful | Switch previews to PDF.js (Apache-2.0) |
| `mozjpeg` build issues in CI | Fall back to `image`'s built-in JPEG encoder, or `jpeg-encoder` (MIT/Apache-2.0) |
| Rust becomes a major blocker | Re-evaluate Electron (§5.3) |

---

## 15. MVP acceptance criteria

1. Installs and launches on supported Windows and macOS versions.
2. Core operations work offline after installation.
3. No account required.
4. Image compression accepts a maximum size and shows a before/after preview.
5. Image resize supports exact pixels and percentage.
6. The UI distinguishes dimensions from file size.
7. Image-to-PDF converts selected JPG/PNG images into one PDF.
8. PDFs merge in the chosen order.
9. PDFs split into individual pages and custom ranges.
10. PDF pages can be reordered with drag-and-drop (mouse and keyboard) and removed, then saved as a new PDF that contains exactly the kept pages in the chosen order. Removed pages' content is not present in the output file.
11. PDF compression offers Low/Medium/High and reports actual size changes.
12. History stores job details and output locations only.
13. The user can clear history without deleting documents.
14. A visible privacy screen explains local processing and network behaviour.
15. No telemetry or automatic update checks.
16. Original inputs are never overwritten automatically.
17. Exported images contain no GPS metadata by default.
18. The CI license gate passes, and third-party notices ship with the app.
19. Representative tests pass on Windows and macOS.

---

## 16. Two-developer split

| Workstream | Developer A (Rust) | Developer B (React) |
|---|---|---|
| Foundation | Tauri setup, commands, capabilities, job queue, CI | App shell, navigation, design system (Radix/Tailwind) |
| Image Toolkit | Decode/resize/encode pipeline, target-size search | Import, controls, before/after preview, save flow |
| PDF Toolkit | `lopdf` merge/split/organize/compress, image-to-PDF, PDFium worker | Thumbnails, file ordering, Organize pages grid (dnd-kit, reorder + remove), undo/redo, range input, progress UI |
| History | SQLite store, clear action | Recent-jobs UI, privacy screen, licenses screen |
| Quality | Rust tests, fixtures, benchmarks, fuzzing | Vitest, Playwright, accessibility, usability |
| Shared | Architecture, code review, licensing, releases | Architecture, code review, licensing, releases |

---

## 17. Open decisions

**Resolved** (see §6.3 and §6.4):
- Quality floor: JPEG/WebP quality 40.
- Image-to-PDF: pages sized to match each image; no page-size controls in the MVP.
- Password-protected PDFs: rejected in the MVP.
- Minimum OS: Windows 10 (1809+)/11 x64, macOS 11+.
- File paths in history: stored locally, clearable.
- PDF page reordering and removal: in the MVP.
- Architecture decisions AD-1 to AD-10: accepted.

**Still open** (none block starting implementation):
1. Are JPEG/PNG/WebP enough for the MVP? (Assumed yes; HEIC is excluded for licensing and patent reasons.)
2. Should upscaling be allowed? (Assumed yes for percentage resize, with a warning.)
3. Are Low/Medium/High presets acceptable when some PDFs can't shrink? (Assumed yes; the original is kept and the user is told.)
4. Can page rotation and insertion wait until after the MVP? (Assumed yes.)
5. Must the installer work fully offline, or is offline use after installation enough? (Assumed the latter; the default WebView2 bootstrapper.)
6. Final DPI and quality values for the Medium and High PDF presets, and final input limits (tune in the proof of concept).

---

## 18. Future expansion

| Feature | Where it fits |
|---|---|
| PDF page rotation, insertion | PDF engine (`lopdf`) + the existing Organize pages grid |
| PDF password tools | PDF engine |
| Batch processing | Job queue (application layer) |
| More image formats (AVIF, TIFF) | Image engine; check each codec's license first |
| Auto-update | `tauri-plugin-updater` (opt-in) |
| Mobile app | Tauri 2 mobile targets, reusing the Rust processing crates |

---

## 19. References

- Tauri 2 — https://v2.tauri.app/
- Tauri Windows installer / WebView2 modes — https://v2.tauri.app/distribute/windows-installer/
- Tauri capabilities — https://v2.tauri.app/security/capabilities/
- Tauri WebDriver testing — https://v2.tauri.app/develop/tests/webdriver/
- Tauri IPC mocking — https://v2.tauri.app/develop/tests/mocking/
- `image` — https://crates.io/crates/image
- `fast_image_resize` — https://crates.io/crates/fast_image_resize
- `mozjpeg` — https://crates.io/crates/mozjpeg
- `webp` — https://crates.io/crates/webp
- `oxipng` — https://crates.io/crates/oxipng
- `kamadak-exif` — https://crates.io/crates/kamadak-exif
- `lopdf` — https://crates.io/crates/lopdf
- `pdfium-render` — https://crates.io/crates/pdfium-render
- PDFium prebuilt binaries — https://github.com/bblanchon/pdfium-binaries
- `rusqlite` — https://crates.io/crates/rusqlite
- `cargo-deny` — https://github.com/EmbarkStudios/cargo-deny
- `cargo-about` — https://github.com/EmbarkStudios/cargo-about
- PDF.js — https://mozilla.github.io/pdf.js/
- Electron — https://www.electronjs.org/docs/latest
- Avalonia — https://docs.avaloniaui.net/
