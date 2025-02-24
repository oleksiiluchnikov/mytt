<script lang="ts">
export let action: string;
export let label: string;
export let description: string | undefined = undefined;
</script>

<button
    data-action={action} on:click>
    <span class="label">{label}</span>
    {#if description}
        <span class="description">{description}</span>
    {/if}
</button>

<style>
button {
    width: 100%;
    padding: var(--button-padding);
    border-radius: calc(var(--border-radius) / 2);
    border: none;
    background-color: var(--background-color-light);
    cursor: pointer;
    display: inline-flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    height: fit-content;
    white-space: normal;
    text-align: center;
    white-space: nowrap;
}

button:hover {
    background-color: var(--background-color-blur);
    color: white;
}

button:focus {
    background-color: var(--background-color-blur);
    color: white;
    outline: none;
}

button:focus-visible {
    outline: 2px solid var(--accent-color, #4a9eff);
    outline-offset: 2px;
}

button:active {
    background-color: var(--background-color-dark, #1a1a1a);
}

.label {
    font-size: var(--font-size-small);
    font-weight: 500;
}

.description {
    font-size: 10px;
    opacity: 0.8;
}
</style>
