<script lang="ts">
    import { convertFileSrc } from "@tauri-apps/api/core";
    import {
        ChevronLeft,
        ChevronRight,
        Columns2,
        Copy,
        FileText,
        Hand,
        Highlighter,
        Maximize2,
        MessageSquare,
        MousePointer2,
        PanelLeft,
        Pencil,
        RotateCcw,
        RotateCw,
        Rows2,
        Search,
        Square,
        Strikethrough,
        Trash2,
        Type,
        Underline,
        Undo2,
        Volume2,
        VolumeX,
        ZoomIn,
        ZoomOut,
    } from "@lucide/svelte";
    import { GlobalWorkerOptions, getDocument } from "pdfjs-dist";
    import type { PDFDocumentProxy, RenderTask } from "pdfjs-dist";
    import { onDestroy, tick } from "svelte";
    import type { BookRow } from "../lib/types";

    export let book: BookRow;
    export let onPageChange: (bookId: string, page: number) => void;

    type RibbonTab = "home" | "annotations" | "edit" | "pages" | "tools";
    type ReaderTool =
        | "select"
        | "hand"
        | "highlight"
        | "underline"
        | "strike"
        | "rectangle"
        | "ink"
        | "text"
        | "note"
        | "signature"
        | "stamp";
    type Point = { x: number; y: number };
    type Annotation = {
        id: string;
        page: number;
        kind: Exclude<ReaderTool, "select" | "hand">;
        x: number;
        y: number;
        width: number;
        height: number;
        text?: string;
        points?: Point[];
    };

    const ribbonTabs: Array<{ id: RibbonTab; label: string }> = [
        { id: "home", label: "Home" },
        { id: "annotations", label: "Annotations" },
        { id: "edit", label: "Edit" },
        { id: "pages", label: "Pages" },
        { id: "tools", label: "Tools" },
    ];

    GlobalWorkerOptions.workerSrc = new URL(
        "pdfjs-dist/build/pdf.worker.mjs",
        import.meta.url,
    ).toString();

    let pdfArea: HTMLDivElement;
    let pagesElement: HTMLDivElement;
    let pdfDocument: PDFDocumentProxy | undefined;
    let renderTasks: RenderTask[] = [];
    let currentPath = "";
    let pageNumber = 1;
    let pageCount = 0;
    let pagesVisible: 1 | 2 = 1;
    let zoom = 1;
    let singlePageZoom = 1;
    let rotation = 0;
    let loading = true;
    let error = "";
    let loadGeneration = 0;
    let renderGeneration = 0;
    let wheelLocked = false;
    let wheelTimer: ReturnType<typeof setTimeout> | undefined;
    let ribbon: RibbonTab = "home";
    let activeTool: ReaderTool = "select";
    let thumbnailsOpen = true;
    let annotations: Annotation[] = [];
    let annotationKey = "";
    let draft: Annotation | null = null;
    let startPoint: Point | null = null;
    let panning = false;
    let panStart = { x: 0, y: 0, left: 0, top: 0 };
    let speaking = false;
    let toolMessage = "";

    $: lastVisiblePage = Math.min(pageNumber + pagesVisible - 1, pageCount);
    $: visiblePages = Array.from(
        { length: Math.max(0, lastVisiblePage - pageNumber + 1) },
        (_, index) => pageNumber + index,
    );
    $: if (book.fingerprint !== annotationKey) {
        annotationKey = book.fingerprint;
        annotations = loadAnnotations();
    }
    $: if (book.filePath && book.filePath !== currentPath) {
        currentPath = book.filePath;
        void loadPdf(book.filePath, book.currentPage);
    }

    function loadAnnotations(): Annotation[] {
        try {
            return JSON.parse(localStorage.getItem(`librik:marks:${book.fingerprint}`) ?? "[]");
        } catch {
            return [];
        }
    }

    function saveAnnotations(next: Annotation[]) {
        annotations = next;
        localStorage.setItem(`librik:marks:${book.fingerprint}`, JSON.stringify(next));
    }

    async function loadPdf(path: string, initialPage: number) {
        const generation = ++loadGeneration;
        loading = true;
        error = "";
        cancelRendering();
        await pdfDocument?.destroy();
        pdfDocument = undefined;

        try {
            // ponytail: whole-file loading is enough until large PDFs show a measurable need for ranges.
            const task = getDocument({
                url: convertFileSrc(path),
                disableRange: true,
                disableStream: true,
            });
            const loaded = await task.promise;
            if (generation !== loadGeneration) {
                await loaded.destroy();
                return;
            }
            pdfDocument = loaded;
            pageCount = loaded.numPages;
            pageNumber = Math.min(Math.max(initialPage, 1), pageCount);
            await tick();
            await fitPageHeight();
        } catch (reason) {
            if (generation === loadGeneration) {
                error = `Could not open this PDF: ${String(reason)}`;
            }
        } finally {
            if (generation === loadGeneration) loading = false;
        }
    }

    function cancelRendering() {
        renderGeneration += 1;
        for (const task of renderTasks) task.cancel();
        renderTasks = [];
    }

    async function renderPages() {
        if (!pdfDocument || !pagesElement) return;
        cancelRendering();
        const generation = renderGeneration;
        const canvases = Array.from(
            pagesElement.querySelectorAll<HTMLCanvasElement>("canvas[data-page]"),
        );
        try {
            await Promise.all(
                canvases.map((canvas) =>
                    renderCanvas(Number(canvas.dataset.page), canvas, generation),
                ),
            );
        } catch (reason) {
            if (reason instanceof Error && reason.name === "RenderingCancelledException") return;
            if (generation === renderGeneration) {
                error = `Could not render page ${pageNumber}: ${String(reason)}`;
            }
        }
    }

    async function renderCanvas(
        number: number,
        canvas: HTMLCanvasElement,
        generation: number,
    ) {
        if (!pdfDocument) return;
        const page = await pdfDocument.getPage(number);
        if (generation !== renderGeneration) return;
        const viewport = page.getViewport({ scale: zoom, rotation });
        const ratio = window.devicePixelRatio || 1;
        const context = canvas.getContext("2d");
        if (!context) throw new Error("Canvas rendering is unavailable");

        canvas.width = Math.floor(viewport.width * ratio);
        canvas.height = Math.floor(viewport.height * ratio);
        canvas.style.width = `${Math.floor(viewport.width)}px`;
        canvas.style.height = `${Math.floor(viewport.height)}px`;
        const task = page.render({
            canvas,
            canvasContext: context,
            viewport,
            transform: ratio === 1 ? undefined : [ratio, 0, 0, ratio, 0, 0],
        });
        renderTasks.push(task);
        await task.promise;
    }

    function thumbnail(node: HTMLCanvasElement, number: number) {
        const observer = new IntersectionObserver(
            (entries) => {
                if (!entries.some((entry) => entry.isIntersecting)) return;
                observer.disconnect();
                void renderThumbnail(node, number);
            },
            { root: node.closest(".thumbnail-list"), rootMargin: "160px" },
        );
        observer.observe(node);
        return { destroy: () => observer.disconnect() };
    }

    async function renderThumbnail(canvas: HTMLCanvasElement, number: number) {
        const loaded = pdfDocument;
        if (!loaded) return;
        const page = await loaded.getPage(number);
        const base = page.getViewport({ scale: 1, rotation });
        const viewport = page.getViewport({ scale: 118 / base.width, rotation });
        const ratio = window.devicePixelRatio || 1;
        const context = canvas.getContext("2d");
        if (!context) return;
        canvas.width = Math.floor(viewport.width * ratio);
        canvas.height = Math.floor(viewport.height * ratio);
        canvas.style.width = `${Math.floor(viewport.width)}px`;
        canvas.style.height = `${Math.floor(viewport.height)}px`;
        await page.render({
            canvas,
            canvasContext: context,
            viewport,
            transform: ratio === 1 ? undefined : [ratio, 0, 0, ratio, 0, 0],
        }).promise;
    }

    async function goToPage(nextPage: number) {
        if (!pdfDocument) return;
        const clampedPage = Math.min(Math.max(nextPage, 1), pageCount);
        if (clampedPage === pageNumber) return;
        pageNumber = clampedPage;
        error = "";
        await tick();
        await renderPages();
        window.document
            .querySelector(`[data-thumbnail-page="${pageNumber}"]`)
            ?.scrollIntoView({ block: "nearest" });
        onPageChange(book.id, pageNumber);
    }

    async function setZoom(nextZoom: number) {
        zoom = Math.min(Math.max(nextZoom, 0.25), 3);
        if (pagesVisible === 1) singlePageZoom = zoom;
        error = "";
        await renderPages();
    }

    async function fitPageWidth() {
        if (!pdfDocument || !pdfArea) return;
        const page = await pdfDocument.getPage(pageNumber);
        const viewport = page.getViewport({ scale: 1, rotation });
        const available = pdfArea.clientWidth - 72;
        const totalWidth = viewport.width * pagesVisible + (pagesVisible - 1) * 12;
        await setZoom(available / totalWidth);
    }

    async function fitPageHeight() {
        if (!pdfDocument || !pdfArea) return;
        const page = await pdfDocument.getPage(pageNumber);
        const viewport = page.getViewport({ scale: 1, rotation });
        await setZoom((pdfArea.clientHeight - 48) / viewport.height);
    }

    async function setPagesVisible(next: 1 | 2) {
        if (next === pagesVisible) return;
        if (next === 2) singlePageZoom = zoom;
        pagesVisible = next;
        await tick();
        if (next === 2) await fitPageWidth();
        else {
            zoom = singlePageZoom;
            await renderPages();
        }
    }

    async function rotate(delta: number) {
        rotation = (rotation + delta + 360) % 360;
        await tick();
        await fitPageWidth();
    }

    function handleWheel(event: WheelEvent) {
        if (
            loading ||
            error ||
            Math.abs(event.deltaY) < 8 ||
            Math.abs(event.deltaY) < Math.abs(event.deltaX)
        ) return;
        event.preventDefault();
        if (wheelLocked) return;
        const nextPage = event.deltaY > 0
            ? pageNumber + pagesVisible
            : pageNumber - pagesVisible;
        if (nextPage < 1 || nextPage > pageCount) return;
        wheelLocked = true;
        void goToPage(nextPage);
        clearTimeout(wheelTimer);
        wheelTimer = setTimeout(() => (wheelLocked = false), 300);
    }

    function pageWheel(node: HTMLElement) {
        node.addEventListener("wheel", handleWheel, { passive: false });
        return { destroy: () => node.removeEventListener("wheel", handleWheel) };
    }

    function pointOnSurface(event: PointerEvent): Point {
        const bounds = (event.currentTarget as SVGElement).getBoundingClientRect();
        return {
            x: Math.min(Math.max((event.clientX - bounds.left) / bounds.width, 0), 1),
            y: Math.min(Math.max((event.clientY - bounds.top) / bounds.height, 0), 1),
        };
    }

    function addPlacedAnnotation(page: number, point: Point) {
        let text: string | null = null;
        if (activeTool === "text") text = window.prompt("Text to add:");
        if (activeTool === "note") text = window.prompt("Note:");
        if (activeTool === "signature") text = window.prompt("Signature:");
        if (activeTool === "stamp") text = window.prompt("Stamp text:", "APPROVED");
        if (!text?.trim()) return;
        const next: Annotation = {
            id: crypto.randomUUID(),
            page,
            kind: activeTool as Annotation["kind"],
            x: point.x,
            y: point.y,
            width: activeTool === "stamp" ? 0.24 : 0.3,
            height: activeTool === "signature" ? 0.07 : 0.05,
            text: text.trim(),
        };
        saveAnnotations([...annotations, next]);
    }

    function startAnnotation(event: PointerEvent, page: number) {
        if (event.button !== 0 || activeTool === "select" || activeTool === "hand") return;
        const point = pointOnSurface(event);
        if (["text", "note", "signature", "stamp"].includes(activeTool)) {
            addPlacedAnnotation(page, point);
            return;
        }
        (event.currentTarget as SVGElement).setPointerCapture(event.pointerId);
        startPoint = point;
        draft = {
            id: crypto.randomUUID(),
            page,
            kind: activeTool as Annotation["kind"],
            x: point.x,
            y: point.y,
            width: 0,
            height: 0,
            points: activeTool === "ink" ? [point] : undefined,
        };
    }

    function moveAnnotation(event: PointerEvent) {
        if (!draft || !startPoint) return;
        const point = pointOnSurface(event);
        if (draft.kind === "ink") {
            draft = { ...draft, points: [...(draft.points ?? []), point] };
            return;
        }
        draft = {
            ...draft,
            x: Math.min(startPoint.x, point.x),
            y: Math.min(startPoint.y, point.y),
            width: Math.abs(point.x - startPoint.x),
            height: Math.abs(point.y - startPoint.y),
        };
    }

    function finishAnnotation() {
        if (!draft) return;
        if (draft.kind === "ink" && (draft.points?.length ?? 0) < 2) {
            draft = null;
            startPoint = null;
            return;
        }
        const completed = {
            ...draft,
            width: draft.width || 0.22,
            height: draft.height || 0.025,
        };
        saveAnnotations([...annotations, completed]);
        draft = null;
        startPoint = null;
    }

    function annotationsFor(page: number) {
        return [
            ...annotations.filter((annotation) => annotation.page === page),
            ...(draft?.page === page ? [draft] : []),
        ];
    }

    function undoAnnotation() {
        for (let index = annotations.length - 1; index >= 0; index -= 1) {
            if (annotations[index].page === pageNumber) {
                saveAnnotations(annotations.filter((_, candidate) => candidate !== index));
                return;
            }
        }
    }

    function clearPageAnnotations() {
        if (
            annotations.some((annotation) => annotation.page === pageNumber) &&
            window.confirm(`Remove all annotations from page ${pageNumber}?`)
        ) saveAnnotations(annotations.filter((annotation) => annotation.page !== pageNumber));
    }

    function startPan(event: PointerEvent) {
        if (activeTool !== "hand" || event.button !== 0) return;
        panning = true;
        panStart = {
            x: event.clientX,
            y: event.clientY,
            left: pdfArea.scrollLeft,
            top: pdfArea.scrollTop,
        };
        pdfArea.setPointerCapture(event.pointerId);
    }

    function movePan(event: PointerEvent) {
        if (!panning) return;
        pdfArea.scrollLeft = panStart.left - (event.clientX - panStart.x);
        pdfArea.scrollTop = panStart.top - (event.clientY - panStart.y);
    }

    async function pageText(number = pageNumber) {
        if (!pdfDocument) return "";
        const page = await pdfDocument.getPage(number);
        const content = await page.getTextContent();
        return content.items
            .map((item) => ("str" in item ? item.str : ""))
            .join(" ")
            .replace(/\s+/g, " ")
            .trim();
    }

    async function copyPageText() {
        const text = await pageText();
        if (!text) return (toolMessage = "No selectable text on this page");
        await navigator.clipboard.writeText(text);
        toolMessage = `Copied text from page ${pageNumber}`;
    }

    async function findText() {
        const query = window.prompt("Find text in this PDF:")?.trim().toLowerCase();
        if (!query || !pdfDocument) return;
        toolMessage = `Searching for “${query}”…`;
        for (let number = 1; number <= pageCount; number += 1) {
            if ((await pageText(number)).toLowerCase().includes(query)) {
                await goToPage(number);
                toolMessage = `Found “${query}” on page ${number}`;
                return;
            }
        }
        toolMessage = `“${query}” was not found`;
    }

    async function toggleReadAloud() {
        if (speaking) {
            speechSynthesis.cancel();
            speaking = false;
            return;
        }
        const text = await pageText();
        if (!text) return (toolMessage = "No readable text on this page");
        const utterance = new SpeechSynthesisUtterance(text);
        utterance.onend = () => (speaking = false);
        utterance.onerror = () => (speaking = false);
        speaking = true;
        speechSynthesis.speak(utterance);
    }

    onDestroy(() => {
        loadGeneration += 1;
        clearTimeout(wheelTimer);
        cancelRendering();
        speechSynthesis.cancel();
        void pdfDocument?.destroy();
    });
