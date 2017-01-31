import twgl from 'twgl-base.js'

import vs from './line.vert'
import fs from './line.frag'

export default function createDraw (canvas, N, samples) {
  let gl = canvas.getContext('webgl')

  let programInfo = twgl.createProgramInfo(gl, [vs, fs])

  let bufferInfo = twgl.createBufferInfoFromArrays(gl, {
    index: {
      numComponents: 1,
      data: Array(N).fill(0).map((_, i) => i)
    }
  })

  gl.useProgram(programInfo.program)

  let texOptions = {
    width: N,
    height: 1,
    mag: gl.NEAREST,
    min: gl.NEAREST,
    wrap: gl.CLAMP_TO_EDGE,
    src: samples
  }
  let tex = twgl.createTexture(gl, texOptions)

  return function draw (samples) {
    twgl.resizeCanvasToDisplaySize(gl.canvas, window.devicePixelRatio)

    gl.clearColor(0, 0, 0, 1)
    gl.clear(gl.COLOR_BUFFER_BIT)

    twgl.setTextureFromArray(gl, tex, samples, texOptions)

    twgl.setBuffersAndAttributes(gl, programInfo, bufferInfo)
    twgl.setUniforms(programInfo, {
      scale: [N, 1],
      samples: tex
    })
    twgl.bindFramebufferInfo(gl)
    gl.viewport(0, 0, gl.canvas.width, gl.canvas.height)
    twgl.drawBufferInfo(gl, bufferInfo, gl.LINE_STRIP)
  }
}
