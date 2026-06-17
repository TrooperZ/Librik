# Librik Codebase Explanation

Librik is currently a Tauri v2 desktop app shell with a Svelte + TypeScript + Vite frontend. The visible UI is shaped as a VS Code/Obsidian-inspired PDF workspace: activity rail, explorer pane, editor tab group, dense library table, mocked reader canvas, persistent PDF details inspector, and compact app chrome.

## Top-Level Structure

```text
.
├── index.html
├── package.json
├── package-lock.json
├── tsconfig.json
├── tsconfig.node.json
├── vite.config.ts
├── src/
│   ├── App.svelte
│   ├── app.css
│   ├── main.ts
│   ├── vite-env.d.ts
│   ├── components/
│   │   ├── ActivityRail.svelte
│   │   ├── LibraryView.svelte
│   │   ├── PdfDetailsPanel.svelte
│   │   ├── ReaderView.svelte
│   │   ├── SourceSidebar.svelte
│   │   ├── WorkspaceTabs.svelte
│   │   └── TopBar.svelte
│   └── lib/
│       ├── mockData.ts
│       └── types.ts
└── src-tauri/
    ├── Cargo.toml
    ├── Cargo.lock
    ├── build.rs
    ├── tauri.conf.json
    ├── capabilities/
    │   └── default.json
    ├── icons/
    │   └── icon.png
    └── src/
        ├── cache.rs
        ├── commands.rs
        ├── database.rs
        ├── indexing.rs
        ├── lib.rs
        ├── library.rs
        ├── main.rs
        ├── pdf.rs
        ├── storage.rs
        └── sync.rs
```

`Librik Basic UI Mockup.fig` is an existing mockup artifact in the repo root. It is not used by the build.

## Frontend Files

### `index.html`

This is the Vite HTML entrypoint. It defines the document shell, sets the viewport, gives the page the title `Librik`, and mounts the TypeScript app through:

```html
<script type="module" src="/src/main.ts"></script>
```

The only body element is `<div id="app"></div>`, which keeps Svelte responsible for the full application UI.

### `package.json`

This defines the Node project, scripts, and frontend/Tauri dependencies.

Scripts:

- `npm run dev` starts the Vite dev server on `127.0.0.1`.
- `npm run build` creates the production frontend bundle in `dist/`.
- `npm run preview` serves the built Vite output.
- `npm run tauri dev` and `npm run tauri build` work through the `tauri` script.

Dependencies:

- `svelte` provides the UI framework.
- `@sveltejs/vite-plugin-svelte` integrates Svelte with Vite.
- `vite` builds and serves the frontend.
- `typescript` provides static typing.
- `@tauri-apps/api` exposes the frontend-to-native command bridge.
- `@tauri-apps/cli` runs Tauri development and build commands.
- `@lucide/svelte` provides compact interface icons.

### `package-lock.json`

This is npm's resolved dependency lockfile. It pins the exact package graph installed by `npm install`, making future installs reproducible.

### `vite.config.ts`

This configures Vite for Svelte:

- `plugins: [svelte()]` enables Svelte compilation.
- `clearScreen: false` keeps Rust/Tauri output readable when Tauri runs Vite as a child process.
- `server.strictPort: true` ensures the dev URL stays on the Tauri-configured port.
- `server.watch.ignored: ['**/src-tauri/**']` prevents frontend hot reload from watching Rust build output.

### `tsconfig.json`

This is the main TypeScript config for app code. It enables strict type checking, bundler-style module resolution, isolated modules, and includes TypeScript and Svelte files under `src/`.

### `tsconfig.node.json`

This TypeScript config covers Node-side tooling files, currently `vite.config.ts`.

### `src/vite-env.d.ts`

This brings in Vite and Svelte ambient types so TypeScript understands `.svelte` modules and Vite-specific globals.

### `src/main.ts`

This is the Svelte app entrypoint. It imports the global stylesheet, imports `App.svelte`, and mounts the component using the Svelte 5 API:

```ts
const app = mount(App, {
  target: document.getElementById('app') as HTMLElement
});
```

Using `mount` is important because Svelte 5 no longer supports `new App(...)` by default.

### `src/App.svelte`

This file is the composition root for the desktop shell.

The `<script>` block:

- Imports the major shell components.
- Imports OpenPencil-derived mock data for sources, books, reader marks, and notebook cards.
- Tracks UI state:
  - active activity rail view.
  - open workspace tabs.
  - active workspace tab.
  - selected source.
  - selected book row.
  - mutable mocked book rows.
- Provides mocked behavior for importing a PDF, adding/removing tags, changing progress, and changing reading status.

The markup defines the major application structure:

- `TopBar`: app title, lightweight app menu, global search, import/sync/settings/profile controls.
- `ActivityRail`: VS Code-style primary activity rail.
- `SourceSidebar`: explorer pane with open editors and a Zotero/Mail/Obsidian/VS Code-style library tree.
- `WorkspaceTabs`: editor-style tab strip for library and open PDFs.
- `.workspace`: active library or reader surface.
- `LibraryView`: dense table-management screen.
- `ReaderView`: mocked reader screen.
- `PdfDetailsPanel`: persistent side inspector for selected PDF preview, metadata, tags, progress, status, and notes.

