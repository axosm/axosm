<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import * as THREE from "three";

    let canvas: HTMLCanvasElement;
    let animationId: number;

    onMount(() => {
        // Scene
        const scene = new THREE.Scene();
        scene.background = new THREE.Color(0x1a1a2e);

        // Camera
        const camera = new THREE.PerspectiveCamera(
            75,
            canvas.clientWidth / canvas.clientHeight,
            0.1,
            1000,
        );
        camera.position.z = 3;

        // Renderer
        const renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
        renderer.setSize(canvas.clientWidth, canvas.clientHeight);
        renderer.setPixelRatio(window.devicePixelRatio);

        // Geometry + Material + Mesh
        const geometry = new THREE.BoxGeometry(1, 1, 1);
        const material = new THREE.MeshStandardMaterial({ color: 0x6c63ff });
        const cube = new THREE.Mesh(geometry, material);
        scene.add(cube);

        // Lights
        const ambientLight = new THREE.AmbientLight(0xffffff, 0.4);
        scene.add(ambientLight);

        const pointLight = new THREE.PointLight(0xffffff, 1);
        pointLight.position.set(5, 5, 5);
        scene.add(pointLight);

        // Resize handler
        const handleResize = () => {
            const width = canvas.clientWidth;
            const height = canvas.clientHeight;
            camera.aspect = width / height;
            camera.updateProjectionMatrix();
            renderer.setSize(width, height);
        };
        window.addEventListener("resize", handleResize);

        // Animation loop
        const animate = () => {
            animationId = requestAnimationFrame(animate);
            cube.rotation.x += 0.01;
            cube.rotation.y += 0.01;
            renderer.render(scene, camera);
        };
        animate();

        return () => {
            window.removeEventListener("resize", handleResize);
            cancelAnimationFrame(animationId);
            renderer.dispose();
        };
    });

    onDestroy(() => {
        cancelAnimationFrame(animationId);
    });
</script>

<canvas bind:this={canvas} />

<style>
    canvas {
        width: 100%;
        height: 100%;
        display: block;
    }
</style>
