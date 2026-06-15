# Librik Codebase Explanation

Librik is currently a Tauri v2 desktop app shell with a Svelte + TypeScript + Vite frontend. The visible UI is shaped as a real library and reader workspace: source navigation, dense library table, reader canvas, notebook panel, and compact app chrome.

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
│   │   ├── LibraryView.svelte
│   │   ├── ReaderView.svelte
│   │   ├── SourceSidebar.svelte
│   │   ├── StatusBar.svelte
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
- Imports OpenPencil-derived mock data for sources, books, and reader marks.
- Tracks UI state:
  - active workspace view: library or reader.
  - selected source.
  - selected book row.

The markup defines the major application structure:

- `TopBar`: app title, workspace pill, global search, import/sync/settings/profile controls.
- `SourceSidebar`: source, collection, and smart filter navigation.
- `.workspace`: tabbed library/reader area.
- `LibraryView`: dense table-management screen.
- `ReaderView`: reader and annotation screen.
- `StatusBar`: current source, visible item count, selected title, and ready state.

The handlers keep the shell interactive without surfacing implementation labels in the UI.

### `src/components/TopBar.svelte`

Top application toolbar based on the OpenPencil mockup. It contains:

- `Librik` wordmark.
- `My Library` workspace pill.
- Global search field.
- Clickable icon buttons for import, sync, settings, and profile.

The controls are active UI elements and keep the top bar feeling like an application toolbar.

### `src/components/SourceSidebar.svelte`

Left source navigation pane. It renders source sections from `sourceSections`:

- Sources.
- Collections.
- Smart filters.

Buttons update the selected source state in `App.svelte`.

### `src/components/LibraryView.svelte`

Dense library management view matching the OpenPencil direction:

- Header and subtitle.
- List/shelf/filter/sort controls.
- Table header.
- Clickable book/PDF rows.
- Per-row `Read` and more-action controls.

Selecting a row updates the active title. Opening the reader switches the app to the reader workspace.

### `src/components/ReaderView.svelte`

Reader and annotation shell based on the second OpenPencil screen:

- Left rail for outline and quick marks.
- Reader toolbar for highlight, note, bookmark, notebook, and tags.
- PDF page surface with text lines, highlight area, bookmark ribbon, and margin note anchor.
- Bottom page/zoom/search control strip.
- Right notebook panel with notes, extracted quotes, reader tools, export notes, and copy citation actions.

The reader is a UI shell for the reading workflow.

### `src/components/StatusBar.svelte`

Bottom status bar. It shows the current source, item count, selected title, and ready state.

### `src/app.css`

This is the complete visual system for the shell. It uses a restrained neutral palette, compact spacing, clear borders, and fixed desktop-style panel layout.

Important layout rules:

- `.app-shell` is a CSS grid with top app bar, source sidebar, workspace, and status bar.
- `.source-sidebar` keeps library navigation compact and stable.
- `.library-table` presents the management view as a dense table.
- `.reader-view` splits outline, reader canvas, and notebook panels.
- `.pdf-page` provides the reader page surface.
- The media query hides the inspector below `1120px`, preserving a usable desktop shell at narrower widths.

## Frontend Bridge

### `src/lib/types.ts`

This file defines shared frontend TypeScript types:

- `ViewMode` constrains the workspace to `library` or `reader`.
- `AppAction` enumerates shell-level clickable actions.
- `SourceSection` and `SourceItem` describe the source sidebar.
- `BookRow` describes the dense table data.
- `ReaderMark` and `NoteCard` describe reader-side content.

Keeping these types separate prevents UI markup from becoming the source of truth for app contracts.

### `src/lib/mockData.ts`

OpenPencil-derived static shell data:

- Source sections and counts.
- Library table rows.
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
- Browser-served Vite shell mounted and exposed the left rail, library pane, workspace, inspector, and status bar

Current build output:

```text
src-tauri/target/release/bundle/macos/Librik.app
```

Note: rustup's shims have been linked through Homebrew so plain `rustc` and `cargo` commands are available in this environment.
