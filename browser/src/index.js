import createAudio from './audio'
import createDraw from './draw'

let canvas = document.getElementById('c')

let N = 512
let M = 4
let textureData = new Uint8Array(N * M)

let audio = createAudio(N)

let B = (1 << 16) - 1
function updateTextureData (textureData, samples) {
  for (let i = 0; i < N; i++) {
    let x = i / (N - 1)
    let y = 0.5 + 0.5 * samples[i]

    x = (x * B) | 0
    y = (y * B) | 0

    let j = i * M
    textureData[j + 0] = x >> 8
    textureData[j + 1] = x & 0xFF
    textureData[j + 2] = y >> 8
    textureData[j + 3] = y & 0xFF
  }
}

let draw = createDraw(canvas, N, textureData)

let timer = null;
(function loop () {
  let samples = audio.getTimeSamples()
  updateTextureData(textureData, samples)

  draw(textureData)
  timer = window.requestAnimationFrame(loop)
  // timer = setTimeout(loop, 1000)
})()

// … the application entry module
// As it doesn’t export it can accept itself. A dispose handler can pass the application state on replacement.
if (module.hot) {
  // this module is hot reloadable
  module.hot.accept()

  module.hot.dispose(() => {
    window.cancelAnimationFrame(timer)
    // clearTimeout(timer)
    audio.getContext().close()
  })
}
