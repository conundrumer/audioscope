import twgl from 'twgl-base.js'

import vs from './line.vert'
import fs from './line.frag'

const maxAmplitude = 4.0
const B = (1 << 16) - 1
const M = 4
function updateTextureData (textureData, samplesX, samplesY, N) {
  for (let i = 0; i < N; i++) {
    let x = Math.max(0, Math.min(2 * maxAmplitude, 0.5 + 0.5 * samplesX[i] / maxAmplitude))
    let y = Math.max(0, Math.min(2 * maxAmplitude, 0.5 + 0.5 * samplesY[i] / maxAmplitude))

    x = (x * B) | 0
    y = (y * B) | 0

    let j = i * M
    textureData[j + 0] = x >> 8
    textureData[j + 1] = x & 0xFF
    textureData[j + 2] = y >> 8
    textureData[j + 3] = y & 0xFF
  }
}

export default function createDisplay (canvas, N) {
  let gl = canvas.getContext('webgl')

  if (gl.getParameter(gl.MAX_VERTEX_TEXTURE_IMAGE_UNITS) === 0) {
    window.alert('sorry, this app wont work on your device. try a different one, or complain to me to make it work on your device')
  }

  // do fancy blending after figuring out better lines with no overlapping triangles
  // gl.enable(gl.BLEND)
  // gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA)

  let programInfo = twgl.createProgramInfo(gl, [vs, fs])
  gl.useProgram(programInfo.program)

  let bufferInfo = twgl.createBufferInfoFromArrays(gl, {
    index: {
      numComponents: 1,
      data: Array(4 * N).fill(0).map((_, i) => i)
    }
  })

  let textureData = new Uint8Array(N * M)
  let texOptions = {
    width: N,
    height: 1,
    mag: gl.NEAREST,
    min: gl.NEAREST,
    wrap: gl.CLAMP_TO_EDGE,
    src: textureData
  }
  let tex = twgl.createTexture(gl, texOptions)

  return {
    draw (samplesX, samplesY) {
      twgl.resizeCanvasToDisplaySize(gl.canvas, window.devicePixelRatio)

      gl.clearColor(0, 0, 0, 1)
      gl.clear(gl.COLOR_BUFFER_BIT)

      updateTextureData(textureData, samplesX, samplesY, N)
      twgl.setTextureFromArray(gl, tex, textureData, texOptions)

      twgl.setBuffersAndAttributes(gl, programInfo, bufferInfo)
      twgl.setUniforms(programInfo, {
        maxAmplitude,
        window: [gl.canvas.width / window.devicePixelRatio, gl.canvas.height / window.devicePixelRatio],
        sampleScale: [texOptions.width, texOptions.height],
        samples: tex
      })
      twgl.bindFramebufferInfo(gl)
      gl.viewport(0, 0, gl.canvas.width, gl.canvas.height)
      twgl.drawBufferInfo(gl, bufferInfo, gl.TRIANGLE_STRIP)
    }
  }
}
