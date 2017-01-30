import createAudio from './audio'
import createDraw from './draw'

let audio = createAudio()

let canvas = document.getElementById('c')

let draw = createDraw(canvas, audio.getSamples())

let timer = null;
(function loop () {
  draw(audio.getSamples())
  timer = window.requestAnimationFrame(loop)
})()

// … the application entry module
// As it doesn’t export it can accept itself. A dispose handler can pass the application state on replacement.
if (module.hot) {
  // this module is hot reloadable
  module.hot.accept()

  module.hot.dispose(() => {
    window.cancelAnimationFrame(timer)
    audio.getContext().close()
  })
}
