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

export default function createAudio (N) {
  let samples = new Float32Array(N)

  let context = new (window.AudioContext || window.webkitAudioContext)()

  let scriptNode = context.createScriptProcessor(N, 1, 1)
  scriptNode.onaudioprocess = (e) => {
    let inputBuffer = e.inputBuffer

    let inputData = inputBuffer.getChannelData(0)

    for (let i = 0; i < inputBuffer.length; i++) {
      samples[i] = inputData[i]
    }
  }

  scriptNode.connect(context.destination)

  getMic(context).then(input => {
    input.connect(scriptNode)
  }).catch((e) => {
    window.alert('audio setup failed')
    console.error(e)
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
