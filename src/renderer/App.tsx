import { MemoryRouter as Router, Routes, Route } from 'react-router-dom';
import { MouseEvent, ReactElement } from 'react';
import { FaBars, FaX } from 'react-icons/fa6';
import './App.css';
import { Game } from '../main/preload';

function MenuButton() {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  function ToggleMenu(event: MouseEvent<HTMLButtonElement, any>): void {
    const elem = window.document.getElementById('SidePanel');
    if (elem !== null) {
      if (elem.classList.contains('Opened')) {
        elem.classList.remove('Opened');
      } else {
        elem.classList.add('Opened');
      }
    }
  }

  return (
    // eslint-disable-next-line jsx-a11y/control-has-associated-label
    <button className="TitleButton Begin" onClick={ToggleMenu} type="button">
      <FaBars />
    </button>
  );
}

function CloseButton() {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  function Close(event: MouseEvent<HTMLButtonElement, any>): void {
    window.electron.ipcRenderer.sendMessage('close', []);
  }

  return (
    // eslint-disable-next-line jsx-a11y/control-has-associated-label
    <button className="TitleButton End" onClick={Close} type="button">
      <FaX />
    </button>
  );
}

function TitleButtonsOther() {
  return (
    <div>
      <div className="TitleButtonsOther">
        <MenuButton />
        <CloseButton />
      </div>
    </div>
  );
}

function TitleBar() {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  function Close(event: MouseEvent<HTMLButtonElement, any>): void {
    window.electron.ipcRenderer.sendMessage('launch', []);
  }

  return (
    <div>
      <div className="TitleBar">
        <TitleButtonsOther />
      </div>
    </div>
  );
}

function PlayButton() {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  function Launch(event: MouseEvent<HTMLButtonElement>): void {
    const elem = event.target as HTMLButtonElement;
    if (elem.classList.contains('Disabled')) return;
    window.electron.ipcRenderer.sendMessage('launch', []);
  }

  return (
    <div>
      <button
        id="PlayButton"
        className="Button Disabled"
        onClick={Launch}
        type="button"
      >
        Play
      </button>
    </div>
  );
}

function BottomPanel() {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  function Launch(event: MouseEvent<HTMLButtonElement, any>): void {
    window.electron.ipcRenderer.sendMessage('launch', []);
  }

  return (
    <div>
      <div className="BottomPanel">
        <PlayButton />
      </div>
    </div>
  );
}

function GameEntry(element: Game): ReactElement {
  const { name } = element;

  return (
    <button className="GameEntry" type="button">
      {name}
    </button>
  );
}

function SidePanel() {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  function Launch(event: MouseEvent<HTMLButtonElement, any>): void {
    window.electron.ipcRenderer.sendMessage('launch', []);
  }

  return (
    <div>
      <div id="SidePanel">
        {window.launcher.games.map((game) => GameEntry(game))}
      </div>
    </div>
  );
}

function Hello() {
  return (
    <div>
      <TitleBar />
      <SidePanel />
      <h1>Ultreon Game Launcher WIP</h1>
      <div className="Hello">
        <a href="https://ultreon.github.io/" target="_blank" rel="noreferrer">
          <button className="Button" type="button">
            Website
          </button>
        </a>
      </div>
      <BottomPanel />
    </div>
  );
}

export default function App() {
  return (
    <Router>
      <Routes>
        <Route path="/" element={<Hello />} />
      </Routes>
    </Router>
  );
}
