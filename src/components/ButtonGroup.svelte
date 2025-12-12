<script lang="ts">
    import { appStore } from '../stores/app';
    import Button from './Button.svelte';

    function send(type: string, payload: any = {}) {
        appStore.intent({ type, ...payload });
    }
</script>

<!-- src/components/ButtonGroup.svelte -->
<div class="buttons-container">
    {#if $appStore.status === 'idle'}
        <div class="duration-selector">
            <Button label="2m" action="start" on:click={() => send('start', { durationS: 120 })} />
            <Button label="5m" action="start"on:click={() => send('start', { durationS: 300 })} />
            <Button label="10m"action="start" on:click={() => send('start', { durationS: 600 })} />
            <Button label="Start" action="start" on:click={() => send('start')} />
        </div>

    {:else if $appStore.status === 'running'}
        <!-- Change 'Pause' to 'pause', 'Stop' to 'stop' -->
        <Button label="Pause" action="pause" on:click={() => send('pause')} />
        <Button label="Stop" action="stop" on:click={() => send('stop')} />

    {:else if $appStore.status === 'paused'}
        <!-- Change 'Resume' to 'resume' -->
        <Button label="Resume" action="resume" on:click={() => send('resume')} />
        <Button label="Stop" action="stop" on:click={() => send('stop')} />

    {:else if $appStore.status === 'ratingPrompt'}
        <div class="rating-buttons">
            <!-- Change 'Rate' to 'rate' -->
            <Button label="Flow" action="flow" on:click={() => send('rate', { rating: 'flow' })} />
            <Button label="Focused" action="focused" on:click={() => send('rate', { rating: 'focused' })} />
            <Button label="OK" action="ok" on:click={() => send('rate', { rating: 'ok' })} />
            <Button label="Distracted" action="distracted" on:click={() => send('rate', { rating: 'distracted' })} />
        </div>
{/if}
</div>

{#if $appStore.status === 'decision'}
    <div class="decision-container">
        <!-- Continue Work Button -->
        <Button
            label="Continue Work"
            description={`Next: ${Math.floor($appStore.nextWorkS / 60)}m`}
            variant="primary"
            action="continue"
            on:click={() => send('chooseBreak', { takeBreak: false })}
        />

        <!-- Break Button with Context -->
        {#if $appStore.breakSuggestion?.type === 'required'}
            <Button
                label="Take Break (Required)"
                description={`${Math.floor($appStore.breakSuggestion.duration / 60)}m rest needed`}
                variant="warning"
                action="break"
                on:click={() => send('chooseBreak', { takeBreak: true })}
            />
        {:else if $appStore.breakSuggestion?.type === 'suggested'}
            <Button
                label="Take Break"
                description={`${Math.floor($appStore.breakSuggestion.duration / 60)}m suggested`}
                variant="secondary"
                action="break"
                on:click={() => send('chooseBreak', { takeBreak: true })}
            />
        {:else}
            <Button
                label="Take Break (Optional)"
                description="If you need it"
                variant="ghost"
                action="break"
                on:click={() => send('chooseBreak', { takeBreak: true })}
            />
        {/if}

    </div>
{/if}

            <!-- Change 'ChooseBreak' to 'chooseBreak' -->

<style>
.buttons-container {
    display: flex;
    flex-direction: column;
    width: 100%;
    gap: var(--spacing-small);
    margin-top: var(--spacing-small);
}

.rating-buttons, .action-buttons {
    display: grid;
    gap: var(--spacing-small);
    justify-content: center;
    grid-template-columns: repeat(auto-fit, minmax(90px, 1fr));
    max-width: 400px;
    width: 100%;
}

/* Flow mode specific layout */
.rating-buttons {
    display: flex;
    flex-direction: column;
    align-items: stretch;
}

@media (min-width: 400px) {
    .action-buttons {
        grid-template-columns: repeat(4, minmax(90px, 1fr));
    }
}

@media (min-width: 600px) {
    .action-buttons {
        grid-template-columns: repeat(5, minmax(90px, 1fr));
    }
}
</style>
