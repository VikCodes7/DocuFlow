# DocuFlow --- MVP Product Requirements & Technology Decision

**Version:** 2.0\
**Date:** 19 November 2026\
**Platforms:** Windows and macOS\
**Status:** Working product and technical proposal

## 1. Executive recommendation

Build the first proof of concept with **Tauri 2 + React + TypeScript +
Rust**. This best matches DocuFlow's priorities: low application
overhead, responsive native file processing, local-first privacy, and
reuse of the team's React/TypeScript experience.

Do not promise that every installer will be under 10 MB. Tauri uses the
system WebView, but Windows WebView2 installation mode affects installer
size: the default bootstrapper is small but may need internet to
download the runtime; bundling the offline runtime adds approximately
127 MB according to Tauri's documentation. Measure actual release
artifacts on both platforms. [Tauri Windows installer
docs](https://v2.tauri.app/distribute/windows-installer/)

**Recommended stack**

  -----------------------------------------------------------------------
  Layer                               Recommendation
  ----------------------------------- -----------------------------------
  Desktop shell                       Tauri 2

  UI                                  React + TypeScript

  Build                               Vite

  Native/backend layer                Rust

  Image processing                    Prototype libvips via
                                      Rust-compatible integration;
                                      compare with pure Rust

  PDF structure operations            qpdf

  PDF rendering/previews              PDFium

  Local history                       SQLite or a small local metadata
                                      store

  Frontend tests                      Vitest + React Testing Library

  Rust tests                          `cargo test`

  E2E                                 Playwright or Tauri-compatible E2E
                                      setup

  CI                                  GitHub Actions or GitLab CI

  Auto-update                         Not in MVP
  -----------------------------------------------------------------------

## 2. Product vision and principles

DocuFlow is a privacy-first desktop utility for common image and PDF
tasks, particularly when websites and portals impose strict file-size or
dimension requirements.

> **Your files. Your device. Your privacy.**

-   Installable desktop application for Windows and macOS.
-   Core image and PDF operations work offline after installation.
-   Files are processed locally; normal workflows do not upload
    documents to DocuFlow servers.
-   No account or cloud workspace in MVP.
-   No telemetry by default.
-   No automatic update checks or auto-update feature in MVP.
-   Small footprint and low memory usage are explicit engineering goals.

## 3. MVP scope

### 3.1 Image Toolkit

  -----------------------------------------------------------------------
  Feature                             MVP behaviour
  ----------------------------------- -----------------------------------
  Compress image to target size       User enters a maximum size (for
                                      example, under 2 MB); DocuFlow
                                      searches for the best acceptable
                                      quality that fits, where feasible.
                                      Show before/after preview and
                                      actual file sizes.

  Resize by exact pixels              User enters target width and
                                      height, with preserve-aspect-ratio
                                      option.

  Resize by percentage                User selects a percentage to shrink
                                      or enlarge.

  Image-to-PDF                        Convert one or more JPG/PNG images
                                      into one PDF, normally one image
                                      per page.

  Before/after preview                Show original and output preview,
                                      dimensions, format and file size.
  -----------------------------------------------------------------------

**Critical UX rule:** Dimensions and file size are separate settings.
Resizing changes pixel dimensions; compression changes encoding/quality
and resulting file size. Explain this in the interface.

### 3.2 PDF Toolkit

  -----------------------------------------------------------------------
  Feature                             MVP behaviour
  ----------------------------------- -----------------------------------
  Merge PDFs                          Combine multiple PDFs in a chosen
                                      order.

  Split PDFs                          Export individual pages or
                                      user-defined page ranges.

  Compress PDFs                       Low, Medium and High presets. No
                                      "compress to exactly X MB" feature
                                      in MVP.

  PDF preview                         Render pages for previews where
                                      required.
  -----------------------------------------------------------------------

PDF compression presets must be tested on text, vector and image-heavy
PDFs. Structural compression may not meaningfully reduce scanned PDFs
unless embedded images are also recompressed or downsampled. Report
actual results and do not promise a reduction that did not happen.

### 3.3 Local recent history

-   Store job details and output locations only.
-   Do not store thumbnails, document contents or copies of source
    files.
-   Provide a visible **Clear history** action.
-   Clearing history removes DocuFlow job records only; it must never
    delete source or output documents.
-   If a referenced file has moved or been deleted, show it as
    unavailable.

Suggested metadata: operation type, date/time, source filenames, output
filename(s), output path(s), status and result sizes. File paths can
reveal personal information, so keep history local and clearable.

### 3.4 Visible proof of privacy

-   Add a short "How your files stay local" screen or panel.
-   Explain that normal processing is on-device and files are not
    uploaded.
-   No telemetry by default.
-   No analytics SDK sending usage or file metadata without consent.
-   No automatic update checks in MVP.
-   Document any installer-time network activity, especially WebView2
    bootstrapper behaviour.
-   Use narrow Tauri permissions/capabilities. [Tauri
    capabilities](https://v2.tauri.app/security/capabilities/)

## 4. Detailed functional requirements

### 4.1 Target-size image compression

**User story:** As a user, I want to enter a maximum file size and have
DocuFlow produce the best-looking image that fits without uploading my
photo.

1.  Import an image through file picker or drag-and-drop.
2.  Show original preview, dimensions, format and file size.
3.  Accept a maximum output size (for example, 2 MB).
4.  Let the user choose or confirm output format.
5.  Search encoding settings for the highest acceptable quality at or
    below the target.
6.  Show before/after preview and actual output size.
7.  Let the user choose the output path.
8.  Never overwrite the original by default.
9.  If the target cannot be reached above the quality floor, explain why
    and offer alternatives.
10. Work offline and locally.

**Technical note:** For JPEG/WebP-like lossy formats, encode candidates
at different quality settings and use a bounded search to find the
highest quality under the byte limit. This cannot guarantee an
acceptable result for every input/target combination.

PNG is often lossless; reaching a very small target may require palette
quantization, converting to a lossy format or resizing. Do not silently
degrade beyond an agreed quality threshold. Define minimum quality
behaviour.

### 4.2 Image resizing

-   Exact width and height in pixels.
-   Percentage-based resizing.
-   Preserve-aspect-ratio option.
-   Show original and resulting dimensions.
-   Explain that dimensions and file size are different.
-   Preview before saving.
-   Respect EXIF orientation.
-   Define safe metadata defaults; avoid unintentionally retaining GPS
    metadata.

### 4.3 Image-to-PDF conversion

-   Import one or more JPG/PNG images.
-   Allow image ordering.
-   Convert selected images into one PDF, normally one image per page.
-   Define default page sizing and fit behaviour.
-   Choose output name and destination.
-   Preserve source images.

**Decision to confirm:** whether MVP needs A4/Letter, margins,
orientation and scaling controls, or a simple automatic fit-to-page
default.

### 4.4 Merge PDFs

-   Select two or more PDFs.
-   Display filename and page count.
-   Reorder input files.
-   Merge in displayed order.
-   Choose output filename and destination.
-   Save as a new file.
-   Show clear errors for invalid, unsupported or password-protected
    files.

### 4.5 Split PDFs

-   Split into one PDF per page.
-   Split using custom page ranges.
-   Validate ranges and explain invalid/overlapping inputs.
-   Preview or summarize outputs before processing.
-   Choose output folder and filename pattern.
-   Preserve original file.

### 4.6 PDF compression presets

-   Offer Low, Medium and High.
-   Explain intended quality/size trade-offs.
-   Show original and resulting file size and percentage change.
-   If the result is not smaller, say so.
-   Save to a new file; never overwrite source automatically.

**Proposed preset intent (validate in POC):** - Low: conservative,
prioritize fidelity; structural optimization/minimal image changes. -
Medium: balanced; moderate image recompression/downsampling where
supported. - High: prioritize smaller output; stronger image
recompression/downsampling with quality warning.

These are product definitions, not guaranteed qpdf capabilities. qpdf
can compress PDF structures and streams, but it is not by itself a full
image-downsampling engine. [qpdf compression
docs](https://qpdf.readthedocs.io/en/latest/cli.html)

## 5. Technology comparison

  -----------------------------------------------------------------------
  Criterion         Tauri 2 + React + Electron +        Avalonia +
                    Rust              React + Node      C#/.NET
  ----------------- ----------------- ----------------- -----------------
  Windows/macOS     Yes               Yes               Yes

  UI                React +           React +           C# + XAML
                    TypeScript        TypeScript        

  Runtime approach  System WebView    Bundled           Avalonia native
                                      Chromium +        platform backend
                                      Node.js           

  Package footprint Usually smallest  Generally larger  Depends on .NET
                    starting point;   due to Chromium   deployment and
                    native libraries  and Node          dependencies
                    add size                            

  Memory baseline   Lower potential   Usually higher    Can be efficient;
                    overhead;         baseline          benchmark
                    benchmark                           required
                    required                            

  Image ecosystem   Rust              Excellent Node    .NET image
                    bindings/native   ecosystem,        libraries
                    libs              including sharp   

  PDF ecosystem     Rust/native       JS                .NET/native
                    libraries         packages/native   bindings
                                      modules           

  Team's React      Reused            Reused            Less reusable
  experience                                            

  Learning curve    Rust              Lowest            C# and XAML

  Best fit          Footprint, native Development speed Existing .NET
                    processing,       and broad         desktop team
                    restricted        ecosystem         
                    permissions                         
  -----------------------------------------------------------------------

### 5.1 Tauri --- recommended

**Pros:** system WebView rather than bundled Chromium; Rust for native
processing; React/TypeScript UI; scoped capabilities; local-first
architecture.

**Cons:** Rust learning curve; native library packaging; WebView
differences between Windows and macOS; PDFium/libvips can add size.

Tauri uses WebView2 on Windows and WKWebView on macOS. Windows installer
modes have different network and size trade-offs. [Tauri
installer](https://v2.tauri.app/distribute/windows-installer/) · [Tauri
capabilities](https://v2.tauri.app/security/capabilities/)

### 5.2 Electron --- development-speed alternative

Electron embeds Chromium and Node.js, providing a mature and consistent
web runtime across platforms. It has easy access to Node packages such
as sharp, but generally increases package size and memory baseline.
[Electron overview](https://www.electronjs.org/docs/latest)

Choose Electron if speed of development, ecosystem convenience or
rendering consistency becomes more important than footprint.

### 5.3 Avalonia --- .NET alternative

Avalonia supports Windows and macOS and is MIT licensed. It is a strong
desktop option for a C#/.NET team, but React/TypeScript experience is
less directly reusable. It is not automatically smaller or faster than
Tauri; benchmark it if the team prefers .NET. [Avalonia
platforms](https://docs.avaloniaui.net/docs/supported-platforms) ·
[Avalonia licensing](https://docs.avaloniaui.net/tools/faq)

### Final recommendation

Choose **Tauri 2 + React + TypeScript + Rust** for the first proof of
concept. Do not make the final decision from marketing claims or
estimated package sizes; measure release builds.

## 6. Image-processing library strategy

### libvips

libvips is a strong performance candidate. The Node `sharp` package uses
libvips and documents fast resizing, format conversion and quality
controls. [sharp docs](https://sharp.pixelplumbing.com/)

Integration caveat: - `sharp` is a Node.js native module. Tauri does not
bundle Node by default. - Adding a Node sidecar/runtime solely for sharp
may erode Tauri's size advantage. - Prototype libvips through
Rust-compatible bindings or its C API. - Compare with a pure-Rust image
pipeline for required formats.

**Recommendation:** benchmark both approaches. Select the one with the
best combined performance, quality, maintenance and packaged footprint.

### Target-size algorithm

1.  Read image and normalize orientation.
2.  Keep dimensions fixed unless user chooses resize.
3.  Encode an initial candidate.
4.  Measure bytes.
5.  Adjust quality up/down based on target.
6.  Stop at highest acceptable quality below target, or at quality
    floor.
7.  Show actual size and preview.

Do not treat PNG quality as equivalent to JPEG quality. Define whether
PNG target compression uses lossless optimization, palette quantization
or format conversion.

## 7. PDF library strategy

### qpdf

qpdf is a content-preserving PDF transformation tool and C++ library. It
supports merge, split and structural transformations; it does not render
pages. It is Apache 2.0 licensed. [qpdf project and
license](https://qpdf.sourceforge.io/) · [qpdf
repository](https://github.com/qpdf/qpdf)

Candidate uses: - Merge. - Split by page ranges. - Structural
optimization and stream/object compression.

It does not automatically downsample embedded images, so image-heavy
scanned PDFs may not shrink substantially.

### PDFium

Use PDFium as a candidate for rendering page thumbnails and previews,
not as the primary merge/split engine. Review its BSD-style license and
all third-party notices before redistribution. [PDFium
license](https://github.com/chromium/pdfium/blob/main/LICENSE)

### pdf-lib

Could be useful in a JavaScript prototype, but do not assume it provides
robust PDF compression. Given the Rust direction, avoid maintaining a
second PDF-processing stack unless it clearly simplifies a required
workflow.

### Licensing caution

-   qpdf: Apache 2.0.
-   PDFium: BSD-style license plus third-party notices.
-   Ghostscript: AGPL or commercial licensing. Its vendor says
    closed-source proprietary distribution requires a commercial license
    unless AGPL obligations are followed. [Ghostscript licensing
    FAQ](https://www.ghostscript.com/faq/index.html)
-   MuPDF: verify current terms directly with Artifex before
    distribution; do not assume permissive licensing.

Review exact versions and transitive dependencies before release. This
is technical planning, not legal advice.

## 8. Performance and package-size principles

-   Keep CPU-heavy work out of the UI thread.
-   Use Rust worker threads/background tasks.
-   Process incrementally where possible.
-   Avoid loading multiple full-resolution images into memory.
-   Generate thumbnails at display resolution.
-   Report progress and support safe cancellation.
-   Avoid repeated image decode/encode cycles.
-   Include only required codecs/native libraries.
-   Avoid bundling Node solely for sharp unless measured benefits
    justify it.
-   Measure installer size and installed size separately.
-   Keep debug symbols out of production packages while retaining them
    for diagnostics.

**Measure these metrics on both operating systems:**

  Metric              Measurement
  ------------------- ----------------------------------------------
  Cold startup        Launch to usable screen
  Idle memory         Settled resident memory
  Peak memory         Large image/PDF operations
  Installer size      Downloadable package size
  Installed size      On-disk application footprint
  Processing time     Fixed test corpus and machine specs
  UI responsiveness   Navigation/scrolling/cancel while processing

## 9. Privacy and security

1.  No account or cloud workspace.
2.  No document uploads in normal workflows.
3.  No telemetry by default.
4.  No automatic update checks or auto-update in MVP.
5.  Explain any installer-time network activity.
6.  Use narrow Tauri capabilities.
7.  Access only user-selected files and authorized output locations.
8.  Store minimal recent-job metadata.
9.  Clean temporary files after success, failure or cancellation.
10. Do not place document contents, thumbnails or full paths in
    telemetry/logs.
11. Avoid leaking GPS metadata in exported images.
12. Test malformed and unusually large files.

## 10. Distribution

-   Produce Windows and macOS installers/packages.
-   Test clean install and launch on supported OS versions.
-   Plan code signing and macOS notarization before public distribution.
-   Include privacy information and third-party notices.
-   **No auto-update feature in MVP.**
-   **No automatic update checks in MVP.**

## 11. MVP acceptance criteria

1.  Installs and launches on supported Windows and macOS versions.
2.  Core operations work offline after installation.
3.  No account required.
4.  Image compression accepts a maximum size and shows before/after
    preview.
5.  Image resize supports exact pixels and percentage.
6.  UI distinguishes dimensions from file size.
7.  Image-to-PDF converts selected JPG/PNG images into one PDF.
8.  PDFs merge in chosen order.
9.  PDFs split into individual pages and custom ranges.
10. PDF compression has Low/Medium/High presets and reports actual size
    changes.
11. Recent history stores job details and output locations only.
12. User can clear history without deleting documents.
13. Visible privacy explanation describes local processing and network
    behaviour.
14. No telemetry or automatic update checks by default.
15. Original inputs are never overwritten automatically.
16. Representative tests pass on Windows and macOS.

## 12. Proof of concept before full implementation

### A. Desktop shell

-   Tauri + React + TypeScript.
-   Build Windows and macOS artifacts.
-   Measure startup, idle memory, installer size and installed size.
-   Test file picker, drag/drop and local writes.
-   Test offline launch after installation.

### B. Image pipeline

-   Resize JPEG/PNG by exact pixels and percentage.
-   Target-size compress JPEG/WebP with quality search.
-   Compare libvips via Rust integration against pure Rust.
-   Measure speed, memory, visual quality and native library footprint.
-   Test EXIF orientation and metadata removal.

### C. PDF pipeline

-   Merge PDFs.
-   Split by ranges.
-   Compress text, vector and scanned/image-heavy PDFs.
-   Render previews with PDFium.
-   Measure fidelity, size change, processing time and package
    footprint.
-   Test encrypted and malformed files.

### Exit criteria

Choose only after reviewing installer/installed size, startup, memory,
processing time, fidelity, licensing and packaging complexity on both
platforms.

## 13. Suggested two-developer split

  -----------------------------------------------------------------------
  Workstream              Developer A             Developer B
  ----------------------- ----------------------- -----------------------
  Desktop foundation      Tauri, Rust commands,   React shell,
                          file access,            navigation, shared UI
                          permissions             

  Image Toolkit           Native image pipeline   Import, controls,
                          and target-size search  preview and save flow

  PDF Toolkit             qpdf integration and    PDF preview, ordering,
                          processing experiments  ranges and progress UI

  History                 Local metadata store    Recent-jobs UI and
                          and clear action        privacy screen

  Quality                 Rust tests, performance Component/E2E tests,
                          and file edge cases     usability and
                                                  accessibility

  Shared                  Architecture, code      Architecture, code
                          reviews, licensing and  reviews, licensing and
                          cross-platform release  cross-platform release
  -----------------------------------------------------------------------

Keep processing behind clear interfaces so the UI is not coupled to a
specific library.

## 14. Open decisions

1.  Are JPEG/PNG/WebP sufficient for MVP, or is HEIC/HEIF required?
2.  What is the minimum acceptable quality when a target size cannot be
    reached?
3.  Should upscaling be allowed?
4.  Does image-to-PDF need A4/Letter, margins and orientation controls?
5.  Are Low/Medium/High PDF presets acceptable if some PDFs cannot
    shrink?
6.  Should password-protected PDFs be rejected in MVP?
7.  Earlier scope included remove-page and reorder-page PDF operations.
    They are not in the latest MVP feature list. Confirm whether to
    retain them or defer them.
8.  What minimum Windows and macOS versions are supported?
9.  Are local filenames and file paths acceptable in recent history?
10. Must the installer itself work fully offline, or is offline
    operation after installation sufficient?

## 15. References

-   [Tauri 2 Windows installer / WebView2
    modes](https://v2.tauri.app/distribute/windows-installer/)
-   [Tauri capabilities and
    permissions](https://v2.tauri.app/security/capabilities/)
-   [Electron overview](https://www.electronjs.org/docs/latest)
-   [Electron process
    model](https://www.electronjs.org/docs/latest/tutorial/process-model)
-   [Avalonia supported
    platforms](https://docs.avaloniaui.net/docs/supported-platforms)
-   [Avalonia licensing FAQ](https://docs.avaloniaui.net/tools/faq)
-   [sharp image processing](https://sharp.pixelplumbing.com/)
-   [sharp output options](https://sharp.pixelplumbing.com/api-output/)
-   [qpdf project](https://qpdf.sourceforge.io/)
-   [qpdf compression
    options](https://qpdf.readthedocs.io/en/latest/cli.html)
-   [PDFium
    license](https://github.com/chromium/pdfium/blob/main/LICENSE)
-   [Ghostscript licensing
    FAQ](https://www.ghostscript.com/faq/index.html)

------------------------------------------------------------------------

**Scope note:** This version reflects the latest feature list: PDF
compression presets are in the MVP; custom PDF target-size compression
and auto-updates are not. Earlier remove-page and reorder-page features
are marked for confirmation rather than silently assumed.