The handlers keep the shell interactive without surfacing implementation labels in the UI.

### `src/components/TopBar.svelte`

Top application toolbar based on the OpenPencil mockup. It contains:

- `Librik` wordmark.
- lightweight File/Edit/View menu buttons.
- Global search field.
- Clickable icon buttons for import, sync, settings, and profile.

The import control is mocked through `App.svelte` and adds a local in-memory PDF row. The other controls are clickable UI shell commands.

### `src/components/ActivityRail.svelte`

Far-left activity rail inspired by VS Code.

It renders icon-only buttons for:

- Library.
- Search.
- Tags.
- Recent.
- Settings.

Clicking an activity changes the highlighted rail item and updates the selected source context without replacing the active editor tab or launching to a blank page.

### `src/components/SourceSidebar.svelte`

Activity-scoped side pane. It renders only the content for the selected activity:

- Library: a single `My Library` folder tree, nested collection folders, special buckets, and a new-folder icon next to the Library label.
- Search: a compact search field with a filter button.
- Tags: a tag search field, add-tag button, scrollable checkbox tag list, and mocked add-tag modal.
- Recent: a compact list of recently opened files.
- Settings: basic mocked workspace settings controls.

The folder tree is only rendered for the Library activity. Buttons update the selected vault view state in `App.svelte`.

### `src/components/WorkspaceTabs.svelte`

Editor-style workspace tab strip.

It renders:

- A permanent Library tab.
- Open PDF tabs with document icons and close buttons.
- A plus button that creates a mocked imported PDF and opens it as a new tab.

Selecting a PDF tab makes that PDF the selected document and switches the workspace to the reader. Closing the active PDF tab falls back to the nearest remaining tab.

### `src/components/LibraryView.svelte`

Dense library management view matching the OpenPencil direction while feeling closer to Obsidian/VS Code density:

- Compact header and library count.
- Import, folder, list, shelf, filter, and sort controls.
- Table header.
- Clickable book/PDF rows with creator, type, tags, progress, source, and modified date.
- Per-row `Read` and more-action controls.

Selecting a row updates the active title and inspector. Opening the reader switches the app to the reader workspace.
Opening a reader creates or focuses a PDF tab rather than using a single global reader tab.

### `src/components/PdfDetailsPanel.svelte`

Persistent right-side inspector for the selected PDF.

It renders:

- Selected PDF title and author.
- Mocked cover/page preview.
- Details, Tags, and Notes tabs.
- Description, file path, metadata, current page, progress, file size, source, modified date, and fingerprint.
- Editable mocked tags.
- Suggested tag buttons.
- Mocked progress controls.
- Mocked reading status controls.
- Notebook preview cards and export/citation actions.

All behavior is in-memory UI state. It does not read files, parse PDFs, edit annotations, or persist data.

### `src/components/ReaderView.svelte`

Reader shell based on the second OpenPencil screen:

- Left rail for outline and quick marks.
- Reader toolbar for highlight, note, bookmark, notebook, and tags.
- PDF page surface with text lines, highlight area, bookmark ribbon, and margin note anchor.
- Bottom page/zoom/search control strip.

The reader is a UI shell for the reading workflow. It keeps mocked PDF visuals only and does not implement real PDF annotation behavior.

### `src/app.css`

This is the complete visual system for the shell. It uses a restrained neutral palette, compact spacing, clear borders, and fixed desktop-style panel layout.

Important layout rules:

- `.app-shell` is a CSS grid with top app bar, activity rail, explorer pane, workspace, and persistent PDF inspector.
- `.activity-rail` provides VS Code-style primary navigation.
- `.source-sidebar` keeps library navigation compact and stable as an explorer pane.
- `.library-panel-header`, `.vault-tree`, `.tree-row`, `.tree-disclosure`, and `.tree-icon` render the nested Zotero-style library tree.
- `.search-panel`, `.tags-panel`, `.tag-check-list`, `.recent-panel`, and `.mock-modal` style the mutually exclusive activity pane content.
- `.workspace-tabs`, `.tab-strip`, `.tab-item`, and `.workspace-tab` provide the editor-style tab behavior and styling.
- `.library-table` presents the management view as a dense table.
- `.reader-view` splits outline and reader canvas while the persistent inspector handles details, tags, and notes.
- `.pdf-inspector` provides the right side metadata/tag/notes panel.
- `.pdf-page` provides the reader page surface.
- The media query tightens columns at narrower desktop widths.

## Frontend Bridge

### `src/lib/types.ts`

This file defines shared frontend TypeScript types:

- `WorkspaceTab` describes the permanent Library tab and closable PDF reader tabs.
- `AppAction` enumerates shell-level clickable actions.
- `BookRow` describes dense table and inspector data, including title, author, status, source, progress, file path, description, metadata, tags, fingerprint, and current page.
- `ReaderMark` and `NoteCard` describe reader-side content.

Keeping these types separate prevents UI markup from becoming the source of truth for app contracts.

