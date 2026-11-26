<script lang="ts">
    import { invoke, isTauri } from "@tauri-apps/api/core";
    import { ChevronLeft, ChevronRight } from "@lucide/svelte";
    import { onMount } from "svelte";
    import ActivityRail from "./components/ActivityRail.svelte";
    import type { ActivityView } from "./components/ActivityRail.svelte";
    import LibraryView from "./components/LibraryView.svelte";
    import PdfDetailsPanel from "./components/PdfDetailsPanel.svelte";
    import ReaderView from "./components/ReaderView.svelte";
    import SourceSidebar from "./components/SourceSidebar.svelte";
    import TopBar from "./components/TopBar.svelte";
    import WorkspaceTabs from "./components/WorkspaceTabs.svelte";
    import type { BookRow, FolderRow, WorkspaceTab } from "./lib/types";

    const desktop = isTauri();
    let selectedBookId = "";
    let activeActivity: ActivityView = "library";
    let selectedFilter = "all";
    let selectedTag = "";
    let selectedFolder = "";
    let searchQuery = "";
    let theme: "light" | "dark" = "light";
    let books: BookRow[] = [];
    let folders: FolderRow[] = [];
    let workspaceTabs: WorkspaceTab[] = [
        { id: "library", kind: "library", title: "Library", closable: false },
    ];
    let activeTabId = "library";
    let leftPanelOpen = true;
    let rightPanelOpen = true;

    $: selectedBook = books.find((book) => book.id === selectedBookId) ?? books[0];
    $: normalizedQuery = searchQuery.trim().toLowerCase();
    $: filteredBooks = books.filter((book) => {
        const matchesSearch =
            !normalizedQuery ||
            [book.title, book.creator, book.description, ...book.tags]
                .join(" ")
                .toLowerCase()
                .includes(normalizedQuery);
        const matchesTag = !selectedTag || book.tags.includes(selectedTag);
        const matchesFolder = !selectedFolder || book.folderId === selectedFolder;
        const matchesFilter =
            selectedFilter === "all" ||
            (selectedFilter === "missing"
                ? book.status === "Missing file"
                : book.status.toLowerCase() === selectedFilter);
        return matchesSearch && matchesTag && matchesFolder && matchesFilter;
    });
    $: libraryTitle = selectedFolder
        ? (folders.find((folder) => folder.id === selectedFolder)?.name ?? "Folder")
        : selectedTag
        ? `Tag: ${selectedTag}`
        : searchQuery.trim()
          ? `Search: ${searchQuery.trim()}`
          : selectedFilter === "all"
            ? "Library"
            : selectedFilter[0].toUpperCase() + selectedFilter.slice(1);
    $: if (typeof document !== "undefined") {
        document.documentElement.dataset.theme = theme;
    }

    onMount(() => {
        const savedTheme = window.localStorage.getItem("librik-theme");
        theme =
            savedTheme === "light" || savedTheme === "dark"
                ? savedTheme
                : window.matchMedia("(prefers-color-scheme: dark)").matches
                  ? "dark"
                  : "light";
        if (desktop) void loadLibrary();
    });

    function toggleTheme() {
        theme = theme === "light" ? "dark" : "light";
        window.localStorage.setItem("librik-theme", theme);
    }

    async function loadLibrary() {
        try {
            [books, folders] = await Promise.all([
                invoke<BookRow[]>("list_books"),
                invoke<FolderRow[]>("list_folders"),
            ]);
            selectedBookId = books[0]?.id ?? "";
        } catch (error) {
            showError("Could not load the library", error);
        }
    }

    async function importPdf() {
        if (!desktop) {
            window.alert("PDF import requires the desktop app. Run: npm run tauri dev");
            return;
        }
        try {
            const book = await invoke<BookRow | null>("import_pdf");
            if (!book) return;
            books = [book, ...books.filter((item) => item.id !== book.id)];
            folders = await invoke<FolderRow[]>("list_folders");
            selectedBookId = book.id;
            openReaderTab(book.id);
        } catch (error) {
            showError("Could not import the PDF", error);
        }
    }

    function showError(message: string, error: unknown) {
        window.alert(`${message}: ${String(error)}`);
    }

    function openReaderTab(bookId: string) {
        const book = books.find((item) => item.id === bookId);
        if (!book) return;
        const tabId = `reader:${bookId}`;
        if (!workspaceTabs.some((tab) => tab.id === tabId)) {
            workspaceTabs = [
                ...workspaceTabs,
                { id: tabId, kind: "reader", title: book.title, bookId, closable: true },
            ];
        }
        selectedBookId = bookId;
        activeTabId = tabId;
        rightPanelOpen = false;
    }

    function showLibrary() {
        activeTabId = "library";
        rightPanelOpen = true;
    }

    function handleSelectTab(tabId: string) {
        const tab = workspaceTabs.find((item) => item.id === tabId);
        if (!tab) return;
        activeTabId = tab.id;
        rightPanelOpen = tabId === "library";
        if (tab.kind === "reader") selectedBookId = tab.bookId;
    }

    function handleCloseTab(tabId: string) {
        const tabIndex = workspaceTabs.findIndex((tab) => tab.id === tabId);
        const tab = workspaceTabs[tabIndex];
        if (!tab || !tab.closable) return;
        const nextTabs = workspaceTabs.filter((item) => item.id !== tabId);
        workspaceTabs = nextTabs;
        if (activeTabId !== tabId) return;
        const nextTab = nextTabs[Math.max(0, tabIndex - 1)] ?? nextTabs[0];
        activeTabId = nextTab.id;
        rightPanelOpen = nextTab.id === "library";
        if (nextTab.kind === "reader") selectedBookId = nextTab.bookId;
    }

    function handleReorderTabs(fromTabId: string, toTabId: string) {
        if (fromTabId === toTabId) return;
        const fromIndex = workspaceTabs.findIndex((tab) => tab.id === fromTabId);
        const toIndex = workspaceTabs.findIndex((tab) => tab.id === toTabId);
        if (fromIndex === -1 || toIndex === -1) return;
        if (workspaceTabs[fromIndex].kind === "library" || workspaceTabs[toIndex].kind === "library") return;
        const nextTabs = [...workspaceTabs];
        const [movedTab] = nextTabs.splice(fromIndex, 1);
        nextTabs.splice(toIndex, 0, movedTab);
        workspaceTabs = nextTabs;
    }

    function handleActivitySelect(view: ActivityView) {
        if (activeActivity === view) {
            leftPanelOpen = !leftPanelOpen;
            return;
        }
        activeActivity = view;
        leftPanelOpen = true;
    }

    function handleFilter(filter: string) {
        selectedFilter = filter;
        selectedTag = "";
        selectedFolder = "";
        showLibrary();
    }

    function handleTag(tag: string) {
        selectedTag = tag;
        selectedFilter = "all";
        selectedFolder = "";
        showLibrary();
    }

    function handleFolder(folder: string) {
        selectedFolder = selectedFolder === folder ? "" : folder;
        selectedFilter = selectedFolder ? "" : "all";
        selectedTag = "";
        showLibrary();
    }

    async function createFolder(name: string, parentId: string | null) {
        if (!desktop) return null;
        try {
            const folder = await invoke<FolderRow>("create_folder", {
                name,
                parentId,
            });
            folders = [...folders, folder].sort((left, right) =>
                left.name.localeCompare(right.name),
            );
            handleFolder(folder.id);
            return folder;
        } catch (error) {
            showError("Could not create the folder", error);
            return null;
        }
    }

    async function renameFolder(id: string, name: string) {
        if (!desktop) return false;
        try {
            await invoke("rename_folder", { id, name });
            folders = folders.map((folder) =>
                folder.id === id ? { ...folder, name } : folder,
            );
            books = books.map((book) =>
                book.folderId === id ? { ...book, source: name } : book,
            );
            return true;
        } catch (error) {
            showError("Could not rename the folder", error);
            return false;
        }
    }

    async function moveFolder(
        id: string,
        parentId: string | null,
        beforeId: string | null,
    ) {
        if (!desktop) return false;
        try {
            await invoke("move_folder", { id, parentId, beforeId });
            folders = await invoke<FolderRow[]>("list_folders");
            return true;
        } catch (error) {
            showError("Could not move the folder", error);
            return false;
        }
    }

    async function deleteFolder(id: string) {
        if (!desktop) return false;
        try {
            await invoke("delete_folder", { id });
            [books, folders] = await Promise.all([
                invoke<BookRow[]>("list_books"),
                invoke<FolderRow[]>("list_folders"),
            ]);
            if (!folders.some((folder) => folder.id === selectedFolder)) {
                selectedFolder = "";
                selectedFilter = "all";
            }
            return true;
        } catch (error) {
            showError("Could not delete the folder", error);
            return false;
        }
    }

    async function setFolderColor(id: string, color: string | null) {
        if (!desktop) return false;
        try {
            await invoke("set_folder_color", { id, color });
            folders = folders.map((folder) =>
                folder.id === id ? { ...folder, color } : folder,
            );
            return true;
        } catch (error) {
            showError("Could not color the folder", error);
            return false;
        }
    }

    function handleSearch(query: string) {
        searchQuery = query;
        showLibrary();
    }

    function updateBook(bookId: string, update: (book: BookRow) => BookRow) {
        books = books.map((book) => (book.id === bookId ? update(book) : book));
        const updatedBook = books.find((book) => book.id === bookId);
        workspaceTabs = workspaceTabs.map((tab) =>
            tab.kind === "reader" && tab.bookId === bookId && updatedBook
                ? { ...tab, title: updatedBook.title }
                : tab,
        );
        if (updatedBook && desktop) {
            void invoke("save_reading_state", {
                state: {
                    id: updatedBook.id,
                    status: updatedBook.status,
                    progress: updatedBook.progress,
                    tags: updatedBook.tags,
                    currentPage: updatedBook.currentPage,
                },
            }).catch((error) => showError("Could not save reading progress", error));
        }
    }

    function handleAddTag(bookId: string, tag: string) {
        updateBook(bookId, (book) =>
            book.tags.includes(tag) ? book : { ...book, tags: [...book.tags, tag] },
        );
    }

    function handleRemoveTag(bookId: string, tag: string) {
        updateBook(bookId, (book) => ({
            ...book,
            tags: book.tags.filter((value) => value !== tag),
        }));
    }

    function handleSetProgress(bookId: string, progress: string) {
        updateBook(bookId, (book) => ({
            ...book,
            progress,
            currentPage: Math.max(1, Math.round((Number.parseInt(progress) / 100) * book.pages)),
            status: progress === "100%" ? "Finished" : "Reading",
        }));
    }

    function handlePageChange(bookId: string, page: number) {
        updateBook(bookId, (book) => {
            const percent = Math.min(100, Math.round((page / Math.max(book.pages, 1)) * 100));
            return {
                ...book,
                currentPage: page,
                progress: `${percent}%`,
                status: page >= book.pages ? "Finished" : "Reading",
            };
        });
    }

    function handleSetStatus(bookId: string, status: string) {
        updateBook(bookId, (book) => ({ ...book, status }));
    }

    async function handleSetFolder(bookId: string, folderId: string) {
        if (!desktop) return;
        try {
            const updated = await invoke<BookRow>("set_document_folder", {
                id: bookId,
                folderId: folderId || null,
            });
            books = books.map((book) => (book.id === bookId ? updated : book));
            folders = await invoke<FolderRow[]>("list_folders");
        } catch (error) {
            showError("Could not move the PDF", error);
        }
    }
