// http://stackoverflow.com/questions/4723213/detect-http-or-https-then-force-https-in-javascript
if (document.location.hostname !== 'localhost' && window.location.protocol !== 'https:') {
  window.location.href = 'https:' + window.location.href.substring(window.location.protocol.length)
}
let AudioContext = window.AudioContext || window.webkitAudioContext
navigator.getUserMedia = (navigator.getUserMedia || navigator.webkitGetUserMedia || navigator.mozGetUserMedia || navigator.msGetUserMedia)

function getMic (audio) {
  return new Promise((resolve, reject) => {
    const options = {
      video: false,
      audio: {
        optional: [
          {channelCount: 2}, // this doesn't work yet, only takes in summed mono
          {echoCancellation: false},
          {mozAutoGainControl: false},
          {mozNoiseSuppression: false},
          {googEchoCancellation: false},
          {googAutoGainControl: false},
          {googNoiseSuppression: false},
          {googHighpassFilter: false}
        ]
      }
    }
    const onGetUserMedia = (stream) => {
      let input = audio.createMediaStreamSource(stream)
      resolve(input)
    }
    navigator.getUserMedia(options, onGetUserMedia, reject)
  })
}

// const FFT_SIZE = 1024 // guessing webaudio will choose this length
function createHilbertFilter (context, N) {
  let filterLength = 768
  // let filterLength = FFT_SIZE - N
  if (filterLength % 2 === 0) {
    filterLength -= 1
  }
  let impulse = new Float32Array(filterLength)

  let mid = ((filterLength - 1) / 2) | 0

  for (let i = 0; i <= mid; i++) {
    // hamming window
    let k = 0.53836 + 0.46164 * Math.cos(i * Math.PI / (mid + 1))
    if (i % 2 === 1) {
      let im = 2 / Math.PI / i
      impulse[mid + i] = k * im
      impulse[mid - i] = k * -im
    }
  }

  let impulseBuffer = context.createBuffer(2, filterLength, context.sampleRate)
  impulseBuffer.copyToChannel(impulse, 0)
  impulseBuffer.copyToChannel(impulse, 1)
  let hilbert = context.createConvolver()
  hilbert.normalize = false
  hilbert.buffer = impulseBuffer

  let delayTime = mid / context.sampleRate
  let delay = context.createDelay(delayTime)
  delay.delayTime.value = delayTime

  return [delay, hilbert]
}

function createBufferCopy (context, buffer) {
  let copyNode = context.createScriptProcessor(buffer.length, 1, 1)
  copyNode.onaudioprocess = (e) => {
    e.inputBuffer.copyFromChannel(buffer, 0)
  }
  return copyNode
}

// function createMidSide (context, N) {
//   let midSide = context.createScriptProcessor(N, 2, 2)
//   midSide.onaudioprocess = e => {
//     let left = e.inputBuffer.getChannelData(0)
//     let right = e.inputBuffer.getChannelData(1)
//     let mid = e.outputBuffer.getChannelData(0)
//     let side = e.outputBuffer.getChannelData(1)
//     for (let i = 0; i < left.length; i++) {
//       mid[i] = 0.5 * (left[i] + right[i])
//       side[i] = 0.5 * (left[i] - right[i])
//     }
//   }
// }

export default function createAudio (N) {
  let context = new AudioContext()

  let timeSamples = new Float32Array(N)
  let quadSamples = new Float32Array(N)

  let [delay, hilbert] = createHilbertFilter(context, N)
  let time = createBufferCopy(context, timeSamples)
  let quad = createBufferCopy(context, quadSamples)

  getMic(context).then(input => {
    input.connect(delay)
    input.connect(hilbert)
    hilbert.connect(time)
    delay.connect(quad)
    time.connect(context.destination)
    quad.connect(context.destination)
  }).catch(err => {
    window.alert('audio setup failed')
    console.error(err)
  })

  return {
    getContext () {
      return context
    },
    getTimeSamples () {
      return timeSamples
    },
    getQuadSamples () {
      return quadSamples
    }
  }
}
