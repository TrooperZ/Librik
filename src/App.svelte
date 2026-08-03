<script lang="ts">
    import { invoke, isTauri } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import ActivityRail from "./components/ActivityRail.svelte";
    import type { ActivityView } from "./components/ActivityRail.svelte";
    import LibraryView from "./components/LibraryView.svelte";
    import PdfDetailsPanel from "./components/PdfDetailsPanel.svelte";
    import ReaderView from "./components/ReaderView.svelte";
    import SourceSidebar from "./components/SourceSidebar.svelte";
    import TopBar from "./components/TopBar.svelte";
    import WorkspaceTabs from "./components/WorkspaceTabs.svelte";
    import type { BookRow, WorkspaceTab } from "./lib/types";

    const desktop = isTauri();
    let selectedBookId = "";
    let activeActivity: ActivityView = "library";
    let selectedFilter = "all";
    let selectedTag = "";
    let searchQuery = "";
    let theme: "light" | "dark" = "light";
    let books: BookRow[] = [];
    let workspaceTabs: WorkspaceTab[] = [
        { id: "library", kind: "library", title: "Library", closable: false },
    ];
    let activeTabId = "library";

    $: selectedBook = books.find((book) => book.id === selectedBookId) ?? books[0];
    $: activeTab = workspaceTabs.find((tab) => tab.id === activeTabId) ?? workspaceTabs[0];
    $: normalizedQuery = searchQuery.trim().toLowerCase();
    $: filteredBooks = books.filter((book) => {
        const matchesSearch =
            !normalizedQuery ||
            [book.title, book.creator, book.description, ...book.tags]
                .join(" ")
                .toLowerCase()
                .includes(normalizedQuery);
        const matchesTag = !selectedTag || book.tags.includes(selectedTag);
        const matchesFilter =
            selectedFilter === "all" ||
            (selectedFilter === "missing"
                ? book.status === "Missing file"
                : book.status.toLowerCase() === selectedFilter);
        return matchesSearch && matchesTag && matchesFilter;
    });
    $: libraryTitle = selectedTag
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
        if (desktop) void loadBooks();
    });

    function toggleTheme() {
        theme = theme === "light" ? "dark" : "light";
        window.localStorage.setItem("librik-theme", theme);
    }

    async function loadBooks() {
        try {
            books = await invoke<BookRow[]>("list_books");
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
    }

    function showLibrary() {
        activeTabId = "library";
    }

    function handleSelectTab(tabId: string) {
        const tab = workspaceTabs.find((item) => item.id === tabId);
        if (!tab) return;
        activeTabId = tab.id;
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
        activeActivity = view;
        showLibrary();
    }

    function handleFilter(filter: string) {
        selectedFilter = filter;
        selectedTag = "";
        showLibrary();
    }

    function handleTag(tag: string) {
        selectedTag = tag;
        selectedFilter = "all";
        showLibrary();
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
    <ActivityRail activeView={activeActivity} onSelect={handleActivitySelect} />
    <SourceSidebar
        {books}
        {activeActivity}
        {selectedFilter}
        {selectedTag}
        {searchQuery}
        onFilter={handleFilter}
        onSearch={handleSearch}
        onTag={handleTag}
        onOpenBook={openReaderTab}
    />

    <section class="workspace" aria-label="Primary workspace">
        <WorkspaceTabs
            tabs={workspaceTabs}
            {activeTabId}
            onSelect={handleSelectTab}
            onClose={handleCloseTab}
            onNew={importPdf}
            onReorder={handleReorderTabs}
        />
        {#if activeTab.kind === "library"}
            <LibraryView
                title={libraryTitle}
                books={filteredBooks}
                {selectedBookId}
                onSelectBook={(bookId) => (selectedBookId = bookId)}
                onOpenReader={openReaderTab}
                onImport={importPdf}
            />
        {:else if selectedBook}
            <ReaderView book={selectedBook} onPageChange={handlePageChange} />
        {/if}
    </section>

    {#if selectedBook}
        <PdfDetailsPanel
            book={selectedBook}
            onOpenReader={openReaderTab}
            onAddTag={handleAddTag}
            onRemoveTag={handleRemoveTag}
            onSetProgress={handleSetProgress}
            onSetStatus={handleSetStatus}
        />
    {:else}
        <aside class="pdf-inspector empty-inspector" aria-label="No PDF selected">
            <strong>Your library is empty</strong>
            <span>Import a PDF to get started.</span>
            <button type="button" onclick={importPdf}>Import PDF</button>
        </aside>
    {/if}
</main>
