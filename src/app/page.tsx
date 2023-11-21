'use client'

import { MouseEvent, ReactElement } from 'react';
import { FaBars, FaX } from 'react-icons/fa6';
import './App.css';
import { invoke } from '@tauri-apps/api'

class Game {
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

let SDKS: any;

fetch('https://ultreon.github.io/metadata/sdks.json')
  .then((res) => {
    SDKS = res.json();
    return undefined;
  })
  .catch(() => null);

function MenuButton() {
  function ToggleMenu(event: MouseEvent<HTMLButtonElement, any>): void {
    const elem = document.getElementById('SidePanel');
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
  function Close(event: MouseEvent<HTMLButtonElement, any>): void {
    invoke("close")
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
      <div className="TitleButtonsOther" data-tauri-drag-region>
        <MenuButton />
        <CloseButton />
      </div>
    </div>
  );
}

function TitleBar() {
  function Close(event: MouseEvent<HTMLButtonElement, any>): void {
    invoke("close")
  }

  return (
    <div className="TitleBar" data-tauri-drag-region>
      <TitleButtonsOther />
    </div>
  );
}

function PlayButton() {
  function Launch(event: MouseEvent<HTMLButtonElement>): void {
    const elem = event.target as HTMLButtonElement;
    if (elem.classList.contains('Disabled')) return;
    invoke("launch")
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
  return (
    <div>
      <div id="SidePanel">
        {GAMES.map((game) => GameEntry(game))}
      </div>
    </div>
  );
}

function Hello() {
  return (
    <div>
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
    <><TitleBar /><Hello /></>
  );
}
