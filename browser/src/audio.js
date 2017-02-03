import getMic from './getMic'

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

const TIME_RE = 0
const TIME_IM = 1
const NUM_CHANNELS = 2
export default function createAudio (context, N, onAudio) {
  let buffer = {
    time: {
      re: new Float32Array(N),
      im: new Float32Array(N)
    }
  }

  let [delay, hilbert] = createHilbertFilter(context, N)

  let processor = context.createScriptProcessor(N, NUM_CHANNELS, 1)
  processor.onaudioprocess = (e) => {
    e.inputBuffer.copyFromChannel(buffer.time.re, TIME_RE)
    e.inputBuffer.copyFromChannel(buffer.time.im, TIME_IM)
    onAudio(buffer)
  }
  let merger = context.createChannelMerger(2)

  delay.connect(merger, 0, TIME_RE)
  hilbert.connect(merger, 0, TIME_IM)
  merger.connect(processor)
  processor.connect(context.destination)

  getMic().then(stream => {
    let input = context.createMediaStreamSource(stream)
    input.connect(delay)
    input.connect(hilbert)
  }).catch(err => {
    window.alert('audio setup failed')
    console.error(err)
  })

  return {
  }
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
