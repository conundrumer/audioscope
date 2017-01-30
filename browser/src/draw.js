import twgl from 'twgl-base.js'

import vs from './line.vert'
import fs from './line.frag'

export default function createDraw (canvas, samples) {
  let gl = canvas.getContext('webgl')

  let programInfo = twgl.createProgramInfo(gl, [vs, fs])

  let bufferInfo = twgl.createBufferInfoFromArrays(gl, {
    position: {
      numComponents: 2,
      data: samples,
      drawType: gl.DYNAMIC_DRAW
    }
  })

  gl.useProgram(programInfo.program)

  return function draw (samples) {
    twgl.resizeCanvasToDisplaySize(gl.canvas, window.devicePixelRatio)

    gl.clearColor(0, 0, 0, 1)
    gl.clear(gl.COLOR_BUFFER_BIT)

    twgl.setAttribInfoBufferFromArray(gl, bufferInfo.attribs.position, samples)
    twgl.setBuffersAndAttributes(gl, programInfo, bufferInfo)
    // twgl.setUniforms(programInfo, {k: k})
    twgl.bindFramebufferInfo(gl)
    gl.viewport(0, 0, gl.canvas.width, gl.canvas.height)
    twgl.drawBufferInfo(gl, bufferInfo, gl.LINE_STRIP)
  }
}