</script>

<section class="reader-view real-reader" aria-label={`Reading ${book.title}`}>
    <nav class="reader-ribbon" aria-label="PDF tools">
        <div class="reader-ribbon-tabs">
            {#each ribbonTabs as tab}
                <button type="button" class:active={ribbon === tab.id} onclick={() => (ribbon = tab.id)}>{tab.label}</button>
            {/each}
        </div>

        <div class="reader-ribbon-tools">
            {#if ribbon === "home"}
                <div class="ribbon-group">
                    <button type="button" class:active={activeTool === "select"} onclick={() => (activeTool = "select")}><MousePointer2 size={19} /><span>Select</span></button>
                    <button type="button" class:active={activeTool === "hand"} onclick={() => (activeTool = "hand")}><Hand size={19} /><span>Hand</span></button>
                </div>
                <div class="ribbon-group">
                    <button type="button" onclick={() => (thumbnailsOpen = !thumbnailsOpen)}><PanelLeft size={19} /><span>Thumbnails</span></button>
                    <button type="button" onclick={() => setZoom(zoom - 0.1)}><ZoomOut size={19} /><span>Zoom out</span></button>
                    <button type="button" onclick={() => setZoom(zoom + 0.1)}><ZoomIn size={19} /><span>Zoom in</span></button>
                    <button type="button" onclick={fitPageWidth}><Maximize2 size={19} /><span>Fit width</span></button>
                    <button type="button" onclick={fitPageHeight}><Rows2 size={19} /><span>Fit height</span></button>
                </div>
                <div class="ribbon-group">
                    <button type="button" class:active={pagesVisible === 1} onclick={() => setPagesVisible(1)}><Square size={19} /><span>One page</span></button>
                    <button type="button" class:active={pagesVisible === 2} onclick={() => setPagesVisible(2)}><Columns2 size={19} /><span>Two pages</span></button>
                    <button type="button" onclick={() => rotate(-90)}><RotateCcw size={19} /><span>Rotate left</span></button>
                    <button type="button" onclick={() => rotate(90)}><RotateCw size={19} /><span>Rotate right</span></button>
                </div>
            {:else if ribbon === "annotations"}
                <div class="ribbon-group">
                    <button type="button" class:active={activeTool === "select"} onclick={() => (activeTool = "select")}><MousePointer2 size={19} /><span>Select</span></button>
                    <button type="button" class:active={activeTool === "hand"} onclick={() => (activeTool = "hand")}><Hand size={19} /><span>Hand</span></button>
                </div>
                <div class="ribbon-group">
                    <button type="button" class:active={activeTool === "highlight"} onclick={() => (activeTool = "highlight")}><Highlighter size={19} /><span>Highlight</span></button>
                    <button type="button" class:active={activeTool === "underline"} onclick={() => (activeTool = "underline")}><Underline size={19} /><span>Underline</span></button>
                    <button type="button" class:active={activeTool === "strike"} onclick={() => (activeTool = "strike")}><Strikethrough size={19} /><span>Strikeout</span></button>
                    <button type="button" class:active={activeTool === "rectangle"} onclick={() => (activeTool = "rectangle")}><Square size={19} /><span>Rectangle</span></button>
                    <button type="button" class:active={activeTool === "ink"} onclick={() => (activeTool = "ink")}><Pencil size={19} /><span>Ink</span></button>
                    <button type="button" class:active={activeTool === "text"} onclick={() => (activeTool = "text")}><Type size={19} /><span>Text</span></button>
                    <button type="button" class:active={activeTool === "note"} onclick={() => (activeTool = "note")}><MessageSquare size={19} /><span>Note</span></button>
                </div>
                <div class="ribbon-group">
                    <button type="button" onclick={undoAnnotation}><Undo2 size={19} /><span>Undo</span></button>
                    <button type="button" onclick={clearPageAnnotations}><Trash2 size={19} /><span>Clear page</span></button>
                </div>
            {:else if ribbon === "edit"}
                <div class="ribbon-group">
                    <button type="button" class:active={activeTool === "text"} onclick={() => (activeTool = "text")}><Type size={19} /><span>Add text</span></button>
                    <button type="button" class:active={activeTool === "note"} onclick={() => (activeTool = "note")}><MessageSquare size={19} /><span>Add note</span></button>
                    <button type="button" onclick={undoAnnotation}><Undo2 size={19} /><span>Undo</span></button>
                    <button type="button" onclick={clearPageAnnotations}><Trash2 size={19} /><span>Clear page</span></button>
                </div>
                <div class="ribbon-hint">Edits are stored with this PDF in Librik.</div>
            {:else if ribbon === "pages"}
                <div class="ribbon-group">
                    <button type="button" onclick={() => (thumbnailsOpen = !thumbnailsOpen)}><PanelLeft size={19} /><span>Thumbnails</span></button>
                    <button type="button" disabled={pageNumber <= 1} onclick={() => goToPage(pageNumber - pagesVisible)}><ChevronLeft size={19} /><span>Previous</span></button>
                    <button type="button" disabled={pageNumber + pagesVisible > pageCount} onclick={() => goToPage(pageNumber + pagesVisible)}><ChevronRight size={19} /><span>Next</span></button>
                </div>
                <div class="ribbon-group">
                    <button type="button" onclick={() => rotate(-90)}><RotateCcw size={19} /><span>Rotate left</span></button>
                    <button type="button" onclick={() => rotate(90)}><RotateCw size={19} /><span>Rotate right</span></button>
                    <button type="button" class:active={pagesVisible === 1} onclick={() => setPagesVisible(1)}><Rows2 size={19} /><span>Single</span></button>
                    <button type="button" class:active={pagesVisible === 2} onclick={() => setPagesVisible(2)}><Columns2 size={19} /><span>Spread</span></button>
                </div>
            {:else if ribbon === "tools"}
                <div class="ribbon-group">
                    <button type="button" onclick={findText}><Search size={19} /><span>Find</span></button>
                    <button type="button" onclick={copyPageText}><Copy size={19} /><span>Copy text</span></button>
                    <button type="button" class:active={speaking} onclick={toggleReadAloud}>{#if speaking}<VolumeX size={19} />{:else}<Volume2 size={19} />{/if}<span>{speaking ? "Stop" : "Read aloud"}</span></button>
                </div>
                <div class="ribbon-hint">{toolMessage || "Search, copy, or listen to the current PDF."}</div>
            {/if}
        </div>
    </nav>

    <div class:with-thumbnails={thumbnailsOpen} class="reader-stage">
        {#if thumbnailsOpen}
            <aside class="reader-thumbnails" aria-label="Page thumbnails">
                <header><FileText size={15} /><span>Thumbnails</span></header>
                <div class="thumbnail-list">
                    {#key `${currentPath}-${rotation}`}
                        {#each Array(pageCount) as _, index}
                            <button type="button" class:active={pageNumber === index + 1} data-thumbnail-page={index + 1} onclick={() => goToPage(index + 1)}>
                                <canvas use:thumbnail={index + 1} aria-label={`Page ${index + 1}`}></canvas>
                                <span>{index + 1}</span>
                            </button>
                        {/each}
                    {/key}
                </div>
            </aside>
        {/if}

        <div
            bind:this={pdfArea}
            use:pageWheel
            class:hand-tool={activeTool === "hand"}
            class:panning
            class="pdf-scroll"
            role="region"
            aria-label="PDF page area"
            onpointerdown={startPan}
            onpointermove={movePan}
            onpointerup={() => (panning = false)}
            onpointercancel={() => (panning = false)}
        >
            {#if loading}
                <p class="reader-status">Opening PDF…</p>
            {:else if error}
                <p class="reader-error">{error}</p>
            {/if}
            <div bind:this={pagesElement} class:spread={pagesVisible === 2} class:hidden={loading || Boolean(error)} class="pdf-pages">
                {#each visiblePages as number (number)}
                    <div class="pdf-page-surface">
                        <canvas class="pdf-canvas" data-page={number}></canvas>
                        <svg
                            class:drawing={!(["select", "hand"] as ReaderTool[]).includes(activeTool)}
                            class="annotation-layer"
                            viewBox="0 0 1000 1000"
                            preserveAspectRatio="none"
                            role="application"
                            aria-label={`Annotations on page ${number}`}
                            onpointerdown={(event) => startAnnotation(event, number)}
                            onpointermove={moveAnnotation}
                            onpointerup={finishAnnotation}
                            onpointercancel={() => {
                                draft = null;
                                startPoint = null;
                            }}
                        >
                            {#each annotationsFor(number) as annotation (annotation.id)}
                                {#if annotation.kind === "highlight"}
                                    <rect x={annotation.x * 1000} y={annotation.y * 1000} width={annotation.width * 1000} height={annotation.height * 1000} class="mark-highlight" />
                                {:else if annotation.kind === "underline"}
                                    <line x1={annotation.x * 1000} y1={(annotation.y + annotation.height) * 1000} x2={(annotation.x + annotation.width) * 1000} y2={(annotation.y + annotation.height) * 1000} class="mark-line" />
                                {:else if annotation.kind === "strike"}
                                    <line x1={annotation.x * 1000} y1={(annotation.y + annotation.height / 2) * 1000} x2={(annotation.x + annotation.width) * 1000} y2={(annotation.y + annotation.height / 2) * 1000} class="mark-line" />
                                {:else if annotation.kind === "rectangle"}
                                    <rect x={annotation.x * 1000} y={annotation.y * 1000} width={annotation.width * 1000} height={annotation.height * 1000} class="mark-rectangle" />
                                {:else if annotation.kind === "ink"}
                                    <polyline points={(annotation.points ?? []).map((point) => `${point.x * 1000},${point.y * 1000}`).join(" ")} class="mark-ink" />
                                {:else if annotation.kind === "text"}
                                    <text x={annotation.x * 1000} y={(annotation.y + annotation.height) * 1000} class="mark-text">{annotation.text}</text>
                                {:else if annotation.kind === "signature"}
                                    <text x={annotation.x * 1000} y={(annotation.y + annotation.height) * 1000} class="mark-signature">{annotation.text}</text>
                                {:else if annotation.kind === "stamp"}
                                    <g class="mark-stamp"><rect x={annotation.x * 1000} y={annotation.y * 1000} width={annotation.width * 1000} height={annotation.height * 1000} /><text x={(annotation.x + 0.015) * 1000} y={(annotation.y + annotation.height * 0.72) * 1000}>{annotation.text}</text></g>
                                {:else if annotation.kind === "note"}
                                    <g class="mark-note"><circle cx={annotation.x * 1000} cy={annotation.y * 1000} r="18" /><text x={annotation.x * 1000} y={annotation.y * 1000 + 7}>i</text><title>{annotation.text}</title></g>
                                {/if}
                            {/each}
                        </svg>
                    </div>
                {/each}
            </div>
        </div>
    </div>

    <footer class="reader-statusbar">
        <strong title={book.title}>{book.title}</strong>
        <div class="page-controls">
            <button type="button" disabled={pageNumber <= 1} aria-label="Previous page" onclick={() => goToPage(pageNumber - pagesVisible)}><ChevronLeft size={15} /></button>
            <input type="number" min="1" max={pageCount} value={pageNumber} aria-label="Current page" onchange={(event) => goToPage(Number(event.currentTarget.value))} />
            <span>/ {pageCount || book.pages}</span>
            <button type="button" disabled={pageNumber + pagesVisible > pageCount} aria-label="Next page" onclick={() => goToPage(pageNumber + pagesVisible)}><ChevronRight size={15} /></button>
        </div>
        <div class="zoom-controls">
            <button type="button" aria-label="Zoom out" onclick={() => setZoom(zoom - 0.1)}><ZoomOut size={15} /></button>
            <span>{Math.round(zoom * 100)}%</span>
            <button type="button" aria-label="Zoom in" onclick={() => setZoom(zoom + 0.1)}><ZoomIn size={15} /></button>
            <button type="button" title="Fit height" aria-label="Fit height" onclick={fitPageHeight}><Maximize2 size={15} /></button>
            <button type="button" class:active={pagesVisible === 1} title="One page" aria-label="One page" onclick={() => setPagesVisible(1)}><Square size={15} /></button>
            <button type="button" class:active={pagesVisible === 2} title="Two pages" aria-label="Two pages" onclick={() => setPagesVisible(2)}><Columns2 size={15} /></button>
        </div>
    </footer>
</section>
