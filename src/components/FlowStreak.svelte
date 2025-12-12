<!-- components/FlowStreak.svelte -->
<script>
    import { appStore } from '../stores/app';
    import { onMount } from 'svelte';

    let prevStreak = 0;
    let celebrating = false;

    $: if ($appStore.flowStreak > prevStreak && $appStore.flowStreak >= 3) {
        celebrating = true;
        setTimeout(() => celebrating = false, 2000);
    }

    $: prevStreak = $appStore.flowStreak;
</script>

<div class="flow-streak" class:celebrating>
    {#if $appStore.flowStreak > 0}
        <div class="streak-display">
            <span class="flame">🔥</span>
            <span class="count">{$appStore.flowStreak}</span>
            {#if celebrating}
                <span class="celebration">🎉</span>
            {/if}
        </div>

        {#if $appStore.flowStreak >= 5}
            <p class="milestone">On fire! Keep it up!</p>
        {/if}
    {/if}
</div>

<style>
    .celebrating {
        animation: bounce 0.5s ease-in-out 3;
    }

    @keyframes bounce {
        0%, 100% { transform: scale(1); }
        50% { transform: scale(1.2); }
    }
</style>
