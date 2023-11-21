let SDKS: any;

fetch('https://ultreon.github.io/metadata/sdks.json')
  .then((res) => {
    if (res.ok) {
      SDKS = res.json();
      return undefined;
    }
  })
  .catch(() => null);
