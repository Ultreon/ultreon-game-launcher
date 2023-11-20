import { ElectronHandler, LauncherHandler } from '../main/preload';

declare global {
  // eslint-disable-next-line no-unused-vars
  interface Window {
    electron: ElectronHandler;
    launcher: LauncherHandler;
  }
}

export {};
