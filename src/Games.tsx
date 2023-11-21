export class Game {
  id: string;

  name: string;

  constructor(id: string, name: string = 'Unknown Game') {
    this.id = id;
    this.name = name;
  }
}

export const GAMES = [
  new Game('ultracraft', 'Ultracraft'),
  new Game('infinicraft', 'Infinicraft'),
  new Game('bubble-blaster', 'Bubble Blaster Java'),
  new Game('bubbles-pygdx', 'Bubble Blaster PyGDX'),
  new Game('bubbles-graalgdx', 'Bubble Blaster GraalGDX'),
  new Game('qplay-bubbles', 'Bubble Blaster Legacy'),
  new Game('bb-og', 'Bubble Blaster OG'),
  new Game('hangman-v5', 'Hangman v5'),
];
