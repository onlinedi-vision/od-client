const sessionEnded = () => ({ code: 'session_expired', message: 'The audio session ended. Sign in again.', cleanupPending: false });

export function audioError(error) {
  if (error && typeof error.code === 'string' && typeof error.message === 'string') {
    return { code: error.code, message: error.message, cleanupPending: error.cleanupPending === true };
  }
  return { code: 'audio_unavailable', message: 'Audio settings are unavailable. Try Refresh.', cleanupPending: false };
}

// Shared across settings mounts; no account token or backend voice membership.
export function createAudioSession(invoke) {
  let login = null;
  let nextOwner = 0;

  async function openNative(activeLogin) {
    try {
      const id = await invoke('audio_open_session');
      if (activeLogin !== login) {
        await invoke('audio_close_session', { session: id });
        throw sessionEnded();
      }
      activeLogin.id = id;
      activeLogin.error = null;
      return id;
    } catch (error) {
      if (activeLogin !== login) throw sessionEnded();
      activeLogin.error = error;
      throw error;
    }
  }

  function ensureOpen(activeLogin) {
    if (activeLogin.opening) return activeLogin.opening;
    if (activeLogin.id) return Promise.resolve(activeLogin.id);
    activeLogin.opening = openNative(activeLogin).finally(() => { activeLogin.opening = null; });
    return activeLogin.opening;
  }

  function open() {
    login ??= { id: null, opening: null, error: null };
    return ensureOpen(login);
  }

  function close() {
    const id = login?.id;
    login = null;
    return id ? invoke('audio_close_session', { session: id }) : Promise.resolve();
  }

  function createPanelClient() {
    // A panel can retry opening the native session within its login only.
    const activeLogin = login;
    const owner = ++nextOwner;
    if (owner > 0xffffffff) throw sessionEnded();
    let intent = 0;
    let closed = false;

    function requireCurrent(closing) {
      if (!activeLogin || activeLogin !== login || (closed && !closing)) throw sessionEnded();
    }

    async function request(command, args = {}, { retryOpen = false, closing = false } = {}) {
      requireCurrent(closing);
      try {
        const session = await (retryOpen ? ensureOpen(activeLogin) : activeLogin.opening ?? activeLogin.id);
        requireCurrent(closing);
        if (!session) throw activeLogin.error ?? sessionEnded();
        return await invoke(command, { ...args, session });
      } finally {
        // Discard both results and errors from a closed panel or an old login.
        requireCurrent(closing);
      }
    }

    return {
      refresh: (retryOpen = false) => request('audio_refresh_devices', {}, { retryOpen }),
      select: (revision, direction, id) => request('audio_select_device', { revision, direction, id }),
      start(revision, inputId, mode = 'input', outputId) {
        const args = { owner, intent: ++intent, revision, inputId };
        if (mode !== 'input') Object.assign(args, { mode, outputId });
        return request('audio_capture_start', args);
      },
      stop: () => request('audio_capture_stop', { owner, intent: ++intent, close: false }),
      state: () => request('audio_capture_state', { owner }),
      close() {
        if (closed) return Promise.resolve();
        closed = true;
        return request('audio_capture_stop', { owner, intent: ++intent, close: true }, { closing: true });
      },
    };
  }

  return { open, close, createPanelClient };
}

// Coalesce UI refreshes. Native admits at most one OS scan, even after timeout.
export function createAudioDevices(onChange, client, timers = globalThis) {
  let state = { status: 'idle', data: null, working: false, error: null };
  let disposed = false;
  let pending = null;
  const publish = (patch) => {
    state = { ...state, ...patch };
    onChange({ ...state });
  };

  function run(action, loading) {
    if (disposed) return Promise.resolve();
    if (pending) return pending.promise;

    let expired = false;
    const operation = { deadline: undefined, promise: undefined };
    pending = operation;
    operation.promise = Promise.resolve().then(async () => {
      try {
        if (disposed) return;
        const data = await action();
        if (!disposed && !expired) publish({ data, status: 'ready', error: null });
      } catch (error) {
        if (!disposed && !expired) publish({ status: 'error', error: audioError(error) });
      } finally {
        timers.clearTimeout(operation.deadline);
        pending = null;
        if (!disposed) {
          publish({ working: false, error: expired ? {
            code: 'inventory_timeout', message: 'The device operation exceeded 5 seconds. It has finished; try Refresh.', cleanupPending: false,
          } : state.error });
        }
      }
    });
    operation.deadline = timers.setTimeout(() => {
      if (disposed) return;
      expired = true;
      publish({ status: 'error', error: {
        code: 'inventory_timeout',
        message: 'The device operation took too long. It is still finishing; Refresh will be available afterwards.',
        cleanupPending: true,
      } });
    }, 5000);
    publish({ working: true, error: null, status: loading ? 'loading' : state.status });
    return operation.promise;
  }

  return {
    getSnapshot: () => ({ ...state }),
    refresh: (retryOpen = false) => run(() => client.refresh(retryOpen), true),
    select(direction, id) {
      if (!state.data || state.status !== 'ready') return Promise.resolve();
      return run(() => client.select(state.data.revision, direction, id), false);
    },
    dispose() {
      disposed = true;
      timers.clearTimeout(pending?.deadline);
    },
  };
}
