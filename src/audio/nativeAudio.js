import { invoke } from '@tauri-apps/api/core';
import { createAudioSession } from './audioDevices.js';

export const audioSession = createAudioSession((command, args) => invoke(`plugin:audio|${command}`, args));
