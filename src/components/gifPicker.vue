<template>
    <div class="gif-overlay" @click.self="$emit('close')">
        <div class="gif-modal">
            <div class="gif-header"> <input v-model="query" @keyup.enter="search" placeholder="Search for a GIF..." />
                <button @click="search">Search</button> </div>
            <div v-if="error" style="color:red"> error </div>
            <div v-else-if="loading" style="color:pink"> Loading.. </div>
            <div v-else class="gif-grid"><img v-for="(g, i) in gifs" :key="i" class="gif-item" :src="g.preview":alt="g.alt" @click="select(g.url)" /></div>
            <div class="gif-footer"> <button @click="$emit('close')">Close</button> </div>
        </div>
    </div>
</template>
<script setup>
import { ref } from 'vue';

const emit = defineEmits(['select', 'close']);

const query = ref('');
const gifs = ref([]);
const loading = ref(false);
const error = ref('');
const API_KEY = import.meta.env.VITE_TENOR_API_KEY;

async function search() {
    if (!API_KEY) {
        error.value = 'API key not found';
        return;
    }
    loading.value = true;
    error.value = '';

    try {
        const endpoint = query.value.trim() ? 'search' : 'featured';
        const params = new URLSearchParams({
            key: API_KEY,
            limit: '15',
            media_filter: 'gif,tinygif'
        });

        if (endpoint === 'search')
            params.set('q', query.value.trim());

        const res = await fetch(`https://tenor.googleapis.com/v2/${endpoint}?${params.toString()}`);

        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        const data = await res.json();

        gifs.value = (data.results || []).map(r => ({
            url: r.media_formats?.gif?.url || r.media_formats?.tinygif?.url,
            preview: r.media_formats?.tinygif?.url || r.media_formats?.gif?.url,
            alt: r.content_description || 'gif'
        })).filter(x => x.url);

        console.log('GIFs found:', gifs.value.length);

    }
    catch (e) {
        error.value = "Gifs did not render corectly.";
    }
    finally {
        loading.value = false;
    }
}

function select(url) {
    emit('select', url);
}

</script>

<style scoped>
.gif-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 2000;
}

.gif-modal {
    width: 640px;
    max-width: 95vw;
    max-height: 80vh;
    background: #1e1e1e;
    border: 1px solid #333;
    border-radius: 10px;
    padding: 12px;
    display: flex;
    flex-direction: column;
}

.gif-header {
    display: flex;
    gap: 8px;
    margin-bottom: 10px;
}

.gif-footer {
    margin-top: 10px;
    display: flex;
    justify-content: flex-end;
}

.gif-grid {
    overflow: auto;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: 8px;
}

.gif-item {
    width: 100%;
    border-radius: 6px;
    cursor: pointer;
}
</style>