### `src/lib/mockData.ts`

OpenPencil-derived static shell data:

- Library table rows with mocked metadata and tags.
- Reader quick marks.
- Notebook cards.

This data keeps the shell populated while real library data is introduced later.

## Tauri And Rust Files

### `src-tauri/tauri.conf.json`

This is the Tauri v2 app configuration.

Important values:

- `productName: "Librik"` sets the product name.
- `identifier: "com.librik.app"` matches the requested bundle identifier.
- `build.devUrl: "http://localhost:5173"` points Tauri dev mode at Vite.
- `build.beforeDevCommand: "npm run dev"` starts the frontend server.
- `build.beforeBuildCommand: "npm run build"` builds the frontend before native packaging.
- `build.frontendDist: "../dist"` points Tauri at the production frontend output.
- `app.windows[0].title: "Librik"` sets the native window title.
- `bundle.targets: ["app"]` builds the macOS `.app` bundle locally.

Tauri warns that identifiers ending in `.app` are not recommended on macOS because `.app` is also the application bundle extension. The value is kept because it was explicitly requested.

### `src-tauri/capabilities/default.json`

This is the default Tauri v2 capability file for the main window. It grants `core:default` permissions to the `main` window. No filesystem, dialog, shell, opener, database, or network permissions are granted yet.

### `src-tauri/Cargo.toml`

This defines the Rust crate for the native backend.

Key parts:

- Package name: `librik`.
- Library crate name: `librik_lib`.
- Crate types include `staticlib`, `cdylib`, and `rlib`, matching Tauri's app structure.
- `tauri-build` runs Tauri code generation at build time.
- `tauri` provides the runtime and command system.
- `serde` serializes Rust command responses into frontend-readable JSON.

### `src-tauri/Cargo.lock`

This is Cargo's resolved Rust dependency lockfile. It pins the native dependency graph used by Tauri and Rust builds.

### `src-tauri/build.rs`

This is Tauri's Rust build script:

```rust
fn main() {
    tauri_build::build();
}
```

It lets Tauri generate build-time context from `tauri.conf.json`, icons, capabilities, and frontend assets.

### `src-tauri/icons/icon.png`

This is the app icon used by Tauri for the desktop bundle.

### `src-tauri/src/main.rs`

This is the native binary entrypoint. It delegates immediately to the library crate:

```rust
fn main() {
    librik_lib::run();
}
```

The `windows_subsystem = "windows"` attribute suppresses an extra console window in Windows release builds.

### `src-tauri/src/lib.rs`

This is the native app composition root.

It declares module boundaries for future backend responsibilities:

- `cache`
- `commands`
- `database`
- `indexing`
- `library`
- `pdf`
- `storage`
- `sync`

The `run()` function creates the Tauri builder, registers command handlers, and starts the app:

```rust
tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![commands::backend_status])
    .run(tauri::generate_context!())
    .expect("failed to run Librik desktop shell");
```

This keeps the native app entrypoint ready for Tauri commands as the product grows.

### `src-tauri/src/commands.rs`

This file currently exposes one internal command: `backend_status`.

`BackendStatus` is a serializable Rust struct. The `#[serde(rename_all = "camelCase")]` attribute converts Rust field names like `app_name` and `platform_targets` into frontend-friendly JSON keys like `appName` and `platformTargets`.

The command:

```rust
#[tauri::command]
pub fn backend_status() -> BackendStatus
```

returns a lightweight health payload. It is not shown in the visible app chrome.

### `src-tauri/src/library.rs`

Module boundary for document records, collections, tags, and recents.

### `src-tauri/src/storage.rs`

Module boundary for local folder sources and cloud storage adapters.

### `src-tauri/src/database.rs`

Module boundary for SQLite access, migrations, and repository implementations.

### `src-tauri/src/indexing.rs`

Module boundary for metadata extraction, text extraction, and background job orchestration.

### `src-tauri/src/pdf.rs`

Module boundary for PDF metadata extraction and renderer bridge code.

### `src-tauri/src/sync.rs`

Module boundary for sidecar and cloud sync coordination.

### `src-tauri/src/cache.rs`

Module boundary for thumbnail, extracted text, and rendered-page cache code.

## Architecture Boundary

The current rule is:

- Svelte owns layout, navigation, controls, and user interaction.
- Rust owns native command implementations and future filesystem, SQLite, indexing, PDF, cache, storage, and sync work.

The frontend does not directly assume a future database schema, absolute filesystem paths, PDF renderer internals, indexing implementation, or sync provider.

## Current Verification

Verified locally:

- `node -v`
- `npm -v`
- Rust stable installed through rustup and verified with `rustc --version`
- Cargo verified with `cargo --version`
- `npm install`
- `npm run build`
- `npm run tauri build`
- `npm run tauri dev` compiled and launched the native debug app
- Browser-served Vite shell mounted and exposed the left rail, library pane, workspace, and inspector

Current build output:

```text
src-tauri/target/release/bundle/macos/Librik.app
```

Note: rustup's shims have been linked through Homebrew so plain `rustc` and `cargo` commands are available in this environment.
