<script lang="ts">
    import { appStore } from '../stores/app';
    import Button from './Button.svelte';

    $: isFirstSessionToday = $appStore.status === 'idle' && $appStore.phase === 'work';

    function start(duration: number) {
        appStore.intent({ type: 'start', durationS: duration });
    }
</script>
<!-- components/EnergyCheck.svelte -->
{#if isFirstSessionToday || $appStore.flowStreak === 0}
    <div class="energy-check">
        <h3>How's your energy?</h3>
        <div class="energy-buttons">
            <Button label="🔋 Charged" action="start" on:click={() => start(600)} />
            <Button label="😐 OK" action="start" on:click={() => start(300)} />
            <Button label="😴 Low" action="start" on:click={() => start(120)} />
        </div>
    </div>
{/if}

<style>
    .energy-check {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: var(--spacing-small);
        margin-top: var(--spacing-small);
    }

    .energy-buttons {
        display: grid;
        gap: var(--spacing-small);
        justify-content: center;
        grid-template-columns: repeat(auto-fit, minmax(90px, 1fr));
        max-width: 400px;
        width: 100%;
    }
</style>
