import createAudio from './audio'
import createDisplay from './display'

let canvas = document.getElementById('c')

let N = 512

let audio = createAudio(N)
// const xAxis = Array(N).fill().map((_, i) => (i / (N - 1)) * 2.0 - 1.0)

let display = createDisplay(canvas, N)

let timer = null;
(function loop () {
  let samplesX = audio.getTimeSamples()
  let samplesY = audio.getQuadSamples()

  display.draw(samplesX, samplesY)
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
