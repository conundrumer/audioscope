import twgl from 'twgl-base.js'

import vs from './line.vert'
import fs from './line.frag'

export default function createDraw (canvas, N, samples) {
  let gl = canvas.getContext('webgl')

  if (gl.getParameter(gl.MAX_VERTEX_TEXTURE_IMAGE_UNITS) === 0) {
    window.alert('sorry, this app wont work on your device. try a different one, or complain to me to make it work on your device')
  }

  let programInfo = twgl.createProgramInfo(gl, [vs, fs])

  let bufferInfo = twgl.createBufferInfoFromArrays(gl, {
    index: {
      numComponents: 1,
      data: Array(4 * N).fill(0).map((_, i) => i)
    }
  })

  gl.useProgram(programInfo.program)
  // do fancy blending after figuring out better lines with no overlapping triangles
  // gl.enable(gl.BLEND)
  // gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA)

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
    twgl.drawBufferInfo(gl, bufferInfo, gl.TRIANGLE_STRIP)
  }
}
