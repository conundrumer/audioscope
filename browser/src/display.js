import twgl from 'twgl-base.js'

import lineVert from './line.vert'
import lineFrag from './line.frag'
import copyVert from './copy.vert'
import copyFrag from './copy.frag'
import bufferFrag from './buffer.frag'

const maxAmplitude = 4.0
const B = (1 << 16) - 1
const M = 4
const X = 4
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

function createSwapBuffer (gl, width, height) {
  let textureData = new Uint8Array(width * height * M)

  let textureOptions = {
    width: width,
    height: height,
    mag: gl.NEAREST,
    min: gl.LINEAR,
    wrap: gl.CLAMP_TO_EDGE,
    src: textureData
  }
  let textureFront = twgl.createTexture(gl, textureOptions)
  let textureBack = twgl.createTexture(gl, textureOptions)
  let fbFront = twgl.createFramebufferInfo(gl, [{attachment: textureFront}])
  let fbBack = twgl.createFramebufferInfo(gl, [{attachment: textureBack}])

  return {
    getTexture () {
      return textureFront
    },
    getNextFramebufferInfo () {
      return fbBack
    },
    swap () {
      [textureFront, textureBack] = [textureBack, textureFront];
      [fbFront, fbBack] = [fbBack, fbFront]
    }
  }
}

const K = 1024

export default function createDisplay (gl, N, numBuffers) {
  if (gl.getParameter(gl.MAX_VERTEX_TEXTURE_IMAGE_UNITS) === 0) {
    window.alert('sorry, this app wont work on your device. try a different one, or complain to me to make it work on your device')
  }

  // do fancy blending after figuring out better lines with no overlapping triangles
  gl.enable(gl.BLEND)
  // gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA)

  let lineProgramInfo = twgl.createProgramInfo(gl, [lineVert, lineFrag])
  let bufferProgramInfo = twgl.createProgramInfo(gl, [copyVert, bufferFrag])
  // let copyProgramInfo = twgl.createProgramInfo(gl, [copyVert, copyFrag])

  let indexBufferInfo = twgl.createBufferInfoFromArrays(gl, {
    index: {
      numComponents: 1,
      // data: Array(4 * 3).fill().map((_, i) => i + M * 255)
      data: Array(X * K).fill(0).map((_, i) => i + X).reverse()
    }
  })
  let quadBufferInfo = twgl.createBufferInfoFromArrays(gl, {
    position: { numComponents: 2, data: [1, 1, 1, -1, -1, 1, -1, -1] }
  })

  let textureData = new Uint8Array(N * X)
  let texOptions = {
    width: N,
    height: 1,
    mag: gl.NEAREST,
    min: gl.NEAREST,
    wrap: gl.CLAMP_TO_EDGE,
    src: textureData
  }
  let tex = twgl.createTexture(gl, texOptions)

  let swapBuffers = createSwapBuffer(gl, N, numBuffers)

  function render ({bufferInfo, programInfo, uniforms, viewport, clear = null, framebufferInfo = null, lines = false}) {
    gl.useProgram(programInfo.program)
    twgl.setBuffersAndAttributes(gl, programInfo, bufferInfo)
    twgl.setUniforms(programInfo, uniforms)
    twgl.bindFramebufferInfo(gl, framebufferInfo)
    gl.viewport(...viewport)
    if (clear) {
      gl.clearColor(...clear)
      gl.clear(gl.COLOR_BUFFER_BIT)
    }
    twgl.drawBufferInfo(gl, bufferInfo, lines ? gl.LINE_STRIP : gl.TRIANGLE_STRIP)
  }

  return {
    update (samplesX, samplesY) {
      gl.blendFunc(gl.ONE, gl.ZERO)

      updateTextureData(textureData, samplesX, samplesY, N)
      twgl.setTextureFromArray(gl, tex, textureData, texOptions)
      render({
        bufferInfo: quadBufferInfo,
        programInfo: bufferProgramInfo,
        uniforms: {
          scale: [N, numBuffers],
          state: swapBuffers.getTexture(),
          samples: tex
        },
        framebufferInfo: swapBuffers.getNextFramebufferInfo(),
        viewport: [0, 0, N, numBuffers]
      })
      swapBuffers.swap()
    },
    draw () {
      twgl.resizeCanvasToDisplaySize(gl.canvas, window.devicePixelRatio)

      // debug
      // render({
      //   bufferInfo: quadBufferInfo,
      //   programInfo: copyProgramInfo,
      //   uniforms: {
      //     window: [gl.canvas.width, gl.canvas.height],
      //     sampleBuffer: swapBuffers.getTexture()
      //   },
      //   viewport: [0, 0, gl.canvas.width, gl.canvas.height]
      // })

      gl.blendFunc(gl.SRC_ALPHA, gl.ONE)

      render({
        clear: [0, 0, 0, 1],
        lines: true,
        bufferInfo: indexBufferInfo,
        programInfo: lineProgramInfo,
        uniforms: {
          maxAmplitude,
          window: [gl.canvas.width / window.devicePixelRatio, gl.canvas.height / window.devicePixelRatio],
          sampleScale: [N, numBuffers],
          sampleBuffer: swapBuffers.getTexture()
        },
        viewport: [0, 0, gl.canvas.width, gl.canvas.height]
      })
    }
  }
}
