if (new URL(location.href).searchParams.has('frame')) {
  document.title = 'Find subframe';
  document.body.replaceChildren(Object.assign(document.createElement('p'), {textContent: 'needle inside iframe'}));
} else {
  document.querySelector('#find-frame').src = new URL('./tab-sample.html?frame=1', location.href).href;
  let audio;
  document.querySelector('#tab-tone').onclick = async () => {
    if (audio) await audio.close();
    audio = new AudioContext();
    const oscillator = audio.createOscillator(); const gain = audio.createGain();
    gain.gain.value = 0.02; oscillator.connect(gain); gain.connect(audio.destination);
    oscillator.start(); await audio.resume();
  };
  document.querySelector('#tab-silence').onclick = async () => { if (audio) await audio.close(); audio = null; };
}
