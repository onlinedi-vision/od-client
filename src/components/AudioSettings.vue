<template>
  <div class="audio-settings">
    <section class="audio-card" aria-labelledby="native-audio-title" :aria-busy="snapshot.working">
      <h3 id="native-audio-title">Audio devices</h3>
      <p>Selecting a device does not start your microphone or play sound.</p>
      <div class="audio-device-fields">
        <div v-for="direction in directions" :key="direction.key" class="audio-field">
          <label :for="`native-audio-${direction.key}`">{{ direction.label }} device</label>
          <select :id="`native-audio-${direction.key}`"
            :value="snapshot.data?.[`${direction.key}Id`] || ''"
            :disabled="!canSelect(direction.key)" aria-describedby="native-audio-status"
            @change="selectDevice(direction.key, $event.target.value)">
            <option v-if="!snapshot.data" value="">{{ snapshot.working ? 'Loading devices…' : 'Unavailable' }}</option>
            <option v-for="device in options(direction.key)" :key="device.id" :value="device.id"
              :disabled="!device.available">
              {{ device.label }}{{ device.currentDevice ? ` — ${device.currentDevice}` : '' }}{{ device.available ? '' : ' (unavailable)' }}
            </option>
          </select>
          <p v-if="snapshot.status === 'ready' && !options(direction.key).some(device => device.available)" class="audio-help">
            No {{ direction.key }} devices were reported. Check the connection and Refresh.
          </p>
        </div>
      </div>
      <p v-show="statusMessage" id="native-audio-status" role="status">{{ statusMessage }}</p>
      <p v-if="snapshot.error" class="audio-error" role="alert">{{ snapshot.error.message }}</p>
      <p v-if="snapshot.data?.backend === 'CoreAudio'" class="audio-help">
        macOS may request microphone access when you start an input test. If access is denied, check System Settings → Privacy &amp; Security → Microphone.
      </p>
      <button class="audio-button" type="button" :disabled="snapshot.working || changingDevices" @click="refreshDevices()">
        {{ snapshot.working ? 'Refreshing…' : 'Refresh devices' }}
      </button>
      <p class="audio-help">Selections are kept when you reopen this panel.</p>
    </section>

    <section class="audio-card" aria-labelledby="native-audio-test-title">
      <h3 id="native-audio-test-title">Local audio tests</h3>
      <p>Start a test explicitly. Microphone and loopback tests last up to 30 seconds; the output tone lasts up to 1 second. No recording files or network audio are created.</p>
      <div class="audio-actions">
        <button class="audio-button" type="button" :disabled="!canStart" @click="startTest('input')">Start microphone test</button>
        <button class="audio-button" type="button" :disabled="!canStartOutput" @click="startTest('output')">Play output tone (1 s)</button>
      </div>
      <p>Loopback plays your selected microphone through the selected output in real time. Use headphones to avoid feedback. Playback starts at a reduced level.</p>
      <label class="audio-headphones"><input v-model="headphones" type="checkbox"> I am using headphones for loopback</label>
      <button class="audio-button" type="button" :disabled="!canLoopback" @click="startTest('loopback')">Start loopback</button>
      <label for="native-audio-level">Input level</label>
      <meter id="native-audio-level" min="0" max="100" :value="test.level">{{ test.level }}%</meter>
      <p>{{ test.level }}%</p>
      <p v-show="testMessage" role="status">{{ testMessage }}</p>
      <p v-if="test.error" class="audio-error" role="alert">{{ test.error.message }}</p>
      <div class="audio-actions">
        <button class="audio-button" type="button" :disabled="!testActive" @click="stopTest()">Stop test</button>
      </div>
      <p class="audio-help">One test at a time. Changing a device, refreshing or closing this panel stops it. Your microphone and system volume settings stay the same.</p>
    </section>

  </div>
</template>

<script setup>
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { createAudioDevices } from '../audio/audioDevices.js';
import { audioSession } from '../audio/nativeAudio.js';
import { createAudioTest, isAudioTestActive } from '../audio/audioTest.js';

const snapshot = ref({ status: 'idle', data: null, working: false, error: null });
const test = ref({ phase: 'idle', level: 0, cleanupPending: false, error: null });
const panel = audioSession.createPanelClient();
const audioTest = createAudioTest(state => { test.value = state; }, panel);
const changingDevices = ref(false);
const headphones = ref(false);
const confirmation = ref(null);
const devices = createAudioDevices(state => { snapshot.value = state; }, panel);
const directions = [{ key: 'input', label: 'Input' }, { key: 'output', label: 'Output' }];
const options = (direction) => snapshot.value.data?.[`${direction}s`] || [];
const devicesReady = computed(() => snapshot.value.status === 'ready' && !snapshot.value.working && !changingDevices.value);
const canSelect = (direction) => devicesReady.value
  && options(direction).some(device => device.available);
