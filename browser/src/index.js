import createAudio from './audio'
import createDisplay from './display'

let canvas = document.getElementById('c')
let gl = canvas.getContext('webgl')

let AudioContext = window.AudioContext || window.webkitAudioContext
let audioContext = new AudioContext()

let N = 256
let numBuffers = 32

let display = createDisplay(gl, N, numBuffers)

createAudio(audioContext, N, buffer => {
  display.update(buffer.time.re, buffer.time.im)
})
// const xAxis = Array(N).fill().map((_, i) => (i / (N - 1)) * 2.0 - 1.0)

let timer = null;
(function loop () {
  display.draw()
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
    audioContext.close()
  })
}
