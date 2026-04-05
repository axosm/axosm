<!-- <h1>Welcome to SvelteKit</h1>
<p>
    Visit <a href="https://svelte.dev/docs/kit">svelte.dev/docs/kit</a> to read the
    documentation
</p> -->
<script lang="ts">
    import ThreeScene from "./../lib/ThreeScene.svelte";
</script>

<main>
    <ThreeScene />
</main>

<style>
    main {
        width: 100vw;
        height: 100vh;
        margin: 0;
        padding: 0;
        overflow: hidden;
    }
</style>
