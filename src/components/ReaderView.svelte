<script lang="ts">
    import { convertFileSrc } from "@tauri-apps/api/core";
    import { Columns2, Square } from "@lucide/svelte";
    import { GlobalWorkerOptions, getDocument } from "pdfjs-dist";
    import type { PDFDocumentProxy, RenderTask } from "pdfjs-dist";
    import { onDestroy, tick } from "svelte";
    import type { BookRow } from "../lib/types";

    export let book: BookRow;
    export let onPageChange: (bookId: string, page: number) => void;

    GlobalWorkerOptions.workerSrc = new URL(
        "pdfjs-dist/build/pdf.worker.mjs",
        import.meta.url,
    ).toString();

    let firstCanvas: HTMLCanvasElement;
    let secondCanvas: HTMLCanvasElement;
    let pdfArea: HTMLDivElement;
    let document: PDFDocumentProxy | undefined;
    let renderTasks: RenderTask[] = [];
    let currentPath = "";
    let pageNumber = 1;
    let pageCount = 0;
    let pagesVisible: 1 | 2 = 1;
    let zoom = 1;
    let singlePageZoom = 1;
    let loading = true;
    let error = "";
    let loadGeneration = 0;
    let renderGeneration = 0;
    let wheelLocked = false;
    let wheelTimer: ReturnType<typeof setTimeout> | undefined;

    $: lastVisiblePage = Math.min(pageNumber + pagesVisible - 1, pageCount);
    $: if (book.filePath && book.filePath !== currentPath) {
        currentPath = book.filePath;
        void loadPdf(book.filePath, book.currentPage);
    }

    async function loadPdf(path: string, initialPage: number) {
        const generation = ++loadGeneration;
        loading = true;
        error = "";
        cancelRendering();
        await document?.destroy();
        document = undefined;

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
            document = loaded;
            pageCount = loaded.numPages;
            pageNumber = Math.min(Math.max(initialPage, 1), pageCount);
            await tick();
            await renderPages();
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
        if (!document || !firstCanvas) return;
        cancelRendering();
        const generation = renderGeneration;
        const targets: Array<[number, HTMLCanvasElement]> = [
            [pageNumber, firstCanvas],
        ];
        const secondPage = Math.min(pageNumber + 1, pageCount);
        if (pagesVisible === 2 && secondPage > pageNumber && secondCanvas) {
            targets.push([secondPage, secondCanvas]);
        }

        try {
            await Promise.all(
                targets.map(([number, canvas]) =>
                    renderCanvas(number, canvas, generation),
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
        if (!document) return;
        const page = await document.getPage(number);
        if (generation !== renderGeneration) return;
        const viewport = page.getViewport({ scale: zoom });
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

    async function goToPage(nextPage: number) {
        if (!document) return;
        const clampedPage = Math.min(Math.max(nextPage, 1), pageCount);
        if (clampedPage === pageNumber) return;
        pageNumber = clampedPage;
        error = "";
        await renderPages();
        onPageChange(book.id, pageNumber);
    }

    async function setZoom(nextZoom: number) {
        zoom = Math.min(Math.max(nextZoom, 0.25), 2);
        if (pagesVisible === 1) singlePageZoom = zoom;
        error = "";
        await renderPages();
    }

    async function setPagesVisible(next: 1 | 2) {
        if (next === pagesVisible) return;
        if (next === 2) {
            singlePageZoom = zoom;
            pagesVisible = 2;
            await tick();
            if (document && pdfArea) {
                const page = await document.getPage(pageNumber);
                const baseWidth = page.getViewport({ scale: 1 }).width;
                // ponytail: fit on selection; add resize observation only if live resizing matters.
                zoom = Math.max(
                    0.25,
                    Math.min(zoom, (pdfArea.clientWidth - 66) / (baseWidth * 2)),
                );
            }
        } else {
            pagesVisible = 1;
            zoom = singlePageZoom;
            await tick();
        }
        error = "";
        await renderPages();
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
        const nextPage =
            event.deltaY > 0
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
        return {
            destroy: () => node.removeEventListener("wheel", handleWheel),
        };
    }

    onDestroy(() => {
        loadGeneration += 1;
        clearTimeout(wheelTimer);
        cancelRendering();
        void document?.destroy();
    });
</script>

<section class="reader-view real-reader" aria-label={`Reading ${book.title}`}>
    <div class="reader-canvas">
        <header class="reader-toolstrip reader-navigation">
            <strong title={book.title}>{book.title}</strong>
            <div>
                <span class="page-layout-control" aria-label="Pages visible">
                    <button type="button" class:active={pagesVisible === 1} aria-pressed={pagesVisible === 1} title="Show one page" onclick={() => setPagesVisible(1)}>
                        <Square size={14} /><span>1</span>
                    </button>
                    <button type="button" class:active={pagesVisible === 2} aria-pressed={pagesVisible === 2} title="Show two pages" onclick={() => setPagesVisible(2)}>
                        <Columns2 size={14} /><span>2</span>
                    </button>
                </span>
                <button type="button" disabled={loading || pageNumber <= 1} onclick={() => goToPage(pageNumber - pagesVisible)}>Previous</button>
                <span>
                    {#if pagesVisible === 2 && lastVisiblePage > pageNumber}
                        Pages {pageNumber}–{lastVisiblePage} of {pageCount || book.pages}
                    {:else}
                        Page {pageNumber} of {pageCount || book.pages}
                    {/if}
                </span>
                <button type="button" disabled={loading || pageNumber + pagesVisible > pageCount} onclick={() => goToPage(pageNumber + pagesVisible)}>Next</button>
                <button type="button" disabled={loading || zoom <= 0.25} aria-label="Zoom out" onclick={() => setZoom(zoom - 0.1)}>−</button>
                <span>{Math.round(zoom * 100)}%</span>
                <button type="button" disabled={loading || zoom >= 2} aria-label="Zoom in" onclick={() => setZoom(zoom + 0.1)}>+</button>
            </div>
        </header>

        <div bind:this={pdfArea} use:pageWheel class="pdf-scroll">
            {#if loading}
                <p class="reader-status">Opening PDF…</p>
            {:else if error}
                <p class="reader-error">{error}</p>
            {/if}
            <div class:spread={pagesVisible === 2} class:hidden={loading || Boolean(error)} class="pdf-pages">
                <canvas bind:this={firstCanvas} class="pdf-canvas"></canvas>
                {#if pagesVisible === 2 && lastVisiblePage > pageNumber}
                    <canvas bind:this={secondCanvas} class="pdf-canvas"></canvas>
                {/if}
            </div>
        </div>
    </div>
</section>
