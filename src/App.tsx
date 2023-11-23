import { MouseEvent, ReactElement, useEffect, useState } from 'react';
import './App.css';
import { invoke } from '@tauri-apps/api'
import { FaBars, FaX } from 'react-icons/fa6';
import { PROFILES, Profile } from './Profiles';

let selectedProfile: Profile | null = null;

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

function ImportButton() {
  return (
    <button className="TitleButton Begin" onClick={showModal} type="button">
      Import
    </button>
  );
}

function TitleButtonsOther() {
  return (
    <div>
      <div className="TitleButtonsOther" data-tauri-drag-region>
        <div className='TitleButtonGroup Begin'>
          <MenuButton />
          { ImportButton() }
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

function TitleBar() {
  return (
    <div className="TitleBar" data-tauri-drag-region>
      {TitleButtonsOther() }
    </div>
  );
}

function PlayButton() {
  function Launch(event: MouseEvent<HTMLButtonElement>): void {
    const elem = event.target as HTMLButtonElement;
    if (elem.classList.contains('Disabled')) return;
    const PROF = selectedProfile
    if (PROF == null) return;
    invoke("launch", { profile: PROF })
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
    const game = PROFILES.find(value => value.name == elem.ariaLabel)
    selectedProfile = game === undefined ? null : game;

    RevalidatePlayState(selectedProfile)
  }

  return (
    <button className="ProfileEntry" aria-label={element.name} key={element.name} type="button" onClick={SelectProfile}>
      {name}
    </button>
  );
}

function Hello(list: ReactElement<HTMLDivElement>, modal: ReactElement<HTMLDivElement>) {
  return (
    <div>
      <div>
        {list}
      </div>
      {/* <h1>🚧</h1>
      <h1>Ultreon Game Launcher</h1>
      <div className="Hello">
        <a href="https://ultreon.github.io/" target="_blank" rel="noreferrer">
          <button className="Button" type="button">
            Website
          </button>
        </a>
      </div> */}
      <div>
        {modal}
      </div>
      {BottomPanel()}
    </div>
  );
}

function showModal() {
  const modal = document.getElementById("InputModalBG");
  modal?.classList.add("Shown");
}

function hideModal() {
  const modal = document.getElementById("InputModalBG");
  modal?.classList.remove("Shown");

  const elem = document.getElementById("InputModal") as HTMLDivElement;
  const inputElem = elem.getElementsByClassName("textInput")[0] as HTMLInputElement;
  inputElem.value = "";
}

export default function App() {
  const [items, setItems] = useState<Profile[]>(PROFILES);
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

  async function importProfile(name: string) {    
    const profile: Profile = await invoke("import", { name: name }) as Profile
    if (profile.game === 'error') {
      return;
    }
    PROFILES.push(profile);
    console.log(PROFILES);

    setNewItem(profile)
    hideModal();
  }

  const MODAL = (
    <div id="InputModalBG" className='ModalBackground'>
      <div id="InputModal" className='Modal'>
        <input type='text' className='textInput' />
        <div className='ButtonGroup'>
          <button type='button' onClick={() => hideModal()}>Cancel</button>
          <button type='button' onClick={() => submitProfileInput(importProfile)}>Import</button>
        </div>
      </div>
    </div>
  )

  return (
    <>{TitleBar()}{Hello(LIST, MODAL)}</>
  );
}
function RevalidatePlayState(selectedProfile: Profile | null) {
  const elem = document.getElementById("PlayButton");
  if (selectedProfile == null) {
    elem?.classList.add("Disabled")
  } else {
    console.log("Selected Profile: " + selectedProfile.name);
    elem?.classList.remove("Disabled")
  }
}

function submitProfileInput(importFunc: (name: string) => void): void {
  const elem = document.getElementById("InputModal") as HTMLDivElement;
  const inputElem = elem.getElementsByClassName("textInput")[0] as HTMLInputElement;
  const value = inputElem.value;
  if (value.trim() === '') {
    console.info('Empty name value!')
    return;
  }
  console.log("Import for: %s", value)
  importFunc(value);
}