const statusMessage = computed(() => {
  if (snapshot.value.working && snapshot.value.error) return 'The device operation is still finishing.';
  if (snapshot.value.working) return 'Updating audio devices…';
  if (snapshot.value.status === 'error') return 'Device settings could not be updated. Your previous selections are kept.';
  if (snapshot.value.status === 'ready') return confirmation.value === 'refresh' ? 'Device list refreshed.' : '';
  return 'Preparing audio settings…';
});

const testActive = computed(() => isAudioTestActive(test.value));
const canStartTest = direction => !testActive.value && devicesReady.value
  && options(direction).some(device => device.id === snapshot.value.data?.[`${direction}Id`] && device.available);
const canStart = computed(() => canStartTest('input'));
const canStartOutput = computed(() => canStartTest('output'));
const canLoopback = computed(() => canStart.value && canStartOutput.value && headphones.value);
const startTest = mode => {
  clearFeedback();
  return audioTest.start(snapshot.value.data.revision, snapshot.value.data.inputId, mode, snapshot.value.data.outputId);
};
const testMessage = computed(() => {
  if (test.value.phase === 'starting') return 'Starting audio test… Stop is available.';
  if (test.value.phase === 'running' && test.value.mode === 'output') return 'Playing the output test tone.';
  if (test.value.phase === 'running' && test.value.mode === 'loopback') return 'Loopback active. You can hear your microphone in the selected headphones.';
  if (test.value.phase === 'running') return 'Microphone active. Speak to check the level.';
  if (test.value.cleanupPending) return 'Stopping audio… please wait.';
  if (test.value.phase === 'failed') return 'The audio test could not complete.';
  if (test.value.ended === 'duration_limit') return test.value.mode === 'output' ? 'Output test finished.' : 'Test finished after 30 seconds. Audio stopped.';
  return confirmation.value === 'stop' ? 'Audio test finished.' : '';
});
let disposed = false;
let feedbackOperation = 0;
function clearFeedback() {
  confirmation.value = null;
  return ++feedbackOperation;
}
async function stopTest() {
  if (!testActive.value || disposed) return;
  const operation = clearFeedback();
  const stopped = await audioTest.stop();
  if (!disposed && operation === feedbackOperation && stopped && !test.value.error) confirmation.value = 'stop';
}
async function withStoppedTest(action) {
  if (changingDevices.value || disposed) return;
  changingDevices.value = true;
  const operation = clearFeedback();
  try {
    const stopped = await audioTest.stop();
    if (stopped && !disposed) await action(operation);
  } finally {
    if (!disposed) changingDevices.value = false;
  }
}
const refreshDevices = () => withStoppedTest(async operation => {
  await devices.refresh(true);
  if (!disposed && operation === feedbackOperation && snapshot.value.status === 'ready' && !snapshot.value.error) confirmation.value = 'refresh';
});
const selectDevice = (direction, id) => withStoppedTest(() => devices.select(direction, id));
// Refresh cancels any previous native test before admitting a device scan.
onMounted(() => withStoppedTest(() => devices.refresh()));
onUnmounted(() => { disposed = true; audioTest.dispose(); devices.dispose(); });
</script>

<style scoped>
.audio-settings, .audio-card, .audio-field { display: flex; flex-direction: column; gap: 12px; min-width: 0; }
.audio-settings { gap: 16px; }
.audio-card { padding: 20px; border: 1px solid var(--settings-text); border-radius: 16px; background: var(--settings-surface); }
.audio-card h3, .audio-card p { margin: 0; }
.audio-card p { line-height: 1.5; }
.audio-device-fields { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 16px; }
.audio-field label, .audio-help { font-size: 0.9rem; }
.audio-field select { width: 100%; min-width: 0; box-sizing: border-box; padding: 12px; border: 1px solid var(--settings-text); border-radius: 12px; background: var(--settings-app-bg); color: var(--settings-text); font: inherit; }
.audio-actions { display: flex; flex-wrap: wrap; gap: 10px; }
.audio-headphones { display: flex; align-items: center; gap: 10px; }
.audio-button { align-self: flex-start; min-height: 44px; padding: 10px 14px; border: 1px solid var(--settings-text); border-radius: 10px; background: var(--settings-surface); color: var(--settings-text); font: inherit; font-weight: 600; cursor: pointer; }
.audio-button:hover:enabled { opacity: 0.85; }
.audio-button:disabled, .audio-field select:disabled { opacity: 0.55; cursor: not-allowed; }
.audio-button:focus-visible, .audio-field select:focus-visible { outline: 3px solid var(--settings-text); outline-offset: 3px; }
.audio-error { color: var(--settings-danger-text); }
meter { width: 100%; height: 24px; }
@media (max-width: 768px) { .audio-device-fields { grid-template-columns: 1fr; } .audio-card { padding: 16px; } }
</style>
