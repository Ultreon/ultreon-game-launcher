import { MemoryRouter as Router, Routes, Route } from 'react-router-dom';
import { MouseEvent } from 'react';
import { FaX } from 'react-icons/fa6';
import './App.css';

function CloseButton() {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  function Close(event: MouseEvent<HTMLButtonElement, any>): void {
    window.electron.ipcRenderer.sendMessage('close', []);
  }

  return (
    <div>
      <button className="TitleButton" onClick={Close} type="button">
        <FaX />
      </button>
    </div>
  );
}

function TitleButtonsOther() {
  return (
    <div>
      <div className="TitleButtonsOther">
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
  function Launch(event: MouseEvent<HTMLButtonElement, any>): void {
    window.electron.ipcRenderer.sendMessage('launch', []);
  }

  return (
    <div>
      <button className="PlayButton" onClick={Launch} type="button">
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

function Hello() {
  return (
    <div>
      <TitleBar />
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
