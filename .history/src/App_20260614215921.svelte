<script lang="ts">
    import ActivityRail from "./components/ActivityRail.svelte";
    import type { ActivityView } from "./components/ActivityRail.svelte";
    import LibraryView from "./components/LibraryView.svelte";
    import PdfDetailsPanel from "./components/PdfDetailsPanel.svelte";
    import ReaderView from "./components/ReaderView.svelte";
    import SourceSidebar from "./components/SourceSidebar.svelte";
    import TopBar from "./components/TopBar.svelte";
    import WorkspaceTabs from "./components/WorkspaceTabs.svelte";
    import {
        bookRows,
        noteCards,
        readerMarks,
        sourceSections,
    } from "./lib/mockData";
    import type { AppAction, BookRow, WorkspaceTab } from "./lib/types";

    let selectedSource = "All PDFs";
    let selectedBookId = bookRows[0].id;
    let activeActivity: ActivityView = "library";
    let books: BookRow[] = bookRows;
    let workspaceTabs: WorkspaceTab[] = [
        { id: "library", kind: "library", title: "Library", closable: false },
        {
            id: `reader:${bookRows[0].id}`,
            kind: "reader",
            title: bookRows[0].title,
            bookId: bookRows[0].id,
            closable: true,
        },
        {
            id: `reader:${bookRows[3].id}`,
            kind: "reader",
            title: bookRows[3].title,
            bookId: bookRows[3].id,
            closable: true,
        },
    ];
    let activeTabId = "library";

    $: selectedBook =
        books.find((book) => book.id === selectedBookId) ?? books[0];
    $: activeTab =
        workspaceTabs.find((tab) => tab.id === activeTabId) ?? workspaceTabs[0];

    function createMockBook(): BookRow {
        const number =
            books.filter((book) => book.id.startsWith("mock-import")).length +
            1;
        return {
            id: `mock-import-${number}`,
            title: `Imported PDF ${number}`,
            creator: "Unknown Author",
            type: "PDF",
            status: "Queued",
            source: "Local",
            progress: "0%",
            pages: 186,
            fileSize: "6.4 MB",
            filePath: `~/Library/Librik/Inbox/imported-pdf-${number}.pdf`,
            description:
                "Newly added PDF awaiting review, tagging, and reading progress.",
            tags: ["inbox"],
            addedAt: "Today",
            modifiedAt: "Today",
            fingerprint: `sha256:mock...${String(number).padStart(4, "0")}`,
            currentPage: 1,
            accent: "#2563eb",
        };
    }

    function handleAction(action: AppAction, _detail?: string) {
        if (action === "Import PDF") {
            const book = createMockBook();
            books = [book, ...books];
            selectedBookId = book.id;
            openReaderTab(book.id);
        }
    }

    function makeReaderTab(book: BookRow): WorkspaceTab {
        return {
            id: `reader:${book.id}`,
            kind: "reader",
            title: book.title,
            bookId: book.id,
            closable: true,
        };
    }

    function openReaderTab(bookId: string) {
        const book = books.find((item) => item.id === bookId);
        if (!book) return;

        const tabId = `reader:${bookId}`;
        if (!workspaceTabs.some((tab) => tab.id === tabId)) {
            workspaceTabs = [...workspaceTabs, makeReaderTab(book)];
        }

        selectedBookId = bookId;
        activeTabId = tabId;
    }

    function handleSelectTab(tabId: string) {
        const tab = workspaceTabs.find((item) => item.id === tabId);
        if (!tab) return;

        activeTabId = tab.id;
        if (tab.kind === "reader") {
            selectedBookId = tab.bookId;
        }
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
        if (nextTab.kind === "reader") {
            selectedBookId = nextTab.bookId;
        }
    }

    function handleSourceSelect(source: string) {
        selectedSource = source;
    }

    function handleActivitySelect(view: ActivityView) {
        activeActivity = view;
        if (view === "library") selectedSource = "All PDFs";
        if (view === "search") selectedSource = "Tag filter";
        if (view === "tags") selectedSource = "Annotated";
        if (view === "recent") selectedSource = "Recently Opened";
    }

    function handleBookSelect(bookId: string) {
        selectedBookId = bookId;
    }

    function handleOpenReader(bookId: string) {
        openReaderTab(bookId);
        handleAction("Open reader");
    }

    function updateBook(bookId: string, update: (book: BookRow) => BookRow) {
        books = books.map((book) => (book.id === bookId ? update(book) : book));
        workspaceTabs = workspaceTabs.map((tab) => {
            if (tab.kind !== "reader" || tab.bookId !== bookId) return tab;
            const book = books.find((item) => item.id === bookId);
            return book ? { ...tab, title: book.title } : tab;
        });
    }

    function handleAddTag(bookId: string, tag: string) {
        updateBook(bookId, (book) => {
            if (book.tags.includes(tag)) return book;
            return { ...book, tags: [...book.tags, tag] };
        });
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
            currentPage: Math.max(
                1,
                Math.round((Number.parseInt(progress) / 100) * book.pages),
            ),
            status: progress === "100%" ? "Annotated" : "Reading",
        }));
    }

    function handleSetStatus(bookId: string, status: string) {
        updateBook(bookId, (book) => ({ ...book, status }));
    }
</script>

<main class="app-shell" aria-label="Librik">
    <TopBar onAction={handleAction} />

    <ActivityRail activeView={activeActivity} onSelect={handleActivitySelect} />

    <SourceSidebar
        sections={sourceSections}
        {selectedSource}
        tabs={workspaceTabs}
        {activeTabId}
        {activeActivity}
        onSelect={handleSourceSelect}
        onSelectTab={handleSelectTab}
        onCloseTab={handleCloseTab}
    />

    <section class="workspace" aria-label="Primary workspace">
        <WorkspaceTabs
            tabs={workspaceTabs}
            {activeTabId}
            onSelect={handleSelectTab}
            onClose={handleCloseTab}
            onNew={() => handleAction("Import PDF")}
        />

        {#if activeTab.kind === "library"}
            <LibraryView
                {books}
                {selectedBookId}
                onSelectBook={handleBookSelect}
                onOpenReader={handleOpenReader}
                onAction={handleAction}
            />
        {:else}
            <ReaderView
                book={selectedBook}
                marks={readerMarks}
                onAction={handleAction}
            />
        {/if}
    </section>

    <PdfDetailsPanel
        book={selectedBook}
        notes={noteCards}
        onAction={handleAction}
        onOpenReader={handleOpenReader}
        onAddTag={handleAddTag}
        onRemoveTag={handleRemoveTag}
        onSetProgress={handleSetProgress}
        onSetStatus={handleSetStatus}
    />
</main>
