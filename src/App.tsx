import { MouseEvent, ReactElement } from 'react';
import './App.css';
import { invoke } from '@tauri-apps/api'
import { FaBars, FaX } from 'react-icons/fa6';
import { json } from 'node:stream/consumers';

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
  new Game('bubble-blaster-py', 'Bubble Blaster Python'),
  new Game('bubble-blaster-py', 'Bubble Blaster Python'),
  new Game('bubble-blaster-py', 'Bubble Blaster Python'),
  new Game('bubble-blaster-py', 'Bubble Blaster Python'),
  new Game('bubble-blaster-py', 'Bubble Blaster Python'),
  new Game('bubble-blaster-py', 'Bubble Blaster Python'),
  new Game('bubble-blaster-py', 'Bubble Blaster Python'),
  new Game('bubble-blaster-py', 'Bubble Blaster Python'),
  new Game('bubble-blaster-py', 'Bubble Blaster Python'),
  new Game('bubble-blaster-py', 'Bubble Blaster Python'),
  new Game('bubble-blaster-py', 'Bubble Blaster Python'),
  new Game('bubble-blaster-py', 'Bubble Blaster Python'),
  new Game('bubble-blaster-py', 'Bubble Blaster Python'),
  new Game('bubble-blaster-py', 'Bubble Blaster Python'),
  new Game('bubble-blaster-py', 'Bubble Blaster Python'),
];

let SDKS: any;

fetch('https://ultreon.github.io/metadata/sdks.json')
  .then((res) => {
    if (res.ok) {
      SDKS = res.json();
      return undefined;
    }
  })
  .catch(() => null);

var selectedGame: Game | null = null;

function MenuButton() {
  function ToggleMenu(event: MouseEvent<HTMLButtonElement, globalThis.MouseEvent>): void {
    const elem = document.getElementById("SidePanel");
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
    <button className="TitleButton Begin" onClick={event => ToggleMenu(event)} type="button">
      <FaBars />
    </button>
  );
}

function CloseButton() {
  function Close(_event: MouseEvent<HTMLButtonElement, any>): void {
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
        <TitleBarText />
        <CloseButton />
      </div>
    </div>
  );
}

function TitleBarText() {
  return (
    <div>
      <p className="TitleBarText" data-tauri-drag-region>
        Ultreon Game Launcher
      </p>
    </div>
  );
}

function TitleBar() {
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

  function SelectGame(event: MouseEvent<HTMLButtonElement, globalThis.MouseEvent>) {
    var elem = event.currentTarget
    const game = GAMES.find(value => value.id == elem.ariaLabel)
    selectedGame = game === undefined ? null : game;

    RevalidatePlayState(selectedGame)
  }

  return (
    <button className="GameEntry" aria-label={element.id} key={element.id} type="button" onClick={SelectGame}>
      {name}
    </button>
  );
}

function SidePanel() {
  return (
    <div>
      <div id="SidePanel">
        {
          GAMES.filter((item, pos, self) => self.findIndex(it => item.name == it.name) == pos ).map((game) => GameEntry(game))
        }
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
function RevalidatePlayState(selectedGame: Game | null) {
  const elem = document.getElementById("PlayButton");
  if (selectedGame == null) {
    elem?.classList.add("Disabled")
  } else {
    console.log("Selected game: " + selectedGame.id);
    elem?.classList.remove("Disabled")
  }
}

