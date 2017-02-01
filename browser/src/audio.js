let AudioContext = window.AudioContext || window.webkitAudioContext

function getMic (audio) {
  return new Promise((resolve, reject) => {
    const options = {
      video: false,
      audio: true
    }
    const onGetUserMedia = (stream) =>
      resolve(audio.createMediaStreamSource(stream))
    navigator.getUserMedia(options, onGetUserMedia, reject)
  })
}

function createBufferCopyNode (context, buffer) {
  let copyNode = context.createScriptProcessor(buffer.length, 1, 1)
  copyNode.onaudioprocess = (e) => {
    let inputBuffer = e.inputBuffer

    let inputData = inputBuffer.getChannelData(0)

    for (let i = 0; i < inputBuffer.length; i++) {
      buffer[i] = inputData[i]
    }
  }
  return copyNode
}

export default function createAudio (N) {
  let samples = new Float32Array(N)
  let context = new AudioContext()

  let timeNode = createBufferCopyNode(context, samples)

  timeNode.connect(context.destination)

  getMic(context).then(input => {
    input.connect(timeNode)
  }).catch(err => {
    window.alert('audio setup failed')
    console.error(err)
  })

  return {
    getContext () {
      return context
    },
    getTimeSamples () {
      return samples
    }
  }
}
