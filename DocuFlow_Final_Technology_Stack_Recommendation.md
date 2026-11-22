# DocuFlow — Final Technology Stack Recommendation

## 1. Final stack

| Layer | Recommended technology | Why |
|---|---|---|
| Desktop framework | **Tauri 2** | Lightweight desktop shell |
| Frontend | **React** | Reusable UI components |
| Language | **TypeScript** | Type safety and maintainability |
| Frontend build | **Vite** | Fast development and builds |
| Backend | **Rust** | Efficient native processing |
| Image processing | **libvips** via Rust bindings | Fast, memory-efficient image operations |
| PDF manipulation | **qpdf** | Merge, split and structural PDF operations |
| PDF preview | **PDFium** | Render pages and thumbnails |
| Local history | **SQLite** | Store job metadata and output paths |
| Frontend testing | **Vitest + React Testing Library** | UI and business logic tests |
| Backend testing | **Rust unit/integration tests** | Validate file processing |
| End-to-end testing | **Playwright** | Test complete desktop workflows |
| CI/CD | **GitHub Actions** | Automated Windows/macOS builds |

Why these choices:

- Tauri doesn't bundle its own browser engine. It uses the operating system's WebView, helping reduce the application footprint.
- Rust keeps expensive image and PDF operations outside the React interface.
- libvips is designed for fast, low-memory image processing and supports a wide range of formats.
- qpdf provides PDF structural operations, but does not render PDF pages. That's why PDFium is a separate component.

## 2. Comparing the desktop frameworks

| Criteria | Tauri + Rust | Electron + Node.js | Avalonia + C# |
|---|---|---|---|
| Installer footprint | **Best potential** | Larger | Moderate |
| Memory usage | Generally lower baseline | Generally higher baseline | Depends on UI and libraries |
| Processing performance | Excellent potential | Good, with native modules | Excellent potential |
| Frontend development | React/TypeScript | React/TypeScript | C# UI or alternatives |
| Native integration | Rust | Node/native modules | .NET |
| Cross-platform | Windows/macOS | Windows/macOS | Windows/macOS |
| Ecosystem maturity | Growing | Very mature | Mature .NET ecosystem |
| Fit for your team | **Best fit** | Good | Less familiar |

Electron deserves a fair assessment: it is not inherently slow. It bundles Chromium and Node.js, and its maintainers emphasize that this can provide a consistent, high-performance rendering environment. Its main disadvantage for our requirements is the additional runtime footprint.

**My ranking for DocuFlow:**

1. Tauri + Rust — best overall fit.
2. Electron + React — best fallback if Rust integration becomes a major development obstacle.
3. Avalonia — worth considering if we decide to move to a C#-centric team.

## 3. The architecture I would build

```text
┌─────────────────────────────────────────────┐
│             Presentation layer              │
│          React · TypeScript · Vite           │
│  Image tools · PDF tools · Settings · UI     │
└──────────────────────┬──────────────────────┘
                       │
                 Tauri IPC commands
                       │
┌──────────────────────▼──────────────────────┐
│              Application layer               │
│                     Rust                     │
│ Validation · Job management · File handling  │
└───────────────┬──────────────────┬───────────┘
                │                  │
       ┌────────▼────────┐  ┌──────▼──────────┐
       │   Image engine  │  │   PDF engine    │
       │     libvips     │  │ qpdf + PDFium   │
       │ Resize          │  │ Merge           │
       │ Compress        │  │ Split           │
       │ Convert         │  │ Compress        │
       │                 │  │ Preview         │
       └─────────────────┘  └─────────────────┘

┌─────────────────────────────────────────────┐
│                Local storage                 │
│ SQLite · Job metadata · Output locations     │
└─────────────────────────────────────────────┘
```

### 4. How we keep the code maintainable

I strongly recommend a modular architecture from the beginning.

```text
docuflow/
├── src/                         # React frontend
│   ├── components/
│   ├── features/
│   │   ├── image-tools/
│   │   ├── pdf-tools/
│   │   ├── history/
│   │   └── settings/
│   └── services/
│
├── src-tauri/
│   └── src/
│       ├── commands/
│       ├── application/
│       ├── processing/
│       │   ├── images/
│       │   └── pdf/
│       ├── storage/
│       └── infrastructure/
│
└── tests/
```

The important design principle is to keep the UI independent of the processing libraries.

For example, React should request:

```typescript
compressImage({
  inputPath: "/path/photo.jpg",
  targetSizeBytes: 2 * 1024 * 1024
});
```

It should not know whether Rust uses libvips, another image library, or a future replacement.

This makes it easier to replace processing engines without rewriting the interface.

## 5. Future updates and expansion

We should design the application so new functionality can be added without major architectural changes.

| Future feature | Where it fits |
|---|---|
| PDF page removal | PDF processing module |
| PDF page reordering | PDF processing module |
| PDF password tools | PDF processing module |
| Image format expansion | Image processing module |
| Batch image processing | Job management |
| Additional image formats | Image processing module |
| Mobile application | Separate frontend/client using reusable processing logic where practical |

For updates, I recommend:

- Keep Rust dependencies and frontend dependencies independently versioned.
- Use automated dependency checks and security scanning.
- Build and test Windows and macOS installers through CI.
- Sign Windows and macOS releases before public distribution.
- Add automatic update functionality later, as requested.

One caveat: libvips is LGPL-licensed, not permissively licensed. We should review its distribution and licensing obligations before shipping it commercially.

## 6. Important: benchmark before locking the processing libraries

I would finalize the application framework now, but keep the image and PDF engine choices subject to a short proof of concept.

Test these scenarios:

| Test | What we measure |
|---|---|
| Compress a 12 MP photograph to under 2 MB | Quality, speed and memory |
| Resize a 50 MB image | Peak memory and responsiveness |
| Merge 100 PDFs | Execution time and output integrity |
| Compress a scanned PDF | Actual size reduction and readability |
| Preview a 200-page PDF | Rendering speed and memory |

Also measure:

- Cold startup time.
- Installer size on Windows and macOS.
- Idle memory consumption.
- Memory consumption during large operations.

Don't promise an installer under 10 MB until we've measured the actual packaged product, including native libraries and platform-specific requirements.

---

## Final decision

**Recommended for DocuFlow**

**Tauri 2 + React + TypeScript + Vite + Rust + libvips + qpdf + PDFium + SQLite.**

I would commit to this architecture, then validate the PDF and image engines with a small prototype before implementing the complete MVP.

This gives us a good balance of performance, package size, maintainability and future expansion without abandoning the React ecosystem you already know.
