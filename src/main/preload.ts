// Disable no-unused-vars, broken for spread args
/* eslint no-unused-vars: off */
import { contextBridge, ipcRenderer, IpcRendererEvent } from 'electron';
import { json } from 'stream/consumers';

export type Channels = 'ipc-example' | 'launch' | 'close';

export class Game {
  id: string;

  name: string;

  constructor(id: string, name: string = 'Unknown Game') {
    this.id = id;
    this.name = name;
  }
}

const GAMES = [
  new Game('ultracraft', 'Ultracraft'),
  new Game('bubble-blaster-je', 'Bubble Blaster Java'),
  new Game('bubble-blaster-py', 'Bubble Blaster Python'),
];

let SDKS: ;

fetch('https://ultreon.github.io/metadata/sdks.json')
  .then((res) => {
    SDKS = res.json();
    return undefined;
  })
  .catch(() => null);

const electronHandler = {
  ipcRenderer: {
    sendMessage(channel: Channels, ...args: unknown[]) {
      ipcRenderer.send(channel, ...args);
    },
    on(channel: Channels, func: (...args: unknown[]) => void) {
      const subscription = (_event: IpcRendererEvent, ...args: unknown[]) =>
        func(...args);
      ipcRenderer.on(channel, subscription);

      return () => {
        ipcRenderer.removeListener(channel, subscription);
      };
    },
    once(channel: Channels, func: (...args: unknown[]) => void) {
      ipcRenderer.once(channel, (_event, ...args) => func(...args));
    },
  },
};

const launcherHandler = {
  games: GAMES,
  sdks: SDKS,
};

contextBridge.exposeInMainWorld('electron', electronHandler);
contextBridge.exposeInMainWorld('launcher', launcherHandler);

export type ElectronHandler = typeof electronHandler;
export type LauncherHandler = typeof launcherHandler;
