// import createAudio from './audio'
import createDraw from './draw'

// let audio = createAudio()

let canvas = document.getElementById('c')

let N = 1024
let M = 4
let samples = new Uint8Array(N * M)

let t = 0
let B = (1 << 16) - 1
function updateSamples () {
  for (let i = 0; i < N; i++) {
    let j = i * M
    let x = Math.sin(2 * Math.PI * (i + 200 * t) / N * 3 / 2)
    let y = Math.cos(2 * Math.PI * (i + 200 * t) / N * (2 / 3 + t))
    x = Math.abs(x)
    y = Math.abs(y)

    x = (x * B) | 0
    y = (y * B) | 0
    samples[j + 0] = x >> 8
    samples[j + 1] = x & 0xFF
    samples[j + 2] = y >> 8
    samples[j + 3] = y & 0xFF
  }
  t += 0.001
}
updateSamples()

let draw = createDraw(canvas, N, samples)

let timer = null;
(function loop () {
  updateSamples()
  draw(samples)
  timer = window.requestAnimationFrame(loop)
})()

// … the application entry module
// As it doesn’t export it can accept itself. A dispose handler can pass the application state on replacement.
if (module.hot) {
  // this module is hot reloadable
  module.hot.accept()

  module.hot.dispose(() => {
    window.cancelAnimationFrame(timer)
    // audio.getContext().close()
  })
}
