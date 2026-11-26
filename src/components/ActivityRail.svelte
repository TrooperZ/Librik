<script module lang="ts">
    export type ActivityView =
        | "library"
        | "search"
        | "tags"
        | "recent"
        | "settings";
</script>

<script lang="ts">
    import { Clock3, Library, Search, Settings, Tag } from "@lucide/svelte";

    type ActivityView = import("./ActivityRail.svelte").ActivityView;

    export let activeView: ActivityView;
    export let panelOpen: boolean;
    export let onSelect: (view: ActivityView) => void;

    const primaryItems: Array<{
        id: ActivityView;
        label: string;
        icon: typeof Library;
    }> = [
        { id: "library", label: "Library", icon: Library },
        { id: "search", label: "Search", icon: Search },
        { id: "tags", label: "Tags", icon: Tag },
        { id: "recent", label: "Recent", icon: Clock3 },
    ];
</script>

<aside class="activity-rail" aria-label="Primary activity">
    <div class="activity-group">
        {#each primaryItems as item}
            <button
                type="button"
                class:active={panelOpen && activeView === item.id}
                title={panelOpen && activeView === item.id ? `Hide ${item.label}` : `Show ${item.label}`}
                aria-label={item.label}
                aria-pressed={panelOpen && activeView === item.id}
                onclick={() => onSelect(item.id)}
            >
                <item.icon size={20} strokeWidth={1.8} />
            </button>
        {/each}
    </div>

    <button
        type="button"
        class:active={panelOpen && activeView === "settings"}
        title={panelOpen && activeView === "settings" ? "Hide Settings" : "Show Settings"}
        aria-label="Settings"
        aria-pressed={panelOpen && activeView === "settings"}
        onclick={() => onSelect("settings")}
    >
        <Settings size={20} strokeWidth={1.8} />
    </button>
</aside>