</script>

<main class="app-shell" aria-label="Librik">
    <TopBar
        {searchQuery}
        {theme}
        onSearch={handleSearch}
        onImport={importPdf}
        onSettings={() => handleActivitySelect("settings")}
        onToggleTheme={toggleTheme}
    />
    <ActivityRail activeView={activeActivity} panelOpen={leftPanelOpen} onSelect={handleActivitySelect} />
    {#if leftPanelOpen}
        <SourceSidebar
            {books}
            {folders}
            {activeActivity}
            {selectedFilter}
            {selectedTag}
            {selectedFolder}
            {searchQuery}
            onFilter={handleFilter}
            onSearch={handleSearch}
            onTag={handleTag}
            onFolder={handleFolder}
            onCreateFolder={createFolder}
            onRenameFolder={renameFolder}
            onMoveFolder={moveFolder}
            onDeleteFolder={deleteFolder}
            onSetFolderColor={setFolderColor}
            onOpenBook={openReaderTab}
        />
    {/if}

    <section class="workspace" aria-label="Primary workspace">
        <WorkspaceTabs
            tabs={workspaceTabs}
            {activeTabId}
            onSelect={handleSelectTab}
            onClose={handleCloseTab}
            onNew={importPdf}
            onReorder={handleReorderTabs}
        />
        {#each workspaceTabs as tab (tab.id)}
            {#if tab.kind === "library"}
                <div class="workspace-pane" hidden={activeTabId !== tab.id}>
                    <LibraryView
                        title={libraryTitle}
                        books={filteredBooks}
                        {selectedBookId}
                        onSelectBook={(bookId) => (selectedBookId = bookId)}
                        onOpenReader={openReaderTab}
                        onImport={importPdf}
                    />
                </div>
            {:else}
                {@const tabBook = books.find((book) => book.id === tab.bookId)}
                {#if tabBook}
                    <div class="workspace-pane" hidden={activeTabId !== tab.id}>
                        <ReaderView book={tabBook} onPageChange={handlePageChange} />
                    </div>
                {/if}
            {/if}
        {/each}
    </section>

    {#if rightPanelOpen}
        <div class="inspector-dock open">
            <button class="inspector-toggle" type="button" title="Hide details" aria-label="Hide details" aria-expanded="true" onclick={() => (rightPanelOpen = false)}>
                <ChevronRight size={16} strokeWidth={2} />
            </button>
            {#if selectedBook}
                <PdfDetailsPanel
                    book={selectedBook}
                    {folders}
                    onOpenReader={openReaderTab}
                    onAddTag={handleAddTag}
                    onRemoveTag={handleRemoveTag}
                    onSetProgress={handleSetProgress}
                    onSetStatus={handleSetStatus}
                    onSetFolder={handleSetFolder}
                />
            {:else}
                <aside class="pdf-inspector empty-inspector" aria-label="No PDF selected">
                    <strong>Your library is empty</strong>
                    <span>Import a PDF to get started.</span>
                    <button type="button" onclick={importPdf}>Import PDF</button>
                </aside>
            {/if}
        </div>
    {:else}
        <div class="inspector-dock collapsed">
            <button class="inspector-toggle" type="button" title="Show details" aria-label="Show details" aria-expanded="false" onclick={() => (rightPanelOpen = true)}>
                <ChevronLeft size={16} strokeWidth={2} />
            </button>
        </div>
    {/if}
</main>
