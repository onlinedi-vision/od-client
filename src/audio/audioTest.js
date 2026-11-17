import { audioError } from './audioDevices.js';

export const isAudioTestActive = state => ['starting', 'running', 'stopping'].includes(state.phase) || state.cleanupPending;
const timeout = () => ({ code: 'capture_timeout', message: 'The audio operation took too long. Stop and wait for cleanup before trying again.', cleanupPending: true });

// Only controls and one latest native level. One poll in flight, <=10 Hz,
// only while a test/cleanup is active; no sample/event queues or PCM in Vue.
export function createAudioTest(onChange, client, timers = globalThis) {
  let state = { mode: 'input', phase: 'idle', level: 0, cleanupPending: false, error: null, format: null, ended: null, overruns: 0 };
  let disposed = false;
  let generation = 0;
  let pollTimer;
  let deadline;
  let pollPending = false;
  let failure = null;
  let stopPromise = null;
  let resolveStop;
  const publish = patch => {
    state = { ...state, ...patch };
    if (!disposed) onChange({ ...state });
  };
  const finishStop = success => {
    if (resolveStop) resolveStop(success);
    resolveStop = null;
    stopPromise = null;
  };
  const clear = () => {
    timers.clearTimeout(pollTimer);
    timers.clearTimeout(deadline);
    pollTimer = undefined;
    deadline = undefined;
  };

  function apply(snapshot, operation) {
    if (disposed || operation !== generation) return;
    if (!snapshot || !['idle', 'starting', 'running', 'stopping', 'failed'].includes(snapshot.phase)) {
      failed({ code: 'capture_failed', message: 'The audio state could not be read.', cleanupPending: true }, operation);
      return;
    }
    const error = snapshot.error ? audioError(snapshot.error) : failure;
    // A read or Stop acknowledgement can still describe a native transition.
    // Once Stop is requested, only completed cleanup may end the stopping UI.
    const stopping = stopPromise && isAudioTestActive(snapshot);
    const phase = stopping ? 'stopping' : snapshot.phase === 'idle' && error ? 'failed' : snapshot.phase;
    publish({ ...snapshot, phase, error, cleanupPending: !!stopping || snapshot.cleanupPending,
      level: phase === 'running' ? Math.max(0, Math.min(100, snapshot.level || 0)) : 0 });
    if (!isAudioTestActive(state)) {
      clear();
      finishStop(true);
    } else {
      // Also bound the time since the last confirmed Running snapshot. A hung
      // status IPC must trigger Stop rather than leave Running visible forever.
      if (phase === 'running') arm(operation, true);
      schedule();
    }
  }

  function schedule() {
    if (disposed || pollTimer !== undefined || !isAudioTestActive(state)) return;
    pollTimer = timers.setTimeout(async () => {
      pollTimer = undefined;
      if (disposed || !isAudioTestActive(state)) return;
      if (pollPending) { schedule(); return; }
      const operation = generation;
      pollPending = true;
      try { apply(await client.state(), operation); }
      catch (error) { failed(error, operation); }
      finally { pollPending = false; if (operation === generation) schedule(); }
    }, 100);
  }

  function arm(operation, starting) {
    timers.clearTimeout(deadline);
    deadline = timers.setTimeout(() => {
      if (disposed || operation !== generation) return;
      if (starting) {
        failure = timeout();
        stop(false);
      } else {
        clear();
        publish({ phase: 'stopping', level: 0, cleanupPending: true, error: failure || timeout() });
        finishStop(false);
      }
    }, 5000);
  }

  function failed(error, operation) {
    if (disposed || operation !== generation) return;
    failure = audioError(error);
    // A failed read/IPC is not proof the OS microphone stopped. Send Stop
    // independently of an outstanding Start, and keep Start blocked if unknown.
    stop(false);
  }

  function stop(explicit = true) {
    if (disposed) return Promise.resolve(false);
    if (stopPromise) return stopPromise;
    if (explicit && !isAudioTestActive(state)) {
      if (state.error || state.ended) publish({ phase: 'idle', error: null, ended: null });
      return Promise.resolve(true);
    }
    if (explicit) failure = null;
    const operation = ++generation;
    clear();
    publish({ phase: 'stopping', level: 0, cleanupPending: true, error: failure });
    stopPromise = new Promise(resolve => { resolveStop = resolve; });
    const result = stopPromise;
    arm(operation, false);
    // Do not wait for Start or a stale status request to finish.
    Promise.resolve().then(() => disposed ? undefined : client.stop()).then(snapshot => apply(snapshot, operation)).catch(error => {
      if (disposed || operation !== generation) return;
      clear();
      publish({ phase: 'stopping', level: 0, cleanupPending: true, error: audioError(error) });
      finishStop(false);
    });
    return result;
  }

  return {
    getSnapshot: () => ({ ...state }),
    start(revision, inputId, mode = 'input', outputId) {
      if (disposed || isAudioTestActive(state)) return Promise.resolve();
      failure = null;
      const operation = ++generation;
      clear();
      publish({ mode, phase: 'starting', level: 0, error: null, format: null, outputFormat: null, ended: null, overruns: 0, playout: null });
      arm(operation, true);
      return Promise.resolve().then(() => {
        if (disposed || operation !== generation) return;
        return client.start(revision, inputId, mode, outputId);
      }).then(snapshot => apply(snapshot, operation)).catch(error => failed(error, operation));
    },
    stop,
    dispose() {
      if (disposed) return;
      disposed = true;
      generation++;
      clear();
      finishStop(false);
      // Includes pending Start/open; native owner/intent rejects late requests.
      client.close().catch(() => {});
    },
  };
}
