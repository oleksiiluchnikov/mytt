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
    transition: background-color 0.3s ease, color 0.3s ease;
    display: inline-flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    min-width: 90px;
    height: 44px;
    height: 100%;
    white-space: normal;
    text-align: center;
    white-space: nowrap;
}

button:hover {
    background-color: var(--background-color);
    color: white;
}

.label {
    font-size: 12px;
    font-weight: 500;
}

.description {
    font-size: 10px;
    opacity: 0.8;
}
</style>
