import { ChangeEvent, MouseEvent, ReactElement, useEffect, useState } from 'react';
import './App.css';
import { invoke } from '@tauri-apps/api'
import { FaBars, FaX } from 'react-icons/fa6';
import { GAMES, Game } from './Games';
import { PROFILES, Profile } from './Profiles';

let selectedGame: Game | null = null;

function MenuButton() {
  function ToggleMenu(): void {
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
    <button className="TitleButton Icon Begin" onClick={() => ToggleMenu()} type="button">
      <FaBars />
    </button>
  );
}

function CloseButton() {
  function Close(): void {
    invoke("close")
  }

  return (
    <button className="TitleButton Icon End" onClick={Close} type="button">
      <FaX />
    </button>
  );
}

function ImportButton(newItem: (profile: Profile) => void) {
  async function Import(): Promise<void> {
    const profile: Profile = await invoke("import") as Profile
    PROFILES.push(profile);
    console.log(PROFILES);

    newItem(profile)
  }

  return (
    <button className="TitleButton Begin" onClick={Import} type="button">
      Import
    </button>
  );
}

function TitleButtonsOther(newItem: (profile: Profile) => void) {
  return (
    <div>
      <div className="TitleButtonsOther" data-tauri-drag-region>
        <div className='TitleButtonGroup Begin'>
          <MenuButton />
          { ImportButton(newItem) }
        </div>
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

function TitleBar(newItem: (profile: Profile) => void) {
  return (
    <div className="TitleBar" data-tauri-drag-region>
      {TitleButtonsOther(newItem) }
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

function ProfileEntry(element: Profile): ReactElement {
  const { name } = element;

  function SelectProfile(event: MouseEvent<HTMLButtonElement, globalThis.MouseEvent>) {
    const elem = event.currentTarget
    const game = GAMES.find(value => value.id == elem.ariaLabel)
    selectedGame = game === undefined ? null : game;

    RevalidatePlayState(selectedGame)
  }

  return (
    <button className="ProfileEntry" aria-label={element.name} key={element.name} type="button" onClick={SelectProfile}>
      {name}
    </button>
  );
}

function Hello(list: ReactElement<HTMLDivElement>) {
  return (
    <div>
      <div>
        {list}
      </div>
      <h1>🚧</h1>
      <h1>Ultreon Game Launcher</h1>
      <div className="Hello">
        <a href="https://ultreon.github.io/" target="_blank" rel="noreferrer">
          <button className="Button" type="button">
            Website
          </button>
        </a>
      </div>
      {BottomPanel()}
    </div>
  );
}

export default function App() {
  const [items, setItems] = useState<Profile[]>([]);
  const [newItem, setNewItem] = useState<Profile | null>(null);

  useEffect(() => {
    if (newItem !== null) {
      setItems((prevItems) => {
        return [...prevItems, newItem];
      });
      setNewItem(null);
    }
  }, [newItem]);

  const LIST = (
    <div id="SidePanel">
      {items.filter((item, pos, self) => self.findIndex(it => item.name == it.name) == pos).map((game) => ProfileEntry(game))}
    </div>
  )

  return (
    <>{TitleBar(setNewItem)}{Hello(LIST)}</>
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

